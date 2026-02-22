# Undone — Handoff

## Current State

**Phase:** Scene engine merged to master. Ready for Scheduler.

58 tests pass, zero clippy warnings. All work on `master`.

Pack disk loader, expression evaluator wired to registry, typed effect system,
minijinja prose rendering, and SceneEngine with event queue all implemented and tested.
Code audit cleanup applied (effect error logging, zero-weight guard, dead code removal,
loader validation, template correctness).

Rain shelter scene demonstrates full end-to-end flow: load packs → load scene →
start scene → NPC fires → condition gating → prose branching on traits → finish.

## Next Action

Design and implement the **Scheduler** —
weekly timeslots, weighted scene selection, pack-contributed event pools.

## Planned Future Sessions

1. ~~Scene engine~~ ✅
2. **Scheduler** — Weekly timeslots, weighted scene selection ← next
3. **UI design** — Dedicated session; typography, layout, choice presentation (gtk4/relm4, slint, or floem — NOT egui)
4. **Save / load** — serde JSON save format, versioning
5. **Writing import** — Port and improve original prose from `newlife-plus`

## Agentic Workflow Reminder

- Background implementation agents need `mode: "bypassPermissions"` — now in global CLAUDE.md
- Use `mcp__rust__get_diagnostics` + `mcp__rust__format_code` after writing each `.rs` file
- Use worktrees for post-scaffold sessions (master is the scaffold baseline)

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
