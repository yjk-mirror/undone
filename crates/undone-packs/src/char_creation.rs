use rand::Rng;
use std::collections::{HashMap, HashSet};
use undone_domain::{
    Age, AlcoholLevel, ArousalLevel, BreastSize, Player, PlayerFigure, Sexuality, TraitId,
};
use undone_world::{GameData, World};

use crate::{
    spawner::{spawn_npcs, NpcSpawnConfig},
    PackRegistry,
};

pub struct CharCreationConfig {
    /// Feminine display name (femininity 70+)
    pub name_fem: String,
    /// Androgynous display name (femininity 31–69)
    pub name_androg: String,
    /// Masculine display name (femininity 0–30)
    pub name_masc: String,
    pub age: Age,
    pub race: String,
    pub figure: PlayerFigure,
    pub breasts: BreastSize,
    /// true = PC was never transformed (always female)
    pub always_female: bool,
    pub before_age: u32,
    pub before_race: String,
    pub before_sexuality: Sexuality,
    /// Trait IDs (already resolved by the caller from registry)
    pub starting_traits: Vec<TraitId>,
    pub male_count: usize,
    pub female_count: usize,
}

/// Create a brand-new World from character creation choices.
///
/// Builds the Player, spawns the NPC pool, and returns a World ready for week 1.
pub fn new_game<R: Rng>(
    config: CharCreationConfig,
    registry: &mut PackRegistry,
    rng: &mut R,
) -> World {
    let starting_femininity = if config.always_female { 75 } else { 10 };

    let traits: HashSet<TraitId> = config.starting_traits.into_iter().collect();

    let player = Player {
        name_fem: config.name_fem,
        name_androg: config.name_androg,
        name_masc: config.name_masc,
        age: config.age,
        race: config.race,
        figure: config.figure,
        breasts: config.breasts,
        eye_colour: "brown".into(),
        hair_colour: "dark".into(),
        traits,
        skills: HashMap::new(),
        money: 500,
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
        always_female: config.always_female,
        femininity: starting_femininity,
        before_age: config.before_age,
        before_race: config.before_race,
        before_sexuality: config.before_sexuality,
    };

    let spawn_config = NpcSpawnConfig {
        male_count: config.male_count,
        female_count: config.female_count,
    };
    let (male_npcs, female_npcs) = spawn_npcs(&spawn_config, registry, rng);

    World {
        player,
        male_npcs,
        female_npcs,
        game_data: GameData::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load_packs;
    use rand::SeedableRng;
    use std::path::PathBuf;
    use undone_domain::{Age, BreastSize, PlayerFigure, Sexuality};

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("packs")
    }

    fn base_config() -> CharCreationConfig {
        CharCreationConfig {
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
            male_count: 7,
            female_count: 2,
        }
    }

    #[test]
    fn new_game_returns_world_with_player() {
        let (mut registry, _) = load_packs(&packs_dir()).unwrap();
        let config = base_config();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
        let world = new_game(config, &mut registry, &mut rng);

        assert_eq!(world.player.name_fem, "Eva");
        assert_eq!(world.player.before_age, 28);
        assert!(!world.player.always_female);
        assert_eq!(world.game_data.week, 0);
    }

    #[test]
    fn new_game_spawns_npc_pool() {
        let (mut registry, _) = load_packs(&packs_dir()).unwrap();
        let config = base_config();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(2);
        let world = new_game(config, &mut registry, &mut rng);

        assert_eq!(world.male_npcs.len(), 7);
        assert_eq!(world.female_npcs.len(), 2);
    }

    #[test]
    fn new_game_applies_starting_traits() {
        let (mut registry, _) = load_packs(&packs_dir()).unwrap();
        let shy = registry.resolve_trait("SHY").unwrap();
        let mut config = base_config();
        config.starting_traits = vec![shy];
        let mut rng = rand::rngs::SmallRng::seed_from_u64(3);
        let world = new_game(config, &mut registry, &mut rng);

        assert!(world.player.has_trait(shy), "player should have SHY trait");
    }

    #[test]
    fn new_game_always_female_sets_high_femininity() {
        let (mut registry, _) = load_packs(&packs_dir()).unwrap();
        let mut config = base_config();
        config.always_female = true;
        config.before_sexuality = Sexuality::AlwaysFemale;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(4);
        let world = new_game(config, &mut registry, &mut rng);

        assert!(
            world.player.femininity >= 70,
            "always-female PC should start with high femininity"
        );
    }
}
