use rand::{rngs::SmallRng, SeedableRng};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use undone_domain::{SkillId, TimeSlot};

use undone_packs::{
    char_creation::{new_game, CharCreationConfig},
    load_packs, PackRegistry,
};
use undone_scene::engine::{EngineEvent, SceneEngine};
use undone_scene::loader::load_scenes;
use undone_scene::scheduler::{load_schedule, validate_entry_scene_references, Scheduler};
use undone_scene::types::SceneDefinition;
use undone_world::World;

/// State available before a character has been created.
/// Holds everything loaded from packs but no world yet.
pub struct PreGameState {
    pub registry: PackRegistry,
    pub scenes: HashMap<String, std::sync::Arc<SceneDefinition>>,
    pub scheduler: Scheduler,
    pub rng: SmallRng,
    /// Set when pack loading fails; checked by app_view to surface the error.
    pub init_error: Option<String>,
}

pub struct LoadedRuntimeContent {
    pub registry: PackRegistry,
    pub scenes: HashMap<String, std::sync::Arc<SceneDefinition>>,
    pub scheduler: Scheduler,
}

pub struct GameState {
    pub world: World,
    pub registry: PackRegistry,
    pub engine: SceneEngine,
    pub scheduler: Scheduler,
    pub rng: SmallRng,
    pub dev_mode: bool,
    /// Set when pack loading fails; checked by app_view to surface the error.
    pub init_error: Option<String>,
    pub opening_scene: Option<String>,
    pub femininity_id: SkillId,
    pub current_scene_time_anchor: Option<SceneTimeAnchor>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneTimeAnchor {
    pub week: u32,
    pub day: u8,
    pub time_slot: TimeSlot,
}

impl SceneTimeAnchor {
    pub fn capture(world: &World) -> Self {
        Self {
            week: world.game_data.week,
            day: world.game_data.day,
            time_slot: world.game_data.time_slot,
        }
    }

    pub fn matches_world(&self, world: &World) -> bool {
        self.week == world.game_data.week
            && self.day == world.game_data.day
            && self.time_slot == world.game_data.time_slot
    }
}

pub struct ResumeGameResult {
    pub events: Vec<EngineEvent>,
    pub started_scene_id: Option<String>,
}

/// Resolve the packs directory. Tries:
/// 1. `<exe_dir>/packs` (distribution layout)
/// 2. `./packs` (cargo run from workspace root)
fn resolve_packs_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join("packs");
            if candidate.is_dir() {
                return candidate;
            }
        }
    }
    PathBuf::from("packs")
}

/// Build a failed `PreGameState` carrying an error message. Logs to stderr.
fn failed_pre(
    registry: PackRegistry,
    scenes: HashMap<String, std::sync::Arc<SceneDefinition>>,
    msg: String,
) -> PreGameState {
    log::error!("[init] {msg}");
    failed_pre_with_rng(registry, scenes, SmallRng::from_entropy(), msg)
}

fn failed_pre_with_rng(
    registry: PackRegistry,
    scenes: HashMap<String, std::sync::Arc<SceneDefinition>>,
    rng: SmallRng,
    msg: String,
) -> PreGameState {
    PreGameState {
        registry,
        scenes,
        scheduler: Scheduler::empty(),
        rng,
        init_error: Some(msg),
    }
}

/// Load all packs and return a `PreGameState` ready for character creation.
/// Does NOT create a world — that happens in `start_game()`.
pub fn init_game() -> PreGameState {
    init_game_from_dir(&resolve_packs_dir())
}

pub fn init_game_from_dir(packs_dir: &Path) -> PreGameState {
    match load_runtime_content(packs_dir) {
        Ok(loaded) => PreGameState {
            registry: loaded.registry,
            scenes: loaded.scenes,
            scheduler: loaded.scheduler,
            rng: SmallRng::from_entropy(),
            init_error: None,
        },
        Err(msg) => failed_pre(PackRegistry::new(), HashMap::new(), msg),
    }
}

#[cfg(test)]
pub(crate) fn test_pre_state_from_dir(packs_dir: &Path) -> PreGameState {
    match load_runtime_content(packs_dir) {
        Ok(loaded) => PreGameState {
            registry: loaded.registry,
            scenes: loaded.scenes,
            scheduler: loaded.scheduler,
            rng: SmallRng::seed_from_u64(7),
            init_error: None,
        },
        Err(msg) => failed_pre_with_rng(
            PackRegistry::new(),
            HashMap::new(),
            SmallRng::seed_from_u64(7),
            msg,
        ),
    }
}

pub fn load_runtime_content(packs_dir: &Path) -> Result<LoadedRuntimeContent, String> {
    let (registry, metas) =
        load_packs(packs_dir).map_err(|e| format!("Failed to load packs: {e}"))?;

    let conflict_errors = registry.validate_trait_conflicts();
    if !conflict_errors.is_empty() {
        return Err(format!(
            "Trait conflict errors:\n{}",
            conflict_errors.join("\n")
        ));
    }

    let mut scenes: HashMap<String, std::sync::Arc<SceneDefinition>> = HashMap::new();
    let mut scene_sources: HashMap<String, String> = HashMap::new();
    for meta in &metas {
        let scene_dir = meta.pack_dir.join(&meta.manifest.content.scenes_dir);
        let pack_scenes = load_scenes(&scene_dir, &registry)
            .map_err(|e| format!("Scene load error in pack '{}': {e}", meta.manifest.pack.id))?;
        extend_scenes_checked(
            &mut scenes,
            &mut scene_sources,
            pack_scenes,
            &meta.manifest.pack.id,
        )
        .map_err(|e| format!("Scene load error in pack '{}': {e}", meta.manifest.pack.id))?;
    }

    undone_scene::loader::validate_cross_references(&scenes)
        .map_err(|e| format!("Scene validation error: {e}"))?;

    let scheduler =
        load_schedule(&metas, &registry).map_err(|e| format!("Schedule load error: {e}"))?;
    scheduler
        .validate_scene_references(&scenes)
        .map_err(|e| format!("Schedule validation error: {e}"))?;
    validate_entry_scene_references(
        &scenes,
        registry.opening_scene(),
        registry.transformation_scene(),
    )
    .map_err(|e| format!("Entry scene validation error: {e}"))?;

    let char_creation_errors =
        crate::char_creation::validate_runtime_contract(&registry, &scheduler);
    if !char_creation_errors.is_empty() {
        return Err(format!(
            "Character creation contract error(s):\n{}",
            char_creation_errors.join("\n")
        ));
    }

    Ok(LoadedRuntimeContent {
        registry,
        scenes,
        scheduler,
    })
}

/// Create a world from character creation config and build the full `GameState`.
pub fn start_game(pre: PreGameState, config: CharCreationConfig, dev_mode: bool) -> GameState {
    start_game_checked(pre, config, dev_mode).unwrap_or_else(|message| panic!("{message}"))
}

pub fn start_game_checked(
    pre: PreGameState,
    config: CharCreationConfig,
    dev_mode: bool,
) -> Result<GameState, String> {
    let startup_errors =
        crate::char_creation::validate_startup_contract(&pre.registry, config.origin);
    if !startup_errors.is_empty() {
        return Err(format!(
            "Character creation contract error(s):\n{}",
            startup_errors.join("\n")
        ));
    }

    let PreGameState {
        mut registry,
        scenes,
        scheduler,
        mut rng,
        init_error,
    } = pre;
    let opening_scene = registry.opening_scene().map(|s| s.to_owned());
    let femininity_id = registry
        .femininity_skill()
        .map_err(|_| {
            "Character creation contract error(s):\ncharacter creation requires skill 'FEMININITY', but it is not registered".to_string()
        })?;
    let world = new_game(config, &mut registry, &mut rng);
    Ok(GameState {
        world,
        registry,
        engine: SceneEngine::new(scenes),
        scheduler,
        rng,
        dev_mode,
        init_error,
        opening_scene,
        femininity_id,
        current_scene_time_anchor: None,
    })
}

pub fn build_throwaway_game_state(
    pre: &mut PreGameState,
    config: CharCreationConfig,
    dev_mode: bool,
) -> Result<GameState, String> {
    let startup_errors =
        crate::char_creation::validate_startup_contract(&pre.registry, config.origin);
    if !startup_errors.is_empty() {
        return Err(format!(
            "Character creation contract error(s):\n{}",
            startup_errors.join("\n")
        ));
    }

    let opening_scene = pre.registry.opening_scene().map(|s| s.to_owned());
    let femininity_id = pre.registry.femininity_skill().map_err(|_| {
        "Character creation contract error(s):\ncharacter creation requires skill 'FEMININITY', but it is not registered".to_string()
    })?;
    let world = new_game(config, &mut pre.registry, &mut pre.rng);
    Ok(GameState {
        world,
        registry: pre.registry.clone(),
        engine: SceneEngine::new(pre.scenes.clone()),
        scheduler: pre.scheduler.clone(),
        rng: SmallRng::from_entropy(),
        dev_mode,
        init_error: pre.init_error.clone(),
        opening_scene,
        femininity_id,
        current_scene_time_anchor: None,
    })
}

fn extend_scenes_checked(
    scenes: &mut HashMap<String, std::sync::Arc<SceneDefinition>>,
    scene_sources: &mut HashMap<String, String>,
    incoming: HashMap<String, std::sync::Arc<SceneDefinition>>,
    source: &str,
) -> Result<(), String> {
    for (scene_id, scene) in incoming {
        if let Some(first_source) = scene_sources.insert(scene_id.clone(), source.to_string()) {
            return Err(format!(
                "duplicate scene id '{scene_id}': '{source}' conflicts with already-loaded '{first_source}'"
            ));
        }
        scenes.insert(scene_id, scene);
    }
    Ok(())
}

/// Build `GameState` from a loaded save world, using already-loaded pack content.
///
/// `opening_scene` is intentionally `None` so resuming from save does not replay
/// the new-game opening scene.
pub fn start_loaded_game(pre: PreGameState, world: World, dev_mode: bool) -> GameState {
    let PreGameState {
        registry,
        scenes,
        scheduler,
        rng,
        init_error,
    } = pre;
    let femininity_id = registry
        .femininity_skill()
        .expect("PackRegistry must include required skill id FEMININITY");
    let engine = SceneEngine::new(scenes);
    GameState {
        world,
        registry,
        engine,
        scheduler,
        rng,
        dev_mode,
        init_error,
        opening_scene: None,
        femininity_id,
        current_scene_time_anchor: None,
    }
}

/// Validate and load a save file into a full `GameState`.
pub fn load_game_state_from_save(
    mut pre: PreGameState,
    save_path: &Path,
    dev_mode: bool,
) -> Result<GameState, String> {
    let loaded_world = undone_save::load_game(save_path, &mut pre.registry)
        .map_err(|e| format!("Load failed: {e}"))?;
    Ok(start_loaded_game(pre, loaded_world, dev_mode))
}

/// Reset transient runtime state, then resume from the current persisted world.
///
/// This is the authoritative resume path for loading a save into an existing
/// `GameState`. It guarantees that stale scene frames and queued events do not
/// survive across the load boundary.
pub fn resume_current_world(gs: &mut GameState) -> ResumeGameResult {
    gs.engine.reset_runtime();
    gs.opening_scene = None;
    gs.current_scene_time_anchor = None;

    let mut started_scene_id = None;
    if let Some(result) = gs.scheduler.pick_next(&gs.world, &gs.registry, &mut gs.rng) {
        if result.once_only {
            gs.world
                .game_data
                .set_flag(format!("ONCE_{}", result.scene_id));
        }
        started_scene_id = Some(result.scene_id.clone());
        gs.current_scene_time_anchor = result
            .consumes_time
            .then(|| SceneTimeAnchor::capture(&gs.world));
        crate::start_scene(&mut gs.engine, &gs.world, &gs.registry, result.scene_id);
    }

    ResumeGameResult {
        events: gs.engine.drain(),
        started_scene_id,
    }
}

pub fn load_world_from_save(gs: &mut GameState, save_path: &Path) -> Result<(), String> {
    let loaded_world = undone_save::load_game(save_path, &mut gs.registry)
        .map_err(|e| format!("Load failed: {e}"))?;
    gs.world = loaded_world;
    gs.opening_scene = None;
    gs.current_scene_time_anchor = None;
    Ok(())
}

/// Load a save into an existing `GameState`, then resume from the persisted world.
pub fn reload_current_game_from_save(
    gs: &mut GameState,
    save_path: &Path,
) -> Result<ResumeGameResult, String> {
    load_world_from_save(gs, save_path)?;
    Ok(resume_current_world(gs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_controller::RuntimeController;
    use crate::runtime_snapshot::snapshot_runtime;
    use crate::{AppSignals, NpcSnapshot};
    use floem::prelude::SignalUpdate;
    use rand::SeedableRng;
    use std::collections::{HashMap, HashSet};
    use std::time::{SystemTime, UNIX_EPOCH};
    use undone_domain::{
        Age, Appearance, BeforeIdentity, BeforeSexuality, BeforeVoice, BreastSize, ButtSize,
        ClitSensitivity, Complexion, EyeColour, HairColour, HairLength, Height, InnerLabiaSize,
        LipShape, MaleFigure, NaturalPubicHair, NippleSensitivity, PcOrigin, PenisSize,
        PlayerFigure, PubicHairStyle, SkinTone, WaistSize, WetnessBaseline,
    };
    use undone_packs::char_creation::CharCreationConfig;
    use undone_scene::engine::{EngineCommand, EngineEvent};
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
        let loaded = load_runtime_content(&packs_dir).unwrap();

        PreGameState {
            registry: loaded.registry,
            scenes: loaded.scenes,
            scheduler: loaded.scheduler,
            rng: SmallRng::seed_from_u64(7),
            init_error: None,
        }
    }

    #[test]
    fn load_runtime_content_returns_validated_registry_scenes_and_scheduler() {
        let packs_dir = packs_dir();
        let loaded = load_runtime_content(&packs_dir).expect("shared loader should succeed");

        assert!(!loaded.scenes.is_empty());
        assert!(loaded.registry.opening_scene().is_some());
    }

    fn workplace_config() -> CharCreationConfig {
        CharCreationConfig {
            name_fem: "Robin".into(),
            name_masc: "Robin".into(),
            age: Age::EarlyTwenties,
            race: "white".into(),
            figure: PlayerFigure::Slim,
            breasts: BreastSize::Handful,
            origin: PcOrigin::CisMaleTransformed,
            before: Some(BeforeIdentity {
                name: "Robin".into(),
                age: Age::MidLateTwenties,
                race: "white".into(),
                sexuality: BeforeSexuality::AttractedToWomen,
                figure: MaleFigure::Average,
                height: Height::Average,
                hair_colour: HairColour::DarkBrown,
                eye_colour: EyeColour::Brown,
                skin_tone: SkinTone::Medium,
                penis_size: PenisSize::Average,
                voice: BeforeVoice::Average,
                traits: HashSet::new(),
            }),
            starting_traits: vec![],
            male_count: 2,
            female_count: 2,
            starting_flags: ["ROUTE_WORKPLACE".to_string()].into(),
            starting_arc_states: HashMap::new(),
            height: Height::Average,
            butt: ButtSize::Round,
            waist: WaistSize::Average,
            lips: LipShape::Average,
            hair_colour: HairColour::DarkBrown,
            hair_length: HairLength::Shoulder,
            eye_colour: EyeColour::Brown,
            skin_tone: SkinTone::Medium,
            complexion: Complexion::Normal,
            appearance: Appearance::Average,
            pubic_hair: PubicHairStyle::Trimmed,
            natural_pubic_hair: NaturalPubicHair::Full,
            nipple_sensitivity: NippleSensitivity::Normal,
            clit_sensitivity: ClitSensitivity::Normal,
            inner_labia: InnerLabiaSize::Average,
            wetness_baseline: WetnessBaseline::Normal,
        }
    }

    fn malformed_pre_state_without_femininity() -> PreGameState {
        let mut pre = test_pre_state();
        pre.registry = PackRegistry::new();
        pre
    }

    fn play_scene_to_finish(gs: &mut GameState) {
        loop {
            let events = gs.engine.drain();
            if events
                .iter()
                .any(|event| matches!(event, EngineEvent::SceneFinished))
            {
                break;
            }

            let actions = events.iter().find_map(|event| {
                if let EngineEvent::ActionsAvailable(actions) = event {
                    Some(actions.clone())
                } else {
                    None
                }
            });

            let actions =
                actions.expect("scene should expose at least one action before finishing");
            let action_id = actions
                .first()
                .expect("scene should expose a selectable action")
                .id
                .clone();
            gs.engine.send(
                EngineCommand::ChooseAction(action_id),
                &mut gs.world,
                &gs.registry,
            );
        }
    }

    fn temp_save_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("undone_{name}_{unique}.json"))
    }

    #[test]
    fn resolve_packs_dir_returns_path_ending_in_packs() {
        let dir = resolve_packs_dir();
        assert_eq!(dir.file_name().unwrap(), "packs");
    }

    #[test]
    fn start_game_reports_missing_femininity_skill_as_init_error() {
        let pre = malformed_pre_state_without_femininity();
        let result = start_game_checked(pre, workplace_config(), false);

        assert!(matches!(result, Err(message) if message.contains("FEMININITY")));
    }

    #[test]
    fn reload_current_game_from_save_resets_runtime_and_resumes_from_persisted_world() {
        let pre = test_pre_state();
        let mut gs = start_game(pre, workplace_config(), false);

        assert_eq!(
            gs.opening_scene.as_deref(),
            Some("base::rain_shelter"),
            "new game should retain the new-game opening scene until first launch"
        );

        let first_pick = gs
            .scheduler
            .pick_next(&gs.world, &gs.registry, &mut gs.rng)
            .expect("workplace route should schedule arrival");
        assert_eq!(first_pick.scene_id, "base::workplace_arrival");
        if first_pick.once_only {
            gs.world
                .game_data
                .set_flag(format!("ONCE_{}", first_pick.scene_id));
        }
        crate::start_scene(&mut gs.engine, &gs.world, &gs.registry, first_pick.scene_id);
        play_scene_to_finish(&mut gs);

        assert_eq!(
            gs.world.game_data.arc_state("base::workplace_opening"),
            Some("arrived"),
            "workplace arrival should advance the persisted arc state"
        );

        let save_path = temp_save_path("resume_runtime_reset");
        undone_save::save_game(&gs.world, &gs.registry, &save_path).unwrap();

        gs.engine.send(
            EngineCommand::StartScene("base::rain_shelter".into()),
            &mut gs.world,
            &gs.registry,
        );

        let resume = reload_current_game_from_save(&mut gs, &save_path).unwrap();

        assert_eq!(
            gs.opening_scene, None,
            "loaded saves must not replay opening scene"
        );
        assert_eq!(
            resume.started_scene_id.as_deref(),
            Some("base::workplace_landlord"),
            "resume should follow persisted arc state, not new-game opening flow"
        );

        let resumed_actions = resume
            .events
            .iter()
            .find_map(|event| {
                if let EngineEvent::ActionsAvailable(actions) = event {
                    Some(
                        actions
                            .iter()
                            .map(|action| action.id.clone())
                            .collect::<Vec<_>>(),
                    )
                } else {
                    None
                }
            })
            .expect("resumed scene should expose actions");

        assert!(
            resumed_actions.iter().any(|id| id == "wait_him_out"),
            "expected workplace_landlord actions after resume, got {:?}",
            resumed_actions
        );
        assert!(
            !resumed_actions
                .iter()
                .any(|id| id == "main" || id == "leave"),
            "stale opening-scene actions leaked through load: {:?}",
            resumed_actions
        );

        std::fs::remove_file(save_path).unwrap();
    }

    #[test]
    fn load_game_state_from_save_uses_shared_initial_controller_progression() {
        let pre = test_pre_state();
        let mut source = start_game(pre, workplace_config(), false);

        let first_pick = source
            .scheduler
            .pick_next(&source.world, &source.registry, &mut source.rng)
            .expect("workplace route should schedule arrival");
        if first_pick.once_only {
            source
                .world
                .game_data
                .set_flag(format!("ONCE_{}", first_pick.scene_id));
        }
        crate::start_scene(
            &mut source.engine,
            &source.world,
            &source.registry,
            first_pick.scene_id,
        );
        play_scene_to_finish(&mut source);

        let save_path = temp_save_path("load_state_through_controller");
        undone_save::save_game(&source.world, &source.registry, &save_path).unwrap();

        let pre = test_pre_state();
        let mut loaded = load_game_state_from_save(pre, &save_path, false).unwrap();
        let signals = AppSignals::new();
        let mut controller = RuntimeController::new(&mut loaded, signals);
        let outcome = controller.continue_flow().unwrap();
        let snapshot = controller.snapshot();

        assert_eq!(controller.gs.opening_scene, None);
        assert_eq!(
            outcome.started_scene_id.as_deref(),
            Some("base::workplace_landlord")
        );
        assert_eq!(
            snapshot.current_scene_id.as_deref(),
            Some("base::workplace_landlord")
        );

        std::fs::remove_file(save_path).unwrap();
    }

    #[test]
    fn resume_current_world_clears_transient_ui_state_after_load_world_from_save() {
        let pre = test_pre_state();
        let mut gs = start_game(pre, workplace_config(), false);

        let first_pick = gs
            .scheduler
            .pick_next(&gs.world, &gs.registry, &mut gs.rng)
            .expect("workplace route should schedule arrival");
        if first_pick.once_only {
            gs.world
                .game_data
                .set_flag(format!("ONCE_{}", first_pick.scene_id));
        }
        crate::start_scene(&mut gs.engine, &gs.world, &gs.registry, first_pick.scene_id);
        play_scene_to_finish(&mut gs);

        let save_path = temp_save_path("resume_snapshot_reset");
        undone_save::save_game(&gs.world, &gs.registry, &save_path).unwrap();

        gs.engine.send(
            EngineCommand::StartScene("base::rain_shelter".into()),
            &mut gs.world,
            &gs.registry,
        );

        let signals = AppSignals::new();
        signals.story.set("stale prose".into());
        signals.actions.set(vec![undone_scene::engine::ActionView {
            id: "stale".into(),
            label: "Stale".into(),
            detail: "stale".into(),
        }]);
        signals.active_npc.set(Some(NpcSnapshot {
            name: "Stale".into(),
            age: "Old".into(),
            personality: "Old".into(),
            relationship: undone_domain::RelationshipStatus::Acquaintance,
            pc_liking: undone_domain::LikingLevel::Like,
            pc_attraction: undone_domain::AttractionLevel::Attracted,
        }));
        signals.awaiting_continue.set(true);

        load_world_from_save(&mut gs, &save_path).unwrap();
        let mut controller = RuntimeController::new(&mut gs, signals);
        let outcome = controller.resume_from_current_world().unwrap();
        let snapshot = snapshot_runtime(signals, &gs);

        assert_eq!(
            outcome.started_scene_id.as_deref(),
            Some("base::workplace_landlord")
        );
        assert_eq!(
            snapshot.current_scene_id.as_deref(),
            Some("base::workplace_landlord")
        );
        assert!(
            snapshot
                .story_paragraphs
                .iter()
                .all(|paragraph| !paragraph.contains("stale prose")),
            "resume snapshot should not keep stale UI story: {:?}",
            snapshot.story_paragraphs
        );
        assert!(
            snapshot
                .visible_actions
                .iter()
                .all(|action| action.id != "stale"),
            "resume snapshot should not keep stale actions: {:?}",
            snapshot.visible_actions
        );
        assert!(!snapshot.awaiting_continue);

        std::fs::remove_file(save_path).unwrap();
    }
}
