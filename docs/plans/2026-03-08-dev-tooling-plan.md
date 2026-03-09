# Dev Tooling Suite — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use ops:executing-plans to implement this plan task-by-task.

**Goal:** Build dev tools that eliminate manual clicking through the game for testing — debug mode with scene jumper, stat bounds enforcement, schedule reachability checker, and scene distribution simulator.

**Architecture:** Runtime `--dev` flag adds a Dev tab to the UI with scene jumping, stat editing, flag management, and state inspection. File-based IPC lets MCP agents send dev commands to the running game. `BoundedStat` newtype enforces stress/anxiety bounds at the type level. `validate-pack` gains static reachability analysis and Monte Carlo scheduling simulation.

**Tech Stack:** Rust, floem 0.2.0, rmcp (MCP framework), serde_json (IPC protocol)

**Design doc:** `docs/plans/2026-03-08-dev-tooling-design.md`

---

## Phase 1: BoundedStat Newtype

### Task 1: BoundedStat type with tests

**Files:**
- Create: `crates/undone-domain/src/bounded_stat.rs`
- Modify: `crates/undone-domain/src/lib.rs` (add `pub mod bounded_stat; pub use bounded_stat::BoundedStat;`)

**Step 1: Write the tests and type**

Create `crates/undone-domain/src/bounded_stat.rs`:

```rust
use serde::{Deserialize, Serialize};

/// A stat clamped to [0, 100]. Used for stress and anxiety.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BoundedStat(i32);

impl BoundedStat {
    pub const MIN: i32 = 0;
    pub const MAX: i32 = 100;

    pub fn new(value: i32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(self) -> i32 {
        self.0
    }

    pub fn apply_delta(&mut self, delta: i32) {
        self.0 = (self.0 + delta).clamp(Self::MIN, Self::MAX);
    }
}

impl Default for BoundedStat {
    fn default() -> Self {
        Self(0)
    }
}

impl std::fmt::Display for BoundedStat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clamps_to_min() {
        assert_eq!(BoundedStat::new(-50).get(), 0);
    }

    #[test]
    fn new_clamps_to_max() {
        assert_eq!(BoundedStat::new(200).get(), 100);
    }

    #[test]
    fn new_preserves_valid_value() {
        assert_eq!(BoundedStat::new(42).get(), 42);
    }

    #[test]
    fn apply_delta_positive() {
        let mut s = BoundedStat::new(50);
        s.apply_delta(30);
        assert_eq!(s.get(), 80);
    }

    #[test]
    fn apply_delta_negative() {
        let mut s = BoundedStat::new(50);
        s.apply_delta(-30);
        assert_eq!(s.get(), 20);
    }

    #[test]
    fn apply_delta_clamps_floor() {
        let mut s = BoundedStat::new(10);
        s.apply_delta(-50);
        assert_eq!(s.get(), 0);
    }

    #[test]
    fn apply_delta_clamps_ceiling() {
        let mut s = BoundedStat::new(90);
        s.apply_delta(50);
        assert_eq!(s.get(), 100);
    }

    #[test]
    fn default_is_zero() {
        assert_eq!(BoundedStat::default().get(), 0);
    }

    #[test]
    fn serde_roundtrips_as_bare_i32() {
        let stat = BoundedStat::new(42);
        let json = serde_json::to_string(&stat).unwrap();
        assert_eq!(json, "42");
        let back: BoundedStat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, stat);
    }

    #[test]
    fn serde_clamps_on_deserialize() {
        let back: BoundedStat = serde_json::from_str("-10").unwrap();
        // Note: serde(transparent) deserializes the raw i32 without clamping.
        // If we want clamping on deserialize, we need a custom Deserialize.
        // For now, document that save files with out-of-range values will
        // load the raw value. The next apply_delta will re-clamp.
        // If this is a problem, add #[serde(deserialize_with = "...")] later.
        assert_eq!(back.get(), -10); // transparent: no clamp on deser
    }
}
```

**Step 2: Run tests**

```bash
cargo test -p undone-domain bounded_stat
```

Expected: ALL PASS. Note: the `serde_clamps_on_deserialize` test documents current behavior — transparent serde does NOT clamp. If save files with out-of-range values are a concern, add a custom deserializer later.

**Step 3: Add module to lib.rs**

In `crates/undone-domain/src/lib.rs`, add:

```rust
pub mod bounded_stat;
pub use bounded_stat::BoundedStat;
```

**Step 4: Run full domain tests**

```bash
cargo test -p undone-domain
```

Expected: ALL PASS

**Step 5: Commit**

```bash
git add crates/undone-domain/src/bounded_stat.rs crates/undone-domain/src/lib.rs
git commit -m "feat(domain): add BoundedStat newtype for clamped 0-100 stats"
```

---

### Task 2: Migrate Player.stress and Player.anxiety to BoundedStat

**Files:**
- Modify: `crates/undone-domain/src/player.rs:96-98` (change field types)
- Modify: `crates/undone-scene/src/effects.rs:139-147` (remove .max(0), use apply_delta)
- Modify: `crates/undone-scene/src/template_ctx.rs` (use .get() for getStress/getAnxiety)
- Modify: `crates/undone-ui/src/lib.rs` (PlayerSnapshot reads .get())
- Modify: ALL test files that construct Player structs (search for `stress:` and `anxiety:`)

**Step 1: Change Player field types**

In `crates/undone-domain/src/player.rs`, change:

```rust
// Before:
pub money: i32,
pub stress: i32,
pub anxiety: i32,

// After:
pub money: i32,
pub stress: BoundedStat,
pub anxiety: BoundedStat,
```

Add `use crate::BoundedStat;` at the top if not already imported.

**Step 2: Fix effects.rs**

In `crates/undone-scene/src/effects.rs`, change:

```rust
// Before (lines 139-147):
EffectDef::ChangeStress { amount } => {
    world.player.stress = (world.player.stress + amount).max(0);
}
EffectDef::ChangeMoney { amount } => {
    world.player.money += amount;
}
EffectDef::ChangeAnxiety { amount } => {
    world.player.anxiety = (world.player.anxiety + amount).max(0);
}

// After:
EffectDef::ChangeStress { amount } => {
    world.player.stress.apply_delta(*amount);
}
EffectDef::ChangeMoney { amount } => {
    world.player.money += amount;
}
EffectDef::ChangeAnxiety { amount } => {
    world.player.anxiety.apply_delta(*amount);
}
```

**Step 3: Fix template_ctx.rs**

In `crates/undone-scene/src/template_ctx.rs`, wherever `self.stress` and `self.anxiety` are read, change to `self.stress.get()` and `self.anxiety.get()`. The `PlayerCtx` struct fields at lines 31-33 need their types updated too:

```rust
// In PlayerCtx struct:
pub stress: i32,    // keep as i32 — PlayerCtx is a display snapshot
pub anxiety: i32,

// In the constructor (render_prose, ~lines 361-363):
stress: world.player.stress.get(),
anxiety: world.player.anxiety.get(),
```

If `PlayerCtx` already stores `i32` snapshots (check first!), only the constructor lines that read from `world.player` need `.get()`. If `PlayerCtx` stores raw field references, update accordingly.

**Step 4: Fix PlayerSnapshot in lib.rs**

In `crates/undone-ui/src/lib.rs`, `PlayerSnapshot::from_player()` at line ~120:

```rust
// Change:
stress: p.stress,
anxiety: p.anxiety,

// To:
stress: p.stress.get(),
anxiety: p.anxiety.get(),
```

**Step 5: Fix ALL test Player construction sites**

Search the entire workspace for `stress:` in test code. Every `Player { ... stress: 0, ... }` must become `stress: BoundedStat::new(0)`. Same for `anxiety:`.

Key files to check:
- `crates/undone-scene/src/scheduler.rs` (tests at bottom)
- `crates/undone-scene/src/effects.rs` (tests)
- `crates/undone-ui/src/lib.rs` (tests)
- `crates/undone-ui/src/game_state.rs` (tests)
- `crates/undone-ui/src/char_creation.rs` (if any test helpers)

Use: `cargo check 2>&1` to find all remaining type errors.

**Step 6: Run full test suite**

```bash
cargo test
```

Expected: ALL PASS (262 tests)

**Step 7: Commit**

```bash
git add -A
git commit -m "refactor: migrate stress/anxiety to BoundedStat, remove .max(0) band-aids"
```

---

## Phase 2: CLI Arg Parsing + Quick Start

### Task 3: CLI arg parsing in main.rs

**Files:**
- Modify: `src/main.rs` (add arg parsing, pass flags to UI)
- Modify: `crates/undone-ui/src/lib.rs` (accept dev_mode + quick_start params)

**Step 1: Add arg parsing to main.rs**

Simple manual parsing — no clap dependency needed for two flags.

In `src/main.rs`, before the `Application::new()` call:

```rust
let args: Vec<String> = std::env::args().collect();
let dev_mode = args.iter().any(|a| a == "--dev");
let quick_start = args.iter().any(|a| a == "--quick");

if quick_start && !dev_mode {
    eprintln!("--quick requires --dev");
    return;
}
```

**Step 2: Pass flags to app_view**

Change `app_view()` signature to accept the flags:

```rust
// In lib.rs, change:
pub fn app_view() -> impl View

// To:
pub fn app_view(dev_mode: bool, quick_start: bool) -> impl View
```

Update the call site in `main.rs`:

```rust
move |_| undone_ui::app_view(dev_mode, quick_start),
```

**Step 3: Store dev_mode on GameState and PreGameState**

Add `pub dev_mode: bool` to both `PreGameState` and `GameState` in `game_state.rs`. Thread it through `init_game()` → `start_game()` → `GameState`.

Actually, simpler: just pass `dev_mode` as a local variable in `app_view()` — it's a `Copy` bool that closures can capture. No need to store it on GameState unless the IPC polling needs it (it will — add it to GameState).

```rust
// In game_state.rs, add to GameState:
pub dev_mode: bool,
```

Set it in `start_game()` and `start_loaded_game()`:

```rust
pub fn start_game(pre: PreGameState, config: CharCreationConfig, dev_mode: bool) -> GameState {
    // ... existing code ...
    GameState {
        // ... existing fields ...
        dev_mode,
    }
}
```

**Step 4: Run full tests**

```bash
cargo test
```

Expected: ALL PASS (fix any call sites that now need the new parameter)

**Step 5: Commit**

```bash
git add src/main.rs crates/undone-ui/src/lib.rs crates/undone-ui/src/game_state.rs
git commit -m "feat: add --dev and --quick CLI flags with plumbing to UI"
```

---

### Task 4: Quick start — skip char creation with Robin preset

**Files:**
- Modify: `crates/undone-ui/src/lib.rs` (handle quick_start in app_view)
- Modify: `crates/undone-ui/src/char_creation.rs` (extract preset config builder)

**Step 1: Extract a `robin_quick_config()` function**

In `crates/undone-ui/src/char_creation.rs`, the Robin preset data is in `PRESET_ROBIN` (lines 74-168). The `fem_form_defaults()` and the "Next" button handler already build a `CharCreationConfig` from preset data. Extract a public function that builds a complete `CharCreationConfig` for Robin without UI interaction:

```rust
/// Build a complete CharCreationConfig for the Robin preset.
/// Used by --quick start and the Dev tab's quick-start button.
pub fn robin_quick_config(registry: &PackRegistry) -> CharCreationConfig {
    let preset = &PRESET_ROBIN;
    let trait_ids: Vec<TraitId> = preset
        .trait_ids
        .iter()
        .filter_map(|id| registry.resolve_trait(id).ok())
        .collect();

    CharCreationConfig {
        name_fem: preset.name_fem.to_string(),
        name_masc: preset.name_masc.to_string(),
        age: preset.age,
        race: preset.race.to_string(),
        figure: preset.figure,
        breasts: preset.breasts,
        origin: preset.origin,
        before: Some(BeforeIdentity {
            name: preset.before_name.to_string(),
            age: preset.before_age,
            race: preset.before_race.to_string(),
            sexuality: preset.before_sexuality,
            figure: preset.before_figure,
            height: preset.before_height,
            hair_colour: preset.before_hair_colour,
            eye_colour: preset.before_eye_colour,
            skin_tone: preset.before_skin_tone,
            penis_size: preset.before_penis_size,
            voice: preset.before_voice,
            traits: HashSet::new(),
        }),
        starting_traits: trait_ids,
        male_count: 6,
        female_count: 3,
        starting_flags: preset.starting_flags.iter().map(|s| s.to_string()).collect(),
        starting_arc_states: HashMap::new(),
        height: preset.height,
        butt: preset.butt,
        waist: preset.waist,
        lips: preset.lips,
        hair_colour: preset.hair_colour,
        hair_length: preset.hair_length,
        eye_colour: preset.eye_colour,
        skin_tone: preset.skin_tone,
        complexion: preset.complexion,
        appearance: preset.appearance,
        pubic_hair: preset.pubic_hair,
        natural_pubic_hair: preset.natural_pubic_hair,
        nipple_sensitivity: preset.nipple_sensitivity,
        clit_sensitivity: preset.clit_sensitivity,
        inner_labia: preset.inner_labia,
        wetness_baseline: preset.wetness_baseline,
    }
}
```

**Step 2: Handle quick_start in app_view**

In `crates/undone-ui/src/lib.rs` `app_view()`, right after `init_game()` loads packs, if `quick_start` is true:

```rust
if quick_start {
    if let Some(pre) = pre_state.borrow_mut().take() {
        let config = crate::char_creation::robin_quick_config(&pre.registry);
        let gs = crate::game_state::start_game(pre, config, true);
        *game_state.borrow_mut() = Some(gs);
        signals.phase.set(AppPhase::InGame);
    }
}
```

This skips Landing, BeforeCreation, TransformationIntro, and FemCreation — goes straight to InGame.

**Step 3: Run tests**

```bash
cargo test
```

Expected: ALL PASS

**Step 4: Manual smoke test**

```bash
cargo run --release --bin undone -- --dev --quick
```

Expected: Game window opens directly in InGame phase with Robin's world. No char creation screens. The first scheduled scene (workplace_arrival) should start automatically.

**Step 5: Commit**

```bash
git add crates/undone-ui/src/char_creation.rs crates/undone-ui/src/lib.rs
git commit -m "feat: --dev --quick skips char creation, starts with Robin preset"
```

---

## Phase 3: Dev Tab UI

### Task 5: AppTab::Dev variant and title bar integration

**Files:**
- Modify: `crates/undone-ui/src/lib.rs` (add `AppTab::Dev`)
- Modify: `crates/undone-ui/src/title_bar.rs` (show Dev tab when dev_mode)

**Step 1: Add Dev variant to AppTab**

In `crates/undone-ui/src/lib.rs`:

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppTab {
    Game,
    Saves,
    Settings,
    Dev,  // Only shown when --dev flag is active
}
```

**Step 2: Store dev_mode on AppSignals**

Add `pub dev_mode: bool` to `AppSignals` (not a signal — a plain bool, since it never changes). Thread it through.

Actually, simpler: pass `dev_mode` as a separate parameter to `title_bar()`. The title bar just needs to know whether to render the Dev tab button.

**Step 3: Add Dev tab button to title_bar.rs**

Read `title_bar.rs` first to understand the tab button pattern. Add a "Dev" button conditionally:

```rust
// In title_bar(), after the Settings tab button, if dev_mode:
if dev_mode {
    // Add "Dev" tab button using same pattern as Game/Saves/Settings
}
```

The exact code depends on the title_bar implementation — read it first, match the pattern.

**Step 4: Route AppTab::Dev in app_view**

In the `dyn_container` match on `signals.tab.get()` inside the InGame phase, add:

```rust
AppTab::Dev => dev_panel(signals, Rc::clone(&gs_cell)).into_any(),
```

For now, create a placeholder `dev_panel`:

```rust
fn dev_panel(signals: AppSignals, _gs: Rc<RefCell<GameState>>) -> impl View {
    container(label(move || "Dev Panel — Coming Soon".to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.color(colors.ink).font_size(16.0)
    }))
    .style(|s| s.size_full().items_center().justify_center())
}
```

**Step 5: Build and verify**

```bash
cargo run --release --bin undone -- --dev --quick
```

Expected: Dev tab visible in title bar. Clicking it shows placeholder text. Without `--dev`, the tab does not appear.

**Step 6: Commit**

```bash
git add crates/undone-ui/src/lib.rs crates/undone-ui/src/title_bar.rs
git commit -m "feat: add Dev tab to title bar, visible only with --dev flag"
```

---

### Task 6: Dev panel — scene jumper

**Files:**
- Create: `crates/undone-ui/src/dev_panel.rs`
- Modify: `crates/undone-ui/src/lib.rs` (replace placeholder with real module)

**Step 1: Create dev_panel.rs with scene jumper**

The scene jumper needs:
- A text input for filtering scene IDs
- A scrollable list of matching scene IDs
- Click a scene ID → jump to it

The scene list comes from `GameState.engine` which holds the scenes HashMap. Add a method to `SceneEngine` to list scene IDs if one doesn't exist:

```rust
// In crates/undone-scene/src/engine.rs, add:
pub fn scene_ids(&self) -> Vec<String> {
    let mut ids: Vec<String> = self.scenes.keys().cloned().collect();
    ids.sort();
    ids
}
```

The dev panel view:

```rust
pub fn dev_panel(signals: AppSignals, gs: Rc<RefCell<GameState>>) -> impl View {
    let filter = RwSignal::new(String::new());

    // Scene list — filtered by search text
    let gs_scenes = Rc::clone(&gs);
    let scene_list = dyn_container(
        move || filter.get(),
        move |filter_text| {
            let gs = gs_scenes.borrow();
            let all_ids = gs.as_ref().map(|g| g.engine.scene_ids()).unwrap_or_default();
            let filtered: Vec<String> = if filter_text.is_empty() {
                all_ids
            } else {
                let lower = filter_text.to_lowercase();
                all_ids.into_iter().filter(|id| id.to_lowercase().contains(&lower)).collect()
            };
            // Build clickable list of scene IDs
            // Each item: label with click handler that calls jump_to_scene
            // ... (build v_stack of clickable labels)
        }
    );

    // Layout: search field at top, scrollable scene list below
    // ... (standard floem layout)
}
```

The click handler for jumping to a scene:

```rust
fn jump_to_scene(gs: &Rc<RefCell<GameState>>, signals: AppSignals, scene_id: String) {
    let mut gs = gs.borrow_mut();
    if let Some(ref mut g) = *gs {
        // Reset engine state
        g.engine.reset_runtime();
        // Start the selected scene
        crate::start_scene(&mut g.engine, &mut g.world, &g.registry, scene_id);
        let events = g.engine.drain();
        crate::process_events(events, signals, &g.world, g.femininity_id);
        // Switch to Game tab to see the scene
        signals.tab.set(AppTab::Game);
    }
}
```

**Important:** Read `crates/undone-ui/src/left_panel.rs` and `right_panel.rs` for the existing floem patterns (text input, scroll, clickable items) before implementing. Match the existing style.

**Step 2: Wire into lib.rs**

Add `pub mod dev_panel;` to `crates/undone-ui/src/lib.rs`. Replace the placeholder `dev_panel` function with `use crate::dev_panel::dev_panel;`.

**Step 3: Build and test**

```bash
cargo run --release --bin undone -- --dev --quick
```

Expected: Dev tab shows search field + scene list. Typing filters. Clicking a scene ID jumps to it and switches to Game tab. Scene prose renders correctly.

**Step 4: Commit**

```bash
git add crates/undone-ui/src/dev_panel.rs crates/undone-ui/src/lib.rs crates/undone-scene/src/engine.rs
git commit -m "feat: dev panel scene jumper with search and click-to-jump"
```

---

### Task 7: Dev panel — stat editors + flag editor + state inspector

**Files:**
- Modify: `crates/undone-ui/src/dev_panel.rs`

**Step 1: Add stat editors**

Below the scene jumper, add editable fields for FEMININITY, stress, anxiety, money. Each is a label + text input + "Set" button pattern.

When "Set" is clicked:
- Parse the text input as i32
- For stress/anxiety: `world.player.stress = BoundedStat::new(value)`
- For money: `world.player.money = value`
- For FEMININITY: `world.player.set_skill(femininity_id, value)` (or direct skills map update)
- Refresh the player snapshot signal

**Step 2: Add flag editor**

- Text input + "Add Flag" button → `world.game_data.set_flag(text)`
- List of current flags with "×" delete buttons → `world.game_data.remove_flag(flag)`
- Current flags from `world.game_data.flags` (it's a `HashSet<String>`)

**Step 3: Add state inspector**

Read-only display section showing:
- Current week/day/timeslot from `world.game_data`
- Arc states from `world.game_data.arc_states`
- NPC names + liking levels from `world.male_npcs` and `world.female_npcs`

**Step 4: Add quick action buttons**

- "Advance 1 Week" → call `world.game_data.advance_time_slot()` 28 times (4 slots × 7 days)
- "Set All NPC Liking → Close" → iterate all NPCs, set `core.npc_liking = LikingLevel::Close`

**Step 5: Build and test**

```bash
cargo run --release --bin undone -- --dev --quick
```

Expected: All editors work. Setting stress to 150 clamps to 100. Adding/removing flags updates the list. State inspector shows current game state. Quick actions modify state and refresh display.

**Step 6: Commit**

```bash
git add crates/undone-ui/src/dev_panel.rs
git commit -m "feat: dev panel stat editors, flag editor, state inspector, quick actions"
```

---

## Phase 4: File-Based IPC for MCP

### Task 8: Dev command protocol types

**Files:**
- Create: `crates/undone-ui/src/dev_ipc.rs`

**Step 1: Define the command and response types**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum DevCommand {
    QuickStart { preset: Option<String> },
    JumpToScene { scene_id: String },
    SetStat { stat: String, value: i32 },
    SetFlag { flag: String },
    RemoveFlag { flag: String },
    GetState,
    AdvanceTime { weeks: u32 },
    SetNpcLiking { npc_name: String, level: String },
}

#[derive(Debug, Serialize)]
pub struct DevResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl DevResponse {
    pub fn ok(message: impl Into<String>) -> Self {
        Self { success: true, message: message.into(), data: None }
    }
    pub fn ok_with_data(message: impl Into<String>, data: serde_json::Value) -> Self {
        Self { success: true, message: message.into(), data: Some(data) }
    }
    pub fn err(message: impl Into<String>) -> Self {
        Self { success: false, message: message.into(), data: None }
    }
}
```

**Step 2: Write unit tests for serde round-trip**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_jump_to_scene() {
        let json = r#"{"command": "jump_to_scene", "scene_id": "base::rain_shelter"}"#;
        let cmd: DevCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, DevCommand::JumpToScene { scene_id } if scene_id == "base::rain_shelter"));
    }

    #[test]
    fn deserialize_set_stat() {
        let json = r#"{"command": "set_stat", "stat": "stress", "value": 50}"#;
        let cmd: DevCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, DevCommand::SetStat { stat, value } if stat == "stress" && value == 50));
    }

    #[test]
    fn deserialize_get_state() {
        let json = r#"{"command": "get_state"}"#;
        let cmd: DevCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, DevCommand::GetState));
    }

    #[test]
    fn serialize_response_ok() {
        let resp = DevResponse::ok("done");
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"success\":true"));
    }
}
```

**Step 3: Run tests**

```bash
cargo test -p undone-ui dev_ipc
```

Expected: ALL PASS

**Step 4: Commit**

```bash
git add crates/undone-ui/src/dev_ipc.rs crates/undone-ui/src/lib.rs
git commit -m "feat: dev IPC command/response protocol types"
```

---

### Task 9: Game-side IPC polling and command execution

**Files:**
- Modify: `crates/undone-ui/src/dev_ipc.rs` (add polling + execute)
- Modify: `crates/undone-ui/src/lib.rs` (start polling when entering InGame with dev_mode)

**Step 1: Add command execution function**

```rust
use crate::game_state::GameState;
use crate::AppSignals;
use undone_domain::BoundedStat;

pub fn execute_command(
    cmd: DevCommand,
    gs: &mut GameState,
    signals: AppSignals,
) -> DevResponse {
    match cmd {
        DevCommand::QuickStart { .. } => {
            // Quick start only works before InGame — if we're here, game is running
            DevResponse::err("Game already running. Use jump_to_scene instead.")
        }
        DevCommand::JumpToScene { scene_id } => {
            if !gs.engine.has_scene(&scene_id) {
                return DevResponse::err(format!("Unknown scene: {scene_id}"));
            }
            gs.engine.reset_runtime();
            crate::start_scene(&mut gs.engine, &mut gs.world, &gs.registry, scene_id.clone());
            let events = gs.engine.drain();
            crate::process_events(events, signals, &gs.world, gs.femininity_id);
            signals.tab.set(crate::AppTab::Game);
            DevResponse::ok(format!("Jumped to {scene_id}"))
        }
        DevCommand::SetStat { stat, value } => {
            match stat.as_str() {
                "stress" => { gs.world.player.stress = BoundedStat::new(value); }
                "anxiety" => { gs.world.player.anxiety = BoundedStat::new(value); }
                "money" => { gs.world.player.money = value; }
                "femininity" => {
                    if let Some(sv) = gs.world.player.skills.get_mut(&gs.femininity_id) {
                        sv.value = value;
                    }
                }
                other => return DevResponse::err(format!("Unknown stat: {other}")),
            }
            signals.player.set(crate::PlayerSnapshot::from_player(
                &gs.world.player, gs.femininity_id,
            ));
            DevResponse::ok(format!("Set {stat} = {value}"))
        }
        DevCommand::SetFlag { flag } => {
            gs.world.game_data.set_flag(flag.clone());
            DevResponse::ok(format!("Set flag: {flag}"))
        }
        DevCommand::RemoveFlag { flag } => {
            gs.world.game_data.remove_flag(&flag);
            DevResponse::ok(format!("Removed flag: {flag}"))
        }
        DevCommand::GetState => {
            let data = serde_json::json!({
                "week": gs.world.game_data.week,
                "day": gs.world.game_data.day,
                "time_slot": format!("{:?}", gs.world.game_data.time_slot),
                "stress": gs.world.player.stress.get(),
                "anxiety": gs.world.player.anxiety.get(),
                "money": gs.world.player.money,
                "flags": gs.world.game_data.flags.iter().collect::<Vec<_>>(),
                "arc_states": gs.world.game_data.arc_states,
            });
            DevResponse::ok_with_data("Current state", data)
        }
        DevCommand::AdvanceTime { weeks } => {
            let slots = weeks * 28; // 4 slots/day × 7 days/week
            for _ in 0..slots {
                gs.world.game_data.advance_time_slot();
            }
            DevResponse::ok(format!("Advanced {weeks} week(s)"))
        }
        DevCommand::SetNpcLiking { npc_name, level } => {
            let liking = match level.as_str() {
                "Neutral" => undone_domain::LikingLevel::Neutral,
                "Ok" => undone_domain::LikingLevel::Ok,
                "Like" => undone_domain::LikingLevel::Like,
                "Close" => undone_domain::LikingLevel::Close,
                other => return DevResponse::err(format!("Unknown liking level: {other}")),
            };
            let mut found = false;
            for (_, npc) in gs.world.male_npcs.iter_mut() {
                if npc.core.name.to_lowercase() == npc_name.to_lowercase() {
                    npc.core.npc_liking = liking;
                    found = true;
                    break;
                }
            }
            if !found {
                for (_, npc) in gs.world.female_npcs.iter_mut() {
                    if npc.core.name.to_lowercase() == npc_name.to_lowercase() {
                        npc.core.npc_liking = liking;
                        found = true;
                        break;
                    }
                }
            }
            if found {
                DevResponse::ok(format!("Set {npc_name} liking to {level}"))
            } else {
                DevResponse::err(format!("NPC not found: {npc_name}"))
            }
        }
    }
}
```

**Step 2: Add has_scene() to SceneEngine**

In `crates/undone-scene/src/engine.rs`:

```rust
pub fn has_scene(&self, scene_id: &str) -> bool {
    self.scenes.contains_key(scene_id)
}
```

**Step 3: Add IPC polling**

```rust
use std::path::PathBuf;

fn dev_cmd_path() -> PathBuf {
    std::env::temp_dir().join("undone-dev-cmd.json")
}

fn dev_result_path() -> PathBuf {
    std::env::temp_dir().join("undone-dev-result.json")
}

/// Start the IPC polling loop. Call once when entering InGame with dev_mode.
pub fn start_dev_ipc_polling(
    gs: Rc<RefCell<GameState>>,
    signals: AppSignals,
) {
    let poll_interval = std::time::Duration::from_millis(100);
    floem::action::exec_after(poll_interval, move |_| {
        poll_dev_commands(Rc::clone(&gs), signals);
    });
}

fn poll_dev_commands(
    gs: Rc<RefCell<GameState>>,
    signals: AppSignals,
) {
    let cmd_path = dev_cmd_path();
    if cmd_path.exists() {
        if let Ok(contents) = std::fs::read_to_string(&cmd_path) {
            let _ = std::fs::remove_file(&cmd_path);
            match serde_json::from_str::<DevCommand>(&contents) {
                Ok(cmd) => {
                    let response = {
                        let mut gs = gs.borrow_mut();
                        execute_command(cmd, &mut gs, signals)
                    };
                    let result_json = serde_json::to_string_pretty(&response)
                        .unwrap_or_else(|e| format!(r#"{{"success":false,"message":"serialize error: {e}"}}"#));
                    let _ = std::fs::write(dev_result_path(), result_json);
                }
                Err(e) => {
                    let resp = DevResponse::err(format!("Invalid command JSON: {e}"));
                    let result_json = serde_json::to_string_pretty(&resp).unwrap_or_default();
                    let _ = std::fs::write(dev_result_path(), result_json);
                }
            }
        }
    }

    // Re-schedule next poll
    let poll_interval = std::time::Duration::from_millis(100);
    floem::action::exec_after(poll_interval, move |_| {
        poll_dev_commands(gs, signals);
    });
}
```

**Step 4: Start polling when entering InGame**

In `crates/undone-ui/src/lib.rs`, in the `AppPhase::InGame` branch, after creating `gs_cell`, if dev_mode:

```rust
if dev_mode {
    crate::dev_ipc::start_dev_ipc_polling(Rc::clone(&gs_cell), signals);
}
```

**Step 5: Build and test manually**

```bash
cargo run --release --bin undone -- --dev --quick
```

In another terminal, test the IPC:

```bash
echo '{"command": "get_state"}' > "$TEMP/undone-dev-cmd.json"
sleep 0.2
cat "$TEMP/undone-dev-result.json"
```

Expected: JSON response with current game state.

```bash
echo '{"command": "jump_to_scene", "scene_id": "base::coffee_shop"}' > "$TEMP/undone-dev-cmd.json"
sleep 0.2
cat "$TEMP/undone-dev-result.json"
```

Expected: Game jumps to coffee_shop scene. Response says "Jumped to base::coffee_shop".

**Step 6: Commit**

```bash
git add crates/undone-ui/src/dev_ipc.rs crates/undone-ui/src/lib.rs crates/undone-scene/src/engine.rs
git commit -m "feat: file-based IPC for dev commands (jump_to_scene, set_stat, get_state, etc.)"
```

---

### Task 10: game-input MCP — dev command tools

**Files:**
- Modify: `tools/game-input-mcp/src/server.rs` (add dev command tools, update start_game)

**Step 1: Update start_game to accept dev_mode**

Change `StartGameInput`:

```rust
#[derive(Debug, Deserialize, JsonSchema)]
struct StartGameInput {
    /// Working directory containing the Cargo workspace
    working_dir: String,
    /// Launch in dev mode with quick start (--dev --quick)
    #[serde(default)]
    dev_mode: bool,
}
```

Update `start_game` handler:

```rust
let mut args = vec!["run", "--release", "--bin", "undone"];
if params.0.dev_mode {
    args.extend_from_slice(&["--", "--dev", "--quick"]);
}
let child = std::process::Command::new("cargo")
    .args(&args)
    // ... rest unchanged
```

**Step 2: Add dev_command tool**

Add a generic `dev_command` tool that writes to the IPC file and reads the response:

```rust
#[derive(Debug, Deserialize, JsonSchema)]
struct DevCommandInput {
    /// JSON command to send (e.g. {"command": "jump_to_scene", "scene_id": "base::rain_shelter"})
    command_json: String,
    /// Timeout in milliseconds to wait for response (default: 2000)
    #[serde(default = "default_timeout")]
    timeout_ms: u64,
}

fn default_timeout() -> u64 { 2000 }
```

Handler:

```rust
#[tool(description = "Send a dev command to a running Undone game in --dev mode. Commands: jump_to_scene, set_stat, set_flag, remove_flag, get_state, advance_time, set_npc_liking. The game must be running with --dev flag.")]
async fn dev_command(
    &self,
    params: Parameters<DevCommandInput>,
) -> Result<CallToolResult, McpError> {
    let cmd_path = std::env::temp_dir().join("undone-dev-cmd.json");
    let result_path = std::env::temp_dir().join("undone-dev-result.json");

    // Clean up any stale result file
    let _ = std::fs::remove_file(&result_path);

    // Write command via atomic rename
    let tmp_path = std::env::temp_dir().join("undone-dev-cmd.tmp");
    std::fs::write(&tmp_path, &params.0.command_json)
        .map_err(|e| McpError::internal_error(format!("write failed: {e}"), None))?;
    std::fs::rename(&tmp_path, &cmd_path)
        .map_err(|e| McpError::internal_error(format!("rename failed: {e}"), None))?;

    // Poll for response
    let deadline = std::time::Instant::now()
        + std::time::Duration::from_millis(params.0.timeout_ms);
    loop {
        if result_path.exists() {
            let result = std::fs::read_to_string(&result_path)
                .map_err(|e| McpError::internal_error(format!("read result failed: {e}"), None))?;
            let _ = std::fs::remove_file(&result_path);
            return Ok(CallToolResult::success(vec![Content::text(result)]));
        }
        if std::time::Instant::now() > deadline {
            return Ok(CallToolResult::success(vec![Content::text(
                r#"{"success": false, "message": "Timeout waiting for game response. Is the game running with --dev?"}"#
            )]));
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
```

**Step 3: Add convenience tools**

Add thin wrapper tools for common operations so MCP clients get proper schemas:

```rust
#[derive(Debug, Deserialize, JsonSchema)]
struct JumpToSceneInput {
    /// Scene ID to jump to (e.g. "base::rain_shelter")
    scene_id: String,
}

#[tool(description = "Jump to a specific scene in a running Undone game (requires --dev mode)")]
async fn jump_to_scene(
    &self,
    params: Parameters<JumpToSceneInput>,
) -> Result<CallToolResult, McpError> {
    let cmd = format!(r#"{{"command":"jump_to_scene","scene_id":"{}"}}"#, params.0.scene_id);
    self.dev_command(Parameters(DevCommandInput {
        command_json: cmd,
        timeout_ms: 2000,
    })).await
}
```

Similarly for `get_game_state` (no args), `set_game_stat` (stat + value), etc. Keep it to the most useful 3-4 convenience wrappers.

**Step 4: Build the MCP server**

```bash
cd tools && cargo build --release
```

Expected: Compiles successfully.

**Step 5: Integration test**

Start the game in one terminal:
```bash
cargo run --release --bin undone -- --dev --quick
```

In Claude Code or another MCP client, call:
- `start_game(working_dir=".", dev_mode=true)` — should launch with --dev --quick
- `get_game_state()` — should return JSON with current state
- `jump_to_scene(scene_id="base::coffee_shop")` — should jump and return success

**Step 6: Commit**

```bash
cd .. && git add tools/game-input-mcp/src/server.rs
git commit -m "feat(mcp): dev command tools — jump_to_scene, get_state, set_stat via IPC"
```

---

## Phase 5: Schedule Reachability Checker

### Task 11: Reachability analysis module

**Files:**
- Create: `crates/undone-scene/src/reachability.rs`
- Modify: `crates/undone-scene/src/lib.rs` (add `pub mod reachability;`)

**Step 1: Write tests first**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_required_but_never_set_is_warned() {
        // Schedule condition requires JAKE_MET flag
        // No scene effect sets JAKE_MET
        // → warning
    }

    #[test]
    fn flag_required_and_set_by_effect_passes() {
        // Schedule condition requires JAKE_MET flag
        // Some scene has SetGameFlag { flag: "JAKE_MET" }
        // → no warning
    }

    #[test]
    fn exact_equality_liking_check_warns_when_overshoot_possible() {
        // Condition: npcLiking == 'Like'
        // Effects can add delta > 1 to liking (Ok→Like→Close skip)
        // → warning about overshoot
    }
}
```

**Step 2: Implement the analysis**

The function signature:

```rust
use crate::types::{SceneDefinition, EffectDef};
use undone_expr::Expr;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct ReachabilityWarning {
    pub context: String,  // e.g. "schedule slot 'free_time', scene 'base::jake_first_date'"
    pub message: String,
}

pub fn check_reachability(
    schedule_conditions: &[(String, Expr)],  // (context, condition expression)
    scenes: &HashMap<String, Arc<SceneDefinition>>,
) -> Vec<ReachabilityWarning> {
    // 1. Walk all scene effects to build:
    //    - set_of_flags_that_can_be_set: HashSet<String>
    //    - set_of_flags_that_can_be_removed: HashSet<String>
    //    - set_of_arc_states_reachable: HashMap<String, HashSet<String>>
    //    - npc_liking_can_change: bool (any AddNpcLiking effect exists)
    //
    // 2. Walk all schedule conditions to extract:
    //    - hasGameFlag('X') references → check X is in set_of_flags_that_can_be_set
    //    - arcState('arc', 'state') references → check state is reachable
    //    - npcLiking == 'Level' exact equality → warn about overshoot
    //
    // 3. Return warnings for unreachable requirements
}
```

**Step 3: Expose schedule conditions from Scheduler**

The Scheduler currently doesn't expose its parsed conditions. Add a method:

```rust
// In crates/undone-scene/src/scheduler.rs, add to Scheduler impl:
pub fn all_conditions(&self) -> Vec<(String, &Expr)> {
    let mut result = Vec::new();
    for (slot_name, events) in &self.slots {
        for event in events {
            let ctx = format!("slot '{}', scene '{}'", slot_name, event.scene);
            if let Some(ref expr) = event.condition {
                result.push((ctx.clone(), expr));
            }
            if let Some(ref expr) = event.trigger {
                result.push((format!("{ctx} (trigger)"), expr));
            }
        }
    }
    result
}
```

**Step 4: Run tests**

```bash
cargo test -p undone-scene reachability
```

Expected: ALL PASS

**Step 5: Commit**

```bash
git add crates/undone-scene/src/reachability.rs crates/undone-scene/src/lib.rs crates/undone-scene/src/scheduler.rs
git commit -m "feat: schedule reachability checker — warns on unreachable conditions"
```

---

### Task 12: Integrate reachability checker into validate-pack

**Files:**
- Modify: `src/bin/validate_pack.rs`

**Step 1: Call check_reachability after loading**

After the schedule and scenes are loaded and validated:

```rust
// Reachability analysis
if let Some(ref scheduler) = scheduler {
    let conditions = scheduler.all_conditions();
    let owned: Vec<(String, undone_scene::reachability::Expr)> = conditions
        .into_iter()
        .map(|(ctx, expr)| (ctx, expr.clone()))
        .collect();
    let warnings = undone_scene::reachability::check_reachability(&owned, &all_scenes);
    for w in &warnings {
        eprintln!("WARN  [reachability] {}: {}", w.context, w.message);
    }
}
```

**Step 2: Build and run against base pack**

```bash
cargo run --bin validate-pack
```

Expected: Existing content should produce zero or minimal warnings (since known reachability bugs like the Jake arc have been fixed). If new warnings appear, verify they're real.

**Step 3: Commit**

```bash
git add src/bin/validate_pack.rs
git commit -m "feat: validate-pack runs reachability analysis on schedule conditions"
```

---

## Phase 6: Scene Distribution Simulator

### Task 13: Simulation function with tests

**Files:**
- Create or modify: `crates/undone-scene/src/simulator.rs`
- Modify: `crates/undone-scene/src/lib.rs`

**Step 1: Write the test first**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_counts_scene_frequencies() {
        // Create a scheduler with 2 scenes of equal weight
        // Run 100 simulations of 10 weeks each
        // Both scenes should appear with roughly equal frequency
        // (within statistical bounds — use a seeded RNG for determinism)
    }

    #[test]
    fn simulation_detects_never_fires() {
        // Create a scheduler with one scene that has an impossible condition
        // Run simulation → that scene should have 0 picks
    }
}
```

**Step 2: Implement**

```rust
use std::collections::HashMap;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use crate::scheduler::Scheduler;
use undone_packs::PackRegistry;
use undone_world::World;

pub struct SimulationConfig {
    pub weeks: u32,
    pub runs: u32,
    pub seed: u64,
}

pub struct SimulationResult {
    pub scene_counts: HashMap<String, u64>,
    pub total_picks: u64,
    pub runs: u32,
    pub weeks: u32,
}

pub struct SceneStats {
    pub scene_id: String,
    pub count: u64,
    pub percentage: f64,
    pub avg_per_run: f64,
    pub warning: Option<String>,  // "DOMINANT", "RARE", "NEVER FIRES"
}

const DOMINANT_THRESHOLD: f64 = 12.0;
const RARE_THRESHOLD: f64 = 1.0;

impl SimulationResult {
    pub fn stats(&self) -> Vec<SceneStats> {
        let mut stats: Vec<SceneStats> = self.scene_counts.iter().map(|(id, &count)| {
            let percentage = if self.total_picks > 0 {
                (count as f64 / self.total_picks as f64) * 100.0
            } else { 0.0 };
            let avg_per_run = count as f64 / self.runs as f64;
            let warning = if count == 0 {
                Some("NEVER FIRES".to_string())
            } else if percentage > DOMINANT_THRESHOLD {
                Some("DOMINANT".to_string())
            } else if percentage < RARE_THRESHOLD {
                Some("RARE".to_string())
            } else {
                None
            };
            SceneStats { scene_id: id.clone(), count, percentage, avg_per_run, warning }
        }).collect();
        stats.sort_by(|a, b| b.count.cmp(&a.count));
        stats
    }
}

pub fn simulate(
    scheduler: &Scheduler,
    registry: &PackRegistry,
    base_world: &World,
    config: SimulationConfig,
) -> SimulationResult {
    let mut rng = SmallRng::seed_from_u64(config.seed);
    let mut scene_counts: HashMap<String, u64> = HashMap::new();
    let mut total_picks = 0u64;

    // Seed the counts map with all known scene IDs (so never-fired scenes show up)
    for (ctx, _) in scheduler.all_conditions() {
        // Extract scene ID from context string — or better, add a method
        // to Scheduler that returns all scene IDs
    }

    for _ in 0..config.runs {
        let mut world = base_world.clone();
        for _ in 0..config.weeks {
            // One pick per week (simplified — real game has multiple picks/week)
            if let Some(result) = scheduler.pick_next(&world, registry, &mut rng) {
                *scene_counts.entry(result.scene_id.clone()).or_insert(0) += 1;
                total_picks += 1;
                if result.once_only {
                    world.game_data.set_flag(format!("ONCE_{}", result.scene_id));
                }
            }
            // Advance time by one week
            for _ in 0..28 {
                world.game_data.advance_time_slot();
            }
        }
    }

    SimulationResult { scene_counts, total_picks, runs: config.runs, weeks: config.weeks }
}
```

**Step 3: Add all_scene_ids() to Scheduler**

```rust
// In scheduler.rs:
pub fn all_scene_ids(&self) -> Vec<String> {
    self.slots.values()
        .flat_map(|events| events.iter().map(|e| e.scene.clone()))
        .collect()
}
```

**Step 4: Run tests**

```bash
cargo test -p undone-scene simulator
```

Expected: ALL PASS

**Step 5: Commit**

```bash
git add crates/undone-scene/src/simulator.rs crates/undone-scene/src/lib.rs crates/undone-scene/src/scheduler.rs
git commit -m "feat: scene distribution simulator with frequency analysis"
```

---

### Task 14: validate-pack --simulate flag

**Files:**
- Modify: `src/bin/validate_pack.rs`

**Step 1: Add CLI arg parsing**

```rust
let args: Vec<String> = std::env::args().collect();
let simulate = args.iter().any(|a| a == "--simulate");
let weeks: u32 = args.iter()
    .position(|a| a == "--weeks")
    .and_then(|i| args.get(i + 1))
    .and_then(|s| s.parse().ok())
    .unwrap_or(52);
let runs: u32 = args.iter()
    .position(|a| a == "--runs")
    .and_then(|i| args.get(i + 1))
    .and_then(|s| s.parse().ok())
    .unwrap_or(1000);
```

**Step 2: Run simulation after validation**

```rust
if simulate {
    println!("\nRunning distribution simulation ({weeks} weeks × {runs} runs)...\n");

    // Build a Robin preset world for simulation
    let config = undone_ui::char_creation::robin_quick_config(&registry);
    let mut sim_registry = registry.clone(); // if needed
    let mut sim_rng = rand::rngs::SmallRng::seed_from_u64(42);
    let world = undone_packs::char_creation::new_game(config, &mut sim_registry, &mut sim_rng);

    let result = undone_scene::simulator::simulate(
        scheduler.as_ref().unwrap(),
        &registry,
        &world,
        undone_scene::simulator::SimulationConfig { weeks, runs, seed: 42 },
    );

    println!("Scene Distribution ({weeks} weeks × {runs} runs):");
    for stat in result.stats() {
        let warning = stat.warning.as_ref().map(|w| format!("  ⚠ {w}")).unwrap_or_default();
        println!(
            "  {:<40} — {:>5.1}% (avg {:.1}/run){}",
            stat.scene_id, stat.percentage, stat.avg_per_run, warning
        );
    }

    let warnings: Vec<_> = result.stats().into_iter().filter(|s| s.warning.is_some()).collect();
    if !warnings.is_empty() {
        println!("\nWarnings:");
        for stat in warnings {
            println!("  - {} {}", stat.scene_id, stat.warning.unwrap());
        }
    }
}
```

**Step 3: Build and run**

```bash
cargo run --bin validate-pack -- --simulate --weeks 52 --runs 100
```

Expected: Validation passes, then simulation output shows scene frequencies with dominant/rare/never-fires warnings.

**Step 4: Commit**

```bash
git add src/bin/validate_pack.rs
git commit -m "feat: validate-pack --simulate for scene distribution analysis"
```

---

## Phase 7: Acceptance Tests

### Task 15: Acceptance tests for BoundedStat

**Files:**
- Tests in: `crates/undone-scene/src/effects.rs` (existing test module)

**Acceptance Criteria:**
- Stress cannot go below 0 via effects
- Stress cannot go above 100 via effects
- Anxiety cannot go below 0 via effects
- Anxiety cannot go above 100 via effects
- Money is unbounded (can go negative, can go high)

**Step 1: Write acceptance tests**

Add to the existing effects.rs test module:

```rust
#[test]
fn change_stress_cannot_go_below_zero() {
    let mut world = make_world();
    world.player.stress = BoundedStat::new(5);
    let registry = PackRegistry::new();
    let mut ctx = SceneCtx::new();
    apply_effect(&EffectDef::ChangeStress { amount: -50 }, &mut world, &mut ctx, &registry).unwrap();
    assert_eq!(world.player.stress.get(), 0);
}

#[test]
fn change_stress_cannot_exceed_100() {
    let mut world = make_world();
    world.player.stress = BoundedStat::new(90);
    let registry = PackRegistry::new();
    let mut ctx = SceneCtx::new();
    apply_effect(&EffectDef::ChangeStress { amount: 50 }, &mut world, &mut ctx, &registry).unwrap();
    assert_eq!(world.player.stress.get(), 100);
}

#[test]
fn change_anxiety_cannot_go_below_zero() {
    let mut world = make_world();
    world.player.anxiety = BoundedStat::new(3);
    let registry = PackRegistry::new();
    let mut ctx = SceneCtx::new();
    apply_effect(&EffectDef::ChangeAnxiety { amount: -10 }, &mut world, &mut ctx, &registry).unwrap();
    assert_eq!(world.player.anxiety.get(), 0);
}

#[test]
fn change_anxiety_cannot_exceed_100() {
    let mut world = make_world();
    world.player.anxiety = BoundedStat::new(95);
    let registry = PackRegistry::new();
    let mut ctx = SceneCtx::new();
    apply_effect(&EffectDef::ChangeAnxiety { amount: 20 }, &mut world, &mut ctx, &registry).unwrap();
    assert_eq!(world.player.anxiety.get(), 100);
}

#[test]
fn change_money_can_go_negative() {
    let mut world = make_world();
    world.player.money = 10;
    let registry = PackRegistry::new();
    let mut ctx = SceneCtx::new();
    apply_effect(&EffectDef::ChangeMoney { amount: -50 }, &mut world, &mut ctx, &registry).unwrap();
    assert_eq!(world.player.money, -40);
}
```

**Step 2: Run**

```bash
cargo test -p undone-scene
```

Expected: ALL PASS

**Step 3: Commit**

```bash
git add crates/undone-scene/src/effects.rs
git commit -m "test: acceptance tests for stat bounds enforcement"
```

---

### Task 16: Acceptance tests for dev IPC

**Acceptance Criteria:**
- MCP can get game state from a running dev-mode game
- MCP can jump to a scene and the game responds
- MCP can set a stat and the value updates
- MCP can set/remove flags
- Invalid commands return error responses
- Timeout is handled when game is not running

These are tested via the IPC protocol unit tests (Task 8) and manual integration testing (Task 10 Step 5). The IPC layer is thin enough that the unit tests + manual smoke test provide adequate coverage.

**Step 1: Add integration-style test for execute_command**

In `crates/undone-ui/src/dev_ipc.rs` tests:

```rust
// These tests require a real GameState — use the test_pre_state() pattern
// from game_state.rs tests.

#[test]
fn execute_jump_to_unknown_scene_returns_error() {
    // Build a GameState, try jumping to "nonexistent::scene"
    // Assert response.success == false
}

#[test]
fn execute_set_stat_stress_clamps_to_bounds() {
    // Build a GameState, set stress to 999
    // Assert world.player.stress.get() == 100
}

#[test]
fn execute_get_state_returns_valid_json() {
    // Build a GameState, call GetState
    // Assert response.data is Some and contains expected fields
}
```

**Step 2: Run**

```bash
cargo test -p undone-ui dev_ipc
```

Expected: ALL PASS

**Step 3: Commit**

```bash
git add crates/undone-ui/src/dev_ipc.rs
git commit -m "test: acceptance tests for dev IPC command execution"
```

---

### Task 17: Final integration — build, validate, run

**Step 1: Run full test suite**

```bash
cargo test
```

Expected: ALL PASS (262 + new tests)

**Step 2: Run validate-pack with simulation**

```bash
cargo run --bin validate-pack -- --simulate --weeks 52 --runs 100
```

Expected: All checks pass. Simulation output shows scene distribution. Note any warnings for content tuning.

**Step 3: Build MCP tools**

```bash
cd tools && cargo build --release && cd ..
```

Expected: All MCP servers build successfully.

**Step 4: Runtime smoke test**

```bash
cargo run --release --bin undone -- --dev --quick
```

Verify:
- Game launches directly into InGame (no char creation)
- Dev tab visible in title bar
- Scene jumper works (search + click)
- Stat editors work
- Flag editor works
- State inspector shows correct data

**Step 5: IPC smoke test**

With game running in --dev mode:

```bash
echo '{"command":"get_state"}' > "$TEMP/undone-dev-cmd.json" && sleep 0.2 && cat "$TEMP/undone-dev-result.json"
echo '{"command":"set_stat","stat":"stress","value":75}' > "$TEMP/undone-dev-cmd.json" && sleep 0.2 && cat "$TEMP/undone-dev-result.json"
echo '{"command":"jump_to_scene","scene_id":"base::coffee_shop"}' > "$TEMP/undone-dev-cmd.json" && sleep 0.2 && cat "$TEMP/undone-dev-result.json"
```

**Step 6: Update docs**

Update `docs/plans/2026-02-21-engine-design.md` with:
- BoundedStat type in the Player struct
- Dev mode / IPC section
- `--dev` / `--quick` flags

Update `HANDOFF.md` with session results.

**Step 7: Final commit**

```bash
git add -A
git commit -m "docs: update engine design and handoff for dev tooling suite"
```

---

## Summary

| Phase | Tasks | Estimated scope |
|---|---|---|
| 1: BoundedStat | 1-2 | Small — type + migration |
| 2: CLI + Quick Start | 3-4 | Medium — arg parsing, preset extraction |
| 3: Dev Tab UI | 5-7 | Large — new UI module, multiple panels |
| 4: File-based IPC | 8-10 | Medium — protocol, polling, MCP tools |
| 5: Reachability | 11-12 | Medium — static analysis, validate-pack |
| 6: Simulator | 13-14 | Medium — Monte Carlo, reporting |
| 7: Acceptance | 15-17 | Small — tests + integration |

**Execution order:** Phases 1-2 first (foundational). Then 3-4 (the main deliverable). Then 5-6 (validate-pack extensions). Phase 7 acceptance tests run alongside each phase.
