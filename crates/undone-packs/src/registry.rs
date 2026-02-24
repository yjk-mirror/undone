use std::collections::{HashMap, HashSet};

use lasso::{Key, Rodeo, Spur};
use thiserror::Error;
use undone_domain::{NpcTraitId, PersonalityId, SkillId, StatId, StuffId, TraitId};

use crate::data::{ArcDef, CategoryDef, NpcTraitDef, SkillDef, StatDef, TraitDef};

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
    races: Vec<String>,
    categories: HashMap<String, CategoryDef>,
    arcs: HashMap<String, ArcDef>,
    opening_scene: Option<String>,
    default_slot: Option<String>,
    transformation_scene: Option<String>,
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
            races: Vec::new(),
            categories: HashMap::new(),
            arcs: HashMap::new(),
            opening_scene: None,
            default_slot: None,
            transformation_scene: None,
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

    /// Resolve a SkillId back to its string ID (spur → str). Used for template rendering.
    pub fn skill_id_to_str(&self, id: SkillId) -> &str {
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

    /// Resolve a PersonalityId back to its string name.
    pub fn personality_name(&self, id: PersonalityId) -> &str {
        self.rodeo.resolve(&id.0)
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

    pub fn register_races(&mut self, races: Vec<String>) {
        self.races.extend(races);
    }

    pub fn races(&self) -> &[String] {
        &self.races
    }

    /// Register category definitions from a pack data file.
    pub fn register_categories(&mut self, defs: Vec<CategoryDef>) {
        for def in defs {
            self.categories.insert(def.id.clone(), def);
        }
    }

    /// Check if a value is a member of a category.
    pub fn in_category(&self, category_id: &str, value: &str) -> bool {
        self.categories
            .get(category_id)
            .map(|cat| cat.members.iter().any(|m| m == value))
            .unwrap_or(false)
    }

    /// Get a category definition by ID.
    pub fn get_category(&self, id: &str) -> Option<&CategoryDef> {
        self.categories.get(id)
    }

    /// Register arc definitions from a pack data file.
    pub fn register_arcs(&mut self, arcs: Vec<ArcDef>) {
        for arc in arcs {
            self.arcs.insert(arc.id.clone(), arc);
        }
    }

    /// Look up an arc definition by ID.
    pub fn get_arc(&self, id: &str) -> Option<&ArcDef> {
        self.arcs.get(id)
    }

    /// Set the opening scene ID for the first pack that declares one.
    /// Subsequent packs cannot override it (first-writer wins).
    pub fn set_opening_scene(&mut self, id: String) {
        if self.opening_scene.is_none() {
            self.opening_scene = Some(id);
        }
    }

    /// Set the default scheduler slot for the first pack that declares one.
    /// Subsequent packs cannot override it (first-writer wins).
    pub fn set_default_slot(&mut self, slot: String) {
        if self.default_slot.is_none() {
            self.default_slot = Some(slot);
        }
    }

    /// Return the opening scene ID declared by the pack, if any.
    pub fn opening_scene(&self) -> Option<&str> {
        self.opening_scene.as_deref()
    }

    /// Return the default scheduler slot declared by the pack, if any.
    pub fn default_slot(&self) -> Option<&str> {
        self.default_slot.as_deref()
    }

    /// Set the transformation scene ID for the first pack that declares one.
    /// Subsequent packs cannot override it (first-writer wins).
    pub fn set_transformation_scene(&mut self, id: String) {
        if self.transformation_scene.is_none() {
            self.transformation_scene = Some(id);
        }
    }

    /// Return the transformation scene ID declared by the pack, if any.
    pub fn transformation_scene(&self) -> Option<&str> {
        self.transformation_scene.as_deref()
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

    /// Validate that all `conflicts` entries in every registered trait reference
    /// known trait IDs. Returns a list of error messages for unknown references.
    /// Call this after all packs have been loaded via `register_traits`.
    pub fn validate_trait_conflicts(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for (tid, def) in &self.trait_defs {
            let owner = self.rodeo.resolve(&tid.0);
            for conflict_id in &def.conflicts {
                if self.resolve_trait(conflict_id).is_err() {
                    errors.push(format!(
                        "trait '{}': conflicts entry '{}' is not a known trait id",
                        owner, conflict_id
                    ));
                }
            }
        }
        errors
    }

    /// Return the slice of conflict trait ID strings declared by a given trait.
    /// Returns an empty slice if the trait is unknown or has no conflicts.
    pub fn trait_conflicts(&self, id: TraitId) -> &[String] {
        self.trait_defs
            .get(&id)
            .map(|def| def.conflicts.as_slice())
            .unwrap_or(&[])
    }

    /// Check whether adding `new_trait` to `existing` would violate any declared
    /// conflict. Returns `Some(message)` if there is a conflict, `None` if safe.
    pub fn check_trait_conflict(
        &self,
        existing: &HashSet<TraitId>,
        new_trait: TraitId,
    ) -> Option<String> {
        let conflicts = self.trait_conflicts(new_trait);
        let new_name = self.rodeo.resolve(&new_trait.0);
        for conflict_id in conflicts {
            if let Ok(conflict_tid) = self.resolve_trait(conflict_id) {
                if existing.contains(&conflict_tid) {
                    return Some(format!(
                        "cannot add trait '{}': conflicts with already-present trait '{}'",
                        new_name, conflict_id
                    ));
                }
            }
        }
        None
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
    use crate::data::{CategoryType, TraitDef};

    fn make_registry() -> PackRegistry {
        let mut reg = PackRegistry::new();
        reg.register_traits(vec![
            TraitDef {
                id: "SHY".into(),
                name: "Shy".into(),
                description: "...".into(),
                hidden: false,
                group: None,
                conflicts: vec![],
            },
            TraitDef {
                id: "POSH".into(),
                name: "Posh".into(),
                description: "...".into(),
                hidden: false,
                group: None,
                conflicts: vec![],
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
    fn personality_name_returns_string() {
        let mut reg = PackRegistry::new();
        let id = reg.intern_personality("ROMANTIC");
        assert_eq!(reg.personality_name(id), "ROMANTIC");
    }

    #[test]
    fn register_names_accumulates() {
        let mut reg = PackRegistry::new();
        reg.register_names(vec!["James".into(), "Thomas".into()], vec!["Emma".into()]);
        assert_eq!(reg.male_names(), &["James", "Thomas"]);
        assert_eq!(reg.female_names(), &["Emma"]);
    }

    #[test]
    fn validate_trait_conflicts_no_errors_when_all_valid() {
        let mut reg = PackRegistry::new();
        reg.register_traits(vec![
            TraitDef {
                id: "SHY".into(),
                name: "Shy".into(),
                description: "...".into(),
                hidden: false,
                group: Some("personality".into()),
                conflicts: vec!["OUTGOING".into()],
            },
            TraitDef {
                id: "OUTGOING".into(),
                name: "Outgoing".into(),
                description: "...".into(),
                hidden: false,
                group: Some("personality".into()),
                conflicts: vec!["SHY".into()],
            },
        ]);
        let errors = reg.validate_trait_conflicts();
        assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
    }

    #[test]
    fn validate_trait_conflicts_reports_unknown_references() {
        let mut reg = PackRegistry::new();
        reg.register_traits(vec![TraitDef {
            id: "SHY".into(),
            name: "Shy".into(),
            description: "...".into(),
            hidden: false,
            group: None,
            conflicts: vec!["NONEXISTENT".into()],
        }]);
        let errors = reg.validate_trait_conflicts();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("NONEXISTENT"));
    }

    #[test]
    fn check_trait_conflict_detects_conflict() {
        let mut reg = PackRegistry::new();
        reg.register_traits(vec![
            TraitDef {
                id: "SHY".into(),
                name: "Shy".into(),
                description: "...".into(),
                hidden: false,
                group: Some("personality".into()),
                conflicts: vec!["OUTGOING".into()],
            },
            TraitDef {
                id: "OUTGOING".into(),
                name: "Outgoing".into(),
                description: "...".into(),
                hidden: false,
                group: Some("personality".into()),
                conflicts: vec!["SHY".into()],
            },
        ]);
        let shy_id = reg.resolve_trait("SHY").unwrap();
        let outgoing_id = reg.resolve_trait("OUTGOING").unwrap();
        let mut existing = HashSet::new();
        existing.insert(shy_id);
        let result = reg.check_trait_conflict(&existing, outgoing_id);
        assert!(result.is_some(), "expected conflict message");
        assert!(result.unwrap().contains("OUTGOING"));
    }

    #[test]
    fn check_trait_conflict_none_when_no_conflict() {
        let mut reg = PackRegistry::new();
        reg.register_traits(vec![
            TraitDef {
                id: "SHY".into(),
                name: "Shy".into(),
                description: "...".into(),
                hidden: false,
                group: Some("personality".into()),
                conflicts: vec!["OUTGOING".into()],
            },
            TraitDef {
                id: "POSH".into(),
                name: "Posh".into(),
                description: "...".into(),
                hidden: false,
                group: None,
                conflicts: vec![],
            },
            TraitDef {
                id: "OUTGOING".into(),
                name: "Outgoing".into(),
                description: "...".into(),
                hidden: false,
                group: Some("personality".into()),
                conflicts: vec!["SHY".into()],
            },
        ]);
        let shy_id = reg.resolve_trait("SHY").unwrap();
        let posh_id = reg.resolve_trait("POSH").unwrap();
        let mut existing = HashSet::new();
        existing.insert(shy_id);
        // POSH has no conflicts, so adding it alongside SHY should be fine
        assert!(reg.check_trait_conflict(&existing, posh_id).is_none());
    }

    #[test]
    fn in_category_returns_true_for_member() {
        let mut reg = PackRegistry::new();
        reg.register_categories(vec![CategoryDef {
            id: "RACE_PRIVILEGED".into(),
            description: "...".into(),
            category_type: CategoryType::Race,
            members: vec!["White".into()],
        }]);
        assert!(reg.in_category("RACE_PRIVILEGED", "White"));
        assert!(!reg.in_category("RACE_PRIVILEGED", "Black"));
        assert!(!reg.in_category("NONEXISTENT", "White"));
    }
}
