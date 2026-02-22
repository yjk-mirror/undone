# Undone — Handoff

## Current State

**Phase:** Scaffold complete. All 13 tasks done.

30 tests pass, zero clippy warnings, eframe window builds.
See `docs/status.md` for full task-by-task history.

## Next Action

Design and implement the **scene engine** — the first post-scaffold session.

Start with `superpowers:brainstorming` to design the scene engine, then
`superpowers:writing-plans` for the implementation plan, then
`superpowers:using-git-worktrees` before touching code.

Key design questions for that session (pre-thought):
- TOML scene format → `SceneDefinition` structs (already partially designed in engine-design.md)
- Effect enum deserialization and execution against `&mut World`
- Scene execution loop (action selection → prose rendering → effect application)
- Wiring expression evaluator stubs to `PackRegistry` (hasTrait, getSkill)
- Pack loader: walk `packs/` directory, load manifests, build registry

## Planned Future Sessions

1. **Scene engine** ← next
2. **Scheduler** — Weekly timeslots, weighted scene selection
3. **UI design** — Dedicated session; typography, layout, choice presentation
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
