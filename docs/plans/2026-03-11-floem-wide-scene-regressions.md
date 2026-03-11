# Floem Wide-Window And Scene-Transition Regression Plan

> **For Claude:** REQUIRED SUB-SKILL: Use ops-executing-plans to implement this plan task-by-task.

**Goal:** Verify and fix remaining Floem layout regressions when the main game window becomes wide and when scene transitions occur, then record verified Floem 0.2 guidance for this repo.

**Architecture:** Use the existing responsive window metrics plus live dev IPC/runtime snapshots to isolate whether the regression is caused by flex sizing, action-bar width constraints, or phase/scene re-entry rebuilding unsized wrappers. Lock any discovered bug with focused `undone-ui` tests before changing production code. Finish by updating the repo Floem skill with behavior verified against local Floem 0.2.0 source and the live app.

**Tech Stack:** Rust workspace, Floem 0.2.0, dev IPC runtime snapshot contract, window screenshot tooling

---

### Task 1: Reproduce the wide-window and scene-transition regressions live

**Files:**
- Read: `crates/undone-ui/src/lib.rs`
- Read: `crates/undone-ui/src/left_panel.rs`
- Read: `crates/undone-ui/src/right_panel.rs`
- Read: `.claude/skills/floem-layout/SKILL.md`

**Step 1: Launch the app in dev quick-start mode**

Run:

```bash
cargo run --release --bin undone -- --dev --quick
```

Expected: The app opens directly into the in-game UI with dev IPC polling enabled.

**Step 2: Capture baseline state and wide-window screenshots**

Use the available tooling to:

- capture the initial window
- resize to a clearly wide state
- capture another screenshot
- read `get_runtime_state` before and after

Expected: Clear evidence of whether the action buttons stay in a narrow centered strip or expand/wrap appropriately.

**Step 3: Advance gameplay and capture post-transition state**

Use dev IPC commands and screenshots to:

- choose visible actions until the runtime changes scene or reaches continue
- continue when appropriate
- capture runtime snapshots and screenshots after the scene change

Expected: Clear evidence of whether resized-window behavior persists across scene changes.

---

### Task 2: Add failing regression tests for the verified root cause

**Files:**
- Modify: `crates/undone-ui/src/left_panel.rs`
- Modify: `crates/undone-ui/src/lib.rs`
- Test: `crates/undone-ui/src/left_panel.rs`

**Step 1: Write the smallest failing test that matches the observed regression**

Examples:

- action-bar column/width math is wrong for wide windows
- scene reset loses window-sensitive layout state
- a parent wrapper falls back to unsized flex behavior after scene rebuild

**Step 2: Run the focused tests to verify RED**

Run:

```bash
cargo test -p undone-ui left_panel
```

Expected: The new regression test fails for the intended reason.

---

### Task 3: Implement the minimal Floem fix

**Files:**
- Modify: `crates/undone-ui/src/left_panel.rs`
- Modify: `crates/undone-ui/src/lib.rs`
- Modify: `crates/undone-ui/src/layout.rs`
- Modify: `crates/undone-ui/src/right_panel.rs`

**Step 1: Patch only the proven root cause**

Keep the fix constrained to the verified issue:

- width allocation in the story/action region
- full-size wrappers for rebuilt scene containers
- reactive height/width constraints that must survive scene changes

**Step 2: Run the focused test suite**

Run:

```bash
cargo test -p undone-ui
```

Expected: PASS

---

### Task 4: Acceptance Tests for live resize and scene progression behavior

**Acceptance Criteria:**
- User can widen the game window and the visible action area uses the added width instead of staying artificially narrow.
- User can progress to the next scene and the story/action area keeps the resized-window behavior.
- User can inspect the runtime through dev IPC while the live screenshots match the reported visible actions.

**Files:**
- Modify: `.claude/skills/floem-layout/SKILL.md`

**Step 1: Re-run the live workflow**

Repeat the live resize and scene progression checks after the fix.

**Step 2: Update the Floem skill with verified guidance**

Record only guidance confirmed from:

- local Floem 0.2.0 source
- docs.rs Floem 0.2.0 references if needed
- this repo’s live behavior

**Step 3: Final verification**

Run:

```bash
cargo test -p undone-ui
```

Expected: PASS
