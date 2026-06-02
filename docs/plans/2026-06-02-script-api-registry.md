# Script API Registry Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use ops:executing-plans to implement this plan task-by-task.
> **Design doc:** `docs/plans/2026-06-02-script-api-registry-design.md` — read it first. This plan implements it.

**Goal:** Replace the three independently-maintained copies of the content-facing scripting method surface (Rhai receivers, the `validate.rs` spec table, the Minijinja snapshot objects) with one declarative `REGISTRY` table that drives all consumers, and add load-time validation of prose templates.

**Architecture:** A `static REGISTRY: &[MethodDescriptor]` is the single source of truth. Each method's logic is a pure accessor `fn` over borrowed engine state. Rhai registration, the static gate, Minijinja prose dispatch, and a new prose load gate are all driven from the table. The Minijinja snapshot is eliminated: prose receiver objects become zero-sized and read live `World` through the existing thread-local `ReadCtxGuard`.

**Tech Stack:** Rust (workspace), `rhai` (sync, metadata), `minijinja` 2.x, `slotmap`, `lasso`. Crate under work: `crates/undone-scene`. Devtools: `tools/minijinja-mcp-server`.

---

## Conventions for this plan

- **Worktree first.** Before Task 1, create the worktree (the executing-plans skill handles this). All work happens in `.worktrees/script-api-registry/`.
- **Cargo commands** (run from the worktree root unless noted):
  - Compile one crate: `cargo check -p undone-scene`
  - Run one test: `cargo test -p undone-scene <test_name> -- --nocolor`
  - Format a file after writing: `cargo fmt -p undone-scene` (or `rustfmt <file>`)
  - Full workspace: `cargo test` (game workspace only; tools are a separate workspace)
- **Library logging:** use `log::warn!`/`log::error!`, never `eprintln!` (engine principle #9).
- **Commit cadence:** every task ends in a commit. Messages are simple, no co-author footer.
- **The mechanical-lift phases (C, D) carry full method tables.** For those phases the *recipe* is identical per `ArgShape`, so each task shows one fully-worked example per shape it touches, then the complete enumerated method list to apply it to. The list + recipe is the complete content — do not invent methods beyond the list, and do not skip any in it.

---

## Phase A — Registry infrastructure (no behavior change)

### Task A1: Neutral value, arg, and error types

**Files:**
- Create: `crates/undone-scene/src/script/api/mod.rs`
- Modify: `crates/undone-scene/src/script/mod.rs` (add `pub mod api;`)

**Step 1: Write the failing test**

Create `crates/undone-scene/src/script/api/mod.rs` with only the test (types come next so it fails to compile):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_value_converts_to_rhai_and_minijinja() {
        let b = ApiValue::Bool(true);
        let i = ApiValue::Int(7);
        let s = ApiValue::Str("hi".to_string());

        // Rhai conversion
        assert!(b.clone().into_dynamic().as_bool().unwrap());
        assert_eq!(i.clone().into_dynamic().as_int().unwrap(), 7);
        assert_eq!(s.clone().into_dynamic().into_string().unwrap(), "hi");

        // Minijinja conversion
        assert_eq!(b.into_minijinja(), minijinja::Value::from(true));
        assert_eq!(i.into_minijinja(), minijinja::Value::from(7i64));
        assert_eq!(s.into_minijinja(), minijinja::Value::from("hi"));
    }
}
```

**Step 2: Run it to confirm it fails**

Run: `cargo test -p undone-scene api_value_converts -- --nocolor`
Expected: FAIL — `ApiValue` not found.

**Step 3: Write the types above the test**

```rust
//! The single-source-of-truth registry for the content-facing scripting API.
//!
//! `REGISTRY` (in `table.rs`) lists every method authors can call in conditions,
//! effect call-lists, and prose. Each entry names a pure accessor fn over borrowed
//! engine state; the Rhai engines (`rhai_bind`), the static gate (`validate.rs`),
//! the Minijinja prose objects (`minijinja_bind`), and the prose load gate
//! (`prose_validate`) are all driven from this one table.

use undone_packs::PackRegistry;
use undone_world::World;

use crate::effects::EffectError;
use crate::scene_ctx::SceneCtx;

/// A value produced by a read accessor, convertible to both script backends.
#[derive(Clone, Debug, PartialEq)]
pub enum ApiValue {
    Bool(bool),
    Int(i64),
    Str(String),
}

impl ApiValue {
    pub fn into_dynamic(self) -> rhai::Dynamic {
        match self {
            ApiValue::Bool(b) => rhai::Dynamic::from_bool(b),
            ApiValue::Int(i) => rhai::Dynamic::from_int(i),
            ApiValue::Str(s) => rhai::Dynamic::from(s),
        }
    }

    pub fn into_minijinja(self) -> minijinja::Value {
        match self {
            ApiValue::Bool(b) => minijinja::Value::from(b),
            ApiValue::Int(i) => minijinja::Value::from(i),
            ApiValue::Str(s) => minijinja::Value::from(s),
        }
    }
}

/// A literal argument as seen by an accessor, borrowed from the call site.
/// (The resolved npc/role ref is supplied as `ApiArg::Str` at index 0 by the
/// adapters; see `Receiver` doc.)
#[derive(Clone, Copy, Debug)]
pub enum ApiArg<'a> {
    Str(&'a str),
    Int(i64),
    Bool(bool),
}

impl<'a> ApiArg<'a> {
    pub fn as_str(&self) -> Option<&'a str> {
        match self { ApiArg::Str(s) => Some(s), _ => None }
    }
    pub fn as_int(&self) -> Option<i64> {
        match self { ApiArg::Int(i) => Some(*i), _ => None }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self { ApiArg::Bool(b) => Some(*b), _ => None }
    }
}

/// Why an accessor failed. Every variant must convert to BOTH a Rhai error and a
/// Minijinja error so the two adapters share one error vocabulary.
#[derive(Clone, Debug)]
pub enum ApiError {
    UnknownId { kind: &'static str, id: String },
    NoActiveNpc { sex: &'static str },
    UnboundRole { role: String },
    BadArgs { method: &'static str },
}

impl ApiError {
    pub fn message(&self) -> String {
        match self {
            ApiError::UnknownId { kind, id } => format!("unknown {kind} '{id}'"),
            ApiError::NoActiveNpc { sex } => format!("no active {sex} NPC in scene context"),
            ApiError::UnboundRole { role } => format!("no NPC bound to role '{role}'"),
            ApiError::BadArgs { method } => format!("bad arguments to '{method}'"),
        }
    }
    pub fn into_rhai(self) -> Box<rhai::EvalAltResult> {
        self.message().into()
    }
    pub fn into_minijinja(self) -> minijinja::Error {
        minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, self.message())
    }
}

/// Accessor signatures. Reads are pure over their borrows EXCEPT `checkSkill`/
/// `checkSkillRed`, which mutate the per-scene roll cache via `SceneCtx` interior
/// mutability — which is exactly why those two are condition-only (see `table.rs`).
pub type ReadFn = fn(&World, &PackRegistry, &SceneCtx, &[ApiArg]) -> Result<ApiValue, ApiError>;
pub type WriteFn = fn(&mut World, &mut SceneCtx, &PackRegistry, &[ApiArg]) -> Result<(), EffectError>;
```

Add `pub mod api;` to `crates/undone-scene/src/script/mod.rs`.

**Step 4: Run the test**

Run: `cargo test -p undone-scene api_value_converts -- --nocolor`
Expected: PASS. (If `from_bool`/`from_int` names differ in the pinned rhai, use `rhai::Dynamic::from(b)` etc. and adjust the assertions to match — verify with `cargo check -p undone-scene` first.)

**Step 5: Format + commit**

```bash
cargo fmt -p undone-scene
git add crates/undone-scene/src/script/api/mod.rs crates/undone-scene/src/script/mod.rs
git commit -m "feat(script): ApiValue/ApiArg/ApiError neutral types for the registry"
```

---

### Task A2: Descriptor, Receiver, ArgShape, Contexts, and the empty REGISTRY

**Files:**
- Create: `crates/undone-scene/src/script/api/table.rs`
- Modify: `crates/undone-scene/src/script/api/mod.rs` (add descriptor types + `pub mod table;` + `lookup`)

**Step 1: Write the failing test**

Append to `api/mod.rs` tests:

```rust
    #[test]
    fn registry_lookup_and_uniqueness() {
        // Every (receiver, name) is unique, and no key is both Read and Write.
        let mut seen = std::collections::HashSet::new();
        for d in table::REGISTRY {
            assert!(
                seen.insert((d.receiver, d.name)),
                "duplicate registry key: {:?}.{}",
                d.receiver, d.name
            );
        }
        // lookup returns None for a nonsense method
        assert!(lookup(Receiver::W, "definitelyNotAMethod").is_none());
    }
```

**Step 2: Run it to confirm it fails**

Run: `cargo test -p undone-scene registry_lookup_and_uniqueness -- --nocolor`
Expected: FAIL — `table`, `Receiver`, `lookup` not found.

**Step 3: Implement descriptor types in `api/mod.rs`**

```rust
use crate::script::validate::IdKind; // reuse the existing kind enum (made pub(crate) in Task F1)

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Receiver { W, Gd, M, F, Role, Scene, Npc }

/// Declarative argument contract. `ArgShape` describes the **source** arg list the
/// static gate counts. For `Receiver::Role` the leading role id IS source-arg 0
/// (counted here). For `Receiver::Npc` the ref comes from the `npc(ref)` constructor
/// and is NOT part of these source args; the adapter injects it as `ApiArg` index 0.
#[derive(Clone, Copy, Debug)]
pub enum ArgShape {
    /// No source args.
    None,
    /// One content-id string literal validated against the registry.
    Id(IdKind),
    /// Content-id string literal (arg0) + one trailing int literal (arg1). e.g. checkSkill.
    IdInt(IdKind),
    /// One int literal. `i8_range` => must fit i8 (step deltas).
    Int { i8_range: bool },
    /// One opaque string (NOT registry-validated): flags, stuff, job titles, refs.
    Str,
    /// One bool literal.
    Bool,
    /// One opaque string (arg0) + one int literal (arg1), neither id-validated.
    /// Only `gd.npcLikingAtLeast(role, threshold)`.
    StrInt,
    /// Two opaque strings (role.hasFlag / role.hasRole: role id + flag/role).
    StrStr,
    /// A leading bool (arg0) plus an optional trailing string (arg1) — overloaded
    /// arity. Only `setVirgin(value)` / `setVirgin(value, "type")`.
    StrOpt,
}

#[derive(Clone, Copy, Debug)]
pub struct Contexts { pub condition: bool, pub effect: bool, pub prose: bool }

impl Contexts {
    pub const COND: Contexts = Contexts { condition: true, effect: false, prose: false };
    pub const READ: Contexts = Contexts { condition: true, effect: false, prose: true };
    pub const WRITE: Contexts = Contexts { condition: false, effect: true, prose: false };
}

#[derive(Clone, Copy)]
pub enum Accessor { Read(ReadFn), Write(WriteFn) }

pub struct MethodDescriptor {
    pub receiver: Receiver,
    pub name: &'static str,
    pub args: ArgShape,
    pub contexts: Contexts,
    pub accessor: Accessor,
}

/// Look up a descriptor by receiver token + method name. O(n) over a small static
/// slice — fine; called at load time and once per script call.
pub fn lookup(receiver: Receiver, method: &str) -> Option<&'static MethodDescriptor> {
    table::REGISTRY.iter().find(|d| d.receiver == receiver && d.name == method)
}

/// Map a receiver source token (`"w"`, `"gd"`, …) to the enum. `npc` is the chained
/// receiver; the free constructor is `("npc","npc")`.
pub fn receiver_from_token(tok: &str) -> Option<Receiver> {
    Some(match tok {
        "w" => Receiver::W,
        "gd" => Receiver::Gd,
        "m" => Receiver::M,
        "f" => Receiver::F,
        "role" => Receiver::Role,
        "scene" => Receiver::Scene,
        "npc" => Receiver::Npc,
        _ => return None,
    })
}

pub mod table;
```

Create `crates/undone-scene/src/script/api/table.rs`:

```rust
//! The single source of truth: every content-facing scripting method.
//!
//! Entries are added by Phase C (reads) and Phase D (writes). KEEP THIS GROUPED BY
//! RECEIVER and in the same order as the source modules so it reads as a manifest.

use super::{Accessor, ArgShape, Contexts, MethodDescriptor, Receiver};
use crate::script::validate::IdKind;
use crate::script::api::{read, write};

pub static REGISTRY: &[MethodDescriptor] = &[
    // Populated in Phase C (reads) and Phase D (writes).
];
```

> The `use ...::{read, write}` line will not compile until Phase C/D create those modules. To keep the tree green now, temporarily comment that `use` line and leave `REGISTRY` empty; uncomment it in Task C1.

**Step 4: Run the test**

Run: `cargo test -p undone-scene registry_lookup_and_uniqueness -- --nocolor`
Expected: PASS (empty registry trivially unique).

**Step 5: Format + commit**

```bash
cargo fmt -p undone-scene
git add crates/undone-scene/src/script/api/
git commit -m "feat(script): MethodDescriptor/Receiver/ArgShape/Contexts + empty REGISTRY"
```

---

## Phase B — Divergence-audit harness FIRST (design §8 step 0)

> This phase runs BEFORE any accessor is lifted. It builds the read/prose equivalence
> test against the **current two-implementation tree**, where it WILL fail on real
> divergences (`getName` at minimum). Those failures are the spec for which behavior
> the unified accessors must adopt. Skipping this is how `getName` silently regresses.

### Task B1: Read/prose equivalence harness over `w`/`gd`/`scene`

**Files:**
- Create: `crates/undone-scene/tests/read_prose_equivalence.rs`

**Step 1: Write the harness**

For every zero-arg `w`/`gd`/`scene` read method, evaluate it through Rhai (condition engine) and through Minijinja (`render_prose`) against the **same** world, and assert equal string renderings. This uses only existing public APIs (`build_engines`, `render_prose`).

```rust
//! Pre-migration divergence audit (design §8 step 0). Against the CURRENT two-impl
//! tree this surfaces every Rhai-vs-prose divergence as a failure to be decided.
//! After the migration (one shared accessor) it becomes the regression guard.

use undone_scene::script::engine::{build_engines, eval_string_for_test}; // helper added in Step 3
use undone_scene::template_ctx::render_prose;
use undone_scene::scene_ctx::SceneCtx;
use undone_world::test_helpers::make_test_world;

/// Render `expr` through prose; returns the rendered string.
fn via_prose(expr: &str) -> String {
    let world = make_test_world();
    let ctx = SceneCtx::new();
    let registry = undone_packs::PackRegistry::new();
    render_prose(&format!("{{{{ {expr} }}}}"), &world, &ctx, &registry)
        .unwrap_or_else(|e| format!("<<prose error: {e}>>"))
}

/// Evaluate `expr` through the Rhai condition/string path; returns the string form.
fn via_rhai(expr: &str) -> String {
    let world = make_test_world();
    let ctx = SceneCtx::new();
    let registry = undone_packs::PackRegistry::new();
    eval_string_for_test(expr, &world, &ctx, &registry)
        .unwrap_or_else(|e| format!("<<rhai error: {e}>>"))
}

/// The zero-arg read surface that BOTH backends should agree on.
const ZERO_ARG_WGD: &[&str] = &[
    "w.getHeight()", "w.getFigure()", "w.getArousal()", "w.getAlcohol()",
    "w.getName()", "w.getRace()", "w.getAge()", "w.pcOrigin()",
    "gd.timeSlot()", // string-returning
    // numeric (compared as strings to keep one assertion form):
    "w.getMoney()", "w.getStress()", "gd.week()", "gd.day()", "gd.desire()",
    // bool:
    "w.isVirgin()", "w.alwaysFemale()", "gd.isWeekday()", "gd.isWeekend()",
];

#[test]
fn w_gd_zero_arg_reads_agree_across_backends() {
    let mut mismatches = Vec::new();
    for expr in ZERO_ARG_WGD {
        let r = via_rhai(expr);
        let p = via_prose(expr);
        if r != p {
            mismatches.push(format!("  {expr}: rhai={r:?} prose={p:?}"));
        }
    }
    assert!(
        mismatches.is_empty(),
        "read/prose divergences (decide the unified value for each):\n{}",
        mismatches.join("\n")
    );
}
```

**Step 2: Run it — EXPECT FAILURES, and record them**

Run: `cargo test -p undone-scene w_gd_zero_arg_reads_agree -- --nocolor`
Expected: This may FAIL (e.g. numeric type rendering `7` vs `7`, or `isWeekday` inlining). **Record every reported mismatch in the design doc's §8-step-0 ledger as a decision.** For each: the unified accessor adopts ONE value; note which and why. (Numeric ones likely already agree once rendered to string; the real ones surface in Task B2 for NPCs.)

**Step 3: Add the test helper to the engine**

The harness needs a stable string-eval entry point. Add to `crates/undone-scene/src/script/engine.rs`:

```rust
/// Test-only: evaluate an expression to its string form via the condition engine,
/// installing a read context. Mirrors how prose method results stringify so the
/// equivalence harness can compare apples to apples.
#[doc(hidden)]
pub fn eval_string_for_test(
    expr: &str,
    world: &World,
    ctx: &SceneCtx,
    registry: &PackRegistry,
) -> Result<String, String> {
    // Compile `expr` as a string expression: bools -> "true"/"false", ints -> decimal,
    // strings -> themselves. Reuse eval_string with a `\`\${expr}\`` cast or to_string().
    // Implementation: wrap as `(`+expr+`).to_string()` is not valid Rhai for all types;
    // instead eval as Dynamic and format. See engine eval_* for the existing pattern.
    eval_dynamic_to_string(expr, world, ctx, registry).map_err(|e| e.to_string())
}
```

Implement `eval_dynamic_to_string` next to the existing `eval_string`/`eval_bool` using the same `ReadCtxGuard::install` + `eval_ast_with_scope` pattern, formatting the resulting `Dynamic`: `Bool→"true"/"false"`, `Int→i.to_string()`, `String→s`. (Match exactly how Minijinja renders: minijinja renders a bool value as `true`/`false`, an i64 as its decimal, a string as itself — confirm against `via_prose` output and align the formatting.)

**Step 4: Re-run; commit the harness even though it may still flag known, decided divergences**

For divergences you have consciously decided (and recorded), add the expression to an allowlist with the decision noted, OR keep the test red and let Phase C/G turn it green by adopting the chosen behavior. Prefer the latter: leave it red, with the failure message naming the decisions. Commit the harness:

```bash
cargo fmt -p undone-scene
git add crates/undone-scene/tests/read_prose_equivalence.rs crates/undone-scene/src/script/engine.rs
git commit -m "test(script): read/prose equivalence harness (pre-migration divergence audit)"
```

---

### Task B2: NPC-surface equivalence + the `getName` divergence (synthetic fixtures)

**Files:**
- Modify: `crates/undone-scene/tests/read_prose_equivalence.rs`

> The live pack has ZERO NPC prose, so these fixtures are necessarily synthetic
> (design §9 caveat). Build a world with an active male whose `display_name` differs
> from `core.name` to expose `getName`.

**Step 1: Write the failing test**

```rust
/// Build a world with an active male NPC whose spawn name and display name differ.
fn world_with_named_male() -> (undone_world::World, SceneCtx) {
    let mut world = make_test_world();
    // Spawn a male with core.name = "Spawny", then set a display name "Theo".
    // (Use the same spawn/test helper the npc write tests use; set display_name.)
    let key = undone_world::test_helpers::spawn_test_male(&mut world, "Spawny");
    world.male_npc_mut(key).core.display_name = Some("Theo".to_string());
    let mut ctx = SceneCtx::new();
    ctx.active_male = Some(key);
    (world, ctx)
}

#[test]
fn get_name_uses_effective_name_not_spawn_name() {
    let (world, ctx) = world_with_named_male();
    let registry = undone_packs::PackRegistry::new();
    // PROSE today returns effective_name ("Theo"); Rhai role.getName returns core.name
    // ("Spawny"). The UNIFIED accessor must adopt "Theo".
    let prose = render_prose(r#"{{ role.getName("ROLE_X") }}"#, &world, &ctx, &registry);
    // After migration this asserts "Theo" for BOTH backends. Pre-migration it documents
    // the divergence. Pin the post-migration expectation here:
    // (role binding setup omitted — use the test helper that binds ROLE_X to `key`.)
    let _ = prose; // see Step 2 note
}
```

**Step 2: Run it; record the decision**

Run: `cargo test -p undone-scene get_name_uses_effective_name -- --nocolor`
Expected (pre-migration): documents that Rhai `role.getName` = `"Spawny"`, prose = `"Theo"`. **Decision (already made in the design): the unified `getName` accessor returns `effective_name()`.** Make this the asserted post-migration expectation. If `spawn_test_male`/role-binding helpers don't exist, add minimal ones to `undone_world::test_helpers` / use the existing npc test scaffolding in `crates/undone-scene/src/set_npc_name_tests.rs` as the pattern.

**Step 3: Commit**

```bash
cargo fmt -p undone-scene
git add crates/undone-scene/tests/read_prose_equivalence.rs
git commit -m "test(script): getName divergence pinned to effective_name (NPC equivalence, synthetic)"
```

---

## Phase C — Lift read accessors (one receiver per task)

> **Recipe (identical for every read method):** move the body of the current Rhai
> `with_read_ctx(|world, reg, ctx| …)` closure into a free `fn name(w, r, c, args) ->
> Result<ApiValue, ApiError>` in `api/read/<receiver>.rs`, wrapping the value in the
> matching `ApiValue` variant and translating the error to `ApiError`. Then add the
> `MethodDescriptor` row to `table.rs`. Do **not** yet delete `read_api/` and do not
> yet wire Rhai/Minijinja to the table — that's Phase E/G. Keep both alive so the
> tree stays green.

> **Worked examples — one per ArgShape (apply the same shape to every method in the
> per-receiver list):**
>
> ```rust
> // ArgShape::None, bool return:
> pub fn is_virgin(w: &World, _r: &PackRegistry, _c: &SceneCtx, _a: &[ApiArg]) -> Result<ApiValue, ApiError> {
>     Ok(ApiValue::Bool(w.player.virgin))
> }
> // ArgShape::None, i64 return:
> pub fn get_money(w: &World, _r: &PackRegistry, _c: &SceneCtx, _a: &[ApiArg]) -> Result<ApiValue, ApiError> {
>     Ok(ApiValue::Int(w.player.money as i64))
> }
> // ArgShape::None, String via Debug:
> pub fn get_height(w: &World, _r: &PackRegistry, _c: &SceneCtx, _a: &[ApiArg]) -> Result<ApiValue, ApiError> {
>     Ok(ApiValue::Str(format!("{:?}", w.player.height)))
> }
> // ArgShape::Id(Trait):
> pub fn has_trait(w: &World, r: &PackRegistry, _c: &SceneCtx, a: &[ApiArg]) -> Result<ApiValue, ApiError> {
>     let id = a[0].as_str().ok_or(ApiError::BadArgs { method: "hasTrait" })?;
>     let tid = r.resolve_trait(id).map_err(|_| ApiError::UnknownId { kind: "trait", id: id.to_string() })?;
>     Ok(ApiValue::Bool(w.player.has_trait(tid)))
> }
> // ArgShape::IdInt(Skill) — checkSkill; NOTE side effect on ctx roll cache:
> pub fn check_skill(w: &World, r: &PackRegistry, c: &SceneCtx, a: &[ApiArg]) -> Result<ApiValue, ApiError> {
>     let id = a[0].as_str().ok_or(ApiError::BadArgs { method: "checkSkill" })?;
>     let dc = a[1].as_int().ok_or(ApiError::BadArgs { method: "checkSkill" })?;
>     // exact body lifted from read_api/player.rs::check_skill (rolls + caches via c)
>     # /* lift verbatim */ unimplemented!()
> }
> // Receiver::Role, ArgShape::Str (role id is arg0):
> pub fn role_get_liking(w: &World, _r: &PackRegistry, c: &SceneCtx, a: &[ApiArg]) -> Result<ApiValue, ApiError> {
>     let role = a[0].as_str().ok_or(ApiError::BadArgs { method: "getLiking" })?;
>     // lift body of read_api/role.rs::get_liking, mapping resolve errors to ApiError::UnboundRole
>     # unimplemented!()
> }
> ```

### Task C1: Lift `w` (player) reads

**Files:**
- Create: `crates/undone-scene/src/script/api/read/mod.rs` (`pub mod player; pub mod game_data; pub mod npc; pub mod role; pub mod scene;` — add modules as each task creates them; create empty stubs now to keep it compiling, or add per-task)
- Create: `crates/undone-scene/src/script/api/read/player.rs`
- Modify: `crates/undone-scene/src/script/api/table.rs` (add `w` rows; uncomment the `use read/write` line)
- Modify: `crates/undone-scene/src/script/api/mod.rs` (`pub mod read;`)

**Step 1: Write a per-method unit test (parity vs the live Rhai impl)**

Add `crates/undone-scene/src/script/api/read/player.rs` tests at the bottom that call each accessor directly against a test world and assert the value matches the corresponding `read_api` method's output. Example:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use undone_world::test_helpers::make_test_world;
    use crate::scene_ctx::SceneCtx;

    #[test]
    fn is_virgin_matches_world() {
        let w = make_test_world();
        let r = undone_packs::PackRegistry::new();
        let c = SceneCtx::new();
        assert_eq!(is_virgin(&w, &r, &c, &[]).unwrap(), ApiValue::Bool(w.player.virgin));
    }
    // …one per non-trivial method; the bulk getX-Debug methods can share a table test.
}
```

**Step 2: Run to confirm fail**

Run: `cargo test -p undone-scene -- read::player --nocolor`
Expected: FAIL — accessors not defined.

**Step 3: Implement every `w` read accessor**

Apply the recipe to ALL of these (source line in `read_api/player.rs` for the body to lift). Group the 30+ `getX` debug-format methods with a shared helper if you like, but each must exist as its own `fn` referenced by the table:

`hasTrait`(Id Trait), `isVirgin`, `isAnalVirgin`, `isDrunk`, `isVeryDrunk`, `isMaxDrunk`, `isSingle`, `isOnPill`, `isPregnant`, `alwaysFemale`, `hasStuff`(Str — NOT id-validated, return false if unresolved), `wasMale`, `wasTransformed`, `hasSmoothLegs`, `checkSkill`(IdInt Skill — **condition-only**, see Contexts below), `checkSkillRed`(IdInt Skill — **condition-only**), `hadTraitBefore`(Id Trait), `inCategory`(Id Category), `beforeInCategory`(Id Category), `getMoney`, `getStress`, `getAnxiety`, `getSkill`(Id Skill — return `world.player.skill(id) as i64`; **unknown id → ApiError::UnknownId**, matching Rhai, NOT the snapshot's silent 0), `composure`, `pcOrigin`, `beforeName`, `beforeRace`, `beforeAge`, `beforeSexuality`, `getName`, `getRace`, `getAge`, `getArousal`, `getAlcohol`, `getHeight`, `getFigure`, `getBreasts`, `getButt`, `getWaist`, `getLips`, `getHairColour`, `getHairLength`, `getEyeColour`, `getSkinTone`, `getComplexion`, `getAppearance`, `getNippleSensitivity`, `getClitSensitivity`, `getPubicHair`, `getNaturalPubicHair`, `getInnerLabia`, `getWetness`, `beforeVoice`, `beforeHeight`, `beforeHairColour`, `beforeEyeColour`, `beforeSkinTone`, `beforePenisSize`, `beforeFigure`.

**Step 4: Add the `w` rows to `table.rs`**

```rust
use super::{Accessor::Read, ArgShape::*, Contexts, Receiver::W, MethodDescriptor as M};
use crate::script::api::read::player as p;
// inside REGISTRY:
M { receiver: W, name: "isVirgin", args: None, contexts: Contexts::READ, accessor: Read(p::is_virgin) },
M { receiver: W, name: "hasTrait", args: Id(IdKind::Trait), contexts: Contexts::READ, accessor: Read(p::has_trait) },
M { receiver: W, name: "checkSkill", args: IdInt(IdKind::Skill), contexts: Contexts::COND, accessor: Read(p::check_skill) },
M { receiver: W, name: "checkSkillRed", args: IdInt(IdKind::Skill), contexts: Contexts::COND, accessor: Read(p::check_skill_red) },
// … one row per method above. checkSkill/checkSkillRed use Contexts::COND; all others Contexts::READ.
```

> **`Contexts` rule for `w` reads:** all are `Contexts::READ` (condition + prose) EXCEPT `checkSkill`/`checkSkillRed` which are `Contexts::COND` (condition only — RNG side effect, design §4.2).

**Step 5: Run accessor tests + uniqueness**

Run: `cargo test -p undone-scene -- read::player --nocolor` then `cargo test -p undone-scene registry_lookup_and_uniqueness -- --nocolor`
Expected: PASS.

**Step 6: Format + commit**

```bash
cargo fmt -p undone-scene
git add crates/undone-scene/src/script/api/
git commit -m "feat(script): lift w (player) read accessors into registry"
```

---

### Task C2: Lift `gd` (game data) reads

Same recipe. Methods (source `read_api/game_data.rs`), all `Contexts::READ`:

`hasGameFlag`(Str), `isWeekday`(None — lift `world.game_data.is_weekday()`, NOT the snapshot's inlined `day<=4`), `isWeekend`(None — `world.game_data.is_weekend()`), `arcStarted`(Str), `npcLikingAtLeast`(StrInt — role string arg0 + threshold int arg1), `week`(Int return), `day`, `desire`, `getStat`(Str — not id-validated, return 0 if absent, matching Rhai), `timeSlot`(String Debug), `getJobTitle`(String clone), `arcState`(Str — return `arc_state(id).unwrap_or("")`), `npcLiking`(Str — role arg, default `"Neutral"`).

> `ArgShape::StrInt` is defined in Task A2; its gate mapping (arity 2, no id validation) is added to `method_spec_from_argshape` in Task F1.

**Commit:** `feat(script): lift gd (game data) read accessors`

---

### Task C3: Lift `m`, `f`, `role`, `scene` reads

Three receivers in one task (they share the NPC resolution pattern). **Preserve the per-receiver sets exactly — do NOT union them.**

- **`m`** (source `read_api/male_npc.rs`), all `Contexts::READ`: `isPartner`, `isFriend`, `isCohabiting`, `isContactable`, `hadOrgasm`, `hasTrait`(Id NpcTrait), `isNpcAttractionOk`, `isNpcAttractionLust`, `isWAttractionOk`, `isNpcLoveCrush`, `isNpcLoveSome`, `isWLoveCrush`, `hasFlag`(Str), `hasRole`(Str), `getLiking`, `getLove`, `getAttraction`, `getBehaviour`. **No active male → `ApiError::NoActiveNpc { sex: "male" }`.**
  - **ADD `getName`** on `m` as `Contexts::READ`, accessor returns `effective_name()` (design decision; Rhai lacked it, prose had it — unify on the prose behavior). Record in `table.rs` comment.
- **`f`** (source `read_api/female_npc.rs`), all `Contexts::READ`: `isPartner`, `isFriend`, `isPregnant`, `isVirgin`, `hasFlag`(Str), `hasRole`(Str), `getLiking`, `getLove`, `getAttraction`, `getBehaviour`. **ADD `getName`** (effective_name). **Do NOT add `hasTrait`, `isCohabiting`, or the attraction/love predicates to `f`** — they are not in the `f` surface and must stay absent (negative test in Phase F).
- **`role`** (source `read_api/role.rs`), all `Contexts::READ`, role id is source-arg 0: `isPartner`(Str), `isFriend`(Str), `isContactable`(Str), `isPregnant`(Str), `isVirgin`(Str), `hadOrgasm`(Str), `hasFlag`(StrStr), `hasRole`(StrStr), `getName`(Str — **lift to `effective_name()`, NOT `core.name`; this is the headline divergence fix**), `getLiking`(Str), `getLove`(Str), `getAttraction`(Str), `getBehaviour`(Str). **Unbound role → `ApiError::UnboundRole`.**
- **`scene`** (source `read_api/scene.rs`): `hasFlag`(Str), `Contexts::READ`.

**Step: the equivalence harness (Task B2) `getName` test must now be satisfiable** by these accessors — but it isn't wired to prose yet (Phase G). For now, add direct unit tests asserting `role_get_name` returns `effective_name()` ("Theo") given the synthetic world.

**Commit:** `feat(script): lift m/f/role/scene read accessors (getName unified on effective_name)`

---

## Phase D — Lift write accessors

> **Recipe:** move each `with_write_ctx(|world, ctx, reg| …)` body into
> `fn name(w: &mut World, c: &mut SceneCtx, r: &PackRegistry, a: &[ApiArg]) ->
> Result<(), EffectError>` in `api/write/<receiver>.rs`. Add `Contexts::WRITE` rows.

### Task D1: Lift `w` writes

Source `write_api/player.rs`. Methods + `ArgShape`:
`changeStress`(Int), `changeMoney`(Int), `changeAnxiety`(Int), `changeComposure`(Int), `addArousal`(Int{i8}), `changeAlcohol`(Int{i8}), `skillIncrease`(Id Skill + int → use `IdInt(Skill)`), `addTrait`(Id Trait), `removeTrait`(Id Trait), `addStuff`(Str), `removeStuff`(Str), `setVirgin`(StrOpt — overloaded: 1 bool arg, or bool+type-string; see Task E2 for double-registration), `setPartner`(Str), `addFriend`(Str).

> `skillIncrease(skill, n)` is id+int. Reuse `IdInt(Skill)` (arg0 id, arg1 int). `addArousal`/`changeAlcohol` are `Int { i8_range: true }`.

Unit test per method: apply to a test world, assert the mutation (e.g. `change_money` then `assert_eq!(world.player.money, …)`). Preserve `EffectError` variants (TraitConflict, UnknownStuff, etc.) exactly.

**Commit:** `feat(script): lift w write accessors`

### Task D2: Lift `gd` + `scene` writes

`gd` (source `write_api/game_data.rs`): `setGameFlag`(Str), `removeGameFlag`(Str), `addStat`(Id Stat + int → `IdInt(Stat)`), `setStat`(IdInt Stat), `setJobTitle`(Str), `addDesire`(Int), `setDesire`(Int), `advanceTime`(Int), `advanceArc`(Id Arc + state → keep the existing arc+state validation; model `ArgShape::Id(Arc)` with the index-1 state validated in the gate as today), `failRedCheck`(Id Skill).
`scene` (source `write_api/scene.rs`): `setFlag`(Str), `removeFlag`(Str).

**Commit:** `feat(script): lift gd + scene write accessors`

### Task D3: Lift `npc(ref).*` writes + the constructor

Source `write_api/npc.rs`. The accessor receives the **resolved ref as `ApiArg::Str` at index 0** (injected by the adapter — Phase E), and the method's own args follow. Methods (`Contexts::WRITE`):
- `npc` constructor — special: arity-1 source (the ref), no mutation. Modeled as `Receiver::Npc` name `"npc"`, `ArgShape::Str`. The adapter handles it as ref-binding, not a table accessor call (see Task E3).
- `addLiking`(Int{i8}), `addLove`(Int{i8}), `addWLiking`(Int{i8}), `setAttraction`(Int{i8}), `setFlag`(Str), `addTrait`(Id NpcTrait), `setRelationship`(Str), `setBehaviour`(Str), `setContactable`(Bool), `addSexualActivity`(Str), `setRole`(Str), `setName`(Str).

> The accessor signature for npc writes: index 0 of `args` is the ref string; the
> method arg (delta/flag/…) is index 1. Each accessor resolves the npc via the
> existing `resolve_npc_ref(ref, world, ctx)` and applies the mutation.

**Commit:** `feat(script): lift npc(ref) write accessors`

---

## Phase E — Rhai registration from the table

### Task E1: Arity-shaped read registrars

**Files:**
- Create: `crates/undone-scene/src/script/api/rhai_bind.rs`
- Modify: `crates/undone-scene/src/script/api/mod.rs` (`pub mod rhai_bind;`)

**Step 1:** Write a test that builds an engine via the new binder and evaluates one method per shape, asserting parity with the old engine for a fixture world (e.g. `w.getHeight()`, `w.hasTrait("SHY")`, `gd.week()`).

**Step 2–3:** Implement registrars. The receiver handles are the existing ZSTs (`W`, `Gd`, `M`, `F`, `Role`, `Scene`) — keep them (they are also used by Minijinja in Phase G as the ZST views, or define fresh ZSTs here; reuse to avoid duplication). For each `MethodDescriptor` whose `accessor` is `Read`, register based on `ArgShape`:

```rust
fn reg_read_0(engine: &mut rhai::Engine, recv_register: impl Fn(&mut rhai::Engine, &str, ...), name: &'static str, f: ReadFn) {
    engine.register_fn(name, move |_this: &mut W| -> Result<rhai::Dynamic, Box<rhai::EvalAltResult>> {
        with_read_ctx(|w, r, c| f(w, r, c, &[]).map_err(ApiError::into_rhai)).map(ApiValue::into_dynamic)
    });
}
// reg_read_1_str (Id/Str): closure |this, a: ImmutableString| → [ApiArg::Str(&a)]
// reg_read_1_int (Int): |this, a: i64| → [ApiArg::Int(a)]
// reg_read_2_str_int (IdInt): |this, a: ImmutableString, b: i64| → [Str, Int]
// reg_read_2_str_str (StrStr, for role.hasFlag/hasRole): |this, a, b| → [Str, Str]
// reg_read_1_str for role's single-arg methods (role id is the one source arg)
```

Because Rhai keys methods on the receiver Rust type, register each name against the correct ZST per `Receiver`. Drive the whole thing from a loop over `REGISTRY` with a `match (d.receiver, d.args)`.

**Step 4–5:** Run parity test; commit `feat(script): Rhai read registration driven by REGISTRY`.

### Task E2: Write registrars + setVirgin double-registration

Same pattern for `Accessor::Write` on the effect engine only. `ArgShape::StrOpt` (`setVirgin`) registers **two** native fns (1-arg and 2-arg) both calling the one accessor (which branches on `args.len()`). Add a parity test for `setVirgin(true)` and `setVirgin(true, "anal")`.

**Commit:** `feat(script): Rhai write registration + setVirgin overload`

### Task E3: NPC chained-call binding + rewire `build_engines`; delete `read_api/` + `write_api/`

**Files:**
- Modify: `crates/undone-scene/src/script/engine.rs` (`build_engines` calls `rhai_bind::register_all` instead of `read_api::register_read_api` / `write_api::register_write_api`)
- Modify: `crates/undone-scene/src/script/api/rhai_bind.rs` (npc constructor + chained dispatch)
- Delete: `crates/undone-scene/src/script/read_api/` and `write_api/` (whole dirs), remove their `mod` lines

The `npc(ref)` free fn returns the `Npc` handle (keep that mechanism); chained methods on `Npc` are registered from the `Receiver::Npc` write rows, with the adapter prepending the handle's ref as `ApiArg::Str` at index 0. Keep `effects.rs::resolve_npc_ref` as-is.

**Step: run the FULL existing Rhai test suite** (`cargo test -p undone-scene`) — the golden `rhai_parity.rs` corpus and all condition/effect tests must pass. This is the Rhai-side behavior-preservation gate.

**Commit:** `refactor(script): drive Rhai engines from REGISTRY; remove read_api/write_api`

---

## Phase F — Static gate on the table

### Task F1: Make `IdKind` shareable; route `validate.rs` through the registry

**Files:**
- Modify: `crates/undone-scene/src/script/validate.rs`

**Step 1:** Make `IdKind` `pub(crate)` (it's referenced by `api/mod.rs`). Add `ArgShape::StrInt` handling if introduced in C2.

**Step 2:** Replace `read_spec`/`write_spec` bodies with registry lookups:

```rust
fn read_spec(receiver: &str, method: &str) -> Option<MethodSpec> {
    let recv = crate::script::api::receiver_from_token(receiver)?;
    let d = crate::script::api::lookup(recv, method)?;
    // Only descriptors valid in conditions are "read methods" for the gate:
    if !d.contexts.condition { return None; }
    Some(method_spec_from_argshape(d.args))
}
fn write_spec(receiver: &str, method: &str) -> Option<MethodSpec> {
    let recv = crate::script::api::receiver_from_token(receiver)?;
    let d = crate::script::api::lookup(recv, method)?;
    if !d.contexts.effect { return None; }
    Some(method_spec_from_argshape(d.args))
}
```

`method_spec_from_argshape` maps each `ArgShape` to the existing `MethodSpec` (arity, `id_arg`, int/i8 constraints) — this is the inverse of the old hand-written table, now derived. Delete the old giant match arms.

**Step 3:** Run the full existing validation tests + load the base pack (`cargo run --bin validate-pack` or the loader tests). All 153 conditions + effect lists must still validate.

**Step 4 — negative surface tests.** Add `crates/undone-scene/src/script/validate.rs` tests:

```rust
#[test] fn f_has_trait_is_unknown_method() { assert!(read_spec("f", "hasTrait").is_none()); }
#[test] fn m_is_pregnant_is_unknown_method() { assert!(read_spec("m", "isPregnant").is_none()); }
#[test] fn check_skill_rejected_in_prose_context() { /* via prose gate, Task H */ }
#[test] fn write_in_condition_is_unknown() { assert!(read_spec("w", "changeMoney").is_none()); }
```

**Commit:** `refactor(script): static gate reads its contract from REGISTRY; negative surface tests`

---

## Phase G — Eliminate the snapshot (Minijinja → ZST + live context)

### Task G1: ZST views with table-driven `call_method`

**Files:**
- Create: `crates/undone-scene/src/script/api/minijinja_bind.rs`
- Modify: `crates/undone-scene/src/script/api/mod.rs` (`pub mod minijinja_bind;`)

**Step 1:** Test: render `{{ w.getHeight() }}` and `{% if w.hasTrait("SHY") %}…` against a live world using the new views, asserting correct output (parity with the old snapshot output).

**Step 2–3:** Implement one ZST per receiver implementing `minijinja::value::Object`:

```rust
pub struct WView;
impl minijinja::value::Object for WView {
    fn call_method(self: &Arc<Self>, _s: &State, method: &str, args: &[minijinja::Value]) -> Result<minijinja::Value, minijinja::Error> {
        let d = lookup(Receiver::W, method).filter(|d| d.contexts.prose)
            .ok_or_else(|| minijinja::Error::new(ErrorKind::UnknownMethod, format!("w has no prose method '{method}'")))?;
        let Accessor::Read(f) = d.accessor else { return Err(/* write in prose */); };
        let api_args = marshal_args(d.args, method, args)?; // Vec<ApiArg> incl. role/npc ref for Role/Npc views
        with_read_ctx(|w, r, c| f(w, r, c, &api_args).map_err(ApiError::into_minijinja)).map(ApiValue::into_minijinja)
    }
}
// GdView, MView, FView, RoleView, SceneView similarly. RoleView marshals the role id
// as ApiArg index 0 from the call's first arg.
```

`marshal_args` converts `&[minijinja::Value]` to `Vec<ApiArg>` per `ArgShape` (string/int/bool extraction), borrowing strings for the duration of the call.

**Step 4–5:** Test passes; commit `feat(script): ZST Minijinja views dispatching via REGISTRY`.

### Task G2: Rewrite `render_prose`; delete snapshot structs; ban state-returning render

**Files:**
- Modify: `crates/undone-scene/src/template_ctx.rs`

**Step 1:** Rewrite `render_prose` to: compute `m`/`f` presence from `ctx.active_male`/`active_female` (bind the ZST view or `Value::UNDEFINED`); bind `w`/`gd`/`scene`/`role` to always-present ZSTs; install `ReadCtxGuard`; `tmpl.render(ctx_map)`. **Use plain `render` only** — add a code comment forbidding `render_and_return_state` (design §6).

```rust
pub fn render_prose(template_str: &str, world: &World, ctx: &SceneCtx, registry: &PackRegistry)
    -> Result<String, minijinja::Error>
{
    let active_male = ctx.active_male.map(|_| Value::from_object(MView)).unwrap_or(Value::UNDEFINED);
    let active_female = ctx.active_female.map(|_| Value::from_object(FView)).unwrap_or(Value::UNDEFINED);
    // SAFETY/INVARIANT: render is synchronous & single-threaded; the guard lives for the
    // whole render call. NEVER switch to render_and_return_state (design §6).
    let _guard = ReadCtxGuard::install(world, registry, ctx);
    let mut env = minijinja::Environment::new();
    env.add_template("prose", template_str)?;
    let tmpl = env.get_template("prose")?;
    tmpl.render(minijinja::context! {
        w => Value::from_object(WView), gd => Value::from_object(GdView),
        scene => Value::from_object(SceneView), role => Value::from_object(RoleView),
        m => active_male, f => active_female,
    })
}
```

**Step 2:** Delete `PlayerCtx`, `GameDataCtx`, `NpcCtx`, `RoleLookupCtx`, `SceneCtxView` and all their `call_method` impls and snapshot construction.

**Step 3:** Run `read_prose_equivalence.rs` (Phase B) — it must now **PASS** (one shared accessor; `getName` returns `effective_name()` on both sides). Run the npc-name tests (`set_npc_name_tests.rs`) and the existing intro-render parity test.

**Step 4 — presence acceptance tests:**

```rust
#[test] fn if_m_truthy_when_male_bound_falsy_when_not() { /* render "{% if m %}Y{% else %}N{% endif %}" both ways */ }
#[test] fn m_method_with_no_male_errors_loud() { /* render "{{ m.getName() }}" with no male → Err */ }
#[test] fn unbound_role_lookup_errors() { /* render r#"{{ role.getName("NOPE") }}"# → Err */ }
```

**Commit:** `refactor(script): eliminate Minijinja snapshot; prose reads live World via ZST views`

---

## Phase H — Prose load gate

### Task H1: Single-quote-aware tokenizer + region extraction

**Files:**
- Modify: `crates/undone-scene/src/script/validate.rs` (tokenizer: accept `'…'` strings)
- Create: `crates/undone-scene/src/script/api/prose_validate.rs`

**Step 1 — failing test (the single-quote regression):**

```rust
#[test]
fn prose_gate_accepts_single_quoted_id() {
    let r = test_registry_with_skill("FEMININITY");
    // single quotes are legal in minijinja and used in the live pack
    assert!(validate_prose(r#"{% if w.getSkill('FEMININITY') < 20 %}x{% endif %}"#, &r).is_ok());
}
#[test]
fn prose_gate_rejects_unknown_method() {
    let r = test_registry_with_skill("FEMININITY");
    assert!(validate_prose(r#"{{ w.notAReal() }}"#, &r).is_err());
}
#[test]
fn prose_gate_rejects_write_in_prose() {
    let r = test_registry_with_skill("FEMININITY");
    assert!(validate_prose(r#"{{ w.changeMoney(5) }}"#, &r).is_err());
}
#[test]
fn prose_gate_rejects_checkskill_in_prose() {
    let r = test_registry_with_skill("CHARM");
    assert!(validate_prose(r#"{% if w.checkSkill('CHARM', 10) %}x{% endif %}"#, &r).is_err());
}
```

**Step 2:** Run → FAIL (`validate_prose` missing; tokenizer rejects single quotes).

**Step 3a — tokenizer:** in `validate.rs::tokenize`, add a `'\''` arm mirroring the `'"'` arm (single-quoted string literal, same escape handling). Both produce `Tok::Str`.

**Step 3b — `prose_validate.rs`:**

```rust
pub fn validate_prose(template: &str, registry: &PackRegistry) -> Result<(), ScriptError> {
    for region in expression_regions(template) {       // skips {# #}, {% raw %}, handles {%- -%}
        for call in extract_calls(&tokenize(region)?) {  // reuse validate.rs call extractor
            let Some(recv) = call.receiver.as_deref().and_then(receiver_from_token) else { continue };
            let Some(d) = lookup(recv, &call.method) else {
                return Err(ScriptError::unknown_method(&call.receiver_str(), &call.method));
            };
            if !d.contexts.prose {
                return Err(ScriptError::not_prose(&call.method));
            }
            // Validate string-literal content-id args resolve (reuse the gate's id check).
            validate_id_args(d.args, &call, registry)?;
            // Arity: lenient in prose (filters/arithmetic defeat the comma splitter) —
            // validate receiver/method/context + id resolution only (design §5.4).
        }
    }
    Ok(())
}
```

`expression_regions` returns the substrings inside `{{ }}` / `{% %}`, skipping `{# #}` comments and `{% raw %}…{% endraw %}`, tolerating `{%-`/`-%}`. Keep it small; the live corpus uses none of the exotic forms but the extractor must not false-positive on them.

**Step 4:** Tests pass. **Commit:** `feat(script): prose load gate (single-quote aware, method-surface validation)`

### Task H2: Wire the gate into every load path

**Files:**
- Modify: `crates/undone-scene/src/loader.rs` (call `validate_prose` for intro, intro_variants, action prose, thought prose, npc-action prose alongside `compile_condition`/`compile_effect`)
- Modify: `crates/undone-packs/src/preset.rs` (validate each discovery-beat `prose` during `load_presets` — add `undone-scene` dep if not present, OR have the loader validate beats after preset load; choose the path that keeps the dependency DAG legal — `undone-packs` is BELOW `undone-scene`, so put beat-prose validation in `undone-scene`'s loader, NOT in `undone-packs`)
- Modify: `crates/undone-ui/src/char_creation.rs` (remove the `.unwrap_or_else(|_| beat.prose.clone())` masking; on error log + surface, do not leak template source)

> **Dependency-DAG note:** `undone-packs` is below `undone-scene`, so `validate_prose` (in `undone-scene`) cannot be called from `undone-packs::preset`. Validate discovery-beat prose from wherever presets are consumed in `undone-scene`/`undone-ui`, OR add a prose-validation pass in the scene loader that also walks preset beats. Do not introduce an upward dependency.

**Step — live-pack gate-passes test (the audit):**

```rust
#[test]
fn prose_gate_passes_over_entire_base_pack() {
    let (registry, scenes) = load_base_pack_for_test();
    for scene in &scenes {
        for prose in scene.all_prose_fields() {
            validate_prose(prose, &registry)
                .unwrap_or_else(|e| panic!("base pack prose failed gate: {} in {}", e, scene.id));
        }
    }
}
```

Run it. **If it fails, a latent prose bug exists in the base pack — triage:** real bug (fix the scene) vs gate bug (fix the gate, e.g. a construct not handled). The 8 single-quote sites must NOT be among the failures.

**Commit:** `feat(script): validate prose at load across scenes/presets; de-mask char_creation errors`

---

## Phase I — Authoring-tool parity (minijinja-mcp-server)

### Task I1: Surface `validate_prose` in the minijinja MCP server

**Files:**
- Modify: `tools/minijinja-mcp-server/Cargo.toml` (add `undone-scene`, `undone-packs` path deps)
- Modify: `tools/minijinja-mcp-server/src/validator.rs` + `server.rs` (new `validate_prose` tool, or extend `validate_template` to also run the method-surface gate when given a registry/pack path)

**Step 1:** Test (in the tools workspace): a template calling `w.notAReal()` is reported invalid; `w.getSkill('FEMININITY')` valid.
**Step 2–3:** Implement, reusing `undone_scene::script::api::prose_validate::validate_prose`. The server needs a `PackRegistry` — load the base pack (or accept a pack path arg) the way `rhai-mcp-server` does.
**Step 4:** Build the tools workspace: `cd tools && cargo build --release`. Verify the binary.
**Commit:** `feat(tools): minijinja-mcp-server validates prose against the method surface`

> If adding `undone-scene` to the tools workspace proves heavy (pulls the GUI dep tree), fall back to the design's §5.2 alternative: factor `api/` (table + types + prose_validate + validate) into a small leaf crate both the engine and the MCP servers depend on. Decide based on what `cargo tree` shows; record the decision in the design doc.

---

## Phase J — Acceptance tests + docs

### Task J1: End-to-end acceptance pass

**Acceptance Criteria:**
- A scene with a valid condition, effect, and prose loads and plays unchanged.
- A scene whose prose calls an unknown method **fails to load** with a clear error (not a runtime surprise).
- A scene whose prose calls `w.checkSkill(...)` **fails to load** (condition-only).
- `{{ m.getName() }}` after a `setName` effect renders the **display** name, not the spawn name.
- The whole base pack loads, the gate passes, and `cargo test` is green.

**Files:**
- Create: `crates/undone-scene/tests/prose_gate_acceptance.rs`

Write tests for each criterion using the loader against fixture scene TOML (string literals in the test) + the base pack load. **Commit:** `test(script): prose gate + getName acceptance tests`

### Task J2: Playtester pass

Per project mandate (UI/flow-affecting change): launch the game (`game-input` MCP `start_game`) and run the `playtester` agent over 2-3 scenes that use prose-heavy `w.`/`gd.` branches and at least one NPC interaction, confirming prose renders correctly and names are right. Record findings; fix any regression before proceeding. (No commit unless a fix is needed.)

### Task J3: Docs (engine principle #10)

**Files:**
- Modify: `CLAUDE.md` (dependency DAG + workspace structure: remove any `undone-expr` reference — confirm none remain)
- Modify: `docs/plans/2026-02-21-engine-design.md` (replace `undone-expr` description with the Rhai + registry reality)
- Modify: `docs/content-schema.md` (note the registry as the method-surface source of truth; the prose load gate)
- Modify: `HANDOFF.md` (Current State + Session Log entry)

**Commit:** `docs: registry as method-surface source of truth; purge undone-expr references`

---

## Phase K — Finish

### Task K1: Full verification + branch finish

**Step 1:** `cargo test` (full game workspace) — all green.
**Step 2:** `cargo fmt --all -- --check` and `cargo clippy -p undone-scene` — clean.
**Step 3:** `cd tools && cargo build --release` — MCP binaries build.
**Step 4:** Confirm deletions landed: `read_api/`, `write_api/`, snapshot structs, `read_spec`/`write_spec` match arms all gone; one `REGISTRY` is the only method-surface list.
**Step 5:** Invoke `ops:finishing-a-development-branch` (override: always merge).

---

## Self-review checklist (run before handing off)

- [ ] **Spec coverage:** every design section maps to a task — registry (A), divergence audit (B), read lift (C), write lift (D), Rhai bind (E), static gate (F), snapshot elimination (G), prose gate (H), MCP parity (I), tests+docs (J), finish (K).
- [ ] **getName decision** is asserted (effective_name) in B2/C3/G2/J1.
- [ ] **Single-quote** regression covered (H1) and asserted to pass over the base pack (H2).
- [ ] **npc vs role arity** handled: `ArgShape` = source arity; ref injected for `npc` only (D3/E3/G1).
- [ ] **Per-receiver sets** preserved; negative tests in F1.
- [ ] **checkSkill condition-only**; rejected in prose (C1/H1).
- [ ] **No placeholders** other than the explicit "lift verbatim" markers that point at exact source file:line — those are intentional (the bodies are mechanical lifts of named existing fns, not invented logic).
- [ ] **Type names consistent:** `ApiValue`/`ApiArg`/`ApiError`/`ReadFn`/`WriteFn`/`MethodDescriptor`/`Receiver`/`ArgShape`/`Contexts`/`Accessor`/`lookup`/`receiver_from_token`/`validate_prose` used identically across tasks.

---

## Execution handoff

```
Use `ops:executing-plans` to implement the plan at `docs/plans/2026-06-02-script-api-registry.md`
```
