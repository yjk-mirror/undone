use crate::{Age, AlcoholLevel, ArousalLevel, BreastSize, PlayerFigure, Sexuality, SkillId, StuffId, TraitId};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub femininity: i32,     // 0–100, starts low for male-start

    // Before-transformation data
    pub before_age: u32,
    pub before_race: String,
    pub before_sexuality: Sexuality,
}

impl Player {
    /// Returns the currently active display name based on femininity score.
    /// 0–30 → masculine name, 31–69 → androgynous name, 70+ → feminine name.
    pub fn active_name(&self) -> &str {
        if self.femininity >= 70 {
            &self.name_fem
        } else if self.femininity >= 31 {
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

    fn make_player() -> Player {
        Player {
            name_fem: "Eva".into(),
            name_androg: "Evan".into(),
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
            femininity: 10,
        }
    }

    #[test]
    fn active_name_picks_correct_variant() {
        let mut p = make_player();
        p.name_masc = "Evan".into();
        p.name_androg = "Ev".into();
        p.name_fem = "Eva".into();

        p.femininity = 0;
        assert_eq!(p.active_name(), "Evan");

        p.femininity = 30;
        assert_eq!(p.active_name(), "Evan");

        p.femininity = 31;
        assert_eq!(p.active_name(), "Ev");

        p.femininity = 69;
        assert_eq!(p.active_name(), "Ev");

        p.femininity = 70;
        assert_eq!(p.active_name(), "Eva");

        p.femininity = 100;
        assert_eq!(p.active_name(), "Eva");
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
        let _p = make_player();
        // Test SkillValue directly — can't make SkillId without a Rodeo here
        let sv = SkillValue {
            value: 50,
            modifier: -10,
        };
        assert_eq!(sv.effective(), 40);
    }
}
