use crate::{
    Age, AlcoholLevel, ArousalLevel, AttractionLevel, Behaviour, BreastSize, CharTypeId,
    LikingLevel, LoveLevel, MaleFigure, NpcTraitId, PersonalityId, PlayerFigure, PregnancyState,
    RelationshipStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcCore {
    pub name: String,
    pub age: Age,
    pub race: String,
    pub eye_colour: String,
    pub hair_colour: String,
    pub personality: PersonalityId,
    pub traits: HashSet<NpcTraitId>,

    // Relationship state
    pub relationship: RelationshipStatus,
    pub pc_liking: LikingLevel,  // PC's liking of NPC
    pub npc_liking: LikingLevel, // NPC's liking of PC
    pub pc_love: LoveLevel,
    pub npc_love: LoveLevel,
    pub pc_attraction: AttractionLevel,
    pub npc_attraction: AttractionLevel,
    pub behaviour: Behaviour,

    // Memory
    pub relationship_flags: HashSet<String>,
    pub sexual_activities: HashSet<String>,
    pub custom_flags: HashMap<String, String>,
    pub custom_ints: HashMap<String, i32>,
    pub knowledge: i32,

    pub contactable: bool,
    pub arousal: ArousalLevel,
    pub alcohol: AlcoholLevel,

    #[serde(default)]
    pub roles: HashSet<String>, // route role assignments e.g. "ROLE_LANDLORD"
}

impl NpcCore {
    pub fn has_trait(&self, id: NpcTraitId) -> bool {
        self.traits.contains(&id)
    }

    pub fn is_partner(&self) -> bool {
        matches!(
            self.relationship,
            RelationshipStatus::Partner { .. } | RelationshipStatus::Married
        )
    }

    pub fn is_friend(&self) -> bool {
        self.relationship == RelationshipStatus::Friend
    }

    pub fn is_cohabiting(&self) -> bool {
        matches!(
            self.relationship,
            RelationshipStatus::Partner { cohabiting: true } | RelationshipStatus::Married
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaleClothing {
    pub trousers_worn: bool,
    pub trousers_open: bool,
    pub shirt_worn: bool,
    pub shirt_open: bool,
    pub jacket_worn: bool,
    pub jacket_open: bool,
    pub has_condom: bool,
    pub wearing_condom: bool,
}

impl Default for MaleClothing {
    fn default() -> Self {
        Self {
            trousers_worn: true,
            trousers_open: false,
            shirt_worn: true,
            shirt_open: false,
            jacket_worn: false,
            jacket_open: false,
            has_condom: false,
            wearing_condom: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaleNpc {
    pub core: NpcCore,
    pub figure: MaleFigure,
    pub clothing: MaleClothing,
    pub had_orgasm: bool,
    pub has_baby_with_pc: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FemaleClothing {
    pub bra_worn: bool,
    pub top_worn: bool,
    pub bottom_worn: bool,
    pub panties_worn: bool,
    pub legwear_worn: bool,
}

impl Default for FemaleClothing {
    fn default() -> Self {
        Self {
            bra_worn: true,
            top_worn: true,
            bottom_worn: true,
            panties_worn: true,
            legwear_worn: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FemaleNpc {
    pub core: NpcCore,
    pub char_type: CharTypeId,
    pub figure: PlayerFigure,
    pub breasts: BreastSize,
    pub clothing: FemaleClothing,
    pub pregnancy: Option<PregnancyState>,
    pub virgin: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_partner_matches_married_and_partner_variants() {
        let r1 = RelationshipStatus::Partner { cohabiting: false };
        let r2 = RelationshipStatus::Married;
        let r3 = RelationshipStatus::Friend;
        assert!(matches!(
            r1,
            RelationshipStatus::Partner { .. } | RelationshipStatus::Married
        ));
        assert!(matches!(
            r2,
            RelationshipStatus::Partner { .. } | RelationshipStatus::Married
        ));
        assert!(!matches!(
            r3,
            RelationshipStatus::Partner { .. } | RelationshipStatus::Married
        ));
    }

    #[test]
    fn male_clothing_default_is_dressed() {
        let c = MaleClothing::default();
        assert!(c.trousers_worn);
        assert!(c.shirt_worn);
        assert!(!c.wearing_condom);
    }
}
