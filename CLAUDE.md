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

### The Four PC Origins

| `PcOrigin` variant | `w.alwaysFemale()` | FEMININITY start | Description |
|---|---|---|---|
| `CisMaleTransformed` | `false` | 10 | Transformed from a cis man. The primary experience. |
| `TransWomanTransformed` | `false` | 70 | Transformed from a trans woman. Relief/recognition register. |
| `CisFemaleTransformed` | `true` | 75 | Transformed from a cis woman. Auto-injects `ALWAYS_FEMALE` trait. |
| `AlwaysFemale` | `true` | 75 | No transformation frame. Auto-injects `ALWAYS_FEMALE` + `NOT_TRANSFORMED`. |

The `FEMININITY` skill (0–100+) tracks adaptation. `CisMaleTransformed` begins low (10);
`TransWomanTransformed` begins at 70 (she knew herself already); `CisFemale`/`AlwaysFemale`
begin at 75. The richest transformation writing lives in the 0–50 range.

Hidden traits auto-injected at game start by `new_game()` based on origin — do not inject them
manually in UI code. Use `w.hasTrait("TRANS_WOMAN")` in scene templates to branch the
emotional register for trans woman PCs.

## Key Documents (Read Before Working)

- `docs/plans/2026-02-21-engine-design.md` — Living architecture document. The
  authoritative design reference. **Update it when implementation reveals surprises.**
- `docs/plans/2026-02-21-scaffold.md` — 13-task scaffold plan. ✅ Complete.
- `docs/writing-guide.md` — Prose standard for all scene content. **Read before
  writing any scene prose or designing a prose-writing agent.**
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
├── Cargo.toml               # game workspace root (game code only — do not add tools here)
├── src/main.rs              # entry point
├── crates/
│   ├── undone-domain/       # pure types — no IO, no game logic
│   ├── undone-world/        # World struct, all mutable game state
│   ├── undone-packs/        # pack loading, manifest parsing, content registry
│   ├── undone-expr/         # custom expression parser & evaluator
│   ├── undone-scene/        # scene execution engine
│   ├── undone-save/         # serde save / load
│   └── undone-ui/           # floem views and widgets
├── packs/
│   └── base/                # base game content (is itself a pack)
└── tools/                   # ⚠ AGENT DEVTOOLS — separate Cargo workspace, not game code
    ├── Cargo.toml           # tools workspace root (independent of game workspace)
    ├── rhai-mcp-server/     # MCP server: Rhai script validation
    ├── minijinja-mcp-server/# MCP server: Minijinja template validation + preview
    ├── screenshot-mcp/      # MCP server: screen capture for agent visual feedback
    ├── game-input-mcp/      # MCP server: keyboard/mouse/scroll injection into the game window
    └── rust-mcp/            # MCP server: rust-analyzer LSP integration (navigation, rename)
```

### Two separate Cargo workspaces

`Cargo.toml` at the root is the **game workspace** — `cargo build`, `cargo check`, and
`cargo test` run here for game development. Never add `tools/` members to this workspace.

`tools/Cargo.toml` is a separate **devtools workspace**. To build the MCP servers:

```sh
cd tools && cargo build --release
```

Binaries land in `tools/target/release/`. The `.mcp.json` at the repo root points here.
The game workspace and the devtools workspace share nothing at the Cargo level.

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

## Engineering Principles

These are constraints, not aspirations. Violating them is a bug.

1. **Fail fast, fail loud.** Invalid data is caught at load time, not runtime.
   Runtime errors are visible — never silently swallowed. A broken condition
   that defaults to `false` still logs the error.

2. **No hardcoded content IDs in engine code.** Scene IDs, slot names, skill
   names, trait names belong in pack data files. The engine reads from the
   registry. Structural IDs (like FEMININITY) must be declared as required
   skills in the pack manifest — not magic strings scattered across crates.

3. **Data-driven over code-driven.** If a value could come from a pack file,
   it should. The engine is a platform — it should not know what game it runs.

4. **No silent defaults for content errors.** A typo in a condition, an unknown
   trait name, a broken goto target — these are content bugs, not edge cases.
   Visible errors at the earliest possible moment: load time > runtime > silent.

5. **Bounded resources.** Stacks, buffers, and accumulating strings have
   depth/size limits. Unbounded growth is a latent crash.

6. **Separation of concerns across crate boundaries.** Engine logic stays out
   of the UI crate. UI concerns stay out of the domain crate. The dependency
   DAG is enforced and maintained.

7. **Tests before content.** New engine capabilities get tests before scenes
   use them. Content authors should never discover a broken engine feature first.

8. **No tech debt. No workarounds. No hacks.** Do it correctly the first time.
   If a proper solution requires more work, do the work — don't ship a shortcut
   and plan to fix it later. If you're unsure whether something is the right
   approach, surface it explicitly. The user will always choose correctness over
   speed. This applies to game code, tooling, infrastructure, and agent workflows
   equally. Workarounds accumulate; correct solutions compose.

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

## Runtime Testing Notes

- **floem scroll requires `shrink_to_fit()`.** A `scroll()` widget inside a flex
  container (v_stack/h_stack) must use `.scroll_style(|s| s.shrink_to_fit())` and
  `.style(|s| s.flex_grow(1.0).flex_basis(0.0))` — otherwise taffy sizes the
  scroll viewport to content height and scrolling never activates.
- **game-input MCP supports keys, clicks, scroll, and hover.** All four use
  PostMessage (no focus steal). `scroll(title, x, y, delta)` sends WM_MOUSEMOVE
  then WM_MOUSEWHEEL (floem routes wheel events using cached cursor_position,
  so the preceding WM_MOUSEMOVE is required for correct widget targeting).

## Guardrails — Runtime

### Background task completion ≠ game exit
- **Trigger**: A `cargo run` background task notification says "completed"
- **Rule**: This means the BUILD finished and the process was SPAWNED. The GUI
  window keeps running independently. **Never say the game exited or closed based
  on a background task completing.** Always verify with
  `Get-Process undone -ErrorAction SilentlyContinue` before making any claim about
  the game process state. If the process is running, the game is running.

## Agentic Workflow

### Skills — required invocations

| Situation | Required skill |
|---|---|
| Starting a plan | `superpowers:executing-plans` |
| Before touching code on a plan | `superpowers:using-git-worktrees` (worktree per plan) |
| Debugging any failure | `superpowers:systematic-debugging` |
| About to claim done | `superpowers:verification-before-completion` |
| Finishing a branch | `superpowers:finishing-a-development-branch` |

### Skill overrides

- **finishing-a-development-branch**: Always merge. Never offer "discard" as an
  option. Work is always worth keeping — skip the discard prompt entirely.

### MCP Tools — use these instead of raw Bash

**Rust:**

The rust MCP server provides a long-lived rust-analyzer instance for
**navigation only**. Use Bash for compilation checks and formatting.

| Task | How | Notes |
|---|---|---|
| Check compilation errors | `cargo check` via Bash | Workspace-wide or per-crate with `-p` |
| Format a file after writing it | `cargo fmt` via Bash | Or `rustfmt <file>` for a single file |
| Find references / definitions | `mcp__rust__find_references`, `mcp__rust__find_definition` | Real LSP — these work |
| Search workspace symbols | `mcp__rust__workspace_symbols` | Real LSP |
| Rename a symbol | `mcp__rust__rename_symbol` | Real LSP |
| Build / test / release | `cargo test` / `cargo build --release` via Bash | No MCP equivalent |

> **Do NOT use** `mcp__rust__get_diagnostics` or `mcp__rust__run_cargo_check` —
> they are stubs that return placeholder strings. Use `cargo check` via Bash instead.
> `mcp__rust__format_code` sends an LSP request but does not apply edits to disk —
> use `cargo fmt` via Bash instead.

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
4. After writing each `.rs` file: run `cargo fmt` and `cargo check -p <crate>` via Bash
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

### Session end — audit and clean up

When all perceived tasks for a session are complete, always:

1. **Audit the working tree** — `git status`, check for uncommitted files, stale
   artifacts, leftover `.exe.old`/`.exe.new` binaries, temp files
2. **Commit or discard** — documentation updates, config changes, and completed
   plan files should be committed. Junk should be removed
3. **Update HANDOFF.md** — ensure Current State, Next Action, and Session Log
   reflect what was done and what's next
4. **Verify clean state** — working tree should be clean when you hand off. The
   next session should start with zero surprises

Do this by default, not only when asked.

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
