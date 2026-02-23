# Undone — Project Instructions

## What This Is

**Undone** is a life-simulation adult text game engine with a transgender/transformation
premise. Built from scratch in Rust. Inspired by Newlife (Splendid Ostrich Games) but
fully redesigned: new engine, new content format, complete ownership.

The engine is a **platform**. The first release is set in a fictional Northeast US city
(near-future, date unspecified). Other settings, stories, and cultural contexts are
first-class citizens — not afterthoughts. Extensibility is a core design constraint,
not a future enhancement.

### The Premise (shared across all stories)

A player character navigates adult life — relationships, work, social dynamics. She may
have started life as a man. The transformation is not backstory; it is a lens that
changes how every socially-gendered experience lands. She knows how men think because
she was one. Different story packs will use this premise in different settings.

### The Three PC Types

| Type | `always_female` | Description |
|---|---|---|
| Male-start (transformed) | `false` | Definitively transformed from male. The primary experience. |
| Female-start (variant) | `true`, no `NOT_TRANSFORMED` | Female from birth with a transformation element |
| Always female | `true` + `NOT_TRANSFORMED` | No transformation frame at all |

The `FEMININITY` skill (0–100+) tracks adaptation. Male-start begins low; always-female
begins at 75. The richest transformation writing lives in the 0–50 range.

## Key Documents (Read Before Working)

- `docs/plans/2026-02-21-engine-design.md` — Living architecture document. The
  authoritative design reference. **Update it when implementation reveals surprises.**
- `docs/plans/2026-02-21-scaffold.md` — 13-task scaffold plan. ✅ Complete.
- `HANDOFF.md` — Current state and session log. **Always read this first.**
- `docs/status.md` — Scaffold progress history (all 13 tasks done).

## Tech Stack

| Concern | Choice |
|---|---|
| Language | Rust (workspace, 7 crates) |
| GUI | floem (reactive, Lapce team, single binary) |
| Template rendering | minijinja (Jinja2 syntax) |
| Scene conditions | Custom recursive descent parser (validated at load time) |
| Serialisation | serde + serde_json + toml |
| NPC storage | slotmap (stable typed keys) |
| String interning | lasso (TraitId/SkillId/etc as u32) |

## Workspace Structure

```
undone/
├── Cargo.toml               # workspace root
├── src/main.rs              # entry point
├── crates/
│   ├── undone-domain/       # pure types — no IO, no game logic
│   ├── undone-world/        # World struct, all mutable game state
│   ├── undone-packs/        # pack loading, manifest parsing, content registry
│   ├── undone-expr/         # custom expression parser & evaluator
│   ├── undone-scene/        # scene execution engine
│   ├── undone-save/         # serde save / load
│   └── undone-ui/           # floem views and widgets
└── packs/
    └── base/                # base game content (is itself a pack)
```

## Design Philosophy

- **Platform, not product.** The engine is setting-agnostic. All content — traits,
  skills, stats, scenes, NPC personalities, cultural references — lives in packs.
  The base game is set in a fictional Northeast US city. Nothing setting-specific is
  hardcoded into the engine.

- **Redesign, not port.** The Java source is obfuscated. We reverse-engineered the
  API surface for reference only. The Rust engine is designed from scratch.

- **Data-driven everywhere.** Traits, skills, stats — TOML data files in each pack.
  The engine reasons about enums for closed sets (arousal, alcohol, relationship
  status); content-level IDs are interned strings validated at pack load time.

- **Pack system is the extensibility mechanism.** Community packs, alternate settings,
  and future first-party stories all drop into `packs/`. The scheduler, NPC spawner,
  and scene registry are all pack-aware.

- **Content validated at load time.** Unknown trait/skill names in scene files fail
  fast with a clear error before the game runs.

- **Writing is everything.** The engine exists to serve the prose.

- **Transformation is structurally present.** Every scene that involves gendered
  social dynamics should ask: does this feel different for a woman who used to be
  a man? If yes, write the branch. This is not optional flavour — it is the
  game's distinctive register.

## UI Direction

The UI is a significant open design question. We are not replicating the original
Newlife layout. A dedicated design session will determine what makes this the most
engaging experience — typography, layout, how choices are presented, how the world
and character state are surfaced.

The scaffold produced a minimal eframe window (900×600, placeholder text, no logic).
Do not iterate on the UI until the scene engine can run scenes end-to-end.

## Writing and Content

The base pack is set in a fictional Northeast US city (near-future). All prose is
original — the setting deliberately diverges from Newlife's British context to
differentiate. Do not work on prose content until the engine can run scenes end-to-end
and the writing guide session has established continuity-of-self principles.

## Expression Parser Notes

The expression system (lexer + recursive descent parser + evaluator) is complete.
The evaluator returns stub values (`false`/`0`) for `hasTrait()`, `getSkill()`,
`hasStuff()`, `getStat()` — these are wired to `PackRegistry` in the scene engine
session. The stubs are marked `// TODO: wire to registry` in `undone-expr/src/eval.rs`.

One deviation from the original plan: `gd.week` was changed to `gd.week()` — the
parser requires method-call syntax everywhere, and the original plan had an inconsistency.

## Agentic Workflow

### Skills — required invocations

| Situation | Required skill |
|---|---|
| Starting a plan | `superpowers:executing-plans` |
| Before touching code on a plan | `superpowers:using-git-worktrees` (worktree per plan) |
| Debugging any failure | `superpowers:systematic-debugging` |
| About to claim done | `superpowers:verification-before-completion` |
| Finishing a branch | `superpowers:finishing-a-development-branch` |

### MCP Tools — use these instead of raw Bash

**Rust** (prefer over `cargo` in Bash for per-file work):

| Task | Tool |
|---|---|
| Check compilation errors | `mcp__rust__run_cargo_check` |
| Diagnostics on a specific file after writing it | `mcp__rust__get_diagnostics` |
| Format a file after writing it | `mcp__rust__format_code` |
| Find references / definitions | `mcp__rust__find_references`, `mcp__rust__find_definition` |

Still use `cargo test` / `cargo build --release` via Bash for workspace-wide commands
and release builds — those don't have MCP equivalents.

**Minijinja** (use after writing any `.j2` template):

| Task | Tool |
|---|---|
| Validate template syntax | `mcp__minijinja__jinja_validate_template` |
| Preview render with test data | `mcp__minijinja__jinja_render_preview` |

**Rhai** (use after writing any `.rhai` script):

| Task | Tool |
|---|---|
| Syntax check | `mcp__rhai__rhai_check_syntax` |
| Full diagnostics with runtime errors | `mcp__rhai__rhai_get_diagnostics` |
| Validate a file | `mcp__rhai__rhai_validate_script` |

### Workflow for implementing a plan

1. Invoke `superpowers:executing-plans`
2. Invoke `superpowers:using-git-worktrees` — create a worktree for the plan
3. Execute tasks in batches of ~3, reporting between batches
4. After writing each `.rs` file: call `mcp__rust__get_diagnostics` + `mcp__rust__format_code`
5. After writing each `.j2` file: call `mcp__minijinja__jinja_validate_template`
6. When all tasks done: invoke `superpowers:finishing-a-development-branch`

### Dispatching background agents

When using parallel background subagents (Task tool with `run_in_background: true`):
- **Always** include `mode: "bypassPermissions"` — background agents cannot answer
  permission prompts and will silently block on every Bash/Write/Edit call without it
- **Never** have background agents commit to git — they may run concurrently and will
  corrupt the index. Have them implement + test only; the lead commits after review.
- Scope safety comes from the prompt, not the permission gate — be precise about
  which files/crates each agent owns

## Dependency Direction (enforced, no cycles)

```
undone-domain
    ↑
undone-world ← undone-packs
    ↑               ↑
undone-expr    undone-save
    ↑
undone-scene
    ↑
undone-ui
```
