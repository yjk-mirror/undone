//! Story-map: derive the base pack's scene-connectivity graph and reconcile it
//! against an authored roadmap, for writers deciding what to write next.
//!
//! Flags and `ARC=STATE` pairs are both treated as **signals**. A scene
//! *produces* signals (effects) and *requires* signals (gates). Dangling =
//! produced but never required (an open door); broken = required but never
//! produced (an unreachable gate).

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use undone_packs::load_packs;
use undone_scene::reachability::{arc_state_eqs, required_game_flags};
use undone_scene::scheduler::SceneBinding;
use undone_scene::script::validate::{source_advance_arcs, source_set_game_flags};
use undone_scene::types::SceneDefinition;
use undone_scene::{load_scenes, load_schedule};

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

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SceneStatus {
    /// Bound (or an entry/goto target) and all gate signals are producible.
    Reachable,
    /// Bound but a gate signal is produced by nothing.
    BrokenGate,
    /// No schedule binding, not an entry scene, no inbound goto.
    #[default]
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

/// Render the agent-facing JSON sidecar (pretty-printed for diff-ability).
pub fn render_json(map: &StoryMap) -> Result<String, String> {
    serde_json::to_string_pretty(map).map_err(|e| format!("json serialize error: {e}"))
}

/// Build the reconciled story map from a packs directory (production path).
pub fn build_story_map(packs_dir: &Path) -> Result<StoryMap, String> {
    let (registry, pack_metas) =
        load_packs(packs_dir).map_err(|e| format!("pack load failed: {e}"))?;

    // Load all scenes across packs.
    let mut scenes = HashMap::new();
    let mut existing: BTreeSet<String> = BTreeSet::new();
    for meta in &pack_metas {
        let scenes_dir = meta.pack_dir.join(&meta.manifest.content.scenes_dir);
        let loaded = load_scenes(&scenes_dir, &registry)
            .map_err(|e| format!("scene load failed for '{}': {e}", meta.manifest.pack.id))?;
        for (id, scene) in loaded {
            existing.insert(short_id(&id).to_string());
            scenes.insert(id, scene);
        }
    }

    // Schedule bindings (gate sources + slot metadata).
    let scheduler =
        load_schedule(&pack_metas, &registry).map_err(|e| format!("schedule load failed: {e}"))?;
    let bindings = scheduler.bindings();

    // Entry scenes are reachable without a binding.
    let mut entry_scenes: HashSet<String> = HashSet::new();
    if let Some(opening) = registry.opening_scene() {
        entry_scenes.insert(opening.to_string());
    }
    if let Some(transformation) = registry.transformation_scene() {
        entry_scenes.insert(transformation.to_string());
    }

    // Preset starting flags are seeded at game start and set by no scene effect,
    // so gates on them are reachable (mirrors the engine's reachability check).
    // Fail loud on a preset-load error — silently degrading would flood the map
    // with false broken-gate findings.
    let mut starting_flags: HashSet<String> = HashSet::new();
    for meta in &pack_metas {
        let presets = undone_packs::preset::load_presets(&meta.pack_dir)
            .map_err(|e| format!("preset load failed for '{}': {e}", meta.manifest.pack.id))?;
        for preset in presets {
            starting_flags.extend(preset.starting_flags);
        }
    }

    let facts = collect_scene_facts(
        &scenes,
        &HashMap::new(),
        &bindings,
        &entry_scenes,
        &starting_flags,
    );

    // Roadmap (one per pack; base pack is the only one for now). Concatenate any
    // that exist so additional packs extend the thread list.
    let mut roadmap = Roadmap::default();
    for meta in &pack_metas {
        let path = meta.pack_dir.join("roadmap.toml");
        if path.exists() {
            let text = std::fs::read_to_string(&path)
                .map_err(|e| format!("read {}: {e}", path.display()))?;
            let parsed = parse_roadmap(&text)?;
            roadmap.threads.extend(parsed.threads);
        }
    }

    let existing_set: HashSet<String> = existing.into_iter().collect();
    Ok(reconcile(&facts, &roadmap, &existing_set, &starting_flags))
}

/// Regenerate both outputs into memory and compare against the committed files.
/// Returns `Ok(true)` if up-to-date, `Ok(false)` if stale.
pub fn is_up_to_date(packs_dir: &Path, md_path: &Path, json_path: &Path) -> Result<bool, String> {
    let map = build_story_map(packs_dir)?;
    let md = render_markdown(&map);
    let json = render_json(&map)?;
    let md_current = std::fs::read_to_string(md_path).unwrap_or_default();
    let json_current = std::fs::read_to_string(json_path).unwrap_or_default();
    Ok(md_current == md && json_current == json)
}

/// Derived facts for one scene, keyed by full `pack::id`.
#[derive(Debug, Clone, Default)]
pub(crate) struct SceneFacts {
    pub produces: Vec<String>,
    /// Signals gating scene ENTRY (schedule binding condition/trigger). Drives
    /// `status` and the per-thread broken-gate findings.
    pub requires: Vec<String>,
    /// Every signal required ANYWHERE in the scene — entry gate plus
    /// action/npc-action/next-branch/intro-variant/thought conditions. Used to
    /// build the global "consumed" set so a flag read only by an action choice
    /// is not mis-reported as dangling.
    pub consumes: Vec<String>,
    pub goto_targets: Vec<String>,
    pub status: SceneStatus,
    pub binding: Option<Binding>,
    pub repeatable: bool,
}

/// Every condition source in a scene that gates a CHOICE or narrator variant
/// (as opposed to scene entry): action conditions and their next-branch + thought
/// conditions, npc-action conditions and their next branches, and intro
/// variant/thought conditions. These are *consumers* of signals — a flag read
/// here is genuinely consumed even though it does not gate scene entry.
fn scene_internal_condition_srcs(scene: &SceneDefinition) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for a in &scene.actions {
        if let Some(c) = &a.condition {
            out.push(c.source.clone());
        }
        for nb in &a.next {
            if let Some(c) = &nb.condition {
                out.push(c.source.clone());
            }
        }
        for t in &a.thoughts {
            if let Some(c) = &t.condition {
                out.push(c.source.clone());
            }
        }
    }
    for na in &scene.npc_actions {
        if let Some(c) = &na.condition {
            out.push(c.source.clone());
        }
        for nb in &na.next {
            if let Some(c) = &nb.condition {
                out.push(c.source.clone());
            }
        }
    }
    for v in &scene.intro_variants {
        out.push(v.condition.source.clone());
    }
    for t in &scene.intro_thoughts {
        if let Some(c) = &t.condition {
            out.push(c.source.clone());
        }
    }
    out
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
/// - `starting_flags`: flags seeded by presets at game start. These are producible
///   without any scene effect, so a gate on one is reachable (mirrors the engine's
///   reachability check) — folded into the producible set for the status pass.
pub(crate) fn collect_scene_facts(
    scenes: &HashMap<String, Arc<SceneDefinition>>,
    gate_sources: &HashMap<String, Vec<String>>,
    bindings: &[SceneBinding],
    entry_scenes: &HashSet<String>,
    starting_flags: &HashSet<String>,
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

        // `consumes` = the entry gate PLUS every internal choice/variant
        // condition, so a flag read only inside an action is still counted as
        // consumed (otherwise it would mis-report as dangling).
        f.consumes.extend(f.requires.iter().cloned());
        for src in scene_internal_condition_srcs(scene) {
            f.consumes.extend(required_signals(&src));
        }

        // Goto targets are matched as-is against the full `pack::id` scene keys,
        // mirroring the engine (`SceneEngine::start_scene` looks up the raw goto
        // string in its scene map). Bare intra-scene targets therefore correctly
        // resolve to nothing and never mark a scene reachable.
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
        dedup(&mut f.consumes);
        dedup(&mut f.goto_targets);

        f.binding = binding_for.get(id).cloned();
        f.repeatable = f.binding.as_ref().is_some_and(|b| !b.once_only);

        all_produced.extend(f.produces.iter().cloned());
        all_goto.extend(f.goto_targets.iter().cloned());
        facts.insert(id.clone(), f);
    }

    // 3. Status pass (needs the global produced set + goto set). Preset starting
    // flags are producible without any scene effect, so gates on them are reachable.
    let mut producible: HashSet<String> = all_produced;
    producible.extend(starting_flags.iter().cloned());
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
/// - `starting_flags` = preset-seeded flags; producible without any scene effect,
///   so a gate on one is not "broken".
pub(crate) fn reconcile(
    facts: &HashMap<String, SceneFacts>,
    roadmap: &Roadmap,
    existing: &HashSet<String>,
    starting_flags: &HashSet<String>,
) -> StoryMap {
    // Global "consumed" set: every signal required ANYWHERE (entry gate or an
    // action/variant condition), so a flag read only inside a choice is not
    // mis-reported as dangling.
    let mut consumed: HashSet<String> = HashSet::new();
    for f in facts.values() {
        consumed.extend(f.consumes.iter().cloned());
    }
    // Global "producible" set: every signal produced by any scene, plus the
    // preset-seeded starting flags (producible without any scene effect).
    let mut producible: HashSet<String> = HashSet::new();
    for f in facts.values() {
        producible.extend(f.produces.iter().cloned());
    }
    producible.extend(starting_flags.iter().cloned());

    // Assign each scene to a thread. Two passes so explicit `scenes`-list
    // membership (direct authorial intent) always wins over `flag_prefix`
    // inference — otherwise an earlier thread's prefix could steal a scene a
    // later thread lists by name. Within each pass, roadmap order breaks ties.
    let mut owner: HashMap<String, usize> = HashMap::new();
    // Pass 1: explicit scenes-list membership.
    for (ti, rt) in roadmap.threads.iter().enumerate() {
        for full in facts.keys() {
            let sid = short_id(full).to_string();
            if owner.contains_key(&sid) {
                continue;
            }
            if rt.scenes.iter().any(|s| s == &sid) {
                owner.insert(sid, ti);
            }
        }
    }
    // Pass 2: flag_prefix inference over whatever remains.
    for (ti, rt) in roadmap.threads.iter().enumerate() {
        let Some(prefix) = rt.flag_prefix.as_deref() else {
            continue;
        };
        for (full, f) in facts {
            let sid = short_id(full).to_string();
            if owner.contains_key(&sid) {
                continue;
            }
            let hit = f
                .produces
                .iter()
                .chain(f.requires.iter())
                .any(|sig| matches_prefix(sig, prefix));
            if hit {
                owner.insert(sid, ti);
            }
        }
    }
    let claimed: HashSet<String> = owner.keys().cloned().collect();

    let mut threads: Vec<Thread> = Vec::new();
    for (ti, rt) in roadmap.threads.iter().enumerate() {
        let mut members: Vec<(String, &SceneFacts)> = facts
            .iter()
            .filter(|(full, _)| owner.get(short_id(full)) == Some(&ti))
            .map(|(full, f)| (full.clone(), f))
            .collect();

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
    for (i, &is_placed) in placed.iter().enumerate() {
        if !is_placed {
            order.push(i);
        }
    }
    order.into_iter().map(|i| owned[i].clone()).collect()
}

/// Render the human-facing Markdown report.
pub fn render_markdown(map: &StoryMap) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "# Story Map");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "> Generated by `cargo run --bin story-map`. Do not edit by hand —"
    );
    let _ = writeln!(
        s,
        "> regenerate after content changes. Threads are declared in"
    );
    let _ = writeln!(s, "> `packs/base/roadmap.toml`.");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "> **Signal model:** connectivity is tracked through game flags and `arc=state` \
         pairs only. Gates on NPC liking, skills, stats, or arousal are NOT modeled, so a \
         scene whose only prerequisite is such a gate will show an empty `requires`. Preset \
         starting flags count as producible."
    );
    let _ = writeln!(s);

    // Write Next digest.
    let _ = writeln!(s, "## Write Next");
    let _ = writeln!(s);
    if map.write_next.is_empty() {
        let _ = writeln!(
            s,
            "_Nothing flagged — every thread is closed and reachable._"
        );
    } else {
        for w in &map.write_next {
            let _ = writeln!(s, "- **[{}]** _{}_ — {}", w.kind, w.thread, w.detail);
        }
    }
    let _ = writeln!(s);

    // Per-thread sections.
    for t in &map.threads {
        let _ = writeln!(s, "## {} ({} scenes)", t.name, t.scenes.len());
        if !t.note.is_empty() {
            let _ = writeln!(s, "_{}_", t.note);
        }
        let _ = writeln!(s);
        for n in &t.scenes {
            let marker = match n.status {
                SceneStatus::Reachable => "",
                SceneStatus::BrokenGate => " ⚠ broken-gate",
                SceneStatus::Unbound => " ⚠ unbound",
            };
            let rep = if n.binding.is_none() {
                "no binding"
            } else if n.repeatable {
                "repeatable"
            } else {
                "once"
            };
            let slot = n.binding.as_ref().map(|b| b.slot.as_str()).unwrap_or("—");
            let _ = writeln!(s, "- `{}` [{} · {}]{}", n.id, slot, rep, marker);
            if !n.requires.is_empty() {
                let _ = writeln!(s, "  - ← requires: {}", n.requires.join(", "));
            }
            if !n.produces.is_empty() {
                let _ = writeln!(s, "  - → sets: {}", n.produces.join(", "));
            }
        }
        if !t.dangling.is_empty() {
            let _ = writeln!(s, "- ⚠ **dangling (write-next):**");
            for d in &t.dangling {
                let _ = writeln!(
                    s,
                    "  - `{}` set by `{}`, consumed by nothing",
                    d.signal, d.set_by
                );
            }
        }
        if !t.broken.is_empty() {
            let _ = writeln!(s, "- ⚠ **broken gates:**");
            for b in &t.broken {
                let _ = writeln!(
                    s,
                    "  - `{}` gates on `{}` (produced by nothing)",
                    b.scene, b.missing
                );
            }
        }
        if !t.planned.is_empty() {
            let _ = writeln!(s, "- 📝 **planned:** {}", t.planned.join(", "));
        }
        let _ = writeln!(s);
    }

    // Footer.
    if !map.orphans.is_empty() {
        let _ = writeln!(s, "## ⚠ Orphan scenes (in no thread)");
        let _ = writeln!(s);
        for o in &map.orphans {
            let _ = writeln!(s, "- `{o}` — add it to a thread in `roadmap.toml`");
        }
        let _ = writeln!(s);
    }
    if !map.drift.is_empty() {
        let _ = writeln!(s, "## ⚠ Roadmap drift");
        let _ = writeln!(s);
        for d in &map.drift {
            let _ = writeln!(s, "- **[{}]** _{}_ — {}", d.kind, d.thread, d.detail);
        }
        let _ = writeln!(s);
    }

    s
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

    fn compile_cond(src: &str) -> undone_scene::script::CompiledScript {
        undone_scene::script::compile_condition(src, &test_registry(), "test").unwrap()
    }

    /// A scene whose single action is gated on `action_cond` (no effect). Used to
    /// prove action-level conditions are counted as consumers.
    fn scene_with_action_condition(id: &str, action_cond: &str) -> Arc<SceneDefinition> {
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
                condition: Some(compile_cond(action_cond)),
                prose: String::new(),
                allow_npc_actions: false,
                effect: None,
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
        let facts = collect_scene_facts(
            &scenes,
            &HashMap::new(),
            &[],
            &Default::default(),
            &HashSet::new(),
        );
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
        let facts = collect_scene_facts(
            &scenes,
            &gates,
            &bindings,
            &Default::default(),
            &HashSet::new(),
        );
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
        let facts = collect_scene_facts(
            &scenes,
            &HashMap::new(),
            &[],
            &Default::default(),
            &HashSet::new(),
        );

        let roadmap = parse_roadmap(
            r#"
[[thread]]
name = "Marcus affair"
flag_prefix = "MARCUS_"
"#,
        )
        .unwrap();

        let existing: HashSet<String> = ["marcus_leverage".to_string()].into_iter().collect();
        let map = reconcile(&facts, &roadmap, &existing, &HashSet::new());

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
        let facts = collect_scene_facts(
            &scenes,
            &HashMap::new(),
            &[],
            &Default::default(),
            &HashSet::new(),
        );
        let roadmap = parse_roadmap(
            r#"
[[thread]]
name = "Jake romance"
flag_prefix = "JAKE_"
"#,
        )
        .unwrap();
        let existing: HashSet<String> = ["lonely".to_string()].into_iter().collect();
        let map = reconcile(&facts, &roadmap, &existing, &HashSet::new());
        assert_eq!(map.orphans, vec!["lonely".to_string()]);
    }

    #[test]
    fn reconcile_flags_planned_now_exists_drift() {
        // BREAKS IF: a planned scene that now exists stops being promoted as drift.
        let scenes: HashMap<String, Arc<SceneDefinition>> = HashMap::new();
        let facts = collect_scene_facts(
            &scenes,
            &HashMap::new(),
            &[],
            &Default::default(),
            &HashSet::new(),
        );
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
        let map = reconcile(&facts, &roadmap, &existing, &HashSet::new());
        assert!(map
            .drift
            .iter()
            .any(|d| d.kind == "planned_now_exists" && d.detail.contains("gym_regular_first")));
    }

    #[test]
    fn markdown_renders_write_next_and_threads() {
        // BREAKS IF: the report stops surfacing the write-next digest or thread headers.
        let mut scenes: HashMap<String, Arc<SceneDefinition>> = HashMap::new();
        scenes.insert(
            "base::marcus_leverage".into(),
            scene(
                "base::marcus_leverage",
                r#"gd.setGameFlag("MARCUS_AFFAIR_COOLING");"#,
            ),
        );
        let facts = collect_scene_facts(
            &scenes,
            &HashMap::new(),
            &[],
            &Default::default(),
            &HashSet::new(),
        );
        let roadmap =
            parse_roadmap("[[thread]]\nname = \"Marcus affair\"\nflag_prefix = \"MARCUS_\"\n")
                .unwrap();
        let existing: HashSet<String> = ["marcus_leverage".to_string()].into_iter().collect();
        let map = reconcile(&facts, &roadmap, &existing, &HashSet::new());

        let md = render_markdown(&map);
        assert!(md.contains("# Story Map"));
        assert!(md.contains("## Write Next"));
        assert!(md.contains("MARCUS_AFFAIR_COOLING"));
        assert!(md.contains("## Marcus affair"));
    }

    #[test]
    fn json_roundtrips_thread_names() {
        // BREAKS IF: the JSON sidecar schema breaks and agents can't read threads.
        let map = StoryMap {
            threads: vec![Thread {
                name: "Jake romance".into(),
                note: String::new(),
                scenes: vec![],
                dangling: vec![],
                broken: vec![],
                planned: vec![],
            }],
            orphans: vec![],
            drift: vec![],
            write_next: vec![],
        };
        let json = render_json(&map).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["threads"][0]["name"], "Jake romance");
    }

    #[test]
    fn action_condition_consumes_so_flag_not_dangling() {
        // BREAKS IF (C1): a flag read only by an action-level condition is
        // mis-reported as dangling because only entry gates were scanned.
        let mut scenes: HashMap<String, Arc<SceneDefinition>> = HashMap::new();
        scenes.insert(
            "base::producer".into(),
            scene("base::producer", r#"gd.setGameFlag("DOOR_OPENED");"#),
        );
        scenes.insert(
            "base::consumer".into(),
            scene_with_action_condition("base::consumer", r#"gd.hasGameFlag("DOOR_OPENED")"#),
        );
        let facts = collect_scene_facts(
            &scenes,
            &HashMap::new(),
            &[],
            &Default::default(),
            &HashSet::new(),
        );
        assert!(
            facts
                .get("base::consumer")
                .unwrap()
                .consumes
                .contains(&"DOOR_OPENED".to_string()),
            "action-level condition must be recorded in `consumes`"
        );
        let roadmap =
            parse_roadmap("[[thread]]\nname = \"T\"\nscenes = [\"producer\", \"consumer\"]\n")
                .unwrap();
        let existing: HashSet<String> = ["producer".to_string(), "consumer".to_string()]
            .into_iter()
            .collect();
        let map = reconcile(&facts, &roadmap, &existing, &HashSet::new());
        assert!(
            !map.threads[0]
                .dangling
                .iter()
                .any(|d| d.signal == "DOOR_OPENED"),
            "flag consumed by an action condition must not be dangling"
        );
    }

    #[test]
    fn preset_starting_flag_satisfies_gate_not_broken() {
        // BREAKS IF (C2): a gate on a preset-seeded flag reads as broken/unreachable
        // because preset starting flags were not folded into the producible set.
        let mut scenes: HashMap<String, Arc<SceneDefinition>> = HashMap::new();
        scenes.insert(
            "base::routed".into(),
            scene("base::routed", r#"w.changeStress(1);"#),
        );
        let bindings = vec![undone_scene::scheduler::SceneBinding {
            scene: "base::routed".into(),
            slot: "free_time".into(),
            weight: 1,
            once_only: false,
            npc_role: None,
            desire_scaled: false,
            condition_source: None,
            trigger_source: Some(r#"gd.hasGameFlag("ROUTE_X")"#.into()),
        }];
        let starting: HashSet<String> = ["ROUTE_X".to_string()].into_iter().collect();
        let facts = collect_scene_facts(
            &scenes,
            &HashMap::new(),
            &bindings,
            &Default::default(),
            &starting,
        );
        assert_eq!(
            facts.get("base::routed").unwrap().status,
            SceneStatus::Reachable
        );
        let roadmap = parse_roadmap("[[thread]]\nname = \"T\"\nscenes = [\"routed\"]\n").unwrap();
        let existing: HashSet<String> = ["routed".to_string()].into_iter().collect();
        let map = reconcile(&facts, &roadmap, &existing, &starting);
        assert!(
            map.threads[0].broken.is_empty(),
            "gate on a preset starting flag must not be broken"
        );
    }

    #[test]
    fn explicit_scenes_list_beats_earlier_prefix_claim() {
        // BREAKS IF (I1): an explicitly-listed scene is stolen by an earlier
        // thread's flag_prefix instead of honoring the author's `scenes` list.
        let mut scenes: HashMap<String, Arc<SceneDefinition>> = HashMap::new();
        scenes.insert(
            "base::locker".into(),
            scene("base::locker", r#"gd.setGameFlag("GYM_LOCKER");"#),
        );
        let facts = collect_scene_facts(
            &scenes,
            &HashMap::new(),
            &[],
            &Default::default(),
            &HashSet::new(),
        );
        let roadmap = parse_roadmap(
            "[[thread]]\nname = \"Gym\"\nflag_prefix = \"GYM_\"\n\n[[thread]]\nname = \"Ambient\"\nscenes = [\"locker\"]\n",
        )
        .unwrap();
        let existing: HashSet<String> = ["locker".to_string()].into_iter().collect();
        let map = reconcile(&facts, &roadmap, &existing, &HashSet::new());
        let gym = map.threads.iter().find(|t| t.name == "Gym").unwrap();
        let ambient = map.threads.iter().find(|t| t.name == "Ambient").unwrap();
        assert!(
            !gym.scenes.iter().any(|s| s.id == "locker"),
            "explicit list in a later thread must not be stolen by GYM_ prefix"
        );
        assert!(
            ambient.scenes.iter().any(|s| s.id == "locker"),
            "explicitly-listed scene must land in its declared thread"
        );
    }
}
