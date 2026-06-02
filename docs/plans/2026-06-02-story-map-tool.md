# Story-Map Tool Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use ops:executing-plans to implement this plan task-by-task.

**Goal:** Build a `story-map` CLI that derives the scene-connectivity graph from base-pack data, reconciles it against an authored `roadmap.toml`, and emits a writer-facing Markdown report + an agent-readable JSON sidecar answering "what exists, what connects, what to write next."

**Architecture:** A new library module `src/story_map.rs` (logic, unit-testable) + a thin CLI `src/bin/story_map.rs`, mirroring the existing `validate_pack.rs` split. It reuses the existing pack loader (`undone_packs::load_packs`, `undone_scene::{load_scenes, load_schedule}`) and the source-scan helpers in `undone-scene` (`script::validate::source_*`, `reachability::*`). It treats game flags and `ARC=STATE` pairs uniformly as **signals**: a scene *produces* signals (via effects) and *requires* signals (via gates). Dangling = produced-but-never-required; broken = required-but-never-produced. Threads are declared in `packs/base/roadmap.toml`; any scene claimed by no thread is reported as an orphan.

**Tech Stack:** Rust (root `undone` crate, game workspace), `serde`/`serde_json` for the JSON sidecar, `toml` for the roadmap, reused `undone-scene` scan helpers.

**Design reference:** `docs/plans/2026-06-02-story-map-tool-design.md`.

---

## File Structure

**New files:**
- `src/story_map.rs` — all logic: data types (serde-derived), `build_story_map`, `render_markdown`, `render_json`, staleness check. One responsibility: turn a packs dir + roadmap into a `StoryMap` and its two rendered forms.
- `src/bin/story_map.rs` — thin CLI: regenerate the two output files, or `--check` for staleness.
- `packs/base/roadmap.toml` — authored thread roadmap (authoring-only; engine never loads it).
- `tests/story_map_acceptance.rs` — acceptance tests against the real base pack.

**Modified files:**
- `crates/undone-scene/src/reachability.rs` — expose two `pub` scan wrappers (`required_game_flags`, `arc_state_eqs`) over the existing private byte-scanners.
- `crates/undone-scene/src/scheduler.rs` — add `pub struct SceneBinding` + `pub fn bindings()`.
- `Cargo.toml` (root) — add the `story-map` bin + `serde`/`serde_json` deps.
- `src/lib.rs` — add `pub mod story_map;`.
- `docs/content-schema.md`, `.claude/agents/scene-writer.md`, `HANDOFF.md` — doc updates (Task 11).

---

## Task 1: Expose negation-aware scan wrappers in `reachability.rs`

The story-map needs the flags/arc-states a condition *requires*, with negation distinguished (`!hasGameFlag("X")` is an anti-requirement, not a dependency). `reachability.rs` already has private byte-scanners that track this. Expose thin `pub` wrappers — no logic duplication.

**Files:**
- Modify: `crates/undone-scene/src/reachability.rs`

**Step 1: Write the failing test**

Add to the `#[cfg(test)] mod tests` block at the bottom of `crates/undone-scene/src/reachability.rs`:

```rust
    #[test]
    fn required_game_flags_reports_flag_and_negation() {
        // BREAKS IF: positive vs negated flag refs stop being distinguished —
        // story-map would treat `!hasGameFlag` as a dependency.
        let got = required_game_flags(r#"gd.hasGameFlag("A") && !gd.hasGameFlag("B")"#);
        assert!(got.contains(&("A".to_string(), false)));
        assert!(got.contains(&("B".to_string(), true)));
    }

    #[test]
    fn arc_state_eqs_reports_arc_and_state() {
        // BREAKS IF: arcState equality extraction breaks — story-map loses arc edges.
        let got = arc_state_eqs(r#"gd.arcState("base::workplace_opening") == "settled""#);
        assert_eq!(got, vec![("base::workplace_opening".to_string(), "settled".to_string())]);
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p undone-scene required_game_flags_reports_flag_and_negation`
Expected: FAIL — `cannot find function required_game_flags in this scope`.

**Step 3: Add the public wrappers**

In `crates/undone-scene/src/reachability.rs`, immediately above the private `fn find_hasgameflag` (around line 130), add:

```rust
/// All `hasGameFlag("FLAG")` references in a condition source, each paired with
/// whether it is logically negated (`!gd.hasGameFlag(...)`). A negated reference
/// is an anti-requirement (the flag must be ABSENT), not a dependency.
pub fn required_game_flags(src: &str) -> Vec<(String, bool)> {
    find_hasgameflag(src)
}

/// All `arcState("ARC") == "STATE"` equality references in a condition source,
/// returned as `(arc, state)` pairs.
pub fn arc_state_eqs(src: &str) -> Vec<(String, String)> {
    find_eq_call(src, "arcState")
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p undone-scene reachability`
Expected: PASS (the two new tests plus the existing reachability tests).

**Step 5: Commit**

```bash
git add crates/undone-scene/src/reachability.rs
git commit -m "feat(scene): expose required_game_flags + arc_state_eqs scan wrappers"
```

---

## Task 2: Add `SceneBinding` + `Scheduler::bindings()` to the scheduler

The story-map needs per-event binding metadata (slot, weight, once_only, npc_role, desire_scaled, and the raw gate sources). The internal `ScheduleEvent`/`ScheduleSlot` types are `pub(crate)`, so add a public projection.

**Files:**
- Modify: `crates/undone-scene/src/scheduler.rs`

**Step 1: Write the failing test**

Add to the `#[cfg(test)] mod tests` block in `crates/undone-scene/src/scheduler.rs`. (The test module already imports `super::*` and has helpers like `bare_event`; this test builds a scheduler from a real slot.)

```rust
    #[test]
    fn bindings_projects_event_metadata() {
        // BREAKS IF: the public binding projection drops a field story-map needs.
        let reg = PackRegistry::new();
        let event = ScheduleEvent {
            scene: "base::coffee_shop".to_string(),
            condition: None,
            weight: 0,
            once_only: true,
            trigger: Some(compile_condition(r#"gd.week() >= 2"#, &reg, "test").unwrap()),
            npc_role: Some("ROLE_JAKE".to_string()),
            desire_scaled: false,
        };
        let mut slots = HashMap::new();
        slots.insert("free_time".to_string(), vec![event]);
        let scheduler = Scheduler::from_slots_for_tests(slots);

        let bindings = scheduler.bindings();
        assert_eq!(bindings.len(), 1);
        let b = &bindings[0];
        assert_eq!(b.scene, "base::coffee_shop");
        assert_eq!(b.slot, "free_time");
        assert!(b.once_only);
        assert_eq!(b.npc_role.as_deref(), Some("ROLE_JAKE"));
        assert_eq!(b.trigger_source.as_deref(), Some("gd.week() >= 2"));
        assert_eq!(b.condition_source, None);
    }
```

If `compile_condition` is not already imported in the test module, add `use crate::script::compile_condition;` at the top of the `mod tests` block (check the existing `use super::*;` first — `compile_condition` is re-exported from `crate::script`).

**Step 2: Run test to verify it fails**

Run: `cargo test -p undone-scene bindings_projects_event_metadata`
Expected: FAIL — `no method named bindings found for struct Scheduler`.

**Step 3: Add the public struct and method**

In `crates/undone-scene/src/scheduler.rs`, in the "Public result types" section (just after the `PickResult` struct, around line 132), add:

```rust
/// A read-only projection of one scheduled event's binding metadata, for
/// authoring tools (story-map). Carries the raw gate sources so callers can
/// source-scan them without re-parsing the schedule.
#[derive(Debug, Clone)]
pub struct SceneBinding {
    pub scene: String,
    pub slot: String,
    pub weight: u32,
    pub once_only: bool,
    pub npc_role: Option<String>,
    pub desire_scaled: bool,
    pub condition_source: Option<String>,
    pub trigger_source: Option<String>,
}
```

Then add this method inside `impl Scheduler` (place it right after `all_conditions`, around line 198):

```rust
    /// Project every scheduled event's binding metadata for authoring tools.
    /// One entry per (slot, event); a scene bound in multiple slots yields
    /// multiple entries.
    pub fn bindings(&self) -> Vec<SceneBinding> {
        let mut out = Vec::new();
        for slot in self.slots.values() {
            for event in &slot.events {
                out.push(SceneBinding {
                    scene: event.scene.clone(),
                    slot: slot.name.clone(),
                    weight: event.weight,
                    once_only: event.once_only,
                    npc_role: event.npc_role.clone(),
                    desire_scaled: event.desire_scaled,
                    condition_source: event.condition.as_ref().map(|s| s.source.clone()),
                    trigger_source: event.trigger.as_ref().map(|s| s.source.clone()),
                });
            }
        }
        out
    }
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p undone-scene bindings_projects_event_metadata`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/undone-scene/src/scheduler.rs
git commit -m "feat(scene): add Scheduler::bindings() projection for authoring tools"
```

---

## Task 3: Scaffold the `story_map` module + wiring

Create the module with its serde data types and a stub `build_story_map`, wire it into the crate and Cargo, so later tasks have a compiling home.

**Files:**
- Create: `src/story_map.rs`
- Modify: `src/lib.rs`
- Modify: `Cargo.toml`

**Step 1: Add deps + bin + module declaration**

In root `Cargo.toml`, add a second `[[bin]]` right after the existing `validate-pack` bin (after line 55):

```toml
[[bin]]
name = "story-map"
path = "src/bin/story_map.rs"
```

In the same file's `[dependencies]` section (after the `toml` line, line 66), add:

```toml
serde         = { workspace = true }
serde_json    = { workspace = true }
```

In `src/lib.rs`, add below the existing line:

```rust
pub mod story_map;
```

**Step 2: Create the module with data types + stub**

Create `src/story_map.rs`:

```rust
//! Story-map: derive the base pack's scene-connectivity graph and reconcile it
//! against an authored roadmap, for writers deciding what to write next.
//!
//! Flags and `ARC=STATE` pairs are both treated as **signals**. A scene
//! *produces* signals (effects) and *requires* signals (gates). Dangling =
//! produced but never required (an open door); broken = required but never
//! produced (an unreachable gate).

use serde::Serialize;

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

#[derive(Debug, Clone, Serialize)]
pub struct WriteNext {
    pub priority: u8,
    /// `dangling` | `broken` | `planned`.
    pub kind: String,
    pub thread: String,
    pub detail: String,
}
```

> **Scope note (deliberate):** the design lists a fourth write-next category,
> "thin endings" (a thread's terminal scene sets a flag with no consumer). That is
> *exactly* a `dangling` signal already — implementing it as a separate kind would
> double-report the same finding. It is intentionally folded into `dangling`; there
> is no separate `thin_ending` kind. Do not add one without first de-duplicating.

/// Build the reconciled story map from a packs directory. Stub for now.
pub fn build_story_map(_packs_dir: &std::path::Path) -> Result<StoryMap, String> {
    Ok(StoryMap::default())
}
```

**Step 3: Verify it compiles**

Run: `cargo build -p undone --bin validate-pack`
Expected: SUCCESS (the new module compiles; `story-map` bin does not exist yet so build just that bin, or run `cargo check`):

Run: `cargo check`
Expected: SUCCESS. (`story-map` bin path does not exist yet — `cargo check` checks lib targets; if it errors on the missing bin path, proceed to Task 9 which creates it, or temporarily run `cargo check --lib`.)

> Note: because `Cargo.toml` now references `src/bin/story_map.rs` which is created in Task 9, run `cargo check --lib` for Tasks 3–8 and the full `cargo check` from Task 9 onward.

**Step 4: Commit**

```bash
git add Cargo.toml src/lib.rs src/story_map.rs
git commit -m "feat(story-map): scaffold module, data types, and cargo wiring"
```

---

## Task 4: Extract per-scene derived facts

Walk every loaded scene, computing produced and required signals, goto targets, and binding-derived status. This is the heart of the derived half.

**Files:**
- Modify: `src/story_map.rs`

**Step 1: Write the failing test**

Add a `#[cfg(test)] mod tests` block at the bottom of `src/story_map.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use undone_scene::types::{Action, NextBranch, SceneDefinition};
    use undone_packs::PackRegistry;

    fn compile_effect(src: &str) -> undone_scene::script::CompiledScript {
        undone_scene::script::compile_effect(src, &PackRegistry::new(), "test").unwrap()
    }
    fn compile_cond(src: &str) -> undone_scene::script::CompiledScript {
        undone_scene::script::compile_condition(src, &PackRegistry::new(), "test").unwrap()
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
                next: vec![NextBranch { condition: None, goto: None, slot: None, finish: true }],
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
            scene("base::a", r#"gd.setGameFlag("JAKE_MET"); gd.advanceArc("base::arc", "settled");"#),
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
        scenes.insert("base::b".into(), scene("base::b", r#"gd.changeStress(1);"#));
        let mut gates: HashMap<String, Vec<String>> = HashMap::new();
        gates.insert("base::b".into(), vec![r#"gd.hasGameFlag("NEVER_SET")"#.into()]);
        let bindings = vec![undone_scene::scheduler::SceneBinding {
            scene: "base::b".into(), slot: "free_time".into(), weight: 1, once_only: false,
            npc_role: None, desire_scaled: false, condition_source: None,
            trigger_source: Some(r#"gd.hasGameFlag("NEVER_SET")"#.into()),
        }];
        let facts = collect_scene_facts(&scenes, &gates, &bindings, &Default::default());
        assert_eq!(facts.get("base::b").unwrap().status, SceneStatus::BrokenGate);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p undone --lib story_map::tests::scene_facts_collects_produced_signals`
Expected: FAIL — `cannot find function collect_scene_facts`.

**Step 3: Implement the facts collector**

Add to `src/story_map.rs` (above the `tests` module). The `_gate_sources` map is an explicit test seam; in production it is derived from bindings (see below — production calls pass an empty map and rely on `bindings`).

```rust
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use undone_scene::reachability::{arc_state_eqs, required_game_flags};
use undone_scene::scheduler::SceneBinding;
use undone_scene::script::validate::{source_advance_arcs, source_set_game_flags};
use undone_scene::types::SceneDefinition;

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
        binding_for.entry(b.scene.clone()).or_insert_with(|| Binding {
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
```

> Note on `source_set_game_flags`/`source_advance_arcs` import path: they live in
> `crates/undone-scene/src/script/validate.rs` as `pub`. Confirm `validate` is a
> `pub mod` re-exported under `undone_scene::script`. If the path
> `undone_scene::script::validate::source_set_game_flags` does not resolve, check
> `crates/undone-scene/src/script/mod.rs` for the actual re-export (it may be
> `undone_scene::script::source_set_game_flags`) and adjust the `use`.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p undone --lib story_map`
Expected: PASS (both Task 4 tests).

**Step 5: Commit**

```bash
git add src/story_map.rs
git commit -m "feat(story-map): per-scene derived facts (produces/requires/status)"
```

---

## Task 5: Roadmap parsing + author the base roadmap

Parse `packs/base/roadmap.toml` into typed threads, and author the real base roadmap.

**Files:**
- Modify: `src/story_map.rs`
- Create: `packs/base/roadmap.toml`

**Step 1: Write the failing test**

Add to the `tests` module in `src/story_map.rs`:

```rust
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p undone --lib story_map::tests::roadmap_parses_threads`
Expected: FAIL — `cannot find function parse_roadmap`.

**Step 3: Implement the parser**

Add to `src/story_map.rs` (above the `tests` module):

```rust
use serde::Deserialize;

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
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p undone --lib story_map::tests::roadmap_parses_threads`
Expected: PASS.

**Step 5: Author the base roadmap**

Create `packs/base/roadmap.toml`. Start from this skeleton; the implementer will refine `scenes` lists in Task 10 until the orphan list is empty. Scene ids are short (no `base::`).

```toml
# Story roadmap for the base pack. AUTHORING-ONLY — the engine never loads this
# file; only the `story-map` tool reads it. Declares the narrative threads scenes
# are grouped into. A scene is claimed by a thread if any flag it sets/requires
# starts with `flag_prefix`, OR its short id is in `scenes`. Any scene claimed by
# no thread is reported as an orphan. `planned` lists intended, not-yet-written ids.

[[thread]]
name = "Opening / first weeks"
note = "Arrival, the workplace opening arc, settling in, the first callbacks."
scenes = [
  "transformation_intro", "rain_shelter", "morning_routine", "plan_your_day",
  "opening_callback_status_assertion", "opening_callback_first_week_solitude",
  "opening_callback_mirror_afterglow", "opening_callback_transactional_defense",
  "landlord_repair",
]

[[thread]]
name = "Jake romance"
flag_prefix = "JAKE_"
scenes = ["coffee_shop", "coffee_shop_return", "jake_outside", "jake_first_date",
          "jake_second_date", "jake_stays_over", "jake_text_messages"]
note = "Met at the coffee shop -> dating -> intimacy -> domesticity -> she-seeks."

[[thread]]
name = "Marcus affair"
flag_prefix = "MARCUS_"
scenes = ["marcus_repeat_office", "marcus_pushes", "marcus_leverage",
          "marcus_monday_rejected"]
note = "Workplace tension -> affair -> escalation -> leverage cost -> cooling."

[[thread]]
name = "Cal / gym"
flag_prefix = "GYM_"
scenes = []
note = "Gym-regular power-inversion + submission ladder."

[[thread]]
name = "Theo / campus"
scenes = ["campus_arrival", "campus_dorm", "campus_orientation", "campus_library",
          "campus_call_home", "campus_study_session", "campus_dining_hall",
          "campus_theo_morning", "campus_dining_after_theo"]
note = "Campus route: orientation -> Theo -> morning-after -> public visibility."

[[thread]]
name = "Desire / looping adult"
flag_prefix = "DESIRE_"
scenes = ["desire_solo_night", "desire_ambush"]
note = "The recurring need-state: release valves and ambushes."

[[thread]]
name = "Ambient life"
scenes = ["bookstore", "park_walk", "grocery_store", "evening_home",
          "neighborhood_bar", "laundromat_night", "shopping_mall", "weekend_morning",
          "bad_date", "bar_closing_time", "gym_changing_room", "party_invitation",
          "work_corridor", "work_friday", "work_late", "work_lunch",
          "work_marcus_aftermath", "work_marcus_coffee", "work_marcus_drinks",
          "work_marcus_favor", "workplace_work_meeting", "workplace_evening"]
note = "Texture scenes not owned by a single arc. Reassign to a thread as arcs grow."
```

> The implementer does NOT hand-verify this is complete here — Task 10's
> acceptance test fails loudly if any non-archived scene is unclaimed, and the
> fix is to move that scene id into the right thread's `scenes` list.

**Step 6: Commit**

```bash
git add src/story_map.rs packs/base/roadmap.toml
git commit -m "feat(story-map): roadmap parser + base pack thread roadmap"
```

---

## Task 6: Reconciliation — assemble threads, findings, write-next

Combine derived facts + roadmap into the `StoryMap`: assign scenes to threads, order them, compute dangling/broken/orphan/drift, and rank write-next.

**Files:**
- Modify: `src/story_map.rs`

**Step 1: Write the failing test**

Add to the `tests` module:

```rust
    #[test]
    fn reconcile_assigns_threads_and_flags_dangling() {
        // BREAKS IF: a produced-but-unconsumed signal stops surfacing as a write-next hook.
        let mut scenes: HashMap<String, Arc<SceneDefinition>> = HashMap::new();
        scenes.insert("base::marcus_leverage".into(),
            scene("base::marcus_leverage", r#"gd.setGameFlag("MARCUS_AFFAIR_COOLING");"#));
        let facts = collect_scene_facts(&scenes, &HashMap::new(), &[], &Default::default());

        let roadmap = parse_roadmap(r#"
[[thread]]
name = "Marcus affair"
flag_prefix = "MARCUS_"
"#).unwrap();

        let existing: HashSet<String> = ["marcus_leverage".to_string()].into_iter().collect();
        let map = reconcile(&facts, &roadmap, &existing);

        let marcus = map.threads.iter().find(|t| t.name == "Marcus affair").unwrap();
        assert!(marcus.scenes.iter().any(|s| s.id == "marcus_leverage"));
        assert!(marcus.dangling.iter().any(|d| d.signal == "MARCUS_AFFAIR_COOLING"));
        assert!(map.write_next.iter().any(|w| w.kind == "dangling"
            && w.detail.contains("MARCUS_AFFAIR_COOLING")));
        assert!(map.orphans.is_empty());
    }

    #[test]
    fn reconcile_reports_orphan_for_unclaimed_scene() {
        // BREAKS IF: a scene matching no thread silently disappears from the map.
        let mut scenes: HashMap<String, Arc<SceneDefinition>> = HashMap::new();
        scenes.insert("base::lonely".into(), scene("base::lonely", r#"gd.changeStress(1);"#));
        let facts = collect_scene_facts(&scenes, &HashMap::new(), &[], &Default::default());
        let roadmap = parse_roadmap(r#"
[[thread]]
name = "Jake romance"
flag_prefix = "JAKE_"
"#).unwrap();
        let existing: HashSet<String> = ["lonely".to_string()].into_iter().collect();
        let map = reconcile(&facts, &roadmap, &existing);
        assert_eq!(map.orphans, vec!["lonely".to_string()]);
    }

    #[test]
    fn reconcile_flags_planned_now_exists_drift() {
        // BREAKS IF: a planned scene that now exists stops being promoted as drift.
        let scenes: HashMap<String, Arc<SceneDefinition>> = HashMap::new();
        let facts = collect_scene_facts(&scenes, &HashMap::new(), &[], &Default::default());
        let roadmap = parse_roadmap(r#"
[[thread]]
name = "Cal / gym"
flag_prefix = "GYM_"
planned = ["gym_regular_first"]
"#).unwrap();
        let existing: HashSet<String> = ["gym_regular_first".to_string()].into_iter().collect();
        let map = reconcile(&facts, &roadmap, &existing);
        assert!(map.drift.iter().any(|d| d.kind == "planned_now_exists"
            && d.detail.contains("gym_regular_first")));
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p undone --lib story_map::tests::reconcile_assigns_threads_and_flags_dangling`
Expected: FAIL — `cannot find function reconcile`.

**Step 3: Implement reconciliation**

Add to `src/story_map.rs` (above `tests`). `short_id` strips the `pack::` prefix.

```rust
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
            let by_prefix = rt
                .flag_prefix
                .as_deref()
                .is_some_and(|p| {
                    f.produces.iter().chain(f.requires.iter()).any(|sig| matches_prefix(sig, p))
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
                    dangling.push(DanglingSignal { signal: sig.clone(), set_by: sid.clone() });
                }
            }
            for sig in &f.requires {
                if !producible.contains(sig) {
                    broken.push(BrokenGate { scene: sid.clone(), missing: sig.clone() });
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
            let prefix_used = producible.iter().chain(consumed.iter()).any(|s| matches_prefix(s, prefix));
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
                detail: format!("'{}' set by {} — no scene consumes it (write a follow-up)",
                    d.signal, d.set_by),
            });
        }
    }
    for t in &threads {
        for b in &t.broken {
            write_next.push(WriteNext {
                priority: 2,
                kind: "broken".into(),
                thread: t.name.clone(),
                detail: format!("{} gates on '{}' which no scene produces (fix gate or write producer)",
                    b.scene, b.missing),
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

    StoryMap { threads, orphans, drift, write_next }
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
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p undone --lib story_map`
Expected: PASS (all Task 4–6 tests).

**Step 5: Commit**

```bash
git add src/story_map.rs
git commit -m "feat(story-map): reconcile facts + roadmap into threads/findings/write-next"
```

---

## Task 7: Markdown renderer

Render the `StoryMap` to the human-facing report.

**Files:**
- Modify: `src/story_map.rs`

**Step 1: Write the failing test**

Add to the `tests` module:

```rust
    #[test]
    fn markdown_renders_write_next_and_threads() {
        // BREAKS IF: the report stops surfacing the write-next digest or thread headers.
        let mut scenes: HashMap<String, Arc<SceneDefinition>> = HashMap::new();
        scenes.insert("base::marcus_leverage".into(),
            scene("base::marcus_leverage", r#"gd.setGameFlag("MARCUS_AFFAIR_COOLING");"#));
        let facts = collect_scene_facts(&scenes, &HashMap::new(), &[], &Default::default());
        let roadmap = parse_roadmap("[[thread]]\nname = \"Marcus affair\"\nflag_prefix = \"MARCUS_\"\n").unwrap();
        let existing: HashSet<String> = ["marcus_leverage".to_string()].into_iter().collect();
        let map = reconcile(&facts, &roadmap, &existing);

        let md = render_markdown(&map);
        assert!(md.contains("# Story Map"));
        assert!(md.contains("## Write Next"));
        assert!(md.contains("MARCUS_AFFAIR_COOLING"));
        assert!(md.contains("## Marcus affair"));
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p undone --lib story_map::tests::markdown_renders_write_next_and_threads`
Expected: FAIL — `cannot find function render_markdown`.

**Step 3: Implement the renderer**

Add to `src/story_map.rs`:

```rust
use std::fmt::Write as _;

/// Render the human-facing Markdown report.
pub fn render_markdown(map: &StoryMap) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "# Story Map");
    let _ = writeln!(s);
    let _ = writeln!(s, "> Generated by `cargo run --bin story-map`. Do not edit by hand —");
    let _ = writeln!(s, "> regenerate after content changes. Threads are declared in");
    let _ = writeln!(s, "> `packs/base/roadmap.toml`.");
    let _ = writeln!(s);

    // Write Next digest.
    let _ = writeln!(s, "## Write Next");
    let _ = writeln!(s);
    if map.write_next.is_empty() {
        let _ = writeln!(s, "_Nothing flagged — every thread is closed and reachable._");
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
                let _ = writeln!(s, "  - `{}` set by `{}`, consumed by nothing", d.signal, d.set_by);
            }
        }
        if !t.broken.is_empty() {
            let _ = writeln!(s, "- ⚠ **broken gates:**");
            for b in &t.broken {
                let _ = writeln!(s, "  - `{}` gates on `{}` (produced by nothing)", b.scene, b.missing);
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
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p undone --lib story_map::tests::markdown_renders_write_next_and_threads`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/story_map.rs
git commit -m "feat(story-map): markdown report renderer"
```

---

## Task 8: JSON rendering, the real `build_story_map`, and staleness check

Wire the production path: load the pack, build facts from the real scheduler bindings, reconcile, and provide JSON + staleness helpers.

**Files:**
- Modify: `src/story_map.rs`

**Step 1: Write the failing test**

Add to the `tests` module:

```rust
    #[test]
    fn json_roundtrips_thread_names() {
        // BREAKS IF: the JSON sidecar schema breaks and agents can't read threads.
        let map = StoryMap {
            threads: vec![Thread {
                name: "Jake romance".into(), note: String::new(), scenes: vec![],
                dangling: vec![], broken: vec![], planned: vec![],
            }],
            orphans: vec![], drift: vec![], write_next: vec![],
        };
        let json = render_json(&map).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["threads"][0]["name"], "Jake romance");
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p undone --lib story_map::tests::json_roundtrips_thread_names`
Expected: FAIL — `cannot find function render_json`.

**Step 3: Implement JSON, build, and staleness**

In `src/story_map.rs`, add the imports needed for the production loader at the top of the existing `use` block:

```rust
use std::collections::BTreeSet;
use std::path::Path;
use undone_packs::load_packs;
use undone_scene::{load_scenes, load_schedule};
```

Replace the stub `build_story_map` with the real implementation, and add the JSON + staleness helpers:

```rust
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
    let scheduler = load_schedule(&pack_metas, &registry)
        .map_err(|e| format!("schedule load failed: {e}"))?;
    let bindings = scheduler.bindings();

    // Entry scenes are reachable without a binding.
    let mut entry_scenes: HashSet<String> = HashSet::new();
    entry_scenes.insert(registry.opening_scene().to_string());
    entry_scenes.insert(registry.transformation_scene().to_string());

    let facts = collect_scene_facts(&scenes, &HashMap::new(), &bindings, &entry_scenes);

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
    Ok(reconcile(&facts, &roadmap, &existing_set))
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
```

> `registry.opening_scene()` / `registry.transformation_scene()` are used by
> `validate_pack.rs` (`validate_entry_scene_references(... registry.opening_scene(),
> registry.transformation_scene())`) — confirm their exact return type there. If
> they return `Option<&str>` rather than `&str`, adjust to insert only when `Some`.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p undone --lib story_map`
Expected: PASS (all unit tests).

**Step 5: Commit**

```bash
git add src/story_map.rs
git commit -m "feat(story-map): production build path, JSON sidecar, staleness check"
```

---

## Task 9: The CLI binary

A thin entry point: regenerate the two files, or `--check` for staleness.

**Files:**
- Create: `src/bin/story_map.rs`

**Step 1: Write the binary**

Create `src/bin/story_map.rs`:

```rust
use std::path::PathBuf;
use std::process;

use undone::story_map::{build_story_map, is_up_to_date, render_json, render_markdown};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let check = args.iter().any(|a| a == "--check");

    let packs_dir = PathBuf::from("packs");
    let md_path = PathBuf::from("docs/story-map.md");
    let json_path = PathBuf::from("docs/story-map.json");

    if check {
        match is_up_to_date(&packs_dir, &md_path, &json_path) {
            Ok(true) => {
                println!("story-map: up to date.");
            }
            Ok(false) => {
                eprintln!(
                    "story-map: STALE. Run `cargo run --bin story-map` and commit the result."
                );
                process::exit(1);
            }
            Err(e) => {
                eprintln!("story-map: {e}");
                process::exit(1);
            }
        }
        return;
    }

    let map = match build_story_map(&packs_dir) {
        Ok(map) => map,
        Err(e) => {
            eprintln!("story-map: {e}");
            process::exit(1);
        }
    };

    let md = render_markdown(&map);
    let json = match render_json(&map) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("story-map: {e}");
            process::exit(1);
        }
    };

    if let Err(e) = std::fs::write(&md_path, md) {
        eprintln!("story-map: write {}: {e}", md_path.display());
        process::exit(1);
    }
    if let Err(e) = std::fs::write(&json_path, json) {
        eprintln!("story-map: write {}: {e}", json_path.display());
        process::exit(1);
    }

    let threads = map.threads.len();
    let orphans = map.orphans.len();
    let writes = map.write_next.len();
    println!(
        "story-map: wrote {} and {} ({threads} threads, {writes} write-next items, {orphans} orphans).",
        md_path.display(),
        json_path.display()
    );
    if orphans > 0 {
        println!("  ⚠ {orphans} orphan scene(s) — add them to a thread in packs/base/roadmap.toml.");
    }
}
```

**Step 2: Verify it builds**

Run: `cargo build --bin story-map`
Expected: SUCCESS.

**Step 3: Run it for real**

Run: `cargo run --bin story-map`
Expected: prints a summary line; creates `docs/story-map.md` and `docs/story-map.json`. It MAY report orphans — that is expected before Task 10 tightens the roadmap.

**Step 4: Commit**

```bash
git add src/bin/story_map.rs docs/story-map.md docs/story-map.json
git commit -m "feat(story-map): CLI binary + first generated map"
```

---

## Task 10: Acceptance tests against the real base pack

Prove the tool works end-to-end on real content, that the thread partition is a true partition, and that **every** non-archived scene is claimed (drives the roadmap-authoring work to completion).

**Files:**
- Create: `tests/story_map_acceptance.rs`

**Acceptance Criteria:**
- Running `build_story_map` on the real `packs/` dir succeeds.
- No scene is assigned to two threads (true partition).
- The orphan list is empty (every non-archived scene is claimed by a thread).
- The JSON sidecar round-trips through `serde_json`.
- `is_up_to_date` returns true for the committed files (the generated map is current).

**Step 1: Write acceptance tests**

Create `tests/story_map_acceptance.rs`:

```rust
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use undone::story_map::{build_story_map, is_up_to_date, render_json};

fn packs_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("packs")
}

#[test]
fn builds_on_real_base_pack() {
    // BREAKS IF: the tool can't process the shipping content.
    let map = build_story_map(&packs_dir()).expect("story map should build on base pack");
    assert!(!map.threads.is_empty(), "expected declared threads");
}

#[test]
fn thread_assignment_is_a_true_partition() {
    // BREAKS IF: a scene gets claimed by two threads (assignment double-counts).
    let map = build_story_map(&packs_dir()).unwrap();
    let mut seen: HashSet<String> = HashSet::new();
    for thread in &map.threads {
        for node in &thread.scenes {
            assert!(
                seen.insert(node.id.clone()),
                "scene '{}' appears in more than one thread",
                node.id
            );
        }
    }
}

#[test]
fn every_scene_is_claimed_no_orphans() {
    // BREAKS IF: a non-archived scene is in no roadmap thread. FIX: add its short
    // id to the right thread's `scenes` list in packs/base/roadmap.toml.
    let map = build_story_map(&packs_dir()).unwrap();
    assert!(
        map.orphans.is_empty(),
        "unclaimed scenes (add to packs/base/roadmap.toml): {:?}",
        map.orphans
    );
}

#[test]
fn json_sidecar_roundtrips() {
    // BREAKS IF: the JSON schema stops being valid/parseable for agents.
    let map = build_story_map(&packs_dir()).unwrap();
    let json = render_json(&map).unwrap();
    let _: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
}

#[test]
fn committed_map_is_up_to_date() {
    // BREAKS IF: docs/story-map.{md,json} drift from the content. FIX: rerun
    // `cargo run --bin story-map` and commit.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fresh = is_up_to_date(
        &packs_dir(),
        &root.join("docs/story-map.md"),
        &root.join("docs/story-map.json"),
    )
    .unwrap();
    assert!(fresh, "regenerate with `cargo run --bin story-map` and commit");
}
```

**Step 2: Run acceptance tests**

Run: `cargo test -p undone --test story_map_acceptance`
Expected: Likely FAIL on `every_scene_is_claimed_no_orphans` first (the skeleton roadmap misses some scenes) and possibly `committed_map_is_up_to_date`.

**Step 3: Resolve orphans, then regenerate**

For each scene listed in the `every_scene_is_claimed_no_orphans` failure, add its short id to the most appropriate thread's `scenes` list in `packs/base/roadmap.toml`. Repeat until:

Run: `cargo run --bin story-map`
Expected: `0 orphans` in the summary line.

Then regenerate the committed outputs (the run above already wrote them) and rerun:

Run: `cargo test -p undone --test story_map_acceptance`
Expected: ALL PASS.

**Step 4: Commit**

```bash
git add tests/story_map_acceptance.rs packs/base/roadmap.toml docs/story-map.md docs/story-map.json
git commit -m "test(story-map): acceptance suite + complete base roadmap (0 orphans)"
```

---

## Task 11: Documentation + workflow wiring

Record the tool so future sessions and the `scene-writer` agent use it (Principle 10).

**Files:**
- Modify: `docs/content-schema.md`
- Modify: `.claude/agents/scene-writer.md`
- Modify: `HANDOFF.md`

**Step 1: Document the tool in `content-schema.md`**

Find the top-level structure section of `docs/content-schema.md` (the pack → schedule → scenes overview). Add a short subsection:

```markdown
## Story map (authoring tool)

`cargo run --bin story-map` regenerates `docs/story-map.{md,json}` — a writer-facing
map of every scene, its thread, its flag/arc dependencies, and **what to write next**
(dangling threads, broken gates, planned scenes). Threads are declared in
`packs/base/roadmap.toml` (authoring-only; the engine never loads it). Run it after
any content change; `cargo run --bin story-map -- --check` fails if the committed map
is stale. The JSON sidecar's `write_next` array is the `scene-writer` agent's input
for self-selecting work.
```

**Step 2: Wire the `scene-writer` agent**

In `.claude/agents/scene-writer.md`, add a step near the top of its working procedure (after it reads the writing guide / character docs):

```markdown
- **Ground the scene in the story map.** Read `docs/story-map.json`. Prefer scenes that
  resolve a `write_next` item for the target thread — especially `dangling` signals (an
  existing scene opened a thread nobody followed) and `planned` entries. State which
  `write_next` item the new scene addresses.
```

**Step 3: Update HANDOFF.md**

Add a new Current State entry at the top of `HANDOFF.md` summarizing the story-map tool, and a Session Log row. Include the regenerate reminder:

```markdown
- **Story-map tool** (`cargo run --bin story-map`) — derives the scene-connectivity
  graph from pack data, reconciles against `packs/base/roadmap.toml`, emits
  `docs/story-map.{md,json}`. `--check` guards staleness. Regenerate after content changes.
```

**Step 4: Verify docs build / nothing references missing paths**

Run: `cargo run --bin story-map -- --check`
Expected: `story-map: up to date.` (exit 0).

**Step 5: Commit**

```bash
git add docs/content-schema.md .claude/agents/scene-writer.md HANDOFF.md
git commit -m "docs(story-map): schema note, scene-writer wiring, handoff"
```

---

## Final Verification

Run the full gate before declaring done:

```bash
cargo fmt --all
cargo test -p undone-scene          # reachability + scheduler additions
cargo test -p undone --lib story_map
cargo test -p undone --test story_map_acceptance
cargo run --bin story-map -- --check   # must print "up to date."
cargo run --bin validate-pack          # unchanged: must still pass
```

Expected: all green; `--check` reports up-to-date; `validate-pack` still "All checks passed."

Then follow `ops:finishing-a-development-branch` to merge.

---

## Self-Review Notes (for the implementer)

- **Import paths are the most likely break.** Three are flagged inline with fallbacks:
  `source_set_game_flags`/`source_advance_arcs` re-export location (Task 4), and
  `registry.opening_scene()`/`transformation_scene()` return type (Task 8). Resolve by
  reading the referenced existing call site in `src/validate_pack.rs` / `script/mod.rs`.
- **`load_scenes` is non-recursive over `_archive`** (the runtime loader skips
  underscore dirs), so archived scenes never enter `scenes` — the partition covers only
  shipping content, as intended.
- **`goto` targets** are full `pack::id` strings in content; status uses them as-is
  against the `facts` keys (also full ids), so no normalization is needed for the
  goto-target reachability check.
- **Determinism:** all ordering is id-stable (sorts by short id), so regenerated output
  is byte-stable run to run — required for `--check` to be meaningful.
```
