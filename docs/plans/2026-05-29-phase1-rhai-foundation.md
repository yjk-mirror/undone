# Phase 1 — Rhai Foundation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use ops:executing-plans to implement this plan task-by-task.
> REQUIRED SUB-SKILL: Use ops:test-driven-development for every code task.

**Goal:** Replace the custom `undone-expr` condition parser AND the closed `EffectDef` enum with embedded Rhai, so conditions and effects become authored scripts — without losing the load-time fail-fast guarantee or changing any observable game behavior.

**Architecture:** Two `rhai::Engine` instances per session (a read-only engine for conditions, a read+write engine for effects). Every authored condition/effect string compiles to a cached `Arc<rhai::AST>` at pack load (the direct analog of today's pre-parsed `Expr`). Game state is exposed to scripts ONLY through a curated set of registered receiver methods (`w`, `gd`, `m`, `f`, `role`, `scene` for reads; `w.*`/`npc(x).*`/`scene.*` mutators for effects) that wrap the existing `eval.rs` accessors and `apply_effect` logic — never by exposing `World` directly. Fail-fast is reconstructed via a mandatory load-time validation pass (compile + strict-variables + an ID dry-run against a probe registry).

**Tech Stack:** Rust, the `rhai` crate (already a dependency of `tools/rhai-mcp-server`), `cargo`, `validate-pack`, the `rhai` MCP server (authoring-time validation), the `playtester` agent (behavior-parity smoke).

**Parent design:** `docs/plans/2026-05-29-rhai-fragment-architecture-design.md` (§4 Rhai foundation, §8 fail-fast).

**This phase is invisible to players.** Success = the game plays IDENTICALLY, every existing test passes, and `validate-pack` still rejects bad content IDs at load. No new player-facing behavior. The fragment model (Phase 2+) builds ON this.

---

## Background the engineer MUST read before starting

Read these in full before Task 1. The whole phase is "do what these already do, but through Rhai."

1. `crates/undone-expr/src/eval.rs` — the `eval()` entry point and the `eval_call_*` dispatch.
   This is the COMPLETE read-method surface scripts must keep (`gd.week()`, `w.hasTrait("X")`,
   `w.getSkill("FEMININITY")`, `m.isPartner()`, `w.checkSkill("CHARM", 50)`, `scene.hasFlag("x")`,
   the physical-attribute accessors, etc.). Every method dispatched here must have a Rhai equivalent.
2. `crates/undone-expr/src/parser.rs` + `lib.rs` — the `Expr` AST and `parse_condition` API that
   Rhai's `compile()` + `Arc<AST>` replaces.
3. `crates/undone-scene/src/effects.rs` — `apply_effect()` (the big `match` at line 132) and the
   `EffectError` enum. The ~33 mutation arms are the COMPLETE write-method surface.
4. `crates/undone-scene/src/types.rs` — every struct with a `condition: Option<Expr>` field
   (`Action`, `NextBranch`, `Thought`, `NarratorVariant`) and the `EffectDef` enum (35 variants,
   lines 104-236). These fields change type in the cutover.
5. `crates/undone-scene/src/loader.rs` — `parse_condition_checked`, `validate_condition_ids`,
   `validate_call_signature`, `validate_effects`. **These functions ARE the fail-fast guarantee.**
   Whatever they do for `Expr` today, the Rhai path must do at load.
6. `crates/undone-scene/src/scheduler.rs` — `ScheduleEvent { condition: Option<Expr>, trigger:
   Option<Expr> }` and the `eval(expr, world, ctx, registry)` call sites in `pick_next`.
7. `tools/rhai-mcp-server/src/` — the existing Rhai validation server (currently `Engine::new()`
   only; Task 11 makes it use the real game engine).

Key constraint (design §4.4, the highest risk): Rhai's `compile()` + `strict_variables` catches
syntax and unknown *identifiers*, but NOT unknown content IDs — `w.hasTrait("TYPpO")` compiles
clean. `validate_condition_ids` is what makes that fail at load today. **You MUST reconstruct it**
(Task 7) or fail-fast silently regresses to runtime.

## File map

- **Create:** `crates/undone-scene/src/script/mod.rs` — module root for the Rhai layer.
- **Create:** `crates/undone-scene/src/script/engine.rs` — `ScriptEngines`, `build_engines()`.
- **Create:** `crates/undone-scene/src/script/context.rs` — the per-call context handle + the
  borrow-bridging mechanism chosen in Task 2.
- **Create:** `crates/undone-scene/src/script/read_api.rs` — `register_read_api()` + receiver
  handles (`W`, `Gd`, `M`, `F`, `Role`, `Scene`).
- **Create:** `crates/undone-scene/src/script/write_api.rs` — `register_write_api()` + effect calls.
- **Create:** `crates/undone-scene/src/script/compiled.rs` — `CompiledScript`, `ScriptError`,
  the load-time `compile_condition` / `compile_effect` gate + the ID dry-run.
- **Modify:** `crates/undone-scene/src/types.rs` — `Option<Expr>` → `Option<CompiledScript>`;
  effect blocks → `Option<CompiledScript>`.
- **Modify:** `crates/undone-scene/src/loader.rs` — route condition/effect compilation through the
  new gate; preserve duplicate-id / cross-ref passes.
- **Modify:** `crates/undone-scene/src/scheduler.rs` — `ScheduleEvent` fields + `pick_next` eval.
- **Modify:** `crates/undone-scene/src/effects.rs` — `apply_effect` becomes "run the effect AST";
  the per-variant logic moves into `write_api.rs` registered fns (reuse the step helpers).
- **Modify:** `crates/undone-scene/Cargo.toml` — add `rhai`.
- **Delete (final task):** `crates/undone-expr/` — once nothing references it.
- **Modify:** `tools/rhai-mcp-server/src/` — validate against the real engine (Task 11).

Follow the existing module style in `undone-scene` (one responsibility per file; the `script/`
submodule keeps the Rhai surface isolated from the engine loop).

---

## Task 1: Add Rhai dependency and the ScriptError type

**Files:**
- Modify: `crates/undone-scene/Cargo.toml`
- Create: `crates/undone-scene/src/script/mod.rs`
- Create: `crates/undone-scene/src/script/compiled.rs`
- Modify: `crates/undone-scene/src/lib.rs` (add `mod script;`)

**Step 1: Pin the same rhai version/features as the tools workspace.**

Check `tools/rhai-mcp-server/Cargo.toml` for the exact `rhai` version. Add to
`crates/undone-scene/Cargo.toml` `[dependencies]` the SAME version, but DECIDE features per design
§4.1: the game is single-threaded, so do NOT enable `sync`. Enable `metadata` (needed for the MCP
server's signature listing in Task 11). Example (match the actual version you found):

```toml
rhai = { version = "1.x", default-features = true, features = ["metadata"] }
```

**Step 2: Create the module skeleton + error type.**

`crates/undone-scene/src/script/mod.rs`:
```rust
pub mod compiled;
pub mod context;
pub mod engine;
pub mod read_api;
pub mod write_api;

pub use compiled::{CompiledScript, ScriptError};
pub use engine::{build_engines, ScriptEngines};
```

`crates/undone-scene/src/script/compiled.rs` (start with just the error + the type; the gate fns
come in Task 7):
```rust
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScriptError {
    #[error("script compile error in {context}: {message}\n  source: {source_text}")]
    Compile { context: String, message: String, source_text: String },
    #[error("unknown content id '{id}' ({kind}) in {context}\n  source: {source_text}")]
    UnknownId { context: String, kind: String, id: String, source_text: String },
    #[error("script runtime error in {context}: {message}")]
    Runtime { context: String, message: String },
}

/// A compiled condition or effect script. The AST is the direct analog of the
/// pre-parsed `undone_expr::Expr` it replaces: compiled once at pack load,
/// evaluated many times at runtime.
#[derive(Debug, Clone)]
pub struct CompiledScript {
    pub ast: Arc<rhai::AST>,
    pub source: String,
}
```

Create stub `context.rs`, `engine.rs`, `read_api.rs`, `write_api.rs` with `// filled in Task N`
comments so `mod script;` compiles. Add `mod script;` to `lib.rs`.

**Step 3: Compile.**

Run: `cargo check -p undone-scene 2>&1 | tail -20`
Expected: compiles (warnings about unused are fine).

**Step 4: Commit.**

```bash
git add crates/undone-scene/Cargo.toml crates/undone-scene/src/script/ crates/undone-scene/src/lib.rs
git commit -m "feat(script): add rhai dep + script module skeleton + ScriptError"
```

---

## Task 2: SPIKE — decide the borrow-bridging mechanism on ONE method

The design (§4.3) left open HOW the borrowed `&World` reaches registered functions: raw-pointer
handles injected into the `Scope`, vs `Rc<RefCell<…>>` shared context. This is the load-bearing
unknown. Resolve it on a single method BEFORE porting 113 of them.

**Files:**
- Modify: `crates/undone-scene/src/script/context.rs`
- Modify: `crates/undone-scene/src/script/engine.rs`
- Test: inline `#[cfg(test)]` in `engine.rs`

**Step 1: Implement BOTH candidate bindings for one read method (`gd.week() -> i64`).**

Candidate A (raw-pointer handle, scope-injected — design's recommended path):
```rust
// context.rs
pub(crate) struct ReadCtx {
    pub world: *const undone_world::World,
}
// SAFETY invariant: a ReadCtx is valid ONLY for the duration of one
// eval_ast_with_scope call on the single-threaded UI thread; it is never stored,
// returned from a registered fn, or moved across threads.
```

Candidate B (`Rc<RefCell<…>>`): a shared context cloned into the closure at registration.

Write a `bench`-style test that evaluates `gd.week() == 1` against a `make_test_world()` with week
set to 1, 10_000 times, for BOTH bindings. Assert correctness and print elapsed time.

**Step 2: Run and decide.**

Run: `cargo test -p undone-scene script::engine::tests::spike_binding -- --nocapture`
Expected: both return `true`; note the timings.

Decision rule: prefer Candidate A (raw-ptr) unless it requires unsafe that can't be confined to one
module with a clear invariant, OR Candidate B is within ~20% on the 10k-eval bench. Write the
decision and reasoning as a doc-comment at the top of `context.rs`. Delete the rejected candidate.

**Step 3: Commit.**

```bash
git add crates/undone-scene/src/script/context.rs crates/undone-scene/src/script/engine.rs
git commit -m "feat(script): spike + decide borrow-bridging mechanism for Rhai handles"
```

> Gate: do not proceed to Task 4 until this decision is committed. The chosen mechanism is the
> pattern every handle in Tasks 4–5 follows.

---

## Task 3: ScriptEngines scaffold (two engines, bounded, strict)

**Files:**
- Modify: `crates/undone-scene/src/script/engine.rs`

**Step 1: Implement `build_engines()`** per design §4.1:
```rust
pub struct ScriptEngines {
    pub cond: rhai::Engine,    // read API only
    pub effect: rhai::Engine,  // read + write API
}

pub fn build_engines() -> ScriptEngines {
    let mut cond = rhai::Engine::new();
    cond.set_strict_variables(true);
    cond.set_max_operations(50_000);
    cond.set_max_expr_depths(64, 64);
    crate::script::read_api::register_read_api(&mut cond);

    let mut effect = rhai::Engine::new();
    effect.set_strict_variables(true);
    effect.set_max_operations(50_000);
    effect.set_max_expr_depths(64, 64);
    crate::script::read_api::register_read_api(&mut effect);
    crate::script::write_api::register_write_api(&mut effect);

    ScriptEngines { cond, effect }
}
```
(Read API is registered on BOTH so effect scripts can branch on reads; write API ONLY on `effect`,
so a mutating call inside a condition is a compile error — design §4.1.)

**Step 2: Test** that `build_engines()` succeeds and that compiling `w.addArousal(1)` FAILS on the
`cond` engine but SUCCEEDS on the `effect` engine (proves the read/write split). Write this test;
it will pass only after Tasks 4–5 register those fns, so mark it `#[ignore]` now with a comment to
un-ignore in Task 6.

**Step 3: Commit.**

```bash
git add crates/undone-scene/src/script/engine.rs
git commit -m "feat(script): two-engine scaffold (read-only cond + read-write effect)"
```

---

## Task 4: Port the READ API (conditions)

**Files:**
- Modify: `crates/undone-scene/src/script/read_api.rs`, `context.rs`

Port EVERY read method dispatched in `undone-expr/src/eval.rs` onto registered Rhai functions,
following the Task-2 pattern. Group by receiver: `W` (player/world reads), `Gd` (game data),
`M`/`F` (active male/female npc), `Role` (role-bound npc), `Scene` (scene-local flags).

**Step 1: Implement the `W` handle and register its methods.** Representative example (design §4.3
shows the shape; replicate for every method):
```rust
#[derive(Clone)]
struct W; // zero-sized; reads the ReadCtx injected into the scope (Task 2 mechanism)

impl W {
    fn has_trait(&mut self, id: &str) -> Result<bool, Box<rhai::EvalAltResult>> {
        with_read_ctx(|w, reg, _ctx| {
            let tid = reg.resolve_trait(id).map_err(|_| rhai_err("unknown trait", id))?;
            Ok(w.player.has_trait(tid))
        })
    }
    fn get_skill(&mut self, id: &str) -> Result<i64, Box<rhai::EvalAltResult>> { /* … */ }
    fn check_skill(&mut self, id: &str, dc: i64) -> Result<bool, Box<rhai::EvalAltResult>> { /* … */ }
    // … one method per eval.rs `w.*` accessor
}

pub fn register_read_api(e: &mut rhai::Engine) {
    e.register_type::<W>()
     .register_fn("hasTrait", W::has_trait)
     .register_fn("getSkill", W::get_skill)
     .register_fn("checkSkill", W::check_skill);
    // … Gd, M, F, Role, Scene likewise
}
```

**Method inventory:** open `eval.rs` and list every method name dispatched in its `eval_call_bool`,
`eval_call_int`, `eval_call_string` (and any string-comparison accessors like `w.getHeight()`).
Port each one, preserving the exact name and argument shape so existing condition strings work
verbatim. Resolver calls (`resolve_trait`/`resolve_skill`/`resolve_stat`/`resolve_arc`/
`resolve_category`) MUST return a Rhai `Err` on unknown id (this is what Task 7's dry-run relies on).

`checkSkill` / `checkSkillRed` read `SceneCtx`'s `RefCell` roll cache via `get_or_roll_skill` —
inject the `&SceneCtx` into the read context alongside `&World`/`&PackRegistry`.

**Step 2: Test** a representative condition end-to-end:
```rust
#[test]
fn rhai_condition_reads_trait_and_skill() {
    let engines = build_engines();
    let mut world = make_test_world();
    // give the player SHY + FEMININITY 12 using existing world mutators
    let ast = engines.cond.compile(r#"w.hasTrait("SHY") && w.getSkill("FEMININITY") < 15"#).unwrap();
    let got = eval_bool(&ast, &engines.cond, &world, &SceneCtx::new(), &registry);
    assert_eq!(got.unwrap(), true);
}
```
Add `eval_bool`/`eval_int`/`eval_string` helpers in `engine.rs` (scope injection per Task 2).

**Step 3: Run** `cargo test -p undone-scene script:: -- --nocapture` → PASS.

**Step 4: Commit** `feat(script): port read API (conditions) to Rhai`.

---

## Task 5: Port the WRITE API (effects)

**Files:**
- Modify: `crates/undone-scene/src/script/write_api.rs`, `context.rs`; reuse the step helpers in
  `effects.rs` (`step_liking`, `step_arousal`, `parse_relationship_status`, etc.).

Per design §4.1 + decision D5 (constrained call-list): effects are a flat statement list of
registered mutator calls — no loops/user-fns needed (branching lives in fragment structure later).
Each registered mutator wraps the corresponding `apply_effect` arm.

**Step 1: Implement a write context** (`&mut World` + `&mut SceneCtx` + `&PackRegistry`) using the
Task-2 mechanism (mutable variant) and register one mutator per `EffectDef` variant:
```rust
// representative — one per the 35 EffectDef variants / apply_effect arms
fn register_write_api(e: &mut rhai::Engine) {
    e.register_fn("addArousal", |w: &mut W, delta: i64| with_write_ctx(|world, ctx, reg| {
        // reuse step_arousal from effects.rs
    }));
    e.register_fn("setGameFlag", |w: &mut W, flag: &str| { /* world.game_data.set_flag */ });
    // npc("m").addLiking(2): register an `npc(id) -> NpcHandle` fn + methods on NpcHandle
    // … cover every EffectDef variant in types.rs:104-236
}
```
Keep the `mutates_persistent_world` distinction (design §5 / save-dirtiness): tag each mutator so
the save layer still knows when persistent state changed. Preserve the existing best-effort error
semantics (collect + `ErrorOccurred`, design §9).

**Step 2: Test** an effect script mutates correctly:
```rust
#[test]
fn rhai_effect_applies_arousal_and_flag() {
    let engines = build_engines();
    let mut world = make_test_world();
    let ast = engines.effect.compile(r#"w.addArousal(1); w.setGameFlag("X");"#).unwrap();
    apply_effect_script(&ast, &engines.effect, &mut world, &mut SceneCtx::new(), &registry).unwrap();
    assert!(world.game_data.has_flag("X"));
    // assert arousal stepped up one level
}
```

**Step 3:** Un-ignore the Task-3 read/write split test; confirm `w.addArousal(1)` FAILS to compile
on `cond` but compiles on `effect`. Run the suite.

**Step 4: Commit** `feat(script): port write API (effects) to Rhai as constrained call-list`.

---

## Task 6: Load-time fail-fast gate (the critical task)

**Files:**
- Modify: `crates/undone-scene/src/script/compiled.rs`

Reconstruct the `validate_condition_ids` guarantee (design §4.4). This is non-negotiable.

**Step 1: Implement `compile_condition` / `compile_effect`:**
```rust
pub fn compile_condition(src: &str, eng: &rhai::Engine, reg: &PackRegistry, ctx: &str)
    -> Result<CompiledScript, ScriptError>
{
    // (a) compile — strict_variables makes syntax + unknown fn/var fail HERE
    let ast = eng.compile(src).map_err(|e| ScriptError::Compile {
        context: ctx.into(), message: e.to_string(), source_text: src.into() })?;
    // (b) ID dry-run: run once against a probe World where resolver fns return Err
    //     on any unknown id; the Err id+kind becomes a load error. (design §4.4)
    dry_run_probe(&ast, eng, reg, ctx, src)?;
    Ok(CompiledScript { ast: Arc::new(ast), source: src.into() })
}
```

**Step 2: Implement `dry_run_probe`** — evaluate the AST against a synthetic probe `World` with a
registry mode where `resolve_*` on an unknown id returns a hard `Err` carrying `(kind, id)`. Map
that to `ScriptError::UnknownId`. Document the rule (design open question, resolved): **content-ID
args MUST be string literals** so a static literal-arg scan covers branches the dry-run doesn't
execute; add a literal-arg AST inspection as the belt-and-suspenders check.

**Step 3: TDD the gate — write the failing test FIRST:**
```rust
#[test]
fn typo_trait_id_fails_at_compile_not_runtime() {
    let engines = build_engines();
    let reg = base_registry();
    let err = compile_condition(r#"w.hasTrait("TYPpO_NOT_A_TRAIT")"#, &engines.cond, &reg, "test")
        .unwrap_err();
    assert!(matches!(err, ScriptError::UnknownId { .. }),
        "unknown trait id must fail at LOAD, got: {err:?}");
}

#[test]
fn valid_condition_compiles() {
    let engines = build_engines();
    let reg = base_registry();
    assert!(compile_condition(r#"w.hasTrait("SHY")"#, &engines.cond, &reg, "test").is_ok());
}
```
Run: expected FAIL (gate not wired) → implement → PASS.

**Step 4: Commit** `feat(script): load-time compile + ID dry-run gate (preserves fail-fast)`.

---

## Task 7: Cutover — route conditions through Rhai

This is the integration point. Do it in ONE coordinated change so the crate compiles; keep the
suite green at the commit.

**Files:**
- Modify: `types.rs` (`Option<Expr>` → `Option<CompiledScript>` on `Action`, `NextBranch`,
  `Thought`, `NarratorVariant`), `scheduler.rs` (`ScheduleEvent` fields + `pick_next` eval),
  `loader.rs` (`parse_condition_checked` → `compile_condition`; keep duplicate-id/cross-ref passes).

**Step 1:** Thread a `&ScriptEngines` through the loader and scheduler load paths (build it once
after `load_packs`, design open question O-engine — build after the registry is final).

**Step 2:** Replace every `Expr` field type and every `eval(expr, …)` call site with
`eval_bool(&script, &engines.cond, …)`. Replace `parse_condition_checked` body with
`compile_condition`. Keep the existing `validate-pack` cross-reference and duplicate-id passes.

**Step 3:** Run the FULL suite + validate-pack:
```
cargo test -p undone-scene 2>&1 | tail -10
cargo run --bin validate-pack 2>&1 | tail -20
```
Expected: all existing condition-driven tests pass; validate-pack still rejects the known
bad-id fixtures (e.g. `load_schedule_rejects_invalid_condition_ids` must still fail the bad pack).

**Step 4: Commit** `refactor(script): cut conditions over from undone-expr to Rhai`.

---

## Task 8: Cutover — route effects through Rhai

**Files:**
- Modify: `types.rs` (effect blocks → `Option<CompiledScript>`), `effects.rs` (`apply_effect`
  becomes "run the effect AST"), `engine.rs` (effect application call sites), `loader.rs`
  (compile effect scripts at load via `compile_effect`).

**Step 1:** Change the TOML effect representation. The current `[[actions.effects]]` table array
becomes a single `effect = "…"` Rhai string field. (Migration of existing scene files is Task 10;
for THIS task, support the new field and keep tests green using new-style fixtures.)

**Step 2:** `apply_effect` becomes `apply_effect_script(&script, &engines.effect, &mut world, …)`.
The per-variant logic now lives in the Task-5 registered mutators.

**Step 3:** Full suite + validate-pack green.

**Step 4: Commit** `refactor(script): cut effects over from EffectDef enum to Rhai call-lists`.

---

## Task 9: Migrate existing content + delete undone-expr

**Files:**
- Modify: all `packs/base/scenes/*.toml` + `packs/base/data/schedule.toml` (conditions are already
  near-Rhai; effect tables → call-list strings).
- Delete: `crates/undone-expr/`, and its workspace + Cargo.toml references.

**Step 1:** Conditions: the existing syntax (`w.hasTrait('X') && …`, single-quote strings) is
already valid Rhai — most need no change. Effects: convert each `[[…effects]]` table to a
`effect = "call(); call();"` string. Write a one-shot migration script (`tools/migrate-effects.mjs`)
OR do it by hand per file; either way `validate-pack` is the gate.

**Step 2:** Remove `undone-expr` from `Cargo.toml` workspace members and every `use undone_expr::…`.
Delete the crate directory.

**Step 3:** `cargo build --workspace` + `cargo test --workspace` + `cargo run --bin validate-pack`
ALL green. The dependency DAG no longer contains `undone-expr`.

**Step 4: Commit** `refactor(script): migrate base pack to Rhai; remove undone-expr crate`.

---

## Task 10: Realign the rhai-mcp-server to the real engine

**Files:**
- Modify: `tools/rhai-mcp-server/src/` (the validator that currently uses bare `Engine::new()`).

**Step 1:** Build the validation engine from the SAME `build_engines()` + a probe registry loaded
from the pack, so `rhai_validate_script` produces the exact unknown-fn / unknown-ID errors the game
loader produces (design §4.1, decision: tool engine == runtime engine). Reuse `gen_fn_signatures`
for the API listing.

**Step 2:** Test that the MCP server rejects `w.hasTrait("NOPE")` and `w.addArousal(1)` in a
condition context, matching the loader.

**Step 3: Commit** `feat(tools): rhai-mcp-server validates against the real game engine`.

---

## Task 11: Acceptance — behavior parity + fail-fast proof

**Acceptance Criteria:**
- Every pre-existing test passes (`cargo test --workspace`), proving conditions/effects behave
  identically through Rhai.
- `validate-pack` loads all 62 scenes + schedule clean AND still rejects a fixture with a typo'd
  content ID at LOAD (fail-fast intact).
- The running game plays identically: Robin opening arc → settled → Jake chain fires, effects
  apply (flags/liking/arousal change), no behavior change vs. pre-Phase-1.

**Files:**
- Create: `crates/undone-scene/tests/rhai_parity.rs` (acceptance test).

**Step 1: Write acceptance tests** that load the real base pack and assert a representative set of
conditions + effects evaluate/apply correctly through Rhai (use recorded expectations, not inline
construction matching the impl — design §10 anti-circular). Include the fail-fast fixture: a temp
pack with `w.hasTrait("TYPO")` must fail `load_packs`/`validate-pack` at load.

**Step 2: Run** `cargo test --workspace 2>&1 | tail -15` → ALL PASS.
Run `cargo run --bin validate-pack 2>&1 | tail -20` → clean + bad-id fixture rejected.

**Step 3: Dispatch the `playtester`** for a behavior-parity smoke: Robin opening arc → settled →
Jake chain, confirming scenes fire and effects apply exactly as before Phase 1 (it should be
indistinguishable). Report any behavior delta as a bug.

**Step 4: Update HANDOFF.md** (Current State + Session Log) and commit
`test(script): Rhai parity + fail-fast acceptance; Phase 1 complete`.

---

## Self-review (completed by plan author)

- **Spec coverage:** Implements design §4 in full — two engines (Task 3), binding surface (Tasks
  2/4/5), `CompiledScript`/AST caching (Tasks 1/6), the load-time dry-run fail-fast gate (Task 6,
  the §4.4 risk), the read/write split that forbids effects-in-conditions (Tasks 3/5), MCP
  realignment (Task 10), and the §10 testing strategy (Task 11). Performance (§4.5): AST-reuse is
  inherent; a bench is the Task-2 spike — no premature memoization.
- **Out of scope (correctly):** fragments, the composer, the check fragment kind, COMPOSURE
  (Phases 2–4). Phase 1 only swaps the *language*, not the content model.
- **Placeholder check:** Bulk method porting (Tasks 4/5) references the EXISTING `eval.rs` /
  `effects.rs` surfaces with the full translation pattern shown — the executor mechanically ports
  named methods that already exist in Rust, the same faithful-to-codebase instruction Phase 0 used
  for test helpers. Not behavioral hand-waving.
- **Risk-first sequencing:** the borrow-bridge spike (Task 2) and the fail-fast gate (Task 6) — the
  two genuine unknowns — are isolated and TDD'd before the cutover (Tasks 7–8) depends on them.
- **Green-at-every-commit:** new infrastructure (Tasks 1–6) is additive and unused until the
  coordinated cutovers (Tasks 7–8); the crate compiles and the suite passes at each commit.
- **Consistency:** `CompiledScript`, `ScriptEngines { cond, effect }`, `build_engines`,
  `compile_condition`, `eval_bool`, `apply_effect_script` names are used consistently across tasks.

## Notes for execution

- Dedicated worktree: `git worktree add ~/.config/ops/worktrees/undone/phase1-rhai -b phase1-rhai`.
  This is a big, multi-file refactor — isolate it. Set `CARGO_TARGET_DIR` to the main `target` to
  avoid a cold build (design/project convention).
- Strong fit for **Workflow execution**: Tasks 4 and 5 (port ~80 read + ~33 write methods) are
  embarrassingly parallel once the Task-2 pattern is fixed — fan out method-group porting, then a
  single verification pass. Tasks 7–8 (cutover) are sequential and must be one agent.
- Phase 2 (vertical slice) builds on this and is BLOCKED on a creative scene spec from the user.
