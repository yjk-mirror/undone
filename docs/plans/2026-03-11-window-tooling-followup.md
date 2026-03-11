# Window Tooling And Responsive Audit Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use ops-executing-plans to implement this plan task-by-task.

**Goal:** Turn window resizing into a first-class dev-tooling primitive, add stronger acceptance coverage for resize-plus-scene-progression flows, improve the Dev tab resize UX, and audit the remaining Floem UI for fixed-size assumptions.

**Architecture:** Reuse the existing in-app `DevCommand::SetWindowSize` path instead of creating another resize mechanism. Expose that path through the existing MCP/dev-command server, add runtime snapshot fields so acceptance checks can assert window metrics directly, then layer Dev tab improvements and a focused responsive audit on top. Keep the work test-first where possible, and treat live screenshots plus runtime snapshots as the acceptance gate for all UI changes.

**Tech Stack:** Rust workspace, Floem 0.2.0, `tools/game-input-mcp`, dev IPC JSON commands, runtime snapshots, screenshot tooling

---

### Task 1: MCP resize tool over the existing dev-command path

**Files:**
- Modify: `tools/game-input-mcp/src/server.rs`
- Test: `tools/game-input-mcp/src/server.rs`

**Step 1: Write the failing payload/unit tests**

Add tests for a new resize payload helper near the existing payload tests:

```rust
#[test]
fn set_window_size_payload_uses_resize_command_name() {
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&set_window_size_payload(1800.0, 1000.0)).unwrap(),
        json!({
            "command": "set_window_size",
            "width": 1800.0,
            "height": 1000.0
        })
    );
}
```

**Step 2: Run the focused test to verify RED**

Run:

```bash
cargo test -p game-input-mcp set_window_size_payload_uses_resize_command_name
```

Expected: FAIL because the payload helper does not exist yet.

**Step 3: Add the MCP resize input type and tool**

In `tools/game-input-mcp/src/server.rs`, add:

- `SetWindowSizeInput { width: f64, height: f64 }`
- `set_window_size_payload(width, height) -> String`
- `set_window_size(...)` tool method that sends:

```json
{"command":"set_window_size","width":1800.0,"height":1000.0}
```

Use the existing `dev_command(...)` helper instead of writing another IPC path.

**Step 4: Run the focused MCP tests**

Run:

```bash
cargo test -p game-input-mcp
```

Expected: PASS

**Step 5: Commit**

```bash
git add tools/game-input-mcp/src/server.rs
git commit -m "feat(mcp): add set_window_size dev tool for Undone"
```

---

### Task 2: Expose live window metrics in runtime snapshots

**Files:**
- Modify: `crates/undone-ui/src/runtime_snapshot.rs`
- Modify: `crates/undone-ui/src/dev_ipc.rs`
- Test: `crates/undone-ui/src/runtime_snapshot.rs`
- Test: `crates/undone-ui/src/dev_ipc.rs`

**Step 1: Write the failing snapshot tests**

Extend the runtime snapshot tests to require visible window metrics:

```rust
assert_eq!(snapshot.window_width, 1800.0);
assert_eq!(snapshot.window_height, 1000.0);
```

Extend the dev IPC runtime-state JSON test to require:

```rust
assert!(data.get("window_width").is_some());
assert!(data.get("window_height").is_some());
```

**Step 2: Run the focused tests to verify RED**

Run:

```bash
cargo test -p undone-ui runtime_snapshot
cargo test -p undone-ui execute_get_runtime_state_returns_visible_story_and_actions
```

Expected: FAIL because `RuntimeSnapshot` does not expose the window metrics yet.

**Step 3: Add the metrics to `RuntimeSnapshot`**

In `crates/undone-ui/src/runtime_snapshot.rs`, add:

```rust
pub window_width: f64,
pub window_height: f64,
```

Populate them from `signals.window_width.get_untracked()` and `signals.window_height.get_untracked()`.

Keep `dev_ipc::GetRuntimeState` unchanged except for the updated serialized payload.

**Step 4: Run the focused tests**

Run:

```bash
cargo test -p undone-ui runtime_snapshot
cargo test -p undone-ui dev_ipc
```

Expected: PASS

**Step 5: Commit**

```bash
git add crates/undone-ui/src/runtime_snapshot.rs crates/undone-ui/src/dev_ipc.rs
git commit -m "feat(ui): expose window metrics in runtime snapshots"
```

---

### Task 3: Dev tab resize UX cleanup

**Files:**
- Modify: `crates/undone-ui/src/dev_panel.rs`
- Modify: `crates/undone-ui/src/layout.rs`
- Test: `crates/undone-ui/src/dev_panel.rs`

**Step 1: Write the failing unit tests for the new resize controls**

Add pure helper tests for:

- parsing width/height input
- rejecting empty or non-positive values
- restoring the shared default size

Example helper behavior:

```rust
assert_eq!(parse_window_size_inputs("1800", "1000"), Some((1800.0, 1000.0)));
assert_eq!(parse_window_size_inputs("", "1000"), None);
assert_eq!(parse_window_size_inputs("0", "1000"), None);
```

**Step 2: Run the focused tests to verify RED**

Run:

```bash
cargo test -p undone-ui dev_panel
```

Expected: FAIL because the parsing/default helpers do not exist yet.

**Step 3: Add custom resize inputs and a default-size action**

In `crates/undone-ui/src/dev_panel.rs`:

- add width and height `RwSignal<String>` inputs
- add an `Apply Size` button that calls `DevCommand::SetWindowSize`
- add a `Default` button that uses:

```rust
crate::layout::DEFAULT_WINDOW_WIDTH
crate::layout::DEFAULT_WINDOW_HEIGHT
```

Prefer pure parsing helpers so the input behavior stays unit-testable.

**Step 4: Run the UI tests**

Run:

```bash
cargo test -p undone-ui dev_panel
```

Expected: PASS

**Step 5: Commit**

```bash
git add crates/undone-ui/src/dev_panel.rs crates/undone-ui/src/layout.rs
git commit -m "feat(ui): add custom window size controls to dev panel"
```

---

### Task 4: Acceptance tests for resize plus scene progression

**Acceptance Criteria:**
- User can resize the running game window through MCP/dev tooling without dragging.
- User can verify the new width and height through `get_runtime_state`.
- User can advance through a slot-driven scene transition after resizing and the resized behavior persists.
- User can capture a screenshot after resize and after scene change, and both screenshots match the runtime snapshot contract.

**Files:**
- Modify: `tools/game-input-mcp/src/server.rs`
- Modify: `.claude/skills/floem-layout/SKILL.md`
- Optionally create: `docs/plans/acceptance-notes/2026-03-11-window-tooling.md` if notes need to be retained

**Step 1: Build the MCP server and app**

Run:

```bash
cargo build
cd tools && cargo build -p game-input-mcp && cd ..
```

Expected: PASS

**Step 2: Launch the game in dev quick-start mode**

Run:

```bash
target\debug\undone.exe --dev --quick
```

Expected: running window titled `Undone`

**Step 3: Run the acceptance workflow**

Use tooling in this exact order:

1. `set_tab("dev")`
2. `set_window_size(1800, 1000)`
3. `get_runtime_state()`
4. capture screenshot
5. `jump_to_scene("base::plan_your_day")`
6. `advance_time(1)`
7. `choose_action("go_out")`
8. `get_runtime_state()`
9. capture screenshot

Verify:

- the first runtime snapshot reports `window_width = 1800`, `window_height = 1000`
- the second runtime snapshot shows a different `current_scene_id`
- the second snapshot does not show stale `plan_your_day` actions
- screenshots show the wide layout preserved after the scene change

**Step 4: Update the Floem skill with the accepted verification path**

Add verified repo guidance to `.claude/skills/floem-layout/SKILL.md` covering:

- using MCP/tooling resize instead of drag-based repro
- checking runtime window metrics alongside screenshots
- verifying slot-driven scene changes after resize

**Step 5: Commit**

```bash
git add tools/game-input-mcp/src/server.rs .claude/skills/floem-layout/SKILL.md
git commit -m "test: add resize-driven acceptance coverage guidance"
```

---

### Task 5: Responsive audit of the remaining Floem surfaces

**Files:**
- Read/Modify as needed: `crates/undone-ui/src/saves_panel.rs`
- Read/Modify as needed: `crates/undone-ui/src/settings_panel.rs`
- Read/Modify as needed: `crates/undone-ui/src/title_bar.rs`
- Read/Modify as needed: `crates/undone-ui/src/landing_page.rs`
- Read/Modify as needed: `crates/undone-ui/src/char_creation.rs`
- Modify: `.claude/skills/floem-layout/SKILL.md`
- Test: whichever file receives extracted helper logic

**Step 1: Audit for hardcoded widths and unsized wrappers**

Check for:

- hardcoded widths that should derive from window metrics
- `dyn_container(...)` wrappers missing `.style(|s| s.size_full())`
- scroll containers missing `.scroll_style(|s| s.shrink_to_fit())`
- button rows that cannot wrap at narrower widths

Do not change behavior until the specific issue is reproduced.

**Step 2: If a regression is found, add the smallest failing test first**

Examples:

- extract a pure width helper and test it
- add a regression test around responsive row counts
- add a focused unit test for a panel sizing helper

Run the smallest relevant test target first:

```bash
cargo test -p undone-ui saves_panel
```

or:

```bash
cargo test -p undone-ui settings_panel
```

Expected: RED before the fix.

**Step 3: Apply the minimal fix**

Keep changes constrained to the reproduced issue. Reuse `crates/undone-ui/src/layout.rs` if a panel needs shared responsive constants or helper math.

**Step 4: Run the full UI suite**

Run:

```bash
cargo test -p undone-ui
```

Expected: PASS

**Step 5: Record verified findings**

Update `.claude/skills/floem-layout/SKILL.md` with only the confirmed guidance from this audit.

**Step 6: Commit**

```bash
git add crates/undone-ui/src/*.rs .claude/skills/floem-layout/SKILL.md
git commit -m "fix(ui): audit remaining Floem panels for responsive layout assumptions"
```

---

### Task 6: Final acceptance and cleanup

**Acceptance Criteria:**
- There is a first-class MCP resize operation for the running game.
- Runtime snapshots expose current window metrics.
- The Dev tab supports presets, custom sizes, and restoring defaults.
- The resize-plus-scene-progression workflow is verified on a live app.
- The responsive audit leaves the repo in a clean, documented state.

**Files:**
- Modify: `HANDOFF.md`

**Step 1: Run the final verification commands**

Run:

```bash
cargo build
cargo test -p undone-ui
cd tools && cargo test -p game-input-mcp && cd ..
```

Expected: PASS

**Step 2: Run one final live workflow**

Use the same acceptance flow from Task 4 on a fresh app launch and verify:

- resize works through MCP
- runtime snapshot reports the requested size
- post-transition layout still matches expectations

**Step 3: Update handoff**

Record:

- what tooling now exists
- which acceptance flow was verified
- any remaining gaps

**Step 4: Commit**

```bash
git add HANDOFF.md
git commit -m "docs: record window tooling and responsive audit results"
```

