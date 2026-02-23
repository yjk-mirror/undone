# PC Origin System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace `always_female: bool` + hidden trait system with an explicit `PcOrigin` enum. Add trans woman PC type. Rename `Sexuality` to `BeforeSexuality` with orientation-only variants.

**Architecture:** New `PcOrigin` enum in `undone-domain/src/enums.rs` becomes the source of truth. Hidden traits (`TRANS_WOMAN`, `ALWAYS_FEMALE`, `NOT_TRANSFORMED`) are auto-injected at game start based on origin. The expression evaluator gains `w.pcOrigin()` and keeps backward-compat `w.alwaysFemale()`. Character creation UI becomes a two-step flow (transformed? → which kind?). Save format bumps to v2 with migration.

**Tech Stack:** Rust, floem (reactive UI), minijinja (templates), serde_json (saves)

---

### Task 1: Add PcOrigin enum and rename Sexuality → BeforeSexuality

**Files:**
- Modify: `crates/undone-domain/src/enums.rs`
- Modify: `crates/undone-domain/src/lib.rs` (re-export)

**Step 1: Add PcOrigin enum to enums.rs**

Add after the existing `Sexuality` enum (around line 109):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PcOrigin {
    CisMaleTransformed,
    TransWomanTransformed,
    CisFemaleTransformed,
    AlwaysFemale,
}

impl PcOrigin {
    /// Was this PC magically transformed?
    pub fn was_transformed(self) -> bool {
        !matches!(self, PcOrigin::AlwaysFemale)
    }

    /// Did this PC have a male body before transformation?
    pub fn was_male_bodied(self) -> bool {
        matches!(self, PcOrigin::CisMaleTransformed | PcOrigin::TransWomanTransformed)
    }

    /// Should the "before" section show in character creation?
    pub fn has_before_life(self) -> bool {
        self.was_transformed()
    }

    /// For backward compat: equivalent to the old `always_female` bool.
    /// True for CisFemaleTransformed and AlwaysFemale.
    pub fn is_always_female(self) -> bool {
        matches!(self, PcOrigin::CisFemaleTransformed | PcOrigin::AlwaysFemale)
    }
}

impl std::fmt::Display for PcOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PcOrigin::CisMaleTransformed => write!(f, "Transformed (was cis man)"),
            PcOrigin::TransWomanTransformed => write!(f, "Transformed (was trans woman)"),
            PcOrigin::CisFemaleTransformed => write!(f, "Transformed (was cis woman)"),
            PcOrigin::AlwaysFemale => write!(f, "Always Female"),
        }
    }
}
```

**Step 2: Rename Sexuality → BeforeSexuality, remove AlwaysFemale variant**

Replace the `Sexuality` enum (lines 103-109) with:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BeforeSexuality {
    AttractedToWomen,
    AttractedToMen,
    AttractedToBoth,
}

impl std::fmt::Display for BeforeSexuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BeforeSexuality::AttractedToWomen => write!(f, "Attracted to Women"),
            BeforeSexuality::AttractedToMen => write!(f, "Attracted to Men"),
            BeforeSexuality::AttractedToBoth => write!(f, "Attracted to Both"),
        }
    }
}
```

Remove the old `Sexuality` Display impl (lines 216-225).

**Step 3: Update lib.rs re-exports**

In `crates/undone-domain/src/lib.rs`, the `pub use enums::*` already re-exports everything. No change needed unless `Sexuality` is explicitly named in the re-export line. Check and update if needed.

**Step 4: Run cargo check on undone-domain only**

Run: `cargo check -p undone-domain`
Expected: Passes (other crates will break — we fix them in subsequent tasks).

**Step 5: Commit**

```
git add crates/undone-domain/src/enums.rs
git commit -m "feat: add PcOrigin enum, rename Sexuality to BeforeSexuality"
```

---

### Task 2: Update Player struct

**Files:**
- Modify: `crates/undone-domain/src/player.rs`

**Step 1: Update Player struct fields**

Replace lines 81-87:
```rust
    // Transformation axis
    pub always_female: bool, // false = male-start PC

    // Before-transformation data
    pub before_age: u32,
    pub before_race: String,
    pub before_sexuality: Sexuality,
```

With:
```rust
    // Transformation axis
    pub origin: PcOrigin,

    // Before-transformation data (meaningful when origin.has_before_life())
    pub before_age: u32,
    pub before_race: String,
    pub before_sexuality: Option<BeforeSexuality>,
```

**Step 2: Update imports at top of player.rs**

Line 2 imports `Sexuality` — change to `BeforeSexuality, PcOrigin`.

**Step 3: Update Player methods**

The `active_name()` method reads skills, no change needed. But check if any method references `always_female` — none do currently.

**Step 4: Update test helper `make_player()`**

Replace `always_female: false` and `before_sexuality: crate::Sexuality::StraightMale` with:
```rust
origin: crate::PcOrigin::CisMaleTransformed,
before_sexuality: Some(crate::BeforeSexuality::AttractedToWomen),
```

**Step 5: Run cargo check on undone-domain**

Run: `cargo check -p undone-domain`
Expected: PASS

**Step 6: Commit**

```
git add crates/undone-domain/src/player.rs
git commit -m "feat: Player uses PcOrigin and Option<BeforeSexuality>"
```

---

### Task 3: Update undone-packs (char creation config + new_game)

**Files:**
- Modify: `crates/undone-packs/src/char_creation.rs`
- Modify: `packs/base/data/traits.toml`

**Step 1: Add TRANS_WOMAN trait to traits.toml**

Append after the NOT_TRANSFORMED entry (line 102):

```toml
[[trait]]
id          = "TRANS_WOMAN"
name        = "Trans Woman"
description = "PC was a trans woman before the magical transformation."
hidden      = true
```

**Step 2: Update CharCreationConfig**

Replace `always_female: bool` and `before_sexuality: Sexuality` with:
```rust
pub origin: PcOrigin,
pub before_sexuality: Option<BeforeSexuality>,
```

Update import line: replace `Sexuality` with `BeforeSexuality, PcOrigin`.

**Step 3: Update new_game() function**

Replace the FEMININITY starting value logic (line 44):
```rust
let starting_femininity = if config.always_female { 75 } else { 10 };
```
With:
```rust
let starting_femininity = match config.origin {
    PcOrigin::CisMaleTransformed => 10,
    PcOrigin::TransWomanTransformed => 70,
    PcOrigin::CisFemaleTransformed | PcOrigin::AlwaysFemale => 75,
};
```

In the Player construction, replace `always_female: config.always_female` with `origin: config.origin`, and `before_sexuality: config.before_sexuality` stays as-is (now `Option<BeforeSexuality>`).

**Step 4: Auto-inject origin-based traits**

After the Player is constructed and before returning, add trait injection logic. Find where `starting_traits` are inserted into the player (this happens in the config — traits are already resolved). Add origin-based trait injection in `new_game()` after the player is built:

```rust
// Auto-inject origin-based hidden traits
match config.origin {
    PcOrigin::TransWomanTransformed => {
        if let Ok(id) = registry.resolve_trait("TRANS_WOMAN") {
            player.traits.insert(id);
        }
    }
    PcOrigin::CisFemaleTransformed => {
        if let Ok(id) = registry.resolve_trait("ALWAYS_FEMALE") {
            player.traits.insert(id);
        }
    }
    PcOrigin::AlwaysFemale => {
        if let Ok(id) = registry.resolve_trait("ALWAYS_FEMALE") {
            player.traits.insert(id);
        }
        if let Ok(id) = registry.resolve_trait("NOT_TRANSFORMED") {
            player.traits.insert(id);
        }
    }
    PcOrigin::CisMaleTransformed => {} // no auto-injected traits
}
```

This replaces the UI-side trait injection that currently happens in `char_creation.rs` (UI) lines 449-451.

**Step 5: Update tests**

Update `new_game_creates_world_with_player` and `new_game_always_female_sets_high_femininity` tests:
- Replace `always_female: false` with `origin: PcOrigin::CisMaleTransformed`
- Replace `before_sexuality: Sexuality::StraightMale` with `before_sexuality: Some(BeforeSexuality::AttractedToWomen)`
- Replace `assert!(!world.player.always_female)` with `assert_eq!(world.player.origin, PcOrigin::CisMaleTransformed)`
- Update the always_female test to use `origin: PcOrigin::AlwaysFemale` and `before_sexuality: None`
- Add new test for TransWomanTransformed starting FEMININITY = 70

**Step 6: Run cargo check + tests on undone-packs**

Run: `cargo check -p undone-packs && cargo test -p undone-packs`
Expected: PASS

**Step 7: Commit**

```
git add crates/undone-packs/src/char_creation.rs packs/base/data/traits.toml
git commit -m "feat: new_game uses PcOrigin, auto-injects origin traits, TRANS_WOMAN trait"
```

---

### Task 4: Update undone-expr evaluator

**Files:**
- Modify: `crates/undone-expr/src/eval.rs`

**Step 1: Update alwaysFemale accessor**

Line 191, replace:
```rust
"alwaysFemale" => Ok(world.player.always_female),
```
With:
```rust
"alwaysFemale" => Ok(world.player.origin.is_always_female()),
```

**Step 2: Add pcOrigin accessor**

In `eval_call_string` (or add a new match arm in `eval_call_bool`'s parent function for string returns — check the structure), add:

```rust
"pcOrigin" => {
    let s = match world.player.origin {
        PcOrigin::CisMaleTransformed => "CisMaleTransformed",
        PcOrigin::TransWomanTransformed => "TransWomanTransformed",
        PcOrigin::CisFemaleTransformed => "CisFemaleTransformed",
        PcOrigin::AlwaysFemale => "AlwaysFemale",
    };
    Ok(Value::String(s.to_string()))
}
```

Note: Check how `eval_call_string` works — the evaluator may need `pcOrigin` as a string-returning method on the `Receiver::Player` arm. Inspect the exact evaluator dispatch structure.

**Step 3: Update test helpers**

All `make_world()` / `make_player()` / Player struct literals in eval.rs tests: replace `always_female: false` with `origin: PcOrigin::CisMaleTransformed`, and `before_sexuality: Sexuality::StraightMale` with `before_sexuality: Some(BeforeSexuality::AttractedToWomen)`.

**Step 4: Run tests**

Run: `cargo test -p undone-expr`
Expected: PASS

**Step 5: Commit**

```
git add crates/undone-expr/src/eval.rs
git commit -m "feat: evaluator uses PcOrigin, adds w.pcOrigin() accessor"
```

---

### Task 5: Update undone-scene (template_ctx, effects, engine, scheduler, lib)

**Files:**
- Modify: `crates/undone-scene/src/template_ctx.rs`
- Modify: `crates/undone-scene/src/effects.rs`
- Modify: `crates/undone-scene/src/engine.rs`
- Modify: `crates/undone-scene/src/lib.rs`
- Modify: `crates/undone-scene/src/scheduler.rs`

**Step 1: Update PlayerCtx in template_ctx.rs**

Replace `always_female: bool` field (line 19) with `origin: PcOrigin`. Import `PcOrigin` from `undone_domain`.

Update `alwaysFemale` method (line 48):
```rust
"alwaysFemale" => Ok(Value::from(self.origin.is_always_female())),
```

Add `pcOrigin` method:
```rust
"pcOrigin" => {
    let s = match self.origin {
        PcOrigin::CisMaleTransformed => "CisMaleTransformed",
        PcOrigin::TransWomanTransformed => "TransWomanTransformed",
        PcOrigin::CisFemaleTransformed => "CisFemaleTransformed",
        PcOrigin::AlwaysFemale => "AlwaysFemale",
    };
    Ok(Value::from(s))
}
```

Update `render_prose` (line 181): replace `always_female: world.player.always_female` with `origin: world.player.origin`.

**Step 2: Update all test helpers across undone-scene**

Every `Player { ... }` literal in:
- `template_ctx.rs` tests (~line 255)
- `effects.rs` tests (~line 277-301)
- `engine.rs` tests (~line 377-401)
- `lib.rs` tests (~line 45-73)
- `scheduler.rs` tests (~line 231-255)

Replace `always_female: false` → `origin: PcOrigin::CisMaleTransformed`
Replace `before_sexuality: Sexuality::StraightMale` → `before_sexuality: Some(BeforeSexuality::AttractedToWomen)`
Add import `use undone_domain::{PcOrigin, BeforeSexuality, ...}` where needed.

**Step 3: Run tests**

Run: `cargo test -p undone-scene`
Expected: PASS

**Step 4: Commit**

```
git add crates/undone-scene/
git commit -m "feat: undone-scene uses PcOrigin, template_ctx adds w.pcOrigin()"
```

---

### Task 6: Update undone-save (migration + version bump)

**Files:**
- Modify: `crates/undone-save/src/lib.rs`

**Step 1: Bump SAVE_VERSION**

Change `pub const SAVE_VERSION: u32 = 1` to `pub const SAVE_VERSION: u32 = 2`.

**Step 2: Add v1 migration types**

Add serde-compatible structs for the old format:

```rust
#[derive(Deserialize)]
struct V1Player {
    #[serde(default)]
    always_female: bool,
    #[serde(default)]
    before_sexuality: V1Sexuality,
    // ... all other fields same as Player
}

#[derive(Deserialize, Default)]
enum V1Sexuality {
    #[default]
    StraightMale,
    GayMale,
    BiMale,
    AlwaysFemale,
}
```

Alternatively, if feasible: use `#[serde(alias = "always_female")]` and a custom deserializer on `Player` that can read the old format. The simplest approach depends on serde complexity. If the v1→v2 conversion is complex, use an intermediate type. If it's just field renames, use `#[serde(alias)]`.

**Step 3: Add migration logic in load path**

In the load function, when version == 1:
- Deserialize the raw JSON
- Map `always_female: false` → `origin: "CisMaleTransformed"`
- Map `always_female: true` + has `NOT_TRANSFORMED` trait → `origin: "AlwaysFemale"`
- Map `always_female: true` without → `origin: "CisFemaleTransformed"`
- Map sexuality: `StraightMale` → `Some(AttractedToWomen)`, `GayMale` → `Some(AttractedToMen)`, `BiMale` → `Some(AttractedToBoth)`, `AlwaysFemale` → `None`

**Step 4: Update test helpers**

Replace `always_female: false` and `before_sexuality: Sexuality::StraightMale` in test Player literals.

**Step 5: Run tests**

Run: `cargo test -p undone-save`
Expected: PASS

**Step 6: Commit**

```
git add crates/undone-save/src/lib.rs
git commit -m "feat: save v2 with PcOrigin, v1 migration"
```

---

### Task 7: Update undone-ui character creation (two-step flow)

**Files:**
- Modify: `crates/undone-ui/src/char_creation.rs`
- Modify: `crates/undone-ui/src/game_state.rs`
- Modify: `crates/undone-ui/src/lib.rs`

**Step 1: Update CharFormSignals**

Replace:
```rust
always_female: RwSignal<bool>,
sexuality: RwSignal<Sexuality>,
```
With:
```rust
was_transformed: RwSignal<bool>,   // Step 1: true = yes
before_kind: RwSignal<BeforeKind>, // Step 2: which kind
sexuality: RwSignal<BeforeSexuality>,
```

Add a local enum for the step-2 choice:
```rust
#[derive(Clone, Copy, PartialEq, Eq)]
enum BeforeKind {
    CisMale,
    TransWoman,
    CisFemale,
}
```

Default: `was_transformed: RwSignal::new(true)`, `before_kind: RwSignal::new(BeforeKind::CisMale)`.

Derive `PcOrigin` from signals:
```rust
fn resolve_origin(was_transformed: bool, before_kind: BeforeKind) -> PcOrigin {
    if !was_transformed {
        PcOrigin::AlwaysFemale
    } else {
        match before_kind {
            BeforeKind::CisMale => PcOrigin::CisMaleTransformed,
            BeforeKind::TransWoman => PcOrigin::TransWomanTransformed,
            BeforeKind::CisFemale => PcOrigin::CisFemaleTransformed,
        }
    }
}
```

**Step 2: Rebuild section_your_past with two-step flow**

Step 1: Two radio buttons — "Yes, something happened" / "No, I've always been a woman"
Step 2 (conditional on was_transformed == true): Three radio buttons — "I was a man" / "I was a trans woman" / "I was a woman"

Floem doesn't have a built-in RadioGroup — use styled buttons or checkboxes with mutual exclusion (same pattern as Beautiful/Plain). Each option is an `h_stack` with a styled indicator + label, using `on_click_stop` to set the signal.

**Conditional fields:**
- `CisMaleTransformed` / `TransWomanTransformed`: show age + sexuality dropdowns
- `CisFemaleTransformed`: show age only
- `AlwaysFemale`: hide entire section

Sexuality dropdown labels: "Attracted to women", "Attracted to men", "Attracted to both" (the `BeforeSexuality::Display` impl handles this).

**Step 3: Update build_begin_button**

Replace the `is_always_female` logic (lines 401-451). Compute `origin` from signals:
```rust
let origin = resolve_origin(
    form.was_transformed.get_untracked(),
    form.before_kind.get_untracked(),
);
```

Remove manual ALWAYS_FEMALE / NOT_TRANSFORMED trait injection (lines 449-452) — this now happens in `new_game()` (Task 3).

Compute `before_sexuality`:
```rust
let before_sexuality = if origin.has_before_life() && origin.was_male_bodied() {
    Some(form.sexuality.get_untracked())
} else {
    None
};
```

Update `CharCreationConfig` construction:
```rust
origin,
before_sexuality,
```

**Step 4: Update game_state.rs and lib.rs Player literals**

Replace `always_female: false` → `origin: PcOrigin::CisMaleTransformed` and `before_sexuality: Sexuality::StraightMale` → `before_sexuality: Some(BeforeSexuality::AttractedToWomen)` in error/placeholder Player literals.

**Step 5: Run cargo check**

Run: `cargo check -p undone-ui`
Expected: PASS

**Step 6: Commit**

```
git add crates/undone-ui/
git commit -m "feat: two-step char creation flow, PcOrigin selection"
```

---

### Task 8: Update scene content + writing guide

**Files:**
- Modify: `packs/base/scenes/morning_routine.toml`
- Modify: `packs/base/scenes/rain_shelter.toml`
- Modify: `packs/base/scenes/coffee_shop.toml`
- Modify: `docs/writing-guide.md`

**Step 1: Add TRANS_WOMAN branches to scenes**

Existing scenes use `{% if not w.alwaysFemale() %}` for transformation content. This still works — a trans woman PC returns `false` for `alwaysFemale()`, so she enters transformation branches. But within those branches, add a sub-branch for `w.hasTrait('TRANS_WOMAN')` where the emotional register differs.

Example for morning_routine.toml mirror scene:
```jinja
{% if not w.alwaysFemale() %}
  {% if w.hasTrait("TRANS_WOMAN") %}
    The bathroom mirror catches her before she's ready — and for once, what she sees is *right*. The face looking back is the one she always knew was there, underneath. She touches her cheek. Real skin. Real bone structure. No hormones, no surgery — just *her*, finally, completely.
  {% else %}
    The bathroom mirror is the first checkpoint of the day...
    *This is you now.* Not news. Just something that needs restating every morning...
  {% endif %}
{% else %}
  She catches her reflection. Mondays.
{% endif %}
```

Apply similar patterns to rain_shelter and coffee_shop scenes.

**Step 2: Update writing guide**

In `docs/writing-guide.md`, update the PC type section to document:
- The `PcOrigin` enum and its four variants
- `w.pcOrigin()` expression accessor
- `w.hasTrait('TRANS_WOMAN')` for emotional register branching
- Updated branching pattern showing the three-level gate (always_female → trans_woman → default transformed)
- Remove references to `always_female: bool` field

**Step 3: Validate templates**

Use `mcp__minijinja__jinja_validate_template` on each modified scene template.

**Step 4: Commit**

```
git add packs/base/scenes/ docs/writing-guide.md
git commit -m "content: add trans woman branches to scenes, update writing guide"
```

---

### Task 9: Update CLAUDE.md and HANDOFF.md

**Files:**
- Modify: `CLAUDE.md`
- Modify: `HANDOFF.md`

**Step 1: Update CLAUDE.md**

- Update "The Three PC Types" table to show four types with `PcOrigin` enum values
- Update the `FEMININITY` skill description for trans woman starting value
- Remove references to `always_female: bool`

**Step 2: Update HANDOFF.md**

- Update Current State to reflect PcOrigin system
- Add session log entry
- Move character creation redesign from Open Items (partially addressed)

**Step 3: Commit**

```
git add CLAUDE.md HANDOFF.md
git commit -m "docs: update CLAUDE.md and HANDOFF.md for PcOrigin system"
```

---

### Task 10: Final verification

**Step 1: Run full test suite**

Run: `cargo test --workspace`
Expected: All tests pass (should be ~104+ tests)

**Step 2: Run cargo clippy**

Run: `cargo clippy --workspace -- -D warnings`
Expected: No warnings

**Step 3: Build release**

Run: `cargo build --release`
Expected: PASS

**Step 4: Manual smoke test**

Launch the game, go through character creation with each of the four origin types. Verify:
- Two-step flow works correctly
- Form fields show/hide appropriately per origin
- Game starts successfully with each origin
- FEMININITY starts at correct value per origin

---

## Parallelization Notes

Tasks 1-2 are sequential (enum must exist before Player can use it).
Tasks 3-6 can run in parallel after Task 2 (each crate is independent).
Task 7 depends on Tasks 1-3 (needs PcOrigin + BeforeSexuality + CharCreationConfig).
Task 8 can run in parallel with Task 7 (scene content is independent of UI).
Task 9 runs after all code tasks are done.
Task 10 is final verification.

```
T1 → T2 → ┬→ T3 → T7 → T9 → T10
           ├→ T4 ──┘
           ├→ T5 ──┘
           ├→ T6 ──┘
           └→ T8 ──────┘
```
