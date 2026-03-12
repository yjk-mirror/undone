use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use floem::action::exec_after;
use floem::kurbo::Size;
use floem::prelude::SignalUpdate;
use floem::WindowIdExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use undone_domain::{BoundedStat, SkillValue};

use crate::game_state::GameState;
use crate::runtime_controller::RuntimeController;
use crate::runtime_snapshot::{snapshot_runtime, RuntimeSnapshot};
use crate::{AppSignals, AppTab, PlayerSnapshot};

fn saves_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("undone").join("saves"))
}

#[derive(Debug, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum DevCommand {
    JumpToScene { scene_id: String },
    GetState,
    GetRuntimeState,
    ChooseAction { action_id: String },
    ContinueScene,
    SetTab { tab: String },
    SetStat { stat: String, value: i32 },
    SetFlag { flag: String },
    RemoveFlag { flag: String },
    AdvanceTime { weeks: u32 },
    SetWindowSize { width: f64, height: f64 },
    SetNpcLiking { npc_name: String, level: String },
    SetAllNpcLiking { level: String },
    ListScenes,
    GetSceneInfo { scene_id: String },
    SaveGame { name: String },
    LoadSave { name: String },
    ListSaves,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DevCommandResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct GameStateSnapshot {
    pub current_scene_id: Option<String>,
    pub week: u32,
    pub day: u8,
    pub time_slot: String,
    pub player_name: String,
    pub femininity: i32,
    pub money: i32,
    pub stress: i32,
    pub anxiety: i32,
    pub game_flags: Vec<String>,
    pub arc_states: Vec<(String, String)>,
}

pub fn start_polling(signals: AppSignals, gs: Rc<RefCell<GameState>>) {
    if !gs.borrow().dev_mode {
        return;
    }
    schedule_poll(signals, gs);
}

pub fn command_file_path() -> PathBuf {
    std::env::temp_dir().join("undone-dev-cmd.json")
}

pub fn result_file_path() -> PathBuf {
    std::env::temp_dir().join("undone-dev-result.json")
}

pub fn game_state_snapshot(gs: &GameState) -> GameStateSnapshot {
    let mut flags: Vec<String> = gs.world.game_data.flags.iter().cloned().collect();
    flags.sort();

    let mut arc_states: Vec<(String, String)> = gs
        .world
        .game_data
        .arc_states
        .iter()
        .map(|(arc, state)| (arc.clone(), state.clone()))
        .collect();
    arc_states.sort_by(|a, b| a.0.cmp(&b.0));

    let player = PlayerSnapshot::from_player(&gs.world.player, gs.femininity_id);

    GameStateSnapshot {
        current_scene_id: gs.engine.current_scene_id(),
        week: gs.world.game_data.week,
        day: gs.world.game_data.day,
        time_slot: format!("{:?}", gs.world.game_data.time_slot),
        player_name: player.name,
        femininity: player.femininity,
        money: player.money,
        stress: player.stress,
        anxiety: player.anxiety,
        game_flags: flags,
        arc_states,
    }
}

pub fn runtime_state_snapshot(gs: &GameState, signals: AppSignals) -> RuntimeSnapshot {
    snapshot_runtime(signals, gs)
}

pub fn execute_command(
    gs: &mut GameState,
    signals: AppSignals,
    command: DevCommand,
) -> DevCommandResponse {
    match command {
        DevCommand::JumpToScene { scene_id } => jump_to_scene(gs, signals, &scene_id),
        DevCommand::GetState => DevCommandResponse {
            success: true,
            message: "State captured".to_string(),
            data: Some(
                serde_json::to_value(game_state_snapshot(gs))
                    .unwrap_or_else(|_| json!({"error": "failed to serialize state"})),
            ),
        },
        DevCommand::GetRuntimeState => DevCommandResponse {
            success: true,
            message: "Runtime state captured".to_string(),
            data: Some(
                serde_json::to_value(runtime_state_snapshot(gs, signals))
                    .unwrap_or_else(|_| json!({"error": "failed to serialize runtime state"})),
            ),
        },
        DevCommand::ChooseAction { action_id } => choose_action(gs, signals, &action_id),
        DevCommand::ContinueScene => continue_scene(gs, signals),
        DevCommand::SetTab { tab } => set_tab(gs, signals, &tab),
        DevCommand::SetStat { stat, value } => set_stat(gs, signals, &stat, value),
        DevCommand::SetFlag { flag } => set_flag(gs, signals, &flag),
        DevCommand::RemoveFlag { flag } => remove_flag(gs, signals, &flag),
        DevCommand::AdvanceTime { weeks } => advance_time(gs, weeks),
        DevCommand::SetWindowSize { width, height } => set_window_size(gs, signals, width, height),
        DevCommand::SetNpcLiking { npc_name, level } => set_npc_liking(gs, &npc_name, &level),
        DevCommand::SetAllNpcLiking { level } => set_all_npc_liking(gs, &level),
        DevCommand::ListScenes => list_scenes(gs),
        DevCommand::GetSceneInfo { scene_id } => get_scene_info(gs, &scene_id),
        DevCommand::SaveGame { name } => save_game(gs, &name),
        DevCommand::LoadSave { name } => load_save(gs, signals, &name),
        DevCommand::ListSaves => list_saves(),
    }
}

fn schedule_poll(signals: AppSignals, gs: Rc<RefCell<GameState>>) {
    exec_after(Duration::from_millis(100), move |_| {
        let should_tick = {
            let mut gs = gs.borrow_mut();
            if gs.dev_mode {
                poll_once(&mut gs, signals)
            } else {
                return;
            }
        };
        if should_tick {
            signals.dev_tick.update(|tick| *tick += 1);
        }
        schedule_poll(signals, Rc::clone(&gs));
    });
}

fn poll_once(gs: &mut GameState, signals: AppSignals) -> bool {
    let command_path = command_file_path();
    if !command_path.exists() {
        return false;
    }

    let result_path = result_file_path();
    let response = match std::fs::read_to_string(&command_path) {
        Ok(raw) => {
            let _ = std::fs::remove_file(&command_path);
            match serde_json::from_str::<DevCommand>(&raw) {
                Ok(command) => execute_command(gs, signals, command),
                Err(err) => DevCommandResponse {
                    success: false,
                    message: format!("Invalid dev command: {err}"),
                    data: None,
                },
            }
        }
        Err(err) => {
            let _ = std::fs::remove_file(&command_path);
            DevCommandResponse {
                success: false,
                message: format!("Failed to read dev command: {err}"),
                data: None,
            }
        }
    };

    let payload = serde_json::to_string(&response).unwrap_or_else(|err| {
        format!(
            r#"{{"success":false,"message":"Failed to serialize dev response: {}"}}"#,
            err
        )
    });
    let tmp_path = result_path.with_extension("tmp");
    if std::fs::write(&tmp_path, &payload).is_ok() {
        let _ = std::fs::rename(&tmp_path, &result_path);
    }

    response.success
}

fn jump_to_scene(gs: &mut GameState, signals: AppSignals, scene_id: &str) -> DevCommandResponse {
    let mut controller = RuntimeController::new(gs, signals);
    match controller.jump_to_scene(scene_id) {
        Ok(_) => success_runtime_response(
            format!("Jumped to scene '{scene_id}'"),
            controller.snapshot(),
        ),
        Err(message) => error_response(message),
    }
}

fn choose_action(gs: &mut GameState, signals: AppSignals, action_id: &str) -> DevCommandResponse {
    let mut controller = RuntimeController::new(gs, signals);
    match controller.choose_action(action_id) {
        Ok(_) => {
            success_runtime_response(format!("Chose action '{action_id}'"), controller.snapshot())
        }
        Err(message) => error_response(message),
    }
}

fn continue_scene(gs: &mut GameState, signals: AppSignals) -> DevCommandResponse {
    let mut controller = RuntimeController::new(gs, signals);
    match controller.continue_flow() {
        Ok(_) => success_runtime_response("Continued scene".to_string(), controller.snapshot()),
        Err(message) => error_response(message),
    }
}

fn set_tab(gs: &mut GameState, signals: AppSignals, tab: &str) -> DevCommandResponse {
    let normalized = tab.trim().to_lowercase();
    let target = match normalized.as_str() {
        "game" => AppTab::Game,
        "saves" => AppTab::Saves,
        "settings" => AppTab::Settings,
        "dev" if gs.dev_mode => AppTab::Dev,
        "dev" => {
            return error_response("Dev tab is unavailable when dev mode is disabled".to_string());
        }
        _ => {
            return error_response(format!(
                "Unknown tab '{tab}'. Supported: game, saves, settings, dev"
            ));
        }
    };

    signals.tab.set(target);
    success_runtime_response(
        format!("Switched to tab '{normalized}'"),
        runtime_state_snapshot(gs, signals),
    )
}

fn success_runtime_response(message: String, snapshot: RuntimeSnapshot) -> DevCommandResponse {
    DevCommandResponse {
        success: true,
        message,
        data: Some(
            serde_json::to_value(snapshot)
                .unwrap_or_else(|_| json!({"error": "failed to serialize runtime state"})),
        ),
    }
}

fn error_response(message: String) -> DevCommandResponse {
    DevCommandResponse {
        success: false,
        message,
        data: None,
    }
}

fn set_stat(gs: &mut GameState, signals: AppSignals, stat: &str, value: i32) -> DevCommandResponse {
    let normalized = stat.trim().to_lowercase();
    match normalized.as_str() {
        "money" => {
            gs.world.player.money = value;
        }
        "stress" => {
            gs.world.player.stress = BoundedStat::new(value);
        }
        "anxiety" => {
            gs.world.player.anxiety = BoundedStat::new(value);
        }
        "femininity" => {
            let clamped = gs
                .registry
                .get_skill_def(&gs.femininity_id)
                .map(|def| value.clamp(def.min, def.max))
                .unwrap_or(value);
            gs.world
                .player
                .skills
                .entry(gs.femininity_id)
                .or_insert(SkillValue {
                    value: 0,
                    modifier: 0,
                })
                .value = clamped;
        }
        _ => {
            return DevCommandResponse {
                success: false,
                message: format!(
                    "Unknown stat '{stat}'. Supported: money, stress, anxiety, femininity"
                ),
                data: None,
            };
        }
    }

    signals.player.set(PlayerSnapshot::from_player(
        &gs.world.player,
        gs.femininity_id,
    ));

    DevCommandResponse {
        success: true,
        message: format!("Updated {normalized}"),
        data: None,
    }
}

fn set_flag(gs: &mut GameState, _signals: AppSignals, flag: &str) -> DevCommandResponse {
    let trimmed = flag.trim();
    if trimmed.is_empty() {
        return DevCommandResponse {
            success: false,
            message: "Flag cannot be empty".to_string(),
            data: None,
        };
    }

    gs.world.game_data.set_flag(trimmed.to_string());
    DevCommandResponse {
        success: true,
        message: format!("Set flag '{trimmed}'"),
        data: None,
    }
}

fn remove_flag(gs: &mut GameState, _signals: AppSignals, flag: &str) -> DevCommandResponse {
    let trimmed = flag.trim();
    if trimmed.is_empty() {
        return DevCommandResponse {
            success: false,
            message: "Flag cannot be empty".to_string(),
            data: None,
        };
    }

    gs.world.game_data.remove_flag(trimmed);
    DevCommandResponse {
        success: true,
        message: format!("Removed flag '{trimmed}'"),
        data: None,
    }
}

fn advance_time(gs: &mut GameState, weeks: u32) -> DevCommandResponse {
    let slots = weeks * 28; // 4 slots/day × 7 days/week
    for _ in 0..slots {
        gs.world.game_data.advance_time_slot();
    }
    DevCommandResponse {
        success: true,
        message: format!("Advanced {weeks} week(s)"),
        data: None,
    }
}

fn set_window_size(
    gs: &mut GameState,
    signals: AppSignals,
    width: f64,
    height: f64,
) -> DevCommandResponse {
    if !(width.is_finite() && height.is_finite()) {
        return error_response("Window size must be finite".to_string());
    }
    if width <= 0.0 || height <= 0.0 {
        return error_response("Window size must be positive".to_string());
    }

    signals.window_width.set(width);
    signals.window_height.set(height);
    if let Some(window_id) = signals.window_id {
        window_id.set_content_size(Size::new(width, height));
    }

    success_runtime_response(
        format!("Set window size to {:.0}x{:.0}", width, height),
        runtime_state_snapshot(gs, signals),
    )
}

fn set_all_npc_liking(gs: &mut GameState, level: &str) -> DevCommandResponse {
    use undone_domain::LikingLevel;

    let liking = match level {
        "Neutral" => LikingLevel::Neutral,
        "Ok" => LikingLevel::Ok,
        "Like" => LikingLevel::Like,
        "Close" => LikingLevel::Close,
        other => {
            return DevCommandResponse {
                success: false,
                message: format!(
                    "Unknown liking level '{other}'. Supported: Neutral, Ok, Like, Close"
                ),
                data: None,
            };
        }
    };

    for (_, npc) in gs.world.male_npcs.iter_mut() {
        npc.core.npc_liking = liking;
    }
    for (_, npc) in gs.world.female_npcs.iter_mut() {
        npc.core.npc_liking = liking;
    }

    DevCommandResponse {
        success: true,
        message: format!("Set all NPC liking to {level}"),
        data: None,
    }
}

fn list_scenes(gs: &GameState) -> DevCommandResponse {
    let summaries = gs.engine.all_scene_summaries();
    DevCommandResponse {
        success: true,
        message: format!("{} scenes loaded", summaries.len()),
        data: Some(
            serde_json::to_value(summaries)
                .unwrap_or_else(|_| json!({"error": "failed to serialize scene list"})),
        ),
    }
}

fn get_scene_info(gs: &GameState, scene_id: &str) -> DevCommandResponse {
    match gs.engine.scene_info(scene_id) {
        Some(info) => DevCommandResponse {
            success: true,
            message: format!("Scene '{scene_id}' info"),
            data: Some(
                serde_json::to_value(info)
                    .unwrap_or_else(|_| json!({"error": "failed to serialize scene info"})),
            ),
        },
        None => error_response(format!("Unknown scene '{scene_id}'")),
    }
}

fn save_game(gs: &GameState, name: &str) -> DevCommandResponse {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return error_response("Save name cannot be empty".to_string());
    }

    let dir = match saves_dir() {
        Some(d) => d,
        None => return error_response("Cannot determine saves directory".to_string()),
    };
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return error_response(format!("Cannot create saves directory: {e}"));
    }

    let path = dir.join(format!("{trimmed}.json"));
    match undone_save::save_game(&gs.world, &gs.registry, &path) {
        Ok(()) => DevCommandResponse {
            success: true,
            message: format!("Saved to '{}'", path.display()),
            data: None,
        },
        Err(e) => error_response(format!("Save failed: {e}")),
    }
}

fn load_save(gs: &mut GameState, signals: AppSignals, name: &str) -> DevCommandResponse {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return error_response("Save name cannot be empty".to_string());
    }

    let dir = match saves_dir() {
        Some(d) => d,
        None => return error_response("Cannot determine saves directory".to_string()),
    };

    let path = dir.join(format!("{trimmed}.json"));
    if !path.exists() {
        return error_response(format!("Save file not found: '{}'", path.display()));
    }

    if let Err(e) = crate::game_state::load_world_from_save(gs, &path) {
        return error_response(format!("Load failed: {e}"));
    }

    let mut controller = RuntimeController::new(gs, signals);
    match controller.resume_from_current_world() {
        Ok(_) => {
            success_runtime_response(format!("Loaded save '{trimmed}'"), controller.snapshot())
        }
        Err(e) => error_response(format!("Resume after load failed: {e}")),
    }
}

fn list_saves() -> DevCommandResponse {
    let dir = match saves_dir() {
        Some(d) => d,
        None => {
            return DevCommandResponse {
                success: true,
                message: "No saves directory".to_string(),
                data: Some(json!([])),
            };
        }
    };

    if !dir.exists() {
        return DevCommandResponse {
            success: true,
            message: "No saves yet".to_string(),
            data: Some(json!([])),
        };
    }

    let mut saves: Vec<serde_json::Value> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                let name = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let modified = entry
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                saves.push(json!({
                    "name": name,
                    "modified_epoch": modified,
                }));
            }
        }
    }
    saves.sort_by(|a, b| {
        b["modified_epoch"]
            .as_u64()
            .cmp(&a["modified_epoch"].as_u64())
    });

    DevCommandResponse {
        success: true,
        message: format!("{} save(s) found", saves.len()),
        data: Some(json!(saves)),
    }
}

fn set_npc_liking(gs: &mut GameState, npc_name: &str, level: &str) -> DevCommandResponse {
    use undone_domain::LikingLevel;

    let liking = match level {
        "Neutral" => LikingLevel::Neutral,
        "Ok" => LikingLevel::Ok,
        "Like" => LikingLevel::Like,
        "Close" => LikingLevel::Close,
        other => {
            return DevCommandResponse {
                success: false,
                message: format!(
                    "Unknown liking level '{other}'. Supported: Neutral, Ok, Like, Close"
                ),
                data: None,
            };
        }
    };

    let name_lower = npc_name.trim().to_lowercase();
    let mut found = false;

    for (_, npc) in gs.world.male_npcs.iter_mut() {
        if npc.core.name.to_lowercase() == name_lower {
            npc.core.npc_liking = liking;
            found = true;
            break;
        }
    }
    if !found {
        for (_, npc) in gs.world.female_npcs.iter_mut() {
            if npc.core.name.to_lowercase() == name_lower {
                npc.core.npc_liking = liking;
                found = true;
                break;
            }
        }
    }

    if found {
        DevCommandResponse {
            success: true,
            message: format!("Set {npc_name} liking to {level}"),
            data: None,
        }
    } else {
        DevCommandResponse {
            success: false,
            message: format!("NPC '{npc_name}' not found"),
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::char_creation::robin_quick_config;
    use crate::game_state::{start_game, PreGameState};
    use crate::runtime_controller::RuntimeController;
    use floem::prelude::SignalGet;
    use std::path::PathBuf;

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("packs")
    }

    fn test_pre_state() -> PreGameState {
        crate::game_state::test_pre_state_from_dir(&packs_dir())
    }

    fn test_game_state() -> GameState {
        let pre = test_pre_state();
        let config = robin_quick_config(&pre.registry);
        start_game(pre, config, true)
    }

    fn boot_runtime(gs: &mut GameState, signals: AppSignals) {
        let mut controller = RuntimeController::new(gs, signals);
        controller.continue_flow().unwrap();
    }

    #[test]
    fn execute_jump_to_unknown_scene_returns_error() {
        let mut gs = test_game_state();
        let signals = AppSignals::new();

        let response = execute_command(
            &mut gs,
            signals,
            DevCommand::JumpToScene {
                scene_id: "nonexistent::scene".to_string(),
            },
        );

        assert!(!response.success);
        assert!(response.message.contains("Unknown scene"));
    }

    #[test]
    fn execute_jump_to_scene_resets_transient_scene_ui_state() {
        let mut gs = test_game_state();
        let signals = AppSignals::new();
        signals.story.set("old prose".into());
        signals.actions.set(vec![undone_scene::engine::ActionView {
            id: "stale".into(),
            label: "Stale".into(),
            detail: "stale detail".into(),
        }]);
        signals.awaiting_continue.set(true);
        signals.scroll_gen.set(7);

        let response = execute_command(
            &mut gs,
            signals,
            DevCommand::JumpToScene {
                scene_id: "base::coffee_shop".to_string(),
            },
        );

        assert!(response.success);
        assert!(!signals.awaiting_continue.get());
        assert_eq!(signals.scroll_gen.get(), 0);
        assert_ne!(signals.story.get(), "old prose");
        assert!(
            !signals.actions.get().is_empty(),
            "jumped scene should expose its actions"
        );
    }

    #[test]
    fn execute_set_stat_stress_clamps_to_bounds() {
        let mut gs = test_game_state();
        let signals = AppSignals::new();

        let response = execute_command(
            &mut gs,
            signals,
            DevCommand::SetStat {
                stat: "stress".to_string(),
                value: 999,
            },
        );

        assert!(response.success);
        assert_eq!(gs.world.player.stress.get(), 100);
    }

    #[test]
    fn execute_get_state_returns_valid_json() {
        let mut gs = test_game_state();
        let signals = AppSignals::new();

        let response = execute_command(&mut gs, signals, DevCommand::GetState);

        assert!(response.success);
        let data = response
            .data
            .expect("get_state should include snapshot data");
        assert!(data.get("money").is_some());
        assert!(data.get("stress").is_some());
        assert!(data.get("game_flags").is_some());
    }

    #[test]
    fn execute_get_runtime_state_returns_visible_story_and_actions() {
        let mut gs = test_game_state();
        let signals = AppSignals::new();
        boot_runtime(&mut gs, signals);

        let response = execute_command(&mut gs, signals, DevCommand::GetRuntimeState);

        assert!(response.success);
        let data = response
            .data
            .expect("get_runtime_state should include snapshot data");
        assert!(data.get("story_paragraphs").is_some());
        assert!(data.get("visible_actions").is_some());
        assert!(data.get("window_width").is_some());
        assert!(data.get("window_height").is_some());
    }

    #[test]
    fn execute_advance_time_increments_week() {
        let mut gs = test_game_state();
        let signals = AppSignals::new();
        let week_before = gs.world.game_data.week;

        let response = execute_command(&mut gs, signals, DevCommand::AdvanceTime { weeks: 2 });

        assert!(response.success);
        assert_eq!(gs.world.game_data.week, week_before + 2);
    }

    #[test]
    fn execute_set_npc_liking_unknown_level_returns_error() {
        let mut gs = test_game_state();
        let signals = AppSignals::new();

        let response = execute_command(
            &mut gs,
            signals,
            DevCommand::SetNpcLiking {
                npc_name: "Jake".to_string(),
                level: "BestFriend".to_string(),
            },
        );

        assert!(!response.success);
        assert!(response.message.contains("Unknown liking level"));
    }

    #[test]
    fn execute_choose_action_returns_error_for_invalid_action_id() {
        let mut gs = test_game_state();
        let signals = AppSignals::new();
        boot_runtime(&mut gs, signals);

        let response = execute_command(
            &mut gs,
            signals,
            DevCommand::ChooseAction {
                action_id: "not-visible".to_string(),
            },
        );

        assert!(!response.success);
        assert!(response.message.contains("not currently visible"));
    }

    #[test]
    fn execute_continue_scene_returns_error_when_not_awaiting_continue() {
        let mut gs = test_game_state();
        let signals = AppSignals::new();
        boot_runtime(&mut gs, signals);

        let response = execute_command(&mut gs, signals, DevCommand::ContinueScene);

        assert!(!response.success);
        assert!(response.message.contains("not awaiting continue"));
    }

    #[test]
    fn execute_set_tab_rejects_unknown_tab_names() {
        let mut gs = test_game_state();
        let signals = AppSignals::new();

        let response = execute_command(
            &mut gs,
            signals,
            DevCommand::SetTab {
                tab: "arcade".to_string(),
            },
        );

        assert!(!response.success);
        assert!(response.message.contains("Unknown tab"));
    }

    #[test]
    fn execute_set_window_size_updates_window_metric_signals() {
        let mut gs = test_game_state();
        let signals = AppSignals::new();

        let response = execute_command(
            &mut gs,
            signals,
            DevCommand::SetWindowSize {
                width: 1800.0,
                height: 1000.0,
            },
        );

        assert!(response.success);
        assert_eq!(signals.window_width.get(), 1800.0);
        assert_eq!(signals.window_height.get(), 1000.0);
        assert!(response.message.contains("1800"));
        assert!(response.message.contains("1000"));
    }

    #[test]
    fn acceptance_runtime_dev_commands_expose_runtime_state_and_progression() {
        let mut gs = test_game_state();
        let signals = AppSignals::new();
        boot_runtime(&mut gs, signals);

        let runtime = execute_command(&mut gs, signals, DevCommand::GetRuntimeState);
        assert!(runtime.success);
        let runtime_data = runtime.data.expect("runtime state should include data");
        let first_action = runtime_data["visible_actions"][0]["id"]
            .as_str()
            .expect("visible action should expose stable id")
            .to_string();

        let choose = execute_command(
            &mut gs,
            signals,
            DevCommand::ChooseAction {
                action_id: first_action,
            },
        );
        assert!(choose.success);

        let mut attempts = 0;
        loop {
            let runtime = execute_command(&mut gs, signals, DevCommand::GetRuntimeState);
            let data = runtime.data.expect("runtime state should include data");
            if data["awaiting_continue"].as_bool() == Some(true) {
                break;
            }

            let next_action = data["visible_actions"][0]["id"]
                .as_str()
                .expect("visible action should expose stable id")
                .to_string();
            let choose = execute_command(
                &mut gs,
                signals,
                DevCommand::ChooseAction {
                    action_id: next_action,
                },
            );
            assert!(choose.success);
            attempts += 1;
            assert!(attempts < 8, "expected runtime to reach a continue state");
        }

        let continue_response = execute_command(&mut gs, signals, DevCommand::ContinueScene);
        assert!(continue_response.success);
        let data = continue_response
            .data
            .expect("continue_scene should include updated runtime state");
        assert!(
            data["current_scene_id"].is_string() || data["current_scene_id"].is_null(),
            "continue_scene should return a coherent runtime snapshot"
        );
    }

    #[test]
    fn test_set_all_npc_liking() {
        use undone_domain::LikingLevel;

        let mut gs = test_game_state();
        let signals = AppSignals::new();

        // Confirm precondition: 2+ NPCs spawned by robin_quick_config (6 male + 3 female).
        let total_npcs = gs.world.male_npcs.len() + gs.world.female_npcs.len();
        assert!(
            total_npcs >= 2,
            "expected 2+ NPCs from test_game_state, got {total_npcs}"
        );

        let response = execute_command(
            &mut gs,
            signals,
            DevCommand::SetAllNpcLiking {
                level: "Close".to_string(),
            },
        );

        assert!(response.success);
        for (_, npc) in gs.world.male_npcs.iter() {
            assert_eq!(
                npc.core.npc_liking,
                LikingLevel::Close,
                "male NPC '{}' liking not updated",
                npc.core.name
            );
        }
        for (_, npc) in gs.world.female_npcs.iter() {
            assert_eq!(
                npc.core.npc_liking,
                LikingLevel::Close,
                "female NPC '{}' liking not updated",
                npc.core.name
            );
        }
    }
}
