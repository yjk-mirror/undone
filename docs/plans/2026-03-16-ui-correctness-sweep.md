# UI Correctness Sweep Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use ops-executing-plans to implement this plan task-by-task.

**Goal:** Fix the first tier of player-visible UI correctness bugs in Undone and add durable TDD coverage so visible affordances, layout, and runtime state do not drift again.

**Architecture:** Work in layered user-facing TDD slices. For each bug class, first lock the failure at the smallest stable seam with a focused test, then implement the minimal fix, then prove the player-facing behavior through the runtime acceptance contract and a targeted live audit. Keep the first sweep constrained to current desktop UI surfaces and proven visible bugs.

**Tech Stack:** Rust workspace, Floem 0.2 UI, `undone-ui` runtime acceptance harness, runtime snapshot contract, dev-mode live app checks

---

### Task 1: Lock Bottom-Bar Hitbox Ownership

**Files:**
- Modify: `crates/undone-ui/src/left_panel.rs`
- Test: `crates/undone-ui/src/left_panel.rs`

**Step 1: Write the failing test**

Add a pure helper in `left_panel.rs` that makes hit-target ownership explicit for the action area, then write tests that express the player-facing contract:

- visible `Continue` chrome is clickable
- outer bottom-bar dead space is not clickable
- action-button containers do not implicitly turn the whole bar into one button

Start with assertions that fail against the current behavior.

**Step 2: Run the focused test to verify RED**

Run:

```bash
cargo test -p undone-ui left_panel -- --nocapture
```

Expected: FAIL for the new hitbox-ownership test.

**Step 3: Implement the minimal fix**

Patch `continue_button()` in `left_panel.rs` so the click handler belongs to the visible button view, not the full-width wrapper container. If needed, extract a small helper to keep the ownership rule explicit and testable.

Do not refactor unrelated layout code in this step.

**Step 4: Re-run the focused test**

Run:

```bash
cargo test -p undone-ui left_panel -- --nocapture
```

Expected: PASS for the new hitbox test and existing `left_panel` tests.

**Step 5: Commit**

```bash
git add crates/undone-ui/src/left_panel.rs
git commit -m "fix(ui): constrain bottom bar clicks to visible controls"
```

### Task 2: Add Acceptance Coverage For Continue-Flow Interaction

**Acceptance Criteria:**
- User can reach a `Continue` state and the runtime snapshot reports `awaiting_continue = true`.
- User can activate only the visible continue control to advance progression.
- Enter and Space continue behavior still works while dead space remains non-interactive.

**Files:**
- Modify: `crates/undone-ui/src/runtime_test_support.rs`
- Test: `crates/undone-ui/src/runtime_test_support.rs`

**Step 1: Write the failing acceptance test**

Add an acceptance-style test that:

1. starts a real runtime
2. advances until `awaiting_continue` is true
3. asserts the runtime snapshot is coherent in that state
4. proves continue progression happens only through the explicit continue path, not via stale action state

Keep the assertions on structured runtime state, not screenshots.

**Step 2: Run the acceptance test to verify RED or current gap**

Run:

```bash
cargo test -p undone-ui acceptance_runtime_continue -- --nocapture
```

Expected: Either FAIL because the new acceptance assertion is not yet satisfied, or expose a missing runtime assertion that must be added before the fix is complete.

**Step 3: Implement the minimal runtime-side support if needed**

Only if the acceptance test reveals a runtime contract gap, add the smallest supporting assertion/helper in `runtime_test_support.rs` or the snapshot plumbing needed to express the behavior cleanly.

Do not broaden scope beyond the continue interaction contract.

**Step 4: Re-run the acceptance test**

Run:

```bash
cargo test -p undone-ui acceptance_runtime_continue -- --nocapture
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/undone-ui/src/runtime_test_support.rs crates/undone-ui/src/left_panel.rs
git commit -m "test(ui): add continue-flow acceptance coverage"
```

### Task 3: Expand Responsive Action-Bar Layout Coverage

**Files:**
- Modify: `crates/undone-ui/src/layout.rs`
- Modify: `crates/undone-ui/src/left_panel.rs`
- Test: `crates/undone-ui/src/left_panel.rs`

**Step 1: Write failing layout tests**

Add focused tests for player-visible layout rules that are currently undercovered:

- action-bar columns collapse predictably across widths
- row counts stay correct after width changes
- story region width never drops below a usable action width
- wide windows increase usable action width instead of trapping buttons in a narrower-than-necessary strip

Use pure calculations only.

**Step 2: Run the focused layout tests to verify RED**

Run:

```bash
cargo test -p undone-ui action_button -- --nocapture
```

Expected: FAIL for at least one new assertion that matches a verified layout gap or exposes missing guardrails.

**Step 3: Implement the minimal layout fix**

Patch only the proven calculation or width constraint in `layout.rs` / `left_panel.rs`.

Do not change styling that is not necessary for the tested behavior.

**Step 4: Re-run the focused layout tests**

Run:

```bash
cargo test -p undone-ui action_button -- --nocapture
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/undone-ui/src/layout.rs crates/undone-ui/src/left_panel.rs
git commit -m "test(ui): lock responsive action bar layout rules"
```

### Task 4: Add Runtime Acceptance For Resize-Sensitive Scene Progression

**Acceptance Criteria:**
- User can start a runtime, resize to a wider window, and keep a coherent runtime snapshot.
- User can progress to another scene and the resized runtime remains coherent.
- Window metrics and visible actions remain aligned with the runtime contract before and after progression.

**Files:**
- Modify: `crates/undone-ui/src/runtime_test_support.rs`
- Modify: `crates/undone-ui/src/runtime_snapshot.rs`
- Test: `crates/undone-ui/src/runtime_test_support.rs`

**Step 1: Write the failing acceptance test**

Add an acceptance-style test that simulates resize-sensitive runtime state by setting or asserting window metrics in `AppSignals`, then:

1. launches into gameplay
2. records a snapshot
3. advances through at least one action and continue boundary when present
4. records another snapshot
5. asserts the runtime contract stays coherent across the scene change

**Step 2: Run the acceptance test to verify RED**

Run:

```bash
cargo test -p undone-ui acceptance_runtime_resize -- --nocapture
```

Expected: FAIL if any required window-metric or progression assertion is missing.

**Step 3: Implement the minimal snapshot/runtime support**

If needed, add only the missing runtime snapshot fields or test helpers required to express the resize-sensitive contract clearly.

**Step 4: Re-run the acceptance test**

Run:

```bash
cargo test -p undone-ui acceptance_runtime_resize -- --nocapture
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/undone-ui/src/runtime_test_support.rs crates/undone-ui/src/runtime_snapshot.rs
git commit -m "test(ui): add resize-sensitive runtime acceptance coverage"
```

### Task 5: Lock State-Transition UI Reset Behavior

**Files:**
- Modify: `crates/undone-ui/src/lib.rs`
- Modify: `crates/undone-ui/src/left_panel.rs`
- Test: `crates/undone-ui/src/lib.rs`
- Test: `crates/undone-ui/src/runtime_test_support.rs`

**Step 1: Write failing reset-behavior tests**

Add tests for user-visible stale-state risks:

- highlighted action state clears on scene change
- hovered detail text does not survive a new action set
- continue state clears when a new scene starts
- tab or phase changes do not leave interaction state inconsistent with visible controls

Reuse existing reset helpers where possible; extract a helper only if the rule is not currently testable.

**Step 2: Run the focused tests to verify RED**

Run:

```bash
cargo test -p undone-ui reset_scene_ui_state -- --nocapture
```

Expected: FAIL for the new stale-state regression.

**Step 3: Implement the minimal state-reset fix**

Patch only the missing reset path or helper.

**Step 4: Re-run the focused tests**

Run:

```bash
cargo test -p undone-ui reset_scene_ui_state -- --nocapture
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/undone-ui/src/lib.rs crates/undone-ui/src/left_panel.rs crates/undone-ui/src/runtime_test_support.rs
git commit -m "fix(ui): clear stale interaction state across transitions"
```

### Task 6: Expand Text And Rendering Regression Coverage

**Files:**
- Modify: `crates/undone-ui/src/left_panel.rs`
- Test: `crates/undone-ui/src/left_panel.rs`

**Step 1: Write failing rendering tests**

Add focused tests around markdown rendering rules that are player-visible and deterministic:

- paragraph breaks stay preserved
- headings produce larger text than body text
- horizontal rules render as a separator line
- soft breaks and hard breaks stay readable

Keep the tests at the `markdown_to_text_layout()` seam.

**Step 2: Run the focused test to verify RED**

Run:

```bash
cargo test -p undone-ui markdown_to_text_layout -- --nocapture
```

Expected: FAIL for at least one new player-visible rendering assertion.

**Step 3: Implement the minimal rendering fix**

Patch only the rendering rule proven by the failing test.

**Step 4: Re-run the focused test**

Run:

```bash
cargo test -p undone-ui markdown_to_text_layout -- --nocapture
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/undone-ui/src/left_panel.rs
git commit -m "test(ui): lock markdown rendering behavior"
```

### Task 7: Live Acceptance Audit For Current Desktop UI Surfaces

**Acceptance Criteria:**
- User can click visible title-bar and bottom-bar controls without dead space triggering actions.
- User can resize the game window and keep usable action-bar layout before and after scene progression.
- User can switch tabs and return without stale bottom-strip state or obviously broken layout.

**Files:**
- Read: `crates/undone-ui/src/lib.rs`
- Read: `crates/undone-ui/src/left_panel.rs`
- Read: `crates/undone-ui/src/right_panel.rs`
- Read: `crates/undone-ui/src/title_bar.rs`

**Step 1: Launch the app in dev quick-start mode**

Run:

```bash
cargo run --release --bin undone -- --dev --quick
```

Expected: App launches directly into an in-game runtime.

**Step 2: Verify bottom-bar and title-bar interaction manually**

Check:

- clicking visible `Continue` or action buttons works
- clicking adjacent dead space does nothing
- title-bar tabs switch only when the visible tab control is clicked

Expected: No dead-space clicks trigger a user action.

**Step 3: Verify resize-plus-progression behavior**

In the running app:

- widen the window
- capture the visible action state
- progress an action and continue when needed
- verify the action area remains usable and aligned after the scene change

Expected: No obvious clipping, trapped narrow strip, or stale bottom-strip state.

**Step 4: Run the full `undone-ui` test suite**

Run:

```bash
cargo test -p undone-ui -- --nocapture
```

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/undone-ui/src/lib.rs crates/undone-ui/src/layout.rs crates/undone-ui/src/left_panel.rs crates/undone-ui/src/runtime_snapshot.rs crates/undone-ui/src/runtime_test_support.rs crates/undone-ui/src/title_bar.rs docs/plans/2026-03-16-ui-correctness-sweep-design.md docs/plans/2026-03-16-ui-correctness-sweep.md
git commit -m "fix(ui): add first-pass player-visible correctness coverage"
```
