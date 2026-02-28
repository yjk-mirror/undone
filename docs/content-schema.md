# Undone — Content Schema Reference

How game content is structured, from pack manifest to individual scene actions. Read this
before writing scenes, designing arcs, or building content tools.

---

## Layer Overview

```
Pack (pack.toml)
  ├── Data files (traits, skills, stats, arcs, categories, names, races)
  ├── Schedule (schedule.toml) → Slots → Events → scene IDs
  └── Scenes (scenes/*.toml)
        ├── Intro + IntroVariants + Thoughts
        ├── Actions → Effects + NextBranches + Thoughts
        └── NpcActions → Effects
```

---

## Pack Manifest (`pack.toml`)

```toml
[pack]
id       = "base"                    # unique pack ID, used as prefix: "base::scene_name"
name     = "Base Game"
version  = "0.1.0"
author   = "Undone"
requires = []                        # other pack IDs this pack depends on

opening_scene        = "base::rain_shelter"           # first scene for new games
transformation_scene = "base::transformation_intro"   # char creation intro scene
default_slot         = "free_time"                    # scheduler fallback slot

[content]
traits          = "data/traits.toml"
npc_traits      = "data/npc_traits.toml"
skills          = "data/skills.toml"
scenes_dir      = "scenes/"
schedule_file   = "data/schedule.toml"
names_file      = "data/names.toml"      # optional
stats_file      = "data/stats.toml"      # optional
races_file      = "data/races.toml"      # optional
categories_file = "data/categories.toml" # optional
arcs_file       = "data/arcs.toml"       # optional
```

---

## Data Files

### Traits (`data/traits.toml`)

```toml
[[trait]]
id          = "SHY"                    # all-caps string ID (interned at load time)
name        = "Shy"                    # display name
description = "Avoids eye contact..."  # UI tooltip
hidden      = false                    # hidden traits not shown in char creation
group       = "personality"            # "personality", "attitude", or "appearance"
conflicts   = ["OUTGOING", "FLIRTY"]   # traits that cannot coexist
```

Groups (13 total, 126 traits in the base pack):

| Group | Examples |
|---|---|
| `personality` | SHY, POSH, CUTE, OUTGOING, FLIRTY, … |
| `attitude` | SEXIST, HOMOPHOBIC, OBJECTIFYING, ANALYTICAL, CONFIDENT |
| `appearance` | PLAIN, BEAUTIFUL |
| `hair` | hair-related appearance traits |
| `voice` | voice quality traits |
| `eyes` | eye-related appearance traits |
| `body_detail` | detailed body appearance traits |
| `skin` | skin-related appearance traits |
| `scent` | scent traits |
| `sexual` | sexual characteristic traits |
| `sexual_preference` | attraction and preference traits |
| `dark_content` | opt-in content gate traits |
| `lactation` | lactation-related traits |
| `fertility` | fertility-related traits |
| `menstruation` | menstruation-related traits |
| `arousal_response` | arousal response traits |

Hidden traits are auto-injected by the engine based on PC origin (ALWAYS_FEMALE, NOT_TRANSFORMED,
BLOCK_ROUGH, LIKES_ROUGH). Do not inject hidden traits manually in UI or scene code.

### Skills (`data/skills.toml`)

```toml
[[skill]]
id  = "FEMININITY"
name = "Femininity"
description = "Adaptation to female identity."
min = -100
max = 100
```

FEMININITY is the primary writing dial. The base pack defines 48 skills covering body,
mind, social, and domestic domains. Other skills (FITNESS, CHARM, FASHION, etc.) exist
but are not yet used in scene content. Access any skill value in expressions with
`w.getSkill("SKILL_ID")`.

### Stats (`data/stats.toml`)

```toml
[[stat]]
id          = "TIMES_KISSED"
name        = "Times Kissed"
description = "Number of times the player has been kissed."
```

Counters incremented by `add_stat` effects. No min/max. The base pack defines 48 stats
tracking lifetime event counts (TIMES_KISSED, etc.). Currently unused in scene conditions.

### Arcs (`data/arcs.toml`)

```toml
[[arc]]
id            = "base::robin_opening"
states        = ["arrived", "week_one", "working", "settled"]
initial_state = "arrived"
```

Arcs are state machines. The `advance_arc` effect in scene actions transitions between states.
Arc state is checked in schedule conditions via `gd.arcState("base::robin_opening") == "arrived"`.

### Categories (`data/categories.toml`)

```toml
[[category]]
id      = "RACE_PRIVILEGED"
type    = "race"                         # race | age | trait | personality
members = ["White"]
```

Used in conditions: `gd.inCategory("RACE_PRIVILEGED", value)`.

---

## Schedule (`data/schedule.toml`)

The schedule controls **which scene fires next**. It consists of **slots** containing **events**.

### Slot

```toml
[[slot]]
name = "free_time"              # unique slot name
```

Slots group related events. Current slots: `free_time`, `robin_opening`, `camila_opening`.

### Event

```toml
  [[slot.events]]
  scene     = "base::rain_shelter"     # target scene ID
  condition = "gd.week() > 0"         # eligibility gate (expression)
  weight    = 10                       # probability weight (0 = trigger-only)
  once_only = false                    # if true, fires at most once
  trigger   = "..."                    # deterministic fire condition
```

### How `pick_next()` works

The scheduler evaluates ALL slots in two phases:

1. **Triggers first.** Scan every slot (alphabetical order) for events with `trigger`
   expressions. The first trigger that evaluates to `true` fires immediately — no RNG.
2. **Weighted random.** All events with `weight > 0` and passing `condition` across all
   slots are pooled. One is selected by weighted random.

### Condition vs. Trigger

- **`condition`**: Controls whether the event enters the weighted pick pool.
- **`trigger`**: Fires the scene deterministically, bypassing weighted selection.
- **`weight = 0` + `trigger`**: The idiom for mandatory narrative beats (invisible to
  the weighted pool, fire when their moment arrives).

### `once_only` mechanism

When `once_only = true` and a scene fires, the caller sets a persistent game flag
`ONCE_<scene_id>` (e.g. `ONCE_base::workplace_arrival`). On subsequent calls to
`pick_next()`, the scheduler filters out any event whose `ONCE_` flag is already set,
preventing the scene from firing again. This is fully implemented.

### Example: trigger-only mandatory scene

```toml
[[slot.events]]
scene     = "base::robin_arrival"
condition = "gd.hasGameFlag('ROUTE_ROBIN')"
weight    = 0                                  # never in weighted pool
trigger   = "gd.hasGameFlag('ROUTE_ROBIN')"   # fires immediately
once_only = true
```

### Example: weighted optional scene

```toml
[[slot.events]]
scene     = "base::coffee_shop"
condition = "gd.week() >= 1"
weight    = 10                                 # competes with other weighted events
once_only = false                              # can repeat
```

---

## Scene Files (`scenes/*.toml`)

### Scene metadata

```toml
[scene]
id          = "base::rain_shelter"
pack        = "base"
description = "Caught in rain, share a bus shelter with a stranger."
```

### Intro

```toml
[intro]
prose = """
Minijinja template. Full Jinja2 syntax.
{% if w.hasTrait("SHY") %}...{% endif %}
"""
```

Rendered against world state when the scene starts. This is what the player reads first.

### Intro Variants (`[[intro_variants]]`)

```toml
[[intro_variants]]
condition = "!w.alwaysFemale() && w.getSkill('FEMININITY') < 15"
prose = """...replacement intro..."""
```

Evaluated top-to-bottom. First match **completely replaces** the base intro. Use when the
entire opening needs to change based on PC state (not just a section within it).

### Thoughts (`[[thoughts]]`)

```toml
[[thoughts]]
condition = "!w.alwaysFemale()"
prose = "*Internal monologue...*"
style = "inner_voice"              # "inner_voice" (default) or "anxiety"
```

Fire automatically after intro, before actions are shown. Displayed in italics (inner_voice)
or with anxiety styling. Order is source order.

### Actions (`[[actions]]`)

```toml
[[actions]]
id                = "accept_umbrella"
label             = "Share his umbrella"          # button text
detail            = "Step closer. He offered."    # subtext in detail strip
condition         = "scene.hasFlag('umbrella_offered')"
allow_npc_actions = false                         # if true, NPC actions fire after
prose = """...result prose..."""
```

| Field | Type | Default | Meaning |
|---|---|---|---|
| `id` | String | required | Unique within scene |
| `label` | String | required | Choice button text |
| `detail` | String | `""` | Detail strip subtext |
| `condition` | Option | None | Hides action if false |
| `prose` | String | `""` | Displayed when chosen |
| `allow_npc_actions` | bool | false | NPC actions fire after this |
| `effects` | list | `[]` | Side effects |
| `next` | list | `[]` | Navigation after effects |
| `thoughts` | list | `[]` | Post-action inner monologue |

### Effects (`[[actions.effects]]`)

Tagged by `type`:

| Type | Fields | Description |
|---|---|---|
| `change_stress` | `amount: i32` | Modify stress |
| `change_money` | `amount: i32` | Modify money |
| `change_anxiety` | `amount: i32` | Modify anxiety |
| `set_scene_flag` | `flag: String` | Scene-local flag (cleared on exit) |
| `set_game_flag` | `flag: String` | Persistent game flag |
| `skill_increase` | `skill, amount` | Increase a skill |
| `add_trait` | `trait_id` | Add PC trait |
| `add_npc_liking` | `npc, delta` | NPC's liking of PC |
| `advance_arc` | `arc, to_state` | Transition arc state |
| `set_npc_role` | `npc, role` | Bind NPC role for scene |

See `crates/undone-scene/src/types.rs` `EffectDef` enum for the complete list.

### Next Branches (`[[actions.next]]`)

```toml
[[actions.next]]
if     = "scene.hasFlag('something')"   # optional condition
goto   = "action_id"                    # navigate to action within this scene
# OR:
slot   = "free_time"                    # let scheduler pick from slot
# OR:
finish = true                           # end scene, return to game loop
```

Evaluated top-to-bottom. First matching branch is taken.

### NPC Actions (`[[npc_actions]]`)

```toml
[[npc_actions]]
id        = "umbrella_offer"
condition = "!scene.hasFlag('umbrella_offered')"
weight    = 12
prose = """..."""
```

Fire when a player action has `allow_npc_actions = true`. All eligible NPC actions are
weighted-random selected. Their effects and prose are applied. NPC actions cannot navigate
(no `next` branches).

---

## Physical Attribute Accessors

All `w.*` accessors below are available in **both** minijinja templates (`{% if w.getHeight() == "Tall" %}`)
and condition expressions (`w.getHeight() == 'Tall'`). They return the Debug variant name of the
underlying Rust enum as a string, exactly as listed.

### Current Physical Attributes

| Accessor | Return values |
|---|---|
| `w.getHeight()` | `"VeryShort"`, `"Short"`, `"Average"`, `"Tall"`, `"VeryTall"` |
| `w.getFigure()` | `"Petite"`, `"Slim"`, `"Athletic"`, `"Hourglass"`, `"Curvy"`, `"Thick"`, `"Plus"` |
| `w.getBreasts()` | `"Flat"`, `"Perky"`, `"Handful"`, `"Average"`, `"Full"`, `"Big"`, `"Huge"` |
| `w.getButt()` | `"Flat"`, `"Small"`, `"Pert"`, `"Round"`, `"Big"`, `"Huge"` |
| `w.getWaist()` | `"Tiny"`, `"Narrow"`, `"Average"`, `"Thick"`, `"Wide"` |
| `w.getLips()` | `"Thin"`, `"Average"`, `"Full"`, `"Plush"`, `"BeeStung"` |
| `w.getHairColour()` | `"Black"`, `"DarkBrown"`, `"Brown"`, `"Chestnut"`, `"Auburn"`, `"Copper"`, `"Red"`, `"Strawberry"`, `"Blonde"`, `"HoneyBlonde"`, `"PlatinumBlonde"`, `"Silver"`, `"White"` |
| `w.getHairLength()` | `"Buzzed"`, `"Short"`, `"Shoulder"`, `"Long"`, `"VeryLong"` |
| `w.getEyeColour()` | `"Brown"`, `"DarkBrown"`, `"Hazel"`, `"Green"`, `"Blue"`, `"LightBlue"`, `"Grey"`, `"Amber"`, `"Black"` |
| `w.getSkinTone()` | `"VeryFair"`, `"Fair"`, `"Light"`, `"Medium"`, `"Olive"`, `"Tan"`, `"Brown"`, `"DarkBrown"`, `"Deep"` |
| `w.getComplexion()` | `"Clear"`, `"Normal"`, `"Rosy"`, `"Acne"` |
| `w.getRace()` | The player's race string (e.g. `"White"`, `"Black"`, etc.) |
| `w.getAge()` | `"LateTeen"`, `"EarlyTwenties"`, `"MidLateTwenties"`, `"LateTwenties"`, `"Thirties"`, `"Forties"`, `"Fifties"`, `"Old"` |
| `w.getNippleSensitivity()` | `"Low"`, `"Normal"`, `"High"`, `"Extreme"` |
| `w.getClitSensitivity()` | `"Low"`, `"Normal"`, `"High"`, `"Extreme"` |
| `w.getPubicHair()` | `"Natural"`, `"Trimmed"`, `"Landing"`, `"Brazilian"`, `"Bare"` |
| `w.getInnerLabia()` | `"Small"`, `"Average"`, `"Prominent"` |
| `w.getWetness()` | `"Dry"`, `"Normal"`, `"Wet"`, `"Soaking"` |

### Before-Life Accessors

These accessors return the PC's physical attributes from before transformation. They return an empty
string (`""`) if the PC has no before-identity (i.e. `w.alwaysFemale()` is true). Always guard
with `{% if not w.alwaysFemale() %}` before using them in prose.

| Accessor | Return values |
|---|---|
| `w.beforeHeight()` | Height variant string (same values as `w.getHeight()`) |
| `w.beforeHairColour()` | HairColour variant string (same values as `w.getHairColour()`) |
| `w.beforeEyeColour()` | EyeColour variant string (same values as `w.getEyeColour()`) |
| `w.beforeSkinTone()` | SkinTone variant string (same values as `w.getSkinTone()`) |
| `w.beforePenisSize()` | `"None"`, `"Micro"`, `"Small"`, `"Average"`, `"AboveAverage"`, `"Big"`, `"Huge"`, `"Massive"` |
| `w.beforeFigure()` | `"Average"`, `"Skinny"`, `"Toned"`, `"Muscular"`, `"Thickset"`, `"Paunchy"` |

### Usage pattern

In minijinja templates:

```jinja
{% if not w.alwaysFemale() %}
  {% if w.beforeHeight() == "Tall" and w.getHeight() == "Average" %}
You used to tower over most people. Now you slot into the middle of any crowd.
  {% endif %}
{% endif %}
```

In condition expressions (schedule or scene `condition` / `trigger` fields):

```
!w.alwaysFemale() && w.getHeight() == 'Tall'
```

**Important:** String comparisons in condition expressions use single quotes. In minijinja
templates use double quotes (standard Jinja2 string syntax).

---

## Expression Language

All `condition`, `trigger`, and `if` fields use the custom expression parser.

| Object | Key methods |
|--------|-------------|
| `w.` | `hasTrait("ID")`, `getSkill("ID")`, `getMoney()`, `getStress()`, `alwaysFemale()`, `isVirgin()`, `isSingle()`, plus all physical attribute accessors (`getHeight()`, `getFigure()`, `getBreasts()`, etc.) and before-life accessors (`beforeHeight()`, `beforeFigure()`, etc.) — see [Physical Attribute Accessors](#physical-attribute-accessors) above |
| `gd.` | `hasGameFlag("FLAG")`, `week()`, `day()`, `timeSlot()`, `arcState("arc_id")`, `isWeekday()` |
| `scene.` | `hasFlag("FLAG")` |
| `m.` | `hasTrait("ID")`, `isPartner()`, `isFriend()` (NPC receiver) |

Operators: `&&`, `||`, `!`, `==`, `!=`, `<`, `>`, `<=`, `>=`. String literals use single quotes.

---

## Cross-Reference Validation

| Reference | Validated at |
|---|---|
| Trait IDs in effects | Scene load time |
| Skill IDs in effects | Scene load time |
| Arc IDs + states in effects | Scene load time |
| `goto` targets | Post-load cross-reference pass |
| Condition expression syntax | Scene load time |
| Schedule event → scene ID | Runtime only |
| Stat IDs in effects | Runtime only (not validated) |

---

## Interaction Flow Example

```
1. Game starts → opening_scene fires ("base::rain_shelter")
2. Scene loads → intro prose rendered → thoughts fire → actions shown
3. Player picks "Wait it out" (allow_npc_actions = true)
4. Engine rolls NPC actions → "umbrella_offer" wins (weight 12)
5. NPC action sets scene flag "umbrella_offered"
6. Action list updates → accept/decline now visible, "leave" hidden
7. Player picks "Accept umbrella"
8. Effects: add_npc_liking +1, set_game_flag "RAIN_SHELTER_MET"
9. Next: finish = true → scene ends → scheduler.pick_next() → next scene
```
