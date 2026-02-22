# Scene Engine Design

*Session: 2026-02-22. Authors: YJK + Claude.*

---

## Context

The scaffold is complete (30 tests, zero warnings). This document designs the first
post-scaffold session: a fully tested scene engine backend. No UI work — the goal is
a robust backend that fits the game's flow and writing, ready for any GUI frontend.

UI direction is still open (gtk4/relm4, slint, or floem under consideration). The
engine API is decoupled from UI via an event queue.

---

## Scope

Three crates are touched:

| Crate | Work |
|---|---|
| `undone-expr` | Wire eval stubs (`hasTrait`, `getSkill`, `getStat`) to real registry lookups |
| `undone-packs` | Add disk loader: scan packs dir, load data files, populate registry |
| `undone-scene` | Full implementation: scene types, effect system, engine, loader |

`undone-scene` is currently a placeholder (`// placeholder`).

---

## Scene Model: Flat Pool

At any moment, all `[[actions]]` entries whose `condition` evaluates to true are
presented to the player as choices. The action `id` is used for routing (`next.goto`),
not for grouping. `intro.next` is recorded as metadata but does not filter visible
actions — all condition-passing actions are shown.

This matches how Newlife scenes feel and is the simplest model to implement correctly.

---

## Scene TOML Format

Single `.toml` file per scene. Prose is inline minijinja. Effects are typed and
validated at load time.

```toml
[scene]
id          = "base::rain_shelter"
pack        = "base"
description = "Caught in the rain at a bus shelter."

[intro]
prose = """
The rain started ten minutes from home.
{% if w.hasTrait("SHY") %}
You position yourself at the far end, eyes forward.
{% else %}
You nod at the man already there.
{% endif %}
"""

[[actions]]
id    = "main"
label = "Wait it out"
detail = "Stand here until it eases off."
allow_npc_actions = true
# no [[actions.next]] → loops (re-evaluates all conditions)

[[actions]]
id        = "leave"
label     = "Make a run for it"
condition = "!scene.hasFlag('offered_umbrella')"
prose     = "You pull your jacket over your head and step out into it."

  [[actions.effects]]
  type   = "change_stress"
  amount = 2

  [[actions.next]]
  finish = true

[[actions]]
id        = "accept_umbrella"
label     = "Share his umbrella"
condition = "scene.hasFlag('umbrella_offered')"
prose     = "..."

  [[actions.effects]]
  type  = "add_npc_liking"
  npc   = "m"
  delta = 1

  [[actions.next]]
  finish = true

[[npc_actions]]
id        = "offers_umbrella"
condition = "!scene.hasFlag('umbrella_offered')"
weight    = 10

  [[npc_actions.effects]]
  type = "set_scene_flag"
  flag = "umbrella_offered"
```

### `[[actions.next]]` routing rules

- Empty (no `[[actions.next]]`) → loop: re-evaluate all conditions, re-show visible actions
- Single entry, no `if` → unconditional goto or finish
- Multiple entries → evaluated in order; first branch whose `if` passes (or has no `if`) wins

### NPC actions

`allow_npc_actions = true` on a player action means: after the player chooses that
action, NPC actions fire before the next choice screen is shown. NPC actions are
weighted — each has a `weight` (default 1) and optional `condition`. Eligible NPC
actions are weighted-randomly selected; one fires per turn.

---

## Crate Changes

### `undone-expr` — Registry Wiring

Add `registry: &PackRegistry` as 4th parameter to `eval`, `eval_call_bool`,
`eval_call_int`.

Wired stubs:

| Method | Resolution |
|---|---|
| `w.hasTrait("ID")` | `registry.resolve_trait(id)` → look up in `world.player.traits` |
| `m.hasTrait("ID")` | `registry.resolve_npc_trait(id)` → look up in NPC's trait set |
| `w.getSkill("ID")` | `registry.resolve_skill(id)` → `world.player.skill(id)` |
| `gd.getStat("ID")` | `registry.intern_stat(id)` → `world.game_data.stats.get(id)` |
| `w.hasStuff("ID")` | Remains stubbed — no stuff registry yet |

Add `EvalError::UnknownTrait`, `EvalError::UnknownSkill` variants for resolution
failures at runtime (should not happen if load-time validation is correct, but
must be handled).

### `undone-packs` — Disk Loader

New public function:

```rust
pub fn load_packs(packs_dir: &Path) -> Result<(PackRegistry, Vec<LoadedPackMeta>), PackLoadError>
```

`LoadedPackMeta` carries the parsed manifest plus the pack's root directory path
(needed by the scene loader to find `scenes_dir`).

Internally:
1. Walk `packs_dir` for `pack.toml` files
2. Parse each manifest
3. Load data files referenced in `[content]`: traits, npc_traits, skills
4. Register into `PackRegistry`
5. Return registry + metadata list

`PackLoadError` is a new error type covering IO errors, TOML parse errors, and
missing required files.

Scene loading is NOT done here — it lives in `undone-scene` to avoid a dep cycle.

### `undone-scene` — Full Implementation

#### Deserialization types (raw TOML shape)

```rust
SceneToml { scene: SceneMeta, intro: IntroDef, actions: Vec<ActionDef>,
            npc_actions: Vec<NpcActionDef> }

ActionDef { id, label, detail, condition: Option<String>, prose,
            allow_npc_actions, effects: Vec<EffectDef>, next: Vec<NextBranch> }

NpcActionDef { id, condition: Option<String>, prose, weight: u32,
               effects: Vec<EffectDef> }

NextBranch { condition (renamed "if"), goto: Option<String>, finish: bool }

// Tagged by `type` field, snake_case variants:
EffectDef { ChangeStress{amount}, ChangeMoney{amount}, ChangeAnxiety{amount},
            SetSceneFlag{flag}, RemoveSceneFlag{flag}, SetGameFlag{flag},
            RemoveGameFlag{flag}, AddStat{stat,amount}, SetStat{stat,value},
            SkillIncrease{skill,amount}, AddTrait{trait_id}, RemoveTrait{trait_id},
            AddArousal{delta}, AddNpcLiking{npc,delta}, AddNpcLove{npc,delta},
            AddWLiking{npc,delta}, SetNpcFlag{npc,flag}, AddNpcTrait{npc,trait_id},
            Transition{target} }
```

#### Resolved types (post-validation)

`SceneDefinition` (immutable, `Arc`-wrapped after loading):

```rust
pub struct SceneDefinition {
    pub id: String,
    pub pack: String,
    pub intro_prose: String,       // raw template string; rendered at runtime
    pub actions: Vec<Action>,
    pub npc_actions: Vec<NpcAction>,
}
```

`Action` and `NpcAction` hold parsed `Option<Expr>` conditions and `Vec<Effect>`.

`Effect` keeps string IDs for traits/skills/stats (validated at load, resolved via
registry at runtime when applied). This avoids storing registry references in the
scene definition.

#### Scene loader

```rust
pub fn load_scenes(
    scenes_dir: &Path,
    pack_id: &str,
    registry: &PackRegistry,
) -> Result<Vec<(String, Arc<SceneDefinition>)>, SceneLoadError>
```

For each `.toml` file in `scenes_dir`:
1. Deserialize into `SceneToml`
2. Parse all condition strings into `Expr` (fail-fast: unknown IDs rejected here)
3. Validate all effect string IDs against registry
4. Build `SceneDefinition`, wrap in `Arc`
5. Key by scene `id`

#### Effect application

```rust
pub fn apply_effect(
    effect: &Effect,
    world: &mut World,
    ctx: &mut SceneCtx,
    registry: &PackRegistry,
) -> Result<(), EffectError>
```

Ordinal level changes (arousal, liking, love) are clamped to their enum bounds.
Unknown IDs at apply time (should not occur post-validation) return `EffectError`.

#### SceneEngine

```rust
pub struct SceneEngine {
    scenes: HashMap<String, Arc<SceneDefinition>>,
    stack: Vec<SceneFrame>,
    events: VecDeque<EngineEvent>,
    rng: SmallRng,
}

struct SceneFrame {
    def: Arc<SceneDefinition>,
    ctx: SceneCtx,
}
```

Public API:

```rust
impl SceneEngine {
    pub fn new(scenes: HashMap<String, Arc<SceneDefinition>>) -> Self;

    /// Process a command. Queues events internally.
    pub fn send(&mut self, cmd: EngineCommand, world: &mut World, registry: &PackRegistry);

    /// Drain queued events. Call after send().
    pub fn drain(&mut self) -> Vec<EngineEvent>;
}
```

Commands and events:

```rust
pub enum EngineCommand {
    StartScene(String),    // scene_id
    ChooseAction(String),  // action_id
}

pub enum EngineEvent {
    ProseAdded(String),                // rendered minijinja output
    ActionsAvailable(Vec<ActionView>), // choices for the UI to show
    SceneFinished,                     // stack is empty
}

pub struct ActionView {
    pub id: String,
    pub label: String,
    pub detail: String,
}
```

#### Execution flow

`StartScene(id)`:
1. Look up `SceneDefinition`, push new `SceneFrame` with fresh `SceneCtx`
2. Render intro prose via minijinja → emit `ProseAdded`
3. Evaluate all action conditions → emit `ActionsAvailable`

`ChooseAction(id)`:
1. Find action by id in current frame
2. Render action prose (if any) → emit `ProseAdded`
3. Apply action effects
4. If `allow_npc_actions`: run NPC action selection, apply effects, emit prose
5. Evaluate `next` branches in order:
   - No branches → loop: re-evaluate conditions, emit `ActionsAvailable`
   - Branch with `finish = true` → pop frame; if stack empty emit `SceneFinished`,
     else re-emit parent frame's `ActionsAvailable`
   - Branch with `goto` → update frame's current routing hint; re-evaluate
     conditions, emit `ActionsAvailable`
   - Branch with `Transition` effect → push new frame for target scene

---

## Testing

### Unit tests

- **`undone-expr`**: `hasTrait` resolves against registry; `getSkill` returns effective
  value; bad trait ID returns `EvalError`; all existing tests pass with new signature
- **`undone-packs`**: `load_packs("packs/")` returns registry with known traits/skills;
  unknown pack dir returns error
- **`undone-scene` (types)**: parse rain shelter TOML; correct action count; conditions
  are `Some(Expr)`; effect types match

### Integration test

Full round-trip in `undone-scene` tests:

```
1.  load_packs("packs/") → registry
2.  load_scenes("packs/base/scenes/", "base", &registry) → scenes
3.  Create World; give player SHY trait (via registry.resolve_trait)
4.  Insert a male NPC; set ctx.active_male
5.  engine.send(StartScene("base::rain_shelter"))
6.  Assert ProseAdded contains shy-branch text
7.  Assert ActionsAvailable has "main" + "leave" (not "accept_umbrella" yet)
8.  engine.send(ChooseAction("main"))   // allow_npc_actions fires NPC
9.  Assert scene flag "umbrella_offered" is set in ctx
10. Assert ActionsAvailable now includes "accept_umbrella"
11. engine.send(ChooseAction("accept_umbrella"))
12. Assert SceneFinished emitted
13. Assert NPC pc_liking increased by 1 step
```

This test requires the rain shelter scene to exist in `packs/base/scenes/`. Creating
it is part of this session.

---

## Open Questions (deferred)

- `hasStuff` / stuff registry: no `stuff.toml` format defined yet
- `PersonalityId` in NPC: stays as data-driven ID for now
- Save/load: separate session
- Character creation: separate session
- Scheduler (weekly timeslots): separate session; `SceneEngine` is designed to be
  called from a scheduler once that exists
