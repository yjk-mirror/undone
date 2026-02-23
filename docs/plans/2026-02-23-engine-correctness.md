# Engine Correctness & Safety Pass — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix all known correctness, safety, and silent-failure issues in the engine.

**Architecture:** Seven surgical fixes across four crates. No new crates, no new dependencies. Each task is independently testable. Tasks 1–6 are independent of each other; Task 7 depends on Tasks 2 and 6 sharing the scene registry surface.

**Tech Stack:** Rust, floem (scroll API), minijinja, existing workspace crates.

---

### Task 1: Scroll-to-bottom after action selection

The story panel scroll resets to top when new prose is appended because floem
rebuilds the rich_text widget inside `scroll()`, resetting the viewport.

**Files:**
- Modify: `crates/undone-ui/src/lib.rs` (AppSignals struct, ~line 43)
- Modify: `crates/undone-ui/src/lib.rs` (process_events, ~line 334)
- Modify: `crates/undone-ui/src/left_panel.rs` (story_panel, ~line 211–258)

**Step 1: Add scroll_gen signal to AppSignals**

In `crates/undone-ui/src/lib.rs`, add a new signal to `AppSignals`:

```rust
// In the struct definition (~line 43):
pub struct AppSignals {
    pub story: RwSignal<String>,
    pub actions: RwSignal<Vec<ActionView>>,
    pub player: RwSignal<PlayerSnapshot>,
    pub active_npc: RwSignal<Option<NpcSnapshot>>,
    pub prefs: RwSignal<UserPrefs>,
    pub tab: RwSignal<AppTab>,
    pub phase: RwSignal<AppPhase>,
    pub scroll_gen: RwSignal<u64>,   // <-- ADD
}

// In AppSignals::new() (~line 60):
pub fn new() -> Self {
    Self {
        story: RwSignal::new(String::new()),
        actions: RwSignal::new(Vec::new()),
        player: RwSignal::new(PlayerSnapshot::default()),
        active_npc: RwSignal::new(None),
        prefs: RwSignal::new(crate::theme::load_prefs()),
        tab: RwSignal::new(AppTab::Game),
        phase: RwSignal::new(AppPhase::CharCreation),
        scroll_gen: RwSignal::new(0),   // <-- ADD
    }
}
```

**Step 2: Increment scroll_gen in process_events**

In `process_events()` (~line 325), after appending prose:

```rust
EngineEvent::ProseAdded(text) => {
    signals.story.update(|s| {
        if !s.is_empty() {
            s.push_str("\n\n");
        }
        s.push_str(&text);
    });
    signals.scroll_gen.update(|n| *n += 1);  // <-- ADD
}
```

**Step 3: Chain scroll_to_percent on the scroll widget**

In `crates/undone-ui/src/left_panel.rs`, modify the `scroll_area` (~line 253):

```rust
let scroll_gen = signals.scroll_gen;

let scroll_area = scroll(centered_prose)
    .scroll_to_percent(move || {
        scroll_gen.get();  // reactive dependency
        100.0              // always scroll to bottom
    })
    .scroll_style(|s| s.shrink_to_fit())
    .style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.flex_grow(1.0).flex_basis(0.0).background(colors.page)
    });
```

**Step 4: Build and verify**

Run: `cargo check -p undone-ui`
Expected: compiles with no errors.

Run: `cargo test --workspace`
Expected: all 111 tests pass.

**Step 5: Manual verification**

Run the game, choose an action, verify scroll stays at bottom (or near bottom)
after new prose appears. Use screenshot MCP to verify.

**Step 6: Commit**

```bash
git add crates/undone-ui/src/lib.rs crates/undone-ui/src/left_panel.rs
git commit -m "fix: scroll to bottom after prose is appended"
```

---

### Task 2: Scene goto cross-reference validation at load time

A `goto = "base::nonexistent"` in TOML is only caught at runtime with an
`eprintln!`. It should fail at load time.

**Files:**
- Modify: `crates/undone-scene/src/loader.rs` (add `validate_cross_references` fn, add `SceneLoadError::UnknownGotoTarget`)
- Modify: `crates/undone-ui/src/game_state.rs` (call validation after scenes are loaded)
- Test: in `crates/undone-scene/src/loader.rs` (existing tests module)

**Step 1: Write the failing test**

In `crates/undone-scene/src/loader.rs`, add to the `tests` module:

```rust
#[test]
fn validates_goto_cross_references() {
    use std::sync::Arc;
    use crate::types::{SceneDefinition, Action, NextBranch};

    let scene_a = Arc::new(SceneDefinition {
        id: "test::a".into(),
        pack: "test".into(),
        intro_prose: "A".into(),
        actions: vec![Action {
            id: "go".into(),
            label: "Go".into(),
            detail: String::new(),
            condition: None,
            prose: String::new(),
            allow_npc_actions: false,
            effects: vec![],
            next: vec![NextBranch {
                condition: None,
                goto: Some("test::nonexistent".into()),
                finish: false,
            }],
        }],
        npc_actions: vec![],
    });

    let mut scenes = HashMap::new();
    scenes.insert("test::a".into(), scene_a);

    let result = validate_cross_references(&scenes);
    assert!(result.is_err(), "should reject unknown goto target");
}

#[test]
fn valid_goto_passes_cross_reference_check() {
    use std::sync::Arc;
    use crate::types::{SceneDefinition, Action, NextBranch};

    let scene_a = Arc::new(SceneDefinition {
        id: "test::a".into(),
        pack: "test".into(),
        intro_prose: "A".into(),
        actions: vec![Action {
            id: "go".into(),
            label: "Go".into(),
            detail: String::new(),
            condition: None,
            prose: String::new(),
            allow_npc_actions: false,
            effects: vec![],
            next: vec![NextBranch {
                condition: None,
                goto: Some("test::b".into()),
                finish: false,
            }],
        }],
        npc_actions: vec![],
    });

    let scene_b = Arc::new(SceneDefinition {
        id: "test::b".into(),
        pack: "test".into(),
        intro_prose: "B".into(),
        actions: vec![],
        npc_actions: vec![],
    });

    let mut scenes = HashMap::new();
    scenes.insert("test::a".into(), scene_a);
    scenes.insert("test::b".into(), scene_b);

    let result = validate_cross_references(&scenes);
    assert!(result.is_ok(), "valid goto should pass");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p undone-scene validates_goto_cross_references -- --nocapture`
Expected: FAIL — `validate_cross_references` does not exist yet.

**Step 3: Add the error variant and validation function**

In `crates/undone-scene/src/loader.rs`:

Add error variant:
```rust
#[error("unknown goto target '{target}' in scene {scene_id}, action {action_id}")]
UnknownGotoTarget {
    scene_id: String,
    action_id: String,
    target: String,
},
```

Add the public function:
```rust
/// Validate that all `goto` targets in all scenes reference existing scene IDs.
/// Call this after all packs' scenes have been loaded into the combined map.
pub fn validate_cross_references(
    scenes: &HashMap<String, Arc<SceneDefinition>>,
) -> Result<(), SceneLoadError> {
    for (scene_id, def) in scenes {
        for action in &def.actions {
            for branch in &action.next {
                if let Some(ref target) = branch.goto {
                    if !scenes.contains_key(target) {
                        return Err(SceneLoadError::UnknownGotoTarget {
                            scene_id: scene_id.clone(),
                            action_id: action.id.clone(),
                            target: target.clone(),
                        });
                    }
                }
            }
        }
    }
    Ok(())
}
```

Add necessary import at the top of loader.rs:
```rust
use std::sync::Arc;
use crate::types::SceneDefinition;
```

Make sure to also export `validate_cross_references` from `crates/undone-scene/src/lib.rs`.

**Step 4: Call validation in game_state.rs**

In `crates/undone-ui/src/game_state.rs`, add the import:
```rust
use undone_scene::loader::validate_cross_references;
```

After scenes are loaded (~line 74–80), add:
```rust
// Validate cross-references between scenes
if let Err(e) = validate_cross_references(&scenes) {
    let msg = format!("Scene validation error: {e}");
    eprintln!("[init] {msg}");
    return PreGameState {
        registry,
        scenes,
        scheduler,
        rng,
        init_error: Some(msg),
    };
}
```

**Step 5: Run tests**

Run: `cargo test -p undone-scene -- --nocapture`
Expected: both new tests pass, all existing tests still pass.

Run: `cargo test --workspace`
Expected: all tests pass.

**Step 6: Commit**

```bash
git add crates/undone-scene/src/loader.rs crates/undone-scene/src/lib.rs crates/undone-ui/src/game_state.rs
git commit -m "fix: validate scene goto cross-references at load time"
```

---

### Task 3: Scene stack depth guard

A scene cycle (`A goto B, B goto A`) causes unbounded stack growth → OOM.

**Files:**
- Modify: `crates/undone-scene/src/engine.rs` (start_scene, ~line 137)
- Test: in `crates/undone-scene/src/engine.rs` (tests module)

**Step 1: Write the failing test**

In the tests module of `engine.rs`:

```rust
#[test]
fn stack_overflow_emits_error_and_finishes() {
    // Build two scenes that goto each other in a cycle
    let scene_a = SceneDefinition {
        id: "test::a".into(),
        pack: "test".into(),
        intro_prose: "A".into(),
        actions: vec![Action {
            id: "go".into(),
            label: "Go".into(),
            detail: String::new(),
            condition: None,
            prose: String::new(),
            allow_npc_actions: false,
            effects: vec![],
            next: vec![NextBranch {
                condition: None,
                goto: Some("test::b".into()),
                finish: false,
            }],
        }],
        npc_actions: vec![],
    };
    let scene_b = SceneDefinition {
        id: "test::b".into(),
        pack: "test".into(),
        intro_prose: "B".into(),
        actions: vec![Action {
            id: "go".into(),
            label: "Go".into(),
            detail: String::new(),
            condition: None,
            prose: String::new(),
            allow_npc_actions: false,
            effects: vec![],
            next: vec![NextBranch {
                condition: None,
                goto: Some("test::a".into()),
                finish: false,
            }],
        }],
        npc_actions: vec![],
    };

    let mut scenes = HashMap::new();
    scenes.insert("test::a".into(), Arc::new(scene_a));
    scenes.insert("test::b".into(), Arc::new(scene_b));
    let mut engine = SceneEngine::new(scenes);
    let mut world = make_world();
    let registry = PackRegistry::new();

    // Start scene A. When the player chooses "go", it goes to B.
    // When the engine processes B (auto-action with single next branch),
    // we need to trigger the cycle. In practice, evaluate_next pops and
    // calls start_scene, so let's just start A and choose "go" repeatedly.
    engine.send(EngineCommand::StartScene("test::a".into()), &mut world, &registry);
    engine.drain();

    // Choose "go" — triggers A→B→A→B→... cycle via evaluate_next.
    // The stack guard should stop it.
    engine.send(EngineCommand::ChooseAction("go".into()), &mut world, &registry);
    let events = engine.drain();

    // Should NOT have 50+ ProseAdded events (unbounded recursion).
    // Should have SceneFinished (the guard stops the cycle).
    let prose_count = events.iter().filter(|e| matches!(e, EngineEvent::ProseAdded(_))).count();
    assert!(prose_count <= 33, "stack guard should limit recursion, got {prose_count} prose events");
    assert!(
        events.iter().any(|e| matches!(e, EngineEvent::SceneFinished)),
        "stack overflow should emit SceneFinished"
    );
}
```

**Step 2: Run test to verify it fails (hangs or OOMs)**

This test will hang/OOM without the guard. Run with a timeout:
Run: `cargo test -p undone-scene stack_overflow -- --nocapture` (Ctrl+C after 5s if it hangs)

**Step 3: Add the guard**

In `crates/undone-scene/src/engine.rs`, add constant near the top (~line 18):

```rust
/// Maximum scene stack depth. Prevents infinite loops from goto cycles.
const MAX_STACK_DEPTH: usize = 32;
```

In `start_scene()` (~line 137), add the guard at the top of the function:

```rust
fn start_scene(&mut self, id: String, world: &World, registry: &PackRegistry) {
    // Stack depth guard — prevent goto cycles from causing unbounded growth
    if self.stack.len() >= MAX_STACK_DEPTH {
        eprintln!("[scene-engine] stack overflow: depth {} reached starting '{id}'", self.stack.len());
        self.events.push_back(EngineEvent::ProseAdded(
            format!("[Engine error: scene stack overflow (depth {}) — possible cycle involving '{id}']", MAX_STACK_DEPTH),
        ));
        // Clear the entire stack and finish cleanly
        self.stack.clear();
        self.events.push_back(EngineEvent::NpcActivated(None));
        self.events.push_back(EngineEvent::SceneFinished);
        return;
    }

    let def = match self.scenes.get(&id) {
        // ... existing code ...
```

**Step 4: Run tests**

Run: `cargo test -p undone-scene -- --nocapture`
Expected: all tests pass including the new one.

**Step 5: Commit**

```bash
git add crates/undone-scene/src/engine.rs
git commit -m "fix: add scene stack depth guard to prevent goto cycles"
```

---

### Task 4: Fix NPC personality rendering

The sidebar shows `PersonalityId(Spur { idx: ... })` instead of the personality name.

**Files:**
- Modify: `crates/undone-packs/src/registry.rs` (add `personality_name` method)
- Modify: `crates/undone-ui/src/lib.rs` (NpcSnapshot creation in process_events, ~line 346)
- Modify: `crates/undone-scene/src/engine.rs` (NpcActivatedData, ~line 50–58)
- Test: in `crates/undone-packs/src/registry.rs`

**Step 1: Write the failing test**

In `crates/undone-packs/src/registry.rs`, add to tests:

```rust
#[test]
fn personality_name_returns_string() {
    let mut reg = PackRegistry::new();
    let id = reg.intern_personality("ROMANTIC");
    assert_eq!(reg.personality_name(id), "ROMANTIC");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p undone-packs personality_name -- --nocapture`
Expected: FAIL — method doesn't exist.

**Step 3: Add `personality_name` to PackRegistry**

In `crates/undone-packs/src/registry.rs`, add:

```rust
/// Resolve a PersonalityId back to its string name.
pub fn personality_name(&self, id: PersonalityId) -> &str {
    self.rodeo.resolve(&id.0)
}
```

**Step 4: Run test**

Run: `cargo test -p undone-packs personality_name -- --nocapture`
Expected: PASS.

**Step 5: Update NpcActivatedData to carry the resolved name**

In `crates/undone-scene/src/engine.rs`, change the `personality` field in
`NpcActivatedData` (~line 54) from `PersonalityId` to `String`:

```rust
pub struct NpcActivatedData {
    pub name: String,
    pub age: undone_domain::Age,
    pub personality: String,   // <-- changed from PersonalityId
    pub relationship: undone_domain::RelationshipStatus,
    pub pc_liking: undone_domain::LikingLevel,
    pub pc_attraction: undone_domain::AttractionLevel,
}
```

The `From<&NpcCore>` impl can't resolve the name without a registry. Change it to
a constructor that takes the registry:

```rust
impl NpcActivatedData {
    pub fn from_npc(npc: &undone_domain::NpcCore, registry: &PackRegistry) -> Self {
        Self {
            name: npc.name.clone(),
            age: npc.age,
            personality: registry.personality_name(npc.personality).to_owned(),
            relationship: npc.relationship.clone(),
            pc_liking: npc.pc_liking,
            pc_attraction: npc.pc_attraction,
        }
    }
}
```

Remove the old `From<&NpcCore>` impl. Update the two call sites in `engine.rs`
that create `NpcActivatedData` (SetActiveMale ~line 109, SetActiveFemale ~line 120)
to call `NpcActivatedData::from_npc(&npc.core, registry)` instead.

The `send()` method already receives `registry: &PackRegistry`, so it's available.

**Step 6: Update NpcSnapshot in lib.rs**

In `crates/undone-ui/src/lib.rs`, the `process_events` function creates
`NpcSnapshot` from `NpcActivatedData` (~line 346). Since `personality` is now a
`String`, change `NpcSnapshot` creation:

```rust
EngineEvent::NpcActivated(data) => {
    signals.active_npc.set(data.as_ref().map(|d| NpcSnapshot {
        name: d.name.clone(),
        age: format!("{}", d.age),
        personality: d.personality.clone(),   // already a String now
        relationship: format!("{}", d.relationship),
        pc_liking: format!("{}", d.pc_liking),
        pc_attraction: format!("{}", d.pc_attraction),
    }));
}
```

**Step 7: Build and test**

Run: `cargo check --workspace`
Run: `cargo test --workspace`
Expected: all tests pass.

**Step 8: Commit**

```bash
git add crates/undone-packs/src/registry.rs crates/undone-scene/src/engine.rs crates/undone-ui/src/lib.rs
git commit -m "fix: show NPC personality name instead of raw Spur index"
```

---

### Task 5: Log condition evaluation errors

Condition eval errors are `.unwrap_or(false)` throughout the engine — silently
hiding broken conditions from content authors.

**Files:**
- Modify: `crates/undone-scene/src/engine.rs` (~lines 218, 244, 323)

**Step 1: Identify all `.unwrap_or(false)` call sites**

There are three in `engine.rs`:
- `emit_actions()` line 218: `eval(expr, world, &frame.ctx, registry).unwrap_or(false)`
- `run_npc_actions()` line 244: `eval(expr, world, &frame.ctx, registry).unwrap_or(false)`
- `evaluate_next()` line 323: `eval(expr, world, &frame.ctx, registry).unwrap_or(false)`

**Step 2: Replace with logging wrapper**

Add a helper at the top of the `impl SceneEngine` block, or as a free function:

```rust
/// Evaluate a condition expression, logging errors and defaulting to false.
/// This preserves the safe default (broken conditions are conservative)
/// while making failures visible during development.
fn eval_condition(
    expr: &undone_expr::parser::Expr,
    world: &World,
    ctx: &SceneCtx,
    registry: &PackRegistry,
    scene_id: &str,
    context: &str,
) -> bool {
    match eval(expr, world, ctx, registry) {
        Ok(val) => val,
        Err(e) => {
            eprintln!(
                "[scene-engine] condition error in scene '{}' ({}): {}",
                scene_id, context, e
            );
            false
        }
    }
}
```

Then replace all three `.unwrap_or(false)` calls:

In `emit_actions()`:
```rust
let visible = match &action.condition {
    Some(expr) => {
        let scene_id = &frame.def.id;
        eval_condition(expr, world, &frame.ctx, registry, scene_id, &format!("action '{}'", action.id))
    }
    None => true,
};
```

In `run_npc_actions()`:
```rust
let eligible = match &na.condition {
    Some(expr) => {
        let scene_id = &frame.def.id;
        eval_condition(expr, world, &frame.ctx, registry, scene_id, &format!("npc_action '{}'", na.id))
    }
    None => true,
};
```

In `evaluate_next()`:
```rust
let condition_passes = match &branch.condition {
    Some(expr) => {
        let frame = self.stack.last().expect("engine stack must not be empty");
        eval_condition(expr, world, &frame.ctx, registry, &frame.def.id, "next branch")
    }
    None => true,
};
```

**Step 3: Build and test**

Run: `cargo check -p undone-scene`
Run: `cargo test -p undone-scene`
Expected: all tests pass. Behavior is identical; only the error path now logs.

**Step 4: Commit**

```bash
git add crates/undone-scene/src/engine.rs
git commit -m "fix: log condition evaluation errors instead of silently swallowing"
```

---

### Task 6: Surface unknown scene ID in StartScene

When `StartScene` is called with an unknown ID, the engine silently returns,
leaving stale actions in the UI.

**Files:**
- Modify: `crates/undone-scene/src/engine.rs` (start_scene, ~line 137–143)
- Test: in `crates/undone-scene/src/engine.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn start_unknown_scene_emits_error_and_finishes() {
    let mut engine = SceneEngine::new(HashMap::new());
    let mut world = make_world();
    let registry = PackRegistry::new();

    engine.send(
        EngineCommand::StartScene("nonexistent::scene".into()),
        &mut world,
        &registry,
    );

    let events = engine.drain();
    assert!(
        events.iter().any(|e| matches!(e, EngineEvent::ProseAdded(s) if s.contains("not found"))),
        "expected error prose for unknown scene"
    );
    assert!(
        events.iter().any(|e| matches!(e, EngineEvent::SceneFinished)),
        "expected SceneFinished for unknown scene"
    );
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p undone-scene start_unknown_scene -- --nocapture`
Expected: FAIL — no events emitted for unknown scene.

**Step 3: Fix start_scene**

In `start_scene()`, change the `None` arm (~line 140–143):

```rust
None => {
    eprintln!("[scene-engine] unknown scene: {id}");
    self.events.push_back(EngineEvent::ProseAdded(
        format!("[Error: scene not found: '{id}']"),
    ));
    self.events.push_back(EngineEvent::SceneFinished);
    return;
}
```

**Step 4: Run tests**

Run: `cargo test -p undone-scene -- --nocapture`
Expected: all tests pass.

**Step 5: Commit**

```bash
git add crates/undone-scene/src/engine.rs
git commit -m "fix: emit visible error when StartScene receives unknown ID"
```

---

### Task 7: Data-driven opening scene and scheduler slot

`"base::rain_shelter"` and `"free_time"` are hardcoded in the UI.

**Files:**
- Modify: `packs/base/pack.toml` (add opening_scene, default_slot)
- Modify: `crates/undone-packs/src/manifest.rs` (PackMeta fields)
- Modify: `crates/undone-packs/src/registry.rs` (store and expose opening_scene/default_slot)
- Modify: `crates/undone-packs/src/loader.rs` (pass values to registry)
- Modify: `crates/undone-ui/src/lib.rs` (~line 173, 182 — read from registry)
- Modify: `crates/undone-ui/src/left_panel.rs` (~line 202 — read from GameState)
- Modify: `crates/undone-ui/src/game_state.rs` (GameState carries values)
- Test: in `crates/undone-packs/src/manifest.rs`, `crates/undone-packs/src/loader.rs`

**Step 1: Write the failing test**

In `crates/undone-packs/src/manifest.rs`, update the existing test:

```rust
#[test]
fn parses_pack_toml_with_game_config() {
    let src = r#"
        [pack]
        id       = "base"
        name     = "Base Game"
        version  = "0.1.0"
        author   = "Undone"
        requires = []
        opening_scene = "base::rain_shelter"
        default_slot  = "free_time"

        [content]
        traits     = "data/traits.toml"
        npc_traits = "data/npc_traits.toml"
        skills     = "data/skills.toml"
        scenes_dir = "scenes/"
    "#;
    let manifest: PackManifest = toml::from_str(src).unwrap();
    assert_eq!(manifest.pack.opening_scene.as_deref(), Some("base::rain_shelter"));
    assert_eq!(manifest.pack.default_slot.as_deref(), Some("free_time"));
}
```

In `crates/undone-packs/src/loader.rs`, add:

```rust
#[test]
fn base_pack_has_opening_scene() {
    let (registry, _) = load_packs(&packs_dir()).unwrap();
    assert_eq!(registry.opening_scene(), Some("base::rain_shelter"));
}

#[test]
fn base_pack_has_default_slot() {
    let (registry, _) = load_packs(&packs_dir()).unwrap();
    assert_eq!(registry.default_slot(), Some("free_time"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p undone-packs opening_scene -- --nocapture`
Expected: FAIL.

**Step 3: Add fields to PackMeta**

In `crates/undone-packs/src/manifest.rs`:

```rust
#[derive(Debug, Deserialize)]
pub struct PackMeta {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub opening_scene: Option<String>,
    #[serde(default)]
    pub default_slot: Option<String>,
}
```

**Step 4: Add to PackRegistry**

In `crates/undone-packs/src/registry.rs`, add fields:

```rust
pub struct PackRegistry {
    rodeo: Rodeo,
    pub trait_defs: HashMap<TraitId, TraitDef>,
    pub npc_trait_defs: HashMap<NpcTraitId, NpcTraitDef>,
    pub skill_defs: HashMap<SkillId, SkillDef>,
    male_names: Vec<String>,
    female_names: Vec<String>,
    opening_scene: Option<String>,    // <-- ADD
    default_slot: Option<String>,     // <-- ADD
}
```

Update `new()`:
```rust
opening_scene: None,
default_slot: None,
```

Add methods:
```rust
/// Set the opening scene ID (from pack manifest).
/// First pack to declare one wins.
pub fn set_opening_scene(&mut self, id: String) {
    if self.opening_scene.is_none() {
        self.opening_scene = Some(id);
    }
}

/// Set the default scheduler slot (from pack manifest).
/// First pack to declare one wins.
pub fn set_default_slot(&mut self, slot: String) {
    if self.default_slot.is_none() {
        self.default_slot = Some(slot);
    }
}

pub fn opening_scene(&self) -> Option<&str> {
    self.opening_scene.as_deref()
}

pub fn default_slot(&self) -> Option<&str> {
    self.default_slot.as_deref()
}
```

**Step 5: Wire in loader**

In `crates/undone-packs/src/loader.rs`, in `load_one_pack()`, after the manifest
is parsed (~line 72), pass values to registry:

```rust
if let Some(ref scene) = manifest.pack.opening_scene {
    registry.set_opening_scene(scene.clone());
}
if let Some(ref slot) = manifest.pack.default_slot {
    registry.set_default_slot(slot.clone());
}
```

**Step 6: Update pack.toml**

In `packs/base/pack.toml`:

```toml
[pack]
id       = "base"
name     = "Base Game"
version  = "0.1.0"
author   = "Undone"
requires = []
opening_scene = "base::rain_shelter"
default_slot  = "free_time"
```

**Step 7: Update GameState to carry the values**

In `crates/undone-ui/src/game_state.rs`, add fields to `GameState`:

```rust
pub struct GameState {
    pub world: World,
    pub registry: PackRegistry,
    pub engine: SceneEngine,
    pub scheduler: Scheduler,
    pub rng: SmallRng,
    pub init_error: Option<String>,
    pub opening_scene: Option<String>,    // <-- ADD
    pub default_slot: Option<String>,     // <-- ADD
}
```

In `start_game()`, populate them from the registry before moving it:

```rust
pub fn start_game(pre: PreGameState, config: CharCreationConfig) -> GameState {
    let PreGameState {
        mut registry,
        scenes,
        scheduler,
        mut rng,
        init_error,
    } = pre;
    let opening_scene = registry.opening_scene().map(|s| s.to_owned());
    let default_slot = registry.default_slot().map(|s| s.to_owned());
    let world = new_game(config, &mut registry, &mut rng);
    let engine = SceneEngine::new(scenes);
    GameState {
        world,
        registry,
        engine,
        scheduler,
        rng,
        init_error,
        opening_scene,
        default_slot,
    }
}
```

Also update `error_game_state()` to include the new fields with `None` values.

**Step 8: Replace hardcoded IDs in lib.rs**

In `crates/undone-ui/src/lib.rs`, in the `AppPhase::InGame` arm (~line 173):

Replace:
```rust
engine.send(
    EngineCommand::StartScene("base::rain_shelter".into()),
    world,
    registry,
);
```

With:
```rust
if let Some(ref scene_id) = gs.opening_scene {
    engine.send(
        EngineCommand::StartScene(scene_id.clone()),
        world,
        registry,
    );
}
```

Replace the `"free_time"` hardcoded slot (~line 182):
```rust
if let Some(scene_id) = scheduler.pick("free_time", world, registry, rng) {
```
With:
```rust
let slot = gs.default_slot.as_deref().unwrap_or("free_time");
if let Some(scene_id) = scheduler.pick(slot, world, registry, rng) {
```

Note: we keep `"free_time"` as a fallback for packs that don't declare `default_slot`.
This is a conservative default, not a magic string — it's documented and explicit.

**Step 9: Replace hardcoded slot in left_panel.rs**

In `crates/undone-ui/src/left_panel.rs`, the `dispatch_action` function (~line 202)
also references `"free_time"`. This function has access to `state: &Rc<RefCell<GameState>>`,
so it can read `gs.default_slot`:

Replace:
```rust
if let Some(scene_id) = scheduler.pick("free_time", world, registry, rng) {
```
With:
```rust
let slot = gs.default_slot.as_deref().unwrap_or("free_time");
if let Some(scene_id) = scheduler.pick(slot, world, registry, rng) {
```

**Step 10: Build and test**

Run: `cargo check --workspace`
Run: `cargo test --workspace`
Expected: all tests pass.

**Step 11: Commit**

```bash
git add packs/base/pack.toml crates/undone-packs/src/manifest.rs crates/undone-packs/src/registry.rs crates/undone-packs/src/loader.rs crates/undone-ui/src/game_state.rs crates/undone-ui/src/lib.rs crates/undone-ui/src/left_panel.rs
git commit -m "refactor: data-driven opening scene and scheduler slot from pack manifest"
```

---

## Execution Order

Tasks 1–6 are independent — they touch different functions and can be done in any
order. Task 7 depends on nothing but is the largest, so do it last.

Recommended: 1, 6, 3, 5, 2, 4, 7 (start with the user-facing bug, then safety,
then correctness, then the refactor).

After all 7 tasks: run `cargo test --workspace`, then update HANDOFF.md.
