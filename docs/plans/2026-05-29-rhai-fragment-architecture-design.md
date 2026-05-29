# Rhai + Fragment Architecture — Design

> **Date:** 2026-05-29
> **Status:** Design approved (2026-05-29). Decomposing into phase implementation plans.
> **Scope:** Replace `undone-expr` and the `EffectDef` enum with Rhai; pivot content to a
> Disco-Elysium-style fragment model with passive/active/check fragments and a global pool.
> **Source-of-truth order:** live code > CLAUDE.md > creative-direction.md > this file.

---

## 1. Goal

Lower the cost of iteration — both **engineering** (add functionality without recompiling Rust)
and **writing** (richness from composed small pieces, not monolithic hand-branched scenes) —
while preserving the engine's fail-fast / load-time-validation guarantees and the
"platform not product" principle.

This is a **content engine**, not a feature. It decomposes into ~5 independently-shippable
phases (Section 7). The game stays playable after every phase.

## 2. The reframe (why this is smaller than it looks)

Two discoveries from the design-research pass collapse most of the perceived risk:

1. **The DE check mechanic already exists and is tested.** `undone-expr/src/eval.rs` has
   `w.checkSkill(skill, dc)` (white/retryable) and `w.checkSkillRed(skill, dc)` (red/one-shot),
   roll-under math `target = (skill + (50 - dc)).clamp(5, 95)`, per-scene roll caching
   (`SceneCtx::get_or_roll_skill`), and permanent red-failure persistence
   (`GameData::red_check_failures` + the `FailRedCheck` effect). Content simply never used it.

2. **The fragment model is a unification of existing primitives, not an invention.** The four
   current scene structures are already the four fragment behaviors:
   - `intro_variants` → passive fragment, first-match-wins
   - `thoughts` → passive fragment, all-passing-fire
   - `actions` → active fragment (choice button)
   - `npc_actions` → passive fragment with a `world_move` tag, weighted selection

   The composer collapses them into one `Fragment` type and replaces ~3 near-duplicate loops
   in `engine.rs`.

The genuinely new work is therefore: (a) the Rhai plumbing + its fail-fast gate,
(b) the composer + global pool, (c) authoring conventions and the fluid-composure stat.

## 3. Decisions locked

| # | Decision | Choice |
|---|---|---|
| D1 | Rhai's role | Replaces BOTH `undone-expr` conditions AND the `EffectDef` enum. One scripting language. |
| D2 | Fragment granularity | Thin scenes (situations) + dual-source fragments: scene-scoped OR global-pool, joined by tags. |
| D3 | Fragment kinds | passive (auto-narration), active (choice button), check (active + skill roll + pass/fail). |
| D4 | Erotic curve | **Fluid COMPOSURE.** A stat that drifts on behavior: giving in lowers it (next resist harder), holding raises it. Arousal scales DC on top. The slippery slope is the mechanic. |
| D5 | Effect script power | **Constrained call-list** — flat statement-list of registered effect calls only. No loops, no user fns, no conditionals. Branching lives in fragment structure. |
| D6 | Save-scumming | **Reloadable for now** (dev). Seeded-RNG-in-save deferred — see Section 11. |
| D7 | Existing 62 scenes | **Not migrated.** Run on a compatibility path. Migrate opportunistically only when a scene is creatively reworked anyway. |
| D8 | Pacing fix | Ships **first and standalone** (schedule.toml only), independent of all architecture work. |

## 4. Architecture — Rhai foundation

### 4.1 Two engines

```rust
// crates/undone-scene/src/rhai_engine.rs
pub struct ScriptEngines {
    pub cond:   rhai::Engine,  // read-only API only
    pub effect: rhai::Engine,  // read API + write API
}
```

Conditions compile against an engine with **only the read API registered**. Effects compile
against an engine with read + write. Consequence: using a mutating effect inside a condition
is a **compile error for free** (the function isn't registered in `cond`), enforcing
Engineering Principle 6 at load time.

Both engines: `set_strict_variables(true)` (unknown identifier → compile error),
`set_max_operations` + `set_max_expr_depths` (Principle 5, bounded resources).

The game crate does **not** enable Rhai's `sync` feature (single-threaded floem UI thread).
The `rhai-mcp-server` is realigned to the identical feature set so authoring-time validation
matches load-time exactly.

### 4.2 Compiled scripts replace `Expr`

```rust
pub struct CompiledScript {
    pub ast:    std::sync::Arc<rhai::AST>,  // compiled once at LOAD (replaces undone_expr::Expr)
    pub source: String,                     // for error messages + AST-cache key + MCP
}
```

Every `condition: Option<Expr>` field becomes `Option<CompiledScript>`. An effect block
(`Vec<EffectDef>`) becomes a single `Option<CompiledScript>` compiled against the effect engine.

### 4.3 Binding surface (NOT World-as-custom-type)

Do **not** register `World` as a Rhai type and let scripts walk it — that leaks the mutable
state graph and defeats static validation. Instead, register the **exact method surface that
exists today** as functions on thin receiver handles:

- Read receivers: `w`, `gd`, `m`, `f`, `role`, `scene` (mirror the ~80 methods in `eval.rs`)
- Write API (effect engine only): `w.addArousal(1)`, `npc("m").addLiking(2)`,
  `scene.setFlag("x")`, etc. (mirror the ~33 `apply_effect` arms in `effects.rs`)

Handles are injected into the `rhai::Scope` per call, hold keys/IDs (not lifetime-bound
references), and are valid only for the duration of one synchronous `eval_ast_with_scope` call.
The `unsafe` (if raw pointers are used) is confined to one module with the invariant
"valid only within eval"; `Rc<RefCell<…>>` is the no-unsafe fallback, chosen at implementation
review. Single-threaded execution makes the borrow genuinely outlive every script op.

The author-facing TOML condition syntax (`w.hasTrait("SHY") && w.getSkill("FEMININITY") < 15`)
is already nearly Rhai-compatible, so existing condition strings migrate near-verbatim.

### 4.4 FAIL-FAST gate (the one critical risk)

`rhai::compile()` + `strict_variables` catches syntax and unknown *identifiers* but **NOT
unknown content IDs** — `w.hasTrait("TYPpO")` compiles clean and fails at runtime. The current
`validate_condition_ids` guarantee MUST be reconstructed, or fail-fast silently degrades from
load-time to runtime (violating Principles 1 & 4).

**Three-layer load gate per snippet** (in `loader.rs`, replacing `parse_condition_checked`):

1. `engine.compile(src)` → syntax errors map to `SceneLoadError::BadScript`.
2. `strict_variables` → unknown variable/function names fail at compile.
3. **ID validation** — two complementary mechanisms:
   - **String-literal rule:** content IDs passed to resolver functions (`hasTrait`, `getSkill`,
     `addTrait`, …) MUST be string literals. Documented and enforced by static AST inspection
     of resolver-call arguments.
   - **Load-time dry-run:** every compiled snippet is executed once against a synthetic *probe*
     World in a validation mode where `resolve_trait`/`resolve_skill`/`resolve_stat`/`resolve_arc`/
     `resolve_category` return a hard `Err` on any unknown ID — aborting the load with the
     offending ID and scene context. This turns every runtime ID error into a load error.

This gate is **non-negotiable** and is built in Phase 1. It is the load-bearing reason the
architecture preserves the project's engineering principles.

### 4.5 Performance posture

Conditions are evaluated frequently (scheduler `pick_next` scans every event's condition+trigger;
`emit_actions` re-evals each action condition). Interpreted Rhai is materially slower per-op than
the hand-written `Expr` match, but at current scale (low hundreds of conditions, evaluated on
discrete player actions — not per render frame) this is sub-millisecond. Mitigations baked in:
compile-once `Arc<AST>` reuse, one shared engine, and a source-keyed AST cache so duplicate
conditions across files share one AST. A criterion bench gates the work; **do not pre-optimize
before a bench shows a hotspot.** The concern only becomes real if the global fragment pool grows
into the thousands AND is scanned per frame — at which point a coarse Rust pre-filter tag on
fragments (checked before running Rhai) is the escape hatch.

## 5. Architecture — Fragment model & composer

### 5.1 The `Fragment` type

```rust
pub enum FragmentKind { Passive, Active, Check }
pub enum FragmentScope { SceneLocal, Global { tags: Vec<String> } }
pub enum OnceScope { Scene, Game }

pub struct Fragment {
    pub id: String,
    pub kind: FragmentKind,
    pub scope: FragmentScope,
    pub condition: Option<CompiledScript>,   // compiled at LOAD
    pub prose: String,                        // minijinja
    pub effects: Option<CompiledScript>,      // compiled at LOAD (constrained call-list)
    pub priority: i32,                        // passive ordering (default 0)
    pub weight: u32,                          // tie-break / world-move random (default 1)
    pub once: bool,
    pub once_scope: OnceScope,
    pub npc: Option<String>,                  // "m" | "f" | role id
    // active-only:
    pub label: Option<String>, pub detail: Option<String>, pub next: Vec<NextBranch>,
    // check-only:
    pub check: Option<CheckSpec>,
}
```

### 5.2 The composer (replaces `select_intro_prose` + `render_thoughts` + `run_npc_actions`)

At a point in a thin scene:

1. **Gather** — scene-local fragments + every global-pool fragment whose `tags` intersect the
   active scene's `tags`.
2. **Filter** — by compiled Rhai condition (errors → `ErrorOccurred` event, fragment excluded —
   same semantics as `eval_condition` today).
3. **Dedupe** — skip if `once` already fired (scene-scope: `ctx.fired` set; game-scope:
   `ONCE_FRAG_<id>` world flag).
4. **Split by kind:**
   - **passives** → sort by `priority` desc, then seeded-RNG weighted tie-break, then source
     order; **truncate to `max_passives` budget** (default 3, scene-overridable). Emit as
     `ProseAdded`/`ThoughtAdded`.
   - **actives + checks** → `ActionsAvailable` buttons (no budget; condition already gated
     visibility).

This maps 1:1 onto the existing `EngineEvent` enum, so **the UI layer does not change**.

**Budget** (Principle 5, bounded resources) prevents the global pool from flooding a thin scene.
Global fragments should sit at lower priority than scene-local so a scene's own voice wins the
budget.

### 5.3 Thin scene + tags

```toml
[scene]
id   = "base::coffee_shop"
pack = "base"
tags = ["public_space", "queue", "stranger_present"]   # which pool tags are live here

[scene.situation]
prose = "The coffee shop on Union and Third is not a Starbucks. You're third in line."
```

Tags are to fragments what slot names are to schedule events. The engine knows "tags match,"
never what "coffee_shop" means — setting-agnostic (Principles 2/3).

Global-pool fragments live in dedicated files (`packs/base/pool/*.toml`) loaded analogously to
scenes, so writers see all ambient content in one place. (Open question O1 if inline is
preferred.)

### 5.4 Coexistence loader (D7)

```rust
// loader.rs — dispatch per file on presence of [[fragment]]
let def = if raw_value.get("fragment").is_some() {
    resolve_fragment_scene(...)?   // new path
} else {
    resolve_scene(...)?            // existing path, untouched
};
// both → Arc<SceneDefinition>; duplicate-id / cross-ref / validation passes unchanged
```

The Rhai compile point sits **below** the format split, so the language swap (D1) and the format
split (D2) are orthogonal concerns. Legacy scenes get the Rhai condition/effect treatment too,
via a mechanical condition translation + an effects-table→call-list codemod, with `validate-pack`
proving all scenes still load before merge.

## 6. Architecture — Check mechanic & fluid composure

### 6.1 Reuse, don't rebuild

Build on the existing `checkSkill`/`checkSkillRed` + `FailRedCheck` + `get_or_roll_skill`
primitives. A `check` fragment is an **active fragment** whose button runs the roll on selection
and routes to a `pass` or `fail` branch (each: prose + constrained effects + next). Passive
fragments never roll. Checks do not advance the clock unless a branch carries `advance_time`.

```toml
[[fragment]]
id    = "hold_composure"
kind  = "check"
label = "Try to hold still"
resist = true                 # PASS = "you held"; FAIL = the desired erotic branch
skill  = "COMPOSURE"
base_dc = 40
check_type = "white"          # white = retryable; red = one-shot via FailRedCheck
condition = 'scene.hasFlag("he_leaned_in")'

  [fragment.pass]             # rarer: you kept composure
  prose   = "You hold his eyes. Steady. Your voice does what you tell it to."
  effects = 'w.addComposure(4);'

  [fragment.fail]             # failing IS the payoff
  prose   = "Your breath catches before you can stop it. Heat climbs your throat."
  effects = 'w.addArousal(2); w.addComposure(-6); scene.setFlag("flustered");'
```

### 6.2 Fluid COMPOSURE (D4)

COMPOSURE is a **new pack-data stat/skill** (none exists today) that is *not trained by practice*
but **drifts on check outcomes**:

- A resist-check **fail** (gave in) applies a COMPOSURE **decrease** → the next resist is harder.
- A resist-check **pass** (held) applies a COMPOSURE **increase** → the next resist is easier.

Combined with arousal scaling DC, this produces the self-reinforcing spiral the premise wants:
indulgence compounds. The decrease/increase amounts live on the fragment branches (authored),
not in engine code.

### 6.3 Resist DC from traits + arousal — pack-data driven

The resist difficulty rises with the PC's sexual-response traits and current `ArousalLevel`, so
having `HAIR_TRIGGER`/`SUBMISSIVE`/`EASILY_WET` makes holding composure *less* likely (failing =
payoff). To honor Principle 2 (no hardcoded content IDs in engine code), the per-trait modifier
lives in `traits.toml`:

```toml
[[trait]]
id = "HAIR_TRIGGER"
composure_penalty = 20    # raises resist DC by 20 → lowers pass target
```

The engine reads modifiers from the registry; packs tune their own erotic logic. Effective DC:
`base_dc + Σ(trait penalties) + arousal_tier_penalty`, then the existing
`target = (COMPOSURE + (50 - effective_dc)).clamp(5, 95)` math, unchanged.

### 6.4 White vs red, re-keyed to fragment id

White checks persist nothing (re-rollable each encounter; `get_or_roll_skill` only caches within
one scene run). Red checks are one-shot, recorded by the existing `FailRedCheck` effect — but
the red registry is **re-keyed from `scene_id::skill_id` to the (globally unique) fragment id**,
because a pool check can fire across many scenes and `scene_id` is no longer a stable identity.

### 6.5 UI: odds as voice, not numbers

`ActionView` gains optional check metadata (`skill_name`, internal `odds_pass`, `resist`,
`check_type`, `spent`). The button surfaces odds as a **qualitative phrase in the narrator's
voice** ("You won't hold this" / "Maybe" / "You can ride this out"), never a number — the
writing guide forbids the gamey/clinical dice-UI feel. Numeric target stays internal. Red checks
already failed render locked/struck-through. Requires a playtester + gemini-vision pass.

## 7. Phasing & sequencing

| Phase | What | Ships independently? | Needs from user |
|---|---|---|---|
| **0 — Pacing fix** | schedule.toml only: drop redundant `week>=N` floors on the romance chain, keep flag prereqs, raise on-ramp weights so explicit content is reachable week 1–2. | **Yes — ship first, tonight.** | Confirm scheduler trigger-vs-weighted order (O5). |
| **1 — Rhai foundation** | Two engines, `CompiledScript`, binding surface, the load-time dry-run fail-fast gate, MCP-server realignment. Conditions + effects migrate *under* the existing scene engine. No visible content change. | Yes (invisible plumbing swap). | — |
| **2 — Vertical slice** | Minimal fragment+check engine, just enough to run ONE new explicit escalation scene with a real fluid-composure resist check. Validates the full stack + adds reachable adult content. | Yes (adds one scene). | **Creative spec for the slice scene** (rules 1 & 11). |
| **3 — Generalize** | Full composer, global pool, budget/dedupe, coexistence loader, `[[fragment]]` schema across the engine. | Yes. | — |
| **4 — Check formalization** | COMPOSURE stat, `composure_penalty` in traits.toml, resist-DC computation, UI odds-as-phrase, red re-key. | Yes. | Pacing/tuning playtests. |

Each phase is a candidate for Workflow-orchestrated execution. Phase 0 is pure content and can
ship before any of this is built.

**Out of scope:** bulk migration of the 62 existing scenes (D7); seeded-RNG save-scum prevention
(D6, deferred).

## 8. Fail-fast preservation (summary)

The whole architecture hinges on holding the line `load > runtime > silent`:

- Conditions/effects compile to `Arc<AST>` at load (Section 4.2).
- Unknown identifiers fail at compile via `strict_variables` (4.1).
- Unknown content IDs fail at load via the string-literal rule + dry-run probe (4.4).
- Effects can't appear in conditions (separate engines, 4.1).
- Fragment conditions/effects run through the identical gate in `resolve_fragment` (5.1).
- `validate-pack` remains the merge gate proving all scenes (legacy + fragment) load.

## 9. Error handling

- **Load time:** any compile/ID error aborts pack load with scene + snippet + offending ID,
  via `SceneLoadError::BadScript` (and `SchedulerError::Validation` for schedule conditions).
- **Runtime:** a condition that errors during play excludes its fragment and emits
  `ErrorOccurred` (visible, never silently `false` — Principle 1). An effect call-list that
  errors mid-list stops and logs `ErrorOccurred`, matching today's best-effort `effect_errors`
  semantics; the constrained call-list (D5) keeps partial-application risk minimal and auditable.
- **Bounds:** `max_operations` / `max_expr_depths` cap runaway scripts (Principle 5).

## 10. Testing strategy

- **Unit:** roll math, resist-DC computation, composure drift, fragment dedupe/once, composer
  ordering + budget, the load-time gate (a typo'd trait ID must fail the test at load).
- **Contract:** real pool/scene TOML → loader → assert `SceneDefinition` shape; a fixture with a
  bad ID must produce a load error.
- **Acceptance:** `validate-pack` loads all packs clean (legacy + fragment).
- **Runtime:** playtester drives the Phase-2 slice end-to-end via dev IPC — verifies a passive
  pool voice fires, an active fragment appears, and a resist check's fail branch produces the
  hotter content; gemini-vision critique on the check button phrasing (no numeric odds).
- **Anti-circular:** the slice's acceptance is verified by the playtester/QA agent, not the
  implementing agent; fixtures come from real pack data, not inline construction.

## 11. Deferred / open questions

- **O-D6 (save-scum):** revisit locked-roll + seeded-RNG-in-save after the model proves out in dev.
- **O1:** global pool in dedicated `pool/*.toml` files (recommended) vs inline `scope="global"`.
- **O2:** default `max_passives` budget value; per-scene override knob.
- **O3:** does the scheduler eventually subsume into the fragment pool (scenes become fragments),
  or stay a separate layer that picks WHICH thin scene runs? (Recommended: stay separate for now.)
- **O4:** is COMPOSURE the single universal resist skill, or do different sexual checks roll
  different skills (STAMINA to last, COMPOSURE to stay quiet)? Start with one; expand later.
- **O5 (blocks Phase 0):** in `scheduler.rs`, are `trigger` events guaranteed-fire BEFORE the
  weighted pool, or do they join it? Determines whether the pacing fix needs weight changes or
  only condition/week changes. Verify first.
