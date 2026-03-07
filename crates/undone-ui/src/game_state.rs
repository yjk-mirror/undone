use rand::{rngs::SmallRng, SeedableRng};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use undone_domain::SkillId;

use undone_packs::{
    char_creation::{new_game, CharCreationConfig},
    load_packs, PackRegistry,
};
use undone_scene::engine::SceneEngine;
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
    for meta in &metas {
        let scene_dir = meta.pack_dir.join(&meta.manifest.content.scenes_dir);
        match load_scenes(&scene_dir, &registry) {
            Ok(pack_scenes) => scenes.extend(pack_scenes),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_packs_dir_returns_path_ending_in_packs() {
        let dir = resolve_packs_dir();
        assert_eq!(dir.file_name().unwrap(), "packs");
    }
}
