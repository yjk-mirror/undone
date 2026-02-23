use std::collections::HashMap;

use lasso::{Key, Rodeo, Spur};
use thiserror::Error;
use undone_domain::{NpcTraitId, PersonalityId, SkillId, StatId, StuffId, TraitId};

use crate::data::{NpcTraitDef, SkillDef, StatDef, TraitDef};

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("unknown trait id: {0}")]
    UnknownTrait(String),
    #[error("unknown npc trait id: {0}")]
    UnknownNpcTrait(String),
    #[error("unknown skill id: {0}")]
    UnknownSkill(String),
}

/// Central registry for all content-level IDs across all loaded packs.
/// Owns the string interner — all TraitId/SkillId/etc. are valid only
/// within the context of the registry that created them.
pub struct PackRegistry {
    rodeo: Rodeo,
    pub trait_defs: HashMap<TraitId, TraitDef>,
    pub npc_trait_defs: HashMap<NpcTraitId, NpcTraitDef>,
    pub skill_defs: HashMap<SkillId, SkillDef>,
    male_names: Vec<String>,
    female_names: Vec<String>,
}

impl PackRegistry {
    pub fn new() -> Self {
        Self {
            rodeo: Rodeo::new(),
            trait_defs: HashMap::new(),
            npc_trait_defs: HashMap::new(),
            skill_defs: HashMap::new(),
            male_names: Vec::new(),
            female_names: Vec::new(),
        }
    }

    /// Intern a string and return a raw Spur. Used internally.
    fn intern(&mut self, s: &str) -> Spur {
        self.rodeo.get_or_intern(s)
    }

    /// Register player traits from a pack data file.
    pub fn register_traits(&mut self, defs: Vec<TraitDef>) {
        for def in defs {
            let spur = self.intern(&def.id);
            self.trait_defs.insert(TraitId(spur), def);
        }
    }

    /// Register NPC traits from a pack data file.
    pub fn register_npc_traits(&mut self, defs: Vec<NpcTraitDef>) {
        for def in defs {
            let spur = self.intern(&def.id);
            self.npc_trait_defs.insert(NpcTraitId(spur), def);
        }
    }

    /// Register skills from a pack data file.
    pub fn register_skills(&mut self, defs: Vec<SkillDef>) {
        for def in defs {
            let spur = self.intern(&def.id);
            self.skill_defs.insert(SkillId(spur), def);
        }
    }

    /// Register stats from a pack data file, interning each stat id at load time.
    pub fn register_stats(&mut self, defs: Vec<StatDef>) {
        for def in defs {
            self.intern_stat(&def.id);
        }
    }

    /// Resolve a string to a TraitId. Errors if the id is unknown.
    /// Call this at scene load time to validate condition expressions.
    pub fn resolve_trait(&self, id: &str) -> Result<TraitId, RegistryError> {
        self.rodeo
            .get(id)
            .and_then(|s| {
                let tid = TraitId(s);
                self.trait_defs.contains_key(&tid).then_some(tid)
            })
            .ok_or_else(|| RegistryError::UnknownTrait(id.to_string()))
    }

    /// Resolve a string to an NpcTraitId.
    pub fn resolve_npc_trait(&self, id: &str) -> Result<NpcTraitId, RegistryError> {
        self.rodeo
            .get(id)
            .and_then(|s| {
                let tid = NpcTraitId(s);
                self.npc_trait_defs.contains_key(&tid).then_some(tid)
            })
            .ok_or_else(|| RegistryError::UnknownNpcTrait(id.to_string()))
    }

    /// Resolve a string to a SkillId.
    pub fn resolve_skill(&self, id: &str) -> Result<SkillId, RegistryError> {
        self.rodeo
            .get(id)
            .and_then(|s| {
                let sid = SkillId(s);
                self.skill_defs.contains_key(&sid).then_some(sid)
            })
            .ok_or_else(|| RegistryError::UnknownSkill(id.to_string()))
    }

    /// Intern a stat name (stat names don't need definitions, just interning).
    pub fn intern_stat(&mut self, id: &str) -> StatId {
        StatId(self.intern(id))
    }

    /// Look up an already-interned stat name without mutating. Returns None if never interned.
    pub fn get_stat(&self, id: &str) -> Option<StatId> {
        self.rodeo.get(id).map(StatId)
    }

    /// Resolve a TraitId back to its string ID (spur → str). Used for template rendering.
    pub fn trait_id_to_str(&self, id: TraitId) -> &str {
        self.rodeo.resolve(&id.0)
    }

    /// Resolve any Spur back to its string. Used by the save system to build the id_strings
    /// validation table.
    pub fn resolve_spur(&self, spur: Spur) -> &str {
        self.rodeo.resolve(&spur)
    }

    /// Intern a stuff/item name, returning a StuffId.
    pub fn intern_stuff(&mut self, id: &str) -> StuffId {
        StuffId(self.intern(id))
    }

    /// Look up an already-interned stuff name. Returns None if never interned.
    pub fn resolve_stuff(&self, id: &str) -> Option<StuffId> {
        self.rodeo.get(id).map(StuffId)
    }

    /// Intern a personality name, returning a PersonalityId.
    /// Personalities don't require registered definitions — any string is valid.
    pub fn intern_personality(&mut self, id: &str) -> PersonalityId {
        PersonalityId(self.intern(id))
    }

    /// Resolve a PersonalityId to the engine Personality enum.
    /// Returns None for custom/unknown personalities.
    pub fn core_personality(&self, id: PersonalityId) -> Option<undone_domain::Personality> {
        use undone_domain::Personality;
        match self.rodeo.resolve(&id.0) {
            "ROMANTIC" => Some(Personality::Romantic),
            "JERK" => Some(Personality::Jerk),
            "FRIEND" => Some(Personality::Friend),
            "INTELLECTUAL" => Some(Personality::Intellectual),
            "LAD" => Some(Personality::Lad),
            _ => None,
        }
    }

    /// Store male and female NPC name lists from a pack's names file.
    pub fn register_names(&mut self, male: Vec<String>, female: Vec<String>) {
        self.male_names.extend(male);
        self.female_names.extend(female);
    }

    pub fn male_names(&self) -> &[String] {
        &self.male_names
    }

    pub fn female_names(&self) -> &[String] {
        &self.female_names
    }

    /// Return all interned strings in Spur-index order (index 0 first).
    /// The save system records these so it can detect if the pack load order changed
    /// between saving and loading.
    pub fn all_interned_strings(&self) -> Vec<String> {
        (0..self.rodeo.len())
            .map(|i| {
                let spur = Spur::try_from_usize(i).expect("valid spur index");
                self.rodeo.resolve(&spur).to_string()
            })
            .collect()
    }
}

impl Default for PackRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::TraitDef;

    fn make_registry() -> PackRegistry {
        let mut reg = PackRegistry::new();
        reg.register_traits(vec![
            TraitDef {
                id: "SHY".into(),
                name: "Shy".into(),
                description: "...".into(),
                hidden: false,
            },
            TraitDef {
                id: "POSH".into(),
                name: "Posh".into(),
                description: "...".into(),
                hidden: false,
            },
        ]);
        reg
    }

    #[test]
    fn resolves_known_trait() {
        let reg = make_registry();
        assert!(reg.resolve_trait("SHY").is_ok());
        assert!(reg.resolve_trait("POSH").is_ok());
    }

    #[test]
    fn errors_on_unknown_trait() {
        let reg = make_registry();
        assert!(reg.resolve_trait("TYPO").is_err());
    }

    #[test]
    fn same_id_string_resolves_to_same_spur() {
        let reg = make_registry();
        let a = reg.resolve_trait("SHY").unwrap();
        let b = reg.resolve_trait("SHY").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn intern_and_resolve_personality() {
        let mut reg = PackRegistry::new();
        let id = reg.intern_personality("ROMANTIC");
        assert_eq!(
            reg.core_personality(id),
            Some(undone_domain::Personality::Romantic)
        );
    }

    #[test]
    fn unknown_personality_returns_none() {
        let mut reg = PackRegistry::new();
        let id = reg.intern_personality("CUSTOM_PACK_PERSONALITY");
        assert_eq!(reg.core_personality(id), None);
    }

    #[test]
    fn register_names_accumulates() {
        let mut reg = PackRegistry::new();
        reg.register_names(vec!["James".into(), "Thomas".into()], vec!["Emma".into()]);
        assert_eq!(reg.male_names(), &["James", "Thomas"]);
        assert_eq!(reg.female_names(), &["Emma"]);
    }
}
