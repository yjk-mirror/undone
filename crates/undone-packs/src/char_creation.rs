use rand::Rng;
use std::collections::{HashMap, HashSet};
use undone_domain::{
    Age, AlcoholLevel, Appearance, ArousalLevel, BeforeIdentity, BreastSize, ButtSize,
    ClitSensitivity, Complexion, EyeColour, HairColour, HairLength, Height, InnerLabiaSize,
    LipShape, NaturalPubicHair, NippleSensitivity, PcOrigin, Player, PlayerFigure, PubicHairStyle,
    SkillValue, SkinTone, TraitId, WaistSize, WetnessBaseline,
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
    pub origin: PcOrigin,
    /// Pre-transformation identity snapshot. Only meaningful when
    /// `origin.has_before_life()` is true; should be `None` for AlwaysFemale.
    pub before: Option<BeforeIdentity>,
    /// Trait IDs (already resolved by the caller from registry)
    pub starting_traits: Vec<TraitId>,
    pub male_count: usize,
    pub female_count: usize,
    /// Game flags set at game start (e.g. "ROUTE_WORKPLACE").
    pub starting_flags: HashSet<String>,
    /// Arc states to initialise at game start. Maps arc_id → initial state name.
    pub starting_arc_states: HashMap<String, String>,

    // Physical attributes
    pub height: Height,
    pub butt: ButtSize,
    pub waist: WaistSize,
    pub lips: LipShape,
    pub hair_colour: HairColour,
    pub hair_length: HairLength,
    pub eye_colour: EyeColour,
    pub skin_tone: SkinTone,
    pub complexion: Complexion,
    pub appearance: Appearance,
    pub pubic_hair: PubicHairStyle,
    pub natural_pubic_hair: NaturalPubicHair,

    // Sexual attributes
    pub nipple_sensitivity: NippleSensitivity,
    pub clit_sensitivity: ClitSensitivity,
    pub inner_labia: InnerLabiaSize,
    pub wetness_baseline: WetnessBaseline,
}

/// Create a brand-new World from character creation choices.
///
/// Builds the Player, spawns the NPC pool, and returns a World ready for week 1.
pub fn new_game<R: Rng>(
    config: CharCreationConfig,
    registry: &mut PackRegistry,
    rng: &mut R,
) -> World {
    let starting_femininity = match config.origin {
        PcOrigin::CisMaleTransformed => 10,
        PcOrigin::TransWomanTransformed => 70,
        PcOrigin::CisFemaleTransformed | PcOrigin::AlwaysFemale => 75,
    };
    let traits: HashSet<TraitId> = config.starting_traits.into_iter().collect();

    let mut player = Player {
        name_fem: config.name_fem,
        name_androg: config.name_androg,
        name_masc: config.name_masc,
        age: config.age,
        race: config.race,
        figure: config.figure,
        breasts: config.breasts,
        eye_colour: config.eye_colour,
        hair_colour: config.hair_colour,
        height: config.height,
        hair_length: config.hair_length,
        skin_tone: config.skin_tone,
        complexion: config.complexion,
        appearance: config.appearance,
        butt: config.butt,
        waist: config.waist,
        lips: config.lips,
        nipple_sensitivity: config.nipple_sensitivity,
        clit_sensitivity: config.clit_sensitivity,
        pubic_hair: config.pubic_hair,
        natural_pubic_hair: config.natural_pubic_hair,
        inner_labia: config.inner_labia,
        wetness_baseline: config.wetness_baseline,
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
        origin: config.origin,
        before: config.before,
    };

    // Seed FEMININITY skill in the skills map.
    let femininity_skill = registry
        .resolve_skill("FEMININITY")
        .expect("FEMININITY skill must be registered by base pack");
    player.skills.insert(
        femininity_skill,
        SkillValue {
            value: starting_femininity,
            modifier: 0,
        },
    );

    // Auto-inject origin-based hidden traits
    match config.origin {
        PcOrigin::TransWomanTransformed => {
            if let Ok(id) = registry.resolve_trait("TRANS_WOMAN") {
                player.traits.insert(id);
            }
        }
        PcOrigin::CisFemaleTransformed => {
            if let Ok(id) = registry.resolve_trait("ALWAYS_FEMALE") {
                player.traits.insert(id);
            }
        }
        PcOrigin::AlwaysFemale => {
            if let Ok(id) = registry.resolve_trait("ALWAYS_FEMALE") {
                player.traits.insert(id);
            }
            if let Ok(id) = registry.resolve_trait("NOT_TRANSFORMED") {
                player.traits.insert(id);
            }
        }
        PcOrigin::CisMaleTransformed => {} // no auto-injected traits
    }

    let spawn_config = NpcSpawnConfig {
        male_count: config.male_count,
        female_count: config.female_count,
    };
    let (male_npcs, female_npcs) = spawn_npcs(&spawn_config, registry, rng);

    let mut game_data = GameData::default();
    for flag in config.starting_flags {
        game_data.set_flag(flag);
    }
    for (arc_id, state) in config.starting_arc_states {
        game_data.advance_arc(arc_id, state);
    }

    World {
        player,
        male_npcs,
        female_npcs,
        game_data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load_packs;
    use rand::SeedableRng;
    use std::path::PathBuf;
    use undone_domain::{
        BeforeSexuality, BeforeVoice, EyeColour, HairColour, Height, MaleFigure, PcOrigin,
        PenisSize, SkinTone,
    };

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
            breasts: BreastSize::Full,
            origin: PcOrigin::CisMaleTransformed,
            before: Some(BeforeIdentity {
                name: "Evan".into(),
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
                traits: std::collections::HashSet::new(),
            }),
            starting_traits: vec![],
            male_count: 7,
            female_count: 2,
            starting_flags: HashSet::new(),
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

    #[test]
    fn new_game_returns_world_with_player() {
        let (mut registry, _) = load_packs(&packs_dir()).unwrap();
        let config = base_config();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
        let world = new_game(config, &mut registry, &mut rng);

        assert_eq!(world.player.name_fem, "Eva");
        assert_eq!(
            world.player.before.as_ref().map(|b| b.age),
            Some(Age::MidLateTwenties)
        );
        assert_eq!(world.player.origin, PcOrigin::CisMaleTransformed);
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
        config.origin = PcOrigin::AlwaysFemale;
        config.before = None;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(4);
        let world = new_game(config, &mut registry, &mut rng);

        let fem_id = registry
            .resolve_skill("FEMININITY")
            .expect("FEMININITY must be registered");
        assert_eq!(world.player.origin, PcOrigin::AlwaysFemale);
        assert!(
            world.player.skill(fem_id) >= 70,
            "always-female PC should start with high femininity"
        );
    }

    #[test]
    fn new_game_trans_woman_sets_femininity_70() {
        let (mut registry, _) = load_packs(&packs_dir()).unwrap();
        let mut config = base_config();
        config.origin = PcOrigin::TransWomanTransformed;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(5);
        let world = new_game(config, &mut registry, &mut rng);

        let fem_id = registry.resolve_skill("FEMININITY").unwrap();
        assert_eq!(world.player.skill(fem_id), 70);
    }

    #[test]
    fn new_game_sets_starting_flags() {
        let (mut registry, _) = load_packs(&packs_dir()).unwrap();
        let mut config = base_config();
        config.starting_flags = ["ROUTE_WORKPLACE".to_string()].into();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        let world = new_game(config, &mut registry, &mut rng);
        assert!(
            world.game_data.has_flag("ROUTE_WORKPLACE"),
            "ROUTE_WORKPLACE flag should be set"
        );
    }

    #[test]
    fn new_game_sets_starting_arc_states() {
        let (mut registry, _) = load_packs(&packs_dir()).unwrap();
        let mut config = base_config();
        config
            .starting_arc_states
            .insert("base::workplace_opening".to_string(), "arrived".to_string());
        let mut rng = rand::rngs::SmallRng::seed_from_u64(43);
        let world = new_game(config, &mut registry, &mut rng);
        assert_eq!(
            world.game_data.arc_state("base::workplace_opening"),
            Some("arrived")
        );
    }
}
