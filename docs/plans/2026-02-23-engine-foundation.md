# Engine Foundation Plan — Identity, Time, and the Gameplay Loop

**Date:** 2026-02-23
**Status:** Draft — needs user review before execution
**Goal:** Solidify the engine so that content writing can begin in earnest.
The engine must handle identity contrasts, time progression, activity selection,
and composable conditions — before we write a single new scene.

---

## Design Philosophy

This game's distinctive register is the **transformation contrast**. Every
socially-gendered experience asks: does this feel different for someone who
used to be a man? Who used to be outgoing but is now shy? Who used to be
white but is now Asian? Who aged up or down?

The engine must make these contrasts **first-class queryable data**, not
hardcoded branches. A scene author writes:

```
condition = "w.beforeRace() != w.getRace()"
```

...and it works for *any* race change, present or future. They write:

```
condition = "w.hadTraitBefore('OUTGOING') && w.hasTrait('SHY')"
```

...and it fires for exactly that contrast, composable with everything else.

The engine is a **platform**. Content is data. Every condition that could
apply to multiple routes must be expressible generically. Specific routes
are written by combining generic conditions, not by adding engine code.

---

## Part 1: Before/After Identity System

### Problem

The Player currently tracks:
- `before_age: u32` (raw number, not Age enum)
- `before_race: String`
- `before_sexuality: Option<BeforeSexuality>`

This is incomplete. The transformation changes *everything* about how the
world treats you. We need to track the full "before" identity:

- Before name (masculine)
- Before age (as Age enum, not u32)
- Before race
- Before sexuality
- Before personality traits
- Before figure (male figure)

And we need *all* of these queryable in conditions.

### Design

Add a `BeforeIdentity` struct to `undone-domain`:

```rust
/// Frozen snapshot of the player's pre-transformation identity.
/// Populated during character creation, immutable after transformation.
/// Only meaningful when `origin.has_before_life()` is true.
pub struct BeforeIdentity {
    pub name: String,           // masculine name
    pub age: Age,               // age category (not raw u32)
    pub race: String,
    pub sexuality: BeforeSexuality,
    pub figure: MaleFigure,
    pub traits: HashSet<TraitId>,  // personality traits before transformation
}
```

Replace the three flat fields on `Player` with:

```rust
pub before: Option<BeforeIdentity>,  // None for AlwaysFemale
```

### Expression Evaluator Additions

| Function | Returns | Description |
|---|---|---|
| `w.beforeName()` | String | Masculine name |
| `w.beforeAge()` | String | Age category as string (e.g. "EarlyTwenties") |
| `w.beforeRace()` | String | Race before transformation |
| `w.beforeSexuality()` | String | "AttractedToWomen" etc. |
| `w.getRace()` | String | Current race |
| `w.getAge()` | String | Current age category |
| `w.getAnxiety()` | i64 | Current anxiety level |
| `w.getArousal()` | String | Current arousal level name |
| `w.getAlcohol()` | String | Current alcohol level name |
| `w.hadTraitBefore(id)` | bool | Did the before-identity have this trait? |
| `w.wasMale()` | bool | Shorthand: origin is CisMale or TransWoman |
| `w.wasTransformed()` | bool | Shorthand: origin is not AlwaysFemale |

**Contrast conditions become natural:**

```toml
# Fires for anyone whose race changed
condition = "w.wasTransformed() && w.beforeRace() != w.getRace()"

# Fires for someone who was outgoing but is now shy
condition = "w.hadTraitBefore('OUTGOING') && w.hasTrait('SHY')"

# Fires for someone whose age group changed (older or younger)
condition = "w.wasTransformed() && w.beforeAge() != w.getAge()"

# Fires for someone who aged DOWN (was older, now younger)
condition = "w.wasTransformed() && w.beforeAge() > w.getAge()"

# Fires for any cis male who became a different race
condition = "w.pcOrigin() == 'CisMaleTransformed' && w.beforeRace() != w.getRace()"
```

No engine code changes needed for each new contrast — it's all composable
from the primitives.

### Save Format

`BeforeIdentity` serializes cleanly. Save format bumps to v3 with v2
migration (populate `BeforeIdentity` from the three flat fields + defaults
for missing data).

---

## Part 2: Trait System — Groups, Exclusion, and Categories

### Problem

Traits are currently an unstructured `HashSet<TraitId>`. There is no
mechanism for:
- Mutual exclusion (SHY and OUTGOING can't coexist)
- Trait groups (personality traits vs. physical traits vs. hidden traits)
- Trait categories (data-defined groupings for condition queries)

### Design

Extend `TraitDef` in pack data:

```toml
# packs/base/data/traits.toml

[[traits]]
id = "SHY"
name = "Shy"
description = "Withdrawn, avoids attention."
group = "personality"
conflicts = ["OUTGOING", "FLIRTY"]

[[traits]]
id = "OUTGOING"
name = "Outgoing"
description = "Social, confident, draws attention."
group = "personality"
conflicts = ["SHY"]

[[traits]]
id = "BEAUTIFUL"
name = "Beautiful"
description = "Conventionally attractive."
group = "appearance"
conflicts = ["PLAIN"]
```

Extend `TraitDef` struct:

```rust
pub struct TraitDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub hidden: bool,
    pub group: Option<String>,          // NEW: "personality", "appearance", etc.
    pub conflicts: Vec<String>,         // NEW: trait IDs that can't coexist
}
```

**Validation at pack load time:**
- If `conflicts` references an unknown trait ID → load error
- If player has both a trait and its conflict → load error (char creation validates)
- If `AddTrait` effect would create a conflict → runtime warning, effect skipped

### Categories (Data-Defined Groupings)

Categories are a separate concept from traits. They let scene authors
define *abstract groups* that can be queried. Example: a pack defines
"privileged" and "non-privileged" race categories.

```toml
# packs/base/data/categories.toml

[[categories]]
id = "RACE_PRIVILEGED"
description = "Socially privileged races in this setting"
type = "race"           # applies to race values
members = ["White"]

[[categories]]
id = "RACE_NON_PRIVILEGED"
description = "Non-privileged races in this setting"
type = "race"
members = ["Black", "Latina", "East Asian", "South Asian", "Mixed", "Other"]

[[categories]]
id = "AGE_YOUNG"
description = "Young age brackets"
type = "age"
members = ["LateTeen", "EarlyTwenties", "Twenties"]

[[categories]]
id = "AGE_MATURE"
description = "Mature age brackets"
type = "age"
members = ["Thirties", "Forties", "Fifties", "Old"]
```

**Expression evaluator additions:**

| Function | Returns | Description |
|---|---|---|
| `w.inCategory(cat_id)` | bool | Is the player's current value in this category? |
| `w.beforeInCategory(cat_id)` | bool | Was the before-value in this category? |

**Composable contrast queries:**

```toml
# Any non-privileged → privileged race change
condition = "w.beforeInCategory('RACE_NON_PRIVILEGED') && w.inCategory('RACE_PRIVILEGED')"

# Any privileged → non-privileged race change
condition = "w.beforeInCategory('RACE_PRIVILEGED') && w.inCategory('RACE_NON_PRIVILEGED')"

# Young → mature age shift
condition = "w.beforeInCategory('AGE_YOUNG') && w.inCategory('AGE_MATURE')"

# Mature → young age shift (de-aging)
condition = "w.beforeInCategory('AGE_MATURE') && w.inCategory('AGE_YOUNG')"
```

**Why categories instead of hardcoding:** When a new race is added to the
pack, the author adds it to the appropriate category. All existing scenes
that query `RACE_PRIVILEGED` / `RACE_NON_PRIVILEGED` automatically include
it. No scene files need to change. No engine code changes. This is the
composability guarantee.

Categories are pack data — different packs in different settings can define
different categories. A Japanese setting pack would categorize races
differently than the NE US base pack.

### Implementation Notes

- `PackRegistry` gets `categories: HashMap<String, CategoryDef>`
- `CategoryDef { id, description, category_type: CategoryType, members: Vec<String> }`
- `CategoryType` enum: `Race`, `Age`, `Trait`, `Personality` — determines
  which player field to check
- Validation at load time: all members must be valid values for their type
  (races must exist in the races list, ages must be valid Age variants, etc.)

---

## Part 3: Time System

### Problem

`game_data.week: u32` exists but never advances. There is no day/time
structure, no end-of-week effects, no sense of time passing.

### Design

```rust
// undone-domain/src/enums.rs
pub enum TimeSlot {
    Morning,
    Afternoon,
    Evening,
    Night,
}

// undone-world/src/lib.rs — additions to GameData
pub struct GameData {
    // ... existing fields ...
    pub week: u32,
    pub day: u8,              // 0–6 (Mon–Sun)
    pub time_slot: TimeSlot,
}
```

**Time advancement:**
- Each activity consumes one time slot
- After Night → advance to next day's Morning
- After Sunday Night → advance week, run `end_of_week()`
- `end_of_week()` on World: stress decay, salary credit, expense debit,
  pregnancy advancement, NPC relationship drift, arousal/alcohol reset

**Expression evaluator:**

| Function | Returns | Description |
|---|---|---|
| `gd.day()` | i64 | 0–6 (Mon–Sun) |
| `gd.timeSlot()` | String | "Morning" / "Afternoon" / "Evening" / "Night" |
| `gd.isWeekday()` | bool | Mon–Fri |
| `gd.isWeekend()` | bool | Sat–Sun |

**New effect:**

```rust
EffectDef::AdvanceTime { slots: u32 }  // skip N time slots forward
```

---

## Part 4: Activity Selection (The Gameplay Loop)

### Problem

The scheduler silently picks a scene. The player has no agency over what
they do with their time.

### Design: Hub Scene + Scheduler Slots

The activity selector is a **hub scene** — an "at home" / "plan your day"
scene that presents activity choices as actions. Each action routes to a
scheduler slot.

**New next-branch type: `slot`**

```toml
# packs/base/scenes/hub/plan_your_day.toml

[meta]
id = "base::plan_your_day"
description = "Choose what to do with your time."

[intro]
prose = """What do you want to do?"""

[[actions]]
id = "go_to_work"
label = "Go to work"
detail = "Another day at the office."
condition = "gd.isWeekday()"
[actions.next]
slot = "work"

[[actions]]
id = "go_shopping"
label = "Go shopping"
detail = "Browse the stores downtown."
[actions.next]
slot = "shopping"

[[actions]]
id = "go_out"
label = "Go out"
detail = "See what the city has in store."
[actions.next]
slot = "free_time"

[[actions]]
id = "stay_home"
label = "Stay home"
detail = "A quiet evening in."
[actions.next]
slot = "home_evening"

[[actions]]
id = "call_someone"
label = "Call someone"
detail = "Reach out to a contact."
condition = "!w.isSingle()"
[actions.next]
slot = "phone_call"
```

**Engine changes:**
- `NextBranchDef` gets `slot: Option<String>` alongside `goto`/`finish`
- When the engine resolves a `slot` branch, it calls
  `scheduler.pick(slot, world, registry, rng)` to get a scene, then starts it
- This is a small addition to the engine — the scheduler already exists

**After the scene finishes:**
- Time advances one slot
- If more time slots remain in the day → return to hub scene
- If day is over → advance day (or advance week if Sunday)
- The engine event `SceneFinished` triggers this logic in the UI/game loop

### Scheduler Enhancements

```toml
# packs/base/data/schedule.toml — enhanced format

[[events]]
slot = "free_time"
scene = "base::rain_shelter"
weight = 10
condition = "gd.week() < 4"
once_only = false           # can fire repeatedly (default)
required = false            # not mandatory this week (default)

[[events]]
slot = "free_time"
scene = "base::first_catcall"
weight = 0                  # weight 0 = triggered only, never random
trigger = "gd.week() >= 2 && !gd.hasGameFlag('FIRST_CATCALL_DONE')"
once_only = true            # fires once, then never again
```

**New scheduler fields:**
- `once_only: bool` — if true, auto-sets a game flag after firing so it
  never fires again
- `required: bool` — if true, this event MUST fire this week (queued into a
  mandatory event list, played before free choices)
- `trigger: Option<String>` — condition that, when first met, injects this
  scene into the next available slot (milestone events)

---

## Part 5: Missing Effects

Effects that the current system lacks but content writing needs:

| Effect | Description | Priority |
|---|---|---|
| `AddStuff { item }` | Add item to player inventory | High |
| `RemoveStuff { item }` | Remove item from inventory | High |
| `SetRelationship { npc, status }` | Change NPC relationship status | High |
| `SetNpcAttraction { npc, delta }` | Step NPC attraction level | High |
| `SetNpcBehaviour { npc, behaviour }` | Set NPC behaviour mode | Medium |
| `SetContactable { npc, value }` | Mark NPC as contactable | Medium |
| `AddSexualActivity { npc, activity }` | Record a sexual activity | Medium |
| `SetPlayerPartner { npc }` | Set player's partner | High |
| `AddPlayerFriend { npc }` | Add NPC to friends list | Medium |
| `SetJobTitle { title }` | Change player's job | Medium |
| `ChangeAlcohol { delta }` | Step alcohol level | Medium |
| `SetVirgin { value }` | Set/clear virginity | Medium |
| `AdvanceTime { slots }` | Skip time slots | Medium |

Each is a small addition to `EffectDef` enum + a handler in `effects.rs`.

---

## Part 6: Missing Evaluator Accessors

Condition functions needed for content writing:

| Function | Returns | Why needed |
|---|---|---|
| `w.beforeRace()` | String | Race contrast conditions |
| `w.beforeAge()` | String | Age contrast conditions |
| `w.beforeSexuality()` | String | Sexuality contrast conditions |
| `w.hadTraitBefore(id)` | bool | Personality contrast conditions |
| `w.getRace()` | String | Current race queries |
| `w.getAge()` | String | Current age queries |
| `w.getAnxiety()` | i64 | Anxiety threshold conditions |
| `w.getArousal()` | String | Arousal level conditions |
| `w.getAlcohol()` | String | Alcohol level conditions |
| `w.wasMale()` | bool | Origin shorthand |
| `w.wasTransformed()` | bool | Origin shorthand |
| `w.inCategory(cat)` | bool | Category membership (current) |
| `w.beforeInCategory(cat)` | bool | Category membership (before) |
| `gd.day()` | i64 | Day-of-week conditions |
| `gd.timeSlot()` | String | Time-of-day conditions |
| `gd.isWeekday()` | bool | Work day conditions |
| `gd.isWeekend()` | bool | Weekend conditions |
| `gd.getJobTitle()` | String | Employment conditions |
| `m.getLiking()` | String | NPC liking level (not just threshold) |
| `m.getLove()` | String | NPC love level |
| `m.getAttraction()` | String | NPC attraction level |
| `m.getBehaviour()` | String | NPC behaviour mode |
| `m.hasFlag(flag)` | bool | NPC relationship flags |

---

## Execution Plan — Session Breakdown

### Session A: Identity & Trait Foundation
1. `BeforeIdentity` struct on Player (replace 3 flat fields)
2. Trait groups and conflicts in `TraitDef`
3. Categories system (`categories.toml`, `CategoryDef`, registry)
4. Expression evaluator: all `w.before*()`, `w.get*()`, `w.inCategory()`,
   `w.hadTraitBefore()` functions
5. Save format v3 migration
6. Validation: trait conflicts at load time, category members at load time
7. Tests for all new evaluator functions and trait conflict validation

### Session B: Time & Activity Loop
1. `TimeSlot` enum, `day`/`time_slot` on GameData
2. `end_of_week()` stub on World
3. `GotoSlot` next-branch type in scene engine
4. Hub scene: `plan_your_day.toml`
5. Time advancement logic in UI game loop
6. Scheduler enhancements: `once_only`, `required`, `trigger`
7. Expression evaluator: `gd.day()`, `gd.timeSlot()`, etc.

### Session C: Missing Effects & Evaluator
1. All effects from Part 5
2. All evaluator accessors from Part 6
3. Tests for every new effect and accessor

### Session D: Character Creation Redesign
1. Male-first creation form (name, age, personality, race)
2. Transformation scene (mid-narrative)
3. Female customization as part of the story (defaults from before-identity)
4. `BeforeIdentity` populated at transformation time

### Future Sessions
- NPC archetype rules
- Clothing/inventory system
- Job/money weekly cycle
- Content writing (once the engine supports it all)

---

## Part 7: Writing Agent Tooling

Once the engine foundation is in place, writing agents need documentation
and validation tools so they can write scenes correctly without guessing.

### Writing Guide Updates

Update `docs/writing-guide.md` after each engine session with:

1. **Full condition vocabulary** — every `w.`, `m.`, `gd.`, `scene.`
   function, its return type, and a one-line description. Organized by
   receiver. This becomes the writing agent's API reference.

2. **Condition cookbook** — reusable patterns for common queries:
   - Race change: `w.wasTransformed() && w.beforeRace() != w.getRace()`
   - Personality contrast: `w.hadTraitBefore('X') && w.hasTrait('Y')`
   - Age shift (up): `w.wasTransformed() && w.beforeAge() < w.getAge()`
   - Age shift (down): `w.wasTransformed() && w.beforeAge() > w.getAge()`
   - Category transition: `w.beforeInCategory('X') && w.inCategory('Y')`
   - Any transformation: `w.wasTransformed()`
   - Compound: `(A || B) && (C || D)` — remind agents that full boolean
     logic with parentheses, `&&`, `||`, `!` is supported

3. **Effect reference** — every `EffectDef` variant, its TOML syntax, and
   what it modifies. Scene authors need to know what levers they can pull.

4. **Category reference** — list all defined categories from the base pack
   so authors know what `w.inCategory()` values are available. Note that
   categories are extensible — authors can define new ones in pack data.

### Condition Validator MCP Tool

Build a `validate_condition` MCP tool (or add it to the existing
rhai-mcp-server) that:

1. Parses a condition string through `undone-expr`'s parser
2. Reports syntax errors with line/column
3. Optionally validates referenced IDs against a pack registry snapshot
   (e.g., "trait 'OUTGONG' is not registered — did you mean 'OUTGOING'?")

This gives writing agents (and human authors) instant feedback on whether
a condition is well-formed before committing it to a scene file.

### Scene Validation Pass

Extend the existing cross-reference validation (which already checks
`goto` targets) to also validate:

- All condition strings parse without errors
- All trait/skill/stuff/stat IDs referenced in conditions exist in the registry
- All category IDs referenced in `inCategory()`/`beforeInCategory()` exist
- All effect trait_id/skill/stat references resolve
- Trait conflict violations (effect adds a trait that conflicts with an
  existing trait — warning, not error, since it may be intentional removal)

This runs at pack load time. A scene with a typo in a condition fails
fast with a clear error message, not a silent `false` at runtime.

### Writing Agent Skill

Create a `superpowers:writing-scenes` skill (or equivalent) that enforces:

1. Read the writing guide before writing any scene prose
2. After writing a `.toml` scene file, validate all conditions parse
3. After writing a `.j2` template, validate with `jinja_validate_template`
4. Check that all referenced IDs (traits, skills, categories) exist
5. Test the scene can be loaded by the pack loader without errors
6. Verify scene plays correctly by running through it in-game

The skill chains the existing MCP validation tools into a mandatory
workflow. Writing agents cannot claim a scene is done without running
the validation pass.

---

## Guiding Principles

**Composability:** Every new feature must satisfy: *"Can a scene author
express this as a TOML condition without touching Rust code?"* If yes, the
engine is doing its job. If no, we're missing a primitive.

**Arbitrary boolean logic:** The expression parser handles `&&`, `||`, `!`,
parentheses, comparisons, and method calls with unlimited nesting. Any
conceivable condition is expressible. The only constraint is what primitives
(method calls) are available — and adding a new primitive is a 10-line
function, not a structural change.

**Fail fast:** Invalid conditions, unknown IDs, and broken cross-references
fail at pack load time with clear error messages. A content author should
never discover a broken condition at runtime.

**Expandable without breakage:** Adding a new trait, race, category, or
evaluator function never invalidates existing content. Writing one route
must not break another. This is enforced by the data-driven design — the
engine doesn't know what game it runs.

The engine is a platform. Content is data. Composability is non-negotiable.
