# Workplace Gameplay Strengthening Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use ops-executing-plans to implement this plan task-by-task.

**Goal:** Strengthen the workplace month-one route by making explicit scenes mutate real sexual state, bringing Jake's romantic payoff into week 4, adding the missing party-social stranger payoff, and spending shallow week-one memory branches in later play.

**Architecture:** Implement this as five red-green content slices. First, write failing simulation tests for explicit-state persistence and week-4 Jake timing, then patch the schedule and explicit scenes. Second, add a failing regression for the missing party follow-up and author the new scene. Third, deepen callback consumers for arrival, landlord, lunch, and functional-clothes flags. Fourth, rebalance callback scheduling so week-2 free time blends authored memory with ordinary life-sim scenes. Finish with validation, prose audit, and route-playthrough verification.

**Tech Stack:** TOML scene content in `packs/base/scenes/`, schedule gating in `packs/base/data/schedule.toml`, Rust integration tests in `tests/`, scene simulation via `undone-scene`, prose validation through `validate-pack`, and existing prose-audit coverage.

---

### Task 1: Add failing tests for explicit-state persistence across month-one routes

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/tests/validate_pack_simulation.rs`

**Step 1: Write the failing tests**

Add simulation tests that assert explicit routes clear virginity and record sexual activity.

```rust
#[test]
fn month_one_supports_multiple_virginity_loss_routes() {
    let romantic = sim::play_route("workplace_jake_week4_first_time");
    let workplace = sim::play_route("workplace_marcus_first_time");

    assert!(!romantic.player_virgin);
    assert!(!workplace.player_virgin);
}

#[test]
fn explicit_routes_record_route_specific_sexual_activity() {
    let party = sim::play_route("workplace_party_stranger");
    assert!(party.player_sexual_activities.contains("vaginal"));
    assert!(party.flags.contains("PARTY_STRANGER_SLEPT"));
}
```

**Step 2: Run tests to verify they fail**

Run:
- `cargo test --test validate_pack_simulation month_one_supports_multiple_virginity_loss_routes -- --nocapture`
- `cargo test --test validate_pack_simulation explicit_routes_record_route_specific_sexual_activity -- --nocapture`

Expected: FAIL because current scenes do not mutate virginity/sexual-activity state and the party lane has no follow-up.

**Step 3: Commit the failing tests**

```bash
git add tests/validate_pack_simulation.rs
git commit -m "test: cover month-one sexual state persistence"
```

### Task 2: Make Jake, Marcus, and bar-stranger scenes mutate real sexual state

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/scenes/jake_apartment.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/scenes/work_marcus_closet.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/scenes/bar_stranger_night.toml`

**Step 1: Write minimal implementation**

For explicit actions that imply intercourse:
- add `set_virgin` where appropriate
- add `add_sexual_activity`
- add partner/relationship continuity where appropriate for Jake
- add route-specific flags if aftermath needs distinct routing

Do not add new systems. Use existing effect types only.

**Step 2: Run targeted tests**

Run:
- `cargo test --test validate_pack_simulation month_one_supports_multiple_virginity_loss_routes -- --nocapture`

Expected: still FAIL until Jake timing and party follow-up are addressed, but the explicit-state gap for existing lanes should be covered by scene-level assertions or partial simulation output.

**Step 3: Commit**

```bash
git add packs/base/scenes/jake_apartment.toml packs/base/scenes/work_marcus_closet.toml packs/base/scenes/bar_stranger_night.toml
git commit -m "content: persist sexual state in explicit workplace routes"
```

### Task 3: Add failing tests for Jake week-4 payoff timing

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/tests/validate_pack_simulation.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn jake_can_pay_off_explicitly_by_week_four() {
    let run = sim::play_route("workplace_jake_week4_payoff");
    assert!(run.flags.contains("JAKE_INTIMATE"));
    assert!(run.week_reached <= 4);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test validate_pack_simulation jake_can_pay_off_explicitly_by_week_four -- --nocapture`

Expected: FAIL because `jake_apartment` is still gated to week 5.

**Step 3: Commit the failing test**

```bash
git add tests/validate_pack_simulation.rs
git commit -m "test: require week-four jake payoff"
```

### Task 4: Move Jake payoff into week 4 and keep the buildup earned

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/data/schedule.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/scenes/jake_second_date.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/scenes/jake_apartment.toml`

**Step 1: Write minimal implementation**

Adjust Jake pacing so:
- `jake_first_date` remains week 3
- `jake_second_date` remains week 4
- `jake_apartment` can trigger in week 4 under the correct liking/flag conditions

If needed, sharpen `jake_second_date` so the transition into `jake_apartment` feels earned rather than rushed.

**Step 2: Run targeted tests**

Run:
- `cargo test --test validate_pack_simulation jake_can_pay_off_explicitly_by_week_four -- --nocapture`
- `cargo test --test validate_pack_simulation month_one_supports_multiple_virginity_loss_routes -- --nocapture`

Expected: PASS

**Step 3: Commit**

```bash
git add packs/base/data/schedule.toml packs/base/scenes/jake_second_date.toml packs/base/scenes/jake_apartment.toml tests/validate_pack_simulation.rs
git commit -m "content: bring jake payoff into month one"
```

### Task 5: Add failing test for the party-outside gateway payoff

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/tests/validate_pack_simulation.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn party_outside_gateway_has_real_follow_up_payoff() {
    let run = sim::play_route("workplace_party_outside_payoff");
    assert!(run.flags.contains("PARTY_STRANGER_OUTSIDE"));
    assert!(run.flags.contains("PARTY_STRANGER_SLEPT"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test validate_pack_simulation party_outside_gateway_has_real_follow_up_payoff -- --nocapture`

Expected: FAIL because no follow-up scene is currently scheduled.

**Step 3: Commit the failing test**

```bash
git add tests/validate_pack_simulation.rs
git commit -m "test: require party outside follow-up payoff"
```

### Task 6: Author the party-stranger follow-up lane

**Files:**
- Create: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/scenes/party_stranger_after.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/data/schedule.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/scenes/party_invitation.toml`

**Step 1: Write minimal implementation**

Create a distinct explicit follow-up for the party lane that uses:
- party residue
- narrow privacy
- trait-sensitive escalation
- persistent sexual state effects

Add a dedicated payoff flag such as `PARTY_STRANGER_SLEPT`.

**Step 2: Run targeted tests**

Run:
- `cargo test --test validate_pack_simulation party_outside_gateway_has_real_follow_up_payoff -- --nocapture`
- `cargo test --test validate_pack_simulation explicit_routes_record_route_specific_sexual_activity -- --nocapture`

Expected: PASS

**Step 3: Commit**

```bash
git add packs/base/scenes/party_stranger_after.toml packs/base/data/schedule.toml packs/base/scenes/party_invitation.toml tests/validate_pack_simulation.rs
git commit -m "content: add party stranger payoff route"
```

### Task 7: Add failing tests for week-one memory consumption and free-time pacing

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/tests/validate_pack_simulation.rs`
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/tests/prose_audit.rs`

**Step 1: Write the failing tests**

Add coverage that proves:
- lunch and landlord posture affect later content
- week-2 free time can interleave callbacks with ordinary life-sim scenes

```rust
#[test]
fn workplace_opening_flags_change_later_scene_selection() {
    let desk = sim::play_route("workplace_desk_lunch_branch");
    let group = sim::play_route("workplace_group_lunch_branch");
    assert_ne!(desk.scene_ids, group.scene_ids);
}

#[test]
fn week_two_free_time_is_not_only_callback_queue_drain() {
    let run = sim::play_route("workplace_week_two_free_time_mix");
    assert!(run.scene_ids.iter().any(|id| id == "base::park_walk"));
}
```

**Step 2: Run tests to verify they fail**

Run:
- `cargo test --test validate_pack_simulation workplace_opening_flags_change_later_scene_selection -- --nocapture`
- `cargo test --test validate_pack_simulation week_two_free_time_is_not_only_callback_queue_drain -- --nocapture`

Expected: FAIL while callbacks are too deterministic and some flags are not spent.

**Step 3: Commit the failing tests**

```bash
git add tests/validate_pack_simulation.rs tests/prose_audit.rs
git commit -m "test: cover workplace memory consumption and pacing"
```

### Task 8: Spend arrival, landlord, lunch, and clothes flags in live content

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/scenes/opening_callback_status_assertion.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/scenes/opening_callback_first_week_solitude.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/scenes/opening_callback_mirror_afterglow.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/scenes/opening_callback_transactional_defense.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/scenes/coffee_shop_return.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/scenes/work_lunch.toml`

**Step 1: Write minimal implementation**

Ensure at least one later scene materially changes based on each of:
- arrival posture
- landlord posture
- lunch style
- functional vs reflective clothes handling

Use the flags to alter scene selection, available actions, or substantial prose blocks, not just a single sentence.

**Step 2: Run targeted tests**

Run:
- `cargo test --test validate_pack_simulation workplace_opening_flags_change_later_scene_selection -- --nocapture`
- `cargo test --test prose_audit -- --nocapture`

Expected: PASS

**Step 3: Commit**

```bash
git add packs/base/scenes/opening_callback_status_assertion.toml packs/base/scenes/opening_callback_first_week_solitude.toml packs/base/scenes/opening_callback_mirror_afterglow.toml packs/base/scenes/opening_callback_transactional_defense.toml packs/base/scenes/coffee_shop_return.toml packs/base/scenes/work_lunch.toml tests/validate_pack_simulation.rs tests/prose_audit.rs
git commit -m "content: deepen workplace memory callbacks"
```

### Task 9: Rebalance free-time callback scheduling

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/packs/base/data/schedule.toml`

**Step 1: Write minimal implementation**

Keep only the most important callback as a hard trigger. Convert the others to weighted
conditional scenes so week-2 free time can breathe.

**Step 2: Run targeted tests**

Run:
- `cargo test --test validate_pack_simulation week_two_free_time_is_not_only_callback_queue_drain -- --nocapture`

Expected: PASS

**Step 3: Commit**

```bash
git add packs/base/data/schedule.toml tests/validate_pack_simulation.rs
git commit -m "content: smooth workplace free-time pacing"
```

### Task 10: Acceptance tests for month-one workplace fantasy coverage

**Acceptance Criteria:**
- romantic, workplace-transgressive, bar-stranger, and party-stranger lanes can all reach explicit payoff during month one
- at least two different lanes can clear virginity by week 4
- later content distinguishes some week-one identity branches
- week-2 free time still includes ordinary life-sim content

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/tests/validate_pack_simulation.rs`

**Step 1: Write acceptance tests**

```rust
#[test]
fn month_one_supports_distinct_explicit_fantasy_lanes() {
    let jake = sim::play_route("workplace_jake_week4_payoff");
    let marcus = sim::play_route("workplace_marcus_first_time");
    let bar = sim::play_route("workplace_bar_stranger");
    let party = sim::play_route("workplace_party_outside_payoff");

    assert!(jake.flags.contains("JAKE_INTIMATE"));
    assert!(marcus.flags.contains("MARCUS_INTIMATE"));
    assert!(bar.flags.contains("BAR_STRANGER_SLEPT"));
    assert!(party.flags.contains("PARTY_STRANGER_SLEPT"));
}
```

**Step 2: Run acceptance tests**

Run: `cargo test --test validate_pack_simulation month_one_supports_distinct_explicit_fantasy_lanes -- --nocapture`

Expected: PASS

**Step 3: Commit**

```bash
git add tests/validate_pack_simulation.rs
git commit -m "test: add month-one workplace fantasy acceptance coverage"
```

### Task 11: Full verification and documentation sync

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/docs/arcs/workplace-opening.md`
- Modify: `C:/Users/YJK/dev/mirror/undone/.worktrees/codex/workplace-month-one/docs/content-schema.md`

**Step 1: Sync documentation**

Update route-memory and schema docs so they reflect:
- which week-one flags are now live consumers
- which explicit routes mutate virginity/sexual activity
- new party payoff route flags

**Step 2: Run full verification**

Run:
- `cargo test --test validate_pack_simulation -- --nocapture`
- `cargo test --test prose_audit -- --nocapture`
- `cargo test -p undone-scene workplace_arc_full_playthrough -- --nocapture`
- `cargo run --bin validate-pack`

Expected: PASS, with only any pre-existing unrelated warnings called out explicitly.

**Step 3: Commit**

```bash
git add docs/arcs/workplace-opening.md docs/content-schema.md
git commit -m "docs: sync workplace gameplay strengthening contracts"
```
