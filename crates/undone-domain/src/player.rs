use crate::{
    Age, AlcoholLevel, ArousalLevel, BreastSize, PlayerFigure, Sexuality, SkillId, StuffId, TraitId,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A reference to any NPC (male or female) by their SlotMap key.
/// The World figures out which map to look in via the MaleNpcKey/FemaleNpcKey
/// wrapper types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NpcKey {
    Male(MaleNpcKey),
    Female(FemaleNpcKey),
}

slotmap::new_key_type! {
    pub struct MaleNpcKey;
    pub struct FemaleNpcKey;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillValue {
    pub value: i32,
    pub modifier: i32,
}

impl SkillValue {
    pub fn effective(&self) -> i32 {
        self.value + self.modifier
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PregnancyState {
    pub weeks: u32,
    pub father_key: Option<NpcKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    // Identity — three-name system (Lilith's Throne pattern)
    pub name_fem: String,
    pub name_androg: String,
    pub name_masc: String,
    pub age: Age,
    pub race: String,
    pub figure: PlayerFigure,
    pub breasts: BreastSize,
    pub eye_colour: String,
    pub hair_colour: String,

    // Content-driven (loaded from pack data files, not hardcoded)
    pub traits: HashSet<TraitId>,
    pub skills: HashMap<SkillId, SkillValue>,

    // Economy & wellbeing
    pub money: i32,
    pub stress: i32,
    pub anxiety: i32,
    pub arousal: ArousalLevel,
    pub alcohol: AlcoholLevel,

    // Relationships (keys into World's NPC maps)
    pub partner: Option<NpcKey>,
    pub friends: Vec<NpcKey>,

    // Life state
    pub virgin: bool,
    pub anal_virgin: bool,
    pub lesbian_virgin: bool,
    pub on_pill: bool,
    pub pregnancy: Option<PregnancyState>,

    // Inventory
    pub stuff: HashSet<StuffId>,

    // Per-character scene memory (custom per-scene flags on the player)
    pub custom_flags: HashMap<String, String>,
    pub custom_ints: HashMap<String, i32>,

    // Transformation axis
    pub always_female: bool, // false = male-start PC

    // Before-transformation data
    pub before_age: u32,
    pub before_race: String,
    pub before_sexuality: Sexuality,
}

impl Player {
    /// Returns the currently active display name based on femininity score.
    /// 0–30 → masculine name, 31–69 → androgynous name, 70+ → feminine name.
    /// Pass the resolved FEMININITY skill id so the value is read from the skills map.
    pub fn active_name(&self, femininity_skill: SkillId) -> &str {
        let femininity = self.skill(femininity_skill);
        if femininity >= 70 {
            &self.name_fem
        } else if femininity >= 31 {
            &self.name_androg
        } else {
            &self.name_masc
        }
    }

    pub fn has_trait(&self, id: TraitId) -> bool {
        self.traits.contains(&id)
    }

    pub fn skill(&self, id: SkillId) -> i32 {
        self.skills.get(&id).map(|s| s.effective()).unwrap_or(0)
    }

    pub fn is_drunk(&self) -> bool {
        self.alcohol >= AlcoholLevel::Drunk
    }

    pub fn is_very_drunk(&self) -> bool {
        self.alcohol >= AlcoholLevel::VeryDrunk
    }

    pub fn is_max_drunk(&self) -> bool {
        self.alcohol == AlcoholLevel::MaxDrunk
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lasso::Key;

    fn make_player() -> Player {
        Player {
            name_fem: "Eva".into(),
            name_androg: "Ev".into(),
            name_masc: "Evan".into(),
            before_age: 30,
            before_race: "white".into(),
            before_sexuality: crate::Sexuality::StraightMale,
            age: Age::LateTeen,
            race: "east_asian".into(),
            figure: PlayerFigure::Slim,
            breasts: BreastSize::Large,
            eye_colour: "brown".into(),
            hair_colour: "dark".into(),
            traits: HashSet::new(),
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
            always_female: false,
        }
    }

    #[test]
    fn active_name_picks_correct_variant() {
        // Use a fake SkillId (spur 0) — the test sets the skill value directly in the map.
        let fem_id = SkillId(lasso::Spur::try_from_usize(0).unwrap());

        let mut p = make_player();
        p.name_masc = "Evan".into();
        p.name_androg = "Ev".into();
        p.name_fem = "Eva".into();

        let set_fem = |p: &mut Player, v: i32| {
            p.skills.insert(
                fem_id,
                SkillValue {
                    value: v,
                    modifier: 0,
                },
            );
        };

        set_fem(&mut p, 0);
        assert_eq!(p.active_name(fem_id), "Evan");

        set_fem(&mut p, 30);
        assert_eq!(p.active_name(fem_id), "Evan");

        set_fem(&mut p, 31);
        assert_eq!(p.active_name(fem_id), "Ev");

        set_fem(&mut p, 69);
        assert_eq!(p.active_name(fem_id), "Ev");

        set_fem(&mut p, 70);
        assert_eq!(p.active_name(fem_id), "Eva");

        set_fem(&mut p, 100);
        assert_eq!(p.active_name(fem_id), "Eva");
    }

    #[test]
    fn active_name_reads_from_skills_map() {
        // Verify that active_name reads from the skills map, not a standalone field.
        let fem_id = SkillId(lasso::Spur::try_from_usize(0).unwrap());
        let mut p = make_player();
        p.name_masc = "Evan".into();
        p.name_androg = "Ev".into();
        p.name_fem = "Eva".into();

        // No entry in map → skill() returns 0 → masculine name
        assert_eq!(p.active_name(fem_id), "Evan");

        // Insert value via skills map → name changes accordingly
        p.skills.insert(
            fem_id,
            SkillValue {
                value: 75,
                modifier: 0,
            },
        );
        assert_eq!(p.active_name(fem_id), "Eva");
    }

    #[test]
    fn drunk_checks_respect_ordering() {
        let mut p = make_player();
        assert!(!p.is_drunk());
        p.alcohol = AlcoholLevel::Drunk;
        assert!(p.is_drunk());
        assert!(!p.is_very_drunk());
        p.alcohol = AlcoholLevel::MaxDrunk;
        assert!(p.is_very_drunk());
        assert!(p.is_max_drunk());
    }

    #[test]
    fn skill_effective_adds_modifier() {
        // SkillValue is tested directly — SkillId requires a Rodeo to construct
        let sv = SkillValue {
            value: 50,
            modifier: -10,
        };
        assert_eq!(sv.effective(), 40);
    }
}
