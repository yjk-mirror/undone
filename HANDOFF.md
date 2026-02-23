# Undone — Handoff

## Current State

**Branch:** `engineering-overnight` (worktree off master)
**Tests:** 104 passing, 0 clippy warnings.
**App:** Character creation screen on launch (names, age, figure, traits, content prefs). Settings persist across restarts. Saves tab functional (save/load/delete). Names updated to NE US. Three theme modes. Prose centered with markdown rendering. Literata font embedded. Choice detail strip. Window resizable. Single-instance enforced. Scheduler wired. NPC activation events wired to sidebar.

---

## ⚡ Next Action

Merge `engineering-overnight` branch into master. Then pick from Open Items:
1. **Settings tab UI** — expose font size, line height, and future prefs as UI controls
2. **More scenes** — expand base pack content (apartment, work, social events)
3. **Visual polish** — test char creation UI, refine layout, screenshot audit

---

## game-input-mcp — Done

**Built:** `undone-tools/game-input-mcp` — MCP server for background game interaction.
Uses `PostMessage(WM_KEYDOWN/WM_KEYUP)` and `PostMessage(WM_LBUTTONDOWN/WM_LBUTTONUP)`
— no focus steal, no cursor movement.

**Tools:** `press_key(title, key)` and `click(title, x, y)`.
Keys: `"1"`–`"9"`, `"enter"`, `"tab"`, `"escape"`, `"space"`.

**Binary:** `undone-tools/target/release/game-input-mcp.exe` (2.9 MB).
**.mcp.json** updated with `game-input` server entry. Restart Claude Code to activate.

---

## screenshot-mcp — Working

**Bug fixed (2026-02-23):** `capture_window()` was calling `control.stop()` instead of
`control.wait()`. Fixed binary deployed to `undone-tools/target/release/screenshot-mcp.exe`.
**.mcp.json** points to `screenshot-mcp.exe`. Verified working.

---

## UI — Current State

**Layout:**
- Stats sidebar on the **left** (280px fixed): player name, stats, NPC panel, mode toggle
- Story + choices on the **right** (flex-grow): scrollable prose + choices bar
- Window opens at 1200×800, titled "Undone"

**Theme system:**
- Three modes: Warm Paper (default), Sepia, Night
- Mode toggle at bottom of stats sidebar
- All colors driven by `ThemeColors::from_mode()` reactively

**Keyboard navigation:**
- Number keys 1–9 select choices by position
- Tab/Enter activate focused button

**Key source files:**
- `crates/undone-ui/src/lib.rs` — AppSignals, AppTab, AppPhase, app_view
- `crates/undone-ui/src/char_creation.rs` — character creation form (pre-game phase)
- `crates/undone-ui/src/saves_panel.rs` — save/load/delete UI
- `crates/undone-ui/src/title_bar.rs` — custom title bar, tab nav, window controls
- `crates/undone-ui/src/left_panel.rs` — story panel, centered prose, detail strip, choices bar
- `crates/undone-ui/src/right_panel.rs` — stats sidebar, NPC panel, mode toggle
- `crates/undone-ui/src/theme.rs` — ThemeColors, ThemeMode, UserPrefs, save/load prefs
- `crates/undone-ui/src/game_state.rs` — PreGameState, GameState, init_game(), start_game()
- `.interface-design/system.md` — full design system spec

---

## Planned Future Sessions

1. ~~Scene engine~~ ✅
2. ~~Scheduler~~ ✅
3. ~~Save / load~~ ✅
4. ~~Design research~~ ✅
5. ~~UI quality pass~~ ✅
6. ~~NPC spawning + character creation~~ ✅
7. ~~UI polish~~ ✅
8. ~~Writing guide~~ ✅
9. ~~Engineering hardening~~ ✅
10. ~~Character creation UI~~ ✅
11. ~~Writing import~~ ✅ (3 scenes with original prose)
12. ~~Names update~~ ✅
13. ~~Saves tab~~ ✅
14. **Settings tab UI** — expose UserPrefs as interactive controls
15. **More scenes** — expand base pack content

---

## Recently Completed (overnight autonomous session)

- ✅ Names update — British → NE US (30 male, 30 female, multicultural)
- ✅ Settings persistence — UserPrefs saved to `%APPDATA%/undone/prefs.json`, survives restart
- ✅ Character creation UI — full-screen form with text inputs, dropdowns, checkboxes, trait selection, content prefs; `AppPhase` system splits init into `PreGameState` → `start_game()`
- ✅ Saves tab UI — save/load/delete with `undone-save`; saves to `%APPDATA%/undone/saves/`
- ✅ rain_shelter rewrite — proper prose, 5 trait branches, transformation dimension, 4 player actions, game flag persistence
- ✅ morning_routine scene — domestic intro, mirror moment, wardrobe trait branches, coffee decision, NE US details
- ✅ coffee_shop scene — NPC interaction, trait-dependent dialogue, sit-with-him path, transformation texture, game flags

## Open Items — Future Sessions

- **Settings tab UI** — expose font size, line height as interactive controls (Medium)
- **More base pack scenes** — apartment, work, social events, evening activities (Large)
- **Window drag on char creation** — no title bar during char creation means no drag area (Small)
- **Save metadata display** — show player name / week in save list without full deserialization (Small)

---

## Session Log

| Date | Summary |
|---|---|
| 2026-02-21 | Design session: decompiled Newlife, designed Undone engine, wrote scaffold plan |
| 2026-02-21 | Tooling session: built rhai-mcp-server + minijinja-mcp-server, wired MCP + hooks |
| 2026-02-22 | Scaffold session: Tasks 1–3 complete. MCP confirmed working. Added agentic workflow rules. |
| 2026-02-22 | Scaffold session: Tasks 4–13 complete. Parallel agents for Tasks 7–11. 30 tests pass. Scaffold done. |
| 2026-02-22 | Scene engine: brainstorm + design. Flat pool model, event queue API, full backend scope. |
| 2026-02-22 | Scene engine: 10-task implementation. Pack loader, eval wiring, effect system, minijinja templates, SceneEngine, rain shelter scene. 58 tests, 0 warnings. |
| 2026-02-22 | Scene engine: code audit + cleanup. Merged to master, worktree removed. |
| 2026-02-22 | Autonomous session: Scheduler + Save/Load. 70 tests, 0 warnings. Merged to master. |
| 2026-02-22 | Design research session: character creation, NPC spawning, UI patterns, personality arch. All open questions resolved. |
| 2026-02-22 | NPC spawning + char creation: 7-task plan via agent team. Sexuality/Personality enums, Player three-name system, NPC spawner with diversity guarantees, new_game() factory. 85 tests, 0 warnings. Merged to master. |
| 2026-02-22 | Code audit: reviewer + simplifier. Fixed diversity guarantee bug (.take() truncated required personalities for male_count < 3). Simplified pick_traits to use choose_multiple. |
| 2026-02-22 | Planning session: UI plan written. Setting pivot to fictional NE US city. Design system init (Evening Reader / three modes). system.md written. |
| 2026-02-22 | UI implementation: floem migration + layout (Gemini-authored, unreviewed). 87 tests pass. Warm Paper theme. Scene boots. Module split. |
| 2026-02-23 | UI Quality Pass: Added Dark/Sepia theme modes, keyboard navigation, fixed clippy warnings. |
| 2026-02-23 | UI Review + Fixes: £→$, window size (1200×800), panels swapped (stats left), mode toggle added. Built screenshot-mcp (WGC, no focus steal, Content::image). Registered in .mcp.json. |
| 2026-02-23 | screenshot-mcp debug: fixed stop()→wait() race condition in capture_window(). New binary at .exe.new. .mcp.json updated. Restart Claude Code to activate. UI audit complete — 6 violations documented in HANDOFF ready to implement. |
| 2026-02-23 | UI polish: screenshot-mcp verified working. Applied 5/6 audit fixes (focus_visible, single seam, chrome font, hover signal, border-radius 4px). Fix 3 letter_spacing not available in floem 0.2. Window config + panel swap committed. Code reviewed — fixed missed NPC name font, double prefs.get(), renamed left_panel→story_panel / right_panel→sidebar_panel. Merged to master. |
| 2026-02-23 | Writing guide session: docs/writing-guide.md written. NE US locale, Minijinja syntax, FEMININITY dial, four transformation textures, content gating (BLOCK_ROUGH/LIKES_ROUGH), markdown in prose, scene design principles, full checklist. Adapted from newlife-plus writing-style.md + scene-design.md. Added to CLAUDE.md key documents. |
| 2026-02-23 | UI session: 3-agent team. Custom title bar (no OS chrome, Game/Saves/Settings nav, window controls). Prose centered in story panel. Choice detail strip (hover shows action.detail). Sepia theme darkened (warm amber-cream, not muddy). 87 tests pass. Documented game-input-mcp plan (PostMessage, no focus steal). |
| 2026-02-23 | Built game-input-mcp: press_key + click tools via PostMessage, no focus steal. Release binary built, .mcp.json updated. Restart to activate. |
| 2026-02-23 | Engineering hardening session: 3-agent team. Window resize grips, prose centering, single-instance (fs4), Display impls for all domain enums, lexer overflow fix, engine expects, scheduler wired to SceneFinished, multi-pack scene loading, pack error visibility. 88 tests, 0 warnings. |
| 2026-02-23 | Engineering batch: 4 parallel agents in worktrees. packs_dir fix, female NPC effects, NpcActivated event, Literata font embed, markdown prose rendering. 95 tests, 0 warnings. |
| 2026-02-23 | Engineering hardening 2: FEMININITY unified (removed Player.femininity field, reads from skills map), w.hasStuff() wired to player inventory via StuffId registry, stats registration added to pack system (stats.toml), panics eliminated in error-recovery paths, spawner unwraps hardened. 100 tests, 0 warnings. |
| 2026-02-23 | Overnight autonomous session: 7 tasks via subagent-driven-development. Names → NE US, settings persistence (dirs + serde_json), character creation UI (AppPhase, PreGameState/GameState split, full form with floem widgets), saves tab (save/load/delete), rain_shelter rewrite (proper prose, 5 trait branches, transformation), morning_routine scene (domestic, mirror, wardrobe, Dunkin'), coffee_shop scene (NPC interaction, sit-with-him path, game flags). 104 tests, 0 warnings. |
