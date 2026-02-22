# Scene Engine Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a fully tested scene engine backend — pack disk loader, expression evaluator wired to real registry lookups, typed effect system, minijinja prose rendering, and event-queue-based SceneEngine.

**Architecture:** `undone-packs` gains a disk loader; `undone-expr` gains a `&PackRegistry` param on `eval` wiring the TODO stubs; `undone-scene` is built from scratch with TOML deserialization, Effect application, a scene file loader, and a stateful `SceneEngine`. All decoupled from UI via an event queue (`EngineCommand` / `EngineEvent`).

**Tech stack:** Rust, serde/toml for deserialization, minijinja for prose templates, rand (SmallRng) for NPC action weighting, thiserror for error types, the custom `undone-expr` parser already in place.

**Design doc:** `docs/plans/2026-02-22-scene-engine-design.md` — read this before starting.

---

## Important Notes Before Starting

- **After writing each `.rs` file:** call `mcp__rust__get_diagnostics` on it, then `mcp__rust__format_code` on it.
- **Dependency direction** (no cycles allowed): `undone-domain` ← `undone-world` ← `undone-packs` ← `undone-expr` ← `undone-scene` ← `undone-ui`. Adding `undone-packs` as a dep to `undone-expr` is fine (no cycle).
- **TDD:** Write the failing test first, confirm it fails, then implement, then confirm it passes.
- **Ordinal enums** (`LikingLevel`, `LoveLevel`, `ArousalLevel`) are stepped by `i8` delta and clamped at the min/max variant.
- **minijinja `Object` trait** is how we expose `w.hasTrait("SHY")` in prose templates. Each receiver (`w`, `m`, `gd`, `scene`) becomes a Rust struct implementing `minijinja::value::Object`.

---

## Task 1: Add Missing Dependencies

**Files:**
- Modify: `Cargo.toml` (workspace root)
- Modify: `crates/undone-expr/Cargo.toml`
- Modify: `crates/undone-scene/Cargo.toml`

**Step 1: Add `rand` to workspace deps**

In `Cargo.toml`, add to `[workspace.dependencies]`:

```toml
rand = { version = "0.8", features = ["small_rng"] }
```

**Step 2: Add `undone-packs` to `undone-expr`'s deps**

In `crates/undone-expr/Cargo.toml`, add to `[dependencies]`:

```toml
undone-packs = { path = "../undone-packs" }
```

**Step 3: Add `rand` to `undone-scene`**

In `crates/undone-scene/Cargo.toml`, add to `[dependencies]`:

```toml
rand = { workspace = true }
```

**Step 4: Verify it compiles**

```
cargo check
```

Expected: clean (no changes to logic yet, just new deps available).

**Step 5: Commit**

```
git add Cargo.toml crates/undone-expr/Cargo.toml crates/undone-scene/Cargo.toml
git commit -m "deps: add rand to workspace, undone-packs to undone-expr"
```

---

## Task 2: Wire Expression Evaluator to PackRegistry

**Files:**
- Modify: `crates/undone-packs/src/registry.rs` — add `get_stat`, `resolve_trait_str`
- Modify: `crates/undone-expr/src/eval.rs` — add registry param, wire stubs
- Modify: `crates/undone-expr/src/lib.rs` — re-export updated signature

The eval functions currently have `// TODO: wire to registry` stubs. We'll add `registry: &PackRegistry` as the 4th parameter throughout and implement the lookups.

**Step 1: Add `get_stat` to PackRegistry**

In `crates/undone-packs/src/registry.rs`, add this method to `impl PackRegistry`:

```rust
/// Look up an already-interned stat name without mutating. Returns None if never interned.
pub fn get_stat(&self, id: &str) -> Option<StatId> {
    self.rodeo.get(id).map(StatId)
}
```

Run diagnostics: `mcp__rust__get_diagnostics` on `crates/undone-packs/src/registry.rs`.

**Step 2: Write failing tests for wired eval**

In `crates/undone-expr/src/eval.rs`, add to the `#[cfg(test)]` block (after the existing tests):

```rust
#[test]
fn hasTrait_true_when_player_has_trait() {
    let mut reg = undone_packs::PackRegistry::new();
    reg.register_traits(vec![undone_packs::TraitDef {
        id: "SHY".into(),
        name: "Shy".into(),
        description: "...".into(),
        hidden: false,
    }]);
    let shy_id = reg.resolve_trait("SHY").unwrap();
    let mut world = make_world();
    world.player.traits.insert(shy_id);
    let ctx = SceneCtx::new();
    let expr = parse("w.hasTrait('SHY')").unwrap();
    assert!(eval(&expr, &world, &ctx, &reg).unwrap());
}

#[test]
fn hasTrait_false_when_player_lacks_trait() {
    let mut reg = undone_packs::PackRegistry::new();
    reg.register_traits(vec![undone_packs::TraitDef {
        id: "SHY".into(),
        name: "Shy".into(),
        description: "...".into(),
        hidden: false,
    }]);
    let world = make_world();
    let ctx = SceneCtx::new();
    let expr = parse("w.hasTrait('SHY')").unwrap();
    assert!(!eval(&expr, &world, &ctx, &reg).unwrap());
}

#[test]
fn getSkill_returns_effective_value() {
    let mut reg = undone_packs::PackRegistry::new();
    reg.register_skills(vec![undone_packs::SkillDef {
        id: "FITNESS".into(),
        name: "Fitness".into(),
        description: "...".into(),
        min: 0,
        max: 100,
    }]);
    let skill_id = reg.resolve_skill("FITNESS").unwrap();
    let mut world = make_world();
    world.player.skills.insert(skill_id, undone_domain::SkillValue { value: 60, modifier: -10 });
    let ctx = SceneCtx::new();
    let expr = parse("w.getSkill('FITNESS') > 40").unwrap();
    assert!(eval(&expr, &world, &ctx, &reg).unwrap());
}
```

Run: `cargo test -p undone-expr`
Expected: **compile error** — `eval` doesn't accept 4 args yet.

**Step 3: Update eval signatures**

Rewrite `crates/undone-expr/src/eval.rs` with `registry: &PackRegistry` added to all eval functions. Full updated signatures:

```rust
use undone_packs::PackRegistry;

pub fn eval(expr: &Expr, world: &World, ctx: &SceneCtx, registry: &PackRegistry) -> Result<bool, EvalError>

fn int_compare(l: &Expr, r: &Expr, world: &World, ctx: &SceneCtx, registry: &PackRegistry, cmp: impl Fn(i64, i64) -> bool) -> Result<bool, EvalError>

fn eval_to_value(expr: &Expr, world: &World, ctx: &SceneCtx, registry: &PackRegistry) -> Result<EvalValue, EvalError>

fn eval_to_int(expr: &Expr, world: &World, ctx: &SceneCtx, registry: &PackRegistry) -> Result<i64, EvalError>

pub fn eval_call_bool(call: &Call, world: &World, ctx: &SceneCtx, registry: &PackRegistry) -> Result<bool, EvalError>

pub fn eval_call_int(call: &Call, world: &World, _ctx: &SceneCtx, registry: &PackRegistry) -> Result<i64, EvalError>
```

All recursive calls inside `eval` pass `registry` through.

**Step 4: Wire the stubs**

In `eval_call_bool`, `Receiver::Player` match arm:

```rust
"hasTrait" => {
    let id = str_arg(0)?;
    let trait_id = registry
        .resolve_trait(id)
        .map_err(|_| EvalError::UnknownTrait(id.to_string()))?;
    Ok(world.player.has_trait(trait_id))
}
"hasStuff" => {
    let _ = str_arg(0)?;
    Ok(false) // TODO: wire when stuff registry exists
}
```

In `eval_call_bool`, `Receiver::MaleNpc` match arm:

```rust
"hasTrait" => {
    let id = str_arg(0)?;
    let trait_id = registry
        .resolve_npc_trait(id)
        .map_err(|_| EvalError::UnknownNpcTrait(id.to_string()))?;
    Ok(npc.core.has_trait(trait_id))
}
```

In `eval_call_int`, `Receiver::Player` match arm:

```rust
"getSkill" => {
    let id = str_arg(0)?;
    let skill_id = registry
        .resolve_skill(id)
        .map_err(|_| EvalError::UnknownSkill(id.to_string()))?;
    Ok(world.player.skill(skill_id) as i64)
}
```

In `eval_call_int`, `Receiver::GameData` match arm:

```rust
"getStat" => {
    let id = str_arg(0)?;
    let stat_id = registry.get_stat(id).unwrap_or_else(|| {
        // Stat not interned means it was never set — value is 0
        undone_domain::StatId(lasso::Spur::try_from_usize(usize::MAX).unwrap())
    });
    Ok(world.game_data.get_stat(stat_id) as i64)
}
```

Wait — using `usize::MAX` as a fake Spur is fragile. Better approach: if `registry.get_stat(id)` returns `None`, just return `Ok(0)` directly:

```rust
"getStat" => {
    let id = str_arg(0)?;
    match registry.get_stat(id) {
        Some(stat_id) => Ok(world.game_data.get_stat(stat_id) as i64),
        None => Ok(0), // stat never interned = was never set
    }
}
```

**Step 5: Add new EvalError variants**

```rust
#[error("unknown trait '{0}'")]
UnknownTrait(String),
#[error("unknown npc trait '{0}'")]
UnknownNpcTrait(String),
#[error("unknown skill '{0}'")]
UnknownSkill(String),
```

Also add a helper `str_arg` closure that works in `eval_call_int` (currently only in `eval_call_bool`). Add it to `eval_call_int` the same way.

**Step 6: Fix existing tests**

All existing tests in `eval.rs` call `eval(&expr, &world, &ctx)` — they now need a registry argument. Add `let reg = undone_packs::PackRegistry::new();` to each existing test and pass `&reg` as the 4th arg.

Add to dev-dependencies in `crates/undone-expr/Cargo.toml`:
```toml
undone-packs = { path = "../undone-packs" }
```
(Move from `[dependencies]` — actually `undone-packs` should be in `[dependencies]` since `eval` uses it in its public signature. The dev-dep is already covered.)

**Step 7: Run and confirm all tests pass**

```
cargo test -p undone-expr
```

Expected: all tests pass (existing 7 + new 3 = 10).

Run diagnostics + format: `mcp__rust__get_diagnostics` and `mcp__rust__format_code` on `crates/undone-expr/src/eval.rs`.

**Step 8: Commit**

```
git add crates/undone-expr/ crates/undone-packs/src/registry.rs
git commit -m "feat(expr): wire eval stubs to PackRegistry — hasTrait, getSkill, getStat"
```

---

## Task 3: Pack Disk Loader

**Files:**
- Create: `crates/undone-packs/src/loader.rs`
- Modify: `crates/undone-packs/src/lib.rs` — add `pub mod loader` and re-exports

**Step 1: Write a failing test**

In `crates/undone-packs/src/loader.rs` (create it):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn packs_dir() -> PathBuf {
        // The test runs from the workspace root
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()  // crates/
            .parent().unwrap()  // workspace root
            .join("packs")
    }

    #[test]
    fn loads_base_pack_traits() {
        let (registry, metas) = load_packs(&packs_dir()).unwrap();
        assert!(!metas.is_empty());
        assert!(registry.resolve_trait("SHY").is_ok());
        assert!(registry.resolve_trait("POSH").is_ok());
    }

    #[test]
    fn loads_base_pack_skills() {
        let (registry, _) = load_packs(&packs_dir()).unwrap();
        assert!(registry.resolve_skill("FEMININITY").is_ok());
    }

    #[test]
    fn error_on_nonexistent_dir() {
        let result = load_packs(std::path::Path::new("/nonexistent/packs"));
        assert!(result.is_err());
    }
}
```

Run: `cargo test -p undone-packs loader`
Expected: compile error — `load_packs` not defined yet.

**Step 2: Check what traits/skills exist in packs/base/data/**

Read `packs/base/data/traits.toml` and `packs/base/data/skills.toml` to know what IDs to expect. Adjust test assertions to match actual IDs in those files.

**Step 3: Implement the loader**

```rust
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::{
    data::{NpcTraitFile, SkillFile, TraitFile},
    manifest::PackManifest,
    registry::PackRegistry,
};

#[derive(Debug, Error)]
pub enum PackLoadError {
    #[error("io error reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("toml parse error in {path}: {message}")]
    Toml { path: PathBuf, message: String },
    #[error("packs directory not found: {0}")]
    PacksDirNotFound(PathBuf),
}

pub struct LoadedPackMeta {
    pub manifest: PackManifest,
    pub pack_dir: PathBuf,
}

/// Walk `packs_dir`, load all packs into a PackRegistry.
/// Returns the populated registry and metadata for each loaded pack.
pub fn load_packs(packs_dir: &Path) -> Result<(PackRegistry, Vec<LoadedPackMeta>), PackLoadError> {
    if !packs_dir.exists() {
        return Err(PackLoadError::PacksDirNotFound(packs_dir.to_path_buf()));
    }

    let mut registry = PackRegistry::new();
    let mut metas = Vec::new();

    let entries = std::fs::read_dir(packs_dir).map_err(|e| PackLoadError::Io {
        path: packs_dir.to_path_buf(),
        source: e,
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| PackLoadError::Io {
            path: packs_dir.to_path_buf(),
            source: e,
        })?;
        let pack_dir = entry.path();
        if !pack_dir.is_dir() {
            continue;
        }
        let manifest_path = pack_dir.join("pack.toml");
        if !manifest_path.exists() {
            continue;
        }
        let meta = load_one_pack(&mut registry, &pack_dir)?;
        metas.push(meta);
    }

    Ok((registry, metas))
}

fn load_one_pack(registry: &mut PackRegistry, pack_dir: &Path) -> Result<LoadedPackMeta, PackLoadError> {
    let manifest_path = pack_dir.join("pack.toml");
    let src = read_file(&manifest_path)?;
    let manifest: PackManifest =
        toml::from_str(&src).map_err(|e| PackLoadError::Toml {
            path: manifest_path.clone(),
            message: e.to_string(),
        })?;

    // Load traits
    let traits_path = pack_dir.join(&manifest.content.traits);
    let src = read_file(&traits_path)?;
    let trait_file: TraitFile =
        toml::from_str(&src).map_err(|e| PackLoadError::Toml {
            path: traits_path.clone(),
            message: e.to_string(),
        })?;
    registry.register_traits(trait_file.traits);

    // Load npc_traits
    let npc_traits_path = pack_dir.join(&manifest.content.npc_traits);
    let src = read_file(&npc_traits_path)?;
    let npc_trait_file: NpcTraitFile =
        toml::from_str(&src).map_err(|e| PackLoadError::Toml {
            path: npc_traits_path.clone(),
            message: e.to_string(),
        })?;
    registry.register_npc_traits(npc_trait_file.traits);

    // Load skills
    let skills_path = pack_dir.join(&manifest.content.skills);
    let src = read_file(&skills_path)?;
    let skill_file: SkillFile =
        toml::from_str(&src).map_err(|e| PackLoadError::Toml {
            path: skills_path.clone(),
            message: e.to_string(),
        })?;
    registry.register_skills(skill_file.skills);

    Ok(LoadedPackMeta {
        manifest,
        pack_dir: pack_dir.to_path_buf(),
    })
}

fn read_file(path: &Path) -> Result<String, PackLoadError> {
    std::fs::read_to_string(path).map_err(|e| PackLoadError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}
```

**Step 4: Export from lib.rs**

In `crates/undone-packs/src/lib.rs`:

```rust
pub mod data;
pub mod loader;
pub mod manifest;
pub mod registry;

pub use data::{NpcTraitDef, SkillDef, TraitDef};
pub use loader::{load_packs, LoadedPackMeta, PackLoadError};
pub use manifest::{PackContent, PackManifest, PackMeta};
pub use registry::{PackRegistry, RegistryError};
```

**Step 5: Run diagnostics + format, then run tests**

```
cargo test -p undone-packs
```

Expected: all tests pass.

**Step 6: Commit**

```
git add crates/undone-packs/
git commit -m "feat(packs): add disk loader — walks packs dir, populates registry from data files"
```

---

## Task 4: SceneDefinition Types and Effect Enum

**Files:**
- Create: `crates/undone-scene/src/types.rs`
- Modify: `crates/undone-scene/src/lib.rs` — replace placeholder

This task defines the data structures. No logic yet.

**Step 1: Write failing tests for TOML parsing**

Create `crates/undone-scene/src/types.rs` with tests first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_SCENE: &str = r#"
[scene]
id = "test::minimal"
pack = "test"
description = "A minimal test scene."

[intro]
prose = "It begins."

[[actions]]
id = "wait"
label = "Wait"
detail = "Just wait."

[[actions]]
id = "leave"
label = "Leave"
condition = "!scene.hasFlag('blocked')"
prose = "You leave."

  [[actions.effects]]
  type = "change_stress"
  amount = -1

  [[actions.next]]
  finish = true
"#;

    #[test]
    fn parses_minimal_scene() {
        let raw: SceneToml = toml::from_str(MINIMAL_SCENE).unwrap();
        assert_eq!(raw.scene.id, "test::minimal");
        assert_eq!(raw.actions.len(), 2);
    }

    #[test]
    fn parses_action_effects() {
        let raw: SceneToml = toml::from_str(MINIMAL_SCENE).unwrap();
        let leave = raw.actions.iter().find(|a| a.id == "leave").unwrap();
        assert_eq!(leave.effects.len(), 1);
        assert!(matches!(leave.effects[0], EffectDef::ChangeStress { amount: -1 }));
    }

    #[test]
    fn parses_action_next() {
        let raw: SceneToml = toml::from_str(MINIMAL_SCENE).unwrap();
        let leave = raw.actions.iter().find(|a| a.id == "leave").unwrap();
        assert_eq!(leave.next.len(), 1);
        assert!(leave.next[0].finish);
    }

    #[test]
    fn action_with_no_next_has_empty_vec() {
        let raw: SceneToml = toml::from_str(MINIMAL_SCENE).unwrap();
        let wait = raw.actions.iter().find(|a| a.id == "wait").unwrap();
        assert!(wait.next.is_empty());
    }
}
```

Run: `cargo test -p undone-scene`
Expected: compile error — types not defined yet.

**Step 2: Implement types**

```rust
use serde::Deserialize;

/// Raw TOML deserialization target for a scene file.
#[derive(Debug, Deserialize)]
pub struct SceneToml {
    pub scene: SceneMeta,
    pub intro: IntroDef,
    #[serde(default)]
    pub actions: Vec<ActionDef>,
    #[serde(default)]
    pub npc_actions: Vec<NpcActionDef>,
}

#[derive(Debug, Deserialize)]
pub struct SceneMeta {
    pub id: String,
    pub pack: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct IntroDef {
    pub prose: String,
    /// Informational: which action group follows intro. Not used for filtering.
    #[serde(default)]
    pub next: String,
}

#[derive(Debug, Deserialize)]
pub struct ActionDef {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub detail: String,
    /// Expression string; parsed + validated at load time.
    pub condition: Option<String>,
    #[serde(default)]
    pub prose: String,
    #[serde(default)]
    pub allow_npc_actions: bool,
    #[serde(default)]
    pub use_default_actions: bool,
    #[serde(default)]
    pub effects: Vec<EffectDef>,
    /// Empty = loop (re-evaluate conditions, re-show all actions).
    #[serde(default)]
    pub next: Vec<NextBranchDef>,
}

#[derive(Debug, Deserialize)]
pub struct NpcActionDef {
    pub id: String,
    pub condition: Option<String>,
    #[serde(default)]
    pub prose: String,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default)]
    pub effects: Vec<EffectDef>,
}

fn default_weight() -> u32 { 1 }

#[derive(Debug, Deserialize)]
pub struct NextBranchDef {
    /// Expression string; if absent this branch is unconditional.
    #[serde(rename = "if")]
    pub condition: Option<String>,
    pub goto: Option<String>,
    #[serde(default)]
    pub finish: bool,
}

/// Typed effect, deserialised from `type = "..."` tagged TOML.
/// String IDs (trait_id, skill, stat, npc) are validated against
/// PackRegistry at scene load time but kept as Strings at runtime.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EffectDef {
    ChangeStress { amount: i32 },
    ChangeMoney { amount: i32 },
    ChangeAnxiety { amount: i32 },
    SetSceneFlag { flag: String },
    RemoveSceneFlag { flag: String },
    SetGameFlag { flag: String },
    RemoveGameFlag { flag: String },
    AddStat { stat: String, amount: i32 },
    SetStat { stat: String, value: i32 },
    SkillIncrease { skill: String, amount: i32 },
    AddTrait { trait_id: String },
    RemoveTrait { trait_id: String },
    AddArousal { delta: i8 },
    AddNpcLiking { npc: String, delta: i8 },
    AddNpcLove { npc: String, delta: i8 },
    AddWLiking { npc: String, delta: i8 },
    SetNpcFlag { npc: String, flag: String },
    AddNpcTrait { npc: String, trait_id: String },
    Transition { target: String },
}
```

**Step 3: Define resolved runtime types**

Still in `types.rs`, add the resolved types (post-validation):

```rust
use std::sync::Arc;
use undone_expr::parser::Expr;

/// Resolved action — conditions parsed, ready for runtime evaluation.
#[derive(Debug)]
pub struct Action {
    pub id: String,
    pub label: String,
    pub detail: String,
    pub condition: Option<Expr>,
    pub prose: String,
    pub allow_npc_actions: bool,
    pub effects: Vec<EffectDef>,       // reuse EffectDef at runtime
    pub next: Vec<NextBranch>,
}

#[derive(Debug)]
pub struct NextBranch {
    pub condition: Option<Expr>,
    pub goto: Option<String>,
    pub finish: bool,
}

#[derive(Debug)]
pub struct NpcAction {
    pub id: String,
    pub condition: Option<Expr>,
    pub prose: String,
    pub weight: u32,
    pub effects: Vec<EffectDef>,
}

/// Immutable scene definition. Wrap in Arc for cheap cloning.
#[derive(Debug)]
pub struct SceneDefinition {
    pub id: String,
    pub pack: String,
    pub intro_prose: String,
    pub actions: Vec<Action>,
    pub npc_actions: Vec<NpcAction>,
}
```

**Step 4: Update lib.rs**

Replace `// placeholder` with:

```rust
pub mod types;
pub use types::{
    Action, EffectDef, NpcAction, NextBranch, SceneDefinition, SceneMeta, SceneToml,
};
```

**Step 5: Run diagnostics + format on types.rs, then run tests**

```
cargo test -p undone-scene
```

Expected: all new tests pass.

**Step 6: Commit**

```
git add crates/undone-scene/
git commit -m "feat(scene): SceneDefinition types — TOML deserialization structs and resolved runtime types"
```

---

## Task 5: Effect Application

**Files:**
- Create: `crates/undone-scene/src/effects.rs`
- Modify: `crates/undone-scene/src/lib.rs` — add module

**Step 1: Write failing tests**

```rust
// In crates/undone-scene/src/effects.rs

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use slotmap::SlotMap;
    use undone_domain::*;
    use undone_world::{GameData, World};
    use undone_expr::eval::SceneCtx;
    use undone_packs::PackRegistry;
    use crate::types::EffectDef;

    fn make_world() -> World {
        World {
            player: Player {
                name: "Eva".into(),
                age: Age::LateTeen,
                race: "east_asian".into(),
                figure: PlayerFigure::Slim,
                breasts: BreastSize::Large,
                eye_colour: "brown".into(),
                hair_colour: "dark".into(),
                traits: HashSet::new(),
                skills: HashMap::new(),
                money: 100,
                stress: 10,
                anxiety: 5,
                arousal: ArousalLevel::Comfort,
                alcohol: AlcoholLevel::Sober,
                partner: None,
                friends: vec![],
                virgin: true,
                anal_virgin: true,
                lesbian_virgin: true,
                on_pill: false,
                pregnancy: None,
                stuff: HashSet::new(),
                custom_flags: HashMap::new(),
                custom_ints: HashMap::new(),
                always_female: false,
                femininity: 10,
            },
            male_npcs: SlotMap::with_key(),
            female_npcs: SlotMap::with_key(),
            game_data: GameData::default(),
        }
    }

    #[test]
    fn change_stress_adds_amount() {
        let mut world = make_world();
        let mut ctx = SceneCtx::new();
        let reg = PackRegistry::new();
        apply_effect(&EffectDef::ChangeStress { amount: 5 }, &mut world, &mut ctx, &reg).unwrap();
        assert_eq!(world.player.stress, 15);
    }

    #[test]
    fn change_money_subtracts() {
        let mut world = make_world();
        let mut ctx = SceneCtx::new();
        let reg = PackRegistry::new();
        apply_effect(&EffectDef::ChangeMoney { amount: -30 }, &mut world, &mut ctx, &reg).unwrap();
        assert_eq!(world.player.money, 70);
    }

    #[test]
    fn set_scene_flag_adds_flag() {
        let mut world = make_world();
        let mut ctx = SceneCtx::new();
        let reg = PackRegistry::new();
        apply_effect(&EffectDef::SetSceneFlag { flag: "test_flag".into() }, &mut world, &mut ctx, &reg).unwrap();
        assert!(ctx.has_flag("test_flag"));
    }

    #[test]
    fn set_game_flag_adds_to_game_data() {
        let mut world = make_world();
        let mut ctx = SceneCtx::new();
        let reg = PackRegistry::new();
        apply_effect(&EffectDef::SetGameFlag { flag: "GLOBAL".into() }, &mut world, &mut ctx, &reg).unwrap();
        assert!(world.game_data.has_flag("GLOBAL"));
    }

    #[test]
    fn add_npc_liking_steps_up_clamped() {
        let mut world = make_world();
        let key = world.male_npcs.insert(make_male_npc());
        let mut ctx = SceneCtx::new();
        ctx.active_male = Some(key);
        let reg = PackRegistry::new();
        // Start at Neutral (0), step +2 → Like (index 2)
        apply_effect(&EffectDef::AddNpcLiking { npc: "m".into(), delta: 2 }, &mut world, &mut ctx, &reg).unwrap();
        assert_eq!(world.male_npcs[key].core.pc_liking, LikingLevel::Like);
    }

    #[test]
    fn add_npc_liking_clamps_at_max() {
        let mut world = make_world();
        let key = world.male_npcs.insert(make_male_npc());
        let mut ctx = SceneCtx::new();
        ctx.active_male = Some(key);
        let reg = PackRegistry::new();
        apply_effect(&EffectDef::AddNpcLiking { npc: "m".into(), delta: 99 }, &mut world, &mut ctx, &reg).unwrap();
        assert_eq!(world.male_npcs[key].core.pc_liking, LikingLevel::Close);
    }

    fn make_male_npc() -> undone_domain::MaleNpc {
        use undone_domain::*;
        use std::collections::{HashMap, HashSet};
        MaleNpc {
            core: NpcCore {
                name: "Test".into(),
                age: Age::Twenties,
                race: "white".into(),
                eye_colour: "blue".into(),
                hair_colour: "brown".into(),
                personality: PersonalityId(lasso::Spur::try_from_usize(0).unwrap()),
                traits: HashSet::new(),
                relationship: RelationshipStatus::Stranger,
                pc_liking: LikingLevel::Neutral,
                npc_liking: LikingLevel::Neutral,
                pc_love: LoveLevel::None,
                npc_love: LoveLevel::None,
                pc_attraction: AttractionLevel::Unattracted,
                npc_attraction: AttractionLevel::Unattracted,
                behaviour: Behaviour::Neutral,
                relationship_flags: HashSet::new(),
                sexual_activities: HashSet::new(),
                custom_flags: HashMap::new(),
                custom_ints: HashMap::new(),
                knowledge: 0,
                contactable: true,
                arousal: ArousalLevel::Comfort,
                alcohol: AlcoholLevel::Sober,
            },
            figure: MaleFigure::Average,
            clothing: MaleClothing::default(),
            had_orgasm: false,
            has_baby_with_pc: false,
        }
    }
}
```

Run: `cargo test -p undone-scene effects`
Expected: compile error — `apply_effect` not defined.

**Step 2: Implement step helpers**

```rust
use undone_domain::{ArousalLevel, LikingLevel, LoveLevel};

fn step_liking(current: LikingLevel, delta: i8) -> LikingLevel {
    const LEVELS: [LikingLevel; 4] = [
        LikingLevel::Neutral, LikingLevel::Ok, LikingLevel::Like, LikingLevel::Close,
    ];
    let idx = LEVELS.iter().position(|&l| l == current).unwrap_or(0) as i32;
    LEVELS[(idx + delta as i32).clamp(0, 3) as usize]
}

fn step_love(current: LoveLevel, delta: i8) -> LoveLevel {
    const LEVELS: [LoveLevel; 5] = [
        LoveLevel::None, LoveLevel::Some, LoveLevel::Confused,
        LoveLevel::Crush, LoveLevel::Love,
    ];
    let idx = LEVELS.iter().position(|&l| l == current).unwrap_or(0) as i32;
    LEVELS[(idx + delta as i32).clamp(0, 4) as usize]
}

fn step_arousal(current: ArousalLevel, delta: i8) -> ArousalLevel {
    const LEVELS: [ArousalLevel; 5] = [
        ArousalLevel::Discomfort, ArousalLevel::Comfort, ArousalLevel::Enjoy,
        ArousalLevel::Close, ArousalLevel::Orgasm,
    ];
    let idx = LEVELS.iter().position(|&l| l == current).unwrap_or(0) as i32;
    LEVELS[(idx + delta as i32).clamp(0, 4) as usize]
}
```

**Step 3: Implement apply_effect**

```rust
use thiserror::Error;
use undone_domain::{FemaleNpcKey, MaleNpcKey, NpcTraitId, TraitId};
use undone_expr::eval::SceneCtx;
use undone_packs::PackRegistry;
use undone_world::World;

use crate::types::EffectDef;

#[derive(Debug, Error)]
pub enum EffectError {
    #[error("effect 'add_npc_liking': npc ref '{0}' is not 'm' or 'f'")]
    BadNpcRef(String),
    #[error("effect requires active male NPC but none is set")]
    NoActiveMale,
    #[error("effect requires active female NPC but none is set")]
    NoActiveFemale,
    #[error("NPC key not found in world")]
    NpcNotFound,
    #[error("unknown trait '{0}'")]
    UnknownTrait(String),
    #[error("unknown npc trait '{0}'")]
    UnknownNpcTrait(String),
    #[error("unknown skill '{0}'")]
    UnknownSkill(String),
}

pub fn apply_effect(
    effect: &EffectDef,
    world: &mut World,
    ctx: &mut SceneCtx,
    registry: &PackRegistry,
) -> Result<(), EffectError> {
    match effect {
        EffectDef::ChangeStress { amount } => {
            world.player.stress += amount;
        }
        EffectDef::ChangeMoney { amount } => {
            world.player.money += amount;
        }
        EffectDef::ChangeAnxiety { amount } => {
            world.player.anxiety += amount;
        }
        EffectDef::AddArousal { delta } => {
            world.player.arousal = step_arousal(world.player.arousal, *delta);
        }
        EffectDef::SetSceneFlag { flag } => {
            ctx.set_flag(flag.clone());
        }
        EffectDef::RemoveSceneFlag { flag } => {
            ctx.scene_flags.remove(flag.as_str());
        }
        EffectDef::SetGameFlag { flag } => {
            world.game_data.set_flag(flag.clone());
        }
        EffectDef::RemoveGameFlag { flag } => {
            world.game_data.remove_flag(flag.as_str());
        }
        EffectDef::AddStat { stat, amount } => {
            let id = registry.intern_stat(stat); // intern is &mut — need mut registry
            // NOTE: registry is &PackRegistry (immutable). Use get_stat which doesn't intern.
            // Stats used in effects must have been interned at load time.
            if let Some(sid) = registry.get_stat(stat) {
                world.game_data.add_stat(sid, *amount);
            }
            // If stat was never interned (unknown), silently ignore — shouldn't happen post-validation
        }
        EffectDef::SetStat { stat, value } => {
            if let Some(sid) = registry.get_stat(stat) {
                world.game_data.set_stat(sid, *value);
            }
        }
        EffectDef::AddTrait { trait_id } => {
            let tid = registry.resolve_trait(trait_id)
                .map_err(|_| EffectError::UnknownTrait(trait_id.clone()))?;
            world.player.traits.insert(tid);
        }
        EffectDef::RemoveTrait { trait_id } => {
            let tid = registry.resolve_trait(trait_id)
                .map_err(|_| EffectError::UnknownTrait(trait_id.clone()))?;
            world.player.traits.remove(&tid);
        }
        EffectDef::SkillIncrease { skill, amount } => {
            let sid = registry.resolve_skill(skill)
                .map_err(|_| EffectError::UnknownSkill(skill.clone()))?;
            let entry = world.player.skills.entry(sid).or_insert(undone_domain::SkillValue { value: 0, modifier: 0 });
            entry.value += amount;
        }
        EffectDef::AddNpcLiking { npc, delta } => {
            let key = resolve_male_npc_ref(npc, ctx)?;
            let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
            npc_data.core.pc_liking = step_liking(npc_data.core.pc_liking, *delta);
        }
        EffectDef::AddNpcLove { npc, delta } => {
            let key = resolve_male_npc_ref(npc, ctx)?;
            let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
            npc_data.core.npc_love = step_love(npc_data.core.npc_love, *delta);
        }
        EffectDef::AddWLiking { npc, delta } => {
            let key = resolve_male_npc_ref(npc, ctx)?;
            let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
            npc_data.core.npc_liking = step_liking(npc_data.core.npc_liking, *delta);
        }
        EffectDef::SetNpcFlag { npc, flag } => {
            let key = resolve_male_npc_ref(npc, ctx)?;
            let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
            npc_data.core.relationship_flags.insert(flag.clone());
        }
        EffectDef::AddNpcTrait { npc, trait_id } => {
            let key = resolve_male_npc_ref(npc, ctx)?;
            let tid = registry.resolve_npc_trait(trait_id)
                .map_err(|_| EffectError::UnknownNpcTrait(trait_id.clone()))?;
            let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
            npc_data.core.traits.insert(tid);
        }
        EffectDef::Transition { .. } | EffectDef::Finish => {
            // Handled by the engine's routing logic, not here.
            // Finish/Transition in an effects list means "after all other effects, do this routing."
            // The engine checks for these during next-branch evaluation.
        }
    }
    Ok(())
}

fn resolve_male_npc_ref(npc: &str, ctx: &SceneCtx) -> Result<MaleNpcKey, EffectError> {
    match npc {
        "m" => ctx.active_male.ok_or(EffectError::NoActiveMale),
        _ => Err(EffectError::BadNpcRef(npc.to_string())),
    }
}
```

**Note on `AddStat`:** `intern_stat` takes `&mut self`. We can't call it on `&PackRegistry`. At load time the scene loader will intern all stat IDs referenced in effects. At runtime, use `get_stat` (immutable). If `get_stat` returns `None`, the stat was never used — silently ignore (this indicates a bug in scene authoring that load-time validation should catch).

**Step 4: Add `pub mod effects` to lib.rs**

```rust
pub mod effects;
pub use effects::{apply_effect, EffectError};
```

**Step 5: Run tests**

```
cargo test -p undone-scene effects
```

Expected: all tests pass.

**Step 6: Commit**

```
git add crates/undone-scene/
git commit -m "feat(scene): effect application — apply_effect with ordinal stepping and clamping"
```

---

## Task 6: Scene File Loader

**Files:**
- Create: `crates/undone-scene/src/loader.rs`
- Modify: `crates/undone-scene/src/lib.rs` — add module + exports

The loader walks a `scenes/` directory, parses each `.toml` file, validates all expression strings by parsing them, and validates effect string IDs against the registry.

**Step 1: Write failing tests**

```rust
// In crates/undone-scene/src/loader.rs

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()
            .parent().unwrap()
            .join("packs")
    }

    #[test]
    fn loads_rain_shelter_scene() {
        let (registry, _) = undone_packs::load_packs(&packs_dir()).unwrap();
        let scenes_dir = packs_dir().join("base").join("scenes");
        let scenes = load_scenes(&scenes_dir, "base", &registry).unwrap();
        assert!(scenes.contains_key("base::rain_shelter"));
    }

    #[test]
    fn rain_shelter_has_expected_actions() {
        let (registry, _) = undone_packs::load_packs(&packs_dir()).unwrap();
        let scenes_dir = packs_dir().join("base").join("scenes");
        let scenes = load_scenes(&scenes_dir, "base", &registry).unwrap();
        let scene = &scenes["base::rain_shelter"];
        let action_ids: Vec<&str> = scene.actions.iter().map(|a| a.id.as_str()).collect();
        assert!(action_ids.contains(&"main"));
        assert!(action_ids.contains(&"leave"));
        assert!(action_ids.contains(&"accept_umbrella"));
    }

    #[test]
    fn error_on_nonexistent_scenes_dir() {
        let registry = undone_packs::PackRegistry::new();
        let result = load_scenes(std::path::Path::new("/no/such/dir"), "test", &registry);
        assert!(result.is_err());
    }
}
```

Run: `cargo test -p undone-scene loader`
Expected: compile error (and will fail on missing scene file later — that's fine).

**Step 2: Implement loader**

```rust
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use thiserror::Error;
use undone_expr::parser::{parse, Expr};
use undone_packs::PackRegistry;

use crate::types::{
    Action, EffectDef, NpcAction, NextBranch, SceneDefinition, SceneToml,
};

#[derive(Debug, Error)]
pub enum SceneLoadError {
    #[error("io error reading {path}: {source}")]
    Io { path: PathBuf, #[source] source: std::io::Error },
    #[error("toml parse error in {path}: {message}")]
    Toml { path: PathBuf, message: String },
    #[error("parse error in condition '{expr}' in scene {scene_id}: {message}")]
    BadCondition { scene_id: String, expr: String, message: String },
    #[error("unknown trait '{id}' in scene {scene_id}")]
    UnknownTrait { scene_id: String, id: String },
    #[error("unknown skill '{id}' in scene {scene_id}")]
    UnknownSkill { scene_id: String, id: String },
    #[error("scenes directory not found: {0}")]
    DirNotFound(PathBuf),
}

pub fn load_scenes(
    scenes_dir: &Path,
    _pack_id: &str,
    registry: &PackRegistry,
) -> Result<HashMap<String, Arc<SceneDefinition>>, SceneLoadError> {
    if !scenes_dir.exists() {
        return Err(SceneLoadError::DirNotFound(scenes_dir.to_path_buf()));
    }

    let mut out = HashMap::new();

    let entries = std::fs::read_dir(scenes_dir).map_err(|e| SceneLoadError::Io {
        path: scenes_dir.to_path_buf(),
        source: e,
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| SceneLoadError::Io {
            path: scenes_dir.to_path_buf(),
            source: e,
        })?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let src = std::fs::read_to_string(&path).map_err(|e| SceneLoadError::Io {
            path: path.clone(),
            source: e,
        })?;
        let raw: SceneToml = toml::from_str(&src).map_err(|e| SceneLoadError::Toml {
            path: path.clone(),
            message: e.to_string(),
        })?;

        let scene_id = raw.scene.id.clone();
        let scene = resolve_scene(raw, registry, &scene_id)?;
        out.insert(scene_id, Arc::new(scene));
    }

    Ok(out)
}

fn parse_condition(expr_str: &str, scene_id: &str) -> Result<Expr, SceneLoadError> {
    parse(expr_str).map_err(|e| SceneLoadError::BadCondition {
        scene_id: scene_id.to_string(),
        expr: expr_str.to_string(),
        message: e.to_string(),
    })
}

fn resolve_scene(
    raw: SceneToml,
    registry: &PackRegistry,
    scene_id: &str,
) -> Result<SceneDefinition, SceneLoadError> {
    let mut actions = Vec::new();
    for a in raw.actions {
        let condition = a.condition.as_deref()
            .map(|s| parse_condition(s, scene_id))
            .transpose()?;

        let mut next = Vec::new();
        for b in a.next {
            let cond = b.condition.as_deref()
                .map(|s| parse_condition(s, scene_id))
                .transpose()?;
            next.push(NextBranch { condition: cond, goto: b.goto, finish: b.finish });
        }

        validate_effects(&a.effects, registry, scene_id)?;

        actions.push(Action {
            id: a.id,
            label: a.label,
            detail: a.detail,
            condition,
            prose: a.prose,
            allow_npc_actions: a.allow_npc_actions,
            effects: a.effects,
            next,
        });
    }

    let mut npc_actions = Vec::new();
    for na in raw.npc_actions {
        let condition = na.condition.as_deref()
            .map(|s| parse_condition(s, scene_id))
            .transpose()?;
        validate_effects(&na.effects, registry, scene_id)?;
        npc_actions.push(NpcAction {
            id: na.id,
            condition,
            prose: na.prose,
            weight: na.weight,
            effects: na.effects,
        });
    }

    Ok(SceneDefinition {
        id: raw.scene.id,
        pack: raw.scene.pack,
        intro_prose: raw.intro.prose,
        actions,
        npc_actions,
    })
}

fn validate_effects(
    effects: &[EffectDef],
    registry: &PackRegistry,
    scene_id: &str,
) -> Result<(), SceneLoadError> {
    for effect in effects {
        match effect {
            EffectDef::AddTrait { trait_id } | EffectDef::RemoveTrait { trait_id } => {
                registry.resolve_trait(trait_id).map_err(|_| SceneLoadError::UnknownTrait {
                    scene_id: scene_id.to_string(),
                    id: trait_id.clone(),
                })?;
            }
            EffectDef::SkillIncrease { skill, .. } => {
                registry.resolve_skill(skill).map_err(|_| SceneLoadError::UnknownSkill {
                    scene_id: scene_id.to_string(),
                    id: skill.clone(),
                })?;
            }
            _ => {} // Other effects don't need registry validation at load time
        }
    }
    Ok(())
}
```

**Step 3: Add to lib.rs**

```rust
pub mod loader;
pub use loader::{load_scenes, SceneLoadError};
```

**Step 4: Run diagnostics + format, then tests**

```
cargo test -p undone-scene loader
```

Note: the `loads_rain_shelter_scene` test will fail with "DirNotFound" until Task 8 creates the scene file. That's expected — the other two tests should pass.

**Step 5: Commit**

```
git add crates/undone-scene/
git commit -m "feat(scene): scene file loader — parses .toml files, validates conditions and effects at load time"
```

---

## Task 7: SceneEngine with Minijinja Prose Rendering

**Files:**
- Create: `crates/undone-scene/src/engine.rs`
- Create: `crates/undone-scene/src/template_ctx.rs`
- Modify: `crates/undone-scene/src/lib.rs` — add modules and public API

This is the largest task. Build it in two parts: template context first, then the engine.

### Part A: Template Context

Minijinja templates use `{% if w.hasTrait("SHY") %}` style method calls. We expose
`w`, `m`, `f`, `gd`, and `scene` as minijinja `Value` objects implementing the `Object`
trait. This lets minijinja dispatch method calls to our Rust code.

**Step 1: Write failing template render tests**

Create `crates/undone-scene/src/template_ctx.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use undone_domain::*;
    use undone_world::{GameData, World};
    use undone_expr::eval::SceneCtx;
    use undone_packs::PackRegistry;
    use slotmap::SlotMap;

    fn make_world_with_shy() -> (World, PackRegistry) {
        let mut reg = PackRegistry::new();
        reg.register_traits(vec![undone_packs::TraitDef {
            id: "SHY".into(), name: "Shy".into(), description: "...".into(), hidden: false,
        }]);
        let shy_id = reg.resolve_trait("SHY").unwrap();
        let mut world = World {
            player: Player {
                name: "Eva".into(), age: Age::LateTeen, race: "east_asian".into(),
                figure: PlayerFigure::Slim, breasts: BreastSize::Large,
                eye_colour: "brown".into(), hair_colour: "dark".into(),
                traits: HashSet::new(), skills: HashMap::new(),
                money: 100, stress: 0, anxiety: 0,
                arousal: ArousalLevel::Comfort, alcohol: AlcoholLevel::Sober,
                partner: None, friends: vec![],
                virgin: true, anal_virgin: true, lesbian_virgin: true,
                on_pill: false, pregnancy: None, stuff: HashSet::new(),
                custom_flags: HashMap::new(), custom_ints: HashMap::new(),
                always_female: false, femininity: 10,
            },
            male_npcs: SlotMap::with_key(),
            female_npcs: SlotMap::with_key(),
            game_data: GameData::default(),
        };
        world.player.traits.insert(shy_id);
        (world, reg)
    }

    #[test]
    fn hasTrait_in_template_branches_correctly() {
        let (world, reg) = make_world_with_shy();
        let ctx = SceneCtx::new();
        let rendered = render_prose(
            r#"{% if w.hasTrait("SHY") %}shy{% else %}bold{% endif %}"#,
            &world, &ctx, &reg,
        ).unwrap();
        assert_eq!(rendered.trim(), "shy");
    }

    #[test]
    fn scene_hasFlag_in_template() {
        let (world, reg) = make_world_with_shy();
        let mut ctx = SceneCtx::new();
        ctx.set_flag("umbrella_offered");
        let rendered = render_prose(
            r#"{% if scene.hasFlag("umbrella_offered") %}yes{% else %}no{% endif %}"#,
            &world, &ctx, &reg,
        ).unwrap();
        assert_eq!(rendered.trim(), "yes");
    }
}
```

Run: `cargo test -p undone-scene template_ctx`
Expected: compile error — `render_prose` not defined.

**Step 2: Implement template context structs**

Minijinja's `Object` trait has one key method: `call_method`. We implement it for each receiver type. The trait also requires `fmt::Display` (for `{{ w }}`-style output, which we don't use, but must implement).

```rust
use std::{collections::HashSet, fmt, sync::Arc};
use minijinja::{
    value::{Object, Value},
    Error, ErrorKind, State,
};
use undone_domain::{LikingLevel, NpcTraitId, TraitId};
use undone_expr::eval::SceneCtx;
use undone_packs::PackRegistry;
use undone_world::World;

/// Build a minijinja context map for rendering scene prose.
pub fn build_template_ctx(
    world: &World,
    ctx: &SceneCtx,
    registry: &PackRegistry,
) -> minijinja::Value {
    minijinja::context! {
        w => Value::from_object(PlayerCtx {
            traits: world.player.traits.clone(),
            is_virgin: world.player.virgin,
            always_female: world.player.always_female,
            is_single: world.player.partner.is_none(),
            on_pill: world.player.on_pill,
            is_pregnant: world.player.pregnancy.is_some(),
        }),
        gd => Value::from_object(GameDataCtx {
            week: world.game_data.week,
            flags: world.game_data.flags.clone(),
        }),
        scene => Value::from_object(SceneCtxView {
            flags: ctx.scene_flags.clone(),
        }),
    }
}

/// Render a minijinja prose template.
pub fn render_prose(
    template_str: &str,
    world: &World,
    ctx: &SceneCtx,
    registry: &PackRegistry,
) -> Result<String, minijinja::Error> {
    let env = minijinja::Environment::new();
    let tmpl = env.template_from_str(template_str)?;
    let context = build_template_ctx(world, ctx, registry);
    tmpl.render(context)
}

// --- PlayerCtx ---

#[derive(Debug)]
struct PlayerCtx {
    traits: HashSet<TraitId>,
    is_virgin: bool,
    always_female: bool,
    is_single: bool,
    on_pill: bool,
    is_pregnant: bool,
}

impl fmt::Display for PlayerCtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "[player]") }
}

impl Object for PlayerCtx {
    fn call_method(&self, _state: &State, method: &str, args: &[Value]) -> Result<Value, Error> {
        match method {
            "hasTrait" => {
                // Trait IDs in templates are strings. We resolve by interning.
                // Since we have &self, we can't use the registry (which we don't store here).
                // We store pre-resolved trait string names as a HashSet<String> alongside TraitIds.
                // DESIGN NOTE: For template use, we compare against trait string IDs stored
                // separately. See build_template_ctx below for the approach.
                Err(Error::new(ErrorKind::InvalidOperation,
                    "hasTrait in templates: use PlayerCtxWithRegistry"))
            }
            "isVirgin" => Ok(Value::from(self.is_virgin)),
            "alwaysFemale" => Ok(Value::from(self.always_female)),
            "isSingle" => Ok(Value::from(self.is_single)),
            "isOnPill" => Ok(Value::from(self.on_pill)),
            "isPregnant" => Ok(Value::from(self.is_pregnant)),
            _ => Err(Error::new(ErrorKind::UnknownMethod, format!("w.{method}() not found"))),
        }
    }
}
```

Wait — there's a problem with `hasTrait` in templates. The template calls `w.hasTrait("SHY")`, but at template render time we have `TraitId` (interned spur) in `world.player.traits`, not strings. To check `hasTrait("SHY")` we need to resolve "SHY" to a `TraitId` via the registry. But to do that we need the `&PackRegistry`.

The cleanest solution: store a `HashSet<String>` of the resolved trait STRING IDs alongside the `HashSet<TraitId>` in `PlayerCtx`. When building the context, we look up each `TraitId` to get its string form... but `Rodeo` stores `str → spur` not `spur → str`. We'd need `RodeoReader` or to iterate.

Actually, `lasso::Rodeo` has a `resolve` method: `rodeo.resolve(&spur) -> &str`. So we can pre-build a `HashSet<String>` of trait string IDs by iterating the player's traits and resolving each via the registry's internal rodeo.

Add a method to `PackRegistry`:

```rust
pub fn trait_id_to_str(&self, id: TraitId) -> Option<&str> {
    // Rodeo lets us resolve spur → str
    Some(self.rodeo.resolve(&id.0))
}
```

Then in `build_template_ctx`, compute `trait_strings`:

```rust
let trait_strings: HashSet<String> = world.player.traits.iter()
    .filter_map(|tid| registry.trait_id_to_str(*tid))
    .map(|s| s.to_string())
    .collect();
```

And pass `trait_strings` into `PlayerCtx`. Then `hasTrait` in the `Object` impl:

```rust
"hasTrait" => {
    let id = args.first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::new(ErrorKind::MissingArgument, "hasTrait requires a string arg"))?;
    Ok(Value::from(self.trait_strings.contains(id)))
}
```

Update `PlayerCtx` to include `trait_strings: HashSet<String>` and the method to use it.

Add `trait_id_to_str` to `PackRegistry` in `crates/undone-packs/src/registry.rs`.

**Step 3: Implement all context types fully**

Full `template_ctx.rs` with `PlayerCtx`, `GameDataCtx`, `SceneCtxView` all properly implementing `Object`:

```rust
// GameDataCtx
impl Object for GameDataCtx {
    fn call_method(&self, _state: &State, method: &str, args: &[Value]) -> Result<Value, Error> {
        match method {
            "week" => Ok(Value::from(self.week)),
            "hasGameFlag" => {
                let flag = str_arg(args, "hasGameFlag")?;
                Ok(Value::from(self.flags.contains(flag)))
            }
            _ => Err(Error::new(ErrorKind::UnknownMethod, format!("gd.{method}() not found"))),
        }
    }
}

// SceneCtxView
impl Object for SceneCtxView {
    fn call_method(&self, _state: &State, method: &str, args: &[Value]) -> Result<Value, Error> {
        match method {
            "hasFlag" => {
                let flag = str_arg(args, "hasFlag")?;
                Ok(Value::from(self.flags.contains(flag)))
            }
            _ => Err(Error::new(ErrorKind::UnknownMethod, format!("scene.{method}() not found"))),
        }
    }
}

fn str_arg<'a>(args: &'a [Value], method: &str) -> Result<&'a str, Error> {
    args.first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::new(ErrorKind::MissingArgument, format!("{method} requires a string arg")))
}
```

**Step 4: Run tests**

```
cargo test -p undone-scene template_ctx
```

Expected: all pass.

### Part B: SceneEngine

**Step 5: Write failing engine tests**

Create `crates/undone-scene/src/engine.rs` with tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;
    use slotmap::SlotMap;
    use undone_domain::*;
    use undone_world::{GameData, World};
    use undone_packs::PackRegistry;
    use undone_expr::parser::parse;
    use crate::types::{Action, EffectDef, NextBranch, NpcAction, SceneDefinition};

    fn make_world() -> World {
        World {
            player: Player {
                name: "Eva".into(), age: Age::LateTeen, race: "east_asian".into(),
                figure: PlayerFigure::Slim, breasts: BreastSize::Large,
                eye_colour: "brown".into(), hair_colour: "dark".into(),
                traits: HashSet::new(), skills: HashMap::new(),
                money: 100, stress: 0, anxiety: 0,
                arousal: ArousalLevel::Comfort, alcohol: AlcoholLevel::Sober,
                partner: None, friends: vec![],
                virgin: true, anal_virgin: true, lesbian_virgin: true,
                on_pill: false, pregnancy: None, stuff: HashSet::new(),
                custom_flags: HashMap::new(), custom_ints: HashMap::new(),
                always_female: false, femininity: 10,
            },
            male_npcs: SlotMap::with_key(),
            female_npcs: SlotMap::with_key(),
            game_data: GameData::default(),
        }
    }

    fn simple_scene() -> SceneDefinition {
        SceneDefinition {
            id: "test::simple".into(),
            pack: "test".into(),
            intro_prose: "It begins.".into(),
            actions: vec![
                Action {
                    id: "wait".into(), label: "Wait".into(), detail: "".into(),
                    condition: None, prose: "You wait.".into(),
                    allow_npc_actions: false, effects: vec![],
                    next: vec![],  // loop
                },
                Action {
                    id: "leave".into(), label: "Leave".into(), detail: "".into(),
                    condition: None, prose: "You leave.".into(),
                    allow_npc_actions: false,
                    effects: vec![EffectDef::ChangeStress { amount: 1 }],
                    next: vec![NextBranch { condition: None, goto: None, finish: true }],
                },
            ],
            npc_actions: vec![],
        }
    }

    fn engine_with_scene(scene: SceneDefinition) -> SceneEngine {
        let mut scenes = HashMap::new();
        scenes.insert(scene.id.clone(), Arc::new(scene));
        SceneEngine::new(scenes)
    }

    #[test]
    fn start_scene_emits_prose_and_actions() {
        let mut engine = engine_with_scene(simple_scene());
        let mut world = make_world();
        let reg = PackRegistry::new();
        engine.send(EngineCommand::StartScene("test::simple".into()), &mut world, &reg);
        let events = engine.drain();
        assert!(events.iter().any(|e| matches!(e, EngineEvent::ProseAdded(_))));
        let actions_event = events.iter().find_map(|e| {
            if let EngineEvent::ActionsAvailable(a) = e { Some(a) } else { None }
        });
        assert!(actions_event.is_some());
        let ids: Vec<&str> = actions_event.unwrap().iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"wait"));
        assert!(ids.contains(&"leave"));
    }

    #[test]
    fn choose_action_with_finish_emits_scene_finished() {
        let mut engine = engine_with_scene(simple_scene());
        let mut world = make_world();
        let reg = PackRegistry::new();
        engine.send(EngineCommand::StartScene("test::simple".into()), &mut world, &reg);
        engine.drain();
        engine.send(EngineCommand::ChooseAction("leave".into()), &mut world, &reg);
        let events = engine.drain();
        assert!(events.iter().any(|e| matches!(e, EngineEvent::SceneFinished)));
    }

    #[test]
    fn choose_action_applies_effects() {
        let mut engine = engine_with_scene(simple_scene());
        let mut world = make_world();
        let reg = PackRegistry::new();
        engine.send(EngineCommand::StartScene("test::simple".into()), &mut world, &reg);
        engine.drain();
        assert_eq!(world.player.stress, 0);
        engine.send(EngineCommand::ChooseAction("leave".into()), &mut world, &reg);
        engine.drain();
        assert_eq!(world.player.stress, 1);
    }

    #[test]
    fn choose_loop_action_re_emits_actions_available() {
        let mut engine = engine_with_scene(simple_scene());
        let mut world = make_world();
        let reg = PackRegistry::new();
        engine.send(EngineCommand::StartScene("test::simple".into()), &mut world, &reg);
        engine.drain();
        engine.send(EngineCommand::ChooseAction("wait".into()), &mut world, &reg);
        let events = engine.drain();
        assert!(events.iter().any(|e| matches!(e, EngineEvent::ActionsAvailable(_))));
        assert!(!events.iter().any(|e| matches!(e, EngineEvent::SceneFinished)));
    }

    #[test]
    fn condition_filters_actions() {
        let mut scenes = HashMap::new();
        let scene = SceneDefinition {
            id: "test::cond".into(),
            pack: "test".into(),
            intro_prose: "".into(),
            actions: vec![
                Action {
                    id: "always".into(), label: "Always".into(), detail: "".into(),
                    condition: None, prose: "".into(),
                    allow_npc_actions: false, effects: vec[], next: vec![],
                },
                Action {
                    id: "never".into(), label: "Never".into(), detail: "".into(),
                    condition: Some(parse("false").unwrap()), prose: "".into(),
                    allow_npc_actions: false, effects: vec![], next: vec![],
                },
            ],
            npc_actions: vec![],
        };
        scenes.insert(scene.id.clone(), Arc::new(scene));
        let mut engine = SceneEngine::new(scenes);
        let mut world = make_world();
        let reg = PackRegistry::new();
        engine.send(EngineCommand::StartScene("test::cond".into()), &mut world, &reg);
        let events = engine.drain();
        let actions = events.iter().find_map(|e| {
            if let EngineEvent::ActionsAvailable(a) = e { Some(a) } else { None }
        }).unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].id, "always");
    }
}
```

Run: `cargo test -p undone-scene engine`
Expected: compile error.

**Step 6: Implement SceneEngine**

```rust
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use rand::{rngs::SmallRng, Rng, SeedableRng};
use undone_expr::{
    eval::{eval, SceneCtx},
    parser::Expr,
};
use undone_packs::PackRegistry;
use undone_world::World;

use crate::{
    effects::apply_effect,
    template_ctx::render_prose,
    types::{EffectDef, NpcAction, SceneDefinition},
};

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

pub enum EngineCommand {
    StartScene(String),
    ChooseAction(String),
}

pub enum EngineEvent {
    ProseAdded(String),
    ActionsAvailable(Vec<ActionView>),
    SceneFinished,
}

pub struct ActionView {
    pub id: String,
    pub label: String,
    pub detail: String,
}

impl SceneEngine {
    pub fn new(scenes: HashMap<String, Arc<SceneDefinition>>) -> Self {
        Self {
            scenes,
            stack: Vec::new(),
            events: VecDeque::new(),
            rng: SmallRng::from_entropy(),
        }
    }

    pub fn send(&mut self, cmd: EngineCommand, world: &mut World, registry: &PackRegistry) {
        match cmd {
            EngineCommand::StartScene(id) => self.start_scene(&id, world, registry),
            EngineCommand::ChooseAction(id) => self.choose_action(&id, world, registry),
        }
    }

    pub fn drain(&mut self) -> Vec<EngineEvent> {
        self.events.drain(..).collect()
    }

    fn start_scene(&mut self, scene_id: &str, world: &mut World, registry: &PackRegistry) {
        let def = match self.scenes.get(scene_id) {
            Some(d) => Arc::clone(d),
            None => {
                // Unknown scene — emit nothing (bug in caller)
                return;
            }
        };

        // Render intro prose
        let prose = render_prose(&def.intro_prose, world, &SceneCtx::new(), registry)
            .unwrap_or_else(|e| format!("[template error: {e}]"));
        if !prose.trim().is_empty() {
            self.events.push_back(EngineEvent::ProseAdded(prose));
        }

        let frame = SceneFrame { def, ctx: SceneCtx::new() };
        self.stack.push(frame);
        self.emit_actions(world, registry);
    }

    fn choose_action(&mut self, action_id: &str, world: &mut World, registry: &PackRegistry) {
        let frame = match self.stack.last_mut() {
            Some(f) => f,
            None => return,
        };

        let action = match frame.def.actions.iter().find(|a| a.id == action_id) {
            Some(a) => a,
            None => return,
        };

        // Render action prose
        if !action.prose.trim().is_empty() {
            let prose = render_prose(&action.prose, world, &frame.ctx, registry)
                .unwrap_or_else(|e| format!("[template error: {e}]"));
            self.events.push_back(EngineEvent::ProseAdded(prose));
        }

        // Apply action effects
        for effect in &action.effects {
            let _ = apply_effect(effect, world, &mut frame.ctx, registry);
        }

        let allow_npc = action.allow_npc_actions;
        let next_branches = action.next.clone();

        // NPC actions (if allowed)
        if allow_npc {
            self.run_npc_actions(world, registry);
        }

        // Evaluate next routing
        let frame = self.stack.last_mut().unwrap();
        let mut finish = false;

        if next_branches.is_empty() {
            // Loop: re-show actions
        } else {
            for branch in &next_branches {
                let passes = match &branch.condition {
                    None => true,
                    Some(expr) => eval(expr, world, &frame.ctx, registry).unwrap_or(false),
                };
                if passes {
                    if branch.finish {
                        finish = true;
                    }
                    // goto: just loop back (scene stays active, conditions re-evaluated)
                    break;
                }
            }
        }

        if finish {
            self.stack.pop();
            if self.stack.is_empty() {
                self.events.push_back(EngineEvent::SceneFinished);
            } else {
                self.emit_actions(world, registry);
            }
        } else {
            self.emit_actions(world, registry);
        }
    }

    fn run_npc_actions(&mut self, world: &mut World, registry: &PackRegistry) {
        let frame = match self.stack.last_mut() {
            Some(f) => f,
            None => return,
        };

        // Collect eligible NPC actions and their weights
        let eligible: Vec<(&NpcAction, u32)> = frame.def.npc_actions.iter()
            .filter_map(|na| {
                let passes = match &na.condition {
                    None => true,
                    Some(expr) => eval(expr, world, &frame.ctx, registry).unwrap_or(false),
                };
                if passes { Some((na, na.weight)) } else { None }
            })
            .collect();

        if eligible.is_empty() {
            return;
        }

        let total: u32 = eligible.iter().map(|(_, w)| w).sum();
        let pick = self.rng.gen_range(0..total);
        let mut cumulative = 0u32;
        for (na, w) in &eligible {
            cumulative += w;
            if pick < cumulative {
                // Fire this NPC action
                if !na.prose.trim().is_empty() {
                    let prose = render_prose(&na.prose, world, &frame.ctx, registry)
                        .unwrap_or_else(|e| format!("[template error: {e}]"));
                    self.events.push_back(EngineEvent::ProseAdded(prose));
                }
                for effect in &na.effects {
                    let _ = apply_effect(effect, world, &mut frame.ctx, registry);
                }
                break;
            }
        }
    }

    fn emit_actions(&mut self, world: &World, registry: &PackRegistry) {
        let frame = match self.stack.last() {
            Some(f) => f,
            None => return,
        };

        let visible: Vec<ActionView> = frame.def.actions.iter()
            .filter(|a| {
                match &a.condition {
                    None => true,
                    Some(expr) => eval(expr, world, &frame.ctx, registry).unwrap_or(false),
                }
            })
            .map(|a| ActionView {
                id: a.id.clone(),
                label: a.label.clone(),
                detail: a.detail.clone(),
            })
            .collect();

        self.events.push_back(EngineEvent::ActionsAvailable(visible));
    }
}
```

**Note on `next_branches` borrow:** The `action.next.clone()` is needed because `frame.def` is borrowed through `frame`, and we need to mutate `frame.ctx` when applying effects. Clone `next: Vec<NextBranch>` to avoid the borrow conflict. `NextBranch` needs `#[derive(Clone)]` — add that to `types.rs`.

**Step 7: Add modules to lib.rs**

```rust
pub mod effects;
pub mod engine;
pub mod loader;
pub mod template_ctx;
pub mod types;

pub use effects::{apply_effect, EffectError};
pub use engine::{ActionView, EngineCommand, EngineEvent, SceneEngine};
pub use loader::{load_scenes, SceneLoadError};
pub use types::{Action, EffectDef, NpcAction, NextBranch, SceneDefinition};
```

**Step 8: Run diagnostics + format on engine.rs and template_ctx.rs, then run tests**

```
cargo test -p undone-scene
```

Expected: all tests pass (types, effects, template, engine — ~15 tests).

**Step 9: Commit**

```
git add crates/undone-scene/
git commit -m "feat(scene): SceneEngine — event queue, prose rendering, NPC actions, condition filtering"
```

---

## Task 8: Rain Shelter Scene File

**Files:**
- Create: `packs/base/scenes/rain_shelter.toml`

This is the content file used by the integration test. Write prose that:
- Branches on `w.hasTrait("SHY")` so the integration test can verify prose branching.
- Has a "wait" action with `allow_npc_actions = true`.
- Has an NPC action that sets the `umbrella_offered` flag.
- Has an "accept umbrella" action gated on that flag.

**Step 1: Write the scene file**

```toml
[scene]
id          = "base::rain_shelter"
pack        = "base"
description = "Caught in the rain at a bus shelter."

[intro]
prose = """
The rain started ten minutes from home, a proper downpour that has soaked through your jacket.
{% if w.hasTrait("SHY") %}
You find a spot at the far end of the shelter and stare at the pavement, shoulders hunched.
{% else %}
You step into the shelter with a quick nod at the man already there. He nods back.
{% endif %}
"""

[[actions]]
id                = "main"
label             = "Wait it out"
detail            = "Stand here until it eases off."
allow_npc_actions = true

[[actions]]
id        = "leave"
label     = "Make a run for it"
detail    = "Get soaked. At least you'll be moving."
condition = "!scene.hasFlag('umbrella_offered')"
prose     = "You pull your jacket over your head and step back out into it."

  [[actions.effects]]
  type   = "change_stress"
  amount = 2

  [[actions.next]]
  finish = true

[[actions]]
id        = "accept_umbrella"
label     = "Share his umbrella"
detail    = "Step closer. It's dry under there."
condition = "scene.hasFlag('umbrella_offered')"
prose     = """
{% if w.hasTrait("SHY") %}
"Thanks," you manage, stepping under with as little eye contact as possible.
{% else %}
"Cheers," you say, stepping in. He smells of coffee and damp wool.
{% endif %}
"""

  [[actions.effects]]
  type  = "add_npc_liking"
  npc   = "m"
  delta = 1

  [[actions.next]]
  finish = true

[[npc_actions]]
id        = "offers_umbrella"
condition = "!scene.hasFlag('umbrella_offered')"
prose     = """
The man clears his throat. "Want to share?" He tilts his umbrella slightly in your direction.
"""
weight = 10

  [[npc_actions.effects]]
  type = "set_scene_flag"
  flag = "umbrella_offered"
```

**Step 2: Validate the TOML with mcp__minijinja**

For the intro prose template, run `mcp__minijinja__jinja_validate_template` on the intro template string to confirm it's valid Jinja2.

**Step 3: Run scene loader test (should now pass)**

```
cargo test -p undone-scene loader::tests::loads_rain_shelter_scene
```

Expected: PASS.

**Step 4: Commit**

```
git add packs/base/scenes/rain_shelter.toml
git commit -m "content: add rain shelter scene — bus stop encounter with NPC umbrella offer"
```

---

## Task 9: Integration Test

**Files:**
- Modify: `crates/undone-scene/src/lib.rs` — add integration test module at bottom

**Step 1: Write the integration test**

```rust
#[cfg(test)]
mod integration_tests {
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    use slotmap::SlotMap;
    use undone_domain::*;
    use undone_world::{GameData, World};
    use undone_packs::load_packs;

    use super::engine::{EngineCommand, EngineEvent, SceneEngine};
    use super::loader::load_scenes;

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()
            .parent().unwrap()
            .join("packs")
    }

    fn make_world_with_shy(registry: &undone_packs::PackRegistry) -> World {
        let shy_id = registry.resolve_trait("SHY").unwrap();
        World {
            player: Player {
                name: "Eva".into(), age: Age::LateTeen, race: "east_asian".into(),
                figure: PlayerFigure::Slim, breasts: BreastSize::Large,
                eye_colour: "brown".into(), hair_colour: "dark".into(),
                traits: {
                    let mut s = HashSet::new();
                    s.insert(shy_id);
                    s
                },
                skills: HashMap::new(),
                money: 100, stress: 0, anxiety: 0,
                arousal: ArousalLevel::Comfort, alcohol: AlcoholLevel::Sober,
                partner: None, friends: vec![],
                virgin: true, anal_virgin: true, lesbian_virgin: true,
                on_pill: false, pregnancy: None, stuff: HashSet::new(),
                custom_flags: HashMap::new(), custom_ints: HashMap::new(),
                always_female: false, femininity: 10,
            },
            male_npcs: SlotMap::with_key(),
            female_npcs: SlotMap::with_key(),
            game_data: GameData::default(),
        }
    }

    fn make_male_npc() -> MaleNpc {
        MaleNpc {
            core: NpcCore {
                name: "Stranger".into(), age: Age::Thirties, race: "white".into(),
                eye_colour: "grey".into(), hair_colour: "brown".into(),
                personality: PersonalityId(lasso::Spur::try_from_usize(0).unwrap()),
                traits: HashSet::new(),
                relationship: RelationshipStatus::Stranger,
                pc_liking: LikingLevel::Neutral, npc_liking: LikingLevel::Neutral,
                pc_love: LoveLevel::None, npc_love: LoveLevel::None,
                pc_attraction: AttractionLevel::Unattracted,
                npc_attraction: AttractionLevel::Unattracted,
                behaviour: Behaviour::Neutral,
                relationship_flags: HashSet::new(), sexual_activities: HashSet::new(),
                custom_flags: HashMap::new(), custom_ints: HashMap::new(),
                knowledge: 0, contactable: true,
                arousal: ArousalLevel::Comfort, alcohol: AlcoholLevel::Sober,
            },
            figure: MaleFigure::Average,
            clothing: MaleClothing::default(),
            had_orgasm: false, has_baby_with_pc: false,
        }
    }

    #[test]
    fn rain_shelter_full_flow() {
        // 1. Load packs
        let (registry, metas) = load_packs(&packs_dir()).unwrap();
        assert!(!metas.is_empty());

        // 2. Load scenes
        let scenes_dir = packs_dir().join("base").join("scenes");
        let scenes = load_scenes(&scenes_dir, "base", &registry).unwrap();
        assert!(scenes.contains_key("base::rain_shelter"));

        // 3. Create world with SHY player + active male NPC
        let mut world = make_world_with_shy(&registry);
        let npc_key = world.male_npcs.insert(make_male_npc());
        // Wire the active NPC into the scene context via SceneEngine's internal ctx
        // (The engine creates a fresh SceneCtx; we need to set active_male on it.
        //  For this test we'll use a workaround: inject via EngineCommand after start.)
        // NOTE: The engine currently has no command to set active NPC.
        // For the integration test, we manually set it after the engine creates the frame.
        // This reveals a missing API — see note below.

        // 4. Build engine
        let mut engine = SceneEngine::new(scenes);

        // 5. Start scene
        engine.send(EngineCommand::StartScene("base::rain_shelter".into()), &mut world, &registry);
        let events = engine.drain();

        // 6. Assert intro prose contains shy branch
        let prose_events: Vec<&str> = events.iter().filter_map(|e| {
            if let EngineEvent::ProseAdded(p) = e { Some(p.as_str()) } else { None }
        }).collect();
        assert!(!prose_events.is_empty(), "intro prose should be emitted");
        let all_prose = prose_events.join("\n");
        assert!(all_prose.contains("far end"), "SHY branch should appear in intro");

        // 7. Assert initial actions (main + leave, NOT accept_umbrella yet)
        let actions_event = events.iter().find_map(|e| {
            if let EngineEvent::ActionsAvailable(a) = e { Some(a) } else { None }
        }).unwrap();
        let ids: Vec<&str> = actions_event.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"main"), "main should be available");
        assert!(ids.contains(&"leave"), "leave should be available");
        assert!(!ids.contains(&"accept_umbrella"), "accept_umbrella not available yet");

        // NOTE: For the NPC umbrella offer to fire, we need active_male set in the frame's ctx.
        // The SceneEngine needs a SetActiveNpc command or similar. Add that in the next step.
    }

    #[test]
    fn rain_shelter_npc_fires_and_umbrella_becomes_available() {
        // This test requires SceneEngine::set_active_male(key) or similar.
        // Implement after adding that API to SceneEngine.
        // Placeholder: see Task 9 Step 2.
        todo!("Requires active NPC wiring — add EngineCommand::SetActiveMale");
    }
}
```

The integration test reveals that `SceneEngine` needs a way to set the active NPC for the scene context. This is a gap in the API that we catch here (that's TDD working correctly).

**Step 2: Add `SetActiveMale` command to SceneEngine**

In `engine.rs`, add to `EngineCommand`:

```rust
pub enum EngineCommand {
    StartScene(String),
    ChooseAction(String),
    SetActiveMale(undone_domain::MaleNpcKey),   // wire the active male NPC
    SetActiveFemale(undone_domain::FemaleNpcKey),
}
```

Handle in `send`:

```rust
EngineCommand::SetActiveMale(key) => {
    if let Some(frame) = self.stack.last_mut() {
        frame.ctx.active_male = Some(key);
    }
}
EngineCommand::SetActiveFemale(key) => {
    if let Some(frame) = self.stack.last_mut() {
        frame.ctx.active_female = Some(key);
    }
}
```

**Step 3: Complete the integration test**

Now wire the full flow:

```rust
#[test]
fn rain_shelter_npc_fires_and_umbrella_becomes_available() {
    let (registry, _) = load_packs(&packs_dir()).unwrap();
    let scenes_dir = packs_dir().join("base").join("scenes");
    let scenes = load_scenes(&scenes_dir, "base", &registry).unwrap();

    let mut world = make_world_with_shy(&registry);
    let npc_key = world.male_npcs.insert(make_male_npc());
    assert_eq!(world.male_npcs[npc_key].core.pc_liking, LikingLevel::Neutral);

    let mut engine = SceneEngine::new(scenes);

    // Start scene + wire NPC
    engine.send(EngineCommand::StartScene("base::rain_shelter".into()), &mut world, &registry);
    engine.send(EngineCommand::SetActiveMale(npc_key), &mut world, &registry);
    engine.drain();

    // Pick "main" (allow_npc_actions = true) — NPC should fire and set umbrella_offered
    // Run a few times until NPC fires (it's weighted random, weight=10 with no competitors
    // so it will fire every time when eligible)
    engine.send(EngineCommand::ChooseAction("main".into()), &mut world, &registry);
    let events = engine.drain();

    // NPC prose should appear
    let prose_events: Vec<&str> = events.iter().filter_map(|e| {
        if let EngineEvent::ProseAdded(p) = e { Some(p.as_str()) } else { None }
    }).collect();
    // NPC fired — umbrella_offered is now a scene flag
    // accept_umbrella should now be visible
    let actions = events.iter().find_map(|e| {
        if let EngineEvent::ActionsAvailable(a) = e { Some(a) } else { None }
    }).unwrap();
    let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&"accept_umbrella"), "accept_umbrella should be visible after NPC fires");

    // Accept umbrella — finish scene
    engine.send(EngineCommand::ChooseAction("accept_umbrella".into()), &mut world, &registry);
    let events = engine.drain();
    assert!(events.iter().any(|e| matches!(e, EngineEvent::SceneFinished)));

    // NPC liking should have increased by 1 step (Neutral → Ok)
    assert_eq!(world.male_npcs[npc_key].core.pc_liking, LikingLevel::Ok);
}
```

**Step 4: Run all tests**

```
cargo test -p undone-scene
```

Expected: all tests pass, including integration tests.

**Step 5: Run full workspace test suite**

```
cargo test
```

Expected: 30 existing tests + new tests all pass, zero warnings.

**Step 6: Run clippy**

```
cargo clippy -- -D warnings
```

Expected: clean.

**Step 7: Final commit**

```
git add crates/undone-scene/ packs/base/scenes/
git commit -m "feat(scene): integration test — full rain shelter flow, SetActiveMale command"
```

---

## Task 10: Update HANDOFF.md

**Files:**
- Modify: `HANDOFF.md`

Update Current State, Next Action, and Session Log. Add timestamped entry.

```markdown
## Current State

**Phase:** Scene engine complete.

Pack disk loader, expression evaluator wired to registry, typed effect system,
minijinja prose rendering, and SceneEngine with event queue all implemented and tested.
Rain shelter scene demonstrates full end-to-end flow.

All tests pass, clippy clean.

## Next Action

Design and implement the **Scheduler** — weekly timeslots, weighted scene selection,
pack-contributed event pools.

## Session Log

| Date | Summary |
|---|---|
| 2026-02-21 | Design session: decompiled Newlife, designed Undone engine, wrote scaffold plan |
| 2026-02-21 | Tooling session: built rhai-mcp-server + minijinja-mcp-server, wired MCP + hooks |
| 2026-02-22 | Scaffold session: Tasks 1–3 complete. MCP confirmed working. |
| 2026-02-22 | Scaffold session: Tasks 4–13 complete. 30 tests pass. Scaffold done. |
| 2026-02-22 | Scene engine: brainstorm + design. Flat pool model, event queue API. |
| 2026-02-22 | Scene engine: implementation. Pack loader, eval wiring, SceneEngine, rain shelter scene. |
```

Commit:
```
git add HANDOFF.md
git commit -m "docs: update HANDOFF — scene engine complete"
```

---

## Summary of All Files Changed

| File | Action |
|---|---|
| `Cargo.toml` | Add `rand` to workspace deps |
| `crates/undone-expr/Cargo.toml` | Add `undone-packs` dep |
| `crates/undone-scene/Cargo.toml` | Add `rand` dep |
| `crates/undone-packs/src/registry.rs` | Add `get_stat`, `trait_id_to_str` |
| `crates/undone-packs/src/loader.rs` | **Create**: disk loader |
| `crates/undone-packs/src/lib.rs` | Export loader |
| `crates/undone-expr/src/eval.rs` | Add registry param, wire stubs |
| `crates/undone-scene/src/lib.rs` | Replace placeholder, add modules |
| `crates/undone-scene/src/types.rs` | **Create**: TOML deserialization types + resolved types |
| `crates/undone-scene/src/effects.rs` | **Create**: apply_effect |
| `crates/undone-scene/src/template_ctx.rs` | **Create**: minijinja Object impls |
| `crates/undone-scene/src/loader.rs` | **Create**: scene file loader |
| `crates/undone-scene/src/engine.rs` | **Create**: SceneEngine |
| `packs/base/scenes/rain_shelter.toml` | **Create**: rain shelter content |
| `HANDOFF.md` | Update state + log |
