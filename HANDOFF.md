# Undone — Handoff

## Current State

**Phase:** UI floem migration done (Gemini, unreviewed). Needs Claude audit + quality pass
before merging to master. 87 tests pass, cargo check clean, on `master` branch.
**Update:** Claude audit and quality pass completed. All missing UI features implemented. 87 tests pass, clippy is clean. On `ui-floem-migration` branch.

**UI — floem migration** (this session, Gemini-authored — treat as draft):
- egui/eframe removed. floem 0.2.0 wired across workspace and `undone-ui`
- Module split: `lib.rs` (entry, AppSignals), `left_panel.rs`, `right_panel.rs`, `game_state.rs`, `theme.rs`
- `PlayerSnapshot` / `NpcSnapshot` structs — UI never borrows World directly
- `AppSignals` (RwSignal bundle) drives all reactive updates
- Scene event loop wired: choice buttons → `EngineCommand` → drain events → update signals
- App boots into `base::rain_shelter` scene
- Dark mode, Sepia mode, and Warm Paper color tokens applied from `.interface-design/system.md`
- Keyboard navigation (1-9, Tab/Enter) implemented for choice buttons.
- Floem 0.2.0 pseudo-class styling applied: hover, focus, active, disabled.

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

## Next Action: Writing Guide and Prose

**The UI code is structurally complete and verified.**

### Resolved gaps and concerns:

**Implemented in Quality Pass:**
- [x] Dark mode and Sepia mode
- [x] `UserPrefs` struct — configurable font/size/mode
- [x] Keyboard navigation: number keys 1–9 select choices by position, Tab/Enter activate
- [x] All five choice button states present: default, hover, focus (2px ring), active, disabled
- [x] `rand` and `slotmap` in `undone-ui/Cargo.toml` — verified needed for game init

**Deferred / Noted:**
- [ ] Markdown rendering in prose (pulldown-cmark → floem RichText — intentionally deferred)
- [ ] `PersonalityId` uses Debug formatting — acceptable for now, noted as an open item.

### Writing guide and content

1. Establish Continuity-of-self principles, transformation writing, delta-awareness, Northeast US setting voice.
2. Write before any prose work.
3. Replace placeholder `base::rain_shelter` text with real base pack prose.

---

## Planned Future Sessions

1. ~~Scene engine~~ ✅
2. ~~Scheduler~~ ✅
3. ~~Save / load~~ ✅
4. ~~Design research~~ ✅
5. ~~UI quality pass~~ ✅ (Audit + fix Gemini's work, add dark/sepia modes, UserPrefs, keyboard nav)
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
| 2026-02-23 | UI Quality Pass: Added Dark/Sepia theme modes, keyboard navigation, fixed clippy warnings. UI is complete. |
