# Engine & Routes Foundation — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task.
> Also required at start: `superpowers:using-git-worktrees` — create a worktree for this plan.

**Goal:** Add skill checks, inner voices, arc system, NPC roles, narrator variants, and validation tooling to the engine; write world/character documentation; then write the opening arcs for Robin and Camila.

**Architecture:** The engine gains a check system (percentile-based, visible/invisible, red/white, multi-trait) using `RefCell`-cached rolls in `SceneCtx`. A new arc system tracks multi-scene storylines in `GameData`. Content is structured around two origin characters (Robin, Camila) whose route-specific scenes compose with universal scenes via game flags and arc state conditions.

**Tech Stack:** Rust workspace (7 crates), minijinja templates, TOML scene format, serde/serde_json for save, slotmap for NPCs, lasso for interned IDs.

**Read before starting:**
- `docs/writing-guide.md` — all prose must pass its checklist
- `HANDOFF.md` — current state and engineering guardrails
- `CLAUDE.md` — engineering principles (no tech debt, fail loud, no hardcoded content IDs)

---

## PART 1: ENGINE FOUNDATIONS
### Sessions 1A–1C · Pure Rust · TDD throughout

---

### Task 1: Skill roll cache in SceneCtx

**Why first:** Every check method reads from this cache. Must exist before any evaluator changes.

**Files:**
- Modify: `crates/undone-expr/src/eval.rs` (SceneCtx struct)

**Step 1: Add `RefCell<HashMap>` to SceneCtx**

```rust
// In eval.rs, update SceneCtx:
use std::cell::RefCell;

pub struct SceneCtx {
    pub active_male: Option<MaleNpcKey>,
    pub active_female: Option<FemaleNpcKey>,
    pub scene_flags: HashSet<String>,
    pub weighted_map: HashMap<String, i32>,
    pub skill_rolls: RefCell<HashMap<String, i32>>,  // cached per-scene rolls
}

impl SceneCtx {
    pub fn new() -> Self {
        Self {
            active_male: None,
            active_female: None,
            scene_flags: HashSet::new(),
            weighted_map: HashMap::new(),
            skill_rolls: RefCell::new(HashMap::new()),
        }
    }

    pub fn has_flag(&self, flag: &str) -> bool {
        self.scene_flags.contains(flag)
    }

    pub fn set_flag(&mut self, flag: impl Into<String>) {
        self.scene_flags.insert(flag.into());
    }

    /// Force a specific roll for testing. Call before evaluating checkSkill.
    pub fn set_skill_roll(&self, skill_id: &str, roll: i32) {
        self.skill_rolls.borrow_mut().insert(skill_id.to_string(), roll);
    }

    /// Get cached roll or generate and cache a new one (1–100).
    pub fn get_or_roll_skill(&self, skill_id: &str) -> i32 {
        let mut rolls = self.skill_rolls.borrow_mut();
        *rolls.entry(skill_id.to_string()).or_insert_with(|| {
            rand::thread_rng().gen_range(1_i32..=100)
        })
    }
}
```

**Step 2: Add `rand` import to eval.rs** (it's already in Cargo.toml via undone-scene, check `undone-expr/Cargo.toml` — add `rand = { workspace = true }` if missing)

**Step 3: Write tests**

```rust
#[test]
fn set_and_get_skill_roll_returns_same_value() {
    let ctx = SceneCtx::new();
    ctx.set_skill_roll("CHARM", 42);
    assert_eq!(ctx.get_or_roll_skill("CHARM"), 42);
}

#[test]
fn get_or_roll_is_idempotent_without_set() {
    let ctx = SceneCtx::new();
    let first = ctx.get_or_roll_skill("FITNESS");
    let second = ctx.get_or_roll_skill("FITNESS");
    assert_eq!(first, second); // same roll cached
}

#[test]
fn different_skills_get_independent_rolls() {
    let ctx = SceneCtx::new();
    ctx.set_skill_roll("CHARM", 30);
    ctx.set_skill_roll("FITNESS", 80);
    assert_eq!(ctx.get_or_roll_skill("CHARM"), 30);
    assert_eq!(ctx.get_or_roll_skill("FITNESS"), 80);
}
```

**Step 4:** `cargo test -p undone-expr` — all pass

**Step 5:** `cargo fmt -p undone-expr`

**Commit:** `feat(expr): add skill roll cache to SceneCtx`

---

### Task 2: checkSkill() evaluator method

**Formula:** success if `roll <= clamp(skill_value + (50 - dc), 5, 95)`
- skill=50, dc=50 → 50% chance
- skill=70, dc=50 → 70% chance
- skill=30, dc=70 → 10% chance
- Always at least 5%, never more than 95%

**Files:**
- Modify: `crates/undone-expr/src/eval.rs` (eval_call_bool, Receiver::Player arm)

**Step 1: Add checkSkill to Player arm in eval_call_bool**

```rust
// Inside match call.method.as_str() for Receiver::Player:
"checkSkill" => {
    let skill_id_str = str_arg(0)?;
    let dc = match call.args.get(1) {
        Some(Value::Int(n)) => *n as i32,
        _ => return Err(EvalError::BadArg("checkSkill".into())),
    };
    let skill_id = registry
        .resolve_skill(skill_id_str)
        .map_err(|_| EvalError::UnknownSkill(skill_id_str.to_string()))?;
    let skill_value = world.player.skill(skill_id) as i32;
    let roll = ctx.get_or_roll_skill(skill_id_str);
    let target = (skill_value + (50 - dc)).clamp(5, 95);
    Ok(roll <= target)
}
```

**Step 2: Write tests**

```rust
#[test]
fn checkSkill_succeeds_when_roll_below_target() {
    let mut reg = undone_packs::PackRegistry::new();
    reg.register_skills(vec![undone_packs::SkillDef {
        id: "CHARM".into(), name: "Charm".into(),
        description: "".into(), min: 0, max: 100,
    }]);
    let skill_id = reg.resolve_skill("CHARM").unwrap();
    let mut world = make_world();
    world.player.skills.insert(skill_id, SkillValue { value: 60, modifier: 0 });
    // skill=60, dc=50 → target=60. roll=40 → success
    let ctx = SceneCtx::new();
    ctx.set_skill_roll("CHARM", 40);
    let expr = parse("w.checkSkill('CHARM', 50)").unwrap();
    assert!(eval(&expr, &world, &ctx, &reg).unwrap());
}

#[test]
fn checkSkill_fails_when_roll_above_target() {
    let mut reg = undone_packs::PackRegistry::new();
    reg.register_skills(vec![undone_packs::SkillDef {
        id: "CHARM".into(), name: "Charm".into(),
        description: "".into(), min: 0, max: 100,
    }]);
    let skill_id = reg.resolve_skill("CHARM").unwrap();
    let mut world = make_world();
    world.player.skills.insert(skill_id, SkillValue { value: 60, modifier: 0 });
    // skill=60, dc=50 → target=60. roll=80 → fail
    let ctx = SceneCtx::new();
    ctx.set_skill_roll("CHARM", 80);
    let expr = parse("w.checkSkill('CHARM', 50)").unwrap();
    assert!(!eval(&expr, &world, &ctx, &reg).unwrap());
}

#[test]
fn checkSkill_tiered_uses_same_roll() {
    // Tiered checks: evaluate hard→easy, all use same cached roll.
    // roll=65, target_hard=40 (skill=60,dc=70 → 40), target_easy=80 (skill=60,dc=30 → 80)
    let mut reg = undone_packs::PackRegistry::new();
    reg.register_skills(vec![undone_packs::SkillDef {
        id: "CHARM".into(), name: "Charm".into(),
        description: "".into(), min: 0, max: 100,
    }]);
    let skill_id = reg.resolve_skill("CHARM").unwrap();
    let mut world = make_world();
    world.player.skills.insert(skill_id, SkillValue { value: 60, modifier: 0 });
    let ctx = SceneCtx::new();
    ctx.set_skill_roll("CHARM", 65);
    // dc=70 → target=40 → 65>40 → FAIL
    let hard = parse("w.checkSkill('CHARM', 70)").unwrap();
    assert!(!eval(&hard, &world, &ctx, &reg).unwrap());
    // dc=30 → target=80 → 65<=80 → SUCCESS
    let easy = parse("w.checkSkill('CHARM', 30)").unwrap();
    assert!(eval(&easy, &world, &ctx, &reg).unwrap());
}

#[test]
fn checkSkill_minimum_5_percent_chance() {
    let mut reg = undone_packs::PackRegistry::new();
    reg.register_skills(vec![undone_packs::SkillDef {
        id: "CHARM".into(), name: "Charm".into(),
        description: "".into(), min: 0, max: 100,
    }]);
    let mut world = make_world();
    let skill_id = reg.resolve_skill("CHARM").unwrap();
    world.player.skills.insert(skill_id, SkillValue { value: 0, modifier: 0 });
    // skill=0, dc=100 → raw target=-50 → clamped to 5. roll=4 → success
    let ctx = SceneCtx::new();
    ctx.set_skill_roll("CHARM", 4);
    let expr = parse("w.checkSkill('CHARM', 100)").unwrap();
    assert!(eval(&expr, &world, &ctx, &reg).unwrap());
}
```

**Step 3:** `cargo test -p undone-expr` — all pass

**Step 4:** `cargo fmt -p undone-expr`

**Commit:** `feat(expr): add w.checkSkill(skill, dc) percentile check`

---

### Task 3: Red check failure tracking in GameData

Red checks are one-shot: once failed, permanently blocked. Key format: `"scene_id::skill_id"`.

**Files:**
- Modify: `crates/undone-world/src/game_data.rs`

**Step 1: Add field with serde default (backward-compatible with v3 saves)**

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameData {
    pub flags: HashSet<String>,
    pub stats: HashMap<StatId, i32>,
    pub job_title: String,
    pub allow_anal: bool,
    pub week: u32,
    #[serde(default)]
    pub day: u8,
    #[serde(default = "default_time_slot")]
    pub time_slot: TimeSlot,
    #[serde(default)]
    pub arc_states: HashMap<String, String>,      // arc_id → state name
    #[serde(default)]
    pub red_check_failures: HashSet<String>,      // "scene_id::skill_id"
}

impl GameData {
    // ... existing methods ...

    pub fn fail_red_check(&mut self, scene_id: &str, skill_id: &str) {
        self.red_check_failures.insert(format!("{scene_id}::{skill_id}"));
    }

    pub fn has_failed_red_check(&self, scene_id: &str, skill_id: &str) -> bool {
        self.red_check_failures.contains(&format!("{scene_id}::{skill_id}"))
    }

    pub fn arc_state(&self, arc_id: &str) -> Option<&str> {
        self.arc_states.get(arc_id).map(|s| s.as_str())
    }

    pub fn advance_arc(&mut self, arc_id: impl Into<String>, state: impl Into<String>) {
        self.arc_states.insert(arc_id.into(), state.into());
    }
}
```

**Step 2: Write tests**

```rust
#[test]
fn red_check_absent_initially() {
    let gd = GameData::default();
    assert!(!gd.has_failed_red_check("base::some_scene", "CHARM"));
}

#[test]
fn red_check_present_after_fail() {
    let mut gd = GameData::default();
    gd.fail_red_check("base::some_scene", "CHARM");
    assert!(gd.has_failed_red_check("base::some_scene", "CHARM"));
}

#[test]
fn arc_state_absent_initially() {
    let gd = GameData::default();
    assert_eq!(gd.arc_state("base::jake"), None);
}

#[test]
fn arc_advance_and_query() {
    let mut gd = GameData::default();
    gd.advance_arc("base::jake", "acquaintance");
    assert_eq!(gd.arc_state("base::jake"), Some("acquaintance"));
}
```

**Step 3:** `cargo test -p undone-world` — all pass

**Step 4:** `cargo fmt -p undone-world`

**Commit:** `feat(world): add arc_states and red_check_failures to GameData`

---

### Task 4: checkSkillRed() and arcState() evaluator methods

**Files:**
- Modify: `crates/undone-expr/src/eval.rs`

**Step 1: Add checkSkillRed to Player arm**

```rust
// In eval_call_bool, Receiver::Player:
"checkSkillRed" => {
    // Red check: fails permanently if already failed in this scene+skill combo.
    // Requires scene context to have a scene_id. We use "scene" as a sentinel.
    // The scene_id must be passed by the engine before evaluating red checks.
    let skill_id_str = str_arg(0)?;
    let dc = match call.args.get(1) {
        Some(Value::Int(n)) => *n as i32,
        _ => return Err(EvalError::BadArg("checkSkillRed".into())),
    };
    // If already permanently failed, return false immediately
    let scene_id = ctx.scene_id.as_deref().unwrap_or("unknown");
    if world.game_data.has_failed_red_check(scene_id, skill_id_str) {
        return Ok(false);
    }
    // Otherwise evaluate like a normal check
    let skill_id = registry
        .resolve_skill(skill_id_str)
        .map_err(|_| EvalError::UnknownSkill(skill_id_str.to_string()))?;
    let skill_value = world.player.skill(skill_id) as i32;
    let roll = ctx.get_or_roll_skill(skill_id_str);
    let target = (skill_value + (50 - dc)).clamp(5, 95);
    Ok(roll <= target)
    // NOTE: Actually marking the failure happens as an Effect (FailRedCheck),
    // not in the evaluator. The evaluator only reads state.
}
```

Also add `pub scene_id: Option<String>` to SceneCtx, set by the engine when starting a scene.

**Add arcState(), arcStarted() to GameData arm in eval_call_string / eval_call_bool:**

```rust
// In eval_call_string, Receiver::GameData:
"arcState" => {
    let arc_id = str_arg(0)?;
    Ok(world.game_data.arc_state(arc_id).unwrap_or("").to_string())
}

// In eval_call_bool, Receiver::GameData:
"arcStarted" => {
    let arc_id = str_arg(0)?;
    Ok(world.game_data.arc_state(arc_id).is_some())
}
```

**Step 2: Tests**

```rust
#[test]
fn arcState_returns_empty_when_not_started() {
    let world = make_world();
    let ctx = SceneCtx::new();
    let reg = undone_packs::PackRegistry::new();
    let expr = parse("gd.arcState('base::jake') == ''").unwrap();
    assert!(eval(&expr, &world, &ctx, &reg).unwrap());
}

#[test]
fn arcState_returns_current_state() {
    let mut world = make_world();
    world.game_data.advance_arc("base::jake", "acquaintance");
    let ctx = SceneCtx::new();
    let reg = undone_packs::PackRegistry::new();
    let expr = parse("gd.arcState('base::jake') == 'acquaintance'").unwrap();
    assert!(eval(&expr, &world, &ctx, &reg).unwrap());
}

#[test]
fn arcStarted_false_initially() {
    let world = make_world();
    let ctx = SceneCtx::new();
    let reg = undone_packs::PackRegistry::new();
    let expr = parse("gd.arcStarted('base::jake')").unwrap();
    assert!(!eval(&expr, &world, &ctx, &reg).unwrap());
}
```

**Step 3:** `cargo test -p undone-expr` — all pass

**Commit:** `feat(expr): add checkSkillRed, arcState, arcStarted evaluator methods`

---

### Task 5: Prose template context — full skill/stat access

Currently `w.getSkill()` is unavailable in Jinja2 prose templates. This task exposes the full state.

**Files:**
- Modify: `crates/undone-scene/src/template_ctx.rs`

**Step 1: Extend PlayerCtx struct**

```rust
#[derive(Debug)]
pub struct PlayerCtx {
    pub trait_strings: HashSet<String>,
    pub virgin: bool,
    pub origin: PcOrigin,
    pub partner: bool,
    pub on_pill: bool,
    pub pregnant: bool,
    // New:
    pub skills: HashMap<String, i32>,   // skill_id_string → effective value
    pub money: i32,
    pub stress: i32,
    pub anxiety: i32,
    pub arousal: String,
    pub alcohol: String,
}
```

**Step 2: Extend PlayerCtx::call_method**

```rust
"getSkill" => {
    let id = string_arg(method, args, 0)?;
    Ok(Value::from(*self.skills.get(id.as_str()).unwrap_or(&0)))
}
"getMoney"   => Ok(Value::from(self.money)),
"getStress"  => Ok(Value::from(self.stress)),
"getAnxiety" => Ok(Value::from(self.anxiety)),
"getArousal" => Ok(Value::from(self.arousal.as_str())),
"getAlcohol" => Ok(Value::from(self.alcohol.as_str())),
"wasMale"    => Ok(Value::from(!self.origin.is_always_female() &&
                                self.origin != PcOrigin::TransWomanTransformed)),
"wasTransformed" => Ok(Value::from(self.origin != PcOrigin::AlwaysFemale)),
```

**Step 3: Extend GameDataCtx**

```rust
pub struct GameDataCtx {
    pub week: u32,
    pub day: u8,
    pub time_slot: String,
    pub flags: HashSet<String>,
}

// In call_method:
"timeSlot"  => Ok(Value::from(self.time_slot.as_str())),
"day"       => Ok(Value::from(self.day as i32)),
"isWeekday" => Ok(Value::from(self.day <= 4)),
"isWeekend" => Ok(Value::from(self.day >= 5)),
```

**Step 4: Populate new fields in render_prose()**

```rust
// Resolve skill IDs to (string → value) map
let skills: HashMap<String, i32> = world.player.skills.iter()
    .map(|(&sid, sv)| {
        let name = registry.skill_id_to_str(sid).to_string();
        (name, sv.effective())
    })
    .collect();

let player_ctx = PlayerCtx {
    trait_strings,
    virgin: world.player.virgin,
    origin: world.player.origin,
    partner: world.player.partner.is_some(),
    on_pill: world.player.on_pill,
    pregnant: world.player.pregnancy.is_some(),
    skills,
    money: world.player.money,
    stress: world.player.stress,
    anxiety: world.player.anxiety,
    arousal: format!("{:?}", world.player.arousal),
    alcohol: format!("{:?}", world.player.alcohol),
};

let game_data_ctx = GameDataCtx {
    week: world.game_data.week,
    day: world.game_data.day,
    time_slot: format!("{:?}", world.game_data.time_slot),
    flags: world.game_data.flags.clone(),
};
```

Note: `skill_id_to_str` must exist on PackRegistry. Add it if missing (mirrors `trait_id_to_str`).

**Step 5: Tests**

```rust
#[test]
fn getSkill_in_template_returns_value() {
    let mut registry = undone_packs::PackRegistry::new();
    registry.register_skills(vec![undone_packs::SkillDef {
        id: "CHARM".into(), name: "Charm".into(), description: "".into(), min: 0, max: 100,
    }]);
    let skill_id = registry.resolve_skill("CHARM").unwrap();
    let mut world = make_world();
    world.player.skills.insert(skill_id, SkillValue { value: 65, modifier: 0 });
    let ctx = SceneCtx::new();
    let template = r#"{% if w.getSkill("CHARM") > 50 %}skilled{% else %}unskilled{% endif %}"#;
    let result = render_prose(template, &world, &ctx, &registry).unwrap();
    assert!(result.contains("skilled"));
}

#[test]
fn timeSlot_in_template() {
    let registry = undone_packs::PackRegistry::new();
    let world = make_world(); // time_slot = Morning
    let ctx = SceneCtx::new();
    let template = r#"{% if gd.timeSlot() == "Morning" %}morning{% else %}other{% endif %}"#;
    let result = render_prose(template, &world, &ctx, &registry).unwrap();
    assert!(result.contains("morning"));
}
```

**Step 6:** `cargo test -p undone-scene` — all pass

**Commit:** `feat(scene): expose getSkill, getMoney, getStress, timeSlot etc in prose templates`

---

### Task 6: Thought system — [[thoughts]] in scene TOML

Thoughts are prose blocks that fire automatically based on conditions, displayed in a distinct visual style. They don't replace the intro/action prose; they layer on top.

**Files:**
- Modify: `crates/undone-scene/src/types.rs` — add ThoughtDef, update SceneToml and ActionDef
- Modify: `crates/undone-scene/src/engine.rs` — render thoughts after prose

**Step 1: Add ThoughtDef to types.rs**

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct ThoughtDef {
    pub condition: Option<String>,
    pub prose: String,
    /// Visual style tag. Engine emits this with the prose.
    /// "inner_voice" = italicised inner monologue
    /// "anxiety"     = anxious intrusion
    /// Can be any string — UI renders by style name.
    #[serde(default = "default_thought_style")]
    pub style: String,
}

fn default_thought_style() -> String { "inner_voice".to_string() }
```

**Step 2: Add to SceneToml and ActionDef**

```rust
pub struct SceneToml {
    pub scene: SceneMeta,
    pub intro: IntroDef,
    #[serde(default)]
    pub thoughts: Vec<ThoughtDef>,           // fire after intro
    #[serde(default)]
    pub actions: Vec<ActionDef>,
    #[serde(default)]
    pub npc_actions: Vec<NpcActionDef>,
}

pub struct ActionDef {
    // ... existing fields ...
    #[serde(default)]
    pub thoughts: Vec<ThoughtDef>,           // fire after action prose
}
```

**Step 3: Add ThoughtAdded to EngineEvent in engine.rs**

```rust
pub enum EngineEvent {
    ProseAdded(String),
    ThoughtAdded { text: String, style: String },  // new
    ActionsAvailable(Vec<ActionView>),
    NpcActivated(Option<NpcActivatedData>),
    SceneFinished,
    SlotRequested(String),
    ErrorOccurred(String),
}
```

**Step 4: Render thoughts in engine.rs**

Find where `ProseAdded` events are emitted for intro and action prose. After each, render thoughts:

```rust
fn render_thoughts(
    thoughts: &[ThoughtDef],
    world: &World,
    ctx: &SceneCtx,
    registry: &PackRegistry,
    events: &mut VecDeque<EngineEvent>,
) {
    for thought in thoughts {
        let passes = match &thought.condition {
            None => true,
            Some(cond) => {
                match parse(cond).and_then(|e| eval(&e, world, ctx, registry).map_err(|_|
                    undone_expr::ParseError::UnexpectedEof)) // adapt error type
                {
                    Ok(b) => b,
                    Err(e) => {
                        events.push_back(EngineEvent::ErrorOccurred(
                            format!("thought condition error: {e}")
                        ));
                        false
                    }
                }
            }
        };
        if passes {
            match render_prose(&thought.prose, world, ctx, registry) {
                Ok(text) if !text.trim().is_empty() => {
                    events.push_back(EngineEvent::ThoughtAdded {
                        text,
                        style: thought.style.clone(),
                    });
                }
                Ok(_) => {}
                Err(e) => events.push_back(EngineEvent::ErrorOccurred(
                    format!("thought prose render error: {e}")
                )),
            }
        }
    }
}
```

Call this after emitting the intro `ProseAdded` and after each action `ProseAdded`.

**Step 5: Handle `ThoughtAdded` in UI** (`crates/undone-ui/src/story_panel.rs` or wherever `EngineEvent` is processed)

Find the match on `EngineEvent`. Add:
```rust
EngineEvent::ThoughtAdded { text, style: _ } => {
    // For now, append with italic style marker.
    // UI can differentiate styles later.
    prose_signal.update(|v| v.push(ProseBlock { text, italic: true }));
}
```

(Adapt to the actual prose rendering mechanism in the UI.)

**Step 6:** `cargo check` — no errors. `cargo test` — all pass.

**Commit:** `feat(scene): add thought system — [[thoughts]] blocks with style and condition`

---

### Task 7: Narrator variant blocks — [[intro_variants]]

The first variant whose condition passes replaces the base intro. Enables BG3-style 17 narrator modes.

**Files:**
- Modify: `crates/undone-scene/src/types.rs` — add NarratorVariantDef
- Modify: `crates/undone-scene/src/engine.rs` — variant selection at scene start

**Step 1: Add NarratorVariantDef**

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct NarratorVariantDef {
    pub condition: String,
    pub prose: String,
}
```

**Step 2: Add to SceneToml**

```rust
pub struct SceneToml {
    pub scene: SceneMeta,
    pub intro: IntroDef,
    #[serde(default)]
    pub intro_variants: Vec<NarratorVariantDef>,  // new
    #[serde(default)]
    pub thoughts: Vec<ThoughtDef>,
    #[serde(default)]
    pub actions: Vec<ActionDef>,
    #[serde(default)]
    pub npc_actions: Vec<NpcActionDef>,
}
```

**Step 3: Variant selection in engine**

When rendering intro prose, select the variant instead of base if condition passes:

```rust
fn select_intro_prose<'a>(
    scene: &'a SceneDefinition,
    world: &World,
    ctx: &SceneCtx,
    registry: &PackRegistry,
) -> &'a str {
    for variant in &scene.intro_variants {
        if let Ok(expr) = parse(&variant.condition) {
            if eval(&expr, world, ctx, registry).unwrap_or(false) {
                return &variant.prose;
            }
        }
    }
    &scene.intro.prose
}
```

(Add `intro_variants: Vec<NarratorVariantDef>` to `SceneDefinition` too, populated from TOML.)

**Step 4:** `cargo check` — no errors. `cargo test` — all pass.

**Commit:** `feat(scene): add [[intro_variants]] narrator variant blocks`

---

### Task 8: Arc system effects and NPC roles

**Files:**
- Modify: `crates/undone-scene/src/types.rs` — add AdvanceArc, SetNpcRole, FailRedCheck effects
- Modify: `crates/undone-scene/src/effects.rs` — implement them
- Modify: `crates/undone-domain/src/npc.rs` — add `roles: HashSet<String>` to NpcCore
- Modify: `crates/undone-expr/src/eval.rs` — add `m.hasRole()` evaluator method

**Step 1: Add to EffectDef enum in types.rs**

```rust
// In EffectDef enum (add these variants):
AdvanceArc {
    arc: String,
    to_state: String,
},
SetNpcRole {
    /// "m" or "f"
    npc: String,
    role: String,
},
FailRedCheck {
    skill: String,
},
```

**Step 2: Implement in effects.rs**

```rust
EffectDef::AdvanceArc { arc, to_state } => {
    world.game_data.advance_arc(arc, to_state);
}
EffectDef::SetNpcRole { npc, role } => {
    match npc.as_str() {
        "m" => {
            let key = ctx.active_male.ok_or(EffectError::NoActiveMale)?;
            let npc = world.male_npcs.get_mut(key).ok_or(EffectError::NpcNotFound)?;
            npc.core.roles.insert(role.clone());
        }
        "f" => {
            let key = ctx.active_female.ok_or(EffectError::NoActiveFemale)?;
            let npc = world.female_npcs.get_mut(key).ok_or(EffectError::NpcNotFound)?;
            npc.core.roles.insert(role.clone());
        }
        _ => return Err(EffectError::BadNpcRef(npc.clone())),
    }
}
EffectDef::FailRedCheck { skill } => {
    let scene_id = ctx.scene_id.as_deref().unwrap_or("unknown");
    world.game_data.fail_red_check(scene_id, skill);
}
```

**Step 3: Add roles to NpcCore in npc.rs**

```rust
pub struct NpcCore {
    // ... existing fields ...
    #[serde(default)]
    pub roles: HashSet<String>,    // route role assignments e.g. "ROLE_DAVID"
}
```

**Step 4: Add m.hasRole() to evaluator**

```rust
// In eval_call_bool, Receiver::MaleNpc arm:
"hasRole" => {
    let role = str_arg(0)?;
    Ok(npc.core.roles.contains(role))
}
// Same for Receiver::FemaleNpc.
```

**Step 5: Tests**

```rust
#[test]
fn advance_arc_effect_changes_state() { /* ... */ }

#[test]
fn set_npc_role_adds_role_to_active_male() { /* ... */ }

#[test]
fn m_hasRole_true_when_role_set() { /* ... */ }
```

**Step 6:** `cargo test` — all pass. `cargo fmt`

**Commit:** `feat(scene): arc effects (advance_arc, set_npc_role, fail_red_check) and m.hasRole()`

---

### Task 9: Arc data files and pack loading

**Files:**
- Create: `packs/base/data/arcs.toml`
- Modify: `crates/undone-packs/src/data.rs` — ArcDef struct
- Modify: `crates/undone-packs/src/registry.rs` — arc registry
- Modify: `crates/undone-packs/src/manifest.rs` — arcs field in PackContent
- Modify: `crates/undone-packs/src/loader.rs` — load arcs.toml
- Modify: `packs/base/pack.toml` — point to arcs

**Step 1: ArcDef in data.rs**

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct ArcDef {
    pub id: String,
    /// Ordered list of valid state names.
    pub states: Vec<String>,
    /// Optional NPC role tag that this arc's NPC will receive.
    pub npc_role: Option<String>,
    /// Optional default starting state (if arc auto-starts).
    pub initial_state: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ArcsFile {
    #[serde(default)]
    pub arc: Vec<ArcDef>,
}
```

**Step 2: Registry methods**

```rust
// In PackRegistry:
pub fn register_arcs(&mut self, arcs: Vec<ArcDef>) {
    for arc in arcs {
        self.arcs.insert(arc.id.clone(), arc);
    }
}

pub fn get_arc(&self, id: &str) -> Option<&ArcDef> {
    self.arcs.get(id)
}
```

**Step 3: packs/base/data/arcs.toml** (initial entries — more added when routes are written)

```toml
[[arc]]
id           = "base::robin_opening"
states       = ["arrived", "week_one", "working", "settled"]
initial_state = "arrived"

[[arc]]
id           = "base::camila_opening"
states       = ["arrived", "orientation", "dorm_life", "first_week"]
initial_state = "arrived"
```

**Step 4: pack.toml — add arcs pointer**

```toml
[content]
traits     = "data/traits.toml"
npc_traits = "data/npc_traits.toml"
skills     = "data/skills.toml"
stats      = "data/stats.toml"
races      = "data/races.toml"
scenes_dir = "scenes/"
arcs       = "data/arcs.toml"      # new
```

**Step 5:** `cargo test` — all pass.

**Commit:** `feat(packs): arc data format, registry, and loader`

---

### Task 10: Route flags in new_game()

Route characters (Robin, Camila) are distinguished by game flags set at game creation. Character selection UI is deferred — for now, `CharCreationConfig` accepts `starting_flags`.

**Files:**
- Modify: `crates/undone-packs/src/char_creation.rs`

**Step 1:**

```rust
pub struct CharCreationConfig {
    // ... existing fields ...
    #[serde(default)]
    pub starting_flags: HashSet<String>,   // e.g. {"ROUTE_ROBIN"}
    pub starting_arc_states: HashMap<String, String>, // arc_id → initial state
}

// In new_game(), after building GameData::default():
let mut game_data = GameData::default();
for flag in config.starting_flags {
    game_data.set_flag(flag);
}
for (arc_id, state) in config.starting_arc_states {
    game_data.advance_arc(arc_id, state);
}
```

**Step 2: Tests**

```rust
#[test]
fn new_game_sets_starting_flags() {
    let (mut registry, _) = load_packs(&packs_dir()).unwrap();
    let mut config = base_config();
    config.starting_flags = ["ROUTE_ROBIN".to_string()].into();
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    let world = new_game(config, &mut registry, &mut rng);
    assert!(world.game_data.has_flag("ROUTE_ROBIN"));
}
```

**Step 3:** `cargo test -p undone-packs` — all pass.

**Commit:** `feat(packs): starting_flags and starting_arc_states in CharCreationConfig`

---

### Task 11: validate-pack binary

A standalone tool that loads packs and reports ALL content errors — not just first-fail.

**Files:**
- Modify: `Cargo.toml` (root workspace) — add binary target
- Create: `src/bin/validate_pack.rs`

**Step 1: Add to workspace Cargo.toml**

```toml
[[bin]]
name = "validate-pack"
path = "src/bin/validate_pack.rs"
```

**Step 2: src/bin/validate_pack.rs**

```rust
use std::path::PathBuf;
use undone_packs::load_packs;
use undone_scene::loader::load_scenes;

fn main() {
    let packs_dir = PathBuf::from("packs");
    println!("Loading packs from {:?}", packs_dir);

    let (registry, pack_metas) = match load_packs(&packs_dir) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("FATAL: pack load failed: {e}");
            std::process::exit(1);
        }
    };

    println!("Packs loaded. Loading scenes...");
    let mut error_count = 0;

    for meta in &pack_metas {
        match load_scenes(&meta.scenes_dir, &registry) {
            Ok(scenes) => {
                println!("  {} scenes loaded from {}", scenes.len(), meta.id);
                for scene in &scenes {
                    // Check 1: all goto targets exist within scene
                    for action in &scene.actions {
                        for next in &action.next {
                            if let Some(target) = &next.goto {
                                if !scene.actions.iter().any(|a| &a.id == target) {
                                    eprintln!("ERROR [{}] action '{}': goto '{}' not found",
                                        scene.id, action.id, target);
                                    error_count += 1;
                                }
                            }
                        }
                    }
                    // Check 2: scene sets at least one lasting flag or NPC effect
                    // (warn, not error — some scenes are intentionally stateless)
                    let has_lasting = scene.actions.iter().any(|a| {
                        a.effects.iter().any(|e| matches!(e,
                            undone_scene::types::EffectDef::SetGameFlag { .. } |
                            undone_scene::types::EffectDef::AddNpcLiking { .. } |
                            undone_scene::types::EffectDef::AdvanceArc { .. }
                        ))
                    });
                    if !has_lasting {
                        eprintln!("WARN  [{}] no lasting effects (game flag, NPC liking, or arc advance)", scene.id);
                    }
                }
            }
            Err(e) => {
                eprintln!("ERROR loading scenes for {}: {e}", meta.id);
                error_count += 1;
            }
        }
    }

    if error_count > 0 {
        eprintln!("\n{error_count} error(s) found.");
        std::process::exit(1);
    } else {
        println!("\nAll checks passed.");
    }
}
```

**Step 3:** `cargo build --bin validate-pack` — builds. `cargo run --bin validate-pack` — runs against current packs.

**Commit:** `feat: validate-pack binary — content error checking tool`

---

## PART 2: AGENTIC INFRASTRUCTURE
### Session 2 · Documentation only · No Rust changes

---

### Task 12: World document

**File:** Create `docs/world.md`

This is the canonical reference for all world-building decisions. Every session that writes content must read this first.

```markdown
# Undone — World Reference

> Canonical facts about the game world. All content must be consistent with this document.
> Update this document whenever a new world fact is established in a scene.

## Setting

**City:** Fictional Northeast US city. Dense, transit-rich, economically stratified.
Resembles Boston/Providence/Hartford in scale and character. Two universities:
one large research university, one Ivy League (the city's Ivy).
Culturally: old money neighborhoods, working-class immigrant neighborhoods,
gentrifying artist districts, a financial district, extensive public transit.

**Time period:** Near-future, approximately 2028–2030. Recognizably our world with
small-scale technological and legislative differences. Never specify exact year.

## Technology and Infrastructure

**Airports:** The Northeast Privacy Compact of 2027 prohibited automated biometric
scanning at transit exits in member states. Airport exit screening is verbal/card only.
A customs or gate agent checks ID documents manually. This is how Robin and others
get through with mismatched documents in the first week.

**Phones/devices:** Identical to 2025. No new consumer technology to invent.

**Facial recognition:** Limited in public spaces by the Northeast Privacy Compact.
Still used in private contexts (workplaces, financial institutions) where opted in.

## The Transformation

**What it is:** Unknown. The game never explains it. Do not speculate in-world.
Others may or may not have experienced it — the game never confirms or denies this.
Characters who were transformed do not know of each other unless they tell each other.

**When it happens:** During sleep. Subjects wake transformed. No warning, no process.

**What changes:** Body completely. Skills, knowledge, memories: unchanged.
Personality: unchanged. Preferences, sexuality: may shift (character-specific).

**What doesn't change:** Name (unless player changes it). Job skills. Memories.
Professional and educational credentials (not yet biometrically enforced).

## Common Cultural Details

- **Transit:** Subway + bus network. Most characters use it. Uber/Lyft exist.
- **Coffee:** Dunkin' and Starbucks chains, plus independent cafés.
- **Grocery:** Stop & Shop, ShopRite, Trader Joe's, local bodegas.
- **Bars:** Dive bars, sports bars, cocktail bars. No Wetherspoons equivalents.
- **Food delivery:** Ubiquitous. Characters order in regularly.
- **Currency:** Dollars. "$5", "twenty bucks", "a hundred" — never British.

## Universities

- **The University:** Large public research university. 25,000+ students.
  Main campus in the university district. Liberal arts + STEM + professional schools.
- **The Ivy:** Elite, private, 6,000 students. Smaller campus, older buildings.
  Camila attends the Ivy. Socioeconomic mix: scholarship students and legacy students.

## Neighborhoods (establish as needed, document here)

- **Clement Ave area:** Walkable, mixed-use. Robin's apartment is here. Coffee shop on Union and Third.
- **University district:** Student-heavy. Cheap food, bars, bookstores.
- **Financial district:** Where Robin's tech company office is. 20-minute subway from Clement.
```

**Commit:** `docs: world reference document`

---

### Task 13: Character sheets

**Files:**
- Create: `docs/characters/robin.md`
- Create: `docs/characters/camila.md`

**robin.md** — must be detailed enough for any session to write Robin scenes without drift:

```markdown
# Robin — Character Sheet

## Quick Reference

| Field | Value |
|---|---|
| Route flag | `ROUTE_ROBIN` |
| Arc | `base::robin_opening` |
| Before name | Robin (same — gender-neutral, kept) |
| Current name | Robin |
| Age (before) | Early 30s |
| Age (now) | Late teens (appears ~18-19) |
| Race (before) | White |
| Race (now) | East Asian |
| Figure | Petite, short |
| Breasts | H-cup |
| Face | Heartachingly beautiful — draws immediate attention |
| Job | Software engineer (hired pre-transformation, starts Monday) |
| Arrived | Saturday before first Monday |
| Season | Early spring |
| FEMININITY start | 10 (CisMaleTransformed) |
| Sexuality (before) | Straight (attracted to women) |
| Sexuality (now) | Bi — attraction to men is new and unsettling |

## Character Voice

Robin's interiority is 32 years old. She processes the world like a senior engineer:
systematic, calm under pressure, methodical. She does not panic — she inventories.
When something goes wrong she says *okay* the same way she says it on production incidents.

She is not performing competence. She is competent. The gap is between her internal state
and how every stranger reads her: they see a teenager. She is aware of this constantly.

She is not in denial about the transformation. She has accepted it the way she accepts
any unexpected constraint: pragmatically, with a certain resignation. She is working
the problem.

## The Fetishization Thread

Robin was a white man who had the casual gaze that fetishizes Asian women. He didn't
think much about it. He was not malicious — he was unexamined. Now he receives exactly
that gaze daily, from men who might have been his friends, who might have been him.

He can read it perfectly. He knows the internal monologue of the man looking at him
right now because it was his internal monologue. This creates a specific kind of
cognitive dissonance: understanding something completely and still feeling it.

**Player choice:** Lean into the fetishization (accept it, perhaps use it, find something
in it), or cut it off (deflect, correct, refuse). Both are valid and fully written.

## Adaptation Arc

Week 1: Purely practical. Bra shopping. Period supplies (or will need them).
Not knowing how to move through the world in this body. Learning by doing.
Internally still male — uses male pronouns internally, thinks of himself as he/him
in first person. ("I" = he, but the world is calling her she.)

Over time: gradual adaptation. Small things that become familiar. Not a dramatic arc —
she doesn't "discover herself" as a woman. She just adjusts, like a codebase migration.

## What Robin Doesn't Know Yet

- How to shop for clothes in her size
- How periods work in practice
- How to style her hair
- Why men hold doors now and what to do with that
- What she actually feels about men (she's curious; she's not ready to admit it)
- That her face is going to be a problem in ways she doesn't anticipate

## Scene Conditions

Robin-specific scenes require: `gd.hasGameFlag('ROUTE_ROBIN')`
Robin's arc scenes also require: `gd.arcState('base::robin_opening') == 'state_name'`

## Misrecognition Thread

Robin looks ~18-19. She has a software engineering job, 10+ years of professional
experience, and speaks like someone who's been in boardrooms. She gets carded.
She gets asked if she's a student. She gets explained things she invented.

This is not primarily a source of comedy — it's a constant low-grade friction that
occasionally becomes acute. Write it straight.
```

**camila.md** — similar depth for Camila. See arc design in Task 14.

**Commit:** `docs: character sheets for Robin and Camila`

---

### Task 14: Arc design documents

**Files:**
- Create: `docs/arcs/robin-opening.md`
- Create: `docs/arcs/camila-opening.md`

**robin-opening.md:**

```markdown
# Arc: base::robin_opening

## Narrative Purpose

Establish Robin in the city. Ground the transformation in specific physical and
social experiences. Set the tone for her route: pragmatic, wry, quietly overwhelmed.
Introduce the city as a place with its own life.

## State Machine

arrived → week_one → working → settled

**arrived** (Saturday, day 0–1)
Scenes: robin_arrival, robin_landlord, robin_first_night

**week_one** (Sunday–Sunday, before first workday)
Scenes: robin_first_clothes, robin_city_walk (universal slot)

**working** (Monday onward, has started job)
Scenes: robin_first_day, universal work-adjacent scenes

**settled** (after ~week 2, has basic routines)
Universal scenes fire normally. Robin-specific flavor via intro_variants.

## Scene List

| Scene ID | Arc state | Location | Content level | Sets |
|---|---|---|---|---|
| robin_arrival | arrived | Airport → subway | VANILLA | ROUTE_ROBIN |
| robin_landlord | arrived | Her apartment building | VANILLA | game flag MET_LANDLORD |
| robin_first_night | arrived→week_one | Her apartment | VANILLA | arc→week_one |
| robin_first_clothes | week_one | Clothing store | VANILLA | — |
| robin_first_day | week_one→working | Tech office | VANILLA | arc→working |

## Tone Notes

- Narrator: companion on Robin's shoulder, watching with wry attention
- Inner voice: male pronouns internally ("*you*, he thinks"), then catches himself
- Fetishization: present from arrival. Not melodramatic. Just... there.
- The city: has its own life independent of Robin's distress
```

**Commit:** `docs: arc design documents for Robin and Camila opening arcs`

---

### Task 15: Writing samples document

**File:** Create `docs/writing-samples.md`

This is the quality calibration document. Every writing session reads this before touching a scene. It contains approved prose excerpts that define the target register.

The document should contain:
1. The airport scene opening from the brainstorm (first 250 words — approved prose)
2. The rain_shelter intro paragraph (already exists, approved)
3. The rain_shelter umbrella-offer NPC action (already exists, approved)
4. An example of transformation inner voice at FEMININITY < 25
5. An example of a well-executed trait branch (structural difference, not adjective swap)
6. An example of bad prose with correction (anti-pattern demonstration)

See the prose drafted in the brainstorming session for the airport opening. Include it verbatim as Sample 1. Annotate each sample with what it demonstrates.

**Commit:** `docs: writing samples calibration document`

---

## PART 3: ROBIN'S OPENING ARC
### Session 3 · Content only · Read world.md + robin.md + writing-guide.md first

**Before writing any scene:** Read `docs/world.md`, `docs/characters/robin.md`, `docs/writing-guide.md`, `docs/writing-samples.md`. The quality bar is the airport opening prose from the brainstorm.

**After writing each scene:** Run `mcp__minijinja__jinja_validate_template` on every prose template. Run `cargo run --bin validate-pack`. Run `cargo test`.

---

### Task 16: Robin arrival scene

**File:** Create `packs/base/scenes/robin_arrival.toml`

**Scene specs:**
- ID: `base::robin_arrival`
- Arc condition: fires only if `gd.hasGameFlag('ROUTE_ROBIN')` and `gd.arcState('base::robin_opening') == 'arrived'` (or no arc state set yet)
- Trigger: once_only (this is the opening scene)
- Sets: `ROUTE_ROBIN` flag (if not already set), starts arc

**Scene structure:**

*Intro:* On the plane, waking up wrong. The flight attendant. The jetbridge. First exposure to being looked at (quick male gaze in the terminal). Getting through airport exit on verbal ID check (Northeast Privacy Compact — agent checks ID, sees white guy in his 30s, looks at this tiny Asian girl, she says "I know, I just look young" and he waves her through because the line is long). Finding the right subway platform alone.

*Inner voice thought:* `w.getSkill("FEMININITY") < 20` — she catches herself using male-internal-monologue and it doesn't quite fit anymore. Style: `inner_voice`.

*Thoughts (transformation, earned here):* `!w.alwaysFemale()` — specific body inventory: the weight on her chest she keeps forgetting and then remembering. Not dramatized. Just a recurring fact.

*Player actions:*
1. **Get your bearings** (allow_npc_actions: false) — solo navigation, first street scene
2. **Call a cab** — costs money, avoids transit, ends scene faster

*NPC actions:* Man on subway (optional, low weight) — standard interaction. Vary by PC traits (SHY, BEAUTIFUL).

*Effects:*
- `set_game_flag: ROUTE_ROBIN` (ensures set)
- `advance_arc: base::robin_opening → arrived`

*Prose quality bar:* Match or exceed the airport draft from the brainstorming session. The world has its own life. The narrator notices things Robin notices. The transformation content is earned, not announced.

**Validate:** `mcp__minijinja__jinja_validate_template` on each prose block. `cargo run --bin validate-pack`.

**Commit:** `content: robin_arrival — opening scene, airport and first subway`

---

### Task 17: Robin landlord scene

**File:** Create `packs/base/scenes/robin_landlord.toml`

**Scene specs:**
- ID: `base::robin_landlord`
- Arc condition: `gd.arcState('base::robin_opening') == 'arrived'`
- Schedule: once_only, weight 0, trigger `gd.hasGameFlag('ROUTE_ROBIN') && !gd.hasGameFlag('MET_LANDLORD')`

**Scene structure:**

*Setup:* Robin arrives at her building. She called ahead as Robin — voice matched the name. Now this small Asian girl shows up at the door with a driver's license (white guy photo) and a signed lease.

*The landlord:* Middle-aged. Not unkind. Has a specific kind of confusion: the voice on the phone sounded one way, the person at the door is another, and the ID photo is a third way. He cannot reconcile these three things. He does not ask the question he's thinking.

*Robin doesn't explain.* She waits him out. This is what she does in meetings when someone says something wrong and she's decided not to correct them. The silence does the work.

*Trait branches:*
- `SHY`: She can't maintain the silence. She offers too much. He's more confused.
- `AMBITIOUS`: She's already thinking about Monday. The landlord is an obstacle. Efficient.
- `CUTE`: She smiles and he stops asking questions. She doesn't entirely like how well that worked.

*NPC actions:* The landlord tries to reconcile the documentation. His wife calls from upstairs (world texture — he has a life).

*Effects:*
- `set_game_flag: MET_LANDLORD`
- NPC liking (whatever NPC is assigned): neutral or slight positive

**Commit:** `content: robin_landlord — first social test with mismatched ID`

---

### Task 18: Robin first night scene

**File:** Create `packs/base/scenes/robin_first_night.toml`

**Scene specs:**
- ID: `base::robin_first_night`
- Arc condition: `arrived`, advances to `week_one`
- Once only

**Scene structure:**

*Setup:* Empty apartment. Her boxes aren't here yet (shipping company said Monday). She has her carry-on. She's been awake for 22 hours.

*The inventory:* What she has. What she doesn't have. The body she's in, the quiet of the apartment, the city noise through the window.

*The problem she's going to have tomorrow:* She doesn't have a bra. She doesn't know her size. She didn't think about this on the plane because she was trying to get through the airport. She's thinking about it now.

*Transformation content (earned):* `!w.alwaysFemale() && w.getSkill('FEMININITY') < 20` — inner voice: she keeps reaching for habits that don't fit anymore. Not dramatic. Just the specific exhaustion of trying to figure out what the new defaults are.

*Player actions:*
1. **Order food and sleep** — pragmatic, ends scene, slight stress reduction
2. **Try to figure out the bra situation** — leads to phone research, comic-but-not-comedy realization of how much she doesn't know
3. **Call someone back home** — the call branch: she hears a friend's voice, cannot explain anything, has to pretend everything is normal

*Effects:*
- `advance_arc: base::robin_opening → week_one`
- Relevant stress/anxiety changes based on action

**Commit:** `content: robin_first_night — first evening, inventory, the bra problem`

---

### Task 19: Robin first clothes scene

**File:** Create `packs/base/scenes/robin_first_clothes.toml`

**Scene specs:**
- ID: `base::robin_first_clothes`
- Arc condition: `week_one`
- Once only (or repeatable with variation on return)

**Scene structure:**

*Setup:* Sunday. She needs clothes that fit. She goes to a department store or TJ Maxx.

*The mechanics:* She doesn't know her size. She has to ask. She ends up in a dressing room with clothes that are either practical (bras, underwear, basics) or an attempt at professional (she starts Monday). The dressing room mirror is unavoidable.

*The mirror moment:* Different from the airport mirror. She's been in this body for a day. The shock is different — less acute, more specific. She notices things about herself she didn't have language for yesterday.

*Trait branches:*
- `POSH`: Wrong store. She's in TJ Maxx and she can tell and she buys things anyway because pragmatic.
- `DOWN_TO_EARTH`: Gets functional stuff efficiently. Doesn't dwell on the mirror.
- `OVERACTIVE_IMAGINATION`: Sees her reflection and gets ahead of herself about what Monday is going to be like.

*Fetishization thread:* A male shopper notices her. She clocks it before she means to. Knows exactly what it is. Does not know what to do with the fact that something in her body registered it before her brain did.

*Inner voice thought:* If FEMININITY < 20, the body's responses are ahead of her interpretation. The thought style is `inner_voice`.

**Commit:** `content: robin_first_clothes — Sunday shopping, mirror, the gaze`

---

### Task 20: Robin first day scene

**File:** Create `packs/base/scenes/robin_first_day.toml`

**Scene specs:**
- ID: `base::robin_first_day`
- Arc condition: `week_one`, advances to `working`
- Once only

**Scene structure:**

*Setup:* Monday. She took the subway to the financial district. She's in business casual that doesn't entirely fit her new frame. She arrives at the tech company. The lobby security guard checks her ID (Northeast Privacy Compact means no facial recognition — just the card). The ID says Robin. The face doesn't match the photo. Same beat as the airport, faster this time — she's practiced the move.

*The office:* Open plan. Tech company, software engineering. Her manager meets her. There's a moment where the manager clearly expected someone different. Robin does not acknowledge this. She talks about the project she interviewed for with the specificity of someone who read the PRD twice on the plane.

*The coworker:* Someone explains something to her she already knows. Standard. She decides whether to correct them or let it go.

*Trait branches:*
- `AMBITIOUS`: She corrects them once, clearly, and moves on. Establishes position early.
- `SHY`: She lets it go. Spends the rest of the day wondering if she made the right call.
- `BITCHY`: The correction has an edge. The coworker is flustered. She doesn't feel bad about it.

*The professional/physical gap:* She has institutional knowledge and authority in her speech. Her body is presenting as an intern. The gap is constant. The narrator notices it with wry attention.

*Effects:*
- `advance_arc: base::robin_opening → working`
- `set_game_flag: STARTED_JOB`

**Schedule update:** Add these scenes to `packs/base/data/schedule.toml`:

```toml
[[slot]]
name = "robin_opening"

  [[slot.events]]
  scene     = "base::robin_arrival"
  condition = "gd.hasGameFlag('ROUTE_ROBIN')"
  weight    = 0
  trigger   = "gd.hasGameFlag('ROUTE_ROBIN')"
  once_only = true

  [[slot.events]]
  scene     = "base::robin_landlord"
  condition = "gd.hasGameFlag('ROUTE_ROBIN') && !gd.hasGameFlag('MET_LANDLORD')"
  weight    = 0
  trigger   = "gd.hasGameFlag('ROUTE_ROBIN') && !gd.hasGameFlag('MET_LANDLORD')"
  once_only = true

  # etc.
```

**Commit:** `content: robin_first_day — Monday, the office, proving competence in the wrong body`

---

### Task 21: Validate Robin arc end-to-end

**Step 1:** `cargo run --bin validate-pack` — no errors

**Step 2:** `cargo test` — 169+ tests pass (no regressions)

**Step 3:** Build and run the game. Start as Robin. Walk through arrival → landlord → first night → first clothes → first day. Verify:
- All prose renders without template errors
- All thoughts fire at correct conditions
- Arc state advances correctly
- No UI crashes or panics

**Step 4:** `git status` — clean working tree

**Commit:** If any minor fixes needed, `fix: robin arc integration issues`

---

## PART 4: CAMILA'S OPENING ARC
### Session 4 · Content only · Read world.md + camila.md + writing-guide.md first

---

### Task 22: Camila character sheet

**File:** `docs/characters/camila.md`

| Field | Value |
|---|---|
| Route flag | `ROUTE_CAMILA` |
| Arc | `base::camila_opening` |
| Name before | Raul |
| Name now | Camila (she takes a new name — player can customize) |
| Age | 18 (freshman) |
| Race before | Latino |
| Race now | Latina (same ethnicity, different presentation) |
| Background | Privileged family. Private school. Expected success. |
| University | The Ivy |
| FEMININITY start | 10 (CisMaleTransformed) |
| Sexuality before | Straight (attracted to women). Ambient homophobia — not violent, just unexamined |
| Sexuality now | Bi, with strong attraction to men that emerged immediately |

**Character voice:** Camila's interiority is 18 years old and proud. She doesn't have Robin's patience. She hasn't had to develop it — things have always worked out. She's smart and she knows it and she hasn't been tested yet in the way this will test her. The transformation is not just physical: it's the first thing that has ever happened TO her rather than BY her. She processes this badly and then better and then badly again.

**The misogyny/homophobia arc:** He made gay jokes. Not mean-spirited — ambient, the way certain 18-year-old boys do. He had contempt (mild, unexamined) for women as less capable. The transformation is a crash course in being wrong. The learning curve is not linear. He figures something out, then encounters something that puts him back to zero.

**The sexual arousal thread:** Her new body is responsive in ways that ambush her. She wakes up from dreams she didn't ask for. The attraction to men arrived immediately and without warning. The shame is specific: she used to make those jokes. The desire doesn't care about the shame.

---

### Tasks 23–27: Camila scenes

Following the same structure as Robin tasks 16–20, write:

23. `camila_arrival.toml` — campus check-in, dorm assignment, being greeted as a girl
24. `camila_dorm.toml` — first night in dorm. The body, the discovery, the phone call home
25. `camila_orientation.toml` — freshman orientation. Being talked to as a woman by men she'd have dismissed. Noticing things.
26. `camila_library.toml` — first week study scene. Male classmate. The attraction arrives before the thought.
27. `camila_call_raul.toml` — friend from home calls for Raul. She has to pretend to be someone who knows where Raul is.

Each scene: arc-condition gated, once-only where appropriate, full transformation branching, trait branches that change what happens (not adjective swaps), BG3 companion narrator register.

**Overlap test:** After writing Camila scenes, confirm that universal scenes (rain_shelter, coffee_shop, morning_routine) fire for both Robin and Camila without route-specific content contaminating each other. Both characters should be able to encounter the same coffee shop and get different thoughts/inner voices but the same NPC actions and base prose.

**Commit per scene:** `content: camila_[scene] — ...`

---

### Task 28: Final validation and handoff

**Step 1:** `cargo run --bin validate-pack` — no errors

**Step 2:** `cargo test` — all pass (≥169 + new tests from engine changes)

**Step 3:** Build and run. Test Robin arc. Test Camila arc. Test a universal scene with both characters.

**Step 4:** `git status` — clean

**Step 5:** Update `HANDOFF.md`:
- Current state: describe what was built
- Engine capabilities: check system, arc system, thoughts, narrator variants
- Content: Robin and Camila opening arcs
- Next action: second route NPC for each character, or next universal location scenes
- Session log: add entry

**Step 6:** Push to remote.

**Commit:** `docs: update HANDOFF after engine-routes-foundation session`

---

## Appendix: Check System Reference

### Syntax in TOML condition fields

```toml
# Basic percentile check (skill=50, dc=50 → 50% chance)
condition = "w.checkSkill('CHARM', 50)"

# Tiered: evaluate from hardest to easiest — same roll is reused
# (All in [[actions.next]] branches, ordered hard→easy)
[[actions.next]]
goto = "great_result"
if   = "w.checkSkill('CHARM', 20)"  # needs 80% skill

[[actions.next]]
goto = "okay_result"
if   = "w.checkSkill('CHARM', 60)"  # needs 40% skill

[[actions.next]]
goto = "poor_result"
# No if = unconditional fallback

# Red check (one-shot, failure is permanent for this scene+skill)
condition = "w.checkSkillRed('CHARM', 50)"

# Combined with trait check
condition = "w.hasTrait('FLIRTY') && w.checkSkill('CHARM', 40)"
```

### Effect syntax

```toml
[[actions.effects]]
type     = "advance_arc"
arc      = "base::robin_opening"
to_state = "week_one"

[[actions.effects]]
type  = "set_npc_role"
npc   = "m"
role  = "ROLE_LANDLORD"

[[actions.effects]]
type  = "fail_red_check"
skill = "CHARM"
```

### Thought syntax

```toml
[[thoughts]]
condition = "!w.alwaysFemale() && w.getSkill('FEMININITY') < 20"
style     = "inner_voice"
prose     = """
*Okay*, she thinks — he thinks — *okay*. There is a problem and we are going to solve the problem.
"""
```

### Narrator variant syntax

```toml
[[intro_variants]]
condition = "!w.alwaysFemale() && w.getSkill('FEMININITY') < 20"
prose     = """
# Disorientation register — for very-low-FEMININITY CisMaleTransformed
"""

[[intro_variants]]
condition = "w.hasTrait('TRANS_WOMAN')"
prose     = """
# Relief/recognition register — for TransWomanTransformed
"""

[intro]
prose = """
# Default — always-female or high FEMININITY
"""
```
