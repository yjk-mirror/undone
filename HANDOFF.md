# Undone — Handoff

## Current State

**Phase:** Scheduler + Save/Load merged to master. Backend complete.

70 tests pass, zero clippy warnings. All work on `master`.

**Scheduler** (`undone-scene/src/scheduler.rs`):
- `load_schedule(pack_metas)` → `Scheduler` — loads `schedule.toml` from each pack
- `Scheduler::pick(slot, world, registry, rng)` → `Option<String>` — weighted random selection
- Conditions use the existing expr evaluator with an empty SceneCtx
- Base pack defines `free_time` slot with `rain_shelter` event (condition: `gd.week() > 0`)
- 6 scheduler tests

**Save/Load** (`undone-save/src/lib.rs`):
- `save_game(world, registry, path)` — writes `SaveFile { version, id_strings, world }` as JSON
- `load_game(path, registry)` — validates ID stability before returning `World`
- ID validation: saved `id_strings` (Spur-index order) must match current registry
- Errors: `IdMismatch` (pack content changed), `TooManyIds` (pack removed), `VersionMismatch`
- 6 save tests including full round-trip

**Supporting changes**:
- `PackContent.schedule_file: Option<String>` in pack manifests
- `PackRegistry::all_interned_strings()` for save ID validation
- `World: Clone`

## Next Action

**UI design session** — dedicated session. Typography, layout, choice presentation.
The backend is complete. See UI Direction in engine-design.md.

Note on open questions (from engine-design.md):
- NPC spawning / pool seeding at game start — not yet designed
- Character creation flow — not yet designed
- `w.hasStuff()` returns false (StuffId registry stub) — needed when inventory matters

## Planned Future Sessions

1. ~~Scene engine~~ ✅
2. ~~Scheduler~~ ✅
3. ~~Save / load~~ ✅
4. **UI design** — Dedicated session; typography, layout, choice presentation (NOT egui — see engine-design.md)
5. **NPC spawning + character creation** — needs design session first
6. **Writing import** — Port and improve original prose from `newlife-plus`

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
| 2026-02-22 | Autonomous session: Scheduler + Save/Load. 70 tests, 0 warnings. Merged to master. |
