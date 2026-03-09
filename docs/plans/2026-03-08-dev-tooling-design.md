# Dev Tooling Suite — Design

> **Status:** Approved. Ready for implementation planning.
> **Date:** 2026-03-08
> **Goal:** Eliminate manual clicking overhead in dev/test workflows. Four tools,
> priority-ordered.

---

## 1. Debug Mode & Scene Jumper (highest priority)

### CLI Flags

`cargo run --release --bin undone -- --dev` and `--dev --quick`

- `--dev` adds a "Dev" entry to `AppTab` enum, visible in the title bar. Only
  shown when the flag is present.
- `--quick` auto-creates Robin preset world, skips all creation phases, jumps
  straight to InGame.

### Dev Tab Contents

- **Scene jumper** — Searchable list of all loaded scene IDs. Click to jump.
  Resets current scene stack, starts the selected scene fresh.
- **Stat editors** — Editable fields for FEMININITY, stress, anxiety, money.
  Type a number, press Enter, value updates immediately.
- **Flag editor** — Text field to add/remove game flags. List of current flags
  with delete buttons.
- **State inspector** — Read-only display of: current scene ID, week/day/timeslot,
  arc states, NPC liking levels, active game flags.
- **Quick actions** — "Advance 1 week", "Set all NPC liking to Close", "Reset
  arc state" buttons for common test shortcuts.

### Architecture

A `dev_mode: bool` field on `GameState` (and passed down from main.rs arg parsing).
The Dev tab view reads `GameState` directly — no new signals needed beyond what the
tab switcher already provides. Scene jumping reuses `start_scene()`. Stat edits
mutate `world` directly and refresh the player snapshot signal.

### MCP Integration (file-based IPC)

When `--dev` is active, the game polls `%TEMP%/undone-dev-cmd.json` every 100ms
via `floem::action::exec_after` recurring timer. Protocol:

1. MCP tool writes command to `undone-dev-cmd.tmp`
2. Atomic rename to `undone-dev-cmd.json`
3. Game reads, deletes, executes
4. Game writes result to `undone-dev-result.json`
5. MCP tool polls for result, reads, deletes

#### Commands

| Command | Args | Description |
|---|---|---|
| `quick_start` | `preset?: "robin"\|"raul"` | Skip char creation, start game |
| `jump_to_scene` | `scene_id: string` | Jump to scene immediately |
| `set_stat` | `stat: string, value: i32` | Set stress/anxiety/femininity/money |
| `set_flag` | `flag: string` | Add a game flag |
| `remove_flag` | `flag: string` | Remove a game flag |
| `get_state` | — | Return full game state snapshot |
| `advance_time` | `weeks: u32` | Advance N weeks |
| `set_npc_liking` | `npc_name: string, level: string` | Set NPC liking level |

These tools get added to `game-input-mcp` (it already owns game interaction). The
`start_game` tool gets an optional `dev_mode: bool` parameter that appends
`-- --dev --quick` to the cargo command.

---

## 2. Stat Bounds Enforcement

### Newtypes for stress and anxiety only

Money stays as raw `i32` — the economics (debt, income, job-based modifiers) are
undesigned and need their own creative session before we can set meaningful bounds.

```rust
// In undone-domain
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundedStat(i32);

impl BoundedStat {
    pub const MIN: i32 = 0;
    pub const MAX: i32 = 100;

    pub fn new(value: i32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> i32 { self.0 }

    pub fn apply_delta(&mut self, delta: i32) {
        self.0 = (self.0 + delta).clamp(Self::MIN, Self::MAX);
    }
}
```

- `Player.stress` and `Player.anxiety` change from `i32` to `BoundedStat`
- Remove the `.max(0)` band-aids in `effects.rs`
- `ChangeStress` and `ChangeAnxiety` effects call `apply_delta()` instead of
  raw arithmetic
- Serialization: `BoundedStat` serializes as bare `i32` (transparent serde) for
  save compatibility
- The expression evaluator's `w.getStress()` / `w.getAnxiety()` return `.get()`
  — no template changes needed

### Future: Money Design (deferred)

Money needs its own creative design session covering: starting amounts by
background/job, income sources, debt mechanics, spending sinks. The current raw
`i32` is intentionally left as-is until that design exists. Do not clamp or
newtype money without first designing the economics.

---

## 3. Schedule Reachability Checker (validate-pack extension)

Static analysis pass in `validate-pack` that checks: for each triggered scene,
can the trigger condition ever become true given the effects available in the
content?

### What it checks

- For each schedule event with a `trigger` or `condition`, extract the required
  flags/liking levels/arc states
- Walk all scene effects across all loaded scenes to build a "reachable state"
  map — which flags can be set, which liking levels can be reached, which arc
  states can be advanced to
- Warn when a trigger requires a state that no effect in any scene can produce

### Example catch

Jake romance arc was blocked because `npcLiking == 'Like'` used exact equality
but liking overflowed to `Close`. The checker would see: effects can produce
`Like` AND `Close`, but the condition uses `==` not `>=`, so once liking passes
`Like` the scene becomes permanently unreachable.

### Scope

This is heuristic, not a full theorem prover. It catches:

- Flags referenced in conditions but never set by any effect
- Liking/love level exact-equality checks where effects can overshoot
- Arc states referenced in conditions but never advanced to

It does NOT attempt to reason about condition ordering, mutual exclusivity, or
temporal reachability. Those are too complex for a first pass and would produce
noisy false positives.

### Implementation

New module in `undone-scene` (e.g., `reachability.rs`) exposing a
`check_reachability()` function. Called from `validate-pack` after scenes and
schedule are loaded.

---

## 4. Scene Distribution Simulator (validate-pack extension)

CLI extension to `validate-pack` behind a `--simulate` flag. Simulates N weeks
of scheduling and reports statistics.

```
cargo run --bin validate-pack -- --simulate --weeks 52 --runs 1000
```

### Algorithm

1. Create a Robin preset world (reuse the same config as --quick)
2. For each run: simulate N weeks by calling `scheduler.pick_next()` repeatedly,
   advancing time each pick
3. Accumulate scene frequency counts across all runs

### Output

```
Scene Distribution (52 weeks × 1000 runs):
  base::coffee_shop          — 14.2% (avg 7.4/run)  ⚠ DOMINANT
  base::rain_shelter         —  8.1% (avg 4.2/run)
  base::morning_routine      —  7.9% (avg 4.1/run)
  ...
  base::stranger_approach    —  0.3% (avg 0.2/run)  ⚠ RARE
  base::library_quiet        —  0.0% (avg 0.0/run)  ⚠ NEVER FIRES

Warnings:
  - base::coffee_shop weight 10 dominates free_time (>12% share)
  - base::library_quiet never fired in 1000 runs
```

### Thresholds

- `DOMINANT`: scene takes >12% of all picks
- `RARE`: scene takes <1% of all picks
- `NEVER FIRES`: 0 picks across all runs

These are tunable constants.

### Implementation

New function in `undone-scene::scheduler` (it already has all the types). Called
from `validate-pack` when `--simulate` is present.

---

## Crate Impact Summary

| Crate | Changes |
|---|---|
| `undone-domain` | Add `BoundedStat` newtype, change `Player.stress`/`Player.anxiety` types |
| `undone-world` | Minor — accessor updates for `BoundedStat` |
| `undone-scene` | New `reachability.rs` module. Simulation function in `scheduler.rs`. Remove `.max(0)` in `effects.rs` |
| `undone-ui` | New `dev_panel.rs` view. `AppTab::Dev` variant. IPC polling loop. `--dev`/`--quick` flag plumbing |
| `src/main.rs` | CLI arg parsing, pass `dev_mode`/`quick_start` bools to UI |
| `src/bin/validate_pack.rs` | `--simulate` flag, reachability check call |
| `tools/game-input-mcp` | New dev command tools, `start_game` dev_mode param |
