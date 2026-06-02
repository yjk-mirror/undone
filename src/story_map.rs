//! Story-map: derive the base pack's scene-connectivity graph and reconcile it
//! against an authored roadmap, for writers deciding what to write next.
//!
//! Flags and `ARC=STATE` pairs are both treated as **signals**. A scene
//! *produces* signals (effects) and *requires* signals (gates). Dangling =
//! produced but never required (an open door); broken = required but never
//! produced (an unreachable gate).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
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

/// The authored roadmap: thread declarations. Authoring-only; the engine never
/// loads this. Parsed from `packs/<pack>/roadmap.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct Roadmap {
    #[serde(default, rename = "thread")]
    pub threads: Vec<RoadmapThread>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct RoadmapThread {
    pub name: String,
    #[serde(default)]
    pub flag_prefix: Option<String>,
    #[serde(default)]
    pub scenes: Vec<String>,
    #[serde(default)]
    pub planned: Vec<String>,
    #[serde(default)]
    pub note: String,
}

pub(crate) fn parse_roadmap(toml_src: &str) -> Result<Roadmap, String> {
    toml::from_str(toml_src).map_err(|e| format!("roadmap.toml parse error: {e}"))
}

/// Strip the `pack::` prefix from a full scene id.
fn short_id(full: &str) -> &str {
    full.split_once("::").map(|(_, s)| s).unwrap_or(full)
}

/// Does a signal token belong to a thread's flag prefix? Arc tokens look like
/// `arc=state`; the prefix matches the flag/arc head.
fn matches_prefix(signal: &str, prefix: &str) -> bool {
    signal.starts_with(prefix)
}

/// Reconcile derived facts against the roadmap into the final map.
///
/// - `facts` keyed by full `pack::id`.
/// - `existing` = set of short ids that exist as scene files.
pub(crate) fn reconcile(
    facts: &HashMap<String, SceneFacts>,
    roadmap: &Roadmap,
    existing: &HashSet<String>,
) -> StoryMap {
    // Global "consumed" set: every signal required by any scene gate.
    let mut consumed: HashSet<String> = HashSet::new();
    for f in facts.values() {
        consumed.extend(f.requires.iter().cloned());
    }
    // Global "producible" set: every signal produced by any scene.
    let mut producible: HashSet<String> = HashSet::new();
    for f in facts.values() {
        producible.extend(f.produces.iter().cloned());
    }

    // Assign each scene to the first thread (roadmap order) that claims it.
    let mut claimed: HashSet<String> = HashSet::new();
    let mut threads: Vec<Thread> = Vec::new();

    for rt in &roadmap.threads {
        let mut members: Vec<(String, &SceneFacts)> = Vec::new();
        for (full, f) in facts {
            let sid = short_id(full).to_string();
            if claimed.contains(&sid) {
                continue;
            }
            let by_list = rt.scenes.iter().any(|s| s == &sid);
            let by_prefix = rt.flag_prefix.as_deref().is_some_and(|p| {
                f.produces
                    .iter()
                    .chain(f.requires.iter())
                    .any(|sig| matches_prefix(sig, p))
            });
            if by_list || by_prefix {
                claimed.insert(sid.clone());
                members.push((full.clone(), f));
            }
        }

        // Order members by signal dependency (Kahn-ish, id-stable tiebreak).
        members.sort_by(|a, b| short_id(&a.0).cmp(short_id(&b.0)));
        let ordered = order_by_dependency(members);

        // Thread findings.
        let mut dangling: Vec<DanglingSignal> = Vec::new();
        let mut broken: Vec<BrokenGate> = Vec::new();
        let mut nodes: Vec<SceneNode> = Vec::new();
        for (full, f) in &ordered {
            let sid = short_id(full).to_string();
            for sig in &f.produces {
                if !consumed.contains(sig) {
                    dangling.push(DanglingSignal {
                        signal: sig.clone(),
                        set_by: sid.clone(),
                    });
                }
            }
            for sig in &f.requires {
                if !producible.contains(sig) {
                    broken.push(BrokenGate {
                        scene: sid.clone(),
                        missing: sig.clone(),
                    });
                }
            }
            nodes.push(SceneNode {
                id: sid,
                produces: f.produces.clone(),
                requires: f.requires.clone(),
                status: f.status.clone(),
                binding: f.binding.clone(),
                repeatable: f.repeatable,
            });
        }

        let planned: Vec<String> = rt
            .planned
            .iter()
            .filter(|p| !existing.contains(*p))
            .cloned()
            .collect();

        threads.push(Thread {
            name: rt.name.clone(),
            note: rt.note.clone(),
            scenes: nodes,
            dangling,
            broken,
            planned,
        });
    }

    // Orphans: existing scenes claimed by no thread.
    let mut orphans: Vec<String> = facts
        .keys()
        .map(|full| short_id(full).to_string())
        .filter(|sid| !claimed.contains(sid))
        .collect();
    orphans.sort();

    // Drift: planned ids that now exist.
    let mut drift: Vec<Drift> = Vec::new();
    for rt in &roadmap.threads {
        for p in &rt.planned {
            if existing.contains(p) {
                drift.push(Drift {
                    kind: "planned_now_exists".into(),
                    thread: rt.name.clone(),
                    detail: format!("planned scene '{p}' now exists — promote it to `scenes`"),
                });
            }
        }
        if let Some(prefix) = &rt.flag_prefix {
            let has_member = threads
                .iter()
                .find(|t| t.name == rt.name)
                .is_some_and(|t| !t.scenes.is_empty());
            let prefix_used = producible
                .iter()
                .chain(consumed.iter())
                .any(|s| matches_prefix(s, prefix));
            if !has_member && !prefix_used {
                drift.push(Drift {
                    kind: "empty_prefix".into(),
                    thread: rt.name.clone(),
                    detail: format!("flag_prefix '{prefix}' matches no scene signals yet"),
                });
            }
        }
    }

    // Write-next digest, priority order.
    let mut write_next: Vec<WriteNext> = Vec::new();
    for t in &threads {
        for d in &t.dangling {
            write_next.push(WriteNext {
                priority: 1,
                kind: "dangling".into(),
                thread: t.name.clone(),
                detail: format!(
                    "'{}' set by {} — no scene consumes it (write a follow-up)",
                    d.signal, d.set_by
                ),
            });
        }
    }
    for t in &threads {
        for b in &t.broken {
            write_next.push(WriteNext {
                priority: 2,
                kind: "broken".into(),
                thread: t.name.clone(),
                detail: format!(
                    "{} gates on '{}' which no scene produces (fix gate or write producer)",
                    b.scene, b.missing
                ),
            });
        }
    }
    for t in &threads {
        for p in &t.planned {
            write_next.push(WriteNext {
                priority: 3,
                kind: "planned".into(),
                thread: t.name.clone(),
                detail: format!("planned, not yet written: {p}"),
            });
        }
    }
    write_next.sort_by_key(|w| w.priority);

    StoryMap {
        threads,
        orphans,
        drift,
        write_next,
    }
}

/// Order scenes so a scene requiring signal X follows the scene that produces X.
/// Kahn-style; remaining (cyclic) nodes appended in their incoming id order.
fn order_by_dependency(members: Vec<(String, &SceneFacts)>) -> Vec<(String, SceneFacts)> {
    let owned: Vec<(String, SceneFacts)> =
        members.into_iter().map(|(id, f)| (id, f.clone())).collect();

    // produced signal -> indices that produce it
    let mut produced_by: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, (_, f)) in owned.iter().enumerate() {
        for sig in &f.produces {
            produced_by.entry(sig.clone()).or_default().push(i);
        }
    }
    let n = owned.len();
    let mut indeg = vec![0usize; n];
    let mut edges: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, (_, f)) in owned.iter().enumerate() {
        for sig in &f.requires {
            if let Some(producers) = produced_by.get(sig) {
                for &p in producers {
                    if p != i {
                        edges[p].push(i);
                        indeg[i] += 1;
                    }
                }
            }
        }
    }
    let mut queue: Vec<usize> = (0..n).filter(|&i| indeg[i] == 0).collect();
    queue.sort_by(|&a, &b| short_id(&owned[a].0).cmp(short_id(&owned[b].0)));
    let mut order: Vec<usize> = Vec::new();
    let mut placed = vec![false; n];
    while let Some(i) = queue.first().copied() {
        queue.remove(0);
        if placed[i] {
            continue;
        }
        placed[i] = true;
        order.push(i);
        let mut newly: Vec<usize> = Vec::new();
        for &j in &edges[i] {
            indeg[j] -= 1;
            if indeg[j] == 0 {
                newly.push(j);
            }
        }
        newly.sort_by(|&a, &b| short_id(&owned[a].0).cmp(short_id(&owned[b].0)));
        queue.extend(newly);
        queue.sort_by(|&a, &b| short_id(&owned[a].0).cmp(short_id(&owned[b].0)));
    }
    // Append any cyclic leftovers in id order.
    for i in 0..n {
        if !placed[i] {
            order.push(i);
        }
    }
    order.into_iter().map(|i| owned[i].clone()).collect()
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

    #[test]
    fn roadmap_parses_threads() {
        // BREAKS IF: roadmap schema changes shape and the parser silently drops fields.
        let toml = r#"
[[thread]]
name = "Marcus affair"
flag_prefix = "MARCUS_"
scenes = ["marcus_repeat_office"]
planned = ["marcus_reconcile"]
note = "Recurring affair."
"#;
        let rm = parse_roadmap(toml).unwrap();
        assert_eq!(rm.threads.len(), 1);
        let t = &rm.threads[0];
        assert_eq!(t.name, "Marcus affair");
        assert_eq!(t.flag_prefix.as_deref(), Some("MARCUS_"));
        assert_eq!(t.scenes, vec!["marcus_repeat_office".to_string()]);
        assert_eq!(t.planned, vec!["marcus_reconcile".to_string()]);
    }

    #[test]
    fn reconcile_assigns_threads_and_flags_dangling() {
        // BREAKS IF: a produced-but-unconsumed signal stops surfacing as a write-next hook.
        let mut scenes: HashMap<String, Arc<SceneDefinition>> = HashMap::new();
        scenes.insert(
            "base::marcus_leverage".into(),
            scene(
                "base::marcus_leverage",
                r#"gd.setGameFlag("MARCUS_AFFAIR_COOLING");"#,
            ),
        );
        let facts = collect_scene_facts(&scenes, &HashMap::new(), &[], &Default::default());

        let roadmap = parse_roadmap(
            r#"
[[thread]]
name = "Marcus affair"
flag_prefix = "MARCUS_"
"#,
        )
        .unwrap();

        let existing: HashSet<String> = ["marcus_leverage".to_string()].into_iter().collect();
        let map = reconcile(&facts, &roadmap, &existing);

        let marcus = map
            .threads
            .iter()
            .find(|t| t.name == "Marcus affair")
            .unwrap();
        assert!(marcus.scenes.iter().any(|s| s.id == "marcus_leverage"));
        assert!(marcus
            .dangling
            .iter()
            .any(|d| d.signal == "MARCUS_AFFAIR_COOLING"));
        assert!(map
            .write_next
            .iter()
            .any(|w| w.kind == "dangling" && w.detail.contains("MARCUS_AFFAIR_COOLING")));
        assert!(map.orphans.is_empty());
    }

    #[test]
    fn reconcile_reports_orphan_for_unclaimed_scene() {
        // BREAKS IF: a scene matching no thread silently disappears from the map.
        let mut scenes: HashMap<String, Arc<SceneDefinition>> = HashMap::new();
        scenes.insert(
            "base::lonely".into(),
            scene("base::lonely", r#"w.changeStress(1);"#),
        );
        let facts = collect_scene_facts(&scenes, &HashMap::new(), &[], &Default::default());
        let roadmap = parse_roadmap(
            r#"
[[thread]]
name = "Jake romance"
flag_prefix = "JAKE_"
"#,
        )
        .unwrap();
        let existing: HashSet<String> = ["lonely".to_string()].into_iter().collect();
        let map = reconcile(&facts, &roadmap, &existing);
        assert_eq!(map.orphans, vec!["lonely".to_string()]);
    }

    #[test]
    fn reconcile_flags_planned_now_exists_drift() {
        // BREAKS IF: a planned scene that now exists stops being promoted as drift.
        let scenes: HashMap<String, Arc<SceneDefinition>> = HashMap::new();
        let facts = collect_scene_facts(&scenes, &HashMap::new(), &[], &Default::default());
        let roadmap = parse_roadmap(
            r#"
[[thread]]
name = "Cal / gym"
flag_prefix = "GYM_"
planned = ["gym_regular_first"]
"#,
        )
        .unwrap();
        let existing: HashSet<String> = ["gym_regular_first".to_string()].into_iter().collect();
        let map = reconcile(&facts, &roadmap, &existing);
        assert!(map
            .drift
            .iter()
            .any(|d| d.kind == "planned_now_exists" && d.detail.contains("gym_regular_first")));
    }
}
