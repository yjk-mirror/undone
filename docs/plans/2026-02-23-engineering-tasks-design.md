# Engineering Tasks Design — 2026-02-23

## Scope

Three engineering tasks, no creative/content work:

1. Keyboard controls redesign
2. Settings tab UI
3. Audit fixes (6 items)

---

## 1. Keyboard Controls Redesign

### New types (`theme.rs`)

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NumberKeyMode {
    Instant,  // current behavior — number fires immediately
    Confirm,  // press to highlight, press again or Enter to confirm
}
```

`UserPrefs` gains: `pub number_key_mode: NumberKeyMode` (default `Instant`).
Persisted to `prefs.json` alongside existing fields.

### Local highlight state (`story_panel.rs`)

```rust
let highlighted_idx: RwSignal<Option<usize>> = RwSignal::new(None);
```

Reset to `None` whenever the `actions` signal changes (new scene step).

### Keyboard handler logic

| Key | Instant mode | Confirm mode |
|-----|-------------|-------------|
| `ArrowDown` | move highlight forward (wrapping) | move highlight forward (wrapping) |
| `ArrowUp` | move highlight backward (wrapping) | move highlight backward (wrapping) |
| `1`–`9` | dispatch immediately | if highlighted_idx == Some(N-1) → dispatch; else → set highlight to N-1 |
| `Enter` | dispatch highlighted (if any) | dispatch highlighted (if any) |
| `Escape` | clear highlight | clear highlight |

### Choice button rendering

Each button receives `is_highlighted: bool` derived from `highlighted_idx`. Highlighted button uses the same visual as keyboard-focus/hover (distinct border + background). Detail strip shows highlighted choice's `detail` text (currently hover-only — unify with highlight).

---

## 2. Settings Tab UI

### New file

`crates/undone-ui/src/settings_panel.rs` — parallel to `story_panel.rs` / `sidebar_panel.rs`.

### UserPrefs changes

Only addition: `number_key_mode: NumberKeyMode` (from Section 1). `font_size` and `line_height` already exist.

### Controls

| Setting | Widget | Range / Options |
|---------|--------|----------------|
| Theme mode | 3-button row | Warm / Sepia / Night |
| Font size | Label + `−`/`+` buttons | 14–24, step 1 |
| Line height | Label + `−`/`+` buttons | 1.2–2.0, step 0.1 |
| Number key mode | 2-button toggle | Instant / Confirm |

All controls read/write `prefs: RwSignal<UserPrefs>` from `AppSignals`. Every change immediately calls `save_prefs()`. No save button — instant reactivity.

### Wiring

Settings tab already sets `AppTab::Settings` in `title_bar.rs`. Add a `Settings` arm in `lib.rs` `dyn_container` that renders `settings_panel::settings_view(signals)`.

---

## 3. Audit Fixes

### 3a. Silent stat effects (`effects.rs`)

`AddStat` and `SetStat` currently silently skip unknown stat IDs. Fix: return `Err(EffectError::UnknownStat(stat.clone()))` — same pattern as `SkillIncrease` already uses for unknown skills.

### 3b. Unbounded story string (`lib.rs`)

`story: RwSignal<String>` grows without bound. Fix: cap at **200 paragraphs**. After each `ProseAdded`, if paragraph count (split on `\n\n`) exceeds 200, trim oldest from the front before updating the signal.

### 3c. `free_time` fallback (`story_panel.rs`)

Hardcoded `"free_time"` string survives as fallback. Fix: use the registry's `opening_slot` value (already in `PreGameState`). If missing, log an error and push a visible `EngineEvent::ErrorOccurred` rather than dispatching a bad slot name.

### 3d. Hardcoded race defaults (pack system + `char_creation.rs`)

- Add `packs/base/races.toml`: `races = ["White", "Black", "Latina", "East Asian", "South Asian", "Mixed", "Other"]`
- `PackManifest` gains `races: Vec<String>` loaded from `races.toml`
- `PackRegistry` exposes `fn races(&self) -> &[String]`
- `CharCreation` populates a dropdown from `registry.races()`; default is first entry

### 3e. Scheduler failure without UI feedback (`lib.rs`)

Scheduler errors go to `stderr` only. Fix: add `EngineEvent::ErrorOccurred(String)`. When scheduler returns an error, emit this event. `process_events()` renders it as a visible italic/error-styled line appended to the story panel: `[Scene error: ...]`.

### 3f. Action dispatch in UI crate (`story_panel.rs` → `undone-scene`)

`dispatch_action()` lives in the UI crate and directly calls scene engine internals. Fix: move to `undone-scene` as:

```rust
impl SceneEngine {
    pub fn advance_with_action(&mut self, action_id: &str) -> Vec<EngineEvent>;
}
```

UI calls this method and processes returned events. No engine internals exposed to the UI crate.

---

## Files Affected

| File | Change |
|------|--------|
| `crates/undone-ui/src/theme.rs` | Add `NumberKeyMode` enum, add field to `UserPrefs` |
| `crates/undone-ui/src/story_panel.rs` | Keyboard handler rewrite, highlighted_idx signal, choice button highlight |
| `crates/undone-ui/src/settings_panel.rs` | New file — settings view |
| `crates/undone-ui/src/lib.rs` | Wire Settings tab, story cap logic, ErrorOccurred handler |
| `crates/undone-scene/src/effects.rs` | UnknownStat error on AddStat/SetStat |
| `crates/undone-scene/src/engine.rs` | Add `advance_with_action()` public method |
| `crates/undone-domain/src/lib.rs` | Add `ErrorOccurred(String)` to `EngineEvent` |
| `crates/undone-packs/src/lib.rs` | Load `races.toml`, expose `races()` on registry |
| `packs/base/races.toml` | New file — race list |
