# Playtest Feedback Pass — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Address all playtest feedback across four batches: quick UI/data fixes, transformation intro rewrite, origin presets, and four new week-2 scenes.

**Architecture:** Single worktree (`playtest-fixes` branch). Batches commit separately. Scene writing delegated to `scene-writer` agent with `writing-reviewer` audit pass. Batch 4 uses parallel scene-writer agents.

**Tech Stack:** Rust / floem 0.2 / minijinja / TOML packs / serde save migration

---

## Pre-flight

Create worktree and run clean tests before touching any code:

```bash
# From repo root
git worktree add .worktrees/playtest-fixes -b playtest-fixes
cd .worktrees/playtest-fixes
cargo test --workspace 2>&1 | tail -5
# Expected: 200 tests, 0 failures
```

---

## BATCH 1 — Quick UI/Data Fixes

### Task 1: Fix dark mode dropdown text color

**Files:**
- Modify: `crates/undone-ui/src/char_creation.rs`

The issue: `Dropdown::new_rw(...).style(field_style(signals))` sets the outer container style, but floem 0.2 Dropdown popup list items render with system defaults (dark text on white) regardless of theme. In Night mode the text is light (`colors.ink`) but the popup background stays white → text is near-invisible.

**Investigation step:** Check floem 0.2 Dropdown for a list-item-style API.

Run in the project:
```bash
grep -r "Dropdown" $(cargo metadata --format-version 1 | \
  python -c "import json,sys; d=json.load(sys.stdin); \
  [print(p['manifest_path']) for p in d['packages'] if p['name']=='floem']" \
  2>/dev/null | head -1 | xargs dirname)/src/ 2>/dev/null | grep -i "style\|item" | head -20
```

Alternatively: look at floem's dropdown source in the Cargo cache:
```bash
find ~/.cargo/registry/src -path "*/floem-0.2*/src/views/dropdown*" 2>/dev/null | head -5
```
or on Windows:
```bash
find "C:/Users/YJK/.cargo/registry/src" -path "*/floem-0.2*/src/views/dropdown*" 2>/dev/null | head -5
```

**Step 1: Read floem's Dropdown source** to find available styling hooks.

Expected: either `popup_style()`, `list_style()`, or `list_item_style()` methods exist on `Dropdown`.

**Step 2: If a list/popup style method exists**, modify every `Dropdown::new_rw(...)` call in `char_creation.rs` to chain that method. Set:
- background → `colors.page_raised`
- color → `colors.ink`
- font_family + font_size matching `field_style`

Pattern to add after each `Dropdown::new_rw(...).style(field_style(signals))`:
```rust
.list_style(move |s| {   // ← exact method name TBD from floem source
    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
    s.background(colors.page_raised)
     .color(colors.ink)
     .font_size(14.0)
     .font_family("system-ui, -apple-system, sans-serif".to_string())
})
```

There are 5 Dropdown instances in `char_creation.rs` — both dropdowns in `before_fields` closure + the age dropdown in `fem_creation_view`.

**Step 3: If no list style method exists**, create a `themed_dropdown` helper that wraps the dropdown in a container and sets the popup overlay style via CSS class override:

```rust
fn themed_dropdown<T>(
    signal: RwSignal<T>,
    items: Vec<T>,
    signals: AppSignals,
) -> impl View
where
    T: std::fmt::Display + Clone + PartialEq + 'static,
{
    Dropdown::new_rw(signal, items)
        .style(field_style(signals))
        // fallback: container with explicit background override for popup
}
```

**Step 4: Build and visually verify** — start the game, switch to Night theme, open a dropdown:
```bash
cargo build --release 2>&1 | tail -5
```
Then use screenshot MCP to capture before/after.

**Step 5:** Run tests:
```bash
cargo test -p undone-ui 2>&1 | tail -5
```

---

### Task 2: Fix name field (remove "Evan" default, add randomize button)

**Files:**
- Modify: `crates/undone-ui/src/char_creation.rs`

**Step 1:** Change `BeforeFormSignals::new()` — line 71:
```rust
// Before:
before_name: RwSignal::new("Evan".to_string()),
// After:
before_name: RwSignal::new(String::new()),
```

**Step 2:** Add a `read_male_names` helper near `read_races`:
```rust
fn read_male_names(pre_state: &Rc<RefCell<Option<PreGameState>>>) -> Vec<String> {
    if let Some(ref pre) = *pre_state.borrow() {
        if !pre.registry.male_names().is_empty() {
            return pre.registry.male_names().to_vec();
        }
    }
    vec!["Matt".to_string(), "Ryan".to_string(), "David".to_string()]
}
```

**Step 3:** In `char_creation_view`, read the names and pick an initial hint:
```rust
let male_names = read_male_names(&pre_state);
// Use a random name from the list as placeholder hint
let hint_name: String = {
    use rand::seq::SliceRandom;
    male_names
        .choose(&mut rand::thread_rng())
        .cloned()
        .unwrap_or_else(|| "Matt".to_string())
};
```

**Step 4:** Pass `male_names` and `hint_name` to `section_your_past`. Update `section_your_past` signature:
```rust
fn section_your_past(
    signals: AppSignals,
    form: BeforeFormSignals,
    races: Vec<String>,
    male_names: Vec<String>,
    hint_name: String,
) -> impl View {
```

**Step 5:** Inside the `before_fields` closure, replace the current `name_row` with a row that includes a Randomize button:
```rust
let hint = hint_name.clone();   // captured by the dyn_container closure
let names_for_btn = male_names.clone();

// ... inside dyn_container closure:
let name_row = form_row(
    "Name before",
    signals,
    h_stack((
        text_input(form.before_name)
            .placeholder(hint.clone())   // shows random name as hint
            .style(move |s| field_style(signals)(s).width(160.0)),
        label(|| "Randomize".to_string())
            .keyboard_navigable()
            .on_click_stop(move |_| {
                use rand::seq::SliceRandom;
                if let Some(name) = names_for_btn.choose(&mut rand::thread_rng()) {
                    form.before_name.set(name.clone());
                }
            })
            .style(move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                s.margin_left(8.0)
                 .padding_horiz(12.0)
                 .padding_vert(6.0)
                 .font_size(13.0)
                 .color(colors.ink_dim)
                 .border(1.0)
                 .border_color(colors.seam)
                 .border_radius(4.0)
                 .cursor(floem::style::CursorStyle::Pointer)
                 .font_family("system-ui, -apple-system, sans-serif".to_string())
            }),
    ))
    .style(|s| s.items_center()),
);
```

Note: the `hint_name` is a `String` captured by value; `male_names` is also a `Vec<String>` captured by value. The `dyn_container` closure runs when `origin_idx` changes — we need these values to be `Clone`-captured. Use `let hint = hint_name.clone()` and `let names_for_btn = male_names.clone()` before the closure.

The existing name guard in `build_next_button` already handles the "empty name blocks Next" case — no change needed there.

**Step 6:** Update the call site in `char_creation_view`:
```rust
section_your_past(signals, form, races_list, male_names, hint_name),
```

**Step 7:** Run `cargo fmt -p undone-ui && cargo check -p undone-ui`

---

### Task 3: Age enum simplification + save migration

**Files:**
- Modify: `crates/undone-domain/src/enums.rs`
- Modify: `packs/base/data/categories.toml`
- Modify: `crates/undone-ui/src/char_creation.rs`
- Modify: `crates/undone-packs/src/spawner.rs`
- Modify: `crates/undone-save/src/lib.rs`
- Modify: `crates/undone-expr/src/eval.rs` (test fixtures)
- Modify: `crates/undone-packs/src/char_creation.rs` (test fixtures)
- Modify: `crates/undone-scene/src/effects.rs` (test fixtures)
- Modify: `crates/undone-scene/src/engine.rs` (test fixtures)
- Modify: `crates/undone-scene/src/lib.rs` (test fixture)
- Modify: `crates/undone-scene/src/scheduler.rs` (test fixture)
- Modify: `crates/undone-scene/src/template_ctx.rs` (test fixture)
- Modify: `crates/undone-ui/src/lib.rs` (test fixture)

**Step 1:** In `enums.rs`, replace `Twenties` with `MidLateTwenties`:
```rust
pub enum Age {
    LateTeen,
    EarlyTwenties,
    MidLateTwenties,   // replaces Twenties
    LateTwenties,
    Thirties,
    Forties,
    Fifties,
    Old,
}
```

Update `Display` impl:
```rust
Age::EarlyTwenties => write!(f, "Early 20s"),
Age::MidLateTwenties => write!(f, "Mid to Late 20s"),
Age::LateTwenties => write!(f, "Late Twenties"),  // keep or change as needed
```

Note: The task only explicitly mentions the two new variants' display strings. Keep others as-is or improve at discretion.

**Step 2:** Compile to find all `Age::Twenties` references:
```bash
cargo check --workspace 2>&1 | grep "Twenties"
```

Replace all `Age::Twenties` occurrences in non-documentation files:
- In production code (spawner, char_creation fixtures): `Age::Twenties` → `Age::MidLateTwenties`
- In test fixtures that construct worlds (eval.rs, effects.rs, engine.rs, lib.rs, scheduler.rs, template_ctx.rs, save/lib.rs, packs/char_creation.rs, ui/lib.rs): same replacement

This is safe because the test fixtures are creating arbitrary test players, not asserting specific age values. MidLateTwenties is the semantic equivalent.

**Step 3:** Update `packs/base/data/categories.toml`:
```toml
[[category]]
id = "AGE_YOUNG"
type = "age"
members = ["LateTeen", "EarlyTwenties", "MidLateTwenties", "LateTwenties"]
```

**Step 4:** Update `char_creation.rs` dropdown lists. Both appearances (BeforeCreation and FemCreation) need to change:
```rust
Age::LateTeen,
Age::EarlyTwenties,
Age::MidLateTwenties,   // replaces Age::Twenties
Age::LateTwenties,
Age::Thirties,
Age::Forties,
Age::Fifties,
Age::Old,
```

Also change `before_age` default in `BeforeFormSignals::new()`:
```rust
before_age: RwSignal::new(Age::EarlyTwenties),  // was Twenties
```

**Step 5:** Add v4 save migration in `crates/undone-save/src/lib.rs`:

Bump `SAVE_VERSION` to 4.

Add `migrate_v3_to_v4` function that maps `"Twenties"` to `"MidLateTwenties"` in both player age fields and before age:
```rust
fn migrate_v3_to_v4(mut save_json: serde_json::Value) -> serde_json::Value {
    // Rename Age::Twenties → Age::MidLateTwenties in world.player.age
    if let Some(age) = save_json
        .get_mut("world")
        .and_then(|w| w.get_mut("player"))
        .and_then(|p| p.get_mut("age"))
    {
        if age == "Twenties" {
            *age = serde_json::Value::String("MidLateTwenties".to_string());
        }
    }
    // Same for world.player.before.age
    if let Some(before_age) = save_json
        .get_mut("world")
        .and_then(|w| w.get_mut("player"))
        .and_then(|p| p.get_mut("before"))
        .and_then(|b| b.get_mut("age"))
    {
        if before_age == "Twenties" {
            *before_age = serde_json::Value::String("MidLateTwenties".to_string());
        }
    }
    save_json
}
```

Update `load_game` to run v3→v4 migration in the chain. Update migration comment in `load_game` doc string to mention v3→v4.

Also update the v2→v3 migration in `migrate_v2_to_v3` where the numeric age maps to `"Twenties"`:
```rust
23..=26 => "MidLateTwenties",  // was "Twenties"
```

**Step 6:** Update the `make_world` test helper in `crates/undone-save/src/lib.rs` test module — change `age: Age::Twenties` to `age: Age::MidLateTwenties`:
```rust
before: Some(BeforeIdentity {
    name: "Evan".into(),
    age: Age::MidLateTwenties,   // was Twenties
    ...
```

Add a test for v3→v4 migration:
```rust
#[test]
fn migrate_v3_save_to_v4_renames_twenties() {
    let (registry, _) = undone_packs::load_packs(&packs_dir()).unwrap();
    let world = make_world(&registry);
    let mut save_json = serde_json::to_value(&SaveFile {
        version: 3,
        id_strings: registry.all_interned_strings(),
        world,
    }).unwrap();
    // Patch player.age to "Twenties" to simulate a v3 save
    save_json["world"]["player"]["age"] = serde_json::Value::String("Twenties".into());
    save_json["world"]["player"]["before"]["age"] = serde_json::Value::String("Twenties".into());
    let dir = tempfile_dir();
    let path = dir.join("v3_migration_test.json");
    std::fs::write(&path, serde_json::to_string_pretty(&save_json).unwrap()).unwrap();
    let loaded = load_game(&path, &registry).expect("v3→v4 migration should succeed");
    assert_eq!(loaded.player.age, Age::MidLateTwenties, "age should migrate");
    assert_eq!(
        loaded.player.before.as_ref().unwrap().age,
        Age::MidLateTwenties,
        "before.age should migrate"
    );
}
```

**Step 7:** Run all tests:
```bash
cargo test --workspace 2>&1 | tail -20
```
Expected: all pass. Fix any remaining Age::Twenties compilation errors.

---

### Task 4: Origin radio labels with subtitles

**Files:**
- Modify: `crates/undone-ui/src/char_creation.rs`

**Step 1:** Update `radio_opt` to accept an optional subtitle:
```rust
fn radio_opt(
    opt_label: &'static str,
    subtitle: &'static str,
    is_active: impl Fn() -> bool + Copy + 'static,
    on_select: impl Fn() + Copy + 'static,
    signals: AppSignals,
) -> impl View {
    let indicator = /* ... unchanged ... */;
    v_stack((
        h_stack((
            indicator,
            label(move || opt_label.to_string()).style(move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                s.font_size(14.0)
                    .color(colors.ink)
                    .font_family("system-ui, -apple-system, sans-serif".to_string())
            }),
        ))
        .style(|s| s.items_center()),
        label(move || subtitle.to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(12.0)
                .color(colors.ink_ghost)
                .margin_left(21.0)   // indent to align with label (13px indicator + 8px margin)
                .margin_bottom(4.0)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
    ))
    .style(|s| {
        s.flex_col()
            .cursor(floem::style::CursorStyle::Pointer)
            .margin_bottom(8.0)
    })
    .on_click_stop(move |_| on_select())
}
```

**Step 2:** Update `origin_radios` in `section_your_past` with exact copy from HANDOFF.md:
```rust
let origin_radios = v_stack((
    radio_opt(
        "Something happened to me \u{2014} I was a man",
        "Transformed from male. The core experience.",
        move || origin_idx.get() == 0,
        move || origin_idx.set(0),
        signals,
    ),
    radio_opt(
        "Something happened to me \u{2014} I was a trans woman",
        "Already knew yourself. The transformation was recognition.",
        move || origin_idx.get() == 1,
        move || origin_idx.set(1),
        signals,
    ),
    radio_opt(
        "Something happened to me \u{2014} I was a woman",
        "You were female. Something still changed.",
        move || origin_idx.get() == 2,
        move || origin_idx.set(2),
        signals,
    ),
    radio_opt(
        "I was always a woman",
        "No transformation. Play as yourself.",
        move || origin_idx.get() == 3,
        move || origin_idx.set(3),
        signals,
    ),
))
.style(|s| s.margin_bottom(16.0));
```

**Step 3:** Run `cargo fmt -p undone-ui && cargo check -p undone-ui`

### Batch 1 commit

After all four tasks pass:
```bash
cargo test --workspace 2>&1 | tail -10
git add crates/undone-domain/src/enums.rs \
        crates/undone-ui/src/char_creation.rs \
        crates/undone-save/src/lib.rs \
        crates/undone-packs/src/spawner.rs \
        packs/base/data/categories.toml \
        crates/undone-expr/src/eval.rs \
        crates/undone-packs/src/char_creation.rs \
        crates/undone-scene/src/effects.rs \
        crates/undone-scene/src/engine.rs \
        crates/undone-scene/src/lib.rs \
        crates/undone-scene/src/scheduler.rs \
        crates/undone-scene/src/template_ctx.rs \
        crates/undone-ui/src/lib.rs
git commit -m "fix: batch 1 playtest — dropdown theme, name field, age enum, origin labels"
```

---

## BATCH 2 — Transformation Intro Rewrite

### Task 5/6/7: Rewrite transformation_intro.toml

**File:** `packs/base/scenes/transformation_intro.toml`

**Context for scene-writer agent:**

The current scene is critically broken:
1. Written in third person ("She/He") — must be second person present tense
2. Too short — the character should NOT immediately realise what happened; it takes several moments
3. No trait branches — must add at least 3–4 based on before-traits
4. Has AI-isms to fix

Required structure:
- `[intro]` prose: the waking up, the gradual wrongness, the realisation taking hold
- Inner voice (`[[thoughts]]`) for the first confused internal moment
- Trait branches for: SHY, AMBITIOUS, OUTGOING, OVERACTIVE_IMAGINATION (at minimum)
- CisMale origin: disorientation register, male pronouns starting to slip, mirror as factual confrontation
- TransWoman origin: relief/recognition register, finally, the rightness of it
- CisFemaleTransformed: different change, not a body swap — something shifted
- AlwaysFemale path: must be complete and valid (perhaps just a vivid morning routine with no transformation content)

The scene should feel like: waking up → something is wrong → catalogue symptoms → check mirror → try to understand → the full weight lands → "Continue"

**Step 1:** Dispatch a `scene-writer` agent to write the scene. The agent MUST:
- Read `docs/writing-guide.md` in full before writing any prose
- Read `packs/base/scenes/transformation_intro.toml` (current bad version)
- Read `docs/characters/robin.md` + `docs/characters/camila.md` for character context
- Produce `packs/base/scenes/transformation_intro.toml` rewritten

Dispatch using `subagent_type: "scene-writer"` with detailed prompt.

**Step 2:** Dispatch a `writing-reviewer` agent on the result. It must report Critical/Important/Minor findings on:
- Third-person violations
- AI-isms (staccato, em-dash, over-naming)
- Voice consistency
- Trait branch structural difference (not adjective swap)
- Template syntax

**Step 3:** Apply all Critical findings manually, then validate:
```bash
# Validate minijinja template
# (use mcp__minijinja__jinja_validate_template)
```

**Step 4:** Run test to confirm pack loads:
```bash
cargo test -p undone-packs 2>&1 | tail -5
```

### Batch 2 commit
```bash
git add packs/base/scenes/transformation_intro.toml
git commit -m "content: rewrite transformation_intro — second person, longer, trait branches, no AI-isms"
```

---

## BATCH 3 — Origin Character Presets

### Task 8: Add Robin and Camila as preset starts

**Files:**
- Modify: `crates/undone-ui/src/char_creation.rs`

**Design:**

The `BeforeCreation` screen gets a new section at the top: "Who were you?" with three options:
- `[○] Start as Robin` — 2-sentence blurb, shows attributes read-only
- `[○] Start as Raul` — same for Camila
- `[○] Create your own` — shows existing full form

When a preset is selected: display a static description block. When "Create your own" is selected: show the existing form unchanged.

**Preset mode state:**

Add a new `CharacterMode` enum or use a signal:
```rust
#[derive(Clone, Copy, PartialEq, Eq)]
enum CharMode {
    PresetRobin = 0,
    PresetRaul  = 1,
    Custom      = 2,
}
```
Or just use a `RwSignal<u8>` (0=Robin, 1=Raul, 2=Custom) similar to `origin_idx`.

**Step 1:** Add `char_mode: RwSignal<u8>` (0=Robin, 1=Raul, 2=Custom) to `BeforeFormSignals`:
```rust
char_mode: RwSignal::new(2u8),   // default to Custom
```

**Step 2:** Add preset config data as constants at the top of `char_creation.rs`:

```rust
// Robin preset — who Robin was before transformation
struct PresetData {
    display_name: &'static str,
    before_name:  &'static str,
    before_age:   Age,
    origin:       PcOrigin,
    before_sexuality: BeforeSexuality,
    // trait names for starting_traits resolution
    trait_ids:    &'static [&'static str],
    blurb:        &'static str,
}

const PRESET_ROBIN: PresetData = PresetData {
    display_name: "Robin",
    before_name:  "Robin",
    before_age:   Age::Thirties,
    origin:       PcOrigin::CisMaleTransformed,
    before_sexuality: BeforeSexuality::AttractedToWomen,
    trait_ids:    &["AMBITIOUS"],
    blurb:        "You're thirty-two, a software engineer with ten years of experience. \
                   You took a job offer in a city you didn't know — new company, new start, \
                   boxes shipped to an apartment you've never seen. When things go sideways, \
                   you inventory and solve. You're very good at that.",
};

const PRESET_RAUL: PresetData = PresetData {
    display_name: "Raul",
    before_name:  "Raul",
    before_age:   Age::LateTeen,
    origin:       PcOrigin::CisMaleTransformed,
    before_sexuality: BeforeSexuality::AttractedToWomen,
    trait_ids:    &["AMBITIOUS"],
    blurb:        "You're eighteen, starting at a university your family has talked about for years. \
                   You arrived with your expectations calibrated: you knew who you were, where you \
                   were headed, and what the next four years were supposed to look like. \
                   Things have always worked out. You've never had a real reason to think they wouldn't.",
};
```

**Step 3:** Add `section_preset_select` function that renders the three-way selector at the top of `char_creation_view`:

```rust
fn section_preset_select(signals: AppSignals, char_mode: RwSignal<u8>) -> impl View {
    let mode = char_mode;
    v_stack((
        section_title("Who Were You?", signals),
        hint_label("Choose a character or create your own:", signals),
        v_stack((
            radio_opt("Start as Robin", "Software engineer, 32. Methodical. Arrived Saturday.",
                move || mode.get() == 0, move || mode.set(0), signals),
            radio_opt("Start as Raul", "College freshman, 18. Privileged. Everything expected to go right.",
                move || mode.get() == 1, move || mode.set(1), signals),
            radio_opt("Create your own", "Build your character from scratch.",
                move || mode.get() == 2, move || mode.set(2), signals),
        )),
    ))
    .style(section_style())
}
```

**Step 4:** Add preset detail view that shows when a preset is selected:

```rust
fn section_preset_detail(
    signals: AppSignals,
    preset: &'static PresetData,
) -> impl View {
    v_stack((
        label(move || preset.blurb.to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(15.0)
                .color(colors.ink)
                .line_height(1.6)
                .margin_bottom(20.0)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
        // Read-only attribute display
        v_stack((
            read_only_row("Name before",   preset.before_name,             signals),
            read_only_row("Age before",    &preset.before_age.to_string(), signals),
            read_only_row("Origin",        "Transformed from male",        signals),
        )).style(|s| s.margin_bottom(8.0)),
    ))
    .style(section_style())
}

fn read_only_row(label_text: &'static str, value: &'static str, signals: AppSignals) -> impl View {
    h_stack((
        label(move || label_text.to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.width(180.0).font_size(14.0).color(colors.ink_dim)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
        label(move || value.to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(14.0).color(colors.ink_ghost)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
    ))
    .style(|s| s.items_center().margin_bottom(8.0))
}
```

Note: `before_age.to_string()` can't be `&'static str` easily, so `read_only_row` should take `&str` or use a `Cow<'static, str>`. Simplest fix: take a `String` and use `move || value.clone()`.

**Step 5:** In `char_creation_view`, restructure the main content:

```rust
pub fn char_creation_view(...) -> impl View {
    let form = BeforeFormSignals::new();
    let char_mode = form.char_mode;   // 0=Robin, 1=Raul, 2=Custom

    // ... existing setup ...

    let preset_section = section_preset_select(signals, char_mode);

    // Dynamically show preset detail or the full form
    let main_content = dyn_container(
        move || char_mode.get(),
        move |mode| match mode {
            0 => v_stack((
                section_preset_detail(signals, &PRESET_ROBIN),
            )).into_any(),
            1 => v_stack((
                section_preset_detail(signals, &PRESET_RAUL),
            )).into_any(),
            _ => v_stack((
                section_your_past(signals, form, races_list.clone(), male_names.clone(), hint_name.clone()),
                section_personality(signals, form),
                section_content_prefs(signals, form),
            )).into_any(),
        },
    );

    let content = v_stack((
        heading("Your Story Begins", signals),
        preset_section,
        main_content,
        next_btn,
        empty().style(|s| s.height(40.0)),
    ))
    // ... existing style ...
```

**Step 6:** Update `build_next_button` to handle preset modes:

When char_mode is 0 or 1 (preset selected), bypass form reading and use preset data directly:

```rust
fn build_next_button(...) -> impl View {
    label(|| "Next \u{2192}".to_string())
        .keyboard_navigable()
        .on_click_stop(move |_| {
            let mode = form.char_mode.get_untracked();

            let (origin, before_name, before_age, before_race, before_sexuality, trait_ids) =
                match mode {
                    0 => (
                        PRESET_ROBIN.origin,
                        PRESET_ROBIN.before_name.to_string(),
                        PRESET_ROBIN.before_age,
                        "White".to_string(),
                        PRESET_ROBIN.before_sexuality,
                        PRESET_ROBIN.trait_ids,
                    ),
                    1 => (
                        PRESET_RAUL.origin,
                        PRESET_RAUL.before_name.to_string(),
                        PRESET_RAUL.before_age,
                        "Latino".to_string(),
                        PRESET_RAUL.before_sexuality,
                        PRESET_RAUL.trait_ids,
                    ),
                    _ => {
                        // Custom mode — existing logic unchanged
                        let origin_idx = form.origin_idx.get_untracked();
                        if origin_idx != 3 && form.before_name.get_untracked().trim().is_empty() {
                            return;
                        }
                        // ... collect all traits from form ...
                        // This block resolves traits and builds the partial, then returns
                        // (same code as before, just restructured to fit this match arm)
                        // For clarity, extract into a helper or just inline
                        let origin = origin_from_idx(origin_idx);
                        // ... existing trait collection ...
                        // ... build partial_char ...
                        // ... handle AlwaysFemale / TransformationIntro ...
                        return;
                    },
                };

            // Preset path: resolve trait IDs, build partial, proceed to TransformationIntro
            let starting_traits: Vec<_> = {
                let pre_borrow = pre_state.borrow();
                if let Some(ref pre) = *pre_borrow {
                    trait_ids.iter()
                        .filter_map(|name| pre.registry.resolve_trait(name).ok())
                        .collect()
                } else {
                    vec![]
                }
            };

            let partial = PartialCharState {
                origin,
                before_name: before_name.clone(),
                before_age,
                before_race: before_race.clone(),
                before_sexuality,
                starting_traits,
            };
            partial_char.set(Some(partial.clone()));

            // Create throwaway world and proceed to TransformationIntro (same as custom path)
            // ... (copy the throwaway world creation from existing code)
            signals.phase.set(AppPhase::TransformationIntro);
        })
        // ... style unchanged ...
}
```

Note: The "Custom" match arm needs to return early (or alternatively use `goto` / restructure the flow). The cleanest approach is to extract the existing button logic into a helper function `handle_custom_next(...)` and call it from the `_` arm.

**Step 7:** Run `cargo fmt -p undone-ui && cargo check -p undone-ui`. Fix any lifetime/clone issues.

**Step 8:** Run full tests:
```bash
cargo test --workspace 2>&1 | tail -10
```

### Batch 3 commit
```bash
git add crates/undone-ui/src/char_creation.rs
git commit -m "feat: add Robin and Raul preset starts to character creation"
```

---

## BATCH 4 — Week-2 Scenes (Parallel)

### Tasks 9–10: Four new scenes via parallel scene-writer agents

**Files to create:**
- `packs/base/scenes/robin_work_meeting.toml`
- `packs/base/scenes/robin_evening.toml`
- `packs/base/scenes/camila_study_session.toml`
- `packs/base/scenes/camila_dining_hall.toml`

**Files to modify:**
- `packs/base/data/schedule.toml` (add schedule entries for all four)

**Step 1: Dispatch four scene-writer agents in parallel**, one per scene.

Each agent must:
1. Read `docs/writing-guide.md` before writing anything
2. Read the relevant character doc and arc doc
3. Write the scene TOML file
4. Validate template syntax with `mcp__minijinja__jinja_validate_template`

**Agent prompts (summarised):**

*robin_work_meeting.toml:*
- Arc: `base::robin_opening`, state gate: `"working"`
- Condition: `gd.hasGameFlag('ROUTE_ROBIN') && gd.arcState('base::robin_opening') == 'working'`
- Content: Robin and PC have their first proper scheduled work meeting. Robin has been at the job for a week. She knows her stuff. The world is still reading her as 18. The meeting should feel competent on her end and slightly off-balance on the world's end — someone explains something to her she invented.
- Must include: at least 2 PC-trait branches (SHY, AMBITIOUS are good candidates), transformation content for transformed PCs (she knows the internal monologue of the men explaining things to her — she used to be that)
- Sets: `ROBIN_WORK_MET` game flag

*robin_evening.toml:*
- Arc: `base::robin_opening`, state gate: `"working"`
- Content: Robin invites PC to stay after a long day. Could be just working late, or could be "let's get a drink." The texture depends on relationship stats or traits. Keep it relationship-dependent (early friendship dynamic) — not romantic yet. The fatigue of the week, the wry observation of surviving it.
- Must include: NPC liking-dependent branches (early vs. warmer), transformation content for transformed PCs
- Sets: `ROBIN_EVENING_DONE` game flag

*camila_study_session.toml:*
- Arc: `base::camila_opening`, state gate: `"first_week"`
- Condition: `gd.hasGameFlag('ROUTE_CAMILA') && gd.arcState('base::camila_opening') == 'first_week'`
- Content: Camila and PC study together. Camila is smart, proud, has been unsettled by her first week. The academic performance is fine — she's always been good at this. But the social dynamics of the university are surprising. She expected to be positioned correctly in the hierarchy. She wasn't. The study session is a moment of shared competence in an otherwise disorienting week.
- Must include: trait branches for AMBITIOUS + at least one other, transformation content
- Sets: `CAMILA_STUDY_DONE` game flag

*camila_dining_hall.toml:*
- Arc: `base::camila_opening`, state gate: `"first_week"`
- Content: The dining hall. Camila introduces (or doesn't introduce) the PC to her friends. The social position question is live here. Does Camila include the PC naturally? Does she hesitate? Is she protective, or just awkward? Branches based on PC traits and NPC liking.
- Must include: FLIRTY + SHY trait branches, social dynamics around introduction, transformation content
- Sets: `CAMILA_DINING_MET_FRIENDS` or similar game flag

**Step 2: Dispatch four writing-reviewer agents in parallel**, one per scene.

Each agent reports Critical/Important/Minor findings.

**Step 3:** Apply all Critical findings. Common patterns to catch:
- Any third-person prose
- AI-isms (staccato, em-dash, over-naming)
- Adjective-swap trait branches (must change what happens, not just the adjective)

**Step 4:** Add schedule entries for all four scenes in `packs/base/data/schedule.toml`:

```toml
# Week-2 Robin scenes
  [[slot.events]]
  scene     = "base::robin_work_meeting"
  condition = "gd.hasGameFlag('ROUTE_ROBIN') && gd.arcState('base::robin_opening') == 'working'"
  weight    = 8
  once_only = true

  [[slot.events]]
  scene     = "base::robin_evening"
  condition = "gd.hasGameFlag('ROUTE_ROBIN') && gd.arcState('base::robin_opening') == 'working'"
  weight    = 6
  once_only = true
```

These go inside the existing `[[slot]] name = "robin_opening"` section.

For Camila, add to `[[slot]] name = "camila_opening"`:
```toml
  [[slot.events]]
  scene     = "base::camila_study_session"
  condition = "gd.hasGameFlag('ROUTE_CAMILA') && gd.arcState('base::camila_opening') == 'first_week'"
  weight    = 8
  once_only = true

  [[slot.events]]
  scene     = "base::camila_dining_hall"
  condition = "gd.hasGameFlag('ROUTE_CAMILA') && gd.arcState('base::camila_opening') == 'first_week'"
  weight    = 7
  once_only = true
```

**Step 5:** Run tests to confirm scenes load:
```bash
cargo test -p undone-packs 2>&1 | tail -10
cargo test --workspace 2>&1 | tail -10
```

### Batch 4 commit
```bash
git add packs/base/scenes/robin_work_meeting.toml \
        packs/base/scenes/robin_evening.toml \
        packs/base/scenes/camila_study_session.toml \
        packs/base/scenes/camila_dining_hall.toml \
        packs/base/data/schedule.toml
git commit -m "content: add week-2 scenes — robin work meeting, robin evening, camila study, camila dining hall"
```

---

## Completion

**Step 1:** Final test run:
```bash
cargo test --workspace 2>&1 | tail -20
# Required: 200+ tests, 0 failures, 0 warnings
```

**Step 2:** Update `HANDOFF.md` session log and Next Action.

**Step 3:** Push to origin:
```bash
git push -u origin playtest-fixes
```

**Step 4:** Invoke `superpowers:finishing-a-development-branch` to merge.

---

## Key File Reference

| File | Role |
|---|---|
| `crates/undone-ui/src/char_creation.rs` | UI form — all 4 Batch 1 tasks touch this |
| `crates/undone-domain/src/enums.rs` | `Age` enum — Task 3 |
| `crates/undone-save/src/lib.rs` | Save migration v3→v4 — Task 3 |
| `packs/base/data/categories.toml` | Age category — Task 3 |
| `packs/base/scenes/transformation_intro.toml` | Scene rewrite — Batch 2 |
| `packs/base/scenes/robin_work_meeting.toml` | New — Batch 4 |
| `packs/base/scenes/robin_evening.toml` | New — Batch 4 |
| `packs/base/scenes/camila_study_session.toml` | New — Batch 4 |
| `packs/base/scenes/camila_dining_hall.toml` | New — Batch 4 |
| `packs/base/data/schedule.toml` | Schedule entries — Batch 4 |
