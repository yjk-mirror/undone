# Overnight Autonomous Session Plan

> **For Claude:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` to execute this plan.
> Work in a worktree (`engineering-overnight`). Commit after each task. Update HANDOFF.md after each major chunk.
> Priority order matters — if time runs out, the most impactful work is done first.

**Goal:** Transform Undone from a tech demo into a playable game. Character creation, settings persistence, save/load UI, NE US names, proper opening scenes with real prose.

**Architecture:** All changes build on existing infrastructure. No new crates. The char creation UI adds a new view to `undone-ui` gated by an `AppTab::CharCreation` (or a pre-game flow). Settings persistence serializes `UserPrefs` to JSON. The save tab wires `undone-save` to a new view. Scenes are pure content — TOML files in `packs/base/scenes/`.

**Tech Stack:** Rust, floem, serde_json, toml, minijinja templates.

**Estimated scope:** 8–12 hours autonomous. 7 tasks in priority order.

---

## Task 1: Names Update — British → NE US

**Priority: Highest (quick win, 15 min)**

The current `packs/base/data/names.toml` has British names (Oliver, Harry, Poppy, Daisy). Replace with NE US names that fit the fictional Northeastern city.

**Files:**
- Modify: `packs/base/data/names.toml`

**Male names (30):** Pick from common American names with a Northeast flavor. Mix of traditional and contemporary. Think Boston/Philly/NYC metro — not Southern, not West Coast tech-bro.

Examples: Matt, Ryan, David, Mike, Chris, Jake, Tyler, Brandon, Nick, Dan, Kyle, Josh, Kevin, Brian, Sean, Derek, Marcus, Andre, James, Ben, Sam, Alex, Eric, Will, Tom, Adam, Nate, Carlos, Rob, Ian

**Female names (30):** For NPC spawning. Same NE US feel.

Examples: Jess, Sarah, Ashley, Megan, Lauren, Brittany, Alexis, Morgan, Taylor, Kayla, Courtney, Brooke, Amanda, Nicole, Rachel, Tiffany, Stephanie, Chelsea, Amber, Samantha, Andrea, Vanessa, Natasha, Diana, Carmen, Jade, Maya, Zoe, Hailey, Aria

**Criteria:**
- No overlap with the three default PC names (Eva, Ev, Evan)
- Diverse — not all WASPy Anglo names. The city is multicultural.
- 30 of each, matching current count.

**Test:** Run `cargo test -p undone-packs` — `loads_base_pack_names` should still pass.

**Commit:** `content: update names.toml — British names → NE US`

---

## Task 2: Settings Persistence

**Priority: High (quality of life, 1 hr)**

`UserPrefs` (theme mode, font size, line height) exists as a reactive signal but is lost on restart. Persist to a JSON file.

**Files:**
- Modify: `crates/undone-ui/src/theme.rs` — add `save_prefs()` and `load_prefs()` functions
- Modify: `crates/undone-ui/src/lib.rs` — load prefs at startup, save on change
- Modify: `crates/undone-ui/src/right_panel.rs` — call save after mode toggle

**Design:**

Save location: `dirs::config_dir()` / `undone/prefs.json`. Use the `dirs` crate (add to workspace if not present — check first).

Fallback: if `dirs` is not available, use `./undone_prefs.json` next to executable.

```rust
// theme.rs additions
use std::path::PathBuf;

fn prefs_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("undone").join("prefs.json"))
}

pub fn load_prefs() -> UserPrefs {
    prefs_path()
        .and_then(|p| std::fs::read_to_string(&p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_prefs(prefs: &UserPrefs) {
    if let Some(path) = prefs_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, serde_json::to_string_pretty(prefs).unwrap_or_default());
    }
}
```

Add `Serialize, Deserialize` derives to `UserPrefs` and `ThemeMode`.

In `app_view()`: replace `UserPrefs::default()` with `load_prefs()`.
In the mode toggle handler: call `save_prefs(&new_prefs)` after updating the signal.

**Dependencies:** Add `dirs = "6"` and `serde_json` (already in workspace) to `undone-ui/Cargo.toml`.

**Test:** Manual — change theme, restart app, theme should persist. Add a unit test for round-trip serialize/deserialize of `UserPrefs`.

**Commit:** `feat: persist UserPrefs to disk (theme, font size survive restart)`

---

## Task 3: Character Creation UI

**Priority: High (biggest impact, 3–4 hrs)**

Replace the hardcoded "Eva/Ev/Evan" placeholder with a character creation screen that appears before the game starts.

**Files:**
- Create: `crates/undone-ui/src/char_creation.rs` — the character creation view
- Modify: `crates/undone-ui/src/lib.rs` — add `CharCreation` to module list, change startup flow
- Modify: `crates/undone-ui/src/game_state.rs` — split init: load packs first, defer `new_game()` until after char creation

**Architecture:**

The app starts in a `CharCreation` phase, not directly into the game. The flow:

1. `init_game()` loads packs + registry but does NOT call `new_game()` yet
2. The app shows a character creation screen
3. Player fills in fields → clicks "Begin"
4. `new_game(config, &mut registry, &mut rng)` is called with the player's choices
5. App transitions to `AppTab::Game` and starts the opening scene

**AppSignals change:** Add a `phase: RwSignal<AppPhase>` where:
```rust
pub enum AppPhase {
    CharCreation,
    InGame,
}
```

The `dyn_container` in `app_view()` switches on `phase` first, then on `tab` when `InGame`.

**Character Creation Screen Layout:**

Use the existing theme system. The screen should feel warm and inviting — this is the player's first impression.

```
┌─────────────────────────────────────────────┐
│  Title Bar (same as game)                   │
├─────────────────────────────────────────────┤
│                                             │
│           Create Your Character             │
│                                             │
│  ┌─ Who You Are Now ──────────────────┐     │
│  │  Feminine name:  [Eva        ]     │     │
│  │  Androgynous:    [Ev         ]     │     │
│  │  Masculine name: [Evan       ]     │     │
│  │  Age:            [Early 20s  ▾]    │     │
│  │  Figure:         [Slim       ▾]    │     │
│  │  Breasts:        [Medium     ▾]    │     │
│  └────────────────────────────────────┘     │
│                                             │
│  ┌─ Your Past ────────────────────────┐     │
│  │  ☐ I was always female             │     │
│  │  (unchecked = transformed from     │     │
│  │   male — the primary experience)   │     │
│  │                                    │     │
│  │  Before age:     [28        ]      │     │
│  │  Sexuality:      [Straight  ▾]     │     │
│  └────────────────────────────────────┘     │
│                                             │
│  ┌─ Personality ──────────────────────┐     │
│  │  Pick 2–3 traits:                  │     │
│  │  ☐ Shy         ☐ Cute             │     │
│  │  ☐ Posh        ☐ Sultry           │     │
│  │  ☐ Down to Earth  ☐ Bitchy        │     │
│  │  ☐ Refined     ☐ Romantic         │     │
│  │  ☐ Flirty      ☐ Ambitious        │     │
│  │                                    │     │
│  │  ☐ Beautiful   ☐ Plain            │     │
│  │  (mutually exclusive)             │     │
│  └────────────────────────────────────┘     │
│                                             │
│  ┌─ Content Preferences ─────────────┐      │
│  │  ☐ Include rough/non-con content  │      │
│  │  ☐ I enjoy rougher content        │      │
│  └────────────────────────────────────┘     │
│                                             │
│           [ Begin Your Story ]              │
│                                             │
└─────────────────────────────────────────────┘
```

**Implementation notes:**

- Text inputs for names (floem `text_input` widget)
- Dropdowns for Age, Figure, Breasts, Sexuality (floem `dropdown` or styled buttons)
- Checkboxes for traits and preferences
- "Always female" checkbox hides/shows the "Your Past" section
- "Beautiful" and "Plain" are mutually exclusive (toggling one clears the other)
- BLOCK_ROUGH / LIKES_ROUGH are the hidden traits from the checkbox
- The "Begin" button collects all fields into `CharCreationConfig`, calls `new_game()`, transitions to `AppPhase::InGame`

**Floem widget constraints:**
- floem 0.2 has `text_input`, `checkbox`, `label`, `v_stack`, `h_stack`, `container`, `scroll`
- For dropdowns, use a simple cycling button or a list of radio-style buttons (floem 0.2 may not have a native dropdown widget — check and adapt)
- Style everything with the `ThemeColors` system

**Validation:**
- At least one name must be non-empty
- At least 1 trait selected (soft suggestion, not hard block)
- Before-age must be reasonable (18–60)

**When "Begin" is clicked:**
1. Resolve trait IDs from registry: `registry.resolve_trait("SHY")` etc.
2. Build `CharCreationConfig` from form state
3. Call `new_game(config, &mut registry, &mut rng)`
4. Store world in `GameState`
5. Set `phase` signal to `InGame`
6. Start opening scene

**Test:** At minimum, test that the config-to-new_game pipeline works (this is already tested in `char_creation::tests`). UI testing is visual — use screenshot-mcp if needed.

**Commit:** `feat: character creation UI — player configures identity before game starts`

---

## Task 4: Saves Tab UI

**Priority: Medium (quality of life, 1.5 hrs)**

Wire the existing `undone-save` crate to the Saves tab (currently a placeholder).

**Files:**
- Create: `crates/undone-ui/src/saves_panel.rs` — saves list view
- Modify: `crates/undone-ui/src/lib.rs` — replace Saves placeholder, add module

**Design:**

Save directory: `dirs::data_dir()` / `undone/saves/`. Create on first save.

The saves panel shows:
- A list of save files (sorted by modification time, newest first)
- Each entry shows: save name, date/time, player name
- Three buttons: **Save**, **Load**, **Delete**

```
┌─────────────────────────────────────┐
│  Saves                              │
│                                     │
│  [ Save Current Game ]              │
│                                     │
│  ┌─ Eva — Week 3 ────────────┐     │
│  │  Feb 23, 2026  10:47 PM   │     │
│  │  [Load]  [Delete]         │     │
│  └────────────────────────────┘     │
│  ┌─ Eva — Week 1 ────────────┐     │
│  │  Feb 23, 2026  9:12 PM    │     │
│  │  [Load]  [Delete]         │     │
│  └────────────────────────────┘     │
│                                     │
│  No more saves.                     │
│                                     │
└─────────────────────────────────────┘
```

**Implementation:**

- `save_dir()` returns `dirs::data_dir().join("undone/saves/")`
- `list_saves()` scans directory for `.json` files, reads metadata (player name, week) from each
- Save button: generates filename from player name + timestamp, calls `save_game()`
- Load button: calls `load_game()`, replaces world in `GameState`, restarts scene engine
- Delete button: confirm dialog, then delete file

**Save file naming:** `{player_name}_{timestamp}.json` e.g. `eva_20260223_224700.json`

**Metadata display:** Read the save file header (we can peek at the JSON to extract player name and week without full deserialization, or just deserialize and grab fields).

**Commit:** `feat: saves tab — list, save, load, delete game saves`

---

## Task 5: Rewrite Opening Scene — rain_shelter

**Priority: Medium (creative, 2 hrs)**

The current `rain_shelter.toml` is a placeholder — short prose, minimal branching, British vocabulary ("pavement", "Cheers"). Rewrite it as a proper opening scene following the writing guide.

**Files:**
- Modify: `packs/base/scenes/rain_shelter.toml`

**Design principles (from writing guide):**
- Second person, present tense, wry narrator voice
- American English (NE US setting)
- Something happens TO her before she chooses
- 2–3 genuine choices with different outcomes
- Trait branching that changes what HAPPENS, not adjectives
- Transformation dimension for non-always-female PCs
- At least one lasting consequence (game flag, NPC stat)

**Scene concept — keep rain shelter but make it real:**

The scene: you're caught in rain, duck into a bus shelter. There's a man already there. The rain is heavy enough that you're stuck for a few minutes. What happens in that small space?

**Rewrite goals:**
- Opening prose that sets the city, the weather, the mood — the world has texture
- 3–4 trait branches in the intro (SHY, POSH, CUTE, default) that change HOW she enters and what the man notices
- Transformation branch: she notices being looked at by a strange man in a confined space. For a former man, this registers differently. She knows that look.
- NPC action (umbrella offer) has personality-dependent dialogue
- Three player choices:
  1. Wait it out (safe, minimal interaction)
  2. Make a run for it (avoid the situation, stress cost)
  3. Accept umbrella / engage with him (relationship seed)
- Consequences: game flag `RAIN_SHELTER_MET` for future recognition, NPC liking change

**After rewrite:** Validate the minijinja templates in the prose fields.

**Commit:** `content: rewrite rain_shelter — proper prose, trait branching, transformation dimension`

---

## Task 6: Second Scene — Morning Routine

**Priority: Medium-Low (content, 2 hrs)**

Add a second scene that demonstrates the game's range. A domestic, low-stakes scene that's rich in transformation texture.

**Files:**
- Create: `packs/base/scenes/morning_routine.toml`
- Modify: `packs/base/data/schedule.toml` — add to schedule

**Scene concept — morning_routine:**

First morning of a new week. You wake up, get ready, face the mirror. This is the perfect vehicle for:
- Body unfamiliarity (low FEMININITY) vs. comfortable routine (high FEMININITY)
- Wardrobe choice (trait-dependent — POSH picks carefully, DOWN_TO_EARTH grabs whatever)
- The mirror moment (transformation texture — what does she see?)
- A small decision: coffee at home vs. grab something on the way (sets mood/money)

**Structure:**
- Intro: waking up, apartment details, weather through the window
- Choice 1: get dressed (trait-dependent prose, no mechanical effect)
- Choice 2: coffee at home (-0 money, +0 stress) vs. grab Dunkin' on the way (-5 money, -1 stress)
- The scene sets a game flag and finishes, leading to the scheduler picking a free_time scene

**Schedule integration:**
```toml
[[slot.events]]
scene     = "base::morning_routine"
condition = "gd.week() >= 1"
weight    = 15
```

Put it in a new slot called `morning` or just weight it into `free_time`.

**Commit:** `content: add morning_routine scene — domestic intro, transformation texture, NE US details`

---

## Task 7: Third Scene — Coffee Shop Encounter

**Priority: Low (content, 2 hrs, if time permits)**

A social encounter that introduces NPC interaction with personality-dependent dialogue.

**Files:**
- Create: `packs/base/scenes/coffee_shop.toml`
- Modify: `packs/base/data/schedule.toml` — add to schedule

**Scene concept:**

You stop at a coffee shop (local place, not chain — or make it a Dunkin' for NE US flavor). There's a guy ahead of you in line. He turns around. What happens depends on his personality and yours.

This scene demonstrates:
- NPC personality driving dialogue (JERK vs. ROMANTIC vs. CARING man)
- PC trait interaction (SHY + ROMANTIC man = flustered; BITCHY + JERK man = confrontation)
- Content gating (if LIKES_ROUGH, the JERK path can be more aggressive)
- Consequence: sets `MET_[NPC]` flag, changes NPC liking

**Commit:** `content: add coffee_shop scene — NPC personality interaction, trait branching`

---

## Execution Notes

**Order:** Tasks 1 → 2 → 3 → 4 → 5 → 6 → 7. If time runs out, the first 4 tasks deliver the most value.

**After each task:** Run `cargo test --workspace` and `cargo clippy`. Commit only if green.

**After all tasks:** Update HANDOFF.md with full session summary. Run `cargo test` one final time. Report final test count and status.

**Creative latitude:** For scene prose (Tasks 5–7), write with conviction. Follow the writing guide strictly. If in doubt, reread the anti-patterns list. The narrator is wry, specific, and American. The city feels real. The transformation is present when it's earned.

**What NOT to do:**
- Don't redesign the UI layout (sidebar left, story right is settled)
- Don't change the theme colors
- Don't modify the engine internals
- Don't add new crates to the workspace
- Don't write placeholder prose — if a scene is written, it should be real

**Reconciliation:** All work is in a single worktree on a single branch. If something needs adjustment, the user can revert individual commits. Scene content is the easiest to modify later — engine/UI changes are the structural bets.
