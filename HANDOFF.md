# Undone — Handoff

## Current State

**Branch:** `feat/pc-origin-system` (ready to merge to master)
**Tests:** 112+ passing, 0 failures.
**App:** Character creation → gameplay loop working. Two-step "Your Past" flow: was/wasn't transformed → which kind. Four PC origin types: CisMaleTransformed (FEMININITY=10), TransWomanTransformed (FEMININITY=70), CisFemaleTransformed (FEMININITY=75), AlwaysFemale (FEMININITY=75). Hidden traits auto-injected by new_game(). Save format v2 with v1 migration. Trans woman branches in all 3 scenes. Writing guide updated with four-origin model and emotional register guidance.

---

## ⚡ Next Action

Priority tasks:
1. **Keyboard controls redesign** — arrow keys for choice highlight, configurable number-key behavior (instant vs highlight+confirm)
2. **More scenes** — expand base pack content
3. **Settings tab UI** — expose font size, line height as interactive controls

---

## game-input-mcp — Updated

**Tools:** `press_key(title, key)`, `click(title, x, y)`, `scroll(title, x, y, delta)`, `hover(title, x, y)`.
Keys: `"1"`–`"9"`, `"enter"`, `"tab"`, `"escape"`, `"space"`.
Scroll: positive delta = up, negative = down (one tick = one wheel notch).
Hover: sends WM_MOUSEMOVE to trigger hover effects.

**New binary:** `undone-tools/target/release/game-input-mcp.exe.new` (built with scroll+hover).
**Deploy:** Restart Claude Code. On next session, rename `.exe.new` → `.exe` (old one is locked while running).

---

## screenshot-mcp — Working

**Bug fixed (2026-02-23):** `capture_window()` was calling `control.stop()` instead of
`control.wait()`. Fixed binary deployed to `undone-tools/target/release/screenshot-mcp.exe`.
**.mcp.json** points to `screenshot-mcp.exe`. Verified working.

---

## UI — Current State

**Layout:**
- Title bar always visible: UNDONE branding, Game/Saves/Settings tabs, window controls
- Stats sidebar on the **left** (280px fixed): player name, stats, NPC panel, mode toggle
- Story + choices on the **right** (flex-grow): scrollable prose + choices bar
- Window opens at 1200×800, titled "Undone"

**Scroll (floem):**
- All scroll containers use `.scroll_style(|s| s.shrink_to_fit())` — required for floem scroll in flex layouts
- Story panel scroll: `flex_grow(1.0).flex_basis(0.0)` (flex sibling of detail strip + choices bar)
- Char creation scroll: `size_full()` (sole child of dyn_container)
- Outer dyn_container: `flex_grow(1.0).flex_basis(0.0).min_height(0.0)` — required so taffy constrains children

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
14. ~~**Character creation redesign**~~ ✅ (PcOrigin system: two-step flow, 4 origin types, trans woman PC type)
15. **Keyboard controls redesign** — arrow key highlight, configurable instant vs confirm
16. **Settings tab UI** — expose UserPrefs as interactive controls
17. **More scenes** — expand base pack content

---

## Open Items — Future Sessions

### Character Creation (Remaining Small Items)
- **Trait checkbox UX**: Label text is currently drag-selectable; clicking label should toggle checkbox instead
- **Age before transition**: Should be a dropdown (matching "Age" field), not a text input
- **Form density**: Form too tall for 800px window; consider tighter spacing or two-column layout

### Keyboard Controls (Medium)
- **Arrow key navigation**: Highlight choices with arrow keys, show detail strip for highlighted choice
- **Number key behavior**: Configurable — instant action vs highlight-then-confirm (press number → highlight, press again or Enter → confirm)
- **Current limitation**: Number keys (1-9) currently fire instantly; no highlight-first mode

### UI Polish (Small-Medium)
- **Detail strip hover highlight**: Brief unwanted background highlight in Warm theme on first hover (floem default style leak — partially fixed with explicit hover/focus overrides)
- **Choice button positioning**: Consider better visual balance between prose area and choices
- **Save metadata display**: Show player name / week in save list without full deserialization

### Tooling
- **game-input scroll/hover**: Built but not deployed (exe locked). Rename `.exe.new` → `.exe` on next restart.
- **game-input limitation**: PostMessage-based input may not establish focus like real user input — keyboard shortcuts may not fire after PostMessage click

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
| 2026-02-23 | Playtest + bugfix session: Fixed 3 bugs — char creation skipped (title bar now always visible), scroll broken (floem shrink_to_fit + flex_basis(0)), take().unwrap() crash (replaced with match). Added Runtime Testing Notes to CLAUDE.md. Built game-input-mcp scroll + hover tools. Documented char creation redesign ideas (male-first flow, keyboard controls). 104 tests, 0 failures. |
| 2026-02-23 | PC Origin System: Replace always_female:bool with PcOrigin enum (CisMaleTransformed/TransWomanTransformed/CisFemaleTransformed/AlwaysFemale). Two-step char creation flow. Trans woman PC type (FEMININITY=70). Auto-inject hidden traits in new_game(). w.pcOrigin() evaluator accessor. Save v2 with v1 migration. Trans woman branches in all 3 scenes. Writing guide updated with four-origin model + emotional register guidance. 112+ tests, 0 failures. |
