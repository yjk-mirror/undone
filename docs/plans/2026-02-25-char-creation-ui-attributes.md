# Char Creation UI — Physical Attributes

Wire the new character attribute enums into the char creation UI so players can
customize their appearance. The data layer is complete (enums, Player struct,
save migration, accessors) — this plan adds the UI controls and data plumbing.

## Design Decisions

**Two-panel model** (matches existing flow):
- **BeforeCreation** ("Your Past"): before-body fields — height, eye colour, hair
  colour, skin tone, figure (MaleFigure), penis size. Only shown when origin is
  not AlwaysFemale.
- **FemCreation** ("Who Are You Now?"): after-body fields — all 12 new Player fields
  plus the existing figure/breasts. Organized into "Your Body" (figure, breasts,
  height, butt, waist, lips), "Appearance" (hair colour, hair length, eye colour,
  skin tone, complexion), and "Intimate" (nipple sensitivity, clit sensitivity,
  pubic hair, inner labia, wetness baseline).

**Defaults persist through transformation.** Height, eye colour, hair colour, skin
tone default to carrying over from the before-panel values. The player can change
them — that's part of the fantasy. New female-specific fields (breasts, nipple
sensitivity, etc.) use sensible midpoint defaults.

**PartialCharState expansion.** Before-panel fields need to flow through to
FemCreation so they can seed defaults and be stored on BeforeIdentity. Currently
PartialCharState only carries origin, name, age, race, sexuality, traits, arc_flag.
It needs to gain before_height, before_eye_colour, before_hair_colour,
before_skin_tone, before_figure, before_penis_size.

**CharCreationConfig expansion.** Currently only carries figure and breasts from the
fem panel. Needs all 12 new Player fields + the 5 BeforeIdentity fields so
`new_game()` can build the full Player.

---

## Tasks

### Task 1: Expand PartialCharState and CharCreationConfig

**File:** `crates/undone-ui/src/lib.rs` (PartialCharState)
**File:** `crates/undone-packs/src/char_creation.rs` (CharCreationConfig + new_game)

Add to `PartialCharState`:
```rust
pub before_height: Height,
pub before_eye_colour: EyeColour,
pub before_hair_colour: HairColour,
pub before_skin_tone: SkinTone,
pub before_figure: MaleFigure,
pub before_penis_size: PenisSize,
```

Add to `CharCreationConfig` (all the fem-panel fields that `new_game()` needs):
```rust
pub height: Height,
pub hair_length: HairLength,
pub hair_colour: HairColour,
pub eye_colour: EyeColour,
pub skin_tone: SkinTone,
pub complexion: Complexion,
pub butt: ButtSize,
pub waist: WaistSize,
pub lips: LipShape,
pub nipple_sensitivity: NippleSensitivity,
pub clit_sensitivity: ClitSensitivity,
pub pubic_hair: PubicHairStyle,
pub inner_labia: InnerLabiaSize,
pub wetness_baseline: WetnessBaseline,
```

Update `new_game()` to use these config fields instead of hardcoded defaults when
constructing the Player. Similarly, build `BeforeIdentity` from the before_* fields
on the config (via PartialCharState) instead of hardcoded `Height::Average` etc.

Update `base_config()` in tests to include the new fields.

**Verify:** `cargo check -p undone-packs`, `cargo test -p undone-packs`

### Task 2: Add before-body fields to BeforeCreation panel

**File:** `crates/undone-ui/src/char_creation.rs`

Add to `BeforeFormSignals`:
```rust
before_height: RwSignal<Height>,
before_eye_colour: RwSignal<EyeColour>,
before_hair_colour: RwSignal<HairColour>,
before_skin_tone: RwSignal<SkinTone>,
before_figure: RwSignal<MaleFigure>,
before_penis_size: RwSignal<PenisSize>,
```

Add dropdown rows to `section_your_past()` (inside the `was_male_bodied()` block):
- Height (5 variants)
- Eye colour (9 variants)
- Hair colour (13 variants)
- Skin tone (9 variants)
- Figure — MaleFigure (6 variants, already exists in domain)
- Penis size (8 variants)

Wire these into `build_next_button` so they populate `PartialCharState` and
`BeforeIdentity` on the throwaway world.

Also update preset data: add before_height, before_eye_colour, etc. to `PresetData`
struct and the `PRESET_ROBIN` / `PRESET_RAUL` constants (sensible values per
character docs).

**Verify:** `cargo check -p undone-ui`

### Task 3: Add after-body fields to FemCreation panel

**File:** `crates/undone-ui/src/char_creation.rs`

Add to `FemFormSignals`:
```rust
height: RwSignal<Height>,
hair_length: RwSignal<HairLength>,
hair_colour: RwSignal<HairColour>,
eye_colour: RwSignal<EyeColour>,
skin_tone: RwSignal<SkinTone>,
complexion: RwSignal<Complexion>,
butt: RwSignal<ButtSize>,
waist: RwSignal<WaistSize>,
lips: RwSignal<LipShape>,
nipple_sensitivity: RwSignal<NippleSensitivity>,
clit_sensitivity: RwSignal<ClitSensitivity>,
pubic_hair: RwSignal<PubicHairStyle>,
inner_labia: RwSignal<InnerLabiaSize>,
wetness_baseline: RwSignal<WetnessBaseline>,
```

Initialize defaults from `PartialCharState` (height, eye colour, hair colour,
skin tone carry over from before-panel; female-specific fields use midpoint
defaults).

Organize the "Your Body" section with three subsections:
1. **Shape** — Figure, Breasts, Height, Butt, Waist, Lips
2. **Appearance** — Hair colour, Hair length, Eye colour, Skin tone, Complexion
3. **Intimate** — Nipple sensitivity, Clit sensitivity, Pubic hair, Inner labia, Wetness

Each field is a `Dropdown::new_rw` with the full variant list, using the existing
`themed_trigger` / `themed_item` / `field_style` helpers.

Wire all signals into `build_begin_button` → `CharCreationConfig`.

**Verify:** `cargo check -p undone-ui`

### Task 4: Update test helpers across crates

All test `make_world()` / `base_config()` helpers that construct `CharCreationConfig`
need the new fields. Update:
- `crates/undone-packs/src/char_creation.rs` (tests)
- `crates/undone-ui/src/lib.rs` (test_player)

**Verify:** `cargo test --workspace` — all tests pass

### Task 5: Trait picker for new groups (hair, voice, eyes, body_detail, skin, scent)

**File:** `crates/undone-ui/src/char_creation.rs`

Add a new section to `fem_creation_view` (or optionally split between before/fem):
- **Hair texture** — radio group (pick one): Straight, Wavy, Curly, Coily
- **Voice** — radio group (pick one): Soft, Bright, Husky, Sweet, Breathy
- **Eyes** — radio group (pick one): Big, Narrow, Bright, Heavy-lidded, Almond
- **Body details** — checkbox grid (additive): Long legs, Wide hips, etc.
- **Skin** — checkbox grid (additive): Soft skin, Freckled, etc.
- **Scent** — radio group (pick one): Sweet, Musky, Clean

These are trait checkboxes, same pattern as the existing personality trait grid.
Conflicts are enforced: selecting one radio deselects the others in the group.

Wire selected trait IDs into `starting_traits`.

**Verify:** `cargo check -p undone-ui`

### Task 6: Sexual trait picker (optional, after BLOCK_ROUGH gate)

**File:** `crates/undone-ui/src/char_creation.rs`

Add sexual/sexual_preference trait pickers to FemCreation, only visible if
`include_rough` is true (carried from BeforeCreation content prefs). Section
heading: "Sexual Response" and "Sexual Preferences".

- **Sexual response** — checkbox grid: Hair trigger, Multi-orgasmic, etc.
- **Sexual preferences** — checkbox grid: Likes oral giving, etc.
- **Arousal response** — checkbox grid: Nipple getter, Flusher, etc.

Dark content traits are NOT in the picker — they are assigned by game content,
not player choice.

Conflicts enforced via mutual exclusion (same pattern as Beautiful/Plain).

Wire selected trait IDs into `starting_traits`.

**Verify:** `cargo check -p undone-ui`, `cargo test --workspace`

---

## Files Modified

| File | Change |
|---|---|
| `crates/undone-ui/src/lib.rs` | PartialCharState +6 fields, test_player update |
| `crates/undone-ui/src/char_creation.rs` | BeforeFormSignals +6, FemFormSignals +14, before/fem panel dropdowns, trait pickers, button wiring |
| `crates/undone-packs/src/char_creation.rs` | CharCreationConfig +14 fields, new_game() uses config fields, test base_config update |

## Verification

1. `cargo check` — clean compile
2. `cargo test --workspace` — all tests pass
3. `cargo run --release` — game launches, before-panel shows height/eye/hair/skin/figure/penis fields, fem-panel shows all 14 attribute dropdowns + trait pickers, selections flow through to the game world
