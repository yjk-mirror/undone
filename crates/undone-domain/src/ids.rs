use lasso::Spur;
use serde::{Deserialize, Serialize};

/// A player trait ID — e.g. "SHY", "POSH"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraitId(Spur);

/// An NPC trait ID — e.g. "CHARMING", "VIRILE"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NpcTraitId(Spur);

/// A player skill ID — e.g. "FITNESS", "CHARM"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkillId(Spur);

/// A personality ID — e.g. "JERK", "CARING"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonalityId(Spur);

/// A character type ID (female NPCs) — e.g. "PARTY_GIRL", "INNOCENT"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CharTypeId(Spur);

/// An inventory item ID — e.g. "CONDOMS", "GYM_MEMBERSHIP"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StuffId(Spur);

/// A named stat ID — e.g. "WEEKS_SINCE_SEX", "ALL_ORGASMS"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StatId(Spur);

// Shared accessor + constructor for all ID newtypes.
macro_rules! impl_id {
    ($ty:ident) => {
        impl $ty {
            /// Wrap a raw `Spur` in this ID type.
            pub fn from_spur(spur: Spur) -> Self {
                Self(spur)
            }

            /// Access the underlying `Spur`.
            pub fn inner(self) -> Spur {
                self.0
            }
        }
    };
}

impl_id!(TraitId);
impl_id!(NpcTraitId);
impl_id!(SkillId);
impl_id!(PersonalityId);
impl_id!(CharTypeId);
impl_id!(StuffId);
impl_id!(StatId);
