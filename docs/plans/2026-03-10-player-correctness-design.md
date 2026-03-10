# Player Correctness Runtime Design

## Goal

Establish a robust, code-driven correctness layer for Undone so player-visible behavior can be verified without relying on manual clicking or screenshot interpretation. The design must support three downstream needs with one shared contract:

1. runtime control inside the app
2. acceptance-style flow tests in code
3. MCP/dev-tool inspection and control for agents

This is not a narrow test-infrastructure task. It is a runtime architecture task because correctness currently depends on logic spread across UI event handlers, game-state helpers, and dev IPC. That fragmentation is tolerable for manual development, but it is not robust enough for automated player-correctness verification.

---

## Problem Statement

The current engine has strong unit and integration coverage, but the player-visible runtime still has several structural weaknesses:

- The runtime control path is split across multiple files and call sites.
- The app has no single serializable snapshot of what the player currently sees.
- Dev IPC exposes world-state mutations and scene jumps, but not full runtime observability or player-like scene progression.
- Acceptance-style verification currently depends on ad hoc test helpers or live app interaction.
- Some correctness invariants are encoded indirectly rather than enforced by one runtime controller.

The most important concrete example is active NPC binding. Today, [lib.rs](C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/lib.rs) starts a scene and only then sets fallback active NPC bindings. That means intro prose, intro thoughts, and intro variants can observe a different runtime contract than action execution. Even if current content mostly avoids this edge, the architecture is wrong for a player-correctness system.

The broader issue is that the UI is currently both:

- the renderer of player-visible state
- the owner of core runtime progression semantics

Those responsibilities need a cleaner boundary.

---

## Design Principles

1. Player-visible correctness is a first-class runtime contract.
2. One control path must drive scene start, action choice, continue, and resume.
3. One snapshot format must describe current runtime state for tests, dev IPC, and MCP.
4. Engine/runtime invariants should be enforced in code, not inferred from UI state.
5. The GUI should consume runtime state, not define correctness-critical transitions independently.
6. Acceptance tests should assert on stable structured data, not screenshots.
7. Live black-box runtime checks should be reserved for UI-only behavior that code-level flow tests cannot prove.

---

## Scope

### In Scope

- central runtime controller for scene and flow progression
- structured runtime snapshot combining persisted and transient state
- code-driven acceptance harness over real runtime behavior
- dev IPC expansion to expose runtime-state inspection and player-like commands
- MCP expansion to wrap the richer dev IPC surface
- correctness fixes required by the new harness, including intro-time NPC binding

### Out of Scope

- redesigning the content model
- replacing the existing scene engine
- replacing live MCP playtesting with pure code tests
- broad UI redesign unrelated to correctness
- screenshot/image analysis as a primary correctness mechanism

---

## Current State Audit

### Runtime control is fragmented

The current runtime path spans:

- [lib.rs](C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/lib.rs)
- [left_panel.rs](C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/left_panel.rs)
- [game_state.rs](C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/game_state.rs)
- [dev_ipc.rs](C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/dev_ipc.rs)

The code already contains useful reusable pieces such as `start_scene`, `process_events`, `reset_scene_ui_state`, `resume_current_world`, and `game_state_snapshot`, but they do not yet form a single authoritative runtime API.

### Snapshot coverage is too thin

`game_state_snapshot()` in [dev_ipc.rs](C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/dev_ipc.rs) only captures world-oriented state:

- current scene id
- time
- player summary
- flags
- arc states

That is not enough to verify player-visible correctness. It omits:

- visible prose
- visible action list
- continuation state
- active tab and phase
- active NPC display state
- transient UI/runtime state

### Dev tooling is mutation-heavy and observation-light

The current dev panel and IPC allow useful mutations:

- jump to scene
- set stat
- set/remove flags
- advance time
- adjust NPC liking

But there is no direct command for:

- choose a visible action by id
- continue after scene finish
- retrieve what prose/actions the player currently sees
- inspect runtime phase/tab state in a structured way

### Acceptance testing is present but not systematized

The repo already has strong integration-style tests around scheduler, save/load, IPC, and event processing. What it lacks is a named, deliberate player-flow harness that says:

- initialize runtime
- execute player-like commands
- capture a runtime snapshot
- assert on player-visible outputs and persisted state

That distinction matters because the missing piece is not “more tests”; it is “the right abstraction for testing the product contract”.

---

## Proposed Architecture

The design introduces three connected layers:

1. Runtime Controller
2. Runtime Snapshot
3. Acceptance Harness

The controller owns progression semantics. The snapshot exposes runtime truth. The harness verifies player-visible flows against that truth.

### 1. Runtime Controller

Add a dedicated runtime-control module in `undone-ui`, tentatively `runtime_controller.rs`.

This module becomes the sole owner of correctness-critical transitions:

- start new runtime scene
- bind active NPCs before scene render
- choose action
- continue after scene finish
- resume from current world after load
- jump to scene in dev mode
- apply engine events into signals/runtime state
- emit consistent runtime snapshots

This controller should orchestrate existing helpers rather than replacing the scene engine. The engine remains the source of scene semantics. The controller becomes the source of app/runtime semantics.

#### Proposed API

```rust
pub struct RuntimeController<'a> {
    pub gs: &'a mut GameState,
    pub signals: AppSignals,
}

impl<'a> RuntimeController<'a> {
    pub fn launch_initial_scene(&mut self) -> RuntimeCommandResult;
    pub fn start_scene(&mut self, scene_id: impl Into<String>) -> RuntimeCommandResult;
    pub fn choose_action(&mut self, action_id: &str) -> RuntimeCommandResult;
    pub fn continue_flow(&mut self) -> RuntimeCommandResult;
    pub fn jump_to_scene(&mut self, scene_id: &str) -> RuntimeCommandResult;
    pub fn resume_from_current_world(&mut self) -> RuntimeCommandResult;
    pub fn snapshot(&self) -> RuntimeSnapshot;
}
```

The result type should capture:

- emitted events or derived state if useful
- whether a scene finished
- whether a new scene started
- current scene id after completion

The exact result shape can stay modest. The critical requirement is that callers no longer need to reproduce controller logic manually.

### 2. Runtime Snapshot

Add a serializable `RuntimeSnapshot` that describes what the player can currently observe plus the persisted runtime context needed to reason about it.

#### Proposed shape

```rust
pub struct RuntimeSnapshot {
    pub phase: String,
    pub tab: String,
    pub current_scene_id: Option<String>,
    pub awaiting_continue: bool,
    pub story_paragraphs: Vec<String>,
    pub visible_actions: Vec<VisibleActionSnapshot>,
    pub highlighted_action_index: Option<usize>,
    pub active_npc: Option<NpcSnapshotData>,
    pub player: PlayerSnapshotData,
    pub world: WorldSummarySnapshot,
    pub init_error: Option<String>,
}

pub struct VisibleActionSnapshot {
    pub id: String,
    pub label: String,
    pub detail: String,
}
```

`RuntimeSnapshot` should be produced from two sources:

- persisted runtime state in `GameState`
- transient UI/runtime state in `AppSignals`

This is intentional. Player correctness depends on both.

#### Snapshot invariants

- `visible_actions` must match what the player can currently choose.
- `story_paragraphs` must match the rendered paragraph list as currently accumulated.
- `awaiting_continue` must explain why actions may be absent.
- `current_scene_id` must reflect engine runtime, not inferred story state.
- `phase` and `tab` must reflect current app navigation state.
- `active_npc` must reflect what sidebar/UI logic currently surfaces, not a separate hidden source.

### 3. Acceptance Harness

Add a dedicated acceptance-flow test harness in `undone-ui`, likely under a new test-support module such as `runtime_test_support.rs`.

This harness should:

- build a real `PreGameState`
- start a real `GameState`
- create `AppSignals`
- drive the runtime via `RuntimeController`
- assert against `RuntimeSnapshot`

This is the mainline correctness strategy.

---

## Runtime Semantics To Centralize

The new controller must own the following semantics so there is exactly one authoritative implementation.

### Scene start semantics

Current behavior:

- clear stale scene UI state
- send `StartScene`
- then set fallback active male/female
- drain events
- process events into signals

Target behavior:

1. clear transient scene UI state
2. compute and bind active NPCs before intro-time rendering can observe the scene
3. start scene
4. drain events
5. process events
6. update player snapshot and runtime snapshot

This fixes the intro-time NPC inconsistency and makes scene start deterministic for both tests and dev tooling.

### Action choice semantics

Current behavior is mostly in [left_panel.rs](C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/left_panel.rs) via `dispatch_action`.

Target behavior:

- echo chosen action consistently if that remains a UI contract
- send engine action
- process events
- mark `awaiting_continue` only through the controller
- return a command result and update runtime snapshot

The UI can still trigger action choice, but it should call the controller instead of recreating runtime semantics.

### Continue semantics

Current behavior is in `continue_to_next_scene`.

Target behavior:

- one shared scheduler-pick path
- one shared `once_only` flag path
- one shared new-scene startup path
- same behavior used by normal gameplay and any dev/runtime command that advances flow

### Resume semantics

`resume_current_world` already contains useful logic in [game_state.rs](C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/game_state.rs). The controller should wrap and normalize it so resume, continue, and initial scene launch share the same downstream scene-start contract.

### Dev jump semantics

`jump_to_scene` in [dev_ipc.rs](C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/dev_ipc.rs) should become a thin wrapper over the controller, not a parallel path.

---

## Correctness Issues This Design Intentionally Fixes

### Intro-time NPC binding

This is the first concrete engine/runtime correctness fix the implementation must land.

Current issue:

- fallback active NPC bindings happen after `StartScene`
- intro prose, intro thoughts, and intro variants can therefore see no bound NPC even though later action/effect logic can

Required fix:

- bind active NPCs before scene start for any scene path that relies on fallback binding
- make the contract explicit in code and tests

This should be validated by new acceptance tests that prove intro-time NPC access is stable.

### Divergent progression paths

The implementation must collapse the following into one progression path:

- initial app-driven scene launch
- continue button
- dev jump
- load/resume

These may still have different entrypoints, but they must delegate into one runtime controller.

### Snapshot drift

Structured runtime truth must come from one snapshot builder. Tests, dev IPC, and MCP should not each assemble partial views independently.

### UI-engine seam ambiguity

The design should make it obvious which layer owns each rule:

- engine owns scene semantics
- scheduler owns eligibility/picks
- runtime controller owns app progression semantics
- UI owns presentation and input wiring

---

## Dev IPC Design

Expand [dev_ipc.rs](C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/dev_ipc.rs) to expose both richer observation and player-like control.

### New commands

Add:

- `GetRuntimeState`
- `ChooseAction { action_id }`
- `ContinueScene`
- `SetTab { tab }`

Optional only if required:

- `SetPhase { phase }`
- `ListScenes`

### Command rules

- `ChooseAction` must fail loudly if the action is not currently visible.
- `ContinueScene` must fail loudly if `awaiting_continue` is false.
- `SetTab` should validate tab names and respect dev-mode constraints.
- `GetRuntimeState` should return serialized `RuntimeSnapshot`.

### Existing commands that should be reimplemented through controller paths

- `JumpToScene`
- `GetState`

`GetState` can remain as a world-oriented snapshot for compatibility, but `GetRuntimeState` becomes the primary debugging surface.

---

## MCP Design

Extend [server.rs](C:/Users/YJK/dev/mirror/undone/tools/game-input-mcp/src/server.rs) to expose the richer runtime contract through typed wrappers.

### New MCP tools

- `get_runtime_state()`
- `choose_action(action_id)`
- `continue_scene()`
- `set_tab(tab)`

### MCP principles

- Use stable ids and structured JSON, not fragile UI label matching.
- Treat screenshot capture as optional corroboration, not primary truth.
- Keep raw `dev_command` for escape-hatch debugging, but agents should normally use typed tools.

### Why this matters for agents

Agents should be able to:

1. launch game in dev mode
2. query runtime state
3. choose visible action by id
4. continue flow
5. inspect resulting prose/actions/state
6. repeat deterministically

That creates an actual machine-usable runtime contract instead of a click-and-screenshot loop.

---

## Acceptance Harness Design

The acceptance harness should live in Rust tests and operate over the real runtime path, not a mock facade.

### Test fixture requirements

The fixture should provide:

- loaded packs/scenes/scheduler
- a reproducible RNG seed
- real `GameState`
- real `AppSignals`
- helper to build `RuntimeController`
- helper to snapshot current runtime state

### Canonical acceptance flows

#### Flow A: New game launch

Verify:

- initial runtime launches the correct first playable scene
- visible actions exist and match expectations
- story output is non-empty
- runtime snapshot is coherent

#### Flow B: Scene action progression

Verify:

- choosing a visible action mutates story output
- world state changes appropriately
- continuation state is correct
- next scene behavior is correct after continue

#### Flow C: Once-only scene behavior

Verify:

- once-only scenes set the persistent `ONCE_<scene_id>` flag exactly when they should
- continuing the game does not re-serve the same scene if it is once-only

#### Flow D: Save/load resume

Verify:

- stale runtime state does not survive load
- opening scene is not replayed
- resumed scheduler pick comes from persisted world state
- runtime snapshot after resume reflects the resumed scene, not prior stale UI

#### Flow E: Dev jump

Verify:

- jump clears stale story/actions/continue state
- jumped scene has coherent visible prose and actions
- runtime tab behavior remains correct

#### Flow F: Intro-time NPC access

Verify:

- intro prose or intro variants can safely observe bound NPC context
- active NPC snapshot is coherent immediately after scene start

#### Flow G: Error visibility

Verify:

- runtime errors surface in player-visible story output
- runtime snapshot reflects those diagnostics

### Harness non-goals

- do not recreate floem rendering behavior in tests
- do not test pixel layout
- do not depend on screenshot capture

---

## Live Runtime Verification Strategy

This design does not eliminate live runtime verification. It narrows it to the places where it is actually valuable.

### Live runtime checks should cover

- tab wiring
- dev panel affordances
- keyboard navigation focus behavior
- scroll behavior
- any mismatch between structured runtime snapshot and what the rendered app visibly shows

### Live runtime checks should not be the primary proof for

- scheduler picks
- scene progression correctness
- once-only flags
- save/load resume
- effect-driven player/world state mutations

Those should be proven through the acceptance harness first.

---

## Module-Level Changes

### `crates/undone-ui`

Add:

- `runtime_controller.rs`
- optionally `runtime_snapshot.rs`
- optionally `runtime_test_support.rs`

Refactor:

- [lib.rs](C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/lib.rs)
- [left_panel.rs](C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/left_panel.rs)
- [game_state.rs](C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/game_state.rs)
- [dev_ipc.rs](C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/dev_ipc.rs)
- [dev_panel.rs](C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/dev_panel.rs)

### `tools/game-input-mcp`

Refactor:

- [server.rs](C:/Users/YJK/dev/mirror/undone/tools/game-input-mcp/src/server.rs)

### Docs

Update:

- `docs/engine-contract.md`
- `HANDOFF.md`

Potentially update:

- `docs/plans/2026-02-21-engine-design.md`

The runtime controller and snapshot contract are engine-adjacent enough that the living design doc should reflect them once implemented.

---

## Rollout Strategy

### Phase 1: Runtime controller extraction

Extract and centralize shared scene/action/continue/resume logic without expanding external tooling yet.

Success condition:

- UI paths are routed through the controller
- existing tests remain green

### Phase 2: Runtime snapshot

Add `RuntimeSnapshot` and snapshot builder.

Success condition:

- dev IPC and tests can retrieve structured player-visible state

### Phase 3: Acceptance harness

Write failing acceptance tests against the controller and snapshot.

Success condition:

- tests reproduce at least one currently unguarded correctness edge, starting with intro-time NPC binding or equivalent runtime-path drift

### Phase 4: Correctness fixes

Fix issues exposed by the new harness.

Success condition:

- acceptance suite passes
- no duplicate runtime control paths remain for covered behavior

### Phase 5: Dev IPC and MCP expansion

Expose runtime-state and player-like commands through dev IPC and MCP.

Success condition:

- agents can drive the live game through stable ids and structured state, not screenshot guesses

### Phase 6: Targeted live verification

Run limited runtime/manual or MCP smoke checks for UI-only behaviors.

Success condition:

- UI-specific risks are covered without turning the whole system into a fragile black-box suite

---

## Risks And Mitigations

### Risk: Controller extraction becomes a hidden engine rewrite

Mitigation:

- keep the scene engine untouched unless a correctness issue forces change
- focus on routing and orchestration, not new engine behavior

### Risk: Snapshot duplicates too much UI state

Mitigation:

- snapshot only what is player-visible and correctness-relevant
- avoid view-layout data or styling data

### Risk: Acceptance tests become brittle to prose churn

Mitigation:

- assert on structural behavior first
- when asserting prose, check stable substrings or paragraph count/shape rather than entire scene bodies unless exact text is the contract

### Risk: MCP surface becomes too broad

Mitigation:

- expose only stable, player-meaningful commands
- keep raw `dev_command` as escape hatch

### Risk: Mixed state sources create inconsistent snapshots

Mitigation:

- one snapshot builder, one serialization contract
- no parallel ad hoc snapshot assembly in tests or MCP code

---

## Success Criteria

The design is successful when all of the following are true:

- one runtime controller owns scene launch, action choice, continue, jump, and resume semantics
- a single `RuntimeSnapshot` describes current player-visible state
- code-driven acceptance tests can prove core player flows without live GUI interaction
- intro-time NPC access and similar runtime invariants are enforced, not incidental
- dev IPC can return runtime state and drive player-like progression commands
- MCP exposes those capabilities with typed wrappers
- live runtime checks are reduced to UI-specific concerns instead of carrying product correctness alone

---

## Fresh-Session Execution Target

The follow-on implementation session should:

1. implement the runtime controller
2. implement the runtime snapshot
3. write failing acceptance tests
4. fix exposed correctness issues
5. expand dev IPC and MCP
6. update docs and handoff

That session should not need to redesign the architecture. This document is intended to be sufficient for full execution.
