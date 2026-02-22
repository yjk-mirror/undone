# Undone — Handoff

## Current State

**Phase:** Scaffold in progress — Tasks 1–3 of 13 complete.

Workspace initialised, domain enums and ID types exist. Rust code compiles.
See `docs/status.md` for full task-by-task status.

## Next Action

Continue scaffold — Tasks 4–13 remaining.

```
docs/plans/2026-02-21-scaffold.md   (plan)
docs/status.md                       (progress tracker — update after each task)
```

Use `superpowers:executing-plans`. Resume from **Task 4** (Player struct).

**Before starting implementation:**
- MCP servers confirmed working (`rust`, `rhai`, `minijinja`) ✅
- Use `mcp__rust__get_diagnostics` + `mcp__rust__format_code` after writing each `.rs` file
- See CLAUDE.md "Agentic Workflow" section for full tooling rules

## What Was Completed (Design Session, 2026-02-21)

- Decompiled the Newlife Java engine via `javap` — recovered full API surface:
  28 scene transition types, 100+ NPC traits, all 9 player skills, undocumented methods
- Designed the Undone Rust engine architecture from scratch
- Chose tech stack: Rust + egui/eframe + minijinja + lasso + slotmap
- Decided on Approach B scene format: single TOML per scene, typed effects, Jinja2 prose
- Decided on custom recursive descent expression parser (Option C)
- Wrote full engine design document
- Wrote 13-task scaffold implementation plan
- Initialised git repo

## Scaffold Plan Summary (Tasks 1–13)

| Task | What it builds |
|---|---|
| 1 | Workspace `Cargo.toml` + 7 empty crate stubs |
| 2 | Engine-level enums (ArousalLevel, AlcoholLevel, etc.) in `undone-domain` |
| 3 | Content ID types (TraitId, SkillId, etc. via lasso) in `undone-domain` |
| 4 | Player struct in `undone-domain` |
| 5 | NpcCore, MaleNpc, FemaleNpc structs in `undone-domain` |
| 6 | World + GameData structs in `undone-world` |
| 7 | Pack manifests + base data TOML files in `packs/base/` |
| 8 | PackRegistry with lasso interning in `undone-packs` |
| 9 | Expression lexer in `undone-expr` |
| 10 | Expression parser (recursive descent AST) in `undone-expr` |
| 11 | Expression evaluator + SceneCtx in `undone-expr` (with `todo!()` stubs) |
| 12 | Minimal eframe window in `undone-ui` + `src/main.rs` |
| 13 | Final verification: `cargo test --workspace`, `cargo clippy`, `cargo build --release` |

**Note on Tasks 9–11:** The evaluator stubs (`hasTrait`, `getSkill`, etc.) are
intentional. They get wired to `PackRegistry` in the scene engine session.

## Planned Future Sessions (after scaffold)

1. **Scene engine** — TOML scene loader, effect executor, scene execution loop,
   scene stack, minijinja prose rendering, wiring expression evaluator stubs
2. **Scheduler** — Weekly timeslots, weighted scene selection, pack injection
3. **UI design** — Dedicated session to design the full game UI from scratch.
   Not constrained by the original Newlife layout. What makes this most engaging?
4. **Save / load** — serde JSON save format, versioning strategy
5. **Writing import** — Port and improve original prose from `newlife-plus`

## Session Log

| Date | Summary |
|---|---|
| 2026-02-21 | Design session: decompiled Newlife, designed Undone engine, wrote scaffold plan |
| 2026-02-21 | Tooling session: built rhai-mcp-server (4 tools) + minijinja-mcp-server (3 tools) in undone-tools/, wired .mcp.json + PostToolUse hook + skills here. Smoke test pending (restart required for MCP). |
| 2026-02-22 | Scaffold session: Tasks 1–3 complete (workspace, enums, ID types). MCP tooling confirmed working. Added agentic workflow rules to CLAUDE.md and docs/status.md as living tracker. |
