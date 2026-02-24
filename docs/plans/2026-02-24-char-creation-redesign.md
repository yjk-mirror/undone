# Character Creation Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the all-in-one stat-picker with a 3-phase narrative flow: create your male past → play a short transformation intro scene → choose your new self.

**Architecture:** The new flow splits `AppPhase::CharCreation` into three phases: `BeforeCreation` (who you were), `TransformationIntro` (the transformation scene, transformed paths only), and `FemCreation` (who you are now). A `PartialCharState` signal accumulates choices across phases. The transformation intro runs as a real scene against a throwaway world (no NPCs, discarded after). The real world is created at FemCreation submit via the existing `new_game()`.

**Tech Stack:** Rust, floem (reactive signals), existing SceneEngine/scheduler machinery, TOML scenes, `undone-packs::new_game`, `undone-ui::process_events`.

---

## Reference: Current Flow

```
AppPhase::CharCreation → big form (all fields) → BeginButton → new_game() → AppPhase::InGame
```

## New Flow

```
AppPhase::BeforeCreation   →  form: origin + (if transformed) before-name/age/race/sexuality
                               + personality traits + content prefs → "Next"
                               if AlwaysFemale → skip to FemCreation
                               if transformed  → create throwaway world, start transformation_intro scene

AppPhase::TransformationIntro  →  story panel renders transformation_intro scene
                                   when scene finishes → phase = FemCreation

AppPhase::FemCreation      →  form: feminine name, androgynous name, figure, breasts, race
                               "Begin Your Story" → assemble CharCreationConfig → new_game() → InGame

AppPhase::InGame           →  unchanged (opening scene fires from registry as before)
```

For **AlwaysFemale**: `BeforeCreation` → `FemCreation` → `InGame` (no transformation scene).

---

## Key Files

| File | Role |
|---|---|
| `crates/undone-packs/src/manifest.rs` | Add `transformation_scene` field to `PackMeta` |
| `crates/undone-packs/src/registry.rs` | Add field + getter + setter |
| `crates/undone-packs/src/loader.rs` | Wire `transformation_scene` like `opening_scene` |
| `crates/undone-ui/src/lib.rs` | New `AppPhase` variants, `PartialCharState`, updated `app_view` dyn_container |
| `crates/undone-ui/src/char_creation.rs` | Replaced with `before_creation_view()` and `fem_creation_view()` |
| `crates/undone-ui/src/left_panel.rs` | Add phase-check when `SceneFinished` during `TransformationIntro` |
| `packs/base/scenes/transformation_intro.toml` | New scene (create) |
| `packs/base/pack.toml` | Add `transformation_scene = "base::transformation_intro"` |

---

## Data Structures

### `PartialCharState` (new, lives in `undone-ui/src/lib.rs`)

```rust
/// Accumulated choices from BeforeCreation, used to assemble CharCreationConfig at FemCreation submit.
#[derive(Clone)]
pub struct PartialCharState {
    pub origin: PcOrigin,
    /// The player's masculine/before name. Used as `name_masc` and `before.name`.
    pub before_name: String,
    pub before_age: undone_domain::Age,
    pub before_race: String,
    pub before_sexuality: undone_domain::BeforeSexuality,
    /// Resolved TraitIds for personality traits (SHY, AMBITIOUS, etc.) + content flags.
    pub starting_traits: Vec<undone_domain::TraitId>,
}
```

### `FemFormData` (local to `fem_creation_view` call site, no need for a named struct)

Fields collected: `name_fem: String`, `name_androg: String`, `race: String`, `figure: PlayerFigure`, `breasts: BreastSize`.

### `AppPhase` (modify existing in `undone-ui/src/lib.rs`)

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppPhase {
    BeforeCreation,       // replaces CharCreation
    TransformationIntro,  // transformed paths only
    FemCreation,          // who you are now
    InGame,
}
```

---

### Task 1: Add `transformation_scene` to manifest, registry, loader

**Files:**
- Modify: `crates/undone-packs/src/manifest.rs`
- Modify: `crates/undone-packs/src/registry.rs`
- Modify: `crates/undone-packs/src/loader.rs`
- Test: `crates/undone-packs/src/loader.rs` (in existing tests module)

**Step 1: Add field to `PackMeta` in `manifest.rs`**

In `PackMeta`, after `default_slot`:
```rust
#[serde(default)]
pub transformation_scene: Option<String>,
```

**Step 2: Add field + getter + setter to `PackRegistry` in `registry.rs`**

Add to struct:
```rust
transformation_scene: Option<String>,
```

Add to `PackRegistry::new()`:
```rust
transformation_scene: None,
```

Add methods (alongside `opening_scene` / `default_slot` methods):
```rust
pub fn transformation_scene(&self) -> Option<&str> {
    self.transformation_scene.as_deref()
}
pub fn set_transformation_scene(&mut self, id: String) {
    self.transformation_scene = Some(id);
}
```

**Step 3: Wire in `loader.rs`**

After the `default_slot` wiring block:
```rust
if let Some(ref scene) = manifest.pack.transformation_scene {
    registry.set_transformation_scene(scene.clone());
}
```

**Step 4: Write test**

In `loader.rs` tests (after `base_pack_has_default_slot`):
```rust
#[test]
fn base_pack_has_transformation_scene() {
    let (registry, _) = load_packs(&packs_dir()).unwrap();
    // Will return None until pack.toml is updated in Task 9.
    // This test documents the expectation — update assertion in Task 9.
    let _ = registry.transformation_scene(); // just verify no panic
}
```

**Step 5: Run tests**
```
cargo test -p undone-packs
```
Expected: all existing tests pass (transformation_scene returns None since pack.toml not yet updated).

**Step 6: Commit**
```bash
git add crates/undone-packs/src/manifest.rs crates/undone-packs/src/registry.rs crates/undone-packs/src/loader.rs
git commit -m "feat(packs): transformation_scene field in manifest, registry, loader"
```

---

### Task 2: Add new `AppPhase` variants and `PartialCharState` to `lib.rs`

**Files:**
- Modify: `crates/undone-ui/src/lib.rs`

**Step 1: Replace `AppPhase` enum**

Find:
```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppPhase {
    CharCreation,
    InGame,
}
```

Replace with:
```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppPhase {
    BeforeCreation,
    TransformationIntro,
    FemCreation,
    InGame,
}
```

**Step 2: Add `PartialCharState` struct**

After the `AppPhase` enum:
```rust
/// Accumulated choices from BeforeCreation, passed forward to FemCreation.
#[derive(Clone)]
pub struct PartialCharState {
    pub origin: undone_domain::PcOrigin,
    pub before_name: String,
    pub before_age: undone_domain::Age,
    pub before_race: String,
    pub before_sexuality: undone_domain::BeforeSexuality,
    pub starting_traits: Vec<undone_domain::TraitId>,
}
```

**Step 3: Update `AppSignals::new()` initial phase**

Change:
```rust
phase: RwSignal::new(AppPhase::CharCreation),
```
To:
```rust
phase: RwSignal::new(AppPhase::BeforeCreation),
```

**Step 4: Add `partial_char` state to `app_view()`**

In `app_view()`, after `let game_state: Rc<RefCell<Option<GameState>>> = ...`:
```rust
let partial_char: RwSignal<Option<PartialCharState>> = RwSignal::new(None);
```

**Step 5: Run cargo check**
```
cargo check -p undone-ui
```
Expected: errors for the missing `CharCreation` arm in `dyn_container` — that's correct. Fix the `dyn_container` match arm in the next step.

**Step 6: In `app_view` dyn_container, replace `CharCreation` arm with stub stubs**

The `dyn_container` currently matches `AppPhase::CharCreation` and `AppPhase::InGame`. Replace the entire match with:
```rust
move |current_phase| match current_phase {
    AppPhase::BeforeCreation => {
        char_creation_view(signals, Rc::clone(&pre_state_cc), Rc::clone(&game_state_cc), partial_char)
            .into_any()
    }
    AppPhase::TransformationIntro => {
        // TODO Task 5: wire transformation intro scene
        placeholder_panel("Transformation intro — coming soon", signals).into_any()
    }
    AppPhase::FemCreation => {
        // TODO Task 6: wire fem creation form
        placeholder_panel("Fem creation — coming soon", signals).into_any()
    }
    AppPhase::InGame => {
        // ... existing InGame arm, unchanged ...
    }
}
```

**Step 7: Run cargo check**
```
cargo check -p undone-ui
```
Expected: errors about `char_creation_view` signature mismatch (it now needs `partial_char` param). Fix in Task 3.

**Step 8: Commit**
```bash
git add crates/undone-ui/src/lib.rs
git commit -m "feat(ui): new AppPhase variants (BeforeCreation/TransformationIntro/FemCreation), PartialCharState"
```

---

### Task 3: `before_creation_view()` — the BeforeCreation form

**Files:**
- Modify: `crates/undone-ui/src/char_creation.rs` (replace current contents)

**Overview:** The new form collects: origin (radio: CisMale / TransWoman / CisFemale / AlwaysFemale), before-name + before-age + before-sexuality (only for transformed origins), personality traits checkboxes, content preference checkboxes. A "Next" button that stores choices into `partial_char` and transitions the phase.

**Step 1: Rewrite `char_creation_view` as `before_creation_view`**

The function signature changes to:
```rust
pub fn char_creation_view(
    signals: AppSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
    partial_char: RwSignal<Option<PartialCharState>>,
) -> impl View
```

(Keep the same function name `char_creation_view` for now to avoid updating the call site — it's already plumbed in Task 2.)

**Step 2: Rewrite form signals**

Replace `CharFormSignals` with a slimmed version:

```rust
#[derive(Clone, Copy)]
struct BeforeFormSignals {
    origin_idx: RwSignal<u8>,  // 0=CisMale, 1=TransWoman, 2=CisFemale, 3=AlwaysFemale
    before_name: RwSignal<String>,
    before_age: RwSignal<Age>,
    before_sexuality: RwSignal<BeforeSexuality>,
    before_race: RwSignal<String>,
    // personality traits (same as before)
    trait_shy: RwSignal<bool>,
    trait_cute: RwSignal<bool>,
    trait_posh: RwSignal<bool>,
    trait_sultry: RwSignal<bool>,
    trait_down_to_earth: RwSignal<bool>,
    trait_bitchy: RwSignal<bool>,
    trait_refined: RwSignal<bool>,
    trait_romantic: RwSignal<bool>,
    trait_flirty: RwSignal<bool>,
    trait_ambitious: RwSignal<bool>,
    trait_beautiful: RwSignal<bool>,
    trait_plain: RwSignal<bool>,
    // content prefs
    include_rough: RwSignal<bool>,
    likes_rough: RwSignal<bool>,
}
```

Default values:
- `origin_idx`: 0 (CisMale)
- `before_name`: "Evan"
- `before_age`: Age::Twenties
- `before_sexuality`: BeforeSexuality::AttractedToWomen
- `before_race`: first race from pack registry

**Step 3: Implement origin mapping helper**

```rust
fn origin_from_idx(idx: u8) -> PcOrigin {
    match idx {
        0 => PcOrigin::CisMaleTransformed,
        1 => PcOrigin::TransWomanTransformed,
        2 => PcOrigin::CisFemaleTransformed,
        _ => PcOrigin::AlwaysFemale,
    }
}
```

**Step 4: Section layout**

The form has three sections (same visual style as existing, reuse helpers):

1. **"Your Past"** — origin radio buttons:
   - `(●) Something happened to me — I was a man`
   - `(○) Something happened to me — I was a trans woman`
   - `(○) Something happened to me — I was a woman`
   - `(○) I was always a woman`

2. **"Before" fields** — `dyn_container(origin_idx, ...)` — only shown for transformed origins:
   - Before name (text input)
   - Age before (Dropdown of Age variants)
   - Before sexuality (Dropdown, only for CisMale and TransWoman)
   - Race before (race_picker — reuse existing helper)

3. **"Personality"** — identical to existing `section_personality`, reuse the function

4. **"Content Preferences"** — identical to existing `section_content_prefs`, reuse the function

**Step 5: Implement the "Next" button**

```rust
fn build_next_button(
    signals: AppSignals,
    form: BeforeFormSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
    partial_char: RwSignal<Option<PartialCharState>>,
) -> impl View {
    label(|| "Next →".to_string())
        .on_click_stop(move |_| {
            let pre = match pre_state.borrow().as_ref() {
                Some(p) => p,   // we borrow, not take — pre_state persists through all phases
                None => return,
            };
            let origin = origin_from_idx(form.origin_idx.get_untracked());

            // Resolve starting traits
            let mut trait_names: Vec<&'static str> = Vec::new();
            // ... same trait collection logic as existing build_begin_button ...
            // (SHY, CUTE, POSH, SULTRY, DOWN_TO_EARTH, BITCHY, REFINED, ROMANTIC,
            //  FLIRTY, AMBITIOUS, BEAUTIFUL, PLAIN, BLOCK_ROUGH, LIKES_ROUGH)
            let starting_traits: Vec<_> = trait_names
                .iter()
                .filter_map(|name| pre.registry.resolve_trait(name).ok())
                .collect();

            let partial = PartialCharState {
                origin,
                before_name: form.before_name.get_untracked(),
                before_age: form.before_age.get_untracked(),
                before_race: form.before_race.get_untracked(),
                before_sexuality: form.before_sexuality.get_untracked(),
                starting_traits,
            };
            partial_char.set(Some(partial.clone()));

            if origin == PcOrigin::AlwaysFemale {
                // Skip transformation intro — go straight to fem creation
                signals.phase.set(AppPhase::FemCreation);
            } else {
                // Create throwaway world for transformation intro scene
                let pre_ref = pre_state.borrow();
                let pre = pre_ref.as_ref().unwrap();
                // Build a minimal world from before-data (0 NPCs — this world is discarded)
                let before_identity = if origin.has_before_life() {
                    Some(undone_domain::BeforeIdentity {
                        name: partial.before_name.clone(),
                        age: partial.before_age,
                        race: partial.before_race.clone(),
                        sexuality: partial.before_sexuality,
                        figure: undone_domain::MaleFigure::Average,
                        traits: std::collections::HashSet::new(),
                    })
                } else { None };
                let throwaway_config = undone_packs::char_creation::CharCreationConfig {
                    name_fem: String::new(),
                    name_androg: String::new(),
                    name_masc: partial.before_name.clone(),
                    age: partial.before_age,
                    race: partial.before_race.clone(),
                    figure: undone_domain::PlayerFigure::Slim,
                    breasts: undone_domain::BreastSize::MediumLarge,
                    origin,
                    before: before_identity,
                    starting_traits: partial.starting_traits.clone(),
                    male_count: 0,    // no NPC spawn for throwaway
                    female_count: 0,
                    starting_flags: std::collections::HashSet::new(),
                    starting_arc_states: std::collections::HashMap::new(),
                };
                // Need mutable access to pre_state for new_game — use a separate borrow scope
                // Note: borrow complexity here. See implementation note below.
                drop(pre_ref);
                let mut pre_mut = pre_state.borrow_mut();
                if let Some(ref mut pre) = *pre_mut {
                    let throwaway_world = undone_packs::char_creation::new_game(
                        throwaway_config,
                        &mut pre.registry,
                        &mut pre.rng,
                    );
                    let engine = undone_scene::engine::SceneEngine::new(pre.scenes.clone());
                    let throwaway_gs = crate::game_state::GameState {
                        world: throwaway_world,
                        registry: pre.registry.clone(), // NOTE: see impl note
                        engine,
                        scheduler: pre.scheduler.clone(),
                        rng: rand::SeedableRng::from_entropy(),
                        init_error: None,
                        opening_scene: pre.registry.opening_scene().map(|s| s.to_owned()),
                        default_slot: pre.registry.default_slot().map(|s| s.to_owned()),
                    };
                    *game_state.borrow_mut() = Some(throwaway_gs);
                }
                signals.phase.set(AppPhase::TransformationIntro);
            }
        })
        // ... style same as existing "Begin Your Story" button ...
}
```

> **Implementation note on registry clone:** `PackRegistry` needs to implement `Clone` for the throwaway GameState. Check if it already derives Clone. If not, add `#[derive(Clone)]` to `PackRegistry` — this is safe since Rodeo is Clone. If Clone is too expensive (unlikely), store an `Arc<PackRegistry>` in GameState instead. Prefer the simpler `#[derive(Clone)]` approach.

> **Implementation note on Scheduler clone:** `Scheduler` needs `Clone`. Check if it derives Clone already. If not, add it.

**Step 6: Assemble the final view**

```rust
pub fn char_creation_view(...) -> impl View {
    let form = BeforeFormSignals::new(...);
    // ... races list from pre_state ...
    let next_btn = build_next_button(signals, form, pre_state, game_state, partial_char);
    let content = v_stack((
        heading("Your Story Begins", signals),
        section_your_past(signals, form, races),
        section_personality(signals, form),
        section_content_prefs(signals, form),
        next_btn,
        empty().style(|s| s.height(40.0)),
    ))
    // ... same width/padding/color style as before ...

    // scroll wrapper — same as before
}
```

**Step 7: Run cargo check**
```
cargo check -p undone-ui
```
Fix any type errors. Then:
```
cargo test --workspace
```
Expected: 197 tests pass.

**Step 8: Commit**
```bash
git add crates/undone-ui/src/char_creation.rs crates/undone-ui/src/lib.rs
git commit -m "feat(ui): before_creation_view — step 1 form (origin, before data, traits)"
```

---

### Task 4: Check if `PackRegistry` and `Scheduler` derive `Clone`; add if missing

**Files:**
- `crates/undone-packs/src/registry.rs`
- `crates/undone-scene/src/scheduler.rs` (check)

**Step 1: Check PackRegistry**

Read `registry.rs`. If `PackRegistry` does not `#[derive(Clone)]`, add it. `Rodeo` implements `Clone`. All HashMap/Vec fields are Clone. This should compile cleanly.

**Step 2: Check Scheduler**

Read `scheduler.rs`. Check if `Scheduler` derives `Clone`. If not, add it. The scheduler holds parsed TOML schedule data — all fields should be Clone (Vec<SlotDef> etc.).

**Step 3: Run cargo check**
```
cargo check --workspace
```

**Step 4: Commit if changes were made**
```bash
git add crates/undone-packs/src/registry.rs crates/undone-scene/src/scheduler.rs
git commit -m "feat: derive Clone for PackRegistry and Scheduler (needed for throwaway world)"
```

---

### Task 5: Wire `TransformationIntro` in `app_view`

**Files:**
- Modify: `crates/undone-ui/src/lib.rs`

Replace the `TransformationIntro` placeholder stub in the `dyn_container` with real wiring.

**Step 1: Replace the TransformationIntro arm**

The TransformationIntro arm needs to:
1. Take the throwaway GameState from `game_state`
2. Start the transformation_intro scene from the registry
3. Render the same UI as InGame (sidebar_panel + story_panel)

```rust
AppPhase::TransformationIntro => {
    let gs_ref = Rc::clone(&game_state_ig);
    {
        let mut gs_opt = gs_ref.borrow_mut();
        if let Some(ref mut gs) = *gs_opt {
            if let Ok(fem_id) = gs.registry.resolve_skill("FEMININITY") {
                let GameState {
                    ref mut engine,
                    ref mut world,
                    ref registry,
                    ..
                } = *gs;
                // Start the transformation intro scene
                if let Some(scene_id) = registry.transformation_scene() {
                    engine.send(
                        EngineCommand::StartScene(scene_id.to_owned()),
                        world,
                        registry,
                    );
                    let events = engine.drain();
                    process_events(events, signals, world, fem_id);
                }
            }
        }
    }
    // Extract GameState into Rc<RefCell<GameState>> for story_panel
    let inner_gs: GameState = match gs_ref.borrow_mut().take() {
        Some(gs) => gs,
        None => return placeholder_panel("Transformation intro: game state missing", signals).into_any(),
    };
    let gs_cell: Rc<RefCell<GameState>> = Rc::new(RefCell::new(inner_gs));

    h_stack((
        sidebar_panel(signals),
        story_panel(signals, Rc::clone(&gs_cell)),
    ))
    .style(|s| s.size_full())
    .into_any()
}
```

Note: `game_state_ig` already exists as `Rc::clone(&game_state)` — reuse it. The same `game_state_ig` variable serves both `TransformationIntro` and `InGame` arms.

**Step 2: Run cargo check**
```
cargo check -p undone-ui
```

**Step 3: Commit**
```bash
git add crates/undone-ui/src/lib.rs
git commit -m "feat(ui): wire TransformationIntro phase — start transformation_intro scene"
```

---

### Task 6: Handle scene-finish in `TransformationIntro` — transition to `FemCreation`

**Files:**
- Modify: `crates/undone-ui/src/left_panel.rs`

Currently when a scene finishes, `left_panel.rs:199-205` picks the next scheduler slot. During `TransformationIntro`, it should instead transition to `FemCreation`.

**Step 1: Find the scene-finish handler in `left_panel.rs`**

Look for:
```rust
let finished = crate::process_events(events, signals, world, femininity_id);
if finished {
    if let Some(slot) = default_slot.as_deref() {
        if let Some(result) = scheduler.pick(slot, world, registry, rng) {
```

**Step 2: Add phase check**

Replace the `if finished` block with:
```rust
let finished = crate::process_events(events, signals, world, femininity_id);
if finished {
    if signals.phase.get() == crate::AppPhase::TransformationIntro {
        // Transformation intro complete — move to female customization
        signals.phase.set(crate::AppPhase::FemCreation);
    } else if let Some(slot) = default_slot.as_deref() {
        if let Some(result) = scheduler.pick(slot, world, registry, rng) {
            engine.send(EngineCommand::StartScene(result.scene_id), world, registry);
            let events = engine.drain();
            crate::process_events(events, signals, world, femininity_id);
        }
    }
}
```

**Step 3: Run cargo check**
```
cargo check -p undone-ui
```

**Step 4: Manual test** (build and run)
```
cargo run
```
At this point, `BeforeCreation` form → "Next" → `TransformationIntro` (shows story panel with "Transformation intro — coming soon" from placeholder, or nothing if transformation_scene not yet in pack.toml) → FemCreation placeholder.

**Step 5: Commit**
```bash
git add crates/undone-ui/src/left_panel.rs
git commit -m "feat(ui): scene-finish handler — TransformationIntro transitions to FemCreation"
```

---

### Task 7: `fem_creation_view()` — the FemCreation form

**Files:**
- Modify: `crates/undone-ui/src/char_creation.rs` (add new function)
- Modify: `crates/undone-ui/src/lib.rs` (wire the FemCreation arm)

**Step 1: Add `fem_creation_view` function to `char_creation.rs`**

```rust
pub fn fem_creation_view(
    signals: AppSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
    partial_char: RwSignal<Option<PartialCharState>>,
) -> impl View
```

**Step 2: Form signals (local)**

```rust
let name_fem = RwSignal::new("Eva".to_string());
let name_androg = RwSignal::new("Ev".to_string());
let figure = RwSignal::new(PlayerFigure::Slim);
let breasts = RwSignal::new(BreastSize::MediumLarge);
let race = RwSignal::new(first_race_from_pre_state);
// For AlwaysFemale, also show age:
let age = RwSignal::new(Age::EarlyTwenties);
```

**Step 3: Read origin from `partial_char` to decide what to show**

```rust
let is_always_female = partial_char.get_untracked()
    .map(|p| p.origin == PcOrigin::AlwaysFemale)
    .unwrap_or(false);
```

Show "Age" dropdown only if `is_always_female` (transformed PCs inherit `before_age`).

**Step 4: Section layout**

```
heading: "Who Are You Now?"

section "Your Name"
  - "Feminine name"   [text input]  — used when FEMININITY >= 70
  - "Androgynous name" [text input] — used when FEMININITY 31–69

section "Your Body"
  - "Figure"   [Dropdown: Slim / Toned / Womanly]
  - "Breasts"  [Dropdown: Small / Medium-Small / Medium-Large / Large]

section "Background"
  - "Race"     [race_picker]
  - if is_always_female: "Age" [Dropdown]

[Begin Your Story] button
```

**Step 5: Implement the "Begin Your Story" button**

```rust
fn build_begin_button_fem(
    signals: AppSignals,
    name_fem: RwSignal<String>,
    name_androg: RwSignal<String>,
    age: RwSignal<Age>,
    figure: RwSignal<PlayerFigure>,
    breasts: RwSignal<BreastSize>,
    race: RwSignal<String>,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
    partial_char: RwSignal<Option<PartialCharState>>,
) -> impl View {
    label(|| "Begin Your Story".to_string())
        .on_click_stop(move |_| {
            let pre = pre_state.borrow_mut().take();
            let pre = match pre { Some(p) => p, None => return };

            let partial = match partial_char.get_untracked() {
                Some(p) => p,
                None => {
                    // AlwaysFemale path: partial may not be set, use default
                    PartialCharState {
                        origin: PcOrigin::AlwaysFemale,
                        before_name: String::new(),
                        before_age: age.get_untracked(),
                        before_race: race.get_untracked(),
                        before_sexuality: undone_domain::BeforeSexuality::AttractedToWomen,
                        starting_traits: vec![],
                    }
                }
            };

            let origin = partial.origin;
            let fem_race = race.get_untracked();

            // For transformed PCs, before race = partial.before_race, current race = fem_race
            // (may be different if race changed during transformation — currently same field)
            let before = if origin.has_before_life() {
                Some(undone_domain::BeforeIdentity {
                    name: partial.before_name.clone(),
                    age: partial.before_age,
                    race: partial.before_race.clone(),
                    sexuality: partial.before_sexuality,
                    figure: undone_domain::MaleFigure::Average,
                    traits: std::collections::HashSet::new(),
                })
            } else {
                None
            };

            let pc_age = if origin == PcOrigin::AlwaysFemale {
                age.get_untracked()       // age set in this form
            } else {
                partial.before_age        // transformed: same age as before
            };

            let config = undone_packs::char_creation::CharCreationConfig {
                name_fem: name_fem.get_untracked(),
                name_androg: name_androg.get_untracked(),
                name_masc: partial.before_name.clone(),
                age: pc_age,
                race: fem_race,
                figure: figure.get_untracked(),
                breasts: breasts.get_untracked(),
                origin,
                before,
                starting_traits: partial.starting_traits,
                male_count: 6,
                female_count: 2,
                starting_flags: std::collections::HashSet::new(),
                starting_arc_states: std::collections::HashMap::new(),
            };

            let PreGameState { mut registry, scenes, scheduler, mut rng, init_error } = pre;
            let opening_scene = registry.opening_scene().map(|s| s.to_owned());
            let default_slot = registry.default_slot().map(|s| s.to_owned());
            let world = undone_packs::char_creation::new_game(config, &mut registry, &mut rng);
            let engine = undone_scene::engine::SceneEngine::new(scenes);
            let gs = crate::game_state::GameState {
                world, registry, engine, scheduler, rng, init_error,
                opening_scene, default_slot,
            };
            *game_state.borrow_mut() = Some(gs);
            signals.phase.set(AppPhase::InGame);
        })
        // ... same button style as existing BeginButton ...
}
```

**Step 6: Wire `FemCreation` arm in `app_view`**

In `lib.rs`, replace the `FemCreation` placeholder with:
```rust
AppPhase::FemCreation => {
    fem_creation_view(signals, Rc::clone(&pre_state_cc), Rc::clone(&game_state_cc), partial_char)
        .into_any()
}
```

Make sure `fem_creation_view` is imported at the top of `lib.rs` (it's in `char_creation` module).

**Step 7: Run cargo check**
```
cargo check -p undone-ui
```

**Step 8: Run all tests**
```
cargo test --workspace
```
Expected: 197 tests pass.

**Step 9: Commit**
```bash
git add crates/undone-ui/src/char_creation.rs crates/undone-ui/src/lib.rs
git commit -m "feat(ui): fem_creation_view — step 3 form, assembles CharCreationConfig, starts game"
```

---

### Task 8: Write `transformation_intro.toml` scene

**Files:**
- Create: `packs/base/scenes/transformation_intro.toml`

**Scene spec:**
- ID: `base::transformation_intro`
- Length: 150–200 words of prose (short — this is transitional)
- POV: second-person present tense (same as all other scenes)
- Tone: the before-moment. Mundane life, then something shifts. Body becomes wrong/different/new all at once. Do not over-explain. The scene ends before the player character has processed it.
- TWO voices via `{% if w.hasTrait("TRANS_WOMAN") %}`:
  - **CisMaleTransformed** (default): shock, confusion, the wrongness of a body that was right before
  - **TransWomanTransformed**: something clicked into place, finally right — the body the character always knew she should have
- One action that ends the scene (no choice logic — just "Continue")
- No effects (throwaway world — effects don't persist to the real game)

**Step 1: Write the scene file**

```toml
[scene]
id          = "base::transformation_intro"
pack        = "base"
description = "The moment before everything changed. Establishing the before-self."

[intro]
prose = """
Tuesday morning. The alarm goes off at seven like it always does.

{% if not w.alwaysFemale() %}
{% if w.hasTrait("TRANS_WOMAN") %}
She's been getting up at seven for three years. The body that gets up is wrong in the specific ways she has learned to catalog — the broad line of the shoulders, the flat chest, the way the jaw sits in the mirror. She knows the list by heart. She is very tired of knowing it.

She is putting on a shirt she has worn a hundred times when something happens that she does not have words for yet. It happens fast. It happens the way a held breath finally releases.

She sits down on the edge of the bed.

Something is different. Everything is different. She puts a hand to her face. Her face is different. She is *shaking* but it isn't fear — or it isn't only fear.

*Oh*, she thinks. *Oh. There you are.*
{% else %}
He gets up. Makes coffee. Stands in the kitchen looking at nothing, which is how most mornings go.

He is putting on a jacket he has owned since college when something happens that he has no frame for. It happens fast. It happens the way a mirror breaks — not slowly, all at once. The jacket no longer fits the way it should. Nothing fits the way it should. The body he is standing in is not the body he had three seconds ago.

He grabs the counter.

He is not catastrophizing. He is not still asleep. Something real has happened and he has no language for it and the coffee is still warm and the world outside the window is exactly the same as it was and *he* is not.
{% endif %}
{% endif %}
"""

[[actions]]
id     = "continue"
label  = "Continue"
detail = "The city outside is the same. You are not."

  [[actions.next]]
  finish = true
```

**Step 2: Validate the template syntax**

Use the MCP tool:
```
mcp__minijinja__jinja_validate_template
```
Paste the prose content. Verify: no errors.

**Step 3: Run validate-pack**
```
cargo run --bin validate-pack
```
Expected: scene loads cleanly (once pack.toml is updated in Task 9, which must happen first for the scene to be registered).

Actually: run validate-pack AFTER Task 9.

**Step 4: Commit**
```bash
git add packs/base/scenes/transformation_intro.toml
git commit -m "content: transformation_intro scene — the before-moment"
```

---

### Task 9: Add `transformation_scene` to `pack.toml` + update loader test

**Files:**
- Modify: `packs/base/pack.toml`
- Modify: `crates/undone-packs/src/loader.rs` (update test assertion)

**Step 1: Add field to pack.toml**

In `[pack]` section of `packs/base/pack.toml`, after `default_slot`:
```toml
transformation_scene = "base::transformation_intro"
```

**Step 2: Update the loader test from Task 1**

Find `fn base_pack_has_transformation_scene()` and update:
```rust
#[test]
fn base_pack_has_transformation_scene() {
    let (registry, _) = load_packs(&packs_dir()).unwrap();
    assert_eq!(
        registry.transformation_scene(),
        Some("base::transformation_intro")
    );
}
```

**Step 3: Run validate-pack**
```
cargo run --bin validate-pack
```
Expected: all scenes load cleanly, including `base::transformation_intro`.

**Step 4: Run all tests**
```
cargo test --workspace
```
Expected: 198+ tests pass (197 old + 1 new loader test).

**Step 5: Commit**
```bash
git add packs/base/pack.toml crates/undone-packs/src/loader.rs
git commit -m "feat: wire transformation_scene in pack.toml, update loader test"
```

---

### Task 10: Remove dead code and do a full build+run verification

**Files:**
- Modify: `crates/undone-ui/src/char_creation.rs` (remove any now-unused helpers)
- Various: whatever clippy flags

**Step 1: Remove the old `section_who_you_are` and `section_your_past` functions**

The old sections were replaced by `before_creation_view`. Any helpers that are now unreachable should be removed. Keep: `section_style`, `input_style`, `dropdown_style`, `section_title`, `hint_label`, `form_row`, `trait_checkbox`, `radio_opt`, `race_picker` — these are shared by both forms.

Remove: `resolve_origin`, `BeforeKind` (replaced by `origin_from_idx`), `CharFormSignals` and its impl (replaced by `BeforeFormSignals`), `build_begin_button` (replaced by `build_next_button` and `build_begin_button_fem`).

**Step 2: Run clippy**
```
cargo clippy --workspace
```
Fix any new warnings.

**Step 3: Full build and manual run test**

```
cargo run
```

Walk through the full flow manually:
1. App opens on `BeforeCreation` form
2. Select "Something happened to me — I was a man", set a name, pick traits → "Next"
3. Transformation intro scene renders — "Tuesday morning. The alarm goes off..." → choose "Continue"
4. `FemCreation` form appears — set feminine name, figure, breasts → "Begin Your Story"
5. Game starts with `rain_shelter` opening scene ✓

Test the AlwaysFemale path:
1. Select "I was always a woman" → "Next"
2. `FemCreation` form appears directly (no transformation intro)
3. "Begin Your Story" → game starts ✓

**Step 4: Run all tests**
```
cargo test --workspace
```
Expected: 198+ tests pass.

**Step 5: Commit**
```bash
git add crates/undone-ui/src/char_creation.rs crates/undone-ui/src/lib.rs
git commit -m "chore: remove old char creation code (CharFormSignals, BeforeKind, resolve_origin)"
```

---

## Testing Strategy

The char creation flow is UI-heavy and difficult to unit test directly. Coverage strategy:

**Unit tests (in `undone-packs`):**
- `base_pack_has_transformation_scene()` — registry getter works (Task 9)
- Existing `new_game_*` tests — verify CharCreationConfig assembly still works

**Manual verification (Task 10, Step 3):**
- All four origin paths: CisMale → CisFemale → TransWoman → AlwaysFemale
- Verify correct scene shows (transformation_intro for transformed, skip for AlwaysFemale)
- Verify correct voice branches (TransWoman gets "Oh. There you are." variant)
- Verify the game proceeds normally after the flow (rain_shelter fires)

---

## Known Complexity: Registry Clone

`PackRegistry` contains `Rodeo` from the `lasso` crate. `Rodeo` implements `Clone`. However, cloning a registry mid-game means the clone has the same interned strings but is a fully independent copy. The throwaway world gets a cloned registry, the real world gets the original.

**This is fine because:**
- The throwaway world is only used for the transformation intro scene
- The throwaway registry is discarded when the throwaway world is dropped
- The real `new_game()` call uses the original pre_state.registry, which is unmodified

**If Clone is not derivable on PackRegistry** (e.g., if Rodeo doesn't impl Clone in the version used), the alternative is to pass `&PackRegistry` references to the throwaway `SceneEngine`. Check the lasso version in `Cargo.toml` and confirm `Rodeo: Clone` before writing code.

---

## Open Questions / Deferred

1. **TransWoman intro voice** — the plan uses `w.hasTrait("TRANS_WOMAN")` to branch. This trait is injected by `new_game()` from the origin. For the throwaway world, `new_game()` with `TransWomanTransformed` origin will inject `TRANS_WOMAN` trait automatically. ✓

2. **CisFemaleTransformed intro** — the scene currently uses the CisMaleTransformed voice as default (no `TRANS_WOMAN` trait, `alwaysFemale()` returns true because ALWAYS_FEMALE is injected). The prose says "He gets up" which is wrong for CisFemale. Future work: add a `w.hasTrait("ALWAYS_FEMALE")` branch in the transformation_intro scene for this path.

3. **Race divergence** — the plan uses `before_race` for the "current race" in the transformation intro and allows the player to set a different race in `fem_creation_view`. This models the transformation potentially changing race. Content addressing this (the scene noticing the player's race changed) is future work.

4. **`name_masc` for AlwaysFemale** — the plan sets `name_masc = partial.before_name` even for AlwaysFemale (which is empty string ""). This means `w.name_masc()` returns "" for AlwaysFemale PCs. Content should use `w.alwaysFemale()` to gate masculine-name references. Confirm this is acceptable or add a placeholder.
