# Playable Game Fixes — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use ops:executing-plans to implement this plan task-by-task.

**Goal:** Fix the 4 bugs preventing the game from being playable: action prose invisible on scene transitions, scheduler burying key NPC introductions, AROUSAL stat never moving, FemCreation screen having zero prose. Plus: elaborate the transformation_intro (plane scene) to better ground who the player was before everything changes.

**Architecture:** One UI fix (scene transition "Continue" state), two data fixes (scheduler triggers, arousal effects), and two content passes (FemCreation brief framing prose, transformation_intro elaboration). The discovery of the new body is NOT on the FemCreation screen — it pans out chaotically across the opening arc (workplace_arrival → landlord → first_night → first_clothes). FemCreation just needs a brief bridge. The plane scene needs more character grounding.

**Tech Stack:** Rust (floem UI), TOML scene files, minijinja templates.

---

## Task 1: Fix Action Prose Visibility on Scene Transitions

**Problem:** When a player clicks an action with `finish = true`, the action prose is appended to `signals.story`, but then `dispatch_action` immediately clears the story (`signals.story.set(String::new())`) and loads the next scene. The prose exists for zero rendered frames — the player never sees it.

**Fix:** Add an `awaiting_continue` signal. When a scene finishes, stop. Show the action prose with a "Continue" button. When the player clicks Continue, then clear and load the next scene.

**Files:**
- Modify: `crates/undone-ui/src/lib.rs:73-104` (AppSignals)
- Modify: `crates/undone-ui/src/left_panel.rs:199-242` (dispatch_action)
- Modify: `crates/undone-ui/src/left_panel.rs:244+` (story_panel action bar)

### Step 1: Add `awaiting_continue` signal to AppSignals

In `crates/undone-ui/src/lib.rs`:

```rust
// Add to AppSignals struct (after line 82):
pub awaiting_continue: RwSignal<bool>,

// Add to AppSignals::new() (after line 101):
awaiting_continue: RwSignal::new(false),
```

### Step 2: Modify dispatch_action to pause on scene finish

In `crates/undone-ui/src/left_panel.rs`, replace lines 224-241 (the `if finished { ... }` block):

```rust
    if finished {
        if signals.phase.get_untracked() == crate::AppPhase::TransformationIntro {
            // Transformation intro complete — move to female customisation.
            // (The throwaway world is discarded; FemCreation builds the real one.)
            signals.phase.set(crate::AppPhase::FemCreation);
        } else {
            // Don't auto-advance. Let the player read the action prose.
            // The action bar will show a "Continue" button.
            signals.awaiting_continue.set(true);
        }
    }
```

### Step 3: Add a `continue_to_next_scene` function

In `crates/undone-ui/src/left_panel.rs`, add after `dispatch_action`:

```rust
/// Called when the player clicks "Continue" after reading action prose.
/// Picks the next scene from the scheduler and starts it.
fn continue_to_next_scene(state: &Rc<RefCell<GameState>>, signals: AppSignals) {
    signals.awaiting_continue.set(false);

    let mut gs = state.borrow_mut();
    let GameState {
        ref mut engine,
        ref mut world,
        ref registry,
        ref scheduler,
        ref mut rng,
        femininity_id,
        ..
    } = *gs;

    if let Some(result) = scheduler.pick_next(world, registry, rng) {
        signals.story.set(String::new());
        if result.once_only {
            world
                .game_data
                .set_flag(format!("ONCE_{}", result.scene_id));
        }
        crate::start_scene(engine, world, registry, result.scene_id);
        let events = engine.drain();
        crate::process_events(events, signals, world, femininity_id);
    }
}
```

### Step 4: Show "Continue" button when awaiting_continue is true

In the action bar section of `story_panel` (find where `signals.actions` drives the button list), add a conditional branch. When `signals.awaiting_continue.get()` is true, render a single "Continue" button instead of the normal action list:

```rust
// In the action bar area of story_panel, wrap the existing action buttons in a
// conditional that checks awaiting_continue:
let state_for_continue = Rc::clone(&state);
let continue_btn = container(
    label(move || "Continue".to_string())
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.padding_vert(8.0)
                .padding_horiz(24.0)
                .border_radius(4.0)
                .background(colors.accent)
                .color(colors.page)
                .font_size(signals.prefs.get().font_size as f32)
                .cursor(floem::style::CursorStyle::Pointer)
        })
)
.on_click_stop(move |_| {
    continue_to_next_scene(&state_for_continue, signals);
})
.style(|s| s.width_full().flex_row().justify_center().padding_vert(12.0));
```

Use `dyn_container` to switch between the continue button and the normal action bar based on `signals.awaiting_continue.get()`.

### Step 5: Run `cargo fmt` and `cargo check -p undone-ui`

Run: `cargo fmt && cargo check -p undone-ui`
Expected: compiles clean.

### Step 6: Commit

```bash
git add crates/undone-ui/src/lib.rs crates/undone-ui/src/left_panel.rs
git commit -m "fix: show action prose before transitioning to next scene

Add awaiting_continue state so players can read action response
prose before the next scene loads. Previously, finish=true actions
had their prose immediately cleared by the next scene's intro."
```

---

## Task 2: Fix Scheduler Pacing for NPC Introductions

**Problem:** `coffee_shop` (weight 7) competes in a pool totaling ~130, giving it ~5.4% chance per slot. After 30 scenes of play, the playtester never met Jake. The main romantic arc is lottery-dependent.

**Fix:** Make `coffee_shop` a guaranteed trigger at week 2 (once_only). Make `neighborhood_bar` a trigger at week 3. Reduce `morning_routine` from 15→10 to flatten the rotation.

**Files:**
- Modify: `packs/base/data/schedule.toml:13-53`
- Test: `crates/undone-scene/src/scheduler.rs` (add trigger test)

### Step 1: Write failing test — coffee_shop triggers at week 2

In `crates/undone-scene/src/scheduler.rs`, add to the test module:

```rust
#[test]
fn coffee_shop_triggers_at_week_2() {
    let (registry, _resolver) = make_test_registry();
    let scheduler = registry.scheduler();
    let mut world = make_world(&registry);

    // Set flags for workplace route (Robin preset)
    world.game_data.set_flag("ROUTE_WORKPLACE".into());

    // Advance to week 2
    for _ in 0..56 { // 2 weeks × 28 slots
        world.game_data.advance_time_slot();
    }
    assert_eq!(world.game_data.week, 2);

    // coffee_shop should trigger (not random)
    let mut rng = rand::thread_rng();
    let result = scheduler.pick_next(&world, &registry, &mut rng);
    assert!(result.is_some(), "expected a scene to be picked");
    // The trigger should fire reliably — run 10 times to confirm determinism
    // (If it were weighted random, some runs would pick other scenes)
}
```

Note: this test structure depends on the existing test helpers (`make_test_registry`, `make_world`). Check the existing test module for the exact patterns and adapt.

### Step 2: Run test to verify it fails

Run: `cargo test -p undone-scene coffee_shop_triggers -- --nocapture`
Expected: FAIL (coffee_shop is currently weighted random, not a trigger)

### Step 3: Modify schedule.toml

In `packs/base/data/schedule.toml`:

**Change coffee_shop (lines 18-22) from weighted to trigger:**

```toml
  [[slot.events]]
  scene     = "base::coffee_shop"
  weight    = 0
  trigger   = "gd.week() >= 2 && !gd.hasGameFlag('ONCE_base::coffee_shop')"
  once_only = true
```

**Change neighborhood_bar (lines 50-53) from weighted to trigger:**

```toml
  [[slot.events]]
  scene     = "base::neighborhood_bar"
  weight    = 0
  trigger   = "gd.week() >= 3 && !gd.hasGameFlag('ONCE_base::neighborhood_bar')"
  once_only = true
```

**Reduce morning_routine weight (line 16) from 15 to 10:**

```toml
  weight    = 10
```

### Step 4: Run test to verify it passes

Run: `cargo test -p undone-scene coffee_shop_triggers -- --nocapture`
Expected: PASS

### Step 5: Run full test suite

Run: `cargo test --workspace`
Expected: all tests pass (no regressions)

### Step 6: Commit

```bash
git add packs/base/data/schedule.toml crates/undone-scene/src/scheduler.rs
git commit -m "fix: guarantee Jake/bar introductions via scheduler triggers

coffee_shop now triggers at week 2 (was random weight 7).
neighborhood_bar triggers at week 3 (was random weight 7).
morning_routine reduced from weight 15 to 10 to flatten rotation.
Players will always meet Jake by week 2."
```

---

## Task 3: Add AROUSAL Effects to Scenes

**Problem:** The `add_arousal` effect type is fully implemented (5-level enum: Discomfort → Comfort → Enjoy → Close → Orgasm, `step_arousal()` function, template exposure via `w.getArousal()`). Zero scenes use it. AROUSAL stays at Comfort forever.

**Fix:** Add `add_arousal` effects to scenes with charged/sexual content. The effect syntax is:

```toml
[[actions.effects]]
type  = "add_arousal"
delta = 1
```

Delta values: +1 steps up one level (Comfort→Enjoy), +2 steps two (Comfort→Close), -1 steps down. Clamped to valid range.

**Files:**
- Modify: `packs/base/scenes/jake_apartment.toml`
- Modify: `packs/base/scenes/work_marcus_closet.toml`
- Modify: `packs/base/scenes/bar_stranger_night.toml`
- Modify: `packs/base/scenes/jake_second_date.toml`
- Modify: `packs/base/scenes/work_marcus_drinks.toml`
- Modify: `packs/base/scenes/bar_closing_time.toml`
- Modify: `packs/base/scenes/jake_first_date.toml`
- Modify: `packs/base/scenes/weekend_morning.toml`
- Modify: `packs/base/scenes/morning_routine.toml`

### Step 1: Add arousal to explicit scenes (delta +2 or +3)

**jake_apartment.toml** — both sexual action paths:

Add to `pull_him_close` action's effects:
```toml
  [[actions.effects]]
  type  = "add_arousal"
  delta = 3
```

Add to `let_him_lead` action's effects (if it exists):
```toml
  [[actions.effects]]
  type  = "add_arousal"
  delta = 3
```

Add to any "stop" / "not tonight" action's effects:
```toml
  [[actions.effects]]
  type  = "add_arousal"
  delta = 1
```

**work_marcus_closet.toml** — the conference room scene:

Add to the sexual action's effects:
```toml
  [[actions.effects]]
  type  = "add_arousal"
  delta = 2
```

Add to the "walk away" / decline action's effects:
```toml
  [[actions.effects]]
  type  = "add_arousal"
  delta = 1
```

**bar_stranger_night.toml** — both sexual actions:

Add to `pull_him_close` effects:
```toml
  [[actions.effects]]
  type  = "add_arousal"
  delta = 3
```

Add to `wait_let_him_move` effects:
```toml
  [[actions.effects]]
  type  = "add_arousal"
  delta = 3
```

Add to `change_mind` effects:
```toml
  [[actions.effects]]
  type  = "add_arousal"
  delta = 1
```

### Step 2: Add arousal to charged scenes (delta +1)

**jake_second_date.toml** — first kiss action:
```toml
  [[actions.effects]]
  type  = "add_arousal"
  delta = 1
```

**work_marcus_drinks.toml** — drinks escalation, the "stay" action:
```toml
  [[actions.effects]]
  type  = "add_arousal"
  delta = 1
```

**bar_closing_time.toml** — `slow_down_let_him_catch` and `invite_him_in`:
```toml
  [[actions.effects]]
  type  = "add_arousal"
  delta = 1
```

**jake_first_date.toml** — flirtation actions:
```toml
  [[actions.effects]]
  type  = "add_arousal"
  delta = 1
```

### Step 3: Add arousal to body-awareness scenes (conditional, delta +1)

**weekend_morning.toml** — private body register actions that involve body awareness:
```toml
  [[actions.effects]]
  type  = "add_arousal"
  delta = 1
```

**morning_routine.toml** — ONLY on appearance/body-related actions (if any exist), NOT on the default "get dressed and go" path.

### Step 4: Run validate-pack

Run: `cargo run --bin validate-pack 2>&1`
Expected: no new errors. The `add_arousal` effect is already a valid effect type.

### Step 5: Run workspace tests

Run: `cargo test --workspace`
Expected: all pass.

### Step 6: Commit

```bash
git add packs/base/scenes/jake_apartment.toml packs/base/scenes/work_marcus_closet.toml packs/base/scenes/bar_stranger_night.toml packs/base/scenes/jake_second_date.toml packs/base/scenes/work_marcus_drinks.toml packs/base/scenes/bar_closing_time.toml packs/base/scenes/jake_first_date.toml packs/base/scenes/weekend_morning.toml packs/base/scenes/morning_routine.toml
git commit -m "feat: wire up AROUSAL effects in charged and explicit scenes

Add add_arousal effects to 9 scenes. Explicit scenes (jake_apartment,
marcus_closet, bar_stranger_night) use delta +2/+3. Charged scenes
(dates, drinks, closing time) use delta +1. Body-awareness scenes
(weekend_morning) use conditional delta +1. AROUSAL now actually moves
during gameplay."
```

---

## Task 4: Add Brief Framing Prose to FemCreation Screen

**Problem:** "Who Are You Now?" is a pure data form with zero narrative text. It doesn't need the full body discovery (that's the opening arc's job — arrival, landlord, first night, first clothes). It just needs a brief bridge: you fell asleep as one person, here's who you are now.

**The discovery pans out across the opening arc scenes:**
- `workplace_arrival` — overhead bin height, ID mismatch, cold hands
- `workplace_landlord` — the photo that doesn't match, the key in your hand
- `workplace_first_night` — empty apartment, the bra problem, reflection in dark glass
- `workplace_first_clothes` — fitting room, the mirror, "something tightens and settles and — *oh*"

Those scenes already deliver the chaotic physical discovery. FemCreation is a calm moment between "eyes close over Ohio" and "seat belt sign clicks off." A brief framing — not a sensory inventory.

**Files:**
- Modify: `crates/undone-ui/src/char_creation.rs:596-789` (fem_creation_view)

### Step 1: Add a brief opening prose constant and helper

```rust
const FEM_FRAMING_PROSE: &str = "\
Somewhere between Ohio and here, everything changed. You don't remember it. \
You just woke up and the body was different — the weight, the proportions, \
the face in the airplane bathroom mirror. You're still you. The rest is new.";

fn prose_block(text: &'static str, signals: AppSignals) -> impl View {
    label(move || text.to_string()).style(move |s| {
        let prefs = signals.prefs.get();
        let colors = ThemeColors::from_mode(prefs.mode);
        s.width_full()
            .padding_vert(16.0)
            .padding_horiz(4.0)
            .color(colors.muted)
            .font_size(prefs.font_size as f32 * 0.95)
            .line_height(1.6)
            .font_style(floem::style::FontStyle::Italic)
    })
}
```

### Step 2: Insert prose into fem_creation_view

In `fem_creation_view`, add the prose block between the heading and the first form section (lines 764-771):

```rust
    let content = v_stack((
        heading("Who Are You Now?", signals),
        prose_block(FEM_FRAMING_PROSE, signals),
        names_section,
        body_section,
        background_section,
        begin_btn,
        empty().style(|s| s.height(40.0)),
    ))
```

The tuple arity may not support 8 children. If it doesn't compile, nest into two v_stacks (upper/lower) and combine them.

### Step 3: Run `cargo fmt` and `cargo check -p undone-ui`

Run: `cargo fmt && cargo check -p undone-ui`
Expected: compiles clean.

### Step 4: Commit

```bash
git add crates/undone-ui/src/char_creation.rs
git commit -m "feat: add brief framing prose to FemCreation screen

'Who Are You Now?' now has a short narrative bridge between the
plane scene and the form fields. The real body discovery plays out
across the opening arc scenes — this just sets the frame."
```

---

## Task 5: Elaborate Transformation Intro (Plane Scene)

**Problem:** The plane scene grounds the player in logistics (job, apartment, boxes) but not enough in who this person IS. The creative direction says it should "reflect your background — who you are, why you're going to this city, what you're leaving behind." Right now it's mostly practical. It needs more character.

**What exists:** `packs/base/scenes/transformation_intro.toml` — Gate C31, route-specific logistics, trait-specific gate behavior, jet bridge, safety demo, eyes close over Ohio. Two prose blocks (intro + action).

**What it needs:** More of the person before the transformation. Not more logistics — more character. What are they leaving behind? What do they expect? What's the last thing they think about as a man? The creative direction says this is the last moment of the old identity. It should feel like one.

**Files:**
- Modify: `packs/base/scenes/transformation_intro.toml`

### Step 1: Expand the intro prose

The current intro has route-specific logistics and a few trait branches. Expand with more character grounding. Add after the route-specific block and before the trait-specific block:

For ROUTE_WORKPLACE, add a beat about the career identity being left behind — ten years of experience, the reputation, the name that meant something. The man on the boarding pass had a career. He had a voice people recognized in meetings. He had a life that made sense.

For ROUTE_CAMPUS, add a beat about the identity being stepped into — the acceptance letter, the person who got in, the eighteen-year-old certainty that the world would make room.

Add a universal beat (not route-gated) about the gate area — the last moments as this person. People around you doing normal airport things. You're doing a normal airport thing. Everything is still normal.

### Step 2: Expand the action prose (boarding + falling asleep)

The current action has the jet bridge, safety demo, and eyes closing. Add between the safety demo and falling asleep:

A beat of the old self settling in. The last conscious thoughts as this person. The in-flight magazine. The phone switching to airplane mode. The ordinary machinery of being who you are, one last time.

Trait-specific dozing beats:
- ANALYTICAL: running through the week's logistics, the numbers making sense, the familiar architecture of a mind that works in systems
- AMBITIOUS: already rehearsing Monday, the handshake, the name introduction
- SHY: the relief of the window seat, nobody expecting anything
- OUTGOING: the seatmate conversation that fizzles into comfortable silence

The last line stays: "Somewhere over Ohio, your eyes close." That's the pivot. Everything before it is who you were.

### Step 3: Validate template

Run: Use `mcp__minijinja__jinja_validate_template` on the modified file.
Expected: valid template, no syntax errors.

### Step 4: Run validate-pack

Run: `cargo run --bin validate-pack 2>&1`
Expected: no errors.

### Step 5: Commit

```bash
git add packs/base/scenes/transformation_intro.toml
git commit -m "feat: elaborate plane scene with more character grounding

The transformation intro now has more of who the player was before
everything changes — career identity, what they're leaving behind,
the last ordinary thoughts. The logistics are still there but the
person is more present."
```

---

## Task 6: Runtime Verification

**Acceptance criteria — every one must be verified by launching the game and playing:**

1. **Action prose visible:** Click any action that ends a scene (e.g. "Walk faster" in bar_closing_time). The action's response prose should be visible with a "Continue" button. Clicking Continue loads the next scene.
2. **Transformation intro → FemCreation:** Complete character creation as Robin. The plane scene should still transition to FemCreation correctly (no Continue button — this is a phase change, not a scene transition). The plane scene should feel more grounded in who Robin was.
3. **FemCreation prose:** The "Who Are You Now?" screen should show a brief framing paragraph above the form. Readable, not overwrought.
4. **Opening arc flow:** After FemCreation, the arc should flow: arrival (overhead bin, ID check) → landlord (photo mismatch) → first night (bra problem, reflection) → first clothes (fitting room). Each scene should deliver a distinct discovery beat. Verify the action prose is visible at each transition.
5. **Scheduler pacing:** Start a new Robin game. By week 2, coffee_shop should have triggered (Jake introduction). Verify by checking if the MET_JAKE flag is set (visible in dev panel with `--dev`).
6. **AROUSAL moves:** Play through to jake_apartment or bar_stranger_night. After the explicit scene, AROUSAL should show something other than "Comfort" in the stats sidebar.
7. **No regressions:** Mid-scene actions (that don't finish the scene) should still work normally — no Continue button between actions within the same scene.

### Step 1: Build release

Run: `cargo build --release --bin undone`
Expected: compiles.

### Step 2: Launch and play through full Robin path

Use the `playtester` agent or manual play. Go through:
- Landing → New Game → Robin preset → Plane scene → FemCreation → Workplace arrival
- Verify the opening arc discovery flow (arrival → landlord → first night → first clothes → first day)
- Play through at least 2 in-game weeks into the settled state
- Verify each acceptance criterion above
- Screenshot key moments

### Step 3: Fix any issues found

If issues are found during runtime testing, fix them before claiming done.

### Step 4: Final commit (if any fixes)

```bash
git add -A
git commit -m "fix: runtime fixes from playtest verification"
```

---

## Execution

Use `ops:executing-plans` to implement the plan at `docs/plans/2026-03-09-playable-game-fixes.md`
