# Phase 0 — Adult-Content Pacing Fix Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use ops:executing-plans to implement this plan task-by-task.

**Goal:** Make explicit content reachable on the default Robin/workplace route by ~in-game week 1–2 instead of week 4–6, by removing redundant `week >= N` calendar floors from the romance/explicit scene gates while preserving narrative order and opening-arc precedence.

**Architecture:** Pure `schedule.toml` data change plus one scheduler integration test. No engine code changes. The scheduler (`pick_next`, scheduler.rs:306) fires `trigger` events first (alphabetical by slot, first match wins) before any weighted selection; the romance payoff scenes are already `weight=0 + trigger`, so they fire deterministically the moment their trigger is true. The only thing delaying them is `week >= N` inside those triggers stacked on top of an already-sufficient story-flag chain. We drop the calendar floors and re-anchor the entry gate on arc state.

**Tech Stack:** Rust (cargo), TOML, the `validate-pack` binary, the `playtester` agent (dev-IPC + screenshots).

**Parent design:** `docs/plans/2026-05-29-rhai-fragment-architecture-design.md` (§7 Phase 0).

---

## Background the engineer must understand before editing

Read `crates/undone-scene/src/scheduler.rs:306-404` (`pick_next`). Two facts govern this whole change:

1. **Triggers fire before the weighted pool, and across slots in ALPHABETICAL slot-name order.**
   Slots are `campus_opening`, `free_time`, `work`, `workplace_opening`. Alphabetically,
   `free_time` < `workplace_opening`. So if a `free_time` trigger and a `workplace_opening`
   trigger are both active in the same tick, **the `free_time` one wins.**

2. **Consequence / landmine:** `base::coffee_shop` lives in `free_time` and is `weight=0 + trigger`.
   Today its trigger carries `gd.week() >= 2`, which is what keeps it from being active during
   the opening arc (weeks 0–1). If you drop that floor to "always," coffee_shop would fire
   **before `workplace_arrival`** and break the opening. Therefore coffee_shop's gate must be
   re-anchored on `gd.arcState('base::workplace_opening') == 'settled'` (calendar-free, but still
   strictly after the opening arc completes) — NOT removed outright.

3. **Downstream romance triggers are safe to de-floor.** `jake_first_date`, `jake_second_date`,
   `jake_apartment` are gated on the story-flag chain (`MET_JAKE` → `JAKE_FIRST_DATE` →
   `JAKE_SECOND_DATE`), which can only be satisfied in order and only after coffee_shop has fired.
   Their `week >= N` floors are pure redundant delay and can be dropped with no ordering risk.

The current explicit-content minimum path on the Robin route:
`coffee_shop` (week≥2) → build ROLE_JAKE liking to `Like` → `jake_first_date` (week≥3) →
`jake_second_date` (week≥4) → `jake_apartment` (week≥4, first explicit scene). That staircase
is what produces "week 4–6 before any payoff."

## File map

- **Modify:** `packs/base/data/schedule.toml` — the `free_time` slot events (Tasks 1–3).
- **Create/Modify test:** `crates/undone-scene/src/scheduler.rs` (test module at the bottom) —
  one integration test asserting opening-arc precedence AND calendar-free reachability (Task 4).
- **Verify:** `cargo run --bin validate-pack` (Task 5) and a `playtester` acceptance run (Task 6).

No new files. No engine logic changes.

---

## Task 1: Re-anchor the romance ENTRY gates on arc-state (ordering-safe)

**Files:**
- Modify: `packs/base/data/schedule.toml` (the `free_time` slot, `coffee_shop` event ~lines 20-24)

These two changes remove calendar floors from the *entry points* while preserving opening-arc
precedence by gating on `settled`.

**Step 1: Edit `coffee_shop`**

Find this event in the `free_time` slot:

```toml
  [[slot.events]]
  scene     = "base::coffee_shop"
  weight    = 0
  trigger   = "gd.week() >= 2 && !gd.hasGameFlag('ONCE_base::coffee_shop')"
  once_only = true
```

Replace the `trigger` line with:

```toml
  trigger   = "gd.arcState('base::workplace_opening') == 'settled' && !gd.hasGameFlag('ONCE_base::coffee_shop')"
```

Rationale: `settled` is only reached after the opening arc's 6–7 scenes complete, so coffee_shop
still cannot pre-empt `workplace_arrival` (the alphabetical-slot landmine), but it no longer waits
for calendar week 2 — a player who completes the opening arc quickly reaches Jake immediately.

**Step 2: Verify the file still parses (cheap early check)**

Run: `cargo run --bin validate-pack 2>&1 | tail -20`
Expected: no new errors; the pre-existing non-blocking warnings only (e.g. `work_marcus_drinks`
line 55 "check your phone"). If you see `BadCondition`/parse errors mentioning coffee_shop, you
mistyped the expression — fix before continuing.

**Step 3: Commit**

```bash
git add packs/base/data/schedule.toml
git commit -m "fix(pacing): gate coffee_shop on arc-settled instead of week>=2"
```

---

## Task 2: Drop redundant week floors on the Jake liking-builders and payoff chain

**Files:**
- Modify: `packs/base/data/schedule.toml` (`free_time` slot: `coffee_shop_return`, `jake_outside`,
  `jake_first_date`, `jake_second_date`, `jake_apartment`)

Each edit removes only the `gd.week() >= N &&` prefix; every story-flag prerequisite stays.

**Step 1: Edit the five events**

`coffee_shop_return` — change:
```toml
  condition = "gd.week() >= 2 && gd.hasGameFlag('MET_JAKE')"
```
to:
```toml
  condition = "gd.hasGameFlag('MET_JAKE')"
```

`jake_outside` — change:
```toml
  condition = "gd.week() >= 3 && gd.hasGameFlag('MET_JAKE')"
```
to:
```toml
  condition = "gd.hasGameFlag('MET_JAKE')"
```

`jake_first_date` — change:
```toml
  trigger   = "gd.week() >= 3 && gd.hasGameFlag('MET_JAKE') && !gd.hasGameFlag('JAKE_FIRST_DATE') && gd.npcLikingAtLeast('ROLE_JAKE', 'Like')"
```
to:
```toml
  trigger   = "gd.hasGameFlag('MET_JAKE') && !gd.hasGameFlag('JAKE_FIRST_DATE') && gd.npcLikingAtLeast('ROLE_JAKE', 'Like')"
```

`jake_second_date` — change:
```toml
  trigger   = "gd.week() >= 4 && gd.hasGameFlag('JAKE_FIRST_DATE') && !gd.hasGameFlag('JAKE_SECOND_DATE')"
```
to:
```toml
  trigger   = "gd.hasGameFlag('JAKE_FIRST_DATE') && !gd.hasGameFlag('JAKE_SECOND_DATE')"
```

`jake_apartment` — change:
```toml
  trigger   = "gd.week() >= 4 && gd.hasGameFlag('JAKE_SECOND_DATE') && !gd.hasGameFlag('JAKE_INTIMATE')"
```
to:
```toml
  trigger   = "gd.hasGameFlag('JAKE_SECOND_DATE') && !gd.hasGameFlag('JAKE_INTIMATE')"
```

**Step 2: Validate**

Run: `cargo run --bin validate-pack 2>&1 | tail -20`
Expected: no new errors (only the pre-existing warnings).

**Step 3: Commit**

```bash
git add packs/base/data/schedule.toml
git commit -m "fix(pacing): drop redundant week floors on Jake chain (flag chain enforces order)"
```

---

## Task 3: Open a fast independent explicit on-ramp (bar stranger)

**Files:**
- Modify: `packs/base/data/schedule.toml` (`free_time` slot: `bar_closing_time`)

`bar_closing_time` is the one explicit on-ramp that is `weight`-based (competes in the weighted
pool), so unlike the Jake triggers it CAN be diluted by ~12 SFW universals. Re-anchor it on
`settled` (calendar-free, ordering-safe) and raise its weight so it surfaces as a real option.
`bar_stranger_night` (its trigger-gated payoff) needs no change — it already has no week floor.

**Step 1: Edit `bar_closing_time`**

Change:
```toml
  [[slot.events]]
  scene     = "base::bar_closing_time"
  weight    = 6
  condition = "gd.week() >= 3 && !gd.hasGameFlag('BAR_STRANGER_INVITED')"
```
to:
```toml
  [[slot.events]]
  scene     = "base::bar_closing_time"
  weight    = 12
  condition = "gd.arcState('base::workplace_opening') == 'settled' && !gd.hasGameFlag('BAR_STRANGER_INVITED')"
```

Note: this makes `bar_closing_time` effectively workplace-route (it now keys off the workplace arc
state). That is acceptable — it is a city/nightlife scene on Robin's route. Campus pacing is out of
scope for Phase 0.

**Step 2: Validate**

Run: `cargo run --bin validate-pack 2>&1 | tail -20`
Expected: no new errors.

**Step 3: Commit**

```bash
git add packs/base/data/schedule.toml
git commit -m "fix(pacing): bar stranger on-ramp gates on settled + higher weight"
```

---

## Task 4: Integration test — opening-arc precedence AND calendar-free reachability

This is the regression guard. It proves (a) the ordering landmine is not tripped (coffee_shop does
not pre-empt the opening arc), and (b) the Jake payoff is reachable without advancing the calendar.

**Files:**
- Modify: `crates/undone-scene/src/scheduler.rs` (add tests to the existing `#[cfg(test)] mod tests`)

First read the existing test module (search `fn pick_next_returns_originating_slot_metadata`) to
copy the established construction pattern for a `Scheduler`, `World`, `PackRegistry`, and seeded
`Rng` used in that file. Use the SAME helpers those tests use (do not invent new ones). The two
tests below describe the assertions; fill the setup using the file's existing pattern.

**Step 1: Write the failing tests**

```rust
#[test]
fn opening_arc_fires_before_romance_when_not_settled() {
    // Workplace route, fresh game: arc state is "arrived"/unset, NOT "settled".
    // coffee_shop must NOT be selected; the opening arc trigger must win.
    let (scheduler, registry) = load_base_schedule_and_registry(); // existing helper pattern
    let mut world = new_workplace_world();                          // ROUTE_WORKPLACE, week 0, no arc flags
    let mut rng = seeded_rng();

    let pick = scheduler.pick_next(&world, &registry, &mut rng).expect("a pick");
    assert_ne!(pick.scene_id, "base::coffee_shop",
        "coffee_shop must not pre-empt the opening arc before settled");
    assert_eq!(pick.scene_id, "base::workplace_arrival",
        "the opening-arc trigger should win at game start");
}

#[test]
fn jake_apartment_reachable_without_advancing_weeks() {
    // Settled arc + full Jake flag chain set, week left at 1 (NOT >=4).
    // jake_apartment's trigger must fire — proving the week>=4 floor no longer blocks it.
    let (scheduler, registry) = load_base_schedule_and_registry();
    let mut world = new_workplace_world();
    set_arc_state(&mut world, "base::workplace_opening", "settled");
    set_flag(&mut world, "MET_JAKE");
    set_flag(&mut world, "JAKE_FIRST_DATE");
    set_flag(&mut world, "JAKE_SECOND_DATE");
    // deliberately do NOT advance week past 1
    let mut rng = seeded_rng();

    let pick = scheduler.pick_next(&world, &registry, &mut rng).expect("a pick");
    assert_eq!(pick.scene_id, "base::jake_apartment",
        "jake_apartment trigger should fire on the flag chain alone, no calendar gate");
}
```

If a helper like `set_arc_state` / `set_flag` / `new_workplace_world` does not already exist in the
test module, write a tiny local helper in the test module (not in production code) using
`world.game_data` mutators that the other tests already use. Keep it minimal.

**Step 2: Run to verify they fail (before the TOML edits would have failed; after Tasks 1–3 the
second should pass, the first should pass too). Run them now to confirm GREEN post-edit:**

Run: `cargo test -p undone-scene opening_arc_fires_before_romance_when_not_settled jake_apartment_reachable_without_advancing_weeks -- --nocapture`
Expected: BOTH PASS. If `opening_arc_fires_before_romance...` fails with `coffee_shop`, the Task 1
re-anchor is wrong (coffee_shop is pre-empting the arc) — revisit the trigger expression.

**Step 3: Commit**

```bash
git add crates/undone-scene/src/scheduler.rs
git commit -m "test(pacing): guard opening-arc precedence + calendar-free Jake reachability"
```

---

## Task 5: Full validation gate

**Step 1: Run the whole suite + pack validation**

Run: `cargo test -p undone-scene 2>&1 | tail -20`
Expected: all pass (count should be ≥ prior count + 2).

Run: `cargo run --bin validate-pack 2>&1 | tail -30`
Expected: loads clean; only the documented pre-existing non-blocking warnings. No `BadCondition`,
no duplicate-id, no unknown-flag/reachability regressions on the edited scenes.

**Step 2: Commit any incidental fixes** (only if validate-pack surfaced something you had to fix).

---

## Task 6: Acceptance — playtester measures time-to-first-explicit

**Acceptance Criteria:**
- New game (Robin/workplace) → the opening arc still fires first and in the correct order
  (`workplace_arrival` → … → arc reaches `settled`); coffee_shop does NOT appear during the arc.
- After `settled`, the Jake chain is reachable with NO calendar waiting: building ROLE_JAKE liking
  to `Like`, then `jake_first_date` → `jake_second_date` → `jake_apartment` fire on consecutive
  eligible free-time picks, gated only by the flag chain.
- First explicit scene (`jake_apartment` or `bar_stranger_night`) is reachable by ~in-game week 1–2,
  measured in in-game days, down from the prior week 4–6.
- No scene fires out of narrative order; no NPC-role binding breaks (ROLE_JAKE bound via coffee_shop).

**Files:** none (runtime verification via the `playtester` agent and dev IPC).

**Step 1: Build the release binary**

Run: `cargo build --release --bin undone 2>&1 | tail -5`
Expected: `Finished release`. (Confirm `target/release/undone.exe` exists — do NOT trust a piped
exit code; check the binary timestamp.)

**Step 2: Dispatch the `playtester` agent** with this brief:

> Robin/workplace route. (1) New game, play the opening arc naturally; record each scene id in
> order and confirm `coffee_shop` does NOT appear until the arc reaches `settled`. (2) Once settled,
> drive toward Jake: use dev IPC (`set_npc_liking ROLE_JAKE Like`, `choose_action`, `continue_scene`,
> `advance_time` only as needed) to reach `jake_first_date` → `jake_second_date` → `jake_apartment`.
> Record how many in-game DAYS elapse from game start to the first explicit scene rendering. (3) Also
> confirm `bar_closing_time` now surfaces in free time after settled. Report the day-count to first
> explicit content and any out-of-order firing or broken NPC binding. Screenshot the opening arc
> order and the first explicit scene.

**Step 2 (verification, not a checkbox):** Read the playtester report adversarially. The pass bar is
"first explicit content reachable by ~week 1–2 AND opening arc order intact." If coffee_shop appears
mid-opening-arc, or jake_apartment still needs week≥4, the fix regressed — do not claim success.

**Step 3: Record the result** in HANDOFF.md (Current State + Session Log) with the measured
before/after day-count. Commit:

```bash
git add HANDOFF.md
git commit -m "docs: HANDOFF — Phase 0 pacing fix, measured time-to-first-explicit"
```

---

## Self-review (completed by plan author)

- **Spec coverage:** Implements design §7 Phase 0 (drop `week>=N` floors, keep flag prereqs, ship
  standalone) and resolves open question O5 (triggers fire before weighted pool, alphabetical by
  slot) — the resolution is encoded in Task 1's ordering-safe re-anchor and Task 4's precedence test.
- **Out of scope (intentionally):** Marcus chain (already calendar-free; gated on arc+liking),
  campus route pacing, party route (secondary — can be a follow-up edit if the user wants it), any
  engine/Rhai change.
- **Placeholder scan:** All edits give exact before/after TOML. The only deferred specifics are the
  test-module setup helpers, which must mirror the file's EXISTING test pattern (explicitly
  instructed to read and copy, not invent) because their exact names live in code the executor will
  read — this is a faithful-to-codebase instruction, not a placeholder for behavior.
- **Consistency:** Flag names (`MET_JAKE`, `JAKE_FIRST_DATE`, `JAKE_SECOND_DATE`, `JAKE_INTIMATE`,
  `BAR_STRANGER_INVITED`) and the arc id (`base::workplace_opening`) / state (`settled`) match
  schedule.toml verbatim.

## Notes for execution

- Run in a dedicated worktree per project convention:
  `git worktree add ~/.config/ops/worktrees/undone/phase0-pacing -b phase0-pacing`
- This phase ships independently; it does NOT depend on any Rhai/fragment work and should merge to
  `master` on its own once Task 6 passes.
- Phases 1–4 are separate plans. Phase 1 (Rhai foundation) is authored next; Phase 2 (vertical
  slice) is blocked on a creative scene spec from the user.
