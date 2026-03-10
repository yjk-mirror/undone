use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use floem::action::exec_after;
use floem::prelude::SignalUpdate;
use serde::{Deserialize, Serialize};
use serde_json::json;
use undone_domain::{BoundedStat, SkillValue};

use crate::game_state::GameState;
use crate::{AppSignals, AppTab, PlayerSnapshot};

#[derive(Debug, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum DevCommand {
    JumpToScene { scene_id: String },
    GetState,
    SetStat { stat: String, value: i32 },
    SetFlag { flag: String },
    RemoveFlag { flag: String },
    AdvanceTime { weeks: u32 },
    SetNpcLiking { npc_name: String, level: String },
    SetAllNpcLiking { level: String },
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

pub fn execute_command(
    gs: &mut GameState,
    signals: AppSignals,
    command: DevCommand,
) -> DevCommandResponse {
    let response = match command {
        DevCommand::JumpToScene { scene_id } => jump_to_scene(gs, signals, &scene_id),
        DevCommand::GetState => DevCommandResponse {
            success: true,
            message: "State captured".to_string(),
            data: Some(
                serde_json::to_value(game_state_snapshot(gs))
                    .unwrap_or_else(|_| json!({"error": "failed to serialize state"})),
            ),
        },
        DevCommand::SetStat { stat, value } => set_stat(gs, signals, &stat, value),
        DevCommand::SetFlag { flag } => set_flag(gs, signals, &flag),
        DevCommand::RemoveFlag { flag } => remove_flag(gs, signals, &flag),
        DevCommand::AdvanceTime { weeks } => advance_time(gs, weeks),
        DevCommand::SetNpcLiking { npc_name, level } => set_npc_liking(gs, &npc_name, &level),
        DevCommand::SetAllNpcLiking { level } => set_all_npc_liking(gs, &level),
    };

    if response.success {
        signals.dev_tick.update(|tick| *tick += 1);
    }

    response
}

fn schedule_poll(signals: AppSignals, gs: Rc<RefCell<GameState>>) {
    exec_after(Duration::from_millis(100), move |_| {
        {
            let mut gs = gs.borrow_mut();
            if gs.dev_mode {
                poll_once(&mut gs, signals);
            } else {
                return;
            }
        }
        schedule_poll(signals, Rc::clone(&gs));
    });
}

fn poll_once(gs: &mut GameState, signals: AppSignals) {
    let command_path = command_file_path();
    if !command_path.exists() {
        return;
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
}

fn jump_to_scene(gs: &mut GameState, signals: AppSignals, scene_id: &str) -> DevCommandResponse {
    if !gs.engine.has_scene(scene_id) {
        return DevCommandResponse {
            success: false,
            message: format!("Unknown scene '{scene_id}'"),
            data: None,
        };
    }

    gs.engine.reset_runtime();
    signals.story.set(String::new());
    signals.actions.set(Vec::new());
    signals.active_npc.set(None);
    crate::start_scene(
        &mut gs.engine,
        &mut gs.world,
        &gs.registry,
        scene_id.to_string(),
    );
    let events = gs.engine.drain();
    crate::process_events(events, signals, &gs.world, gs.femininity_id);
    signals.tab.set(AppTab::Game);

    DevCommandResponse {
        success: true,
        message: format!("Jumped to scene '{scene_id}'"),
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
    use rand::{rngs::SmallRng, SeedableRng};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use undone_packs::load_packs;
    use undone_scene::loader::load_scenes;
    use undone_scene::scheduler::{load_schedule, validate_entry_scene_references};
    use undone_scene::types::SceneDefinition;

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("packs")
    }

    fn test_pre_state() -> PreGameState {
        let packs_dir = packs_dir();
        let (registry, metas) = load_packs(&packs_dir).unwrap();

        let mut scenes: HashMap<String, std::sync::Arc<SceneDefinition>> = HashMap::new();
        let mut scene_sources: HashMap<String, String> = HashMap::new();
        for meta in &metas {
            let scene_dir = meta.pack_dir.join(&meta.manifest.content.scenes_dir);
            for (scene_id, scene) in load_scenes(&scene_dir, &registry).unwrap() {
                scene_sources.insert(scene_id.clone(), meta.manifest.pack.id.clone());
                scenes.insert(scene_id, scene);
            }
        }
        undone_scene::loader::validate_cross_references(&scenes).unwrap();

        let scheduler = load_schedule(&metas, &registry).unwrap();
        scheduler.validate_scene_references(&scenes).unwrap();
        validate_entry_scene_references(
            &scenes,
            registry.opening_scene(),
            registry.transformation_scene(),
        )
        .unwrap();

        PreGameState {
            registry,
            scenes,
            scheduler,
            rng: SmallRng::seed_from_u64(7),
            init_error: None,
        }
    }

    fn test_game_state() -> GameState {
        let pre = test_pre_state();
        let config = robin_quick_config(&pre.registry);
        start_game(pre, config, true)
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
