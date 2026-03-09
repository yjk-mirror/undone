# Dev Tooling Cleanup — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use ops:executing-plans to implement this plan task-by-task.

**Goal:** Fix polish issues in Codex's dev tooling implementation — remove unnecessary reactivity in title bar, add atomic IPC writes, add missing commands (AdvanceTime, SetNpcLiking), add quick action buttons to dev panel, fix polling interval, format code.

**Architecture:** All work is in the existing worktree at `.worktrees/dev-tooling-plan/` on branch `codex/dev-tooling-plan`. Fixes are small, isolated edits across 4 files. No new files. No new dependencies.

**Tech Stack:** Rust, floem 0.2.0, serde_json

**Worktree:** `.worktrees/dev-tooling-plan/` (branch `codex/dev-tooling-plan`) — all edits go here.

---

## Task 1: Remove dyn_view from title bar

The title bar wraps tab buttons in `dyn_view` to conditionally include the Dev tab.
`dev_mode` is a static `bool` — no reactivity needed. Replace with a simple `if/else`
at view construction time.

**Files:**
- Modify: `.worktrees/dev-tooling-plan/crates/undone-ui/src/title_bar.rs:30-48`

**Step 1: Replace the dyn_view block**

Replace lines 30-48:

```rust
    // Center zone: tab buttons
    let tabs = dyn_view(move || {
        if dev_mode {
            h_stack((
                tab_button("Game", AppTab::Game, tab, signals, dev_mode),
                tab_button("Saves", AppTab::Saves, tab, signals, dev_mode),
                tab_button("Settings", AppTab::Settings, tab, signals, dev_mode),
                tab_button("Dev", AppTab::Dev, tab, signals, dev_mode),
            ))
            .into_any()
        } else {
            h_stack((
                tab_button("Game", AppTab::Game, tab, signals, dev_mode),
                tab_button("Saves", AppTab::Saves, tab, signals, dev_mode),
                tab_button("Settings", AppTab::Settings, tab, signals, dev_mode),
            ))
            .into_any()
        }
    });
```

With:

```rust
    // Center zone: tab buttons (dev_mode is static — no reactivity needed)
    let tabs = if dev_mode {
        h_stack((
            tab_button("Game", AppTab::Game, tab, signals, dev_mode),
            tab_button("Saves", AppTab::Saves, tab, signals, dev_mode),
            tab_button("Settings", AppTab::Settings, tab, signals, dev_mode),
            tab_button("Dev", AppTab::Dev, tab, signals, dev_mode),
        ))
        .into_any()
    } else {
        h_stack((
            tab_button("Game", AppTab::Game, tab, signals, dev_mode),
            tab_button("Saves", AppTab::Saves, tab, signals, dev_mode),
            tab_button("Settings", AppTab::Settings, tab, signals, dev_mode),
        ))
        .into_any()
    };
```

Also remove the `dyn_view` import if it was added — check imports at top. (It comes from
`floem::prelude::*` so there's nothing to remove.)

**Step 2: Build**

```bash
cd .worktrees/dev-tooling-plan && cargo check -p undone-ui
```

Expected: compiles clean.

**Step 3: Commit**

```bash
cd .worktrees/dev-tooling-plan && cargo fmt && git add crates/undone-ui/src/title_bar.rs && git commit -m "fix: remove unnecessary dyn_view from title bar tab buttons"
```

---

## Task 2: Fix IPC polling interval and add atomic writes

Three issues:
1. Game-side polling is 50ms — design says 100ms.
2. MCP-side command write is not atomic (no tmp+rename).
3. Game-side result write is not atomic.

**Files:**
- Modify: `.worktrees/dev-tooling-plan/crates/undone-ui/src/dev_ipc.rs:120` (polling interval)
- Modify: `.worktrees/dev-tooling-plan/crates/undone-ui/src/dev_ipc.rs:162-168` (atomic result write)
- Modify: `.worktrees/dev-tooling-plan/tools/game-input-mcp/src/server.rs:273-275` (atomic command write)

**Step 1: Change polling interval from 50ms to 100ms**

In `dev_ipc.rs` line 120, change:

```rust
    exec_after(Duration::from_millis(50), move |_| {
```

To:

```rust
    exec_after(Duration::from_millis(100), move |_| {
```

**Step 2: Make game-side result write atomic**

In `dev_ipc.rs`, replace the result write block at lines 162-168:

```rust
    let payload = serde_json::to_string(&response).unwrap_or_else(|err| {
        format!(
            r#"{{"success":false,"message":"Failed to serialize dev response: {}"}}"#,
            err
        )
    });
    let _ = std::fs::write(result_path, payload);
```

With:

```rust
    let payload = serde_json::to_string(&response).unwrap_or_else(|err| {
        format!(
            r#"{{"success":false,"message":"Failed to serialize dev response: {}"}}"#,
            err
        )
    });
    let tmp_path = result_path.with_extension("tmp");
    if std::fs::write(&tmp_path, &payload).is_ok() {
        let _ = std::fs::rename(&tmp_path, &result_path);
    }
```

**Step 3: Make MCP-side command write atomic**

In `tools/game-input-mcp/src/server.rs`, find the `dev_command` tool (around line 273):

```rust
        let _ = std::fs::remove_file(&result_path);
        std::fs::write(&command_path, params.0.command_json)
            .map_err(|e| McpError::internal_error(format!("write command failed: {e}"), None))?;
```

Replace with:

```rust
        let _ = std::fs::remove_file(&result_path);
        let tmp_path = command_path.with_extension("tmp");
        std::fs::write(&tmp_path, &params.0.command_json)
            .map_err(|e| McpError::internal_error(format!("write command failed: {e}"), None))?;
        std::fs::rename(&tmp_path, &command_path)
            .map_err(|e| McpError::internal_error(format!("rename command failed: {e}"), None))?;
```

**Step 4: Build both workspaces**

```bash
cd .worktrees/dev-tooling-plan && cargo check -p undone-ui
cd .worktrees/dev-tooling-plan/tools && cargo check -p game-input-mcp
```

Expected: both compile clean.

**Step 5: Commit**

```bash
cd .worktrees/dev-tooling-plan && cargo fmt && cd tools && cargo fmt && cd .. && git add crates/undone-ui/src/dev_ipc.rs tools/game-input-mcp/src/server.rs && git commit -m "fix: IPC polling interval to 100ms, atomic writes for command and result files"
```

---

## Task 3: Add AdvanceTime and SetNpcLiking commands to IPC

The design doc specified 8 commands. Codex implemented 5. Add the missing two that
are useful for dev workflows: `AdvanceTime` and `SetNpcLiking`.

**Files:**
- Modify: `.worktrees/dev-tooling-plan/crates/undone-ui/src/dev_ipc.rs` (DevCommand enum + execute_command)

**Step 1: Add enum variants**

In `dev_ipc.rs`, add to the `DevCommand` enum (after `RemoveFlag`):

```rust
    AdvanceTime { weeks: u32 },
    SetNpcLiking { npc_name: String, level: String },
```

**Step 2: Add execution handlers**

In `execute_command()`, add match arms inside the `match command { ... }` block:

```rust
        DevCommand::AdvanceTime { weeks } => advance_time(gs, weeks),
        DevCommand::SetNpcLiking { npc_name, level } => set_npc_liking(gs, &npc_name, &level),
```

**Step 3: Implement the handler functions**

Add these functions after `remove_flag()`:

```rust
fn advance_time(gs: &mut GameState, weeks: u32) -> DevCommandResponse {
    let slots = weeks * 28; // 4 slots/day × 7 days/week
    for _ in 0..slots {
        gs.world.game_data.advance_time_slot();
    }
    DevCommandResponse {
        success: true,
        message: format!("Advanced {weeks} week(s)"),
        data: None,
    }
}

fn set_npc_liking(gs: &mut GameState, npc_name: &str, level: &str) -> DevCommandResponse {
    use undone_domain::LikingLevel;

    let liking = match level {
        "Neutral" => LikingLevel::Neutral,
        "Ok" => LikingLevel::Ok,
        "Like" => LikingLevel::Like,
        "Close" => LikingLevel::Close,
        other => {
            return DevCommandResponse {
                success: false,
                message: format!(
                    "Unknown liking level '{other}'. Supported: Neutral, Ok, Like, Close"
                ),
                data: None,
            };
        }
    };

    let name_lower = npc_name.trim().to_lowercase();
    let mut found = false;

    for (_, npc) in gs.world.male_npcs.iter_mut() {
        if npc.core.name.to_lowercase() == name_lower {
            npc.core.npc_liking = liking;
            found = true;
            break;
        }
    }
    if !found {
        for (_, npc) in gs.world.female_npcs.iter_mut() {
            if npc.core.name.to_lowercase() == name_lower {
                npc.core.npc_liking = liking;
                found = true;
                break;
            }
        }
    }

    if found {
        DevCommandResponse {
            success: true,
            message: format!("Set {npc_name} liking to {level}"),
            data: None,
        }
    } else {
        DevCommandResponse {
            success: false,
            message: format!("NPC '{npc_name}' not found"),
            data: None,
        }
    }
}
```

**Step 4: Add unit tests**

Add to the `#[cfg(test)] mod tests` block:

```rust
    #[test]
    fn execute_advance_time_increments_week() {
        let mut gs = test_game_state();
        let signals = AppSignals::new();
        let week_before = gs.world.game_data.week;

        let response = execute_command(&mut gs, signals, DevCommand::AdvanceTime { weeks: 2 });

        assert!(response.success);
        assert_eq!(gs.world.game_data.week, week_before + 2);
    }

    #[test]
    fn execute_set_npc_liking_unknown_level_returns_error() {
        let mut gs = test_game_state();
        let signals = AppSignals::new();

        let response = execute_command(
            &mut gs,
            signals,
            DevCommand::SetNpcLiking {
                npc_name: "Jake".to_string(),
                level: "BestFriend".to_string(),
            },
        );

        assert!(!response.success);
        assert!(response.message.contains("Unknown liking level"));
    }
```

**Step 5: Build and test**

```bash
cd .worktrees/dev-tooling-plan && cargo test -p undone-ui dev_ipc
```

Expected: ALL PASS (existing 3 + new 2 = 5 tests).

**Step 6: Commit**

```bash
cd .worktrees/dev-tooling-plan && cargo fmt && git add crates/undone-ui/src/dev_ipc.rs && git commit -m "feat: add AdvanceTime and SetNpcLiking dev IPC commands"
```

---

## Task 4: Add MCP convenience tools for new commands

Add `advance_time` and `set_npc_liking` tool wrappers to the game-input MCP server,
matching the pattern of the existing convenience tools.

**Files:**
- Modify: `.worktrees/dev-tooling-plan/tools/game-input-mcp/src/server.rs`

**Step 1: Add input types**

After `SetGameFlagInput`, add:

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AdvanceTimeInput {
    /// Number of weeks to advance.
    pub weeks: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetNpcLikingInput {
    /// NPC name (case-insensitive), e.g. "Jake".
    pub npc_name: String,
    /// Liking level: Neutral, Ok, Like, or Close.
    pub level: String,
}
```

**Step 2: Add tool methods**

After the `remove_game_flag` tool, add:

```rust
    #[tool(description = "Advance the game clock by N weeks in a running Undone game in dev mode.")]
    async fn advance_time(
        &self,
        params: Parameters<AdvanceTimeInput>,
    ) -> Result<CallToolResult, McpError> {
        self.dev_command(Parameters(DevCommandInput {
            command_json: json!({
                "command": "advance_time",
                "weeks": params.0.weeks,
            })
            .to_string(),
            timeout_ms: Some(2000),
        }))
        .await
    }

    #[tool(description = "Set an NPC's liking level in a running Undone game in dev mode.")]
    async fn set_npc_liking(
        &self,
        params: Parameters<SetNpcLikingInput>,
    ) -> Result<CallToolResult, McpError> {
        self.dev_command(Parameters(DevCommandInput {
            command_json: json!({
                "command": "set_npc_liking",
                "npc_name": params.0.npc_name,
                "level": params.0.level,
            })
            .to_string(),
            timeout_ms: Some(2000),
        }))
        .await
    }
```

**Step 3: Update server description**

In the `ServerHandler` impl, update the description string to mention the new tools.
Find `advance_time(weeks)` and `set_npc_liking(npc_name, level)` are not listed. Add
them to the tool listing in the description alongside the existing dev tools.

**Step 4: Build**

```bash
cd .worktrees/dev-tooling-plan/tools && cargo check -p game-input-mcp
```

Expected: compiles clean.

**Step 5: Commit**

```bash
cd .worktrees/dev-tooling-plan/tools && cargo fmt && cd .. && git add tools/game-input-mcp/src/server.rs && git commit -m "feat(mcp): add advance_time and set_npc_liking dev tools"
```

---

## Task 5: Add quick action buttons to dev panel

The design doc specified quick actions: "Advance 1 Week" and "Set All NPC Liking → Close".
These are missing from the dev panel.

**Files:**
- Modify: `.worktrees/dev-tooling-plan/crates/undone-ui/src/dev_panel.rs`

**Step 1: Add a quick actions section**

In `dev_panel()`, after `inspector_section` (before the final `scroll(v_stack((...)))`)
add a quick actions section:

```rust
    let quick_section = section_card(
        "Quick Actions",
        h_stack((
            action_button("Advance 1 Week", signals, {
                let gs = Rc::clone(&gs);
                move || {
                    run_command(
                        &gs,
                        signals,
                        status,
                        money_input,
                        stress_input,
                        anxiety_input,
                        femininity_input,
                        DevCommand::AdvanceTime { weeks: 1 },
                    );
                }
            }),
            action_button("All NPC → Close", signals, {
                let gs = Rc::clone(&gs);
                move || {
                    use undone_domain::LikingLevel;
                    {
                        let mut gs_ref = gs.borrow_mut();
                        for (_, npc) in gs_ref.world.male_npcs.iter_mut() {
                            npc.core.npc_liking = LikingLevel::Close;
                        }
                        for (_, npc) in gs_ref.world.female_npcs.iter_mut() {
                            npc.core.npc_liking = LikingLevel::Close;
                        }
                    }
                    status.set("Set all NPC liking to Close".to_string());
                    signals.dev_tick.update(|tick| *tick += 1);
                }
            }),
        ))
        .style(|s| s.gap(8.0).flex_wrap(floem::style::FlexWrap::Wrap)),
        signals,
    );
```

**Step 2: Add quick_section to the layout**

In the `scroll(v_stack((...)))` call at the bottom of `dev_panel()`, add `quick_section`
after `flag_section` and before `inspector_section`:

```rust
    scroll(v_stack((
        heading("Dev Tools", signals),
        label(move || status.get()).style(move |s| { ... }),
        scene_section,
        stats_section,
        flag_section,
        quick_section,
        inspector_section,
    )))
```

**Step 3: Build**

```bash
cd .worktrees/dev-tooling-plan && cargo check -p undone-ui
```

Expected: compiles clean.

**Step 4: Commit**

```bash
cd .worktrees/dev-tooling-plan && cargo fmt && git add crates/undone-ui/src/dev_panel.rs && git commit -m "feat: add quick action buttons to dev panel (advance week, NPC liking)"
```

---

## Task 6: Format, full test suite, validate-pack

**Step 1: Format both workspaces**

```bash
cd .worktrees/dev-tooling-plan && cargo fmt
cd .worktrees/dev-tooling-plan/tools && cargo fmt
```

**Step 2: Run full test suite**

```bash
cd .worktrees/dev-tooling-plan && cargo test --workspace
```

Expected: ALL PASS (284 + new tests).

**Step 3: Run validate-pack**

```bash
cd .worktrees/dev-tooling-plan && cargo run --bin validate-pack
```

Expected: all checks pass. Reachability warnings are expected (they're real findings).

**Step 4: Commit any formatting changes**

```bash
cd .worktrees/dev-tooling-plan && git add -A && git diff --cached --stat
```

If there are formatting changes, commit:

```bash
git commit -m "style: cargo fmt"
```

If no changes, skip.

---

## Summary

| Task | What | Files |
|------|------|-------|
| 1 | Remove dyn_view from title bar | title_bar.rs |
| 2 | Fix IPC: 100ms polling, atomic writes | dev_ipc.rs, server.rs |
| 3 | Add AdvanceTime + SetNpcLiking commands | dev_ipc.rs |
| 4 | Add MCP convenience tools for new commands | server.rs |
| 5 | Add quick action buttons to dev panel | dev_panel.rs |
| 6 | Format + full tests + validate | all |

After Task 6, the branch is ready for merge via `ops:finishing-a-development-branch`.
