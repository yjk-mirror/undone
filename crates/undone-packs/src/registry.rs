use std::collections::HashMap;

use lasso::{Rodeo, Spur};
use thiserror::Error;
use undone_domain::{NpcTraitId, SkillId, StatId, TraitId};

use crate::data::{NpcTraitDef, SkillDef, TraitDef};

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
}

impl PackRegistry {
    pub fn new() -> Self {
        Self {
            rodeo: Rodeo::new(),
            trait_defs: HashMap::new(),
            npc_trait_defs: HashMap::new(),
            skill_defs: HashMap::new(),
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
    pub fn get_stat(&self, id: &str) -> Option<undone_domain::StatId> {
        self.rodeo.get(id).map(undone_domain::StatId)
    }

    /// Resolve a TraitId back to its string ID (spur → str). Used for template rendering.
    pub fn trait_id_to_str(&self, id: undone_domain::TraitId) -> &str {
        self.rodeo.resolve(&id.0)
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
}
