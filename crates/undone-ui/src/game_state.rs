use rand::{rngs::SmallRng, SeedableRng};
use std::collections::HashMap;
use std::path::PathBuf;

use undone_domain::{Age, BreastSize, Player, PlayerFigure, Sexuality};
use undone_packs::{
    char_creation::{new_game, CharCreationConfig},
    load_packs, PackRegistry,
};
use undone_scene::engine::SceneEngine;
use undone_scene::loader::load_scenes;
use undone_scene::scheduler::{load_schedule, Scheduler};
use undone_world::{GameData, World};

pub struct GameState {
    pub world: World,
    pub registry: PackRegistry,
    pub engine: SceneEngine,
    pub scheduler: Scheduler,
    pub rng: SmallRng,
    /// Set when pack loading fails; checked by app_view to surface the error.
    pub init_error: Option<String>,
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

pub fn init_game() -> GameState {
    let packs_dir = resolve_packs_dir();

    // Load all packs from packs/ directory
    let (mut registry, metas) = match load_packs(&packs_dir) {
        Ok(r) => r,
        Err(e) => {
            let msg = format!("Failed to load packs: {e}");
            eprintln!("[init] {msg}");
            return GameState {
                world: World {
                    player: placeholder_player(),
                    male_npcs: slotmap::SlotMap::with_key(),
                    female_npcs: slotmap::SlotMap::with_key(),
                    game_data: GameData::default(),
                },
                registry: PackRegistry::new(),
                engine: SceneEngine::new(HashMap::new()),
                scheduler: Scheduler::empty(),
                rng: SmallRng::from_entropy(),
                init_error: Some(msg),
            };
        }
    };

    // Placeholder character
    let config = CharCreationConfig {
        name_fem: "Eva".into(),
        name_androg: "Ev".into(),
        name_masc: "Evan".into(),
        age: Age::EarlyTwenties,
        race: "white".into(),
        figure: PlayerFigure::Slim,
        breasts: BreastSize::MediumLarge,
        always_female: false,
        before_age: 28,
        before_race: "white".into(),
        before_sexuality: Sexuality::StraightMale,
        starting_traits: vec![],
        male_count: 6,
        female_count: 2,
    };

    let mut rng = SmallRng::from_entropy();
    let world = new_game(config, &mut registry, &mut rng);

    // Load scenes from ALL packs (merge into one map)
    let mut scenes: HashMap<String, _> = HashMap::new();
    for meta in &metas {
        let scene_dir = meta.pack_dir.join(&meta.manifest.content.scenes_dir);
        if let Ok(pack_scenes) = load_scenes(&scene_dir, &registry) {
            scenes.extend(pack_scenes);
        }
    }

    // Build scheduler from all pack metas
    let scheduler = match load_schedule(&metas) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[init] scheduler load error: {e}");
            Scheduler::empty()
        }
    };

    let engine = SceneEngine::new(scenes);
    GameState {
        world,
        registry,
        engine,
        scheduler,
        rng,
        init_error: None,
    }
}

fn placeholder_player() -> Player {
    use std::collections::{HashMap, HashSet};
    use undone_domain::*;

    Player {
        name_fem: "Placeholder".into(),
        name_androg: "Placeholder".into(),
        name_masc: "Placeholder".into(),
        before_age: 18,
        before_race: "white".into(),
        before_sexuality: Sexuality::StraightMale,
        age: Age::LateTeen,
        race: "white".into(),
        figure: PlayerFigure::Slim,
        breasts: BreastSize::Small,
        eye_colour: "brown".into(),
        hair_colour: "brown".into(),
        traits: HashSet::new(),
        skills: HashMap::new(),
        money: 0,
        stress: 0,
        anxiety: 0,
        arousal: ArousalLevel::Comfort,
        alcohol: AlcoholLevel::Sober,
        partner: None,
        friends: vec![],
        virgin: true,
        anal_virgin: true,
        lesbian_virgin: true,
        on_pill: false,
        pregnancy: None,
        stuff: HashSet::new(),
        custom_flags: HashMap::new(),
        custom_ints: HashMap::new(),
        always_female: false,
    }
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
