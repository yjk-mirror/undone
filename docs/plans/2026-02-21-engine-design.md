# Undone — Engine Design

> **LIVING DOCUMENT.**
> This is scaffolding, not gospel. Decisions here represent the best thinking
> at project start. As implementation reveals surprises — and it will — update
> this document rather than working around it. Nothing here is load-bearing
> until it has tests.

---

## Project Overview

**Undone** is a life-simulation text game with adult themes. The player
character is a young woman navigating relationships, work, and social life in a
fictional Northeast US city. The game is a runtime for content — mechanics are
minimal, writing is everything.

Inspired by Newlife (Splendid Ostrich Games). Full redesign: new engine, new
content format, full ownership.

---

## Stack

| Concern | Choice | Rationale |
|---|---|---|
| Language | Rust | Type system pays dividends on complex game state |
| GUI | floem | Reactive, Lapce-team, pure Rust, single binary, cross-platform |
| Template rendering | minijinja | Jinja2 syntax, embeds cleanly, well-maintained |
| Scene conditions | Custom recursive descent parser | Zero dependencies, exact error messages, load-time validation |
| Serialisation | serde + serde_json + toml | Standard, excellent |
| NPC storage | slotmap | Stable typed keys across insert/delete, O(1) access |
| String interning | lasso | TraitId/SkillId/etc as u32, zero-cost comparison |

---

## Workspace Structure

```
undone/
├── Cargo.toml               # workspace root
├── src/
│   └── main.rs              # entry point, launches floem
│
├── crates/
│   ├── undone-domain/       # pure types — no IO, no game logic
│   ├── undone-world/        # World struct, all mutable game state
│   ├── undone-packs/        # pack loading, manifest parsing, content registry
│   ├── undone-expr/         # custom expression parser & evaluator
│   ├── undone-scene/        # scene execution engine
│   ├── undone-save/         # serde save / load
│   └── undone-ui/           # floem views and widgets
│
├── packs/
│   └── base/                # base game content (is itself a pack)
│       ├── pack.toml
│       ├── data/
│       │   ├── traits.toml
│       │   ├── npc_traits.toml
│       │   ├── skills.toml
│       │   └── stats.toml
│       ├── scenes/
│       └── ui/              # fonts, theme (packs can reskin)
│
└── docs/
    └── plans/
```

**Dependency direction** (enforced by workspace, no cycles):

```
undone-domain
    ↑
undone-world ← undone-packs
    ↑               ↑
undone-expr    undone-save
    ↑
undone-scene
    ↑
undone-ui
```

`undone-domain` has zero internal deps (only `serde`, `slotmap`, `lasso`).
Everything flows outward from it.

---

## Domain Model

### World

Single owner of all game state. Scene engine receives `&mut World`.
No shared mutable references anywhere — all cross-entity references use keys.

```rust
pub struct World {
    pub player: Player,
    pub male_npcs: SlotMap<MaleNpcKey, MaleNpc>,
    pub female_npcs: SlotMap<FemaleNpcKey, FemaleNpc>,
    pub game_data: GameData,
    pub week: u32,
}
```

### Player

```rust
pub struct Player {
    // Identity
    pub name: String,
    pub age: Age,
    pub race: String,
    pub figure: PlayerFigure,       // Slim | Toned | Womanly
    pub breasts: BreastSize,

    // Content-driven (data files, not hardcoded)
    pub traits: HashSet<TraitId>,
    pub skills: HashMap<SkillId, SkillValue>,  // { value, modifier }

    // State — engine-level enums
    pub money: i32,
    pub stress: i32,
    pub anxiety: i32,
    pub arousal: ArousalLevel,
    pub alcohol: AlcoholLevel,

    // Relationships (keys, never references)
    pub partner: Option<NpcKey>,
    pub friends: Vec<NpcKey>,

    // Life state
    pub virgin: bool,
    pub anal_virgin: bool,
    pub lesbian_virgin: bool,
    pub on_pill: bool,
    pub pregnancy: Option<PregnancyState>,

    // Inventory
    pub stuff: HashSet<StuffId>,

    // Per-character scene memory
    pub custom_flags: HashMap<String, String>,
    pub custom_ints: HashMap<String, i32>,

    // Transformation axis
    pub always_female: bool,   // false = male-start PC
}
```

### NPC Core (shared via composition)

```rust
pub struct NpcCore {
    pub name: String,
    pub age: Age,
    pub personality: PersonalityId,      // data-driven
    pub traits: HashSet<NpcTraitId>,

    // Relationship state — all engine-level enums
    pub relationship: RelationshipStatus,
    pub pc_liking: LikingLevel,
    pub npc_liking: LikingLevel,
    pub pc_love: LoveLevel,
    pub npc_love: LoveLevel,
    pub pc_attraction: AttractionLevel,
    pub npc_attraction: AttractionLevel,
    pub behaviour: Behaviour,

    // Memory
    pub relationship_flags: HashSet<String>,
    pub sexual_activities: HashSet<String>,
    pub custom_flags: HashMap<String, String>,
    pub custom_ints: HashMap<String, i32>,
    pub knowledge: i32,

    pub contactable: bool,
    pub arousal: ArousalLevel,
    pub alcohol: AlcoholLevel,
}

pub struct MaleNpc {
    pub core: NpcCore,
    pub figure: MaleFigure,
    pub clothing: MaleClothing,
    pub had_orgasm: bool,
}

pub struct FemaleNpc {
    pub core: NpcCore,
    pub char_type: CharTypeId,      // data-driven
    pub figure: PlayerFigure,
    pub breasts: BreastSize,
    pub clothing: FemaleClothing,
    pub pregnancy: Option<PregnancyState>,
    pub virgin: bool,
}
```

### Engine-Level Enums (closed sets, engine reasons about directly)

```rust
pub enum ArousalLevel    { Discomfort, Comfort, Enjoy, Close, Orgasm }
pub enum AlcoholLevel    { Sober, Tipsy, Drunk, VeryDrunk, MaxDrunk }
pub enum LikingLevel     { Neutral, Ok, Like, Close }
pub enum LoveLevel       { None, Some, Confused, Crush, Love }
pub enum AttractionLevel { Unattracted, Ok, Attracted, Lust }
pub enum Behaviour       { Neutral, Romantic, Mean, Cold, Faking }

pub enum RelationshipStatus {
    Stranger,
    Acquaintance,
    Friend,
    Partner { cohabiting: bool },
    Married,
    Ex,
}
```

### Content-Level IDs (interned strings, extensible from pack data files)

```rust
pub struct TraitId(pub lasso::Spur);
pub struct NpcTraitId(pub lasso::Spur);
pub struct SkillId(pub lasso::Spur);
pub struct PersonalityId(pub lasso::Spur);
pub struct CharTypeId(pub lasso::Spur);
pub struct StuffId(pub lasso::Spur);
pub struct StatId(pub lasso::Spur);
```

`lasso::Spur` is a `u32`. The global `Rodeo` interner lives in `PackRegistry`.
Unknown trait/skill names in scene files are caught at load time, not runtime.

### GameData

```rust
pub struct GameData {
    pub flags: HashSet<String>,
    pub stats: HashMap<StatId, i32>,
    pub job_title: String,
    pub allow_anal: bool,
}
```

---

## Pack System

The base game is a pack. Community content drops into `packs/`. The engine
loads all packs at startup in dependency order.

### Pack Manifest (`pack.toml`)

```toml
[pack]
id       = "base"
name     = "Base Game"
version  = "0.1.0"
author   = "Undone"
requires = []

[content]
traits      = "data/traits.toml"
npc_traits  = "data/npc_traits.toml"
skills      = "data/skills.toml"
scenes_dir  = "scenes/"
```

### Trait Definition File

```toml
[[trait]]
id          = "SHY"
name        = "Shy"
description = "Avoids eye contact, defers, gets flustered."
hidden      = false

[[trait]]
id          = "POSH"
name        = "Posh"
description = "Notices class signals. Faintly superior."
hidden      = false
```

All IDs interned into `PackRegistry` at load. Scene files that reference
unknown IDs fail with a clear error before the game runs.

### Scheduler Slots (`schedule.toml`)

Packs can add scenes to existing slots or define new ones:

```toml
[[slot]]
name        = "free_time"
weight_base = 10

  [[slot.events]]
  scene     = "base::rain_shelter"
  condition = "gd.week > 1"
  weight    = 5

  [[slot.events]]
  scene  = "base::corner_store"
  weight = 8
```

The scheduler evaluates conditions against `&World`, weights eligible scenes,
picks one. Community packs inject scenes by adding entries to slots.

---

## Scene Format (Approach B)

Single `.toml` file per scene. Prose inline as Jinja2 multi-line strings.
Effects are typed structs — validated at load time, zero runtime string parsing.

### Full Example

```toml
[scene]
id          = "base::rain_shelter"
pack        = "base"
description = "Caught in the rain at a bus shelter."

[intro]
next  = "main"
prose = """
The rain started ten minutes from home.
{% if w.hasTrait("SHY") %}
You position yourself at the far end of the shelter, eyes forward.
{% else %}
You nod at the man already there. He nods back.
{% endif %}
"""

[[actions]]
id                  = "main"
label               = "Wait it out"
detail              = "Stand here until it eases off."
allow_npc_actions   = true
use_default_actions = true

[[actions]]
id        = "leave"
label     = "Make a run for it"
detail    = "Get soaked. At least you'll be moving."
condition = "!scene.has_flag('offered_umbrella')"
prose     = "You pull your jacket over your head and step out into it."

  [[actions.effects]]
  type   = "change_stress"
  amount = 2

  [actions.next]
  finish = true

[[actions]]
id        = "accept_umbrella"
label     = "Share his umbrella"
condition = "scene.has_flag('umbrella_offered')"
prose     = """
{% if w.hasTrait("SHY") %}
You hesitate, then step closer. "Thanks," you manage.
{% else %}
You step under without much ceremony. "Cheers."
{% endif %}
"""
  [[actions.effects]]
  type  = "add_npc_liking"
  npc   = "m"
  delta = "small"

  [[actions.effects]]
  type = "set_game_flag"
  flag = "base::SHELTERED_WITH_STRANGER"

  [actions.next]
  finish = true

[[npc_actions]]
id        = "offers_umbrella"
condition = "!scene.has_flag('umbrella_offered')"
prose     = ""

  [[npc_actions.weight_multipliers]]
  condition = "m.hasTrait('CHARMING')"

  [[npc_actions.weight_divisors]]
  condition = "true"

  [[npc_actions.effects]]
  type = "set_scene_flag"
  flag = "umbrella_offered"
```

### Conditional Next Routing

```toml
# Unconditional
[actions.next]
goto = "action_id"

# Conditional — first matching branch wins
[[actions.next]]
goto = "flirt_back"
if   = "w.hasTrait('FLIRTY')"

[[actions.next]]
goto = "polite_decline"
# no 'if' = unconditional fallthrough
```

### Typed Effects Enum

```rust
pub enum Effect {
    // Scene state (session-local)
    SetSceneFlag    { flag: String },
    RemoveSceneFlag { flag: String },

    // Game state (persistent)
    SetGameFlag     { flag: String },
    RemoveGameFlag  { flag: String },
    AddStat         { stat: StatId, amount: i32 },
    SetStat         { stat: StatId, value: i32 },

    // Player
    ChangeStress    { amount: i32 },
    ChangeMoney     { amount: i32 },
    ChangeAnxiety   { amount: i32 },
    SkillIncrease   { skill: SkillId, amount: i32 },
    AddTrait        { trait_id: TraitId },
    RemoveTrait     { trait_id: TraitId },
    AddArousal      { delta: ArousalDelta },

    // NPC (npc = "m", "f", or named id from scene context)
    AddNpcLiking    { npc: String, delta: LikingDelta },
    AddNpcLove      { npc: String, delta: LoveDelta },
    AddWLiking      { npc: String, delta: LikingDelta },
    SetNpcFlag      { npc: String, flag: String },
    AddNpcTrait     { npc: String, trait_id: NpcTraitId },

    // Flow
    Transition      { target: SceneId },
    Finish,
}
```

---

## Expression Parser (`undone-expr`)

Conditions in TOML scene files are strings. Parsed at **pack load time** into a
typed AST. Invalid expressions reject the entire scene with a clear error.
Never evaluated as strings at runtime.

### Grammar

```
expr        = or_expr
or_expr     = and_expr ('||' and_expr)*
and_expr    = not_expr ('&&' not_expr)*
not_expr    = '!' not_expr | compare
compare     = call (('<' | '>' | '==' | '!=' | '<=' | '>=') call)?
call        = receiver '.' method '(' args? ')'
receiver    = 'w' | 'm' | 'f' | 'scene' | 'gd'
args        = value (',' value)*
value       = string | integer | bool
```

### Evaluation

```rust
fn eval(expr: &Expr, world: &World, ctx: &SceneCtx) -> Result<bool, EvalError>
```

`SceneCtx` carries: active male NPC key, active female NPC key, scene flags,
scene weighted map. Adding a new queryable method = one new match arm.
The compiler enforces exhaustive handling of all receiver/method combinations.

---

## Scene Engine (`undone-scene`)

`SceneDefinition` is immutable after load, wrapped in `Arc`, shared freely.
`SceneCtx` is mutable per-run session state.

### Execution Loop

```
1. Render intro prose   (minijinja, &World, &SceneCtx)
2. Evaluate action conditions → visible action list
3. If allow_npc_actions: weight NPC actions, pick one, render prose, apply effects
4. Player selects action
5. Render action prose
6. Apply typed effects  (&mut World, &mut SceneCtx)
7. Evaluate next routing → push new scene, goto action, or finish
8. Repeat from 2
```

Scene transitions push onto a **scene stack**. `finish = true` pops.
Sub-scenes work automatically — no special casing.

---

## Scheduler

Weekly timeslots. Each slot holds a weighted event pool. Conditions evaluated
against `&World`. Eligible scenes weighted and selected. Packs inject scenes
by contributing to slot definitions.

### `pick_next()` — unified cross-slot selection (implemented 2026-02-25)

The main entry point for advancing the game loop. Replaces slot-specific
`pick(slot_name, ...)` calls. Algorithm:

1. **Triggers first** — iterate all slots alphabetically. Return the first
   event whose `trigger` condition evaluates true (and hasn't been consumed
   if `once_only`). Order is deterministic.

2. **Weighted pick** — collect all eligible events from all slots (condition
   passes, weight > 0, not filtered by once_only). Random weighted selection.
   Arc slot events are gated by `gd.hasGameFlag('ROUTE_*')` conditions in the
   pack data — no engine hardcoding.

The `default_slot` field on `GameState` is removed. The engine no longer
needs to know which slot is "primary" — all slots compete equally.

### Arc circumstance flags

Arc content is driven by starting circumstance flags set by presets:
- `ROUTE_ROBIN` → workplace scenario (tech job)
- `ROUTE_CAMILA` → college scenario (enrolled student)

These are **pack content** (in `schedule.toml` conditions), not engine
concepts. Custom players start without these flags; their scenario can be
set by future gameplay or a starting-situation question in char creation.

Flags are fluid — future implementation can clear/set them as circumstances
change (quit job → clear ROUTE_ROBIN, take new job → set new flag).

---

## UI (`undone-ui`)

floem (Lapce reactive UI). Three-panel layout:

```
┌─────────────────────┬──────────────────────────────┐
│                     │                              │
│   STORY TEXT        │   CHARACTER / NPC INFO       │
│   (scrollable)      │   (stats, relationship,      │
│                     │    traits on hover)          │
│                     │                              │
├─────────────────────┴──────────────────────────────┤
│  [ Action A ]  [ Action B ]  [ Action C ]          │
└────────────────────────────────────────────────────┘
```

- Story text: floem text view, rich text, variable font size
- Action buttons: tooltip shows `detail` field
- Right panel: contextual — collapses when no NPC present
- Theme and fonts loaded from `packs/base/ui/` — packs can reskin

---

## Open Questions — RESOLVED (2026-02-22)

See `docs/plans/2026-02-22-design-decisions.md` for full rationale.

| Question | Decision |
|---|---|
| Cargo.toml dep versions | Pin at first stable release (not blocking) |
| `PersonalityId` — data-driven or engine enum? | Engine enum for 5 core archetypes; pack extensions remain interned strings |
| Save file versioning / migration | Current approach sufficient; migration framework at v0.1 |
| Character creation flow | Two-phase hybrid: narrative "Before" scene + configured form. Three-name system. |
| NPC spawning / pool seeding | Newlife model: 6–8 men + 2–3 women at game start, diversity guarantees |

### New fields required on `Player`

```rust
pub before_age: u32,
pub before_race: String,
pub before_sexuality: Sexuality,   // engine enum
pub name_masc: String,
pub name_androg: String,
pub name_fem: String,
```

### New engine enum required

```rust
pub enum Sexuality { StraightMale, GayMale, BiMale, AlwaysFemale }
pub enum Personality { Romantic, Jerk, Friend, Intellectual, Lad }
```

---

## Trait Design Philosophy (added 2026-02-24)

### All non-hidden traits are selectable in custom mode

Every trait defined with `hidden = false` appears in the character creation
personality picker. There is no category of traits that is "only for presets" or
"only for NPCs." Traits are shared vocabulary across the entire game system.

### Attitude traits are first-class

The trait groups include `"attitude"` in addition to `"personality"` and
`"appearance"`. Attitude traits — things like `SEXIST`, `HOMOPHOBIC`,
`OBJECTIFYING` — represent unexamined positions that the transformation will
confront. The game does not endorse them; it puts the player inside them and
shows what they encounter. Write them unflinchingly.

**Rules for attitude traits in scenes:**
- They are not required for a scene to be playable — scenes work with or without them
- When present they enrich: add specific interiority, add a specific register of
  discomfort or recognition, unlock paths that require that specific history
- Some scenes may be gated behind attitude traits — this is fine and intended
- NPCs can carry attitude traits too; NPC conditions check for them identically

### Preset characters guarantee content paths

Origin presets (Robin, Raul/Camila, future additions) are not alternative UIs —
they are **content guarantees**. Each preset is designed alongside a specific arc.
When a player picks Robin, they are picking into a set of scenes and branches that
are fully written for Robin's exact trait/origin combination.

As new arcs are written, new presets are added. Custom mode gives full access but
without content guarantees — custom characters may encounter scenes that have no
specific branch for their combination. That is acceptable.

### Race change is structurally present

The engine tracks `before_race` and current `race` independently. Scene templates
can branch on both. The most notable example: Robin was White and becomes East
Asian, and also carries `OBJECTIFYING`. Scenes involving the male gaze on East
Asian women can fire with specific interiority for Robin: she knows that gaze
because it was hers.

Scene conditions for race-change combinations follow this pattern:
- `w.player().race` — current race (string)
- `w.player().before.race` — before-transformation race (string, may be empty for AlwaysFemale)
- Both can appear in conditions alongside trait checks

### "Coming soon" greyout for initial release (not implemented in dev)

For the production initial release, the TransWomanTransformed and AlwaysFemale
origins on the character creation screen should be greyed out with a
"Coming soon" label. Content for these routes is not written yet. CisFemaleTransformed
may also need this treatment.

This is a **release-time UI concern only**. In the current dev build all origins
remain selectable — content gaps show as missing branches in existing scenes.

---

*Design session: 2026-02-21. Authors: YJK + Claude.*
*Trait philosophy section added: 2026-02-24.*
