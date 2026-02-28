use lasso::Spur;
use serde::{Deserialize, Serialize};

/// A player trait ID — e.g. "SHY", "POSH"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraitId(pub Spur);

/// An NPC trait ID — e.g. "CHARMING", "VIRILE"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NpcTraitId(pub Spur);

/// A player skill ID — e.g. "FITNESS", "CHARM"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkillId(pub Spur);

/// A personality ID — e.g. "JERK", "CARING"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonalityId(pub Spur);

/// A character type ID (female NPCs) — e.g. "PARTY_GIRL", "INNOCENT"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CharTypeId(pub Spur);

/// An inventory item ID — e.g. "CONDOMS", "GYM_MEMBERSHIP"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StuffId(pub Spur);

/// A named stat ID — e.g. "WEEKS_SINCE_SEX", "ALL_ORGASMS"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StatId(pub Spur);

