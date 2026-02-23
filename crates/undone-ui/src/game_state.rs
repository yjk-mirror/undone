use std::collections::HashMap;
use std::path::Path;
use rand::{rngs::SmallRng, SeedableRng};

use undone_domain::{Age, BreastSize, PlayerFigure, Sexuality, Player};
use undone_world::{World, GameData};
use undone_packs::{PackRegistry, load_packs, char_creation::{new_game, CharCreationConfig}};
use undone_scene::engine::SceneEngine;
use undone_scene::loader::load_scenes;

pub struct GameState {
    pub world: World,
    pub registry: PackRegistry,
    pub engine: SceneEngine,
}

pub fn init_game() -> GameState {
    let packs_dir = Path::new("packs");

    // Load all packs from packs/ directory
    let (mut registry, metas) = match load_packs(packs_dir) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[init] pack load error: {e}");
            return GameState {
                world: World {
                    player: placeholder_player(),
                    male_npcs: slotmap::SlotMap::with_key(),
                    female_npcs: slotmap::SlotMap::with_key(),
                    game_data: GameData::default(),
                },
                registry: PackRegistry::new(),
                engine: SceneEngine::new(HashMap::new()),
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

    // Load scenes from the base pack
    let scenes = metas.iter()
        .find(|m| m.manifest.pack.id == "base")
        .and_then(|m| {
            let scene_dir = m.pack_dir.join(&m.manifest.content.scenes_dir);
            load_scenes(&scene_dir, &registry).ok()
        })
        .unwrap_or_default();

    let engine = SceneEngine::new(scenes);
    GameState { world, registry, engine }
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
        femininity: 0,
    }
}
