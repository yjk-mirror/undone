use rand::seq::SliceRandom;
use rand::Rng;
use slotmap::SlotMap;
use std::collections::{HashMap, HashSet};
use undone_domain::{
    Age, AlcoholLevel, ArousalLevel, AttractionLevel, Behaviour, BreastSize, CharTypeId,
    FemaleClothing, FemaleNpc, FemaleNpcKey, LikingLevel, LoveLevel, MaleClothing, MaleFigure,
    MaleNpc, MaleNpcKey, NpcCore, NpcTraitId, PersonalityId, PlayerFigure, RelationshipStatus,
};

use crate::PackRegistry;

const MALE_FIGURES: &[MaleFigure] = &[
    MaleFigure::Average,
    MaleFigure::Skinny,
    MaleFigure::Toned,
    MaleFigure::Muscular,
    MaleFigure::Thickset,
    MaleFigure::Paunchy,
];
const FEMALE_FIGURES: &[PlayerFigure] = &[
    PlayerFigure::Slim,
    PlayerFigure::Toned,
    PlayerFigure::Womanly,
];
const BREAST_SIZES: &[BreastSize] = &[
    BreastSize::Small,
    BreastSize::MediumSmall,
    BreastSize::MediumLarge,
    BreastSize::Large,
];
const AGES: &[Age] = &[
    Age::EarlyTwenties,
    Age::Twenties,
    Age::LateTwenties,
    Age::Thirties,
];
const RACES: &[&str] = &["white", "black", "south_asian", "east_asian", "mixed"];
const EYE_COLOURS: &[&str] = &["brown", "blue", "green", "grey", "hazel"];
const HAIR_COLOURS: &[&str] = &["dark", "fair", "auburn", "black", "blonde"];
const CORE_PERSONALITIES: &[&str] = &["ROMANTIC", "JERK", "FRIEND", "INTELLECTUAL", "LAD"];
const REQUIRED_PERSONALITIES: &[&str] = &["ROMANTIC", "JERK", "FRIEND"];

pub struct NpcSpawnConfig {
    pub male_count: usize,
    pub female_count: usize,
}

impl Default for NpcSpawnConfig {
    fn default() -> Self {
        Self {
            male_count: 7,
            female_count: 2,
        }
    }
}

pub fn spawn_npcs<R: Rng>(
    config: &NpcSpawnConfig,
    registry: &mut PackRegistry,
    rng: &mut R,
) -> (
    SlotMap<MaleNpcKey, MaleNpc>,
    SlotMap<FemaleNpcKey, FemaleNpc>,
) {
    let mut males: SlotMap<MaleNpcKey, MaleNpc> = SlotMap::with_key();
    let mut females: SlotMap<FemaleNpcKey, FemaleNpc> = SlotMap::with_key();

    // Build personality assignment list with diversity guarantees.
    // Required slots first (ROMANTIC, JERK, FRIEND), then random fills.
    let mut personality_ids: Vec<PersonalityId> = REQUIRED_PERSONALITIES
        .iter()
        .map(|s| registry.intern_personality(s))
        .collect();
    while personality_ids.len() < config.male_count {
        let p = CORE_PERSONALITIES
            .choose(rng)
            .expect("CORE_PERSONALITIES is non-empty");
        personality_ids.push(registry.intern_personality(p));
    }
    personality_ids.shuffle(rng);

    // Collect NPC trait IDs from registry (snapshot, not borrowed later)
    let npc_trait_ids: Vec<NpcTraitId> = registry.npc_trait_defs.keys().copied().collect();

    // Snapshot name lists before mutable registry calls in the loop
    let male_names = registry.male_names().to_vec();
    let female_names = registry.female_names().to_vec();

    // All female NPCs get char_type FRIEND for now. CharTypeId wraps the same
    // Spur type as PersonalityId — intern via personality gives us the right key.
    let char_type_id = CharTypeId(registry.intern_personality("FRIEND").0);

    for (i, &personality) in personality_ids.iter().enumerate() {
        let name = male_names
            .choose(rng)
            .cloned()
            .unwrap_or_else(|| format!("NPC{}", i));
        let age = *AGES.choose(rng).expect("AGES is non-empty");
        let race = RACES.choose(rng).expect("RACES is non-empty").to_string();
        let eye_colour = EYE_COLOURS
            .choose(rng)
            .expect("EYE_COLOURS is non-empty")
            .to_string();
        let hair_colour = HAIR_COLOURS
            .choose(rng)
            .expect("HAIR_COLOURS is non-empty")
            .to_string();
        let traits = pick_traits(&npc_trait_ids, 2, rng);
        let figure = *MALE_FIGURES.choose(rng).expect("MALE_FIGURES is non-empty");

        let core = make_core(
            name,
            age,
            race,
            eye_colour,
            hair_colour,
            personality,
            traits,
        );
        males.insert(MaleNpc {
            core,
            figure,
            clothing: MaleClothing::default(),
            had_orgasm: false,
            has_baby_with_pc: false,
        });
    }

    for i in 0..config.female_count {
        let name = female_names
            .choose(rng)
            .cloned()
            .unwrap_or_else(|| format!("FNPC{}", i));
        let age = *AGES.choose(rng).expect("AGES is non-empty");
        let race = RACES.choose(rng).expect("RACES is non-empty").to_string();
        let eye_colour = EYE_COLOURS
            .choose(rng)
            .expect("EYE_COLOURS is non-empty")
            .to_string();
        let hair_colour = HAIR_COLOURS
            .choose(rng)
            .expect("HAIR_COLOURS is non-empty")
            .to_string();
        let personality = registry.intern_personality("FRIEND");
        let traits = pick_traits(&npc_trait_ids, 1, rng);
        let figure = *FEMALE_FIGURES
            .choose(rng)
            .expect("FEMALE_FIGURES is non-empty");
        let breasts = *BREAST_SIZES.choose(rng).expect("BREAST_SIZES is non-empty");

        let core = make_core(
            name,
            age,
            race,
            eye_colour,
            hair_colour,
            personality,
            traits,
        );
        females.insert(FemaleNpc {
            core,
            char_type: char_type_id,
            figure,
            breasts,
            clothing: FemaleClothing::default(),
            pregnancy: None,
            virgin: true,
        });
    }

    (males, females)
}

fn pick_traits<R: Rng>(pool: &[NpcTraitId], count: usize, rng: &mut R) -> HashSet<NpcTraitId> {
    pool.choose_multiple(rng, count.min(pool.len()))
        .copied()
        .collect()
}

fn make_core(
    name: String,
    age: Age,
    race: String,
    eye_colour: String,
    hair_colour: String,
    personality: PersonalityId,
    traits: HashSet<NpcTraitId>,
) -> NpcCore {
    NpcCore {
        name,
        age,
        race,
        eye_colour,
        hair_colour,
        personality,
        traits,
        relationship: RelationshipStatus::Stranger,
        pc_liking: LikingLevel::Neutral,
        npc_liking: LikingLevel::Neutral,
        pc_love: LoveLevel::None,
        npc_love: LoveLevel::None,
        pc_attraction: AttractionLevel::Unattracted,
        npc_attraction: AttractionLevel::Unattracted,
        behaviour: Behaviour::Neutral,
        relationship_flags: HashSet::new(),
        sexual_activities: HashSet::new(),
        custom_flags: HashMap::new(),
        custom_ints: HashMap::new(),
        knowledge: 0,
        contactable: false,
        arousal: ArousalLevel::Comfort,
        alcohol: AlcoholLevel::Sober,
        roles: HashSet::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use undone_domain::Personality;

    fn make_registry() -> PackRegistry {
        let mut reg = PackRegistry::new();
        reg.register_npc_traits(vec![
            crate::data::NpcTraitDef {
                id: "CHARMING".into(),
                name: "Charming".into(),
                description: "".into(),
                hidden: false,
            },
            crate::data::NpcTraitDef {
                id: "CRUDE".into(),
                name: "Crude".into(),
                description: "".into(),
                hidden: false,
            },
        ]);
        reg.register_names(
            vec![
                "James".into(),
                "Thomas".into(),
                "William".into(),
                "Oliver".into(),
                "Harry".into(),
                "Charlie".into(),
                "George".into(),
            ],
            vec!["Emma".into(), "Sophie".into()],
        );
        reg
    }

    #[test]
    fn spawn_produces_correct_pool_sizes() {
        let mut reg = make_registry();
        let config = NpcSpawnConfig {
            male_count: 7,
            female_count: 2,
        };
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        let (males, females) = spawn_npcs(&config, &mut reg, &mut rng);
        assert_eq!(males.len(), 7);
        assert_eq!(females.len(), 2);
    }

    #[test]
    fn spawn_guarantees_required_personalities() {
        let mut reg = make_registry();
        let config = NpcSpawnConfig {
            male_count: 7,
            female_count: 0,
        };
        let mut rng = rand::rngs::SmallRng::seed_from_u64(99);
        let (males, _) = spawn_npcs(&config, &mut reg, &mut rng);

        let has_romantic = males
            .values()
            .any(|npc| reg.core_personality(npc.core.personality) == Some(Personality::Romantic));
        let has_jerk = males
            .values()
            .any(|npc| reg.core_personality(npc.core.personality) == Some(Personality::Jerk));
        let has_friend = males
            .values()
            .any(|npc| reg.core_personality(npc.core.personality) == Some(Personality::Friend));
        assert!(has_romantic, "pool must contain a Romantic NPC");
        assert!(has_jerk, "pool must contain a Jerk NPC");
        assert!(has_friend, "pool must contain a Friend NPC");
    }

    #[test]
    fn spawn_is_deterministic_with_seed() {
        let config = NpcSpawnConfig {
            male_count: 5,
            female_count: 2,
        };

        let (names1, names2): (Vec<String>, Vec<String>) = {
            let mut reg1 = make_registry();
            let mut rng1 = rand::rngs::SmallRng::seed_from_u64(7);
            let (males1, _) = spawn_npcs(&config, &mut reg1, &mut rng1);
            let n1: Vec<String> = males1.values().map(|n| n.core.name.clone()).collect();

            let mut reg2 = make_registry();
            let mut rng2 = rand::rngs::SmallRng::seed_from_u64(7);
            let (males2, _) = spawn_npcs(&config, &mut reg2, &mut rng2);
            let n2: Vec<String> = males2.values().map(|n| n.core.name.clone()).collect();

            (n1, n2)
        };
        assert_eq!(names1, names2, "same seed must produce same names");
    }

    #[test]
    fn spawn_min_3_required_personalities_even_with_small_pool() {
        let mut reg = make_registry();
        let config = NpcSpawnConfig {
            male_count: 3,
            female_count: 0,
        };
        let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
        let (males, _) = spawn_npcs(&config, &mut reg, &mut rng);
        assert_eq!(males.len(), 3);
        let has_romantic = males
            .values()
            .any(|n| reg.core_personality(n.core.personality) == Some(Personality::Romantic));
        let has_jerk = males
            .values()
            .any(|n| reg.core_personality(n.core.personality) == Some(Personality::Jerk));
        let has_friend = males
            .values()
            .any(|n| reg.core_personality(n.core.personality) == Some(Personality::Friend));
        assert!(has_romantic && has_jerk && has_friend);
    }
}
