# Story-Map Tool — Design

> **Status:** Approved design (2026-06-02). Next step: implementation plan via `ops:writing-plans`.
> **Author:** Opus session (brainstorming pass).

## 1. Problem

The base pack has ~74 scenes across several storylines (workplace opening, Jake romance,
Marcus affair, Cal/gym, Theo/campus, the desire/looping-adult layer, and ambient
life scenes). Writers — both the human director and the `scene-writer` agents — have
**no map**. There is no single trustworthy answer to:

- What scenes exist, and how do they connect?
- Which branches are reachable, and which are broken?
- **What should be written next?**

Connectivity is currently implicit in two places: scene `.toml` effects/conditions and
`schedule.toml` gates. `validate-pack` already computes *reachability warnings* from this
data (`crates/undone-scene/src/reachability.rs`), but it reports pass/fail diagnostics, not
a navigable story map, and it knows nothing about *intended* (not-yet-written) content.

## 2. Goal

A **`story-map`** tool that:

1. **Derives** the scene-connectivity graph from pack data — never rots, always accurate.
2. **Reconciles** it against a lightweight **authored roadmap** that captures intent the
   code cannot express (planned threads, intended-but-unwritten scenes).
3. Emits a **human Markdown report** + an **agent-readable JSON sidecar**.

It is **advisory cartography, not a validator** — it never blocks a build. Broken-gate
*reachability* warnings stay in `validate-pack`. `story-map` answers "what's the shape of
the story and what's missing," which is a different job (Engineering Principle 6 —
separation of concerns).

This aligns with the project's anti-rot stance: the connectivity half is *generated* from
live data, and the authored half is deliberately minimal and continuously reconciled
against reality (Engineering Principle 10 — docs track implementation).

## 3. Data sources

### 3a. Derived facts (from pack data, per scene)

Extracted from every non-archived scene `.toml` and `schedule.toml`. Reuses the existing
source-scan helpers in `crates/undone-scene/src/script/validate.rs`
(`source_set_game_flags`, `source_advance_arcs`) and the edge-finders in
`reachability.rs` (`find_hasgameflag`, `find_eq_call`) — **no new graph-walking code from
scratch.**

| Fact | Source |
|---|---|
| **Sets** (outgoing edges) | flags via `source_set_game_flags`, arcs via `source_advance_arcs`, over all `actions[].effect` and `npc_actions[].effect` |
| **Requires** (incoming edges) | flags/arcs referenced in the scene's schedule binding (`trigger` + `condition`) and in `actions[].condition` |
| **Goto edges** | `actions[].next[].goto` targets |
| **Binding** | slot name, `weight`, `trigger`/`condition`, `once_only`, `npc_role`, `desire_scaled` |
| **Status** | `reachable` / `broken-gate` (requires a flag/arc nothing sets) / `unbound` (no schedule entry **and** no inbound `goto`) |

Negated gates (`!gd.hasGameFlag("X")`) are *not* treated as a requirement — absence is the
default state — matching the existing `reachability.rs` convention.

### 3b. Authored roadmap (`packs/base/roadmap.toml`)

A **new authoring-only file**. The engine never loads it at runtime; only `story-map`
reads it. It lives in the pack directory so it travels with the pack (platform principle —
each pack owns its own story metadata). Schema:

```toml
[[thread]]
name        = "Marcus affair"
flag_prefix = "MARCUS_"            # claims scenes whose set/required flags share this prefix
scenes      = ["marcus_repeat_office", "marcus_pushes", "marcus_leverage"]  # explicit members (for prefix-less scenes)
planned     = ["marcus_reconcile", "marcus_public"]   # intended, not yet written
note        = "Recurring affair -> leverage cost -> cooling or continuation."
```

- A scene is claimed by a thread if it matches the thread's `flag_prefix` **or** appears in
  the thread's explicit `scenes` list.
- `planned` lists intended scene ids that do not yet exist as `.toml` files.
- `note` is free text for the writer's intent.
- Any **existing** scene claimed by **no** thread is reported as an **orphan** (loud), so
  nothing silently falls out of the map.

## 4. Reconciliation — findings

| Finding | Definition | Writer value |
|---|---|---|
| **Thread chains** | Per thread, its scenes ordered by flag-dependency (a scene requiring `FLAG` sorts after the scene that sets it) | The "tree" — the spine of each storyline |
| **Dangling threads** ⚠ | A flag is *set* by some scene but *required* by no gate/condition anywhere | Open door nobody walks through → prime "write-next" hook |
| **Broken gates** ⚠ | A flag/arc is *required* but *set* by nothing | Scene exists but unreachable — write the producer or fix the gate |
| **Orphan scenes** ⚠ | An existing scene claimed by no roadmap thread | Keeps the map honest |
| **Roadmap drift** ⚠ | A `planned` scene that now exists (promote it) or one that still doesn't; thread prefix matching no scenes | Keeps intent and reality in sync |

**"Write Next" digest** — a ranked list assembled from the above, in priority order:

1. **Dangling flags** — organic continuations (the story already opened the door).
2. **Broken gates** — reachability holes (a written scene nobody can reach).
3. **Roadmap `planned` TODOs** — intended, unwritten.
4. **Thin endings** — a thread's terminal scene sets a flag with no consumer (the story
   trails off rather than resolving).

## 5. Outputs

Both regenerated, committed, and diff-able.

### 5a. `docs/story-map.md` (human surface)

- **Top:** the "Write Next" digest.
- **Body:** one section per thread — the scene chain with `← requires` / `→ sets`
  annotations, `(once)` / `(repeatable)` markers, slot binding, and the thread's ⚠ gaps
  inline.
- **Footer:** orphan scenes + global roadmap drift.

### 5b. `docs/story-map.json` (agent API)

Stable schema for `scene-writer` consumption:

```json
{
  "threads": [
    {
      "name": "Marcus affair",
      "scenes": [
        { "id": "marcus_leverage", "sets": ["MARCUS_LEVERAGE", "MARCUS_TERMS_HERS"],
          "requires": ["MARCUS_INTIMATE"], "status": "reachable",
          "binding": { "slot": "free_time", "once_only": true, "npc_role": "ROLE_MARCUS" } }
      ],
      "dangling": ["MARCUS_AFFAIR_COOLING"],
      "broken": [],
      "planned": ["marcus_reconcile"]
    }
  ],
  "orphans": ["laundromat_night"],
  "drift": [{ "kind": "planned_now_exists", "thread": "Cal", "scene": "gym_regular_first" }],
  "write_next": [
    { "priority": 1, "kind": "dangling", "thread": "Marcus affair", "flag": "MARCUS_AFFAIR_COOLING",
      "set_by": "marcus_leverage", "hint": "no scene consumes this — candidate follow-up" }
  ]
}
```

### 5c. `--check` mode

Like `cargo fmt --check`: regenerate in-memory and exit non-zero if the committed
`docs/story-map.{md,json}` are stale versus the current pack. Lets a hook/CI keep the map
fresh **without** turning content gaps into build failures.

## 6. Where it lives & how it runs

- **New binary `story-map`** in the root game workspace, alongside `validate-pack`:
  - `src/bin/story_map.rs` — thin CLI entry point.
  - `src/story_map.rs` — the library module with all logic (mirrors the
    `src/validate_pack.rs` / `src/bin/validate_pack.rs` split, so the logic is
    unit-testable without spawning a process).
- Reuses the existing pack-load path and `undone-scene::reachability` helpers.

```sh
cargo run --bin story-map            # regenerate docs/story-map.{md,json}
cargo run --bin story-map -- --check # CI/hook: fail if the committed map is stale
```

*Why a separate binary, not a `validate-pack` subcommand:* `validate-pack` is a pass/fail
gate (CI-blocking). `story-map` is advisory cartography that writes files. Separate
binaries keep each contract clean (Principle 6).

## 7. Workflow integration (light — no new enforcement)

- The **`scene-writer` agent brief** (`.claude/agents/scene-writer.md`) gains a step: read
  `docs/story-map.json` `write_next` to ground new scenes in real dangling threads.
- **`HANDOFF.md` / project `CLAUDE.md`** get a one-line pointer: regenerate the map after
  content changes.
- A `--check` post-edit hook is **out of scope** here — flagged as a follow-up.

## 8. Testing

- **Unit tests** (library module, `reachability.rs` style, synthetic mini-packs):
  - scene sets a flag nothing reads → appears in `dangling`.
  - gate on an unset flag → `broken`.
  - scene in no thread → `orphan`.
  - `planned` entry that now exists → `drift` (`planned_now_exists`).
  - thread ordering: a scene requiring `FLAG` sorts after the scene that sets it.
  - Each with a `// BREAKS IF` comment naming the user-visible behavior it guards.
- **Acceptance test** against the **real base pack**:
  - tool runs clean.
  - the scene partition is **total** — every non-archived scene lands in exactly one thread
    **or** the orphan list.
  - JSON sidecar round-trips through its schema.
- **`--check` test**: regenerate, mutate a byte, assert non-zero exit.
- The committed **`packs/base/roadmap.toml`** is authored as part of this work, declaring
  the real threads (opening, Jake, Marcus, Cal/gym, Theo/campus, desire/looping, ambient).
  It is itself verified by the orphan-partition acceptance test.

## 9. Non-goals / YAGNI

- No visual graph rendering (Mermaid/DOT) — 74 nodes across threads turns into spaghetti;
  the threaded Markdown report is the readable form.
- No engine/runtime consumption of the roadmap — it is authoring metadata only.
- No build-blocking on content gaps — advisory only; `--check` only guards map *staleness*.
- No multi-pack aggregation yet — base pack only (the design is pack-scoped and extends
  cleanly when a second pack exists).

## 10. Docs to update on implementation (Principle 10)

- `docs/content-schema.md` — note the `roadmap.toml` authoring file and the `story-map`
  tool.
- `HANDOFF.md` — regenerate-the-map pointer + session log entry.
- `.claude/agents/scene-writer.md` — the `write_next` grounding step.
