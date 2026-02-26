# Sprint 1: "The Engine Works" — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix all engine bugs that cause silent failures, make all opening arc scenes reachable, and ensure load-time validation catches content errors before the game runs.

**Architecture:** TDD throughout — every bug fix starts with a failing test. Six batches: dead code cleanup, data correctness, validation gaps, schedule reachability, engine safety, integration test.

**Tech Stack:** Rust workspace (7 crates), TOML pack data, custom expression parser, minijinja templates.

**Tests before this plan:** 208 passing.

---

## Batch 0: Dead Code Cleanup

Low-risk removals. Shrinks the codebase before we modify it.

### Task 0.1: Remove `default_slot` (stale after arc-system refactor)

The arc-system refactor replaced `default_slot`-based dispatch with `pick_next()`. The field
is parsed from pack.toml but never consumed by any code outside the packs crate.

**Files:**
- Modify: `packs/base/pack.toml:8` — delete `default_slot = "free_time"`
- Modify: `crates/undone-packs/src/manifest.rs:20` — remove `default_slot` field
- Modify: `crates/undone-packs/src/manifest.rs:59,74` — update test fixture + assertion
- Modify: `crates/undone-packs/src/registry.rs:34,51,251-265` — remove field, init, setter, getter
- Modify: `crates/undone-packs/src/loader.rs:77-78` — remove `set_default_slot` wiring
- Modify: `crates/undone-packs/src/loader.rs:260-262` — delete `base_pack_has_default_slot` test

**Step 1: Remove from pack.toml**

Delete line 8 (`default_slot = "free_time"`) from `packs/base/pack.toml`.

**Step 2: Remove from manifest.rs**

Delete the `pub default_slot: Option<String>` field from the `PackManifest` struct. Update
the test `parses_pack_toml` — remove the `default_slot` field from the TOML fixture and
remove the assertion `assert_eq!(m.default_slot, ...)`.

**Step 3: Remove from registry.rs**

Delete the `default_slot: Option<String>` field from `PackRegistry`. Remove the
`set_default_slot()` and `default_slot()` methods. Update `new()` to not initialize
the field.

**Step 4: Remove from loader.rs**

Delete the `set_default_slot` call in the pack loading function. Delete the
`base_pack_has_default_slot` test.

**Step 5: Run tests**

Run: `cargo test --workspace`
Expected: All tests pass (one test deleted, no new failures).

**Step 6: Commit**

```
chore: remove dead default_slot field — replaced by pick_next()
```

---

### Task 0.2: Remove other dead code

**Files:**
- Modify: `crates/undone-domain/src/enums.rs:163-166` — delete `has_before_life()`
- Modify: `crates/undone-ui/src/char_creation.rs` — replace `has_before_life()` calls with `was_transformed()`
- Modify: `crates/undone-scene/Cargo.toml:15` — remove `anyhow` dependency
- Modify: `crates/undone-ui/src/lib.rs:136-137` — check if `From<&NpcCore>` impl is dead; if so, remove

**Step 1: Replace `has_before_life()` with `was_transformed()`**

In `crates/undone-domain/src/enums.rs`, delete the `has_before_life()` method (lines 163-166).
It is a one-line alias for `was_transformed()`.

In `crates/undone-ui/src/char_creation.rs`, find both call sites (around lines 1270 and 1323)
and replace `.has_before_life()` with `.was_transformed()`.

**Step 2: Remove unused `anyhow` dependency**

Delete `anyhow = { workspace = true }` from `crates/undone-scene/Cargo.toml`. No source file
in undone-scene imports anyhow.

**Step 3: Check `From<&NpcCore>` for NpcSnapshot**

In `crates/undone-ui/src/lib.rs`, check if the `impl From<&NpcCore> for NpcSnapshot` (around
line 136) is called anywhere. Grep for `NpcSnapshot::from` and `.into()` on NpcCore. If dead,
remove the impl. If alive, leave it.

**Step 4: Run tests + clippy**

Run: `cargo test --workspace && cargo clippy --workspace`
Expected: All pass, zero warnings.

**Step 5: Commit**

```
chore: remove dead code — has_before_life alias, unused anyhow dep
```

---

## Batch 1: Data Correctness

### Task 1.1: Fix scheduler load failure — promote to visible error

Currently, a schedule.toml parse error is swallowed with `eprintln!` and the game starts
with an empty scheduler. The player is stuck with no scenes.

**Files:**
- Modify: `crates/undone-ui/src/game_state.rs:115-121`

**Step 1: Write a test note**

This is a UI-layer error path. The fix is small and follows the pattern already used for
scene load errors. No unit test needed — the pattern is `return failed_pre(...)` which is
tested by the other error paths. Visual verification: break `schedule.toml` syntax
temporarily, run the game, confirm the error is visible.

**Step 2: Fix the error handler**

In `game_state.rs`, replace lines 115-121:

```rust
// BEFORE (silent failure):
let scheduler = match load_schedule(&metas) {
    Ok(s) => s,
    Err(e) => {
        eprintln!("[init] scheduler load error: {e}");
        Scheduler::empty()
    }
};

// AFTER (visible error, consistent with other load failures):
let scheduler = match load_schedule(&metas) {
    Ok(s) => s,
    Err(e) => {
        return failed_pre(
            registry,
            format!("Schedule load error: {e}"),
        );
    }
};
```

Note: `failed_pre` returns a `PreGameState` with `init_error` set, which the UI displays.
Match the exact pattern used for scene load errors around line 101.

**Step 3: Run tests**

Run: `cargo test --workspace`
Expected: All pass (no test covers this error path; it's a UI-layer guard).

**Step 4: Commit**

```
fix: surface scheduler load failure as init error instead of silent empty
```

---

### Task 1.2: Wire `ArcDef.initial_state` in `new_game()`

`initial_state` is declared in `arcs.toml` but never consumed. Custom (non-preset)
characters get no arc state initialization. Fix: if an arc has `initial_state`, seed it
when the player's starting flags include a route that would activate that arc.

**Files:**
- Modify: `crates/undone-packs/src/char_creation.rs` — `new_game()` function
- Test: `crates/undone-packs/src/char_creation.rs` — new test

**Step 1: Write the failing test**

```rust
#[test]
fn new_game_seeds_arc_initial_state_from_starting_arc_states() {
    // Existing starting_arc_states should be seeded
    let (world, _) = make_config_and_world(|config| {
        config.starting_arc_states.push((
            "base::workplace_opening".into(),
            "arrived".into(),
        ));
    });
    assert_eq!(
        world.game_data.arc_state("base::workplace_opening"),
        Some("arrived"),
        "starting_arc_states must be seeded in new_game"
    );
}
```

This test should already pass because `new_game()` iterates `starting_arc_states`. If it does,
the field is already wired for presets. For custom characters, verify that `starting_arc_states`
is empty when no preset is selected — and decide whether `initial_state` should auto-seed.

**Step 2: Decide: wire or remove**

Check: does the current `new_game()` handle `starting_arc_states` correctly for presets?
If yes, the only gap is custom characters. Two options:

**Option A — Auto-seed from `initial_state`:** After the `starting_arc_states` loop in
`new_game()`, iterate `registry.arcs()`. For each arc with `initial_state`, if the arc is
not already seeded (i.e., the starting_arc_states didn't set it), AND the player has a
route flag matching this arc, seed it. This requires knowing which route flag activates
which arc — which is currently only in schedule.toml conditions, not in arcs.toml.

**Option B — Remove `initial_state`:** Since presets handle arc initialization via
`starting_arc_states`, and custom characters don't select arcs, `initial_state` is
redundant. Remove it from `ArcDef` and `arcs.toml`. The field was never consumed.

**Recommendation:** Option B (remove). The arc initialization path is preset-driven.
`initial_state` is an unused design artifact. Removing it follows "no dead fields."

**Step 3: If removing — delete the field**

In `crates/undone-packs/src/data.rs`, remove `pub initial_state: Option<String>` from
`ArcDef`. In `packs/base/data/arcs.toml`, remove both `initial_state = "arrived"` lines.

**Step 4: Run tests**

Run: `cargo test --workspace`
Expected: All pass.

**Step 5: Commit**

```
chore: remove dead ArcDef.initial_state — arc init is preset-driven
```

---

### Task 1.3: Add skill clamp to `SkillIncrease` effect

`SkillIncrease` adds to the skill value with no bounds checking. `SkillDef` declares
`min` and `max` but they are never enforced. Also fix FEMININITY's `min = -100` to `min = 0`
per the design doc (FEMININITY is 0-100+).

**Files:**
- Modify: `packs/base/data/skills.toml:39` — change FEMININITY `min = -100` to `min = 0`
- Modify: `crates/undone-scene/src/effects.rs:190-199` — add clamp
- Modify: `crates/undone-packs/src/registry.rs` — expose `get_skill_def()` method
- Test: `crates/undone-scene/src/effects.rs` — new test

**Step 1: Write the failing test**

Add to `effects.rs` tests:

```rust
#[test]
fn skill_increase_clamps_to_max() {
    let (mut world, registry) = make_world_with_registry();
    let mut ctx = SceneCtx::new();
    // FEMININITY starts at 10, max is 100
    let fem_id = registry.resolve_skill("FEMININITY").unwrap();
    world.player.skills.entry(fem_id).or_insert(SkillValue { value: 10, modifier: 0 });

    let effect = EffectDef::SkillIncrease {
        skill: "FEMININITY".into(),
        amount: 500,
    };
    apply_effect(&effect, &mut world, &mut ctx, &registry).unwrap();

    let val = world.player.skills[&fem_id].value;
    assert_eq!(val, 100, "skill value must be clamped to SkillDef.max");
}

#[test]
fn skill_increase_clamps_to_min() {
    let (mut world, registry) = make_world_with_registry();
    let mut ctx = SceneCtx::new();
    let fem_id = registry.resolve_skill("FEMININITY").unwrap();
    world.player.skills.entry(fem_id).or_insert(SkillValue { value: 10, modifier: 0 });

    let effect = EffectDef::SkillIncrease {
        skill: "FEMININITY".into(),
        amount: -500,
    };
    apply_effect(&effect, &mut world, &mut ctx, &registry).unwrap();

    let val = world.player.skills[&fem_id].value;
    assert_eq!(val, 0, "skill value must be clamped to SkillDef.min");
}
```

Note: these tests need `make_world_with_registry()` that loads the actual pack (or a test
registry with FEMININITY defined). Check existing test helpers — `effects.rs` tests may
already have a helper that loads the base pack. If not, create a minimal one that registers
FEMININITY with min=0, max=100.

**Step 2: Run tests to verify they fail**

Run: `cargo test -p undone-scene -- skill_increase_clamps`
Expected: FAIL — value goes to 510 / -490 instead of 100 / 0.

**Step 3: Expose skill def lookup**

In `crates/undone-packs/src/registry.rs`, add a method to look up the `SkillDef` by
`SkillId`. Check if `get_skill_def(id: &SkillId) -> Option<&SkillDef>` already exists.
If not, add it. The `skill_defs` field should be a `HashMap<SkillId, SkillDef>` or
similar — check the actual structure.

**Step 4: Implement the clamp**

In `effects.rs`, modify the `SkillIncrease` handler:

```rust
EffectDef::SkillIncrease { skill, amount } => {
    let sid = registry
        .resolve_skill(skill)
        .map_err(|_| EffectError::UnknownSkill(skill.clone()))?;
    let entry = world.player.skills.entry(sid).or_insert(SkillValue {
        value: 0,
        modifier: 0,
    });
    entry.value += amount;
    // Clamp to declared bounds
    if let Some(def) = registry.get_skill_def(&sid) {
        entry.value = entry.value.clamp(def.min, def.max);
    }
}
```

**Step 5: Fix FEMININITY min in skills.toml**

Change `min = -100` to `min = 0` in `packs/base/data/skills.toml` for the FEMININITY entry.

**Step 6: Run tests**

Run: `cargo test --workspace`
Expected: All pass including new clamp tests.

**Step 7: Commit**

```
fix: clamp SkillIncrease to SkillDef min/max, correct FEMININITY min to 0
```

---

## Batch 2: Validation Gaps

### Task 2.1: Add stat/skill name validation to `validate_effects`

`AddStat`, `SetStat`, and `FailRedCheck` effects are not validated at load time.
A typo silently fails at runtime.

**Files:**
- Modify: `crates/undone-scene/src/loader.rs` — `validate_effects` function
- Modify: `crates/undone-scene/src/loader.rs` — add `UnknownStat` to error enum if needed
- Test: `crates/undone-scene/src/loader.rs` — new tests

**Step 1: Write failing tests**

```rust
#[test]
fn validate_effects_rejects_unknown_stat_in_add_stat() {
    // Build a scene with an AddStat effect referencing a non-existent stat
    // Call validate_effects
    // Assert error contains "unknown stat" or similar
}

#[test]
fn validate_effects_rejects_unknown_skill_in_fail_red_check() {
    // Build a scene with a FailRedCheck effect referencing a non-existent skill
    // Call validate_effects
    // Assert error
}
```

Check the existing test patterns in `loader.rs` to match the helper setup. The tests
need a registry that does NOT have the referenced stat/skill.

**Step 2: Run tests to verify they fail**

Expected: FAIL — validate_effects currently has `_ => {}` catch-all that skips these.

**Step 3: Add validation arms**

In `validate_effects`, add match arms:

```rust
EffectDef::AddStat { stat, .. } | EffectDef::SetStat { stat, .. } => {
    if registry.get_stat(stat).is_none() {
        errors.push(format!(
            "scene '{}' action '{}': unknown stat '{}'",
            scene_id, action_id, stat
        ));
    }
}
EffectDef::FailRedCheck { skill } => {
    if registry.resolve_skill(skill).is_err() {
        errors.push(format!(
            "scene '{}' action '{}': unknown skill '{}' in FailRedCheck",
            scene_id, action_id, skill
        ));
    }
}
```

Note: Check how `registry.get_stat()` works. The research noted that `stat_defs` only
interns the string — there may not be a proper validated registry. If `get_stat` returns
`Some` for any interned string (not just declared stats), then validate against the
`stat_defs` hashmap directly instead. Read `registry.rs` to confirm.

**Step 4: Run tests**

Run: `cargo test -p undone-scene`
Expected: All pass including new validation tests.

**Step 5: Commit**

```
fix: validate stat/skill names in AddStat, SetStat, FailRedCheck at load time
```

---

### Task 2.2: Wire `validate_trait_conflicts` into `validate-pack`

Currently only called at game startup, not by the offline validation binary.

**Files:**
- Modify: `src/bin/validate_pack.rs`

**Step 1: Add the call**

After `load_packs` returns and before scene loading, add:

```rust
let conflict_errors = registry.validate_trait_conflicts();
if !conflict_errors.is_empty() {
    for e in &conflict_errors {
        eprintln!("  ERROR: {e}");
    }
    error_count += conflict_errors.len();
}
```

Match the exact error reporting pattern already used in `validate_pack.rs`.

**Step 2: Run validate-pack**

Run: `cargo run --bin validate-pack -- packs/base`
Expected: No errors (base pack has no broken conflict references).

**Step 3: Commit**

```
fix: wire validate_trait_conflicts into validate-pack binary
```

---

### Task 2.3: Add category ID validation at load time

`w.inCategory('TYPO')` silently returns false. We need a validation pass over condition
expressions that checks string arguments against the registry.

**Files:**
- Modify: `crates/undone-scene/src/loader.rs` — add `validate_condition_ids` function
- Modify: `crates/undone-expr/src/parser.rs` or `lib.rs` — expose AST walking if needed
- Test: `crates/undone-scene/src/loader.rs` — new test

**Step 1: Assess the Expr AST**

Read `crates/undone-expr/src/parser.rs` to understand the `Expr` enum. We need to walk
the parsed AST and extract all `MethodCall` nodes where the method is `inCategory`,
`beforeInCategory`, `hasTrait`, `getSkill`, etc., and validate their string arguments.

**Step 2: Write the failing test**

```rust
#[test]
fn validate_condition_rejects_unknown_category() {
    // Parse a condition with w.inCategory('NONEXISTENT')
    // Call validate_condition_ids with a registry that has no such category
    // Assert error
}
```

**Step 3: Implement `validate_condition_ids`**

Write a function that walks the `Expr` tree and collects validation errors for:
- `inCategory` / `beforeInCategory` → check `registry.get_category()`
- `hasTrait` → check `registry.resolve_trait()`
- `getSkill` → check `registry.resolve_skill()`

Call this function from the scene loader after parsing each condition string. Also
wire it into `validate-pack`.

**Step 4: Run tests**

Run: `cargo test --workspace`
Expected: All pass. The existing test `inCategory_returns_false_for_unknown_category`
in eval.rs tests runtime behavior and should still pass (runtime fallback is separate
from load-time validation).

**Step 5: Commit**

```
fix: validate category/trait/skill IDs in condition expressions at load time
```

---

## Batch 3: Schedule Reachability

### Task 3.1: Make `workplace_first_clothes` reachable

Root cause: both `workplace_first_clothes` and `workplace_first_day` gate on arc state
`week_one`. `workplace_first_day` has a trigger; `workplace_first_clothes` does not.
Triggers always fire before weighted picks, so `workplace_first_day` always wins.

Fix: split into two sequential arc states. Clothes shopping fires on `week_one` entry,
advances to `clothes_done`. First day fires on `clothes_done`.

**Files:**
- Modify: `packs/base/data/arcs.toml` — add `clothes_done` state to workplace_opening
- Modify: `packs/base/data/schedule.toml` — restructure triggers
- Modify: `packs/base/scenes/workplace_first_clothes.toml` — add `advance_arc` effect
- Test: `crates/undone-scene/src/scheduler.rs` — new test

**Step 1: Write the failing test**

```rust
#[test]
fn pick_next_workplace_first_clothes_reachable_at_week_one() {
    // Build a world with ROUTE_WORKPLACE flag and arc state "week_one"
    // Call pick_next repeatedly
    // Assert workplace_first_clothes is returned (not workplace_first_day)
}
```

**Step 2: Run test to verify it fails**

Expected: FAIL — pick_next returns workplace_first_day because its trigger fires first.

**Step 3: Update arcs.toml**

Add `clothes_done` to the workplace_opening states list:

```toml
[[arc]]
id            = "base::workplace_opening"
states        = ["arrived", "week_one", "clothes_done", "working", "settled"]
```

**Step 4: Update schedule.toml**

```toml
# Clothes shopping — triggers on week_one
[[slot.events]]
scene     = "base::workplace_first_clothes"
condition = "gd.hasGameFlag('ROUTE_WORKPLACE') && gd.arcState('base::workplace_opening') == 'week_one'"
weight    = 0
trigger   = "gd.hasGameFlag('ROUTE_WORKPLACE') && gd.arcState('base::workplace_opening') == 'week_one'"
once_only = true

# First day — triggers on clothes_done (after shopping)
[[slot.events]]
scene     = "base::workplace_first_day"
condition = "gd.hasGameFlag('ROUTE_WORKPLACE') && gd.arcState('base::workplace_opening') == 'clothes_done'"
weight    = 0
trigger   = "gd.hasGameFlag('ROUTE_WORKPLACE') && gd.arcState('base::workplace_opening') == 'clothes_done'"
once_only = true
```

**Step 5: Update workplace_first_clothes.toml**

Find the action(s) that finish the scene. Add an `advance_arc` effect:

```toml
[[actions.effects]]
type  = "advance_arc"
arc   = "base::workplace_opening"
state = "clothes_done"
```

This replaces any existing arc advance in workplace_first_clothes. Check what the scene
currently advances to — it may already advance to `working`, which would need to change
to `clothes_done`.

**Step 6: Verify workplace_first_day still advances to `working`**

Check `workplace_first_day.toml` — its actions should advance the arc to `working`
(which was the original behavior). No change needed there.

**Step 7: Run tests**

Run: `cargo test --workspace`
Expected: All pass including new reachability test.

**Step 8: Commit**

```
fix: make workplace_first_clothes reachable — split week_one into sequential states
```

---

### Task 3.2: Fix `workplace_landlord` trigger — add arc state guard

The landlord trigger uses only `!gd.hasGameFlag('MET_LANDLORD')` with no arc state gate.
It should require `arcState == 'arrived'` to be consistent with all other arc scenes.

**Files:**
- Modify: `packs/base/data/schedule.toml` — update landlord condition + trigger

**Step 1: Update schedule.toml**

```toml
[[slot.events]]
scene     = "base::workplace_landlord"
condition = "gd.hasGameFlag('ROUTE_WORKPLACE') && gd.arcState('base::workplace_opening') == 'arrived' && !gd.hasGameFlag('MET_LANDLORD')"
weight    = 0
trigger   = "gd.hasGameFlag('ROUTE_WORKPLACE') && gd.arcState('base::workplace_opening') == 'arrived' && !gd.hasGameFlag('MET_LANDLORD')"
once_only = true
```

**Step 2: Verify the arc flow**

Check: does `workplace_arrival.toml` advance the arc to `arrived`? If it sets the flag
`ROUTE_WORKPLACE` but does NOT advance to `arrived`, the landlord trigger will never fire.
Trace the arc state flow: arrival → arrived → first_night → week_one → ...

Also check: does `workplace_landlord.toml` advance the arc? It should not — the landlord
is a side scene during the `arrived` state, not an arc progression scene.

**Step 3: Run tests**

Run: `cargo test --workspace`
Expected: All pass.

**Step 4: Commit**

```
fix: workplace_landlord trigger requires arc state 'arrived'
```

---

## Batch 4: Engine Safety

### Task 4.1: Surface effect errors as `EngineEvent::ErrorOccurred`

Currently, effect errors (including `NoActiveMale` from `add_npc_liking`) are caught
in `engine.rs`, printed to stderr, and silently continued. Fix: emit `ErrorOccurred`
so the error is visible to the player / developer.

This is the pragmatic fix. The architectural fix (scene-level NPC binding) is deferred
to a future sprint when NPC relationship infrastructure is built.

**Files:**
- Modify: `crates/undone-scene/src/engine.rs` — both effect error catch sites
- Test: `crates/undone-scene/src/engine.rs` — new test

**Step 1: Write the failing test**

```rust
#[test]
fn effect_error_emits_error_occurred_event() {
    // Build a scene with an AddNpcLiking effect but no active NPC
    // Advance through the scene, choose the action
    // Assert that the returned events include ErrorOccurred
}
```

Check existing engine test patterns for how to construct a test scene with effects.
The key is: no `SetActiveMale` command is sent, so `ctx.active_male` is `None`, so
`AddNpcLiking` returns `Err(NoActiveMale)`.

**Step 2: Run test to verify it fails**

Expected: FAIL — no `ErrorOccurred` event is emitted (error is only `eprintln!`).

**Step 3: Implement the fix**

In `engine.rs`, at both effect error catch sites (around lines 316-320 and 500-504):

```rust
// BEFORE:
if let Err(e) = apply_effect(effect, world, &mut frame.ctx, registry) {
    eprintln!("[scene-engine] effect error: {e}");
}

// AFTER:
if let Err(e) = apply_effect(effect, world, &mut frame.ctx, registry) {
    let msg = format!("[scene-engine] effect error: {e}");
    eprintln!("{msg}");
    events.push(EngineEvent::ErrorOccurred { message: msg });
}
```

Check the exact signature of `ErrorOccurred` — it may take a `String` field or a
different structure. Match the existing usage.

**Step 4: Run tests**

Run: `cargo test -p undone-scene`
Expected: All pass including new test.

**Step 5: Commit**

```
fix: surface effect errors as ErrorOccurred events, not silent eprintln
```

---

## Batch 5: Integration Test

### Task 5.1: Full workplace arc playthrough test

A capstone test that simulates an entire workplace opening arc from start to finish,
verifying all scenes are reachable and complete without errors.

**Files:**
- Create: `crates/undone-scene/src/integration_tests.rs` (or add to existing)
- Test: new integration test

**Step 1: Design the test**

The test should:
1. Create a world with `ROUTE_WORKPLACE` flag and arc state `arrived`
2. Loop: call `scheduler.pick_next()`, start the scene, choose the first available action
3. Track which scene IDs were visited
4. Continue until `pick_next()` returns `None` or the arc reaches `settled`
5. Assert: all expected scenes were visited (arrival, landlord, first_night, first_clothes,
   first_day, work_meeting, evening)
6. Assert: no `ErrorOccurred` events were emitted

**Step 2: Write the test**

```rust
#[test]
fn workplace_arc_full_playthrough() {
    let (registry, scheduler) = load_test_pack();
    let mut world = make_world_with_route("ROUTE_WORKPLACE", "base::workplace_opening", "arrived");
    let mut engine = SceneEngine::new();
    let mut visited = HashSet::new();

    for _ in 0..50 {  // safety cap
        let pick = scheduler.pick_next(&world.game_data, &world.player, &registry);
        let Some(result) = pick else { break };

        if result.once_only {
            world.game_data.set_flag(format!("ONCE_{}", result.scene_id));
        }

        let events = engine.start_scene(&result.scene_id, &world, &registry);
        visited.insert(result.scene_id.clone());

        // Check no errors in intro
        assert!(
            !events.iter().any(|e| matches!(e, EngineEvent::ErrorOccurred { .. })),
            "error in scene {}: {:?}", result.scene_id, events
        );

        // Choose first available action to advance
        // ... (implementation depends on engine API)

        if world.game_data.arc_state("base::workplace_opening") == Some("settled") {
            break;
        }
    }

    let expected = [
        "base::workplace_arrival",
        "base::workplace_landlord",
        "base::workplace_first_night",
        "base::workplace_first_clothes",
        "base::workplace_first_day",
        "base::workplace_work_meeting",
        "base::workplace_evening",
    ];
    for scene in expected {
        assert!(visited.contains(scene), "scene {} was never reached", scene);
    }
}
```

This is a sketch — adapt to the actual engine API. The key is that this test will fail
if any scene is unreachable or if effects produce errors.

**Step 3: Run the test**

Run: `cargo test -p undone-scene -- workplace_arc_full_playthrough`
Expected: PASS (all prior fixes make this possible).

**Step 4: Commit**

```
test: add full workplace arc playthrough integration test
```

---

## Verification

After all batches:

1. `cargo test --workspace` — all pass
2. `cargo clippy --workspace` — zero warnings
3. `cargo run --bin validate-pack -- packs/base` — zero errors
4. `cargo fmt --check` — no formatting issues
5. Count total tests — should be 208 + ~8-12 new = ~216-220

---

## Not in Sprint 1 (explicitly deferred)

- **Scene-level NPC binding** — the correct long-term fix for `add_npc_liking`. Requires
  TOML schema extension + engine NPC resolution. Deferred to Sprint 4/5 with NPC
  relationship infrastructure.
- **`SceneId` newtype** — use everywhere or remove. Sprint 5.
- **Hardcoded content IDs in char_creation.rs** — Sprint 5.
- **Parser recursion depth limit** — theoretical concern, low priority.
- **`stat_defs` as validated map** (I6) — prerequisite for proper stat validation.
  Task 2.1 works around this by checking existence; full fix deferred.
