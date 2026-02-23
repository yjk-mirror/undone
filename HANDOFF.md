# Undone — Handoff

## Current State

**Branch:** `master`
**Tests:** 87 passing, 0 clippy warnings.
**App:** Boots and runs. Sidebar (stats) left, story/choices right. Three theme modes. screenshot-mcp working.

---

## ⚡ Next Action: Writing Guide session

Begin writing guide session: continuity-of-self principles, transformation writing, NE US voice, delta-awareness.

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
- `crates/undone-ui/src/lib.rs` — AppSignals, snapshots, app_view, process_events
- `crates/undone-ui/src/left_panel.rs` — story panel + choices bar
- `crates/undone-ui/src/right_panel.rs` — stats sidebar, NPC panel, mode toggle
- `crates/undone-ui/src/theme.rs` — ThemeColors, ThemeMode, UserPrefs
- `crates/undone-ui/src/game_state.rs` — GameState, init_game()
- `.interface-design/system.md` — full design system spec

---

## Planned Future Sessions

1. ~~Scene engine~~ ✅
2. ~~Scheduler~~ ✅
3. ~~Save / load~~ ✅
4. ~~Design research~~ ✅
5. ~~UI quality pass~~ ✅
6. ~~NPC spawning + character creation~~ ✅
7. **UI polish** ← current. Fix 6 violations, screenshot-verify, merge.
8. **Writing guide** — Continuity-of-self principles, transformation writing, delta-awareness, NE US voice.
9. **Writing import** — Original prose for base pack
10. **Names update** — names.toml British → NE US

---

## Open Items

- Focus-stays-after-click in floem — being fixed this session (Fix 1 above)
- `w.hasStuff()` returns false (StuffId registry stub) — needed when inventory matters
- `PersonalityId` Display impl missing — using Debug format in NPC panel for now
- Literata font loading from disk — deferred; using Georgia fallback
- Markdown in prose (pulldown-cmark → floem RichText) — planned, not yet designed
- `packs/base/data/names.toml` has British names — writing session

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
