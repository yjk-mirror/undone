# Engineering Tasks Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement keyboard controls redesign, settings tab UI, and 6 audit fixes without any content/creative work.

**Architecture:** Three clusters: (1) undone-scene changes (EngineEvent, SceneEngine, effects), (2) undone-packs changes (races), (3) undone-ui changes (keyboard, settings, story cap, dispatch refactor). Tasks ordered so lower crates are done first.

**Tech Stack:** Rust, floem (UI), serde_json (prefs), lasso (string interning), pulldown-cmark (prose rendering)

---

### Task 1: Add `NumberKeyMode` to `UserPrefs`

**Files:**
- Modify: `crates/undone-ui/src/theme.rs`

**Step 1: Add the enum and field**

Replace the `ThemeMode` block at the top with the following additions (insert after `ThemeMode` enum, before `UserPrefs` struct):

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NumberKeyMode {
    Instant,
    Confirm,
}

impl Default for NumberKeyMode {
    fn default() -> Self {
        NumberKeyMode::Instant
    }
}
```

Add `number_key_mode: NumberKeyMode` to `UserPrefs`:
```rust
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct UserPrefs {
    pub mode: ThemeMode,
    pub font_family: String,
    pub font_size: u8,
    pub line_height: f32,
    #[serde(default)]
    pub number_key_mode: NumberKeyMode,
}
```

Update `Default for UserPrefs` to include:
```rust
number_key_mode: NumberKeyMode::Instant,
```

**Step 2: Add test**

In the `#[cfg(test)]` block, add:
```rust
#[test]
fn number_key_mode_roundtrip_serde() {
    let prefs = UserPrefs {
        number_key_mode: NumberKeyMode::Confirm,
        ..UserPrefs::default()
    };
    let json = serde_json::to_string(&prefs).unwrap();
    let back: UserPrefs = serde_json::from_str(&json).unwrap();
    assert_eq!(back.number_key_mode, NumberKeyMode::Confirm);
    // Old prefs without the field should deserialize to Instant
    let old_json = r#"{"mode":"Light","font_family":"x","font_size":17,"line_height":1.5}"#;
    let old: UserPrefs = serde_json::from_str(old_json).unwrap();
    assert_eq!(old.number_key_mode, NumberKeyMode::Instant);
}
```

**Step 3: Run tests and check**

```bash
cd /home/yjk/dev/mirror/undone && cargo fmt -p undone-ui && cargo test -p undone-ui -- theme 2>&1 | head -30
```
Expected: all theme tests pass.

**Step 4: Check compiles workspace-wide**

```bash
cd /home/yjk/dev/mirror/undone && cargo check 2>&1 | tail -5
```
Expected: no errors.

**Step 5: Commit**

```bash
cd /home/yjk/dev/mirror/undone && git add crates/undone-ui/src/theme.rs && git commit -m "feat: add NumberKeyMode enum to UserPrefs (serde default=Instant)"
```

---

### Task 2: Add `ErrorOccurred` + `advance_with_action` to Scene Engine

**Files:**
- Modify: `crates/undone-scene/src/engine.rs`

**Step 1: Add `ErrorOccurred` variant to `EngineEvent`**

Find the `EngineEvent` enum (around line 49) and add the new variant:
```rust
#[derive(Debug, Clone)]
pub enum EngineEvent {
    ProseAdded(String),
    ActionsAvailable(Vec<ActionView>),
    NpcActivated(Option<NpcActivatedData>),
    SceneFinished,
    ErrorOccurred(String),
}
```

**Step 2: Add `advance_with_action` method to `SceneEngine`**

After the `drain()` method, add:
```rust
/// Convenience: send a ChooseAction command and immediately drain events.
/// Use this from the UI instead of calling send() + drain() separately.
pub fn advance_with_action(
    &mut self,
    action_id: &str,
    world: &mut World,
    registry: &PackRegistry,
) -> Vec<EngineEvent> {
    self.send(EngineCommand::ChooseAction(action_id.to_string()), world, registry);
    self.drain()
}
```

**Step 3: Add test for `advance_with_action`**

In the `#[cfg(test)]` block, add:
```rust
#[test]
fn advance_with_action_returns_events() {
    let mut engine = make_engine_with(make_simple_scene());
    let mut world = make_world();
    let registry = PackRegistry::new();

    engine.send(
        EngineCommand::StartScene("test::simple".into()),
        &mut world,
        &registry,
    );
    engine.drain();

    // advance_with_action("leave") should produce ProseAdded + NpcActivated + SceneFinished
    let events = engine.advance_with_action("leave", &mut world, &registry);
    assert!(
        events.iter().any(|e| matches!(e, EngineEvent::SceneFinished)),
        "expected SceneFinished from advance_with_action"
    );
}
```

**Step 4: Run tests**

```bash
cd /home/yjk/dev/mirror/undone && cargo fmt -p undone-scene && cargo test -p undone-scene 2>&1 | tail -15
```
Expected: all tests pass including the new one.

**Step 5: Check workspace**

```bash
cd /home/yjk/dev/mirror/undone && cargo check 2>&1 | tail -5
```
Expected: may see warnings in undone-ui about non-exhaustive `EngineEvent` match — these will be fixed in Task 5.

**Step 6: Commit**

```bash
cd /home/yjk/dev/mirror/undone && git add crates/undone-scene/src/engine.rs && git commit -m "feat(scene): add ErrorOccurred event, advance_with_action convenience method"
```

---

### Task 3: Fix Silent Stat Effects

**Files:**
- Modify: `crates/undone-scene/src/effects.rs`

**Step 1: Add `UnknownStat` to `EffectError`**

In the `EffectError` enum, add:
```rust
#[error("unknown stat '{0}'")]
UnknownStat(String),
```

**Step 2: Fix `AddStat` and `SetStat` arms**

Replace the silent-ignore pattern:

Old `AddStat`:
```rust
EffectDef::AddStat { stat, amount } => {
    if let Some(sid) = registry.get_stat(stat) {
        world.game_data.add_stat(sid, *amount);
    }
}
```

New `AddStat`:
```rust
EffectDef::AddStat { stat, amount } => {
    let sid = registry
        .get_stat(stat)
        .ok_or_else(|| EffectError::UnknownStat(stat.clone()))?;
    world.game_data.add_stat(sid, *amount);
}
```

Old `SetStat`:
```rust
EffectDef::SetStat { stat, value } => {
    if let Some(sid) = registry.get_stat(stat) {
        world.game_data.set_stat(sid, *value);
    }
}
```

New `SetStat`:
```rust
EffectDef::SetStat { stat, value } => {
    let sid = registry
        .get_stat(stat)
        .ok_or_else(|| EffectError::UnknownStat(stat.clone()))?;
    world.game_data.set_stat(sid, *value);
}
```

**Step 3: Write test**

In the test block, add:
```rust
#[test]
fn add_stat_unknown_returns_error() {
    let mut world = make_world();
    let mut ctx = SceneCtx::new();
    let reg = PackRegistry::new(); // empty registry — no stats registered
    let result = apply_effect(
        &EffectDef::AddStat {
            stat: "NONEXISTENT_STAT".into(),
            amount: 1,
        },
        &mut world,
        &mut ctx,
        &reg,
    );
    assert!(result.is_err(), "expected error for unknown stat");
    assert!(matches!(result, Err(EffectError::UnknownStat(_))));
}

#[test]
fn set_stat_unknown_returns_error() {
    let mut world = make_world();
    let mut ctx = SceneCtx::new();
    let reg = PackRegistry::new();
    let result = apply_effect(
        &EffectDef::SetStat {
            stat: "NONEXISTENT_STAT".into(),
            value: 5,
        },
        &mut world,
        &mut ctx,
        &reg,
    );
    assert!(result.is_err(), "expected error for unknown stat");
    assert!(matches!(result, Err(EffectError::UnknownStat(_))));
}
```

**Step 4: Run tests**

```bash
cd /home/yjk/dev/mirror/undone && cargo fmt -p undone-scene && cargo test -p undone-scene 2>&1 | tail -15
```
Expected: all tests pass including the two new ones.

**Step 5: Commit**

```bash
cd /home/yjk/dev/mirror/undone && git add crates/undone-scene/src/effects.rs && git commit -m "fix: AddStat/SetStat return UnknownStat error instead of silently skipping"
```

---

### Task 4: Races from Pack Data

**Files:**
- Modify: `crates/undone-packs/src/data.rs`
- Modify: `crates/undone-packs/src/manifest.rs`
- Modify: `crates/undone-packs/src/registry.rs`
- Modify: `crates/undone-packs/src/loader.rs`
- Create: `packs/base/data/races.toml`
- Modify: `packs/base/pack.toml`
- Modify: `crates/undone-ui/src/char_creation.rs`

**Step 1: Add `RacesFile` to `data.rs`**

Add at the end of the file:
```rust
#[derive(Debug, Deserialize)]
pub struct RacesFile {
    pub races: Vec<String>,
}
```

**Step 2: Add `races_file` to `PackContent` in `manifest.rs`**

Add to the `PackContent` struct:
```rust
#[serde(default)]
pub races_file: Option<String>,
```

**Step 3: Add races to `PackRegistry` in `registry.rs`**

Add a `races: Vec<String>` field:
```rust
pub struct PackRegistry {
    rodeo: Rodeo,
    pub trait_defs: HashMap<TraitId, TraitDef>,
    pub npc_trait_defs: HashMap<NpcTraitId, NpcTraitDef>,
    pub skill_defs: HashMap<SkillId, SkillDef>,
    male_names: Vec<String>,
    female_names: Vec<String>,
    races: Vec<String>,
    opening_scene: Option<String>,
    default_slot: Option<String>,
}
```

In `PackRegistry::new()`, add `races: Vec::new()`.

Add methods after `female_names()`:
```rust
pub fn register_races(&mut self, races: Vec<String>) {
    self.races.extend(races);
}

pub fn races(&self) -> &[String] {
    &self.races
}
```

**Step 4: Load races in `loader.rs`**

In `load_one_pack()`, after the stats loading block, add:
```rust
if let Some(ref races_rel) = manifest.content.races_file {
    let races_path = pack_dir.join(races_rel);
    let src = read_file(&races_path)?;
    let races_file: crate::data::RacesFile =
        toml::from_str(&src).map_err(|e| PackLoadError::Toml {
            path: races_path.clone(),
            message: e.to_string(),
        })?;
    registry.register_races(races_file.races);
}
```

**Step 5: Create `packs/base/data/races.toml`**

```toml
races = [
    "White",
    "Black",
    "Latina",
    "East Asian",
    "South Asian",
    "Mixed",
    "Other",
]
```

**Step 6: Add to `packs/base/pack.toml`**

Add under `[content]`:
```
races_file    = "data/races.toml"
```

**Step 7: Write loader test**

In `loader.rs` tests, add:
```rust
#[test]
fn loads_base_pack_races() {
    let (registry, _) = load_packs(&packs_dir()).unwrap();
    assert!(
        !registry.races().is_empty(),
        "should have loaded races from base pack"
    );
    assert!(
        registry.races().contains(&"White".to_string()),
        "should include White"
    );
}
```

**Step 8: Run packs tests**

```bash
cd /home/yjk/dev/mirror/undone && cargo fmt -p undone-packs && cargo test -p undone-packs 2>&1 | tail -15
```
Expected: all tests pass including new `loads_base_pack_races`.

**Step 9: Update `char_creation.rs` to use races from registry**

Add `race` and `before_race` to `CharFormSignals`:
```rust
race: RwSignal<String>,
before_race: RwSignal<String>,
```

In `CharFormSignals::new()`, add:
```rust
race: RwSignal::new(String::new()),
before_race: RwSignal::new(String::new()),
```

In `char_creation_view()`, after `let form = CharFormSignals::new();`, add:
```rust
// Read available races from pack registry; set form defaults.
let races_list: Vec<String> = {
    if let Some(ref pre) = *pre_state.borrow() {
        if pre.registry.races().is_empty() {
            vec!["White".to_string()]
        } else {
            pre.registry.races().to_vec()
        }
    } else {
        vec!["White".to_string()]
    }
};
if let Some(first) = races_list.first() {
    form.race.set(first.clone());
    form.before_race.set(first.clone());
}
let races_for_now = races_list.clone();
let races_for_before = races_list.clone();
```

Update `section_who_you_are` signature to accept races:
```rust
fn section_who_you_are(signals: AppSignals, form: CharFormSignals, races: Vec<String>) -> impl View {
```

Pass `races_for_now` when calling: `section_who_you_are(signals, form, races_for_now)`.

Inside `section_who_you_are`, add a race picker row after the existing rows:
```rust
form_row("Race", signals, race_picker(form.race, races, signals)),
```

Add `race_picker` function (after the existing helper functions):
```rust
fn race_picker(selection: RwSignal<String>, races: Vec<String>, signals: AppSignals) -> impl View {
    let races_signal = RwSignal::new(races);
    dyn_stack(
        move || races_signal.get(),
        |r| r.clone(),
        move |race| {
            let race_for_cmp = race.clone();
            let race_for_set = race.clone();
            let is_sel = move || selection.get() == race_for_cmp;
            let set_race = move || selection.set(race_for_set.clone());
            label(move || race.clone())
                .on_click_stop(move |_| set_race())
                .style(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    let selected = is_sel();
                    s.padding_horiz(12.0)
                        .padding_vert(6.0)
                        .margin_right(4.0)
                        .margin_bottom(4.0)
                        .border(1.0)
                        .border_radius(4.0)
                        .font_size(14.0)
                        .font_family("system-ui, -apple-system, sans-serif".to_string())
                        .cursor(floem::style::CursorStyle::Pointer)
                        .border_color(if selected { colors.lamp } else { colors.seam })
                        .color(if selected { colors.lamp } else { colors.ink })
                        .background(if selected { colors.lamp_glow } else { Color::TRANSPARENT })
                })
        },
    )
    .style(|s| s.flex_row().flex_wrap(FlexWrap::Wrap))
}
```

Update `section_your_past` to accept `before_races: Vec<String>` and add a before_race picker inside the transformation-only block (where `before_age_str` and `sexuality` are shown). Pass `races_for_before` when calling.

In the `CharCreationConfig` builder (around line 538–543), replace the hardcoded values:
```rust
race: form.race.get_untracked(),
// ...
before_race: form.before_race.get_untracked(),
```

**Step 10: Check UI compiles**

```bash
cd /home/yjk/dev/mirror/undone && cargo check -p undone-ui 2>&1 | tail -10
```
Expected: no errors.

**Step 11: Run all tests**

```bash
cd /home/yjk/dev/mirror/undone && cargo test 2>&1 | tail -10
```
Expected: all tests pass.

**Step 12: Commit**

```bash
cd /home/yjk/dev/mirror/undone && git add packs/base/data/races.toml packs/base/pack.toml crates/undone-packs/src/data.rs crates/undone-packs/src/manifest.rs crates/undone-packs/src/registry.rs crates/undone-packs/src/loader.rs crates/undone-ui/src/char_creation.rs && git commit -m "feat: races from pack data (races.toml → registry → char creation dropdown)"
```

---

### Task 5: Story Cap, ErrorOccurred Handler, free_time Fix, Dispatch Refactor

**Files:**
- Modify: `crates/undone-ui/src/lib.rs`
- Modify: `crates/undone-ui/src/left_panel.rs`

**Step 1: Handle `ErrorOccurred` in `process_events` (lib.rs)**

In `process_events()`, the `match event` block currently has 4 arms. Add a 5th:
```rust
EngineEvent::ErrorOccurred(msg) => {
    signals.story.update(|s| {
        if !s.is_empty() {
            s.push_str("\n\n");
        }
        s.push_str(&format!("[Scene error: {}]", msg));
    });
    signals.scroll_gen.update(|n| *n += 1);
}
```

**Step 2: Cap story at 200 paragraphs (lib.rs)**

In `process_events()`, replace the `ProseAdded` arm:

Old:
```rust
EngineEvent::ProseAdded(text) => {
    signals.story.update(|s| {
        if !s.is_empty() {
            s.push_str("\n\n");
        }
        s.push_str(&text);
    });
    signals.scroll_gen.update(|n| *n += 1);
}
```

New:
```rust
EngineEvent::ProseAdded(text) => {
    signals.story.update(|s| {
        if !s.is_empty() {
            s.push_str("\n\n");
        }
        s.push_str(&text);
        // Cap at 200 paragraphs to prevent unbounded growth.
        const MAX_PARAGRAPHS: usize = 200;
        let para_count = s.split("\n\n").count();
        if para_count > MAX_PARAGRAPHS {
            let drop = para_count - MAX_PARAGRAPHS;
            let mut remaining = drop;
            let mut byte_offset = 0;
            for (i, _) in s.match_indices("\n\n") {
                remaining -= 1;
                if remaining == 0 {
                    byte_offset = i + 2; // skip past "\n\n"
                    break;
                }
            }
            if byte_offset > 0 {
                *s = s[byte_offset..].to_string();
            }
        }
    });
    signals.scroll_gen.update(|n| *n += 1);
}
```

**Step 3: Fix free_time fallback in `lib.rs`**

In `app_view()`, around line 184, find:
```rust
let slot = default_slot.as_deref().unwrap_or("free_time");
if let Some(scene_id) = scheduler.pick(slot, world, registry, rng) {
    engine.send(EngineCommand::StartScene(scene_id), world, registry);
    let events = engine.drain();
    process_events(events, signals, world, fem_id);
}
```

Replace with:
```rust
if let Some(slot) = default_slot.as_deref() {
    if let Some(scene_id) = scheduler.pick(slot, world, registry, rng) {
        engine.send(EngineCommand::StartScene(scene_id), world, registry);
        let events = engine.drain();
        process_events(events, signals, world, fem_id);
    }
}
```

**Step 4: Fix `dispatch_action` in `left_panel.rs`**

Replace the entire `dispatch_action` function:

Old:
```rust
fn dispatch_action(action_id: String, state: &Rc<RefCell<GameState>>, signals: AppSignals) {
    let mut gs = state.borrow_mut();
    let GameState {
        ref mut engine,
        ref mut world,
        ref registry,
        ref scheduler,
        ref mut rng,
        ref default_slot,
        ..
    } = *gs;
    engine.send(EngineCommand::ChooseAction(action_id), world, registry);
    let events = engine.drain();
    if let Ok(femininity_id) = registry.resolve_skill("FEMININITY") {
        let finished = crate::process_events(events, signals, world, femininity_id);
        if finished {
            let slot = default_slot.as_deref().unwrap_or("free_time");
            if let Some(scene_id) = scheduler.pick(slot, world, registry, rng) {
                engine.send(EngineCommand::StartScene(scene_id), world, registry);
                let events = engine.drain();
                crate::process_events(events, signals, world, femininity_id);
            }
        }
    }
}
```

New:
```rust
fn dispatch_action(action_id: String, state: &Rc<RefCell<GameState>>, signals: AppSignals) {
    let mut gs = state.borrow_mut();
    let GameState {
        ref mut engine,
        ref mut world,
        ref registry,
        ref scheduler,
        ref mut rng,
        ref default_slot,
        ..
    } = *gs;
    if let Ok(femininity_id) = registry.resolve_skill("FEMININITY") {
        let events = engine.advance_with_action(&action_id, world, registry);
        let finished = crate::process_events(events, signals, world, femininity_id);
        if finished {
            if let Some(slot) = default_slot.as_deref() {
                if let Some(scene_id) = scheduler.pick(slot, world, registry, rng) {
                    engine.send(EngineCommand::StartScene(scene_id), world, registry);
                    let events = engine.drain();
                    crate::process_events(events, signals, world, femininity_id);
                }
            } else {
                eprintln!("[scheduler] no default_slot configured — scene finished with no next scene");
            }
        }
    }
}
```

Also remove the now-unused `EngineCommand` import if it becomes unused (keep it — `engine.send` in the scheduler path still uses it).

**Step 5: Run tests**

```bash
cd /home/yjk/dev/mirror/undone && cargo fmt -p undone-ui && cargo test 2>&1 | tail -15
```
Expected: all tests pass.

**Step 6: Commit**

```bash
cd /home/yjk/dev/mirror/undone && git add crates/undone-ui/src/lib.rs crates/undone-ui/src/left_panel.rs && git commit -m "fix: story cap (200 paragraphs), ErrorOccurred handler, free_time fallback removed, dispatch refactored to advance_with_action"
```

---

### Task 6: Keyboard Controls Redesign

**Files:**
- Modify: `crates/undone-ui/src/left_panel.rs`

**Step 1: Update imports**

Ensure these are in the imports at top of file (add if missing):
```rust
use floem::reactive::create_effect;
use crate::theme::NumberKeyMode;
```

**Step 2: Add `highlighted_idx` signal to `story_panel`**

In `pub fn story_panel(...)`, after the `hovered_detail` signal, add:
```rust
let highlighted_idx: RwSignal<Option<usize>> = RwSignal::new(None);

// Reset highlight whenever actions change (new scene step).
let hi_reset = highlighted_idx;
create_effect(move |_| {
    let _ = actions.get(); // reactive dependency
    hi_reset.set(None);
});
```

**Step 3: Rewrite the keyboard handler**

Replace the entire `let keyboard_handler = move |e: &Event| { ... };` block with:

```rust
let keyboard_handler = move |e: &Event| -> bool {
    if let Event::KeyDown(key_event) = e {
        let mode = signals.prefs.get().number_key_mode;
        let key = &key_event.key.logical_key;

        // Arrow navigation (always active regardless of mode).
        if key == &Key::Named(NamedKey::ArrowDown) {
            let len = actions.get().len();
            if len > 0 {
                highlighted_idx.update(|h| {
                    *h = Some(match *h {
                        None => 0,
                        Some(i) => (i + 1) % len,
                    });
                });
            }
            return true;
        }
        if key == &Key::Named(NamedKey::ArrowUp) {
            let len = actions.get().len();
            if len > 0 {
                highlighted_idx.update(|h| {
                    *h = Some(match *h {
                        None => len.saturating_sub(1),
                        Some(i) => if i == 0 { len - 1 } else { i - 1 },
                    });
                });
            }
            return true;
        }

        // Enter: confirm highlighted choice.
        if key == &Key::Named(NamedKey::Enter) {
            if let Some(idx) = highlighted_idx.get() {
                let current_actions = actions.get();
                if idx < current_actions.len() {
                    let action_id = current_actions[idx].id.clone();
                    drop(current_actions);
                    dispatch_action(action_id, &state_clone, signals);
                    return true;
                }
            }
        }

        // Escape: clear highlight.
        if key == &Key::Named(NamedKey::Escape) {
            highlighted_idx.set(None);
            return true;
        }

        // Number keys 1–9.
        if let Key::Character(char_str) = key {
            if let Ok(num) = char_str.parse::<usize>() {
                if num > 0 && num <= 9 {
                    let idx = num - 1;
                    let current_actions = actions.get();
                    if idx < current_actions.len() {
                        match mode {
                            NumberKeyMode::Instant => {
                                let action_id = current_actions[idx].id.clone();
                                drop(current_actions);
                                dispatch_action(action_id, &state_clone, signals);
                                return true;
                            }
                            NumberKeyMode::Confirm => {
                                if highlighted_idx.get() == Some(idx) {
                                    // Already highlighted — confirm.
                                    let action_id = current_actions[idx].id.clone();
                                    drop(current_actions);
                                    dispatch_action(action_id, &state_clone, signals);
                                } else {
                                    // Not highlighted — highlight it.
                                    highlighted_idx.set(Some(idx));
                                }
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
};
```

**Step 4: Update detail strip to show highlighted choice detail**

Replace:
```rust
let detail_strip = label(move || hovered_detail.get()).style(move |s| {
```

With:
```rust
let detail_strip = label(move || {
    // Show highlighted detail when a choice is keyboard-highlighted,
    // otherwise fall back to hovered detail.
    if let Some(idx) = highlighted_idx.get() {
        let acts = actions.get();
        if idx < acts.len() {
            return acts[idx].detail.clone();
        }
    }
    hovered_detail.get()
})
.style(move |s| {
```

**Step 5: Pass `highlighted_idx` to `choices_bar`**

Update the `choices_bar` call in `v_stack`:
```rust
choices_bar(signals, state, hovered_detail, highlighted_idx),
```

Update `choices_bar` signature:
```rust
fn choices_bar(
    signals: AppSignals,
    state: Rc<RefCell<GameState>>,
    hovered_detail: floem::reactive::RwSignal<String>,
    highlighted_idx: RwSignal<Option<usize>>,
) -> impl View {
```

**Step 6: Add highlight style to choice buttons in `choices_bar`**

Inside the `dyn_stack` callback, after computing `index`, add:
```rust
let is_highlighted = move || highlighted_idx.get() == Some(index);
```

Update the button `.style(...)` call to add a highlighted state. After the existing `.active(...)` line, before `.disabled(...)`, add:

The style closure needs to incorporate `is_highlighted`. Since floem styles are computed reactively, change the style function to:
```rust
.style(move |s| {
    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
    let highlighted = is_highlighted();
    s.margin(4.0)
        .padding_horiz(20.0)
        .padding_vert(12.0)
        .min_height(48.0)
        .border(1.0)
        .border_color(if highlighted { colors.lamp } else { colors.seam })
        .border_radius(4.0)
        .background(if highlighted { colors.lamp_glow } else { Color::TRANSPARENT })
        .items_center()
        .hover(|s| s.background(colors.lamp_glow).border_color(colors.lamp))
        .focus_visible(|s| {
            s.background(colors.lamp_glow)
                .border_color(colors.lamp)
                .outline_color(colors.lamp)
                .outline(2.0)
        })
        .active(|s| s.background(colors.lamp_glow).border_color(colors.lamp))
        .disabled(|s| s.border_color(colors.seam).color(colors.ink_ghost))
})
```

**Step 7: Check compiles**

```bash
cd /home/yjk/dev/mirror/undone && cargo fmt -p undone-ui && cargo check -p undone-ui 2>&1 | tail -10
```
Expected: no errors.

**Step 8: Run all tests**

```bash
cd /home/yjk/dev/mirror/undone && cargo test 2>&1 | tail -10
```
Expected: all tests pass.

**Step 9: Commit**

```bash
cd /home/yjk/dev/mirror/undone && git add crates/undone-ui/src/left_panel.rs && git commit -m "feat: keyboard controls redesign — arrow nav, Confirm mode, Escape, highlight style"
```

---

### Task 7: Settings Panel

**Files:**
- Create: `crates/undone-ui/src/settings_panel.rs`
- Modify: `crates/undone-ui/src/lib.rs`

**Step 1: Create `settings_panel.rs`**

```rust
use floem::peniko::Color;
use floem::prelude::*;

use crate::theme::{save_prefs, NumberKeyMode, ThemeColors, ThemeMode};
use crate::AppSignals;

pub fn settings_view(signals: AppSignals) -> impl View {
    let content = v_stack((
        settings_section_label("Theme", signals),
        theme_row(signals),
        settings_section_label("Font Size", signals),
        font_size_row(signals),
        settings_section_label("Line Height", signals),
        line_height_row(signals),
        settings_section_label("Number Key Mode", signals),
        number_key_mode_row(signals),
    ))
    .style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.width_full()
            .max_width(480.0)
            .padding_horiz(40.0)
            .padding_vert(32.0)
            .gap(8.0)
            .color(colors.ink)
    });

    let centered = container(content).style(|s| s.width_full().flex_row().justify_center());

    scroll(centered)
        .scroll_style(|s| s.shrink_to_fit())
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.size_full().background(colors.page)
        })
}

fn settings_section_label(text: &'static str, signals: AppSignals) -> impl View {
    label(move || text.to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.font_size(12.0)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
            .color(colors.ink_ghost)
            .margin_top(16.0)
            .margin_bottom(4.0)
    })
}

fn theme_row(signals: AppSignals) -> impl View {
    let btn = |mode: ThemeMode, label_text: &'static str| {
        let is_active = move || signals.prefs.get().mode == mode;
        label(move || label_text.to_string())
            .on_click_stop(move |_| {
                signals.prefs.update(|p| p.mode = mode);
                save_prefs(&signals.prefs.get());
            })
            .style(move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                let active = is_active();
                s.padding_horiz(16.0)
                    .padding_vert(8.0)
                    .margin_right(4.0)
                    .border(1.0)
                    .border_radius(4.0)
                    .font_size(14.0)
                    .font_family("system-ui, -apple-system, sans-serif".to_string())
                    .cursor(floem::style::CursorStyle::Pointer)
                    .border_color(if active { colors.lamp } else { colors.seam })
                    .color(if active { colors.lamp } else { colors.ink })
                    .background(if active { colors.lamp_glow } else { Color::TRANSPARENT })
                    .hover(|s| s.background(colors.lamp_glow).border_color(colors.lamp))
            })
    };

    h_stack((
        btn(ThemeMode::Light, "Warm"),
        btn(ThemeMode::Sepia, "Sepia"),
        btn(ThemeMode::Dark, "Night"),
    ))
}

fn font_size_row(signals: AppSignals) -> impl View {
    let dec = move || {
        signals.prefs.update(|p| {
            if p.font_size > 14 {
                p.font_size -= 1;
            }
        });
        save_prefs(&signals.prefs.get());
    };
    let inc = move || {
        signals.prefs.update(|p| {
            if p.font_size < 24 {
                p.font_size += 1;
            }
        });
        save_prefs(&signals.prefs.get());
    };

    h_stack((
        stepper_button("−", dec, signals),
        label(move || format!("{}", signals.prefs.get().font_size)).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.width(40.0)
                .text_align(floem::style::TextAlign::Center)
                .font_size(15.0)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
                .color(colors.ink)
        }),
        stepper_button("+", inc, signals),
    ))
    .style(|s| s.items_center())
}

fn line_height_row(signals: AppSignals) -> impl View {
    let dec = move || {
        signals.prefs.update(|p| {
            let next = (p.line_height * 10.0 - 1.0).round() / 10.0;
            if next >= 1.2 {
                p.line_height = next;
            }
        });
        save_prefs(&signals.prefs.get());
    };
    let inc = move || {
        signals.prefs.update(|p| {
            let next = (p.line_height * 10.0 + 1.0).round() / 10.0;
            if next <= 2.0 {
                p.line_height = next;
            }
        });
        save_prefs(&signals.prefs.get());
    };

    h_stack((
        stepper_button("−", dec, signals),
        label(move || format!("{:.1}", signals.prefs.get().line_height)).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.width(40.0)
                .text_align(floem::style::TextAlign::Center)
                .font_size(15.0)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
                .color(colors.ink)
        }),
        stepper_button("+", inc, signals),
    ))
    .style(|s| s.items_center())
}

fn number_key_mode_row(signals: AppSignals) -> impl View {
    let btn = |mode: NumberKeyMode, label_text: &'static str| {
        let is_active = move || signals.prefs.get().number_key_mode == mode;
        label(move || label_text.to_string())
            .on_click_stop(move |_| {
                signals.prefs.update(|p| p.number_key_mode = mode);
                save_prefs(&signals.prefs.get());
            })
            .style(move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                let active = is_active();
                s.padding_horiz(16.0)
                    .padding_vert(8.0)
                    .margin_right(4.0)
                    .border(1.0)
                    .border_radius(4.0)
                    .font_size(14.0)
                    .font_family("system-ui, -apple-system, sans-serif".to_string())
                    .cursor(floem::style::CursorStyle::Pointer)
                    .border_color(if active { colors.lamp } else { colors.seam })
                    .color(if active { colors.lamp } else { colors.ink })
                    .background(if active { colors.lamp_glow } else { Color::TRANSPARENT })
                    .hover(|s| s.background(colors.lamp_glow).border_color(colors.lamp))
            })
    };

    h_stack((
        btn(NumberKeyMode::Instant, "Instant"),
        btn(NumberKeyMode::Confirm, "Confirm"),
    ))
}

fn stepper_button(
    text: &'static str,
    action: impl Fn() + 'static,
    signals: AppSignals,
) -> impl View {
    label(move || text.to_string())
        .on_click_stop(move |_| action())
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.width(32.0)
                .height(32.0)
                .items_center()
                .justify_center()
                .border(1.0)
                .border_radius(4.0)
                .font_size(16.0)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
                .border_color(colors.seam)
                .color(colors.ink)
                .cursor(floem::style::CursorStyle::Pointer)
                .hover(|s| s.background(colors.lamp_glow).border_color(colors.lamp))
        })
}
```

**Step 2: Register the module in `lib.rs`**

Add `pub mod settings_panel;` to the module list at top of `lib.rs`.

Add the import: `use crate::settings_panel::settings_view;`

**Step 3: Wire Settings tab in `lib.rs`**

Replace the placeholder:
```rust
AppTab::Settings => {
    placeholder_panel("Settings \u{2014} coming soon", signals).into_any()
}
```

With:
```rust
AppTab::Settings => settings_view(signals).into_any(),
```

**Step 4: Check compiles**

```bash
cd /home/yjk/dev/mirror/undone && cargo fmt -p undone-ui && cargo check -p undone-ui 2>&1 | tail -10
```
Expected: no errors. Fix any API mismatches (e.g., `text_align` field name, `gap` support in floem 0.2).

> **Note on floem 0.2 API:** If `gap()` is unavailable use `margin_bottom` on child elements instead. If `text_align` is not available as a style method, remove it (centering a label in a fixed-width container can be done with `justify_center()` on the parent). Check `cargo check` output and adapt accordingly — do not fight the API; use what compiles.

**Step 5: Run all tests**

```bash
cd /home/yjk/dev/mirror/undone && cargo test 2>&1 | tail -10
```
Expected: all previous tests still pass. Settings panel has no unit tests (UI only).

**Step 6: Commit**

```bash
cd /home/yjk/dev/mirror/undone && git add crates/undone-ui/src/settings_panel.rs crates/undone-ui/src/lib.rs && git commit -m "feat: settings tab UI — theme, font size, line height, number key mode controls"
```

---

### Final: Run full test suite and update HANDOFF

**Step 1: Full test run**

```bash
cd /home/yjk/dev/mirror/undone && cargo test 2>&1 | tail -20
```
Expected: all tests pass, 0 failures.

**Step 2: Check for warnings**

```bash
cd /home/yjk/dev/mirror/undone && cargo build 2>&1 | grep "^warning" | head -20
```
Fix any warnings about unused imports or dead code introduced by these changes.

**Step 3: Update HANDOFF.md**

- Remove all 6 audit items from "Remaining audit findings"
- Move keyboard controls and settings tab from "Next Action" to done
- Add session log entry
- Update "Current State" to reflect new test count

**Step 4: Commit HANDOFF**

```bash
cd /home/yjk/dev/mirror/undone && git add HANDOFF.md && git commit -m "docs: update HANDOFF — engineering tasks complete"
```
