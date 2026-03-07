use rand::{rngs::SmallRng, SeedableRng};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use undone_domain::SkillId;

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

pub struct GameState {
    pub world: World,
    pub registry: PackRegistry,
    pub engine: SceneEngine,
    pub scheduler: Scheduler,
    pub rng: SmallRng,
    /// Set when pack loading fails; checked by app_view to surface the error.
    pub init_error: Option<String>,
    pub opening_scene: Option<String>,
    pub femininity_id: SkillId,
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
    PreGameState {
        registry,
        scenes,
        scheduler: Scheduler::empty(),
        rng: SmallRng::from_entropy(),
        init_error: Some(msg),
    }
}

/// Load all packs and return a `PreGameState` ready for character creation.
/// Does NOT create a world — that happens in `start_game()`.
pub fn init_game() -> PreGameState {
    let packs_dir = resolve_packs_dir();

    let (registry, metas) = match load_packs(&packs_dir) {
        Ok(r) => r,
        Err(e) => {
            return failed_pre(
                PackRegistry::new(),
                HashMap::new(),
                format!("Failed to load packs: {e}"),
            );
        }
    };

    // Validate trait conflict references (dangling conflicts = content error)
    let conflict_errors = registry.validate_trait_conflicts();
    if !conflict_errors.is_empty() {
        return failed_pre(
            registry,
            HashMap::new(),
            format!("Trait conflict errors:\n{}", conflict_errors.join("\n")),
        );
    }

    // Load scenes from all packs into a combined map
    let mut scenes: HashMap<String, std::sync::Arc<SceneDefinition>> = HashMap::new();
    let mut scene_sources: HashMap<String, String> = HashMap::new();
    for meta in &metas {
        let scene_dir = meta.pack_dir.join(&meta.manifest.content.scenes_dir);
        match load_scenes(&scene_dir, &registry) {
            Ok(pack_scenes) => {
                if let Err(e) = extend_scenes_checked(
                    &mut scenes,
                    &mut scene_sources,
                    pack_scenes,
                    &meta.manifest.pack.id,
                ) {
                    return failed_pre(
                        registry,
                        scenes,
                        format!("Scene load error in pack '{}': {e}", meta.manifest.pack.id),
                    );
                }
            }
            Err(e) => {
                return failed_pre(
                    registry,
                    scenes,
                    format!("Scene load error in pack '{}': {e}", meta.manifest.pack.id),
                );
            }
        }
    }

    // Validate cross-references between scenes
    if let Err(e) = undone_scene::loader::validate_cross_references(&scenes) {
        return failed_pre(registry, scenes, format!("Scene validation error: {e}"));
    }

    let scheduler = match load_schedule(&metas, &registry) {
        Ok(s) => s,
        Err(e) => {
            return failed_pre(registry, scenes, format!("Schedule load error: {e}"));
        }
    };

    if let Err(e) = scheduler.validate_scene_references(&scenes) {
        return failed_pre(registry, scenes, format!("Schedule validation error: {e}"));
    }

    if let Err(e) = validate_entry_scene_references(
        &scenes,
        registry.opening_scene(),
        registry.transformation_scene(),
    ) {
        return failed_pre(
            registry,
            scenes,
            format!("Entry scene validation error: {e}"),
        );
    }

    let char_creation_errors = crate::char_creation::validate_registry_contract(&registry);
    if !char_creation_errors.is_empty() {
        return failed_pre(
            registry,
            scenes,
            format!(
                "Character creation contract error(s):\n{}",
                char_creation_errors.join("\n")
            ),
        );
    }

    PreGameState {
        registry,
        scenes,
        scheduler,
        rng: SmallRng::from_entropy(),
        init_error: None,
    }
}

/// Create a world from character creation config and build the full `GameState`.
pub fn start_game(pre: PreGameState, config: CharCreationConfig) -> GameState {
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
        .expect("PackRegistry must include required skill id FEMININITY");
    let world = new_game(config, &mut registry, &mut rng);
    let engine = SceneEngine::new(scenes);
    GameState {
        world,
        registry,
        engine,
        scheduler,
        rng,
        init_error,
        opening_scene,
        femininity_id,
    }
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
pub fn start_loaded_game(pre: PreGameState, world: World) -> GameState {
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
        init_error,
        opening_scene: None,
        femininity_id,
    }
}

/// Validate and load a save file into a full `GameState`.
pub fn load_game_state_from_save(pre: PreGameState, save_path: &Path) -> Result<GameState, String> {
    let loaded_world = undone_save::load_game(save_path, &pre.registry)
        .map_err(|e| format!("Load failed: {e}"))?;
    Ok(start_loaded_game(pre, loaded_world))
}

/// Reset transient runtime state, then resume from the current persisted world.
///
/// This is the authoritative resume path for loading a save into an existing
/// `GameState`. It guarantees that stale scene frames and queued events do not
/// survive across the load boundary.
pub fn resume_current_world(gs: &mut GameState) -> ResumeGameResult {
    gs.engine.reset_runtime();
    gs.opening_scene = None;

    let mut started_scene_id = None;
    if let Some(result) = gs.scheduler.pick_next(&gs.world, &gs.registry, &mut gs.rng) {
        if result.once_only {
            gs.world
                .game_data
                .set_flag(format!("ONCE_{}", result.scene_id));
        }
        started_scene_id = Some(result.scene_id.clone());
        crate::start_scene(&mut gs.engine, &mut gs.world, &gs.registry, result.scene_id);
    }

    ResumeGameResult {
        events: gs.engine.drain(),
        started_scene_id,
    }
}

/// Load a save into an existing `GameState`, then resume from the persisted world.
pub fn reload_current_game_from_save(
    gs: &mut GameState,
    save_path: &Path,
) -> Result<ResumeGameResult, String> {
    let loaded_world =
        undone_save::load_game(save_path, &gs.registry).map_err(|e| format!("Load failed: {e}"))?;
    gs.world = loaded_world;
    Ok(resume_current_world(gs))
}

#[cfg(test)]
mod tests {
    use super::*;
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
    use undone_scene::scheduler::{load_schedule, validate_entry_scene_references};

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
            extend_scenes_checked(
                &mut scenes,
                &mut scene_sources,
                load_scenes(&scene_dir, &registry).unwrap(),
                &meta.manifest.pack.id,
            )
            .unwrap();
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

        let char_creation_errors = crate::char_creation::validate_registry_contract(&registry);
        assert!(char_creation_errors.is_empty());

        PreGameState {
            registry,
            scenes,
            scheduler,
            rng: SmallRng::seed_from_u64(7),
            init_error: None,
        }
    }

    fn workplace_config() -> CharCreationConfig {
        CharCreationConfig {
            name_fem: "Robin".into(),
            name_androg: "Robin".into(),
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
    fn reload_current_game_from_save_resets_runtime_and_resumes_from_persisted_world() {
        let pre = test_pre_state();
        let mut gs = start_game(pre, workplace_config());

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
        crate::start_scene(
            &mut gs.engine,
            &mut gs.world,
            &gs.registry,
            first_pick.scene_id,
        );
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
}
