# Engine Correctness & Safety Pass — Design

> **Date:** 2026-02-23
> **Goal:** Fix all known correctness, safety, and silent-failure issues in the engine before adding more content.

---

## Motivation

The engine has 111 tests and no stubs, but an audit revealed seven issues where
the engine is wrong, crashy, or silently swallowing errors. These must be fixed
before writing more scenes — content authors can't debug what the engine hides.

This work also establishes documented engineering principles for the project:
correctness over velocity, fail-fast over silent defaults, data-driven over hardcoded.

---

## Items

### 1. Scroll-to-bottom after action selection

**Problem:** When the player chooses an action, new prose appends to `signals.story`,
floem rebuilds the `rich_text` widget inside `scroll()`, and the scroll position
resets to 0. The player loses their place.

**Fix:** Add a `scroll_gen: RwSignal<u64>` signal to `AppSignals`. Chain
`.scroll_to_percent(move || { scroll_gen.get(); 100.0 })` on the scroll widget
in `story_panel()`. Increment `scroll_gen` in `process_events()` whenever
`ProseAdded` fires. Floem's `scroll_to_percent` is layout-deferred — it resolves
after content reflow, so 100% always means the true new bottom.

**Crates touched:** `undone-ui`

### 2. Scene goto cross-reference validation at load time

**Problem:** A `goto = "base::nonexistent_scene"` in TOML is only discovered at
runtime, when the engine silently logs to `eprintln!` and drops the transition.
Content authors have no feedback.

**Fix:** After all scenes from all packs are loaded, run a validation pass over
every `NextBranch` in every loaded scene definition. Any `goto` target not present
in the scene registry becomes a `SceneLoadError`. The game refuses to start with
broken cross-references.

**Crates touched:** `undone-scene` (loader), `undone-packs` (registry surface)

### 3. Scene stack depth guard

**Problem:** A TOML cycle (`scene A goto B, scene B goto A`) or deeply nested
sub-scenes grow the engine's `Vec<SceneFrame>` until OOM. No guard exists.

**Fix:** Add `const MAX_STACK_DEPTH: usize = 32` in the engine. Check depth in
`start_scene()` before pushing a new frame. If exceeded, emit
`EngineEvent::ProseAdded("[Engine error: scene stack overflow — possible cycle]")`
and `EngineEvent::SceneFinished` to safely exit.

**Crates touched:** `undone-scene`

### 4. NPC personality rendering bug

**Problem:** The stats sidebar shows `PersonalityId(Spur { idx: NonZeroU32(3) })`
for active NPC personality because `NpcSnapshot` uses `format!("{:?}", npc.personality)`,
and `PersonalityId` is a newtype around an opaque `Spur`.

**Fix:** Add `fn personality_name(&self, id: PersonalityId) -> &str` to
`PackRegistry` (reverse lookup from the interner). Use it in `NpcSnapshot::from()`
by passing the registry, or by resolving the name at snapshot-creation time.

**Crates touched:** `undone-packs` (registry method), `undone-ui` (snapshot creation)

### 5. Silent condition evaluation errors

**Problem:** Condition eval errors are `.unwrap_or(false)` throughout the engine.
A broken condition (typo in method name, wrong argument type) silently hides the
action — the content author never knows.

**Fix:** Before defaulting to `false`, log the error with the condition string and
scene/action context via `eprintln!`. This preserves the safe default (broken
conditions are conservative) while making failures visible during development.
Future: a diagnostics panel in the UI.

**Crates touched:** `undone-scene` (engine.rs — condition eval call sites)

### 6. Unknown scene ID in StartScene

**Problem:** If `StartScene` is called with an unknown scene ID (typo, removed
scene, cross-pack reference error), the engine logs to `eprintln!` and returns
silently. The UI shows stale actions from the previous scene.

**Fix:** When the scene ID is not found in the registry, emit
`EngineEvent::ProseAdded("[Error: scene not found: {id}]")` and
`EngineEvent::SceneFinished`. This makes the error visible in the game window
and cleanly exits the broken scene rather than leaving stale state.

**Crates touched:** `undone-scene`

### 7. Data-driven opening scene and scheduler slot

**Problem:** `"base::rain_shelter"` (opening scene) and `"free_time"` (scheduler
slot) are hardcoded in two places in the UI layer. Changing the opening experience
requires a code change.

**Fix:** Add `opening_scene` and `default_slot` fields to the pack manifest.
`PackRegistry` exposes these after load. The UI reads from the registry instead
of hardcoded strings. The base pack's `pack.toml` declares:

```toml
[pack]
opening_scene = "base::rain_shelter"
default_slot  = "free_time"
```

**Crates touched:** `undone-packs` (manifest, registry), `undone-ui` (lib.rs, left_panel.rs)

---

## Engineering Principles (to document in CLAUDE.md)

These principles govern all engine development. They are not aspirational — they
are constraints. Violating them is a bug, not a trade-off.

1. **Fail fast, fail loud.** Invalid data should be caught at load time, not
   runtime. When runtime errors are unavoidable, they must be visible — never
   silently swallowed.

2. **No hardcoded content IDs in engine code.** Scene IDs, slot names, skill
   names, and trait names belong in pack data files. The engine reads them from
   the registry. The only exception is truly structural IDs (like "FEMININITY")
   that the engine needs for core mechanics — and even those should be declared
   as required skills in the pack manifest, not magic strings.

3. **Data-driven over code-driven.** If a value could reasonably come from a
   pack file, it should. The engine is a platform — it should not know what
   game it's running.

4. **No silent defaults for content errors.** A typo in a condition, an unknown
   trait name, a broken goto target — these are bugs in the content, not edge
   cases to handle gracefully. They should produce visible errors at the earliest
   possible moment (load time > runtime > silent).

5. **Bounded resources.** Stacks, buffers, and accumulating strings must have
   depth/size limits. Unbounded growth is a latent crash.

6. **Separation of concerns across crate boundaries.** Engine logic does not
   belong in the UI crate. UI concerns do not belong in the domain crate. The
   dependency direction is enforced by the workspace and must be maintained.

7. **Tests before content.** New engine capabilities get tests before scenes
   use them. Content authors should never be the first to discover a broken
   engine feature.

---

## Out of Scope (documented for future sessions)

- Move `game_state.rs` from `undone-ui` to a runtime crate (architecture)
- Prose accumulation limits / scrollback (architecture)
- Remove unused `SceneCtx.weighted_map` (cleanup)
- Move `job_title` / `allow_anal` to flag/stat system (consistency)
- Expand female NPC eval surface (feature)
- Race selection in char creation (feature)
- Keyboard controls redesign (feature)
- Settings tab UI (feature)

---

*Design session: 2026-02-23. Authors: YJK + Claude.*
