//! Per-scene mutable evaluation state.
//!
//! `SceneCtx` carries the scene-local state that conditions and effects read and
//! write during a scene run: the active male/female NPC, role bindings,
//! scene-local flags, the per-scene skill-roll cache, and the current scene id
//! (for red-check tracking). It lives only for the duration of one scene run.
//!
//! Moved here from the deleted `undone-expr` crate when conditions/effects were
//! cut over to Rhai; it has no dependency on the old expression parser.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use rand::Rng;
use undone_domain::{FemaleNpcKey, MaleNpcKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneNpcRef {
    Male(MaleNpcKey),
    Female(FemaleNpcKey),
}

pub struct SceneCtx {
    pub active_male: Option<MaleNpcKey>,
    pub active_female: Option<FemaleNpcKey>,
    pub role_bindings: HashMap<String, SceneNpcRef>,
    pub scene_flags: HashSet<String>,
    pub weighted_map: HashMap<String, i32>,
    /// Cached percentile rolls (1–100) keyed by skill_id string.
    /// Interior mutability so eval() can cache without needing &mut SceneCtx.
    pub skill_rolls: RefCell<HashMap<String, i32>>,
    /// Scene ID set by the engine before evaluating conditions.
    /// Required for red-check failure tracking.
    pub scene_id: Option<String>,
}

impl SceneCtx {
    pub fn new() -> Self {
        Self {
            active_male: None,
            active_female: None,
            role_bindings: HashMap::new(),
            scene_flags: HashSet::new(),
            weighted_map: HashMap::new(),
            skill_rolls: RefCell::new(HashMap::new()),
            scene_id: None,
        }
    }

    pub fn has_flag(&self, flag: &str) -> bool {
        self.scene_flags.contains(flag)
    }

    pub fn set_flag(&mut self, flag: impl Into<String>) {
        self.scene_flags.insert(flag.into());
    }

    pub fn bind_role(&mut self, role: impl Into<String>, npc: SceneNpcRef) {
        self.role_bindings.insert(role.into(), npc);
    }

    pub fn role_binding(&self, role: &str) -> Option<SceneNpcRef> {
        self.role_bindings.get(role).copied()
    }

    /// Force a specific roll value for testing. Call before evaluating checkSkill.
    pub fn set_skill_roll(&self, skill_id: &str, roll: i32) {
        self.skill_rolls
            .borrow_mut()
            .insert(skill_id.to_string(), roll);
    }

    /// Return a cached roll for this skill, or generate and cache a new one (1–100).
    pub fn get_or_roll_skill(&self, skill_id: &str) -> i32 {
        let mut rolls = self.skill_rolls.borrow_mut();
        *rolls
            .entry(skill_id.to_string())
            .or_insert_with(|| rand::thread_rng().gen_range(1_i32..=100))
    }
}

impl Default for SceneCtx {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get_skill_roll_returns_same_value() {
        let ctx = SceneCtx::new();
        ctx.set_skill_roll("CHARM", 42);
        assert_eq!(ctx.get_or_roll_skill("CHARM"), 42);
    }

    #[test]
    fn get_or_roll_is_idempotent_without_set() {
        let ctx = SceneCtx::new();
        let first = ctx.get_or_roll_skill("FITNESS");
        let second = ctx.get_or_roll_skill("FITNESS");
        assert_eq!(first, second);
    }

    #[test]
    fn different_skills_get_independent_rolls() {
        let ctx = SceneCtx::new();
        ctx.set_skill_roll("CHARM", 30);
        ctx.set_skill_roll("FITNESS", 80);
        assert_eq!(ctx.get_or_roll_skill("CHARM"), 30);
        assert_eq!(ctx.get_or_roll_skill("FITNESS"), 80);
    }
}
