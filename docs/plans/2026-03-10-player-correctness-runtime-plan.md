# Player Correctness Runtime Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use ops-executing-plans to implement this plan task-by-task.

**Goal:** Build a single runtime-control and snapshot contract that can prove player-visible correctness in code, expose it through dev IPC/MCP, and eliminate correctness drift between gameplay, save resume, and dev tooling.

**Architecture:** `undone-ui` gains a dedicated runtime controller and structured runtime snapshot. Existing UI and dev IPC paths are refactored to call that controller instead of reimplementing flow semantics. Acceptance tests drive the real runtime through the controller and assert on snapshots. `game-input-mcp` is extended with typed wrappers over the richer dev IPC surface so agents can inspect and drive the live game deterministically.

**Tech Stack:** Rust workspace, floem 0.2.0, serde/serde_json, rmcp, existing scene engine and scheduler

**Design Doc:** `docs/plans/2026-03-10-player-correctness-design.md`

---

### Task 1: Add failing tests for intro-time NPC binding and current runtime snapshot gaps

**Files:**
- Modify: `crates/undone-ui/src/lib.rs`
- Modify: `crates/undone-ui/src/game_state.rs`
- Test: `crates/undone-ui/src/lib.rs`

**Step 1: Write failing test for intro-time NPC access**

Add a test that starts a scene whose intro path requires active `m` or `f` binding immediately at scene start. The test should fail under the current runtime path if fallback binding happens after `StartScene`.

The test should assert:

```rust
assert!(
    !events.iter().any(|event| matches!(event, EngineEvent::ErrorOccurred(_))),
    "intro-time NPC access must be valid during scene start"
);
```

**Step 2: Write failing test describing missing player-visible snapshot data**

Add a failing test that demonstrates the current snapshot is insufficient for acceptance use, for example by asserting that a future runtime snapshot must include visible story and actions.

Use a placeholder target type or temporary test helper if needed. The point is to lock the contract before implementation.

**Step 3: Run the focused tests to verify RED**

Run:

```bash
cargo test -p undone-ui intro_time_npc
```

and

```bash
cargo test -p undone-ui runtime_snapshot
```

Expected: FAIL for the intended reasons.

**Step 4: Commit**

```bash
git add crates/undone-ui/src/lib.rs crates/undone-ui/src/game_state.rs
git commit -m "test: lock intro-time NPC binding and runtime snapshot gaps"
```

---

### Task 2: Introduce `RuntimeSnapshot` and snapshot builder

**Files:**
- Create: `crates/undone-ui/src/runtime_snapshot.rs`
- Modify: `crates/undone-ui/src/lib.rs`
- Modify: `crates/undone-ui/src/dev_ipc.rs`
- Test: `crates/undone-ui/src/runtime_snapshot.rs`

**Step 1: Write the failing snapshot tests**

Create tests in `runtime_snapshot.rs` that assert a snapshot includes:

- phase
- tab
- current scene id
- awaiting continue
- story paragraphs
- visible actions
- active NPC
- player summary
- flags and arc states

Use real `AppSignals` and a small `GameState` fixture where practical.

**Step 2: Implement `RuntimeSnapshot`**

Create:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeSnapshot { ... }
```

and helper snapshot structs for visible actions, player summary, active NPC, and world summary.

**Step 3: Implement `snapshot_runtime(signals, gs)`**

Build the snapshot from:

- `AppSignals`
- `GameState`
- existing `PlayerSnapshot`
- existing NPC snapshot shape where reusable

Story should be exposed as paragraph strings by splitting the accumulated story consistently with the current paragraph model.

**Step 4: Update `lib.rs` exports and any minimal call sites**

Expose the new module through `lib.rs`.

**Step 5: Run tests to verify GREEN**

Run:

```bash
cargo test -p undone-ui runtime_snapshot
```

Expected: PASS

**Step 6: Commit**

```bash
git add crates/undone-ui/src/runtime_snapshot.rs crates/undone-ui/src/lib.rs crates/undone-ui/src/dev_ipc.rs
git commit -m "feat(ui): add structured runtime snapshot for player-visible state"
```

---

### Task 3: Extract a dedicated runtime controller with a single scene-start path

**Files:**
- Create: `crates/undone-ui/src/runtime_controller.rs`
- Modify: `crates/undone-ui/src/lib.rs`
- Modify: `crates/undone-ui/src/left_panel.rs`
- Modify: `crates/undone-ui/src/game_state.rs`
- Test: `crates/undone-ui/src/runtime_controller.rs`

**Step 1: Write failing controller tests**

Write tests for:

- `start_scene()` clears stale scene UI state
- `start_scene()` binds fallback active NPCs before intro-time rendering
- `continue_flow()` starts the next eligible scene and applies `once_only`
- `jump_to_scene()` reuses the same scene-start path

**Step 2: Implement the controller skeleton**

Add a `RuntimeController` that wraps:

- `GameState`
- `AppSignals`

Provide methods:

```rust
pub fn start_scene(&mut self, scene_id: impl Into<String>) -> RuntimeCommandResult;
pub fn choose_action(&mut self, action_id: &str) -> RuntimeCommandResult;
pub fn continue_flow(&mut self) -> RuntimeCommandResult;
pub fn jump_to_scene(&mut self, scene_id: &str) -> RuntimeCommandResult;
pub fn resume_from_current_world(&mut self) -> RuntimeCommandResult;
pub fn snapshot(&self) -> RuntimeSnapshot;
```

**Step 3: Move fallback NPC binding ahead of scene start**

Refactor the current `start_scene` semantics so active NPC bindings are established before intro render can observe the scene.

This is the required correctness fix for this task.

**Step 4: Replace direct runtime flow calls in `lib.rs` and `left_panel.rs`**

Update:

- initial scene launch in `app_view`
- action dispatch path
- continue path

so they delegate to the controller.

**Step 5: Run focused tests**

Run:

```bash
cargo test -p undone-ui runtime_controller
```

Expected: PASS

**Step 6: Commit**

```bash
git add crates/undone-ui/src/runtime_controller.rs crates/undone-ui/src/lib.rs crates/undone-ui/src/left_panel.rs crates/undone-ui/src/game_state.rs
git commit -m "refactor(ui): centralize runtime flow in controller"
```

---

### Task 4: Route save resume and initial scene launch through the same controller semantics

**Files:**
- Modify: `crates/undone-ui/src/game_state.rs`
- Modify: `crates/undone-ui/src/lib.rs`
- Modify: `crates/undone-ui/src/runtime_controller.rs`
- Test: `crates/undone-ui/src/game_state.rs`

**Step 1: Write failing tests for shared progression semantics**

Add tests asserting:

- load/resume uses the same scene-start contract as normal gameplay
- opening scene is not replayed on resume
- runtime snapshot after resume reflects resumed scene state and cleared transient UI state

**Step 2: Refactor resume helpers**

Keep `resume_current_world()` and related helpers if useful, but ensure the authoritative start/continue/resume transition path lives in the controller.

Avoid duplicate logic for:

- `once_only` flag handling
- UI reset
- event draining
- player snapshot refresh

**Step 3: Run the focused tests**

Run:

```bash
cargo test -p undone-ui resume_current_world
```

Expected: PASS

**Step 4: Commit**

```bash
git add crates/undone-ui/src/game_state.rs crates/undone-ui/src/lib.rs crates/undone-ui/src/runtime_controller.rs
git commit -m "refactor(ui): unify resume and initial runtime progression"
```

---

### Task 5: Add acceptance-flow test support and first end-to-end runtime acceptance tests

**Files:**
- Create: `crates/undone-ui/src/runtime_test_support.rs`
- Modify: `crates/undone-ui/src/lib.rs`
- Test: `crates/undone-ui/src/runtime_test_support.rs`

**Step 1: Write the failing acceptance tests**

Create acceptance-style tests that drive the real runtime via the controller and assert on `RuntimeSnapshot`.

Minimum flows:

- new game launch exposes visible prose and visible choices
- choosing an action updates story output and progression state
- continuing to the next scene changes `current_scene_id` or produces the correct no-scene state

**Step 2: Implement test support helpers**

Provide helpers for:

- `make_test_pre_state()`
- `make_test_game_state()`
- `make_test_signals()`
- `make_runtime_controller()`
- `snapshot()`

Use real loaded packs/scenes/scheduler where practical.

**Step 3: Run the focused acceptance tests**

Run:

```bash
cargo test -p undone-ui acceptance_runtime
```

Expected: PASS

**Step 4: Commit**

```bash
git add crates/undone-ui/src/runtime_test_support.rs crates/undone-ui/src/lib.rs
git commit -m "test(ui): add acceptance-style runtime flow harness"
```

---

### Task 6: Extend acceptance coverage for once-only, save/load resume, and error visibility

**Files:**
- Modify: `crates/undone-ui/src/runtime_test_support.rs`
- Modify: `crates/undone-ui/src/game_state.rs`
- Test: `crates/undone-ui/src/runtime_test_support.rs`

**Acceptance Criteria:**
- Player can progress through a once-only scene and it does not repeat incorrectly.
- Player can load a save and resume without stale runtime state or opening-scene replay.
- Player sees runtime errors surfaced as visible diagnostics in story output.

**Step 1: Write the acceptance tests**

Add tests for:

- once-only persistence
- save/load runtime reset
- visible error output in snapshot story paragraphs

**Step 2: Implement minimal fixes required by those tests**

Do not broaden scope. Only fix the behavior the acceptance tests expose.

**Step 3: Run the focused suite**

Run:

```bash
cargo test -p undone-ui acceptance_runtime
```

Expected: PASS

**Step 4: Commit**

```bash
git add crates/undone-ui/src/runtime_test_support.rs crates/undone-ui/src/game_state.rs
git commit -m "test(ui): cover once-only, resume, and visible runtime errors"
```

---

### Task 7: Expand dev IPC with runtime-state and player-like progression commands

**Files:**
- Modify: `crates/undone-ui/src/dev_ipc.rs`
- Modify: `crates/undone-ui/src/runtime_controller.rs`
- Test: `crates/undone-ui/src/dev_ipc.rs`

**Step 1: Write failing dev IPC tests**

Add tests for commands:

- `get_runtime_state`
- `choose_action`
- `continue_scene`
- `set_tab`

Tests should assert that invalid action ids and invalid continue attempts return explicit errors.

**Step 2: Extend the command enum**

Add:

```rust
GetRuntimeState,
ChooseAction { action_id: String },
ContinueScene,
SetTab { tab: String },
```

**Step 3: Reimplement jump/runtime commands through the controller**

`JumpToScene`, `GetRuntimeState`, `ChooseAction`, and `ContinueScene` should delegate to `RuntimeController`.

Keep `GetState` if needed for backward compatibility, but make `GetRuntimeState` the primary player-visible debugging surface.

**Step 4: Run the focused tests**

Run:

```bash
cargo test -p undone-ui dev_ipc
```

Expected: PASS

**Step 5: Commit**

```bash
git add crates/undone-ui/src/dev_ipc.rs crates/undone-ui/src/runtime_controller.rs
git commit -m "feat(ui): extend dev IPC with runtime-state and progression commands"
```

---

### Task 8: Surface runtime-state in the dev panel and remove duplicated flow logic

**Files:**
- Modify: `crates/undone-ui/src/dev_panel.rs`
- Modify: `crates/undone-ui/src/dev_ipc.rs`
- Modify: `crates/undone-ui/src/runtime_snapshot.rs`
- Test: `crates/undone-ui/src/dev_panel.rs`

**Step 1: Write failing dev panel tests where practical**

At minimum, add tests for any pure helper functions added to support richer runtime display or action routing.

**Step 2: Update the dev panel inspector**

Switch the state inspector from the thinner game-state snapshot to `RuntimeSnapshot` or expose both if both are still useful.

**Step 3: Ensure dev actions use shared runtime semantics**

If the dev panel triggers scene jumps or future action controls directly, route them through the same command/controller path rather than local custom logic.

**Step 4: Run the focused tests**

Run:

```bash
cargo test -p undone-ui dev_panel
```

Expected: PASS

**Step 5: Commit**

```bash
git add crates/undone-ui/src/dev_panel.rs crates/undone-ui/src/dev_ipc.rs crates/undone-ui/src/runtime_snapshot.rs
git commit -m "refactor(ui): align dev panel with runtime snapshot contract"
```

---

### Task 9: Extend `game-input-mcp` with typed runtime-state and progression tools

**Files:**
- Modify: `tools/game-input-mcp/src/server.rs`
- Test: `tools/game-input-mcp/src/server.rs`

**Step 1: Write failing tests for request-shape helpers if applicable**

If the MCP server has unit-testable helper functions or serialization helpers, add tests first. If not, keep the TDD cycle on the Rust helper functions you introduce in support of the new tools.

**Step 2: Add typed MCP wrappers**

Add tools:

- `get_runtime_state`
- `choose_action`
- `continue_scene`
- `set_tab`

Each should delegate to `dev_command` using structured JSON payloads.

**Step 3: Update server instructions**

Document the new tool surface in `get_info()`.

**Step 4: Build the tools workspace**

Run:

```bash
cd tools && cargo build --release
```

Expected: PASS

**Step 5: Commit**

```bash
git add tools/game-input-mcp/src/server.rs
git commit -m "feat(mcp): add runtime-state and progression tool wrappers"
```

---

### Task 10: Add acceptance tests for the dev-runtime contract

**Files:**
- Modify: `crates/undone-ui/src/dev_ipc.rs`
- Modify: `tools/game-input-mcp/src/server.rs`
- Test: `crates/undone-ui/src/dev_ipc.rs`

**Acceptance Criteria:**
- Agent can request current runtime state and receive visible prose/actions.
- Agent can choose a visible action by stable id.
- Agent can continue only when the runtime is awaiting continue.
- Invalid runtime commands fail loudly with usable messages.

**Step 1: Write the acceptance-style dev IPC tests**

Use a real `GameState` and `AppSignals` fixture.

**Step 2: Implement any remaining edge-case handling**

Examples:

- invalid tab names
- choosing hidden or absent actions
- continue when not awaiting continue

**Step 3: Run the focused tests**

Run:

```bash
cargo test -p undone-ui dev_ipc
```

Expected: PASS

**Step 4: Commit**

```bash
git add crates/undone-ui/src/dev_ipc.rs tools/game-input-mcp/src/server.rs
git commit -m "test: acceptance coverage for runtime dev contract"
```

---

### Task 11: Run full workspace verification

**Step 1: Format**

Run:

```bash
cargo fmt --all
```

Expected: PASS

**Step 2: Run workspace tests**

Run:

```bash
cargo test --workspace
```

Expected: ALL PASS

**Step 3: Run workspace clippy**

Run:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: PASS

**Step 4: Build tools workspace**

Run:

```bash
cd tools && cargo build --release
```

Expected: PASS

**Step 5: Commit**

```bash
git add -A
git commit -m "chore: finalize player-correctness runtime verification pass"
```

---

### Task 12: Acceptance Tests for live runtime and agent tooling

**Acceptance Criteria:**
- Running game in `--dev --quick` can be inspected through MCP without screenshot parsing.
- Agent can jump to a scene, query runtime state, choose an action, and continue.
- Structured runtime state matches the actual flow reached by the running game.

**Files:**
- No new committed test files required if these remain runtime smoke checks.
- Update docs if the live workflow reveals contract changes.

**Step 1: Launch the game**

Run:

```bash
cargo run --release --bin undone -- --dev --quick
```

Expected: Game launches into the in-game runtime with dev tooling enabled.

**Step 2: Exercise MCP/runtime flow**

Use the MCP tool wrappers to:

1. call `get_runtime_state`
2. call `jump_to_scene` for a known scene
3. call `get_runtime_state` again and confirm scene/action visibility
4. call `choose_action` with a visible action id
5. call `get_runtime_state` and confirm updated prose/state
6. call `continue_scene` when appropriate

Expected: All operations succeed and return coherent structured runtime state.

**Step 3: Verify dev panel alignment**

Use the running app and, if needed, the dev tab to confirm the inspector reflects the same runtime truth exposed by IPC/MCP.

**Step 4: Record any contract adjustments**

If live verification reveals a contract mismatch, fix it now and rerun the relevant tests before proceeding.

**Step 5: Commit**

```bash
git add -A
git commit -m "test: validate live runtime and MCP correctness workflow"
```

---

### Task 13: Update docs and handoff

**Files:**
- Modify: `docs/engine-contract.md`
- Modify: `docs/plans/2026-02-21-engine-design.md`
- Modify: `HANDOFF.md`

**Step 1: Update engine contract**

Document:

- runtime controller ownership of scene/action/continue/resume semantics
- runtime snapshot contract
- dev IPC/MCP runtime-state capabilities
- intro-time NPC binding invariant

**Step 2: Update living engine design**

Reflect the new runtime-control and verification architecture at a high level.

**Step 3: Update handoff**

Record:

- design/implementation completed
- new acceptance/runtime tooling available
- next action if any remains

**Step 4: Run a final verification pass on docs-only changes if needed**

Run:

```bash
cargo test --workspace
```

Expected: PASS

**Step 5: Commit**

```bash
git add docs/engine-contract.md docs/plans/2026-02-21-engine-design.md HANDOFF.md
git commit -m "docs: record player-correctness runtime contract and handoff"
```

---

## Execution Notes

- Keep the controller thin. It should orchestrate runtime behavior, not absorb engine logic.
- Do not maintain parallel flow semantics in UI handlers and dev IPC. Refactor callers to delegate.
- Prefer acceptance assertions on structured state over brittle full-prose equality.
- Treat intro-time NPC binding as a correctness bug, not a follow-up nicety.
- If implementation reveals a better file split than `runtime_controller.rs` plus `runtime_snapshot.rs`, keep the contract stable and adjust the plan pragmatically.

Use `ops-executing-plans` to implement the plan at `docs/plans/2026-03-10-player-correctness-runtime-plan.md`
