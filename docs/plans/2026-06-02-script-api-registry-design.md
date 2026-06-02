# Script API Registry — Single Source of Truth for the Content-Facing Method Surface

> **Status:** Design (approved approach: A1 — runtime descriptor table + shared live accessors)
> **Date:** 2026-06-02
> **Scope:** read + write surface across all consumers
> **Supersedes nothing. Establishes the registry that `read_api/`, `write_api/`,
> `validate.rs`, and the Minijinja objects are migrated onto.**
>
> **Rev. 2 (2026-06-02):** incorporates three Opus design reviews. Material changes:
> `getName` must unify on `effective_name()` not the Rhai spawn-name body (§1, §4.3);
> the prose gate's tokenizer must accept single-quoted strings (§5.4); the
> `npc(ref)` vs `role` arity asymmetry is modeled explicitly (§4.3); per-receiver
> method sets are **preserved, not unioned** (§2, §4.3); `checkSkill`/`checkSkillRed`
> are barred from prose for RNG-side-effect reasons (§4.2, §4.3); the read/prose
> equivalence test runs **pre-migration** as a divergence audit (§8, §9); the
> discovery-beat gate wires into preset *loading*, and `char_creation`'s
> error-swallowing is flagged (§5.5); the unsafe argument is corrected — ZST views
> hold no pointer, so A1 is *safer* than the Rhai path, not a bigger blast radius (§6).

## 1. Problem

The content-facing method surface — the `w.` / `gd.` / `m.` / `f.` / `role.` /
`scene.` and `npc(ref).*` methods that pack authors call in conditions, effects,
and prose — is currently **hand-maintained in three independent places** for the
read surface and **two** for the write surface:

| Definition site | Purpose | Read | Write |
|---|---|---|---|
| `crates/undone-scene/src/script/read_api/*.rs`, `write_api/*.rs` | the real Rhai receiver impls (read live `World` via thread-local context) | ✓ | ✓ |
| `crates/undone-scene/src/script/validate.rs` (`read_spec` / `write_spec`) | the load-time static gate (arity, id-kind, read/write split) | ✓ | ✓ |
| `crates/undone-scene/src/template_ctx.rs` (`Object::call_method` impls) | the Minijinja prose impls (read a **pre-materialized snapshot**) | ✓ | — |

Adding one content method today means editing three files in lockstep. They have
**already drifted**: the Minijinja `PlayerCtx` (`template_ctx.rs`, match arm ending
~L168) is missing read methods that conditions accept — `isDrunk`, `isVeryDrunk`,
`isMaxDrunk`, `isAnalVirgin`, `hadTraitBefore`, `inCategory`, `beforeInCategory`,
`beforeRace`, `beforeAge`, `beforeSexuality`, `hasStuff`.

Two consequences follow:

1. **Prose has no load-time validation.** `render_prose` is invoked only from
   `engine.rs` at runtime — never from any loader. A misspelled method in prose, or
   one of the divergent methods above, surfaces **only when a player reaches that
   exact scene branch**. This violates engine principles #1 (fail fast at load) and
   #4 (no silent runtime content errors). The exposure is real, not hypothetical:
   prose templates contain ~977 `w.hasTrait(...)` and ~351 `w.getSkill(...)`
   call-sites across the live (non-archive) scenes.

2. **Value-computation is duplicated and HAS already diverged.** The Rhai accessor
   and the Minijinja accessor for the same method are separate code. They do **not**
   merely "risk" divergence — at least one content-visible divergence exists today:

   - **`getName` (the most-called NPC interpolation in prose).** Rhai `role.getName`
     returns `npc.core.name` — the raw **spawn name** (`read_api/role.rs`). Minijinja
     builds `NpcCtx.name` from `npc.core.effective_name()` = `display_name.unwrap_or(name)`
     (`template_ctx.rs`), the **story-assigned** name set by the `setName` effect. So
     `{{ m.getName() }}` in prose shows the story name; `role.getName(...)` in a
     condition returns the spawn name. **The unified accessor MUST adopt
     `effective_name()`** (the prose behavior). Mechanically lifting the Rhai body
     here would silently revert every prose NPC name across all 74 scenes to the
     random spawn name the moment an NPC is story-named. This is the canonical proof
     that "lift the Rhai body" is not a safe uniform rule.

   Beyond value drift, the *method sets* themselves differ per receiver and per
   consumer in load-bearing ways — see §2 (per-receiver sets) and §4.3.

The legacy `undone-expr` crate that some docs still reference is **deleted**; Rhai
is the permanent condition/effect language. (Doc cleanup is tracked in §11.)

## 2. Goals / Non-Goals

**Goals**

- One declarative source of truth for the entire content-facing method surface
  (read + write, all six receivers + `npc(ref)`), covering: method name, receiver,
  arg shape, content-id validation kind, numeric-range constraints, and which
  contexts (`condition` / `effect` / `prose`) the method is valid in.
- All four consumers — Rhai condition engine, Rhai effect engine, the load-time
  static gate, and Minijinja prose dispatch — **driven from that one source**.
- **Load-time validation for prose**, reusing the same registry, so an unknown or
  mis-contexted method in a prose template fails at pack load.
- **Behavior preservation**: all 153 conditions and the effect call-lists across
  the 74 live scenes evaluate identically before and after. No content edits.

**Non-Goals**

- No change to the Rhai *language* surface available to authors (operators,
  `if/else` in effect lists, etc.). We are unifying the *method API*, not the
  expression grammar.
- No new methods, and **no change to which `(receiver, method)` pairs are valid in
  which context.** This is sharper than "no new methods" and is the load-bearing
  invariant the reviews surfaced:
  - The read surfaces of `m`, `f`, and `role` are **deliberately different sets**, not
    one "NPC surface." `m` has the attraction/love predicates and `isCohabiting`; `f`
    has `isPregnant`/`isVirgin` but **lacks** `hasTrait` and the attraction predicates;
    `role` is a third set taking a leading role-id. The registry keys by
    `(receiver, method)` and **must preserve these sets exactly — never union them.**
    `f.hasTrait(...)`, `m.isPregnant(...)`, and `m.isVirgin(...)` are "unknown method"
    load errors today and MUST stay load errors. Negative tests are required (§9).
  - Closing the prose read-divergence happens by making the *existing read methods*
    prose-available, which **does grow the prose surface** (e.g. `gd.getStat`,
    `gd.npcLiking`, `gd.arcStarted`, `gd.getJobTitle` become prose-callable for the
    first time). This is intended and now policed by the prose load gate. But it is a
    surface change, acknowledged here, not a no-op.
  - Some methods are valid in Rhai-conditions but currently rejected by the **static
    gate** through pre-existing gate drift (e.g. `m.isCohabiting` is registered in
    Rhai and implemented in prose but absent from `validate.rs::read_spec`). For every
    such drifted method the migration must make an **explicit** decision — add it to
    the table (becomes authorable) or drop the orphan impl — and record that decision.
    Defaulting silently in either direction is forbidden.
- `checkSkill`/`checkSkillRed` stay **condition-only** (not prose, not effect) — they
  have RNG side effects (§4.2). "All read methods become prose-available" is therefore
  **not** a blanket rule; prose-availability is per-descriptor.
- No change to scene TOML schema or authored syntax.

## 3. Approach (A1 — runtime descriptor table + shared live accessors)

A single `static` table of method descriptors is the source of truth. Each method's
logic is a plain, individually-testable accessor function with a neutral signature.
Every consumer is driven from the table:

- **Rhai cond/effect engines** — iterate the table; for each entry register a typed
  adapter that marshals Rhai args into the neutral arg form and calls the accessor.
- **Static gate (`validate.rs`)** — replace the hand-written `read_spec`/`write_spec`
  match with a table lookup. Used by both the loader and `rhai-mcp-server` (which
  already depends on `undone-scene`, so it inherits the registry with no fourth copy).
- **Minijinja prose** — the receiver objects become **zero-sized**; their
  `call_method` looks up the table and runs the *same accessor* against live `World`.
  **The snapshot is eliminated.**
- **Prose load gate** — a static scan of prose templates validates every
  `receiver.method(...)` call-site against the table at load time.

### Why A1 over A2 (the conservative fallback)

A1 makes Minijinja read **live `World`** during render (through the existing
thread-local `ReadCtxGuard`), so a single accessor body serves both Rhai and prose.
This is the only variant that eliminates *value-computation* duplication.

The conservative fallback **A2** keeps the owned snapshot and shares only the
*metadata* table (enabling the static gate + prose load gate + one method list), but
leaves Minijinja with its own snapshot-reading accessors — so the value-drift risk
of §1.2 remains. A2 is documented here as the fallback if review concludes the
unsafe-surface extension in §6 is unacceptable. **Recommendation: A1.** It is the
only option that fully discharges the "no duplication" intent, and the unsafe
invariant it leans on is already trusted for every Rhai condition eval.

> Rejected outright: a `define_api!` macro generating native code for three shapes
> (cryptic errors, high maintenance ceiling — runs against the project's readability
> grain), and external-data-file codegen (method *bodies* are inherently Rust, so it
> splits the source rather than unifying it).

## 4. Registry representation

A new module tree under `crates/undone-scene/src/script/api/`.

### 4.1 Neutral value & argument types

```rust
/// A value produced by a read accessor, convertible to both backends.
pub enum ApiValue { Bool(bool), Int(i64), Str(String) }

/// A literal argument as seen by an accessor, borrowed from the call.
pub enum ApiArg<'a> { Str(&'a str), Int(i64), Bool(bool) }

/// Why an accessor failed at runtime (unknown content id, bad npc ref, …).
/// Closed set — every variant must convert to BOTH a rhai::EvalAltResult and a
/// minijinja::Error so the two adapters share one error vocabulary.
pub enum ApiError {
    UnknownId { kind: &'static str, id: String },  // content id didn't resolve
    NoActiveNpc { sex: &'static str },             // m./f. method with no NPC bound
    UnboundRole { role: String },                  // role.* with no binding
    BadArgs { method: &'static str },              // arity/type the gate should have caught
}
```

`ApiValue` converts to `rhai::Dynamic` and to `minijinja::Value`. `ApiArg` is
extracted from `&[rhai::Dynamic]` (the adapter) or `&[minijinja::Value]`
(`call_method`). The set is intentionally tiny — the whole surface returns only
bools, ints, and strings, and takes only string-literal ids, ints, and bools.
Numeric returns are `u32`/`i32`/`i64` at the Rust level today; all widen to
`ApiValue::Int(i64)` and must compare identically in `{% if gd.week() >= N %}` — the
equivalence test (§9) covers the unsigned/cast cases explicitly, not just strings.

**Per-backend error conversion is a behavior surface, not an afterthought.** Today an
unbound `m` in prose binds `m => Value::UNDEFINED`, so `{{ m.getName() }}` hits
minijinja's undefined path (renders empty / lenient), whereas the Rhai `M` accessor
errors "no active male." Unifying onto one accessor makes prose **fail loud**
(`ApiError::NoActiveNpc`) instead of rendering empty. That is the better behavior
(principle #4) but it **is** a change; §9 adds an acceptance test for
`{{ m.getName() }}` with no male bound and pins the decision (error, not empty).

### 4.2 Accessor signatures

The existing accessors are already closure bodies over `(world, reg, ctx)` (read) or
`(world, ctx, reg)` (write). The migration **lifts each body into a free function**:

```rust
pub type ReadFn  = fn(&World, &PackRegistry, &SceneCtx, &[ApiArg]) -> Result<ApiValue, ApiError>;
pub type WriteFn = fn(&mut World, &mut SceneCtx, &PackRegistry, &[ApiArg]) -> Result<(), EffectError>;
```

The accessor does *not* touch the thread-local context — the thread-local plumbing
(`with_read_ctx` / `with_write_ctx`) stays in the *adapters*, so both Rhai and
Minijinja funnel through the same guard and hand the borrowed state to the same
accessor.

**Caveat — accessors are NOT all side-effect-free.** `checkSkill`/`checkSkillRed`
read through `&SceneCtx` but `SceneCtx::get_or_roll_skill` rolls `thread_rng()` and
**caches the result via interior mutability** (`scene_ctx.rs`). So calling them has an
observable side effect on the per-scene roll cache. This is required for conditions
(a check rolls once and is stable for the scene) but is exactly why they must be
**barred from prose**: rendering must be idempotent/replayable, and a prose
`checkSkill` on a not-yet-rolled skill would seed the cache and change a later
action's check outcome. The descriptor marks them `condition`-only (§4.3), and the
prose load gate rejects them. Do not generalize "reads are pure" into "reads are
prose-safe" — prose-safety is a per-descriptor `Contexts` flag.

Worked example (mechanical lift):

```rust
// before — read_api/player.rs
fn is_drunk(&mut self) -> RhaiResult<bool> {
    with_read_ctx(|world, _reg, _ctx| Ok(world.player.is_drunk()))
}
// after — api/read/player.rs
fn is_drunk(w: &World, _r: &PackRegistry, _c: &SceneCtx, _a: &[ApiArg]) -> Result<ApiValue, ApiError> {
    Ok(ApiValue::Bool(w.player.is_drunk()))
}
```

### 4.3 Descriptor

```rust
pub enum Receiver { W, Gd, M, F, Role, Scene, Npc }

pub enum ArgShape {           // declarative arg contract, drives validation + marshalling
    None,
    Id(IdKind),               // one content-id string literal (Trait/Skill/Stat/Category/Arc/NpcTrait)
    IdInt(IdKind),            // id + trailing int literal (checkSkill, …)
    Int { i8_range: bool },   // one int literal; i8_range gates step deltas
    Str,                      // one opaque string (stuff ids, flags, job titles)
    StrOpt,                   // overloaded arity (setVirgin(value) | setVirgin(value, "type"))
    // …a closed set; one variant per shape observed in validate.rs today
}

pub struct Contexts { pub condition: bool, pub effect: bool, pub prose: bool }

pub enum Accessor { Read(ReadFn), Write(WriteFn) }

pub struct MethodDescriptor {
    pub receiver: Receiver,
    pub name: &'static str,
    pub args: ArgShape,
    pub contexts: Contexts,
    pub accessor: Accessor,
}

pub static REGISTRY: &[MethodDescriptor] = &[ /* one entry per method */ ];
```

**Ref handling for `role` vs `npc` is asymmetric and must NOT be unified into one
arity field.** The two receivers count source arguments differently today, and the
static gate depends on it:

- `role.getLiking("ROLE_X")` — the role id **is a real positional source arg**;
  `validate.rs` counts it (`spec(1)`/`spec(2)`). `role` is always present (it is a
  receiver token, not constructed).
- `npc("m").addLiking(1)` — the ref comes from the **free constructor `npc("m")`**,
  validated as a *separate* call `("npc","npc") = spec(1)`; the chained
  `addLiking` has arity **1** (the int only) — the ref is NOT in its source arg list.

The descriptor therefore distinguishes **source arity** (what the gate counts in the
text) from **accessor arity** (what the accessor receives, which always includes the
resolved ref for both receivers). `ArgShape` describes *source* args; a per-receiver
rule supplies the ref to the accessor: for `Receiver::Role` the ref is source-arg 0
(already counted), for `Receiver::Npc` the ref is injected by the adapter from the
bound constructor and is **not** part of source arity. Getting this wrong rejects
every `npc(...).addLiking(...)` in the live pack (or indexes past the args). This is
called out as the implementation's first correctness checkpoint.

**`getName` accessor override.** Per §1, the unified `getName` (on `M`/`F`/`Role`)
resolves `npc.core.effective_name()`, not `core.name`. Rhai has **no** `m.getName`/
`f.getName` today (only `role.getName`); adding `getName` to the `M`/`F` read set is
a deliberate, recorded surface addition restricted to the contexts those receivers
already serve. The migration table overrides the Rhai source body here — flagged
inline in `table.rs` with a comment pointing at this section.

**Display-vs-Debug is per-method and load-bearing.** Authors branch on literal
strings in prose (`{% if w.getArousal() == "Comfort" %}`). Some methods return
`Display` (`getLiking`, `getAttraction` via `to_string()`), others `Debug`
(`getLove`, `getBehaviour` via `{:?}` — those enums have no `Display`). The unified
accessor must reproduce the **exact** `Display`-vs-`Debug` choice per method. A
uniform "use Display everywhere" rewrite is forbidden; the per-method `ArgShape`/
return checklist (§4.4) records the choice.

**`(receiver, name)` uniqueness is asserted.** The read/write split relies on the key
being unique — no `(receiver, method)` may appear as both a `Read` and a `Write`
(else the effect engine would expose a write under a name a condition can call). A
construction-time / test-time assertion enforces uniqueness; a duplicate key is a
build failure, not silent last-wins.

The read/write split is enforced structurally: `Accessor::Write` entries have
`contexts.condition == false` and `contexts.prose == false`; the registration step
only adds `Write` accessors to the effect engine. A `Write` in a condition is
therefore "unknown method" at the static gate exactly as today.

### 4.4 The per-method table is the correctness contract

`table.rs` is not just a name list — each entry's `ArgShape`, `Contexts`, and
return choice encode behavior that must be copied exactly from the current impls. The
implementation plan carries the **full per-method table** as a checklist. Two classes
that are easy to get wrong and have no margin for error:

- **Intentionally-unvalidated id args → `ArgShape::Str`, never `Id`.** These methods
  deliberately skip registry validation today; validating them now would reject
  currently-loading content: `w.hasStuff`, `w.addStuff`/`removeStuff`, `gd.getStat`,
  `gd.hasGameFlag`, `gd.arcStarted`, `gd.arcState`, `gd.npcLiking`, `gd.setJobTitle`,
  `npc(...).setFlag`/`setName`/`setRelationship`/`setBehaviour`/`addSexualActivity`,
  scene/role flag args. Each maps to `Str`, and the gate must keep *not* resolving them.
- **Empty/default fallbacks that authors branch on.** `gd.arcState` returns `""` for
  an unstarted arc (prose tests `== ""`); `gd.npcLiking` defaults `"Neutral"`;
  `gd.getStat` returns `0` for an un-interned stat; `w.hasStuff` returns `false` for an
  un-interned id. The unified accessor reproduces each exact fallback (empty string,
  not undefined/null).

## 5. Consumer wiring

### 5.1 Rhai registration (`api/rhai_bind.rs`, replaces `read_api.rs` + `write_api.rs`)

Iterate `REGISTRY`. For each entry, register a name+receiver method on the
appropriate engine(s) using a small fixed set of arity-shaped helper registrars
(`reg_read_0`, `reg_read_1_str`, `reg_read_1_int`, `reg_read_2_str_int`,
`reg_write_1_int`, …). Each helper closes over the accessor, marshals the Rhai
args into `[ApiArg]`, calls the accessor through `with_read_ctx`/`with_write_ctx`,
and converts the result. The set of helpers is bounded by the `ArgShape` variants,
**not** by the method count — this is mechanical plumbing, not per-method code.

**Overloaded arity needs two native registrations.** `setVirgin(value)` and
`setVirgin(value, "type")` are two `register_fn`s today (Rhai resolves overloads by
arg count at the native boundary). The `ArgShape::StrOpt` descriptor is one entry with
one accessor that branches on `args.len()`, but the Rhai adapter must register **both**
a 1-arg and a 2-arg native fn pointing at that accessor. So the registrar set is
"~8 shapes + an overload variant that double-registers," not a flat 8 — the plan lists
the exact registrars and shows `setVirgin`'s double-registration explicitly.

> This is the one place a tiny local macro is acceptable (to stamp out the registrar
> helpers), because it generates *plumbing*, not method logic, and its expansion is
> trivial to read.

### 5.2 Static gate (`validate.rs`)

`read_spec` / `write_spec` are deleted. The scanner's per-call check becomes a
`REGISTRY` lookup by `(receiver, method)` returning the `ArgShape` + `Contexts`. The
tokenizer, id-resolution, i8-range checks, and the "writes are unknown in conditions"
rule are unchanged — they now read their contract from the descriptor instead of a
parallel match. `rhai-mcp-server` inherits this automatically (it already depends on
`undone-scene`).

**Authoring-tool symmetry — `minijinja-mcp-server` must surface the prose gate.**
Unlike the Rhai server, `minijinja-mcp-server` depends only on `minijinja` and
validates *syntax* — it is blind to the method surface, so the very errors this effort
exists to catch (unknown/mis-contexted methods in prose) stay invisible in the tool
authors actually use for prose. The plan adds an `undone-scene` dependency to that
server and exposes `validate_prose`, so authoring-time prose validation matches
load-time exactly the way `rhai-mcp-server` already matches the Rhai gate. (If the
extra dependency is judged too heavy for the tools workspace, the fallback is to
factor the gate + registry metadata into a small leaf crate both servers can share —
but parity between authoring-time and load-time is the requirement, not optional.)

### 5.3 Minijinja dispatch (`api/minijinja_bind.rs`, replaces the objects in `template_ctx.rs`)

One zero-sized struct per receiver (`struct WView; struct GdView; …`) implementing
`minijinja::value::Object`. `call_method(method, args)`:

1. Look up `(Self::RECEIVER, method)` in `REGISTRY`.
2. If absent **or** `!contexts.prose` → `UnknownMethod` error (matches the prose
   load gate; belt-and-suspenders for any dynamically-built template).
3. Marshal `args: &[minijinja::Value]` → `[ApiArg]`.
4. `with_read_ctx(|w, r, c| accessor(w, r, c, &args))` and convert `ApiValue` →
   `minijinja::Value`.

`template_ctx.rs` shrinks to: resolve NPC presence (see §6), install the read guard,
and render. All snapshot structs and their `call_method` bodies are **deleted**.
`role` is *not* an NPC-presence case: the `RoleView` ZST is always bound; an unbound
*individual* role lookup (`role.getName("UNBOUND")`) must still error inside the
accessor (`ApiError::UnboundRole`), reproducing today's `RoleLookupCtx` behavior — it
must not silently become undefined/empty. (§6.1 covers only `m`/`f` truthiness.)

### 5.4 Prose load gate (`api/prose_validate.rs`, new)

`pub fn validate_prose(template: &str, registry: &PackRegistry) -> Result<(), ScriptError>`:

- Extract the expression regions of the Minijinja template, robustly skipping
  `{# comments #}`, `{% raw %}…{% endraw %}` blocks, and handling whitespace-control
  markers (`{%-`, `-%}`, `{{-`, `-}}`). The live corpus uses none of these yet, but a
  naive `{{…}}`/`{%…%}` split will false-positive the day an author adds a comment
  containing a method-like token. Cheap now, expensive to retrofit.
- Over those regions, find `receiver.method(...)` call-sites and their string-literal
  args. **The tokenizer must accept BOTH single- and double-quoted strings.** Rhai
  requires double quotes, so `validate.rs::tokenize` only handles `"`. Minijinja
  accepts both, and authored prose **already** uses single quotes —
  `w.getSkill('FEMININITY')` appears at 8 live call-sites. Reusing the Rhai tokenizer
  as-is would reject those valid, currently-rendering templates at load. Teaching the
  tokenizer single-quoted strings (and treating either style as satisfying the
  string-literal-id rule) is a **day-one** requirement, not a follow-up.
- For each call-site: require a `REGISTRY` entry with `contexts.prose == true`, on the
  correct receiver. Validate the receiver/method/context and resolve string-literal
  content-id args against the registry.
- **Arity in prose is validated leniently.** The Rhai `parse_args` comma-splitter is
  approximate (it flushes on top-level `Other` tokens) and Jinja expression grammar is
  richer than Rhai condition grammar — filters (`| default(x)`), arithmetic
  (`getSkill("X") + 1`), and nested calls would make a strict arity check over prose
  produce false rejections. The gate enforces **call-site shape** (receiver.method
  exists, is prose-contexted, string-literal id args resolve) and does not strictly
  enforce arg *count* where the expression grammar can defeat the tokenizer. (Method
  identity + id resolution is where the real authoring errors are; arity in prose is
  secondary and caught at render in the rare case it slips.)

**Out of scope for the gate (stated, not silently missed):** unknown Minijinja
**filters/functions/tests** are not validated — the engine registers no custom ones,
and minijinja resolves them in its environment, not in our method surface. A typo'd
filter (`| titel`) remains a render-time error. The gate is method-surface-only.

Static scan (not trial-render) is deliberate and consistent with `validate.rs`: a
trial render only exercises the branch the probe world happens to take, leaving other
`{% if %}` arms unchecked. The string-literal-arg rule that makes the Rhai scan
complete applies equally to prose.

### 5.5 Wiring the gate into every load path

The gate must run wherever prose is authored, and the wiring targets are not all
obvious:

- **Scenes:** `loader.rs`, for every prose field (intro, intro variants, action
  prose, thought prose, npc-action prose), alongside the existing
  `compile_condition` / `compile_effect` calls.
- **Discovery beats:** `undone-packs::preset` only *deserializes* beats — it has no
  render call to sit "alongside." The gate runs as a dedicated validation pass inside
  preset loading (`load_presets`), over each beat's `prose`.
- **`char_creation.rs` swallows errors today** — the live discovery-beat render is
  `render_prose(...).unwrap_or_else(|_| beat.prose.clone())`, which shows raw template
  source to the player on any error (a standing principle-#4 violation). With the load
  gate in place this masking is both unnecessary and harmful; the migration removes the
  `unwrap_or_else` fallback (or converts it to a logged hard error), so a beat that
  somehow reaches render with a bad template fails loud instead of leaking source.
- **Other `render_prose` callers** (tests, dev IPC, any future tool) are **not** load
  paths and are out of the gate's reach by construction — noted as a known limitation,
  not a hole to plug.

## 6. Eliminating the snapshot — the live-context move

Minijinja `Object` requires `Send + Sync + 'static`, which is why the current code
borrows nothing and instead **copies** `World` state into owned `PlayerCtx` fields.
A zero-sized view is trivially `Send + Sync + 'static`; it reads live `World` through
the **existing** `ReadCtxGuard`, which is sound here because `render_prose` renders
**synchronously on the single UI thread** (`tmpl.render(...)`), exactly the invariant
the guard already documents for Rhai eval.

`render_prose` becomes:

```rust
let _guard = ReadCtxGuard::install(world, registry, ctx);   // same guard Rhai uses
let env = /* env with the six ZST views + npc presence */;
tmpl.render(ctx_map)                                          // ZST call_method reads live World
```

**Unsafe-surface note (corrected after review).** A1 extends the thread-local
read-context to the render path, but it is **strictly safer than the Rhai path**, not
a bigger blast radius. The ZST views hold **no pointer** — they are zero-sized. The
only raw-pointer deref happens inside `with_read_ctx` *while the guard is alive on the
stack*. Even if minijinja retained a `Value`/`Arc<dyn Object>` past the render call
(it does not — see below), a post-render method call would hit the guard's
"no evaluation context installed" error (`context.rs`), not UB. So the snapshot
elimination does not widen the trusted-unsafe surface in any way that can dangle; it
reuses the identical synchronous-single-thread invariant the Rhai engine already
depends on. (A2 — keeping the owned snapshot — remains documented as a fallback only
if review rejects live-context rendering outright, but the safety argument no longer
motivates it.)

**Hard rule: prose rendering uses plain `render` / `render_to_write` only.**
`minijinja::Template::render` returns a `String` and drops all evaluation state before
returning. The `render_and_return_state` variant returns `State`, which can retain the
root `Value` (hence the ZST `Arc`s) and `{% set %}`-captured macros past the call —
which, combined with live-context reading, is the one way to call a view after the
guard drops. It would surface as a clean "no context installed" error rather than UB,
but the design forbids `render_and_return_state` in this codebase outright, enforced by
a comment at the `render_prose` call site. The live-context safety argument depends on
this; do not introduce state-returning renders.

### 6.1 NPC presence must be preserved

Today `m`/`f` are passed to the template as `Value::UNDEFINED` when no NPC of that
sex is bound, so authors write `{% if m %}…{% endif %}`. A ZST view is always
truthy, which would silently break presence tests. **Mitigation:** `render_prose`
computes presence from `ctx.active_male`/`ctx.active_female` and binds `m`/`f` to the
ZST view **or** `Value::UNDEFINED` accordingly. This is the only snapshot-era logic
that remains, and it carries no data — only presence. Presence is computed *before*
`ReadCtxGuard::install` (from the owned `ctx` borrow `render_prose` already holds), so
the guard's "ctx borrowed for the whole call" invariant is unaffected.

`role`, `w`, `gd`, `scene` are **always present** (always-truthy ZSTs) — this already
matches today (they are always non-undefined objects). `role` is explicitly NOT an
`m`/`f`-style presence case: see §5.3 — an unbound *individual* role lookup errors
inside the accessor, it does not make the `role` receiver undefined.

Acceptance tests required: `{% if m %}` truthy for a bound male / falsy for none
(same `f`); `{{ m.getName() }}` with no male bound errors loud (`ApiError::NoActiveNpc`,
per §4.1) rather than rendering empty; `role.getName("UNBOUND")` errors.

## 7. Module layout

```
crates/undone-scene/src/script/
├── api/
│   ├── mod.rs            # Receiver, ApiValue, ApiArg, ApiError, ArgShape, Contexts,
│   │                     #   MethodDescriptor, Accessor; pub fn lookup(receiver, method)
│   ├── table.rs          # static REGISTRY: &[MethodDescriptor]
│   ├── read/             # lifted read accessors (player.rs, game_data.rs, npc.rs,
│   │                     #   role.rs, scene.rs) — pure fns over borrows
│   ├── write/            # lifted write accessors (player.rs, game_data.rs, npc.rs, scene.rs)
│   ├── rhai_bind.rs      # iterate REGISTRY → register adapters on cond/effect engines
│   ├── minijinja_bind.rs # six ZST Object views; call_method dispatches via REGISTRY
│   └── prose_validate.rs # static prose scan
├── validate.rs           # tokenizer kept; spec match → REGISTRY lookup
├── engine.rs             # build_engines() now calls api::rhai_bind
├── context.rs            # unchanged (guards reused by minijinja_bind)
└── template_ctx.rs       # shrinks to: npc presence + install guard + render
```

Deleted: `script/read_api/`, `script/write_api/`, the snapshot structs and
`call_method` impls in `template_ctx.rs`, and the `read_spec`/`write_spec` tables in
`validate.rs`.

## 8. Migration strategy

Sequenced so the tree compiles and tests pass at every step. **Step 0 is not
optional** — it is what makes the migration behavior-preserving rather than
behavior-changing-by-accident.

0. **Divergence audit FIRST (against the current two-impl tree).** Before lifting
   anything, write the **read/prose equivalence test** (§9) and run it on the
   *unmodified* code, where Rhai and Minijinja are still separate impls. It WILL fail
   — at minimum on `getName` (spawn vs `effective_name`), and on the per-receiver
   method-set gaps (`m.isPregnant`, `f.hadOrgasm`, etc.). Capture every failure and
   make an explicit per-method decision (which value/behavior the unified accessor
   adopts), recorded in `table.rs` comments. The captured expectations become the
   regression oracle for the rest of the migration. Skipping this is how `getName`
   silently regresses.
1. **Scaffold** `api/mod.rs` types + empty `table.rs`. No behavior change.
2. **Lift read accessors** one receiver at a time into `api/read/*`; populate
   `REGISTRY` read entries with the §0 decisions baked in. Keep `read_api/`
   temporarily; do not yet rewire.
3. **Lift write accessors** into `api/write/*`; populate write entries.
4. **Rewire Rhai** (`rhai_bind`) to register from `REGISTRY`; delete `read_api/` +
   `write_api/`. Full Rhai parity tests must pass here. (Note: Rhai-vs-Rhai parity
   passing here proves nothing about prose — that is what step 6's gate is for.)
5. **Rewire `validate.rs`** to table lookup; delete `read_spec`/`write_spec`. Run the
   negative-surface tests (`f.hasTrait`, `m.isPregnant`, `checkSkill`-in-prose all
   rejected) and the live-pack gate (must pass, incl. the single-quote cases).
6. **Rewire Minijinja** to ZST views + live context; delete snapshot structs.
   Preserve npc presence (§6.1). **The §0 read/prose equivalence test is the gate on
   THIS step** (not step 4) — it must now pass with the consciously-chosen unified
   values. This is where prose behavior actually changes, so this is where the
   prose-aware test must run.
7. **Add prose load gate**; wire into `loader.rs` + preset loading + de-mask
   `char_creation` (§5.5). Update `minijinja-mcp-server` (§5.2).
8. **Doc cleanup** (§11).

## 9. Testing strategy

- **Rhai parity (Rhai→Rhai regression).** Extend `rhai_parity.rs` into a golden
  corpus: every distinct method, plus the actual 153 authored conditions and the
  effect call-lists harvested from the live scenes, evaluated against fixture worlds —
  asserted equal pre/post migration. Fixtures must come from existing pack content,
  not be hand-authored to the new code (anti-circular-validation rule). **Scope limit
  (important):** this catches Rhai-side regressions only. It is *structurally blind* to
  Rhai-vs-prose divergences (`getName`, etc.) because both pre- and post-migration Rhai
  values use the lifted Rhai body — they match while prose silently changes. Do not
  treat this as the behavior-preservation gate for prose.
- **Effect parity = resulting-World snapshot, not return value.** Effect call-lists
  mutate state and return nothing. Equality is measured by applying each list to a
  fixture world and snapshotting the **resulting `World`** (relevant fields), pre vs
  post migration. Must cover continue-on-error lists, `if/else` effect blocks (whose
  branch depends on an embedded condition evaluating identically), and the
  `npc(ref)` write path.
- **Read/prose equivalence (the prose behavior gate).** For every read method, assert
  the Rhai value and the Minijinja-rendered value agree against the same world.
  **Authored and run FIRST against the pre-migration two-impl tree** (§8 step 0), where
  it surfaces the real divergences (`getName` spawn-vs-display, the per-receiver gaps)
  as *expected failures* to be decided — NOT written after the migration, which would
  be circular (asserting the new code equals itself). After step 6 it becomes the
  regression guard. Enumerated expected pre-migration failures include at least:
  `getName` on `m`/`f`/`role`.
  *Caveat (anti-circular-validation):* the live pack contains **zero** NPC prose
  (`0` `{% if m %}`, `0` `m.*`/`f.*`/`role.*` calls in templates), so the NPC-surface
  equivalence and presence tests are necessarily **synthetic** — they cannot be derived
  from content. This is a known limitation, stated so the synthetic fixtures aren't
  mistaken for content-derived coverage.
- **Negative surface tests (gate must keep rejecting).** `f.hasTrait(...)`,
  `m.isPregnant(...)`, `m.isVirgin(...)`, `f.hadOrgasm(...)` remain "unknown method"
  load errors; `checkSkill`/`checkSkillRed` are rejected in prose and in effects; a
  `Write` method in a condition is rejected. These guard the per-receiver/per-context
  sets the union would silently weaken.
- **Registry invariants (construction/test-time).** `(receiver, name)` uniqueness; no
  key is both `Read` and `Write`; every `Write` has `condition=false, prose=false`.
- **Prose gate acceptance.** Unknown method, mis-contexted (write) method, bad content
  id, and (regression) a **single-quoted** id arg each behave correctly: the first
  three fail at load with a clear error; the single-quote case **passes**. Run the gate
  over the entire live pack and assert it passes (doubles as an audit of existing
  prose — any failure is a latent bug found, and the single-quote cases must not be
  among them).
- **NPC presence.** `{% if m %}` truthy when a male is bound, falsy when not (same
  `f`); `{{ m.getName() }}` with no male errors loud; `role.getName("UNBOUND")` errors.
- **`rhai-mcp-server` / `minijinja-mcp-server`.** Existing Rhai-server tests still pass
  (inherited registry); new `minijinja-server` tests confirm `validate_prose` catches an
  unknown/mis-contexted prose method (§5.2).
- **Full `cargo test` workspace** + a `playtester` pass over a few migrated scenes
  (per project mandate after scene/flow-affecting changes), since prose rendering
  changes execution path.

## 10. Risks & mitigations

| Risk | Mitigation |
|---|---|
| **`getName` (and other Rhai-vs-prose) value regression** — highest risk | §8 step 0 divergence audit forces the conscious decision (`effective_name()`); read/prose equivalence test gates step 6, not step 4. |
| **`npc()` vs `role` arity asymmetry breaks load of existing content** | `ArgShape` = source arity; per-receiver rule injects the ref for `npc` only (§4.3). First correctness checkpoint; live-pack gate proves it. |
| **Single-quoted prose ids rejected by the reused tokenizer** | Tokenizer teaches single quotes day-one (§5.4); live-pack gate-passes test includes the 8 single-quote sites. |
| **Per-receiver sets silently unioned** (`f.hasTrait`, `m.isPregnant` become valid) | Registry keys per `(receiver, method)`; negative surface tests (§9). |
| **`checkSkill` RNG pollutes roll cache if prose-callable** | `condition`-only `Contexts`; prose gate rejects; negative test (§4.2, §9). |
| **Unsafe surface at render** (§6) | ZST views hold no pointer → cannot dangle; *safer* than Rhai path. `render_and_return_state` forbidden. |
| **Unvalidated-id methods get "tightened" and reject content** | Explicit `ArgShape::Str` checklist class (§4.4). |
| **NPC presence regression** (`{% if m %}`) | Presence binding retained in `render_prose` + acceptance tests (§6.1). |
| **Prose gate false positives** (filters/arithmetic defeat the comma-splitter) | Gate validates call-site shape, not strict prose arity (§5.4); live-pack gate-passes test. |
| **`minijinja-mcp-server` stays blind** | Add `validate_prose` to it (§5.2); authoring-time matches load-time. |
| **`setVirgin` overload** | `StrOpt` descriptor double-registers on Rhai (§5.1). |
| **`ApiValue`/`ApiArg` round-trip cost** | One enum match + convert per call; negligible; AST eval dominates (existing `context.rs` spike bench). |

## 11. Out-of-scope cleanup folded in

- Purge `undone-expr` references from `CLAUDE.md` (dependency DAG, workspace
  structure) and `docs/plans/2026-02-21-engine-design.md`; the crate is deleted.
- Update `docs/content-schema.md` and the engine-design doc to describe the registry
  as the method-surface source of truth (per engine principle #10).

## 12. Definition of done

- One `REGISTRY`; `read_api/`, `write_api/`, the snapshot structs, and
  `read_spec`/`write_spec` are gone.
- The §8-step-0 divergence audit is recorded: every Rhai-vs-prose divergence has a
  decided, commented unified value in `table.rs` (`getName` → `effective_name()` at
  minimum).
- Per-receiver/per-context sets preserved: negative surface tests green
  (`f.hasTrait`, `m.isPregnant`, `checkSkill`-in-prose, write-in-condition all
  rejected); registry uniqueness + read/write-split invariants asserted.
- Prose is validated at load (single quotes accepted); the gate passes over the whole
  live pack; `char_creation`'s error-swallowing fallback removed.
- `minijinja-mcp-server` surfaces `validate_prose` (authoring-time == load-time).
- Rhai-parity + effect-parity + read/prose-equivalence + presence tests green; full
  workspace `cargo test` green; `playtester` pass on migrated scenes.
- Docs updated; `undone-expr` references purged.
