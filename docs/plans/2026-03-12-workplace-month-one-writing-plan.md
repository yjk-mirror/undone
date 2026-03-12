# Workplace Month-One Writing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use ops-executing-plans to implement this plan task-by-task.

**Goal:** Deliver a functional, enriched first four weeks of the workplace route by rewriting the weak opening and spine scenes, expanding daily-life/workplace coverage, authoring multiple explicit adult route families, and closing the current multi-NPC authoring gap in the engine.

**Architecture:** Work in four layers. First, add role-addressable multi-NPC scene support while keeping `m` and `f` backward-compatible. Second, rewrite the opening and week-one workplace spine so the route reads at the approved prose standard. Third, expand repeatable daily-life and work slots so weeks 2-4 feel inhabited. Fourth, build three distinct adult ladders (Jake, Marcus, stranger/social) plus aftermath and divergence, then rebalance schedule triggers and validate month-one progression end-to-end.

**Tech Stack:** Rust workspace crates (`undone-expr`, `undone-scene`, `undone-ui`), TOML scene files in `packs/base/scenes/`, weighted/triggered schedule entries in `packs/base/data/schedule.toml`, Minijinja prose templates, Rust integration tests, `validate-pack`, and the existing prose-audit test harness.

---

### Task 1: Add failing tests for role-addressable multi-NPC scene context

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/crates/undone-expr/src/eval.rs`
- Modify: `C:/Users/YJK/dev/mirror/undone/crates/undone-scene/src/template_ctx.rs`
- Modify: `C:/Users/YJK/dev/mirror/undone/crates/undone-scene/src/engine.rs`

**Step 1: Write the failing tests**

Add tests that assert one scene can address more than the legacy `m` / `f` pair by authored role.

```rust
#[test]
fn scene_ctx_can_resolve_bound_npc_by_role_in_expression() {
    let (world, registry, ctx) = world_with_role_bound_npcs();
    let expr = parse("role('ROLE_TEAM_LEAD').hasFlag('introduced')").unwrap();
    assert!(eval(&expr, &world, &ctx, &registry).unwrap());
}

#[test]
fn render_prose_can_access_multiple_bound_npcs() {
    let (world, registry, ctx) = world_with_role_bound_npcs();
    let prose = "{{ role('ROLE_TEAM_LEAD').getName() }} and {{ role('ROLE_DESIGNER').getName() }}";
    let rendered = render_prose(prose, &world, &ctx, &registry).unwrap();
    assert!(rendered.contains("Dan"));
    assert!(rendered.contains("Mia"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p undone-expr role_bound -- --nocapture`

Expected: FAIL because role-addressable scene bindings do not exist yet.

**Step 3: Write minimal implementation**

Extend scene context so a scene can carry a small map of bound NPC roles. Keep `m` and `f` working,
but add a role lookup path usable from both expressions and prose templates.

Touch:
- `crates/undone-expr/src/eval.rs`
- `crates/undone-scene/src/template_ctx.rs`
- `crates/undone-scene/src/engine.rs`

**Step 4: Run tests to verify they pass**

Run: `cargo test -p undone-expr role_bound -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/undone-expr/src/eval.rs crates/undone-scene/src/template_ctx.rs crates/undone-scene/src/engine.rs
git commit -m "feat: add role-addressable multi-npc scene context"
```

### Task 2: Let scene effects target role-bound NPCs

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/crates/undone-scene/src/effects.rs`
- Modify: `C:/Users/YJK/dev/mirror/undone/crates/undone-scene/src/types.rs`
- Modify: `C:/Users/YJK/dev/mirror/undone/docs/content-schema.md`
- Modify: `C:/Users/YJK/dev/mirror/undone/docs/engine-contract.md`

**Step 1: Write the failing test**

Add an effect test that targets a bound role instead of only `m` / `f`.

```rust
#[test]
fn add_npc_liking_can_target_role_bound_npc() {
    let (mut world, registry, mut ctx) = world_with_role_bound_npcs_mut();
    apply_effect(
        &EffectDef::AddNpcLiking { npc: "ROLE_TEAM_LEAD".into(), delta: 1 },
        &mut world,
        &mut ctx,
        &registry,
    ).unwrap();
    assert_eq!(npc_liking_for_role(&world, "ROLE_TEAM_LEAD"), LikingLevel::Ok);
}
```

**Step 2: Run the test to verify it fails**

Run: `cargo test -p undone-scene add_npc_liking_can_target_role_bound_npc -- --nocapture`

Expected: FAIL because effects only resolve `m` and `f`.

**Step 3: Write minimal implementation**

Teach effect resolution to accept authored role ids in addition to `m` and `f`, then document the
expanded contract in the schema and engine docs.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p undone-scene add_npc_liking_can_target_role_bound_npc -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/undone-scene/src/effects.rs crates/undone-scene/src/types.rs docs/content-schema.md docs/engine-contract.md
git commit -m "feat: allow scene effects to target role-bound npcs"
```

### Task 3: Surface multiple active NPCs in runtime/UI tooling

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/crates/undone-scene/src/engine.rs`
- Modify: `C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/lib.rs`
- Modify: `C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/runtime_snapshot.rs`
- Modify: `C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/runtime_test_support.rs`

**Step 1: Write the failing test**

Add a runtime snapshot test that expects multiple active NPC summaries when a scene binds several
roles.

```rust
#[test]
fn runtime_snapshot_lists_multiple_active_npcs() {
    let snapshot = snapshot_for_multi_npc_scene();
    assert!(snapshot.active_npcs.len() >= 2);
}
```

**Step 2: Run the test to verify it fails**

Run: `cargo test -p undone-ui runtime_snapshot_lists_multiple_active_npcs -- --nocapture`

Expected: FAIL because the runtime/UI only surfaces one active NPC summary.

**Step 3: Write minimal implementation**

Update engine events and runtime snapshot plumbing so authoring and testing tools can see multiple
currently relevant NPCs. Keep the UI simple; it only needs enough visibility to support writing
and verification.

**Step 4: Run targeted tests**

Run: `cargo test -p undone-ui runtime_snapshot_lists_multiple_active_npcs -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add crates/undone-scene/src/engine.rs crates/undone-ui/src/lib.rs crates/undone-ui/src/runtime_snapshot.rs crates/undone-ui/src/runtime_test_support.rs
git commit -m "feat: expose multiple active npcs in runtime snapshot"
```

### Task 4: Acceptance tests for multi-NPC authoring support

**Acceptance Criteria:**
- a scene can bind and reference more than one authored NPC role
- prose templates can read those NPCs
- effects can target them by role
- runtime tooling exposes multiple active NPCs clearly enough to validate group scenes

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/crates/undone-expr/src/eval.rs`
- Modify: `C:/Users/YJK/dev/mirror/undone/crates/undone-scene/src/template_ctx.rs`
- Modify: `C:/Users/YJK/dev/mirror/undone/crates/undone-scene/src/effects.rs`
- Modify: `C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/runtime_snapshot.rs`

**Step 1: Write acceptance-style tests**

```rust
#[test]
fn multi_npc_scene_contract_supports_prose_effects_and_snapshot() {
    let harness = multi_npc_harness();
    assert!(harness.rendered_intro.contains("Dan"));
    assert!(harness.rendered_intro.contains("Mia"));
    assert!(harness.snapshot.active_npcs.len() >= 2);
    assert!(harness.effect_targeting_ok);
}
```

**Step 2: Run acceptance tests**

Run:
- `cargo test -p undone-expr role_bound -- --nocapture`
- `cargo test -p undone-scene role_bound -- --nocapture`
- `cargo test -p undone-ui runtime_snapshot_lists_multiple_active_npcs -- --nocapture`

Expected: ALL PASS

**Step 3: Commit**

```bash
git add crates/undone-expr/src/eval.rs crates/undone-scene/src/template_ctx.rs crates/undone-scene/src/effects.rs crates/undone-ui/src/runtime_snapshot.rs
git commit -m "test: add acceptance coverage for multi-npc authoring"
```

### Task 5: Rewrite the opening and transformation bridge

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/transformation_intro.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/char_creation.rs`
- Modify: `C:/Users/YJK/dev/mirror/undone/crates/undone-ui/src/lib.rs`
- Modify: `C:/Users/YJK/dev/mirror/undone/docs/player-experience-map.md`

**Step 1: Write the failing acceptance test**

Add a user-flow test that asserts a new game goes through a real opening bridge before ordinary
route play.

```rust
#[test]
fn new_game_workplace_route_reaches_transformation_bridge_before_arrival() {
    let snapshot = start_new_robin_game_for_test();
    assert!(snapshot.story_paragraphs.iter().any(|p| p.contains("Somewhere over Ohio")));
    assert_ne!(snapshot.current_scene_id.as_deref(), Some("base::rain_shelter"));
}
```

**Step 2: Run the test to verify it fails**

Run: `cargo test -p undone-ui new_game_workplace_route_reaches_transformation_bridge_before_arrival -- --nocapture`

Expected: FAIL while the old opening flow is still in place.

**Step 3: Rewrite the opening**

Deliver:
- stronger plane scene prose
- an actual bridge into being her, not a cold form-only void
- scheduler-first route sequencing so the workplace route starts correctly

Keep the route prose aligned with the current register and the player-experience decisions.

**Step 4: Run validation**

Run:
- `cargo test -p undone-ui new_game_workplace_route_reaches_transformation_bridge_before_arrival -- --nocapture`
- `cargo run --bin validate-pack`

Expected: PASS

**Step 5: Commit**

```bash
git add packs/base/scenes/transformation_intro.toml crates/undone-ui/src/char_creation.rs crates/undone-ui/src/lib.rs docs/player-experience-map.md
git commit -m "feat: rewrite opening and add transformation bridge flow"
```

### Task 6: Rewrite the week-one workplace spine

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/workplace_arrival.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/workplace_landlord.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/workplace_first_night.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/workplace_first_clothes.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/workplace_first_day.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/workplace_work_meeting.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/workplace_evening.toml`

**Step 1: Write the failing regression test**

Add a progression test that expects the route spine to complete cleanly from arrival through the
end of the opening arc.

```rust
#[test]
fn workplace_opening_arc_advances_to_settled_after_week_one_spine() {
    let result = simulate_workplace_opening_arc();
    assert_eq!(result.final_arc_state, "settled");
    assert!(result.visited.contains(&"base::workplace_work_meeting".to_string()));
}
```

**Step 2: Run the test to verify it fails or exposes current weaknesses**

Run: `cargo test --test validate_pack_simulation workplace_opening_arc_advances_to_settled_after_week_one_spine -- --nocapture`

Expected: FAIL or expose incorrect sequencing / weak persistence.

**Step 3: Rewrite the spine scenes**

For each listed scene:
- enforce the current narrator register
- deepen trait branching
- ensure persistent consequences are meaningful
- make the week-one route feel authored, not placeholder

Do not preserve weak prose just because it already exists.

**Step 4: Run validation**

Run:
- `cargo run --bin validate-pack`
- `cargo test --test validate_pack_simulation workplace_opening_arc_advances_to_settled_after_week_one_spine -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add packs/base/scenes/workplace_arrival.toml packs/base/scenes/workplace_landlord.toml packs/base/scenes/workplace_first_night.toml packs/base/scenes/workplace_first_clothes.toml packs/base/scenes/workplace_first_day.toml packs/base/scenes/workplace_work_meeting.toml packs/base/scenes/workplace_evening.toml
git commit -m "content: rewrite workplace opening spine for month one"
```

### Task 7: Expand the daily-life baseline for weeks 2-4

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/morning_routine.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/plan_your_day.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/bookstore.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/park_walk.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/grocery_store.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/evening_home.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/weekend_morning.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/shopping_mall.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/landlord_repair.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/laundromat_night.toml`

**Step 1: Write the failing acceptance test**

Add a simulation test that asserts the free-time pool for weeks 2-4 produces meaningful variety.

```rust
#[test]
fn workplace_month_one_free_time_pool_hits_at_least_six_distinct_scenes() {
    let result = simulate_workplace_month_one(4, 12);
    assert!(result.free_time_scene_count() >= 6);
}
```

**Step 2: Run the test to verify it fails or reveals thin coverage**

Run: `cargo test --test validate_pack_simulation workplace_month_one_free_time_pool_hits_at_least_six_distinct_scenes -- --nocapture`

Expected: FAIL or show low variety.

**Step 3: Rewrite and deepen the daily-life cluster**

Deliver:
- stronger home/body-management texture
- city-life scenes with real consequences
- non-explicit erotic ambient charge where earned
- no filler actions

**Step 4: Run validation**

Run:
- `cargo run --bin validate-pack`
- `cargo test --test validate_pack_simulation workplace_month_one_free_time_pool_hits_at_least_six_distinct_scenes -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add packs/base/scenes/morning_routine.toml packs/base/scenes/plan_your_day.toml packs/base/scenes/bookstore.toml packs/base/scenes/park_walk.toml packs/base/scenes/grocery_store.toml packs/base/scenes/evening_home.toml packs/base/scenes/weekend_morning.toml packs/base/scenes/shopping_mall.toml packs/base/scenes/landlord_repair.toml packs/base/scenes/laundromat_night.toml
git commit -m "content: deepen workplace route daily-life baseline"
```

### Task 8: Expand the general workplace lane

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_standup.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_lunch.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_late.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_corridor.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_friday.toml`

**Step 1: Write the failing acceptance test**

Add a simulation test that expects distinct work-slot coverage after the opening arc.

```rust
#[test]
fn settled_workplace_route_hits_multiple_work_slot_scenes_by_week_four() {
    let result = simulate_workplace_month_one(4, 12);
    assert!(result.work_scene_count() >= 4);
}
```

**Step 2: Run the test to verify it fails or reveals thin work coverage**

Run: `cargo test --test validate_pack_simulation settled_workplace_route_hits_multiple_work_slot_scenes_by_week_four -- --nocapture`

Expected: FAIL or show low workplace variety.

**Step 3: Rewrite the work cluster**

Use the new multi-NPC support where it improves meetings, lunches, hallway clusters, and office
social dynamics.

**Step 4: Run validation**

Run:
- `cargo run --bin validate-pack`
- `cargo test --test validate_pack_simulation settled_workplace_route_hits_multiple_work_slot_scenes_by_week_four -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add packs/base/scenes/work_standup.toml packs/base/scenes/work_lunch.toml packs/base/scenes/work_late.toml packs/base/scenes/work_corridor.toml packs/base/scenes/work_friday.toml
git commit -m "content: expand general workplace lane for month one"
```

### Task 9: Rewrite and complete the Jake romantic lane

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/coffee_shop.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/coffee_shop_return.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/jake_outside.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/jake_first_date.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/jake_second_date.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/jake_apartment.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/jake_morning_after.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/jake_text_messages.toml`

**Step 1: Write the failing acceptance test**

Add a progression test for the romantic lane.

```rust
#[test]
fn jake_lane_reaches_explicit_payoff_by_week_four() {
    let result = simulate_month_one_preferring_jake_lane();
    assert!(result.flags.contains("JAKE_INTIMATE"));
}
```

**Step 2: Run the test to verify it fails or exposes weak progression**

Run: `cargo test --test validate_pack_simulation jake_lane_reaches_explicit_payoff_by_week_four -- --nocapture`

Expected: FAIL or expose route weakness.

**Step 3: Rewrite and deepen the lane**

Requirements:
- strong romantic progression
- explicit scene that earns itself
- no generic tenderness prose
- route aftermath that changes later play

**Step 4: Run validation**

Run:
- `cargo run --bin validate-pack`
- `cargo test --test validate_pack_simulation jake_lane_reaches_explicit_payoff_by_week_four -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add packs/base/scenes/coffee_shop.toml packs/base/scenes/coffee_shop_return.toml packs/base/scenes/jake_outside.toml packs/base/scenes/jake_first_date.toml packs/base/scenes/jake_second_date.toml packs/base/scenes/jake_apartment.toml packs/base/scenes/jake_morning_after.toml packs/base/scenes/jake_text_messages.toml
git commit -m "content: rewrite and complete Jake romantic lane"
```

### Task 10: Rewrite and complete the Marcus workplace-transgression lane

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_marcus_coffee.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_marcus_favor.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_marcus_late.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_marcus_drinks.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_marcus_closet.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_marcus_aftermath.toml`

**Step 1: Write the failing acceptance test**

Add a progression test for the Marcus lane.

```rust
#[test]
fn marcus_lane_reaches_explicit_payoff_and_aftermath_by_week_four() {
    let result = simulate_month_one_preferring_marcus_lane();
    assert!(result.flags.contains("MARCUS_INTIMATE"));
    assert!(result.flags.contains("MARCUS_AFTERMATH"));
}
```

**Step 2: Run the test to verify it fails or exposes weak progression**

Run: `cargo test --test validate_pack_simulation marcus_lane_reaches_explicit_payoff_and_aftermath_by_week_four -- --nocapture`

Expected: FAIL or expose route weakness.

**Step 3: Rewrite and deepen the lane**

Requirements:
- competence-first attraction
- clear difference from Jake's emotional logic
- explicit scene that earns its risk register
- aftermath inside the office ecosystem

**Step 4: Run validation**

Run:
- `cargo run --bin validate-pack`
- `cargo test --test validate_pack_simulation marcus_lane_reaches_explicit_payoff_and_aftermath_by_week_four -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add packs/base/scenes/work_marcus_coffee.toml packs/base/scenes/work_marcus_favor.toml packs/base/scenes/work_marcus_late.toml packs/base/scenes/work_marcus_drinks.toml packs/base/scenes/work_marcus_closet.toml packs/base/scenes/work_marcus_aftermath.toml
git commit -m "content: rewrite and complete Marcus workplace lane"
```

### Task 11: Rewrite and complete the situational/social adult lane

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/neighborhood_bar.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/bar_closing_time.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/bar_stranger_night.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/party_invitation.toml`

**Step 1: Write the failing acceptance test**

Add a progression test for the situational lane.

```rust
#[test]
fn situational_lane_can_reach_explicit_payoff_by_week_four() {
    let result = simulate_month_one_preferring_stranger_lane();
    assert!(result.flags.contains("BAR_STRANGER_SLEPT") || result.flags.contains("PARTY_STRANGER_OUTSIDE"));
}
```

**Step 2: Run the test to verify it fails or exposes weak progression**

Run: `cargo test --test validate_pack_simulation situational_lane_can_reach_explicit_payoff_by_week_four -- --nocapture`

Expected: FAIL or expose route weakness.

**Step 3: Rewrite and deepen the lane**

Requirements:
- keep the content clearly consensual adult fiction
- preserve the unpredictability/loss-of-control register
- make the explicit scene distinct from both Jake and Marcus
- use multi-NPC support in the party scene instead of background fakery

**Step 4: Run validation**

Run:
- `cargo run --bin validate-pack`
- `cargo test --test validate_pack_simulation situational_lane_can_reach_explicit_payoff_by_week_four -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add packs/base/scenes/neighborhood_bar.toml packs/base/scenes/bar_closing_time.toml packs/base/scenes/bar_stranger_night.toml packs/base/scenes/party_invitation.toml
git commit -m "content: rewrite and complete situational adult lane"
```

### Task 12: Add virginity-loss and aftermath coverage across routes

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/jake_apartment.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/bar_stranger_night.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_marcus_closet.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/jake_morning_after.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/work_marcus_aftermath.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/scenes/evening_home.toml`

**Step 1: Write the failing acceptance test**

Add a test that asserts virginity-loss can occur through more than one lane and produces route-
specific aftermath.

```rust
#[test]
fn month_one_supports_multiple_virginity_loss_routes() {
    let romantic = simulate_month_one_preferring_jake_lane();
    let workplace = simulate_month_one_preferring_marcus_lane();
    assert!(!romantic.player_virgin);
    assert!(!workplace.player_virgin);
    assert_ne!(romantic.last_aftermath_scene, workplace.last_aftermath_scene);
}
```

**Step 2: Run the test to verify it fails**

Run: `cargo test --test validate_pack_simulation month_one_supports_multiple_virginity_loss_routes -- --nocapture`

Expected: FAIL while virginity-loss and aftermath are too narrow.

**Step 3: Write minimal implementation**

Use explicit scene effects, flags, and aftermath branches so multiple route families can carry
first-time-in-this-body outcomes without converging into the same scene logic.

**Step 4: Run validation**

Run:
- `cargo run --bin validate-pack`
- `cargo test --test validate_pack_simulation month_one_supports_multiple_virginity_loss_routes -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add packs/base/scenes/jake_apartment.toml packs/base/scenes/bar_stranger_night.toml packs/base/scenes/work_marcus_closet.toml packs/base/scenes/jake_morning_after.toml packs/base/scenes/work_marcus_aftermath.toml packs/base/scenes/evening_home.toml
git commit -m "content: add multi-route virginity loss and aftermath coverage"
```

### Task 13: Rebalance schedule progression for the first month

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/packs/base/data/schedule.toml`
- Modify: `C:/Users/YJK/dev/mirror/undone/docs/arcs/workplace-opening.md`
- Modify: `C:/Users/YJK/dev/mirror/undone/docs/presets/robin.md`

**Step 1: Write the failing regression test**

Add a simulation test that expects month one to expose all major lanes without deadlocking the
route into only one pool.

```rust
#[test]
fn workplace_month_one_schedule_reaches_spine_work_life_and_adult_lanes() {
    let result = simulate_workplace_month_one(4, 16);
    assert!(result.visited_route_spine);
    assert!(result.work_scene_count() >= 4);
    assert!(result.free_time_scene_count() >= 6);
    assert!(result.explicit_route_count() >= 2);
}
```

**Step 2: Run the test to verify it fails or reveals poor balance**

Run: `cargo test --test validate_pack_simulation workplace_month_one_schedule_reaches_spine_work_life_and_adult_lanes -- --nocapture`

Expected: FAIL or show skewed results.

**Step 3: Rebalance schedule entries**

Adjust:
- trigger/once-only structure for mandatory beats
- weighted repeatables for daily life and work
- gate timing for Jake, Marcus, and situational routes
- route aftermath availability

Update the docs so the route contract matches the implementation.

**Step 4: Run validation**

Run:
- `cargo run --bin validate-pack`
- `cargo test --test validate_pack_simulation workplace_month_one_schedule_reaches_spine_work_life_and_adult_lanes -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add packs/base/data/schedule.toml docs/arcs/workplace-opening.md docs/presets/robin.md
git commit -m "content: rebalance workplace month-one schedule progression"
```

### Task 14: Acceptance tests for the month-one route

**Acceptance Criteria:**
- new game flows through the strengthened opening into the workplace route
- week-one workplace spine reaches `settled`
- weeks 2-4 include real life-sim variation across free-time and work scenes
- at least two explicit lanes are reachable by week four
- multiple virginity-loss routes exist
- explicit aftermath changes later play

**Files:**
- Modify: `C:/Users/YJK/dev/mirror/undone/tests/validate_pack_simulation.rs`
- Modify: `C:/Users/YJK/dev/mirror/undone/tests/prose_audit.rs`

**Step 1: Write acceptance-style tests**

```rust
#[test]
fn workplace_month_one_is_playable_through_four_weeks() {
    let result = simulate_workplace_month_one(4, 16);
    assert_eq!(result.final_arc_state, "settled");
    assert!(result.free_time_scene_count() >= 6);
    assert!(result.work_scene_count() >= 4);
    assert!(result.explicit_route_count() >= 2);
}

#[test]
fn touched_month_one_scenes_pass_prose_audit() {
    let report = validate_repo_scenes_for_tests().expect("audit");
    assert!(report.findings_for_prefix("packs/base/scenes/workplace_").is_empty());
}
```

**Step 2: Run acceptance tests**

Run:
- `cargo test --test validate_pack_simulation -- --nocapture`
- `cargo test --test prose_audit -- --nocapture`

Expected: ALL PASS

**Step 3: Commit**

```bash
git add tests/validate_pack_simulation.rs tests/prose_audit.rs
git commit -m "test: add month-one workplace route acceptance coverage"
```

### Task 15: Full verification and repo-wide month-one check

**Files:**
- No code changes required unless verification reveals a real defect

**Step 1: Run focused verification**

Run:
- `cargo run --bin validate-pack`
- `cargo test --test validate_pack_simulation -- --nocapture`
- `cargo test --test prose_audit -- --nocapture`
- `cargo test -p undone-ui --lib`
- `cargo test -p undone-scene --lib`
- `cargo test -p undone-expr --lib`

Expected: PASS

**Step 2: Run repo-wide scans**

Run:
- `rg -n "None of this was conscious|you used to do this|more of that, please|check your phone|went exactly the way|completely fine" packs/base/scenes`
- `rg -n "heat building inside|hungrily|feminine core|bit her lip" packs/base/scenes`
- `rg -n "^She\\b|\\bShe\\b" packs/base/scenes/workplace_ packs/base/scenes/jake_ packs/base/scenes/work_ packs/base/scenes/bar_ packs/base/scenes/party_invitation.toml`

Expected:
- no banned stock phrases in the touched month-one cluster
- no third-person player narration in the touched month-one cluster
- no generic erotic filler in the touched month-one cluster

**Step 3: Commit verification-only fixes if needed**

```bash
git add -A
git commit -m "chore: finalize workplace month-one verification"
```

Use `ops-executing-plans` to implement the plan at `docs/plans/2026-03-12-workplace-month-one-writing-plan.md`
