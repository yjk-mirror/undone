# Undone — Handoff

## Current State

**Phase:** UI floem migration done (Gemini, unreviewed). Needs Claude audit + quality pass
before merging to master. 87 tests pass, cargo check clean, on `master` branch.

**UI — floem migration** (this session, Gemini-authored — treat as draft):
- egui/eframe removed. floem 0.2.0 wired across workspace and `undone-ui`
- Module split: `lib.rs` (entry, AppSignals), `left_panel.rs`, `right_panel.rs`, `game_state.rs`
- `PlayerSnapshot` / `NpcSnapshot` structs — UI never borrows World directly
- `AppSignals` (RwSignal bundle) drives all reactive updates
- Scene event loop wired: choice buttons → `EngineCommand` → drain events → update signals
- App boots into `base::rain_shelter` scene
- Warm Paper color tokens applied from `.interface-design/system.md`
- Floem 0.2.0 adaptations noted by Gemini (see Technical Notes below)

**Setting pivot** (this session):
- Base pack setting changed from British to **fictional Northeast US city, near-future**
- `CLAUDE.md`, `docs/plans/2026-02-22-design-decisions.md` updated accordingly
- `docs/plans/2026-02-22-setting-principles.md` written — setting canon doc
- `packs/base/data/names.toml` still has British names — **needs updating**

**Design system** (this session):
- `.interface-design/system.md` written — full token set, three modes, typography, layout rules
- Design direction: "Evening Reader" — research-backed (Choice of Games, Instapaper, iA Writer,
  Degrees of Lewdity redesign, NN/G dark mode study)

**Previous sessions** (still in place):
- Scene engine, scheduler, save/load, NPC spawner, char creation — all working

---

## Next Action: UI Quality Pass

**The Gemini UI code is structurally complete but needs a thorough review.** Gemini rushes
toward least-friction code; assume the implementation is functional but not clean or complete.

### Known gaps and concerns (audit these specifically):

**Not implemented at all — Gemini didn't attempt:**
- [ ] Dark mode and Sepia mode (only Warm Paper was mentioned)
- [ ] `UserPrefs` struct — no configurable font/size/mode
- [ ] Keyboard navigation: number keys 1–9 select choices by position, Tab/Enter activate
- [ ] Markdown rendering in prose (pulldown-cmark → floem RichText — intentionally deferred)

**Quality concerns — verify these:**
- [ ] All five choice button states present: default, hover, focus (2px ring), active, disabled
- [ ] `letter_spacing` omitted from stat labels (floem doesn't support it) — acceptable for now
- [ ] Font is Georgia fallback, not Literata (Literata loading is a future task) — acceptable
- [ ] `ref mut` destructuring in game init — may be awkward, refactor if needed
- [ ] `PersonalityId` uses Debug formatting — acceptable for now, note it
- [ ] `rand` and `slotmap` added to `undone-ui/Cargo.toml` — verify these are actually needed
  at the UI level vs. being pulled through `undone-packs`/`undone-world` transitively
- [ ] No clippy pass run — run `cargo clippy -- -D warnings`, fix all warnings
- [ ] Code style, naming, redundant code, missing error handling — general Gemini cleanup

**UI plan tasks intentionally NOT done by Gemini — do these in the quality pass:**
- [ ] Task 11 (cleanup): cargo clippy, format, test suite run
- [ ] `superpowers:finishing-a-development-branch` — merge decision

### Floem 0.2.0 adaptations Gemini made (verify each is correct):
- `default-font` feature removed from Cargo.toml (was removed in 0.2.0)
- `font_family` requires `String` in some Style contexts
- `letter_spacing` not in Style API — stat label letter-spacing silently omitted

---

## For the Quality Pass Session

1. Invoke `superpowers:requesting-code-review` — audit Gemini's work against the plan and system.md
2. Fix issues found
3. Add the three missing features: Dark/Sepia modes, UserPrefs, keyboard navigation
4. `cargo clippy -- -D warnings` — zero warnings
5. `cargo test` — all 87 tests pass
6. Invoke `superpowers:finishing-a-development-branch`

The plan file for reference: `docs/plans/2026-02-22-ui-floem.md`
The design system: `.interface-design/system.md`

---

## Planned Future Sessions

1. ~~Scene engine~~ ✅
2. ~~Scheduler~~ ✅
3. ~~Save / load~~ ✅
4. ~~Design research~~ ✅
5. **UI quality pass** — audit + fix Gemini's work, add dark/sepia modes, UserPrefs, keyboard nav
6. ~~NPC spawning + character creation~~ ✅
7. **Writing guide** — Continuity-of-self principles, transformation writing, delta-awareness,
   Northeast US setting voice. Write before any prose work.
8. **Writing import** — Original prose for the base pack (not a port — new setting, new content)
9. **Names update** — `packs/base/data/names.toml` British → American Northeast names

---

## Open Items (not session-specific)

- `w.hasStuff()` still returns false (StuffId registry stub) — needed when inventory matters
- `PersonalityId` Display impl missing — using Debug format in UI for now
- Literata font loading from disk — deferred; using Georgia fallback currently
- Markdown in prose (pulldown-cmark → floem RichText) — planned, not yet designed

---

## Agentic Workflow Reminder

- Background implementation agents need `mode: "bypassPermissions"` — now in global CLAUDE.md
- Use `mcp__rust__get_diagnostics` + `mcp__rust__format_code` after writing each `.rs` file
- Use worktrees for post-scaffold sessions (master is the scaffold baseline)
- Agent teams: `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` in settings env

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
| 2026-02-22 | Planning session: UI plan written (docs/plans/2026-02-22-ui-floem.md). Setting pivot to fictional NE US city. Design system init (Evening Reader / three modes). system.md written. |
| 2026-02-22 | UI implementation: floem migration + layout (Gemini-authored, unreviewed). 87 tests pass. Warm Paper theme. Scene boots. Module split. Missing: dark/sepia modes, UserPrefs, keyboard nav. |
