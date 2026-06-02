//! Story-map: derive the base pack's scene-connectivity graph and reconcile it
//! against an authored roadmap, for writers deciding what to write next.
//!
//! Flags and `ARC=STATE` pairs are both treated as **signals**. A scene
//! *produces* signals (effects) and *requires* signals (gates). Dangling =
//! produced but never required (an open door); broken = required but never
//! produced (an unreachable gate).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde::Serialize;
use undone_scene::reachability::{arc_state_eqs, required_game_flags};
use undone_scene::scheduler::SceneBinding;
use undone_scene::script::validate::{source_advance_arcs, source_set_game_flags};
use undone_scene::types::SceneDefinition;

/// The full reconciled map. Serializes to the JSON sidecar.
#[derive(Debug, Clone, Serialize, Default)]
pub struct StoryMap {
    pub threads: Vec<Thread>,
    /// Existing scenes claimed by no thread.
    pub orphans: Vec<String>,
    pub drift: Vec<Drift>,
    pub write_next: Vec<WriteNext>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Thread {
    pub name: String,
    pub note: String,
    /// Scenes ordered by signal-dependency (a scene requiring X sorts after the
    /// scene that produces X), id-stable otherwise.
    pub scenes: Vec<SceneNode>,
    /// Signals produced inside this thread that no gate anywhere consumes.
    pub dangling: Vec<DanglingSignal>,
    /// Gate signals required inside this thread that no scene produces.
    pub broken: Vec<BrokenGate>,
    /// Roadmap `planned` scene ids that do not yet exist as files.
    pub planned: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SceneNode {
    /// Short id (no `pack::` prefix).
    pub id: String,
    pub produces: Vec<String>,
    pub requires: Vec<String>,
    pub status: SceneStatus,
    pub binding: Option<Binding>,
    pub repeatable: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SceneStatus {
    /// Bound (or an entry/goto target) and all gate signals are producible.
    Reachable,
    /// Bound but a gate signal is produced by nothing.
    BrokenGate,
    /// No schedule binding, not an entry scene, no inbound goto.
    Unbound,
}

#[derive(Debug, Clone, Serialize)]
pub struct Binding {
    pub slot: String,
    pub weight: u32,
    pub once_only: bool,
    pub npc_role: Option<String>,
    pub desire_scaled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DanglingSignal {
    pub signal: String,
    pub set_by: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrokenGate {
    pub scene: String,
    pub missing: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Drift {
    /// `planned_now_exists` | `empty_prefix`.
    pub kind: String,
    pub thread: String,
    pub detail: String,
}

// Scope note (deliberate): the design lists a fourth write-next category, "thin
// endings" (a thread's terminal scene sets a flag with no consumer). That is
// *exactly* a `dangling` signal already — implementing it as a separate kind
// would double-report the same finding. It is intentionally folded into
// `dangling`; there is no separate `thin_ending` kind. Do not add one without
// first de-duplicating.
#[derive(Debug, Clone, Serialize)]
pub struct WriteNext {
    pub priority: u8,
    /// `dangling` | `broken` | `planned`.
    pub kind: String,
    pub thread: String,
    pub detail: String,
}

/// Build the reconciled story map from a packs directory. Stub for now.
pub fn build_story_map(_packs_dir: &std::path::Path) -> Result<StoryMap, String> {
    Ok(StoryMap::default())
}

/// Derived facts for one scene, keyed by full `pack::id`.
#[derive(Debug, Clone, Default)]
pub(crate) struct SceneFacts {
    pub produces: Vec<String>,
    pub requires: Vec<String>,
    pub goto_targets: Vec<String>,
    pub status: SceneStatus,
    pub binding: Option<Binding>,
    pub repeatable: bool,
}

impl Default for SceneStatus {
    fn default() -> Self {
        SceneStatus::Unbound
    }
}

/// Turn an effect source into the signal tokens it produces (flags + ARC=STATE).
fn produced_signals(effect_src: &str) -> Vec<String> {
    let mut out = source_set_game_flags(effect_src);
    for (arc, state) in source_advance_arcs(effect_src) {
        out.push(format!("{arc}={state}"));
    }
    out
}

/// Turn a condition source into the POSITIVE signal tokens it requires.
/// Negated flag checks (`!hasGameFlag`) are anti-requirements and excluded.
fn required_signals(cond_src: &str) -> Vec<String> {
    let mut out: Vec<String> = required_game_flags(cond_src)
        .into_iter()
        .filter(|(_, negated)| !negated)
        .map(|(flag, _)| flag)
        .collect();
    for (arc, state) in arc_state_eqs(cond_src) {
        out.push(format!("{arc}={state}"));
    }
    out
}

/// Collect derived facts for every scene.
///
/// - `gate_sources`: test-only override of per-scene gate condition strings. In
///   production pass an empty map; gates come from `bindings`.
/// - `bindings`: schedule bindings (slot/once_only/gate sources) from `Scheduler::bindings`.
/// - `entry_scenes`: ids reachable without a binding (opening + transformation scenes).
pub(crate) fn collect_scene_facts(
    scenes: &HashMap<String, Arc<SceneDefinition>>,
    gate_sources: &HashMap<String, Vec<String>>,
    bindings: &[SceneBinding],
    entry_scenes: &HashSet<String>,
) -> HashMap<String, SceneFacts> {
    // 1. Per-scene binding + gate signal sets (from schedule).
    let mut binding_for: HashMap<String, Binding> = HashMap::new();
    let mut gate_for: HashMap<String, Vec<String>> = HashMap::new();
    for b in bindings {
        // First binding wins for the display projection; extra slots still
        // contribute their gate signals.
        binding_for
            .entry(b.scene.clone())
            .or_insert_with(|| Binding {
                slot: b.slot.clone(),
                weight: b.weight,
                once_only: b.once_only,
                npc_role: b.npc_role.clone(),
                desire_scaled: b.desire_scaled,
            });
        let entry = gate_for.entry(b.scene.clone()).or_default();
        for src in [b.condition_source.as_deref(), b.trigger_source.as_deref()]
            .into_iter()
            .flatten()
        {
            entry.extend(required_signals(src));
        }
    }
    // Merge the test-only gate override.
    for (scene, srcs) in gate_sources {
        let entry = gate_for.entry(scene.clone()).or_default();
        for src in srcs {
            entry.extend(required_signals(src));
        }
    }

    // 2. Per-scene produced/required signals + goto targets from the scene defs.
    let mut facts: HashMap<String, SceneFacts> = HashMap::new();
    let mut all_produced: HashSet<String> = HashSet::new();
    let mut all_goto: HashSet<String> = HashSet::new();

    for (id, scene) in scenes {
        let mut f = SceneFacts::default();

        let effect_srcs = scene
            .actions
            .iter()
            .filter_map(|a| a.effect.as_ref())
            .chain(scene.npc_actions.iter().filter_map(|a| a.effect.as_ref()))
            .map(|s| s.source.as_str());
        for src in effect_srcs {
            f.produces.extend(produced_signals(src));
        }

        // Requirements that gate the SCENE come from the schedule binding.
        if let Some(g) = gate_for.get(id) {
            f.requires.extend(g.iter().cloned());
        }

        // Goto targets (normalised to full ids below by the caller's id set).
        for action in &scene.actions {
            for nb in &action.next {
                if let Some(goto) = &nb.goto {
                    f.goto_targets.push(goto.clone());
                }
            }
        }
        for npc_action in &scene.npc_actions {
            for nb in &npc_action.next {
                if let Some(goto) = &nb.goto {
                    f.goto_targets.push(goto.clone());
                }
            }
        }

        dedup(&mut f.produces);
        dedup(&mut f.requires);
        dedup(&mut f.goto_targets);

        f.binding = binding_for.get(id).cloned();
        f.repeatable = f.binding.as_ref().is_some_and(|b| !b.once_only);

        all_produced.extend(f.produces.iter().cloned());
        all_goto.extend(f.goto_targets.iter().cloned());
        facts.insert(id.clone(), f);
    }

    // 3. Status pass (needs the global produced set + goto set).
    let producible: HashSet<String> = all_produced;
    for (id, f) in facts.iter_mut() {
        let bound = f.binding.is_some();
        let is_entry = entry_scenes.contains(id);
        let is_goto_target = all_goto.contains(id);
        f.status = if !bound && !is_entry && !is_goto_target {
            SceneStatus::Unbound
        } else if f.requires.iter().any(|sig| !producible.contains(sig)) {
            SceneStatus::BrokenGate
        } else {
            SceneStatus::Reachable
        };
    }

    facts
}

fn dedup(v: &mut Vec<String>) {
    let mut seen = HashSet::new();
    v.retain(|s| seen.insert(s.clone()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use undone_packs::PackRegistry;
    use undone_scene::types::{Action, NextBranch, SceneDefinition};

    /// A registry with the `base::arc` arc registered (so `advanceArc` against it
    /// passes the load-time id-validation gate that `compile_effect` runs).
    fn test_registry() -> PackRegistry {
        let mut reg = PackRegistry::new();
        reg.register_arcs(vec![undone_packs::data::ArcDef {
            id: "base::arc".into(),
            states: vec!["settled".into()],
            npc_role: None,
        }]);
        reg
    }

    fn compile_effect(src: &str) -> undone_scene::script::CompiledScript {
        undone_scene::script::compile_effect(src, &test_registry(), "test").unwrap()
    }

    fn scene(id: &str, effect: &str) -> Arc<SceneDefinition> {
        Arc::new(SceneDefinition {
            id: id.to_string(),
            pack: "base".into(),
            intro_prose: "Intro.".into(),
            intro_variants: vec![],
            intro_thoughts: vec![],
            actions: vec![Action {
                id: "go".into(),
                label: "Go".into(),
                detail: String::new(),
                condition: None,
                prose: String::new(),
                allow_npc_actions: false,
                effect: Some(compile_effect(effect)),
                next: vec![NextBranch {
                    condition: None,
                    goto: None,
                    slot: None,
                    finish: true,
                }],
                thoughts: vec![],
            }],
            npc_actions: vec![],
        })
    }

    #[test]
    fn scene_facts_collects_produced_signals() {
        // BREAKS IF: produced-signal extraction misses setGameFlag/advanceArc.
        let mut scenes: HashMap<String, Arc<SceneDefinition>> = HashMap::new();
        scenes.insert(
            "base::a".into(),
            scene(
                "base::a",
                r#"gd.setGameFlag("JAKE_MET"); gd.advanceArc("base::arc", "settled");"#,
            ),
        );
        let facts = collect_scene_facts(&scenes, &HashMap::new(), &[], &Default::default());
        let a = facts.get("base::a").unwrap();
        assert!(a.produces.contains(&"JAKE_MET".to_string()));
        assert!(a.produces.contains(&"base::arc=settled".to_string()));
    }

    #[test]
    fn scene_facts_status_broken_when_gate_signal_unproducible() {
        // BREAKS IF: a scene gated on a never-produced flag stops reading as BrokenGate.
        let mut scenes: HashMap<String, Arc<SceneDefinition>> = HashMap::new();
        scenes.insert("base::b".into(), scene("base::b", r#"w.changeStress(1);"#));
        let mut gates: HashMap<String, Vec<String>> = HashMap::new();
        gates.insert(
            "base::b".into(),
            vec![r#"gd.hasGameFlag("NEVER_SET")"#.into()],
        );
        let bindings = vec![undone_scene::scheduler::SceneBinding {
            scene: "base::b".into(),
            slot: "free_time".into(),
            weight: 1,
            once_only: false,
            npc_role: None,
            desire_scaled: false,
            condition_source: None,
            trigger_source: Some(r#"gd.hasGameFlag("NEVER_SET")"#.into()),
        }];
        let facts = collect_scene_facts(&scenes, &gates, &bindings, &Default::default());
        assert_eq!(
            facts.get("base::b").unwrap().status,
            SceneStatus::BrokenGate
        );
    }
}
