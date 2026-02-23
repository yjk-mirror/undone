# Engineering Hardening (Batch 2) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix four correctness and robustness issues: unify the FEMININITY dual representation, wire `hasStuff()` to the player's inventory, add a stats registration system, and eliminate panics in error-recovery paths.

**Architecture:** All changes are localized to existing crates — no new crates or public API additions beyond `Scheduler::empty()` and `PackRegistry::resolve_stuff()`. The FEMININITY fix removes a field from `Player` and reads from the skills map instead. The stuff fix adds a `resolve_stuff` method to the registry using the existing `StuffId` type. The stats fix adds an optional `stats_file` to the manifest + a data file. The panic fix adds an `empty()` constructor to `Scheduler`.

**Tech Stack:** Rust, serde/toml, lasso string interning.

---

### Task 1: Unify FEMININITY — remove `Player.femininity` field, read from skills map

The `Player` struct has a standalone `femininity: i32` field, but FEMININITY is also defined as a skill in `skills.toml`. Scenes that use `EffectDef::SkillIncrease { skill: "FEMININITY" }` update `player.skills[FEMININITY]`, but `active_name()` reads `player.femininity`. These are never synchronized. Fix: remove the field, make `active_name()` read from the skills map, and update all construction sites.

**Files:**
- Modify: `crates/undone-domain/src/player.rs` — remove `femininity` field, change `active_name()` to take a skill lookup
- Modify: `crates/undone-packs/src/char_creation.rs` — set starting FEMININITY via skills map instead of field
- Modify: `crates/undone-ui/src/game_state.rs` — remove `femininity` from `placeholder_player()`
- Modify: all test `make_player()` / `make_world()` helpers that set `femininity` — remove the field
- Test: existing `active_name_picks_correct_variant` test must be updated

**Design decision:** `active_name()` currently takes `&self`. After removing the field, it needs the `SkillId` for FEMININITY to look up the value. Two options:
- (A) `active_name()` takes a `SkillId` parameter — caller must resolve it. Clean but changes API.
- (B) Add a `femininity()` method that scans skills by convention. Fragile.

**Go with (A).** Change signature to `pub fn active_name(&self, femininity_skill: SkillId) -> &str`. The caller (UI code, template rendering) already has access to the registry and can resolve it once.

**Step 1: Write failing test in `player.rs`**

Add a test that calls `active_name(skill_id)` with the new signature. It will fail to compile because the field still exists and the method signature hasn't changed.

```rust
// In crates/undone-domain/src/player.rs tests
#[test]
fn active_name_reads_from_skills_map() {
    use crate::SkillId;
    use lasso::{Key, Spur};

    let fem_skill = SkillId(Spur::try_from_usize(0).unwrap());
    let mut p = make_player();
    // Set FEMININITY via skills map, not a field
    p.skills.insert(fem_skill, SkillValue { value: 10, modifier: 0 });
    assert_eq!(p.active_name(fem_skill), "Evan");

    p.skills.insert(fem_skill, SkillValue { value: 50, modifier: 0 });
    assert_eq!(p.active_name(fem_skill), "Ev");

    p.skills.insert(fem_skill, SkillValue { value: 80, modifier: 0 });
    assert_eq!(p.active_name(fem_skill), "Eva");
}
```

**Step 2: Remove `femininity` field, update `active_name` signature**

In `crates/undone-domain/src/player.rs`:
- Remove `pub femininity: i32` from the struct
- Change `active_name`:
```rust
pub fn active_name(&self, femininity_skill: SkillId) -> &str {
    let fem = self.skill(femininity_skill);
    if fem >= 70 {
        &self.name_fem
    } else if fem >= 31 {
        &self.name_androg
    } else {
        &self.name_masc
    }
}
```
- Update existing `active_name_picks_correct_variant` test to use the new signature
- Remove `femininity` from `make_player()` in tests

**Step 3: Fix `char_creation.rs` — set starting femininity via skills**

In `new_game()`, after creating the player, insert the FEMININITY skill:
```rust
let femininity_skill = registry.resolve_skill("FEMININITY")
    .expect("FEMININITY skill must be registered by base pack");
let starting_femininity = if config.always_female { 75 } else { 10 };
player.skills.insert(femininity_skill, SkillValue { value: starting_femininity, modifier: 0 });
```
Remove `femininity: starting_femininity` from the Player literal.

**Step 4: Fix all other construction sites**

Remove `femininity: <value>` from:
- `crates/undone-ui/src/game_state.rs` — `placeholder_player()`
- `crates/undone-expr/src/eval.rs` — `make_world()` in tests
- `crates/undone-scene/src/effects.rs` — `make_world()` in tests
- `crates/undone-scene/src/scheduler.rs` — `make_world()` in tests
- `crates/undone-save/src/lib.rs` — check if save/load tests build a Player

Grep for `femininity:` across all `.rs` files to find every site.

**Step 5: Fix UI call site**

Find where `active_name()` is called in the UI and pass the resolved FEMININITY `SkillId`. The `GameState` has a `registry` — resolve once at game init and store the `SkillId`.

**Step 6: Run `cargo test` — all 95+ tests pass**

**Step 7: Commit**

```
git add -A && git commit -m "fix: unify FEMININITY — remove Player.femininity field, read from skills map"
```

---

### Task 2: Wire `w.hasStuff()` to player inventory

`eval.rs:192-194` validates the argument but always returns `false`. The `Player` already has `stuff: HashSet<StuffId>` and `StuffId` is a newtype over `Spur`. We just need `PackRegistry::resolve_stuff()` to intern/resolve stuff names.

**Files:**
- Modify: `crates/undone-packs/src/registry.rs` — add `intern_stuff()` and `resolve_stuff()` methods
- Modify: `crates/undone-expr/src/eval.rs` — wire `hasStuff` to registry + player.stuff
- Test: `crates/undone-expr/src/eval.rs` — add test for `hasStuff`

**Step 1: Write failing test**

```rust
#[test]
fn hasStuff_true_when_player_has_item() {
    let mut reg = PackRegistry::new();
    let stuff_id = reg.intern_stuff("UMBRELLA");
    let mut world = make_world();
    world.player.stuff.insert(stuff_id);
    let ctx = SceneCtx::new();
    let expr = parse("w.hasStuff('UMBRELLA')").unwrap();
    assert!(eval(&expr, &world, &ctx, &reg).unwrap());
}

#[test]
fn hasStuff_false_when_player_lacks_item() {
    let mut reg = PackRegistry::new();
    reg.intern_stuff("UMBRELLA"); // registered but player doesn't have it
    let world = make_world();
    let ctx = SceneCtx::new();
    let expr = parse("w.hasStuff('UMBRELLA')").unwrap();
    assert!(!eval(&expr, &world, &ctx, &reg).unwrap());
}
```

**Step 2: Add `intern_stuff()` and `resolve_stuff()` to `PackRegistry`**

```rust
/// Intern a stuff/item name, returning a StuffId.
pub fn intern_stuff(&mut self, id: &str) -> StuffId {
    StuffId(self.intern(id))
}

/// Look up an already-interned stuff name. Returns None if never interned.
pub fn resolve_stuff(&self, id: &str) -> Option<StuffId> {
    self.rodeo.get(id).map(StuffId)
}
```

Add `use undone_domain::StuffId;` to imports if not already present.

**Step 3: Wire eval.rs `hasStuff` branch**

Replace:
```rust
"hasStuff" => {
    let _ = str_arg(0)?; // validate arg
    Ok(false) // TODO: wire to StuffId when stuff registry exists
}
```
With:
```rust
"hasStuff" => {
    let id = str_arg(0)?;
    match registry.resolve_stuff(id) {
        Some(stuff_id) => Ok(world.player.stuff.contains(&stuff_id)),
        None => Ok(false), // never interned = player can't have it
    }
}
```

**Step 4: Run tests — all pass**

**Step 5: Commit**

```
git add -A && git commit -m "fix: wire w.hasStuff() to player inventory via StuffId registry"
```

---

### Task 3: Add stats registration to pack system

`AddStat`/`SetStat`/`getStat()` silently no-op because stats are only interned lazily (via `intern_stat()` in effects code), but effects use `get_stat()` which returns `None` for never-interned names. The stat system needs pack-level definitions so stat names are interned at load time.

**Files:**
- Create: `packs/base/data/stats.toml` — define game stats
- Modify: `packs/base/pack.toml` — add `stats_file` key
- Modify: `crates/undone-packs/src/manifest.rs` — add optional `stats_file` to `PackContent`
- Modify: `crates/undone-packs/src/data.rs` — add `StatDef` and `StatFile` structs
- Modify: `crates/undone-packs/src/registry.rs` — add `register_stats()` method
- Modify: `crates/undone-packs/src/loader.rs` — load stats file when present
- Modify: `crates/undone-scene/src/effects.rs` — `AddStat`/`SetStat` should use `resolve_stat()` not `get_stat()` (fail on unknown stat, don't silently ignore)
- Test: `crates/undone-packs/src/loader.rs` — add test for stats loading

**Step 1: Create `packs/base/data/stats.toml`**

```toml
[[stat]]
id          = "TIMES_KISSED"
name        = "Times Kissed"
description = "Number of times the player has been kissed."

[[stat]]
id          = "DATES_ATTENDED"
name        = "Dates Attended"
description = "Number of dates the player has gone on."

[[stat]]
id          = "WEEKS_WORKED"
name        = "Weeks Worked"
description = "Number of weeks the player has worked."
```

**Step 2: Add `stats_file` to manifest**

In `crates/undone-packs/src/manifest.rs`, add to `PackContent`:
```rust
#[serde(default)]
pub stats_file: Option<String>,
```

In `packs/base/pack.toml`, add:
```toml
stats_file    = "data/stats.toml"
```

**Step 3: Add `StatDef` / `StatFile` to `data.rs`**

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct StatDef {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct StatFile {
    #[serde(default)]
    pub stat: Vec<StatDef>,
}
```

**Step 4: Add `register_stats()` to `PackRegistry`**

```rust
pub fn register_stats(&mut self, defs: Vec<StatDef>) {
    for def in defs {
        self.intern_stat(&def.id);
        // Optionally store def metadata — for now, interning is enough
    }
}
```

Import `StatDef` from `crate::data`.

**Step 5: Load stats in `loader.rs`**

After the names loading block, add:
```rust
if let Some(ref stats_rel) = manifest.content.stats_file {
    let stats_path = pack_dir.join(stats_rel);
    let src = read_file(&stats_path)?;
    let stats_file: crate::data::StatFile =
        toml::from_str(&src).map_err(|e| PackLoadError::Toml {
            path: stats_path.clone(),
            message: e.to_string(),
        })?;
    registry.register_stats(stats_file.stat);
}
```

**Step 6: Write test for stats loading**

```rust
#[test]
fn loads_base_pack_stats() {
    let (registry, _) = load_packs(&packs_dir()).unwrap();
    assert!(
        registry.get_stat("TIMES_KISSED").is_some(),
        "TIMES_KISSED stat should be interned"
    );
}
```

**Step 7: Run `cargo test` — all pass**

**Step 8: Commit**

```
git add -A && git commit -m "feat: add stats registration to pack system — stats.toml loaded at pack time"
```

---

### Task 4: Eliminate panics in error-recovery paths + harden spawner unwraps

Two issues: (a) `game_state.rs` panics inside the error-recovery branch for schedule loading, and (b) spawner uses bare `.unwrap()` on const-slice `.choose()` calls.

**Files:**
- Modify: `crates/undone-scene/src/scheduler.rs` — add `Scheduler::empty()` constructor
- Modify: `crates/undone-ui/src/game_state.rs` — replace `panic!` / `expect` with `Scheduler::empty()`
- Modify: `crates/undone-packs/src/spawner.rs` — replace `.unwrap()` with `.expect("non-empty const slice")`

**Step 1: Add `Scheduler::empty()`**

In `crates/undone-scene/src/scheduler.rs`:
```rust
impl Scheduler {
    /// Create an empty scheduler with no slots. Used as a fallback when
    /// pack loading fails.
    pub fn empty() -> Self {
        Self {
            slots: HashMap::new(),
        }
    }
    // ... existing methods
}
```

**Step 2: Write test for `Scheduler::empty()`**

```rust
#[test]
fn empty_scheduler_returns_none_for_any_slot() {
    let scheduler = Scheduler::empty();
    let registry = PackRegistry::new();
    let world = make_world();
    let mut rng = SmallRng::seed_from_u64(42);
    assert!(scheduler.pick("anything", &world, &registry, &mut rng).is_none());
}
```

**Step 3: Fix `game_state.rs` — replace panic with `Scheduler::empty()`**

Replace line 58-61:
```rust
scheduler: load_schedule(&[]).unwrap_or_else(|_| {
    panic!("load_schedule on empty slice failed")
}),
```
With:
```rust
scheduler: Scheduler::empty(),
```

Replace line 103:
```rust
load_schedule(&[]).expect("empty schedule should never fail")
```
With:
```rust
Scheduler::empty()
```

Update import to include `Scheduler` from `undone_scene::scheduler`.

**Step 4: Harden spawner `.unwrap()` calls**

In `crates/undone-packs/src/spawner.rs`, replace all bare `.unwrap()` on `.choose(rng)` with `.expect("non-empty const slice")`:

- Line 76: `.choose(rng).unwrap()` → `.choose(rng).expect("CORE_PERSONALITIES is non-empty")`
- Line 97: `*AGES.choose(rng).unwrap()` → `*AGES.choose(rng).expect("AGES is non-empty")`
- Line 98: `RACES.choose(rng).unwrap()` → `RACES.choose(rng).expect("RACES is non-empty")`
- Line 99: `EYE_COLOURS.choose(rng).unwrap()` → `EYE_COLOURS.choose(rng).expect("EYE_COLOURS is non-empty")`
- Line 100: `HAIR_COLOURS.choose(rng).unwrap()` → `HAIR_COLOURS.choose(rng).expect("HAIR_COLOURS is non-empty")`
- Line 102: `*MALE_FIGURES.choose(rng).unwrap()` → `*MALE_FIGURES.choose(rng).expect("MALE_FIGURES is non-empty")`
- Line 127: `*AGES.choose(rng).unwrap()` → `*AGES.choose(rng).expect("AGES is non-empty")`
- Line 128: `RACES.choose(rng).unwrap()` → `RACES.choose(rng).expect("RACES is non-empty")`
- Line 129: `EYE_COLOURS.choose(rng).unwrap()` → `EYE_COLOURS.choose(rng).expect("EYE_COLOURS is non-empty")`
- Line 130: `HAIR_COLOURS.choose(rng).unwrap()` → `HAIR_COLOURS.choose(rng).expect("HAIR_COLOURS is non-empty")`
- Line 133: `*FEMALE_FIGURES.choose(rng).unwrap()` → `*FEMALE_FIGURES.choose(rng).expect("FEMALE_FIGURES is non-empty")`
- Line 134: `*BREAST_SIZES.choose(rng).unwrap()` → `*BREAST_SIZES.choose(rng).expect("BREAST_SIZES is non-empty")`

**Step 5: Run `cargo test` — all pass**

**Step 6: Commit**

```
git add -A && git commit -m "fix: eliminate panics in error-recovery paths, harden spawner unwraps"
```

---

## Execution Notes

- Tasks 2, 3, and 4 are independent and can be parallelized.
- Task 1 (FEMININITY) touches the most files and is the riskiest — do it first or in isolation.
- After all tasks: run `cargo test`, `cargo clippy`, verify 95+ tests pass with 0 warnings.
- Total scope: ~8 files modified, 1 file created, ~95% of changes are mechanical (removing a field, swapping unwrap→expect).
