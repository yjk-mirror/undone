# NPC Spawning + Character Creation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Sexuality/Personality enums, extend Player with the three-name system and before-fields, and build the NPC spawner and new_game() factory that produces a populated World ready for week 1.

**Architecture:** New enums land in undone-domain; name/personality/spawner support lands in undone-packs; a new `char_creation.rs` module in undone-packs exposes `new_game()` which creates a `World` with a built Player and spawned NPC pool. The spawner enforces diversity guarantees (at least 1 Romantic, 1 Jerk, 1 Friend). No UI; no scene integration; no save format change. The save test make_world() helper must be updated after the Player struct changes.

**Tech Stack:** Rust, lasso (interning), slotmap (NPC storage), rand 0.8 (SmallRng), serde (Player serialisation), toml (names file)

---

## Task 1: Add Sexuality and Personality enums to undone-domain

**Files:**
- Modify: `crates/undone-domain/src/enums.rs`

`Sexuality` must be Serialize/Deserialize (it lives on Player).
`Personality` does not need serde (it is computed from a PersonalityId spur, never stored).

**Step 1: Add both enums to enums.rs**

Append after the existing `Age` enum (before the `#[cfg(test)]` block):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sexuality {
    StraightMale,   // was attracted to women; now attracted to men = new territory
    GayMale,        // was attracted to men; now attracted to men = familiar desire, new position
    BiMale,         // was attracted to both
    AlwaysFemale,   // always_female=true; before_sexuality is not applicable
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Personality {
    Romantic,
    Jerk,
    Friend,
    Intellectual,
    Lad,
}
```

**Step 2: Add serde roundtrip tests for both**

In the existing `#[cfg(test)]` block, add:

```rust
#[test]
fn sexuality_serde_roundtrip() {
    let s = Sexuality::StraightMale;
    let json = serde_json::to_string(&s).unwrap();
    let back: Sexuality = serde_json::from_str(&json).unwrap();
    assert_eq!(s, back);
}

#[test]
fn personality_is_eq() {
    assert_eq!(Personality::Romantic, Personality::Romantic);
    assert_ne!(Personality::Jerk, Personality::Friend);
}
```

**Step 3: Run tests to verify they pass**

```bash
cargo test -p undone-domain -- enums
```

Expected: all enums tests pass.

**Step 4: Check diagnostics on enums.rs**

Use `mcp__rust__get_diagnostics` on `crates/undone-domain/src/enums.rs`.

**Step 5: Format**

Use `mcp__rust__format_code` on `crates/undone-domain/src/enums.rs`.

**Step 6: Commit**

```bash
git add crates/undone-domain/src/enums.rs
git commit -m "feat(domain): add Sexuality and Personality enums"
```

---

## Task 2: Extend Player with before-fields and three-name system

**Files:**
- Modify: `crates/undone-domain/src/player.rs`
- Modify: `crates/undone-domain/src/lib.rs` (re-export Sexuality)
- Modify: `crates/undone-save/src/lib.rs` (fix broken make_world test helper)

**Background:** The existing `name: String` field becomes three fields. The `Player` struct gains `name_fem`, `name_androg`, `name_masc`, plus before-fields. `active_name()` computes which name is active from `self.femininity`.

Rules:
- `femininity` 0–30 → `name_masc`
- `femininity` 31–69 → `name_androg`
- `femininity` 70+ → `name_fem`

**Step 1: Update Player struct in player.rs**

Remove:
```rust
    pub name: String,
```

Add after `pub femininity: i32,`:
```rust
    // Three-name system (Lilith's Throne pattern)
    pub name_fem: String,
    pub name_androg: String,
    pub name_masc: String,

    // Before-transformation data
    pub before_age: u32,
    pub before_race: String,
    pub before_sexuality: crate::Sexuality,
```

The import at the top of player.rs already has `use crate::{...}` — add `Sexuality` to that list.

**Step 2: Add active_name() method to the Player impl block**

```rust
/// Returns the currently active display name based on femininity score.
/// 0–30 → masculine name, 31–69 → androgynous name, 70+ → feminine name.
pub fn active_name(&self) -> &str {
    if self.femininity >= 70 {
        &self.name_fem
    } else if self.femininity >= 31 {
        &self.name_androg
    } else {
        &self.name_masc
    }
}
```

**Step 3: Update make_player() test helper in player.rs**

Replace:
```rust
        Player {
            name: "Eva".into(),
```
with:
```rust
        Player {
            name_fem: "Eva".into(),
            name_androg: "Evan".into(),
            name_masc: "Evan".into(),
            before_age: 30,
            before_race: "white".into(),
            before_sexuality: crate::Sexuality::StraightMale,
```

**Step 4: Update lib.rs to re-export Sexuality**

`undone-domain/src/lib.rs` already has `pub use enums::*;` — no change needed (wildcard re-exports Sexuality automatically).

**Step 5: Add active_name tests to player.rs**

```rust
#[test]
fn active_name_picks_correct_variant() {
    let mut p = make_player();
    p.name_masc   = "Evan".into();
    p.name_androg = "Ev".into();
    p.name_fem    = "Eva".into();

    p.femininity = 0;
    assert_eq!(p.active_name(), "Evan");

    p.femininity = 30;
    assert_eq!(p.active_name(), "Evan");

    p.femininity = 31;
    assert_eq!(p.active_name(), "Ev");

    p.femininity = 69;
    assert_eq!(p.active_name(), "Ev");

    p.femininity = 70;
    assert_eq!(p.active_name(), "Eva");

    p.femininity = 100;
    assert_eq!(p.active_name(), "Eva");
}
```

**Step 6: Fix broken make_world() in undone-save/src/lib.rs**

In the `#[cfg(test)]` block, replace the Player literal construction to use the new fields. Replace `name: "Eva".into(),` with:
```rust
                name_fem: "Eva".into(),
                name_androg: "Ev".into(),
                name_masc: "Evan".into(),
                before_age: 30,
                before_race: "white".into(),
                before_sexuality: Sexuality::StraightMale,
```

Also replace the assertion `assert_eq!(loaded.player.name, world.player.name);` with:
```rust
assert_eq!(loaded.player.name_fem, world.player.name_fem);
```

**Step 7: Run all tests**

```bash
cargo test -p undone-domain -p undone-save
```

Expected: all tests pass.

**Step 8: Diagnostics and format**

Use `mcp__rust__get_diagnostics` on both modified files.
Use `mcp__rust__format_code` on both.

**Step 9: Commit**

```bash
git add crates/undone-domain/src/player.rs crates/undone-save/src/lib.rs
git commit -m "feat(domain): three-name system + before-fields on Player"
```

---

## Task 3: Add personality + names support to PackRegistry

**Files:**
- Modify: `crates/undone-packs/src/registry.rs`

Add three new public methods to `PackRegistry`:

1. `intern_personality(id: &str) -> PersonalityId` — `get_or_intern`, returns the ID (no validation against a def list; personalities don't need definitions).
2. `core_personality(id: PersonalityId) -> Option<Personality>` — resolves the spur to a string, matches against the 5 core names.
3. `register_names(male: Vec<String>, female: Vec<String>)` — stores name lists.
4. `male_names(&self) -> &[String]` — returns slice.
5. `female_names(&self) -> &[String]` — returns slice.

**Step 1: Add name storage to PackRegistry struct**

In registry.rs, add to the `PackRegistry` struct:
```rust
    male_names: Vec<String>,
    female_names: Vec<String>,
```

In `PackRegistry::new()`:
```rust
    male_names: Vec::new(),
    female_names: Vec::new(),
```

**Step 2: Add the five methods**

Add inside the `impl PackRegistry` block:

```rust
/// Intern a personality name, returning a PersonalityId.
/// Personalities don't require registered definitions — any string is valid.
pub fn intern_personality(&mut self, id: &str) -> PersonalityId {
    PersonalityId(self.intern(id))
}

/// Resolve a PersonalityId to the engine Personality enum.
/// Returns None for custom/unknown personalities.
pub fn core_personality(&self, id: PersonalityId) -> Option<undone_domain::Personality> {
    use undone_domain::Personality;
    match self.rodeo.resolve(&id.0) {
        "ROMANTIC"     => Some(Personality::Romantic),
        "JERK"         => Some(Personality::Jerk),
        "FRIEND"       => Some(Personality::Friend),
        "INTELLECTUAL" => Some(Personality::Intellectual),
        "LAD"          => Some(Personality::Lad),
        _              => None,
    }
}

/// Store male and female NPC name lists from a pack's names file.
pub fn register_names(&mut self, male: Vec<String>, female: Vec<String>) {
    self.male_names.extend(male);
    self.female_names.extend(female);
}

pub fn male_names(&self) -> &[String] {
    &self.male_names
}

pub fn female_names(&self) -> &[String] {
    &self.female_names
}
```

You'll need to add `PersonalityId` to the import at the top of registry.rs:
```rust
use undone_domain::{NpcTraitId, PersonalityId, SkillId, StatId, TraitId};
```

**Step 3: Write tests**

```rust
#[test]
fn intern_and_resolve_personality() {
    let mut reg = PackRegistry::new();
    let id = reg.intern_personality("ROMANTIC");
    assert_eq!(reg.core_personality(id), Some(undone_domain::Personality::Romantic));
}

#[test]
fn unknown_personality_returns_none() {
    let mut reg = PackRegistry::new();
    let id = reg.intern_personality("CUSTOM_PACK_PERSONALITY");
    assert_eq!(reg.core_personality(id), None);
}

#[test]
fn register_names_accumulates() {
    let mut reg = PackRegistry::new();
    reg.register_names(
        vec!["James".into(), "Thomas".into()],
        vec!["Emma".into()],
    );
    assert_eq!(reg.male_names(), &["James", "Thomas"]);
    assert_eq!(reg.female_names(), &["Emma"]);
}
```

**Step 4: Run tests**

```bash
cargo test -p undone-packs -- registry
```

Expected: all registry tests pass.

**Step 5: Diagnostics and format**

Use `mcp__rust__get_diagnostics` and `mcp__rust__format_code` on `crates/undone-packs/src/registry.rs`.

**Step 6: Commit**

```bash
git add crates/undone-packs/src/registry.rs
git commit -m "feat(packs): personality intern/resolve and name list registration"
```

---

## Task 4: Names data file — TOML + data types + manifest + loader

**Files:**
- Create: `packs/base/data/names.toml`
- Modify: `crates/undone-packs/src/data.rs` (add NamesFile struct)
- Modify: `crates/undone-packs/src/manifest.rs` (add names_file field to PackContent)
- Modify: `crates/undone-packs/src/loader.rs` (load names if present)
- Modify: `packs/base/pack.toml` (point to names file)

**Step 1: Create packs/base/data/names.toml**

```toml
male_names = [
    "James", "Thomas", "William", "Oliver", "Harry", "Charlie", "George",
    "Jack", "Henry", "Edward", "Arthur", "Frederick", "Albert", "Robert",
    "John", "Michael", "David", "Richard", "Peter", "Samuel", "Daniel",
    "Matthew", "Andrew", "Patrick", "Christopher", "Adam", "Benjamin",
    "Joseph", "Luke", "Nathan"
]

female_names = [
    "Emma", "Sophie", "Charlotte", "Grace", "Lily", "Olivia", "Emily",
    "Amelia", "Chloe", "Hannah", "Mia", "Holly", "Jessica", "Isabelle",
    "Lucy", "Ava", "Ellie", "Kate", "Daisy", "Poppy", "Abigail",
    "Victoria", "Natalie", "Claire", "Rachel", "Laura", "Amy",
    "Eleanor", "Zoe", "Jasmine"
]
```

**Step 2: Add NamesFile to data.rs**

Append to `crates/undone-packs/src/data.rs`:

```rust
#[derive(Debug, Deserialize)]
pub struct NamesFile {
    pub male_names: Vec<String>,
    pub female_names: Vec<String>,
}
```

**Step 3: Add names_file to PackContent in manifest.rs**

In the `PackContent` struct, add:
```rust
    #[serde(default)]
    pub names_file: Option<String>,
```

**Step 4: Update loader to load names when present**

In `load_one_pack()` in loader.rs, after the skills loading block, add:

```rust
    if let Some(ref names_rel) = manifest.content.names_file {
        let names_path = pack_dir.join(names_rel);
        let src = read_file(&names_path)?;
        let names_file: crate::data::NamesFile =
            toml::from_str(&src).map_err(|e| PackLoadError::Toml {
                path: names_path.clone(),
                message: e.to_string(),
            })?;
        registry.register_names(names_file.male_names, names_file.female_names);
    }
```

**Step 5: Update packs/base/pack.toml**

Read the current `packs/base/pack.toml` first, then add `names_file = "data/names.toml"` to the `[content]` section.

**Step 6: Update data.rs lib.rs export**

In `crates/undone-packs/src/lib.rs`, add `NamesFile` to the data re-export:
```rust
pub use data::{NamesFile, NpcTraitDef, SkillDef, TraitDef};
```

**Step 7: Write a loader test**

In `loader.rs` tests, add:
```rust
#[test]
fn loads_base_pack_names() {
    let (registry, _) = load_packs(&packs_dir()).unwrap();
    assert!(
        !registry.male_names().is_empty(),
        "should have loaded male names"
    );
    assert!(
        !registry.female_names().is_empty(),
        "should have loaded female names"
    );
    assert!(
        registry.male_names().contains(&"James".to_string()),
        "should include James"
    );
}
```

**Step 8: Run tests**

```bash
cargo test -p undone-packs -- loader
```

Expected: all loader tests pass, including new names test.

**Step 9: Diagnostics and format**

Use `mcp__rust__get_diagnostics` on loader.rs, data.rs, manifest.rs.
Use `mcp__rust__format_code` on each.

**Step 10: Commit**

```bash
git add packs/base/data/names.toml packs/base/pack.toml \
        crates/undone-packs/src/data.rs \
        crates/undone-packs/src/manifest.rs \
        crates/undone-packs/src/loader.rs \
        crates/undone-packs/src/lib.rs
git commit -m "feat(packs): names data file + loader support"
```

---

## Task 5: NPC spawner

**Files:**
- Modify: `crates/undone-packs/Cargo.toml` (add rand + slotmap)
- Create: `crates/undone-packs/src/spawner.rs`
- Modify: `crates/undone-packs/src/lib.rs` (export spawner)

**Step 1: Add dependencies to undone-packs/Cargo.toml**

```toml
rand    = { workspace = true }
slotmap = { workspace = true }
```

**Step 2: Write failing tests first**

Create `crates/undone-packs/src/spawner.rs` with the test module and stub:

```rust
use rand::Rng;
use slotmap::SlotMap;
use undone_domain::{FemaleNpcKey, MaleNpcKey, FemaleNpc, MaleNpc};
use crate::PackRegistry;

pub struct NpcSpawnConfig {
    pub male_count: usize,
    pub female_count: usize,
}

impl Default for NpcSpawnConfig {
    fn default() -> Self {
        Self { male_count: 7, female_count: 2 }
    }
}

pub fn spawn_npcs<R: Rng>(
    config: &NpcSpawnConfig,
    registry: &mut PackRegistry,
    rng: &mut R,
) -> (SlotMap<MaleNpcKey, MaleNpc>, SlotMap<FemaleNpcKey, FemaleNpc>) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use undone_domain::{Personality, NpcTraitDef};

    fn make_registry() -> PackRegistry {
        let mut reg = PackRegistry::new();
        reg.register_npc_traits(vec![
            undone_packs_test_helpers::npc_trait("CHARMING"),
            undone_packs_test_helpers::npc_trait("CRUDE"),
        ]);
        reg.register_names(
            vec!["James".into(), "Thomas".into(), "William".into()],
            vec!["Emma".into(), "Sophie".into()],
        );
        reg
    }
    // ... tests below
}
```

Wait — avoid the `undone_packs_test_helpers` pattern (doesn't exist). Just inline the struct construction.

**Step 2 (revised): Write tests inline**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use undone_domain::{NpcTraitDef, Personality};
    use crate::data::NpcTraitDef as NpcTraitDefData;

    fn make_registry() -> PackRegistry {
        let mut reg = PackRegistry::new();
        reg.register_npc_traits(vec![
            crate::data::NpcTraitDef {
                id: "CHARMING".into(),
                name: "Charming".into(),
                description: "".into(),
                hidden: false,
            },
            crate::data::NpcTraitDef {
                id: "CRUDE".into(),
                name: "Crude".into(),
                description: "".into(),
                hidden: false,
            },
        ]);
        reg.register_names(
            vec!["James".into(), "Thomas".into(), "William".into(),
                 "Oliver".into(), "Harry".into(), "Charlie".into(), "George".into()],
            vec!["Emma".into(), "Sophie".into()],
        );
        reg
    }

    #[test]
    fn spawn_produces_correct_pool_sizes() {
        let mut reg = make_registry();
        let config = NpcSpawnConfig { male_count: 7, female_count: 2 };
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        let (males, females) = spawn_npcs(&config, &mut reg, &mut rng);
        assert_eq!(males.len(), 7);
        assert_eq!(females.len(), 2);
    }

    #[test]
    fn spawn_guarantees_required_personalities() {
        let mut reg = make_registry();
        let config = NpcSpawnConfig { male_count: 7, female_count: 0 };
        let mut rng = rand::rngs::SmallRng::seed_from_u64(99);
        let (males, _) = spawn_npcs(&config, &mut reg, &mut rng);

        let has_romantic = males.values().any(|npc| {
            reg.core_personality(npc.core.personality) == Some(Personality::Romantic)
        });
        let has_jerk = males.values().any(|npc| {
            reg.core_personality(npc.core.personality) == Some(Personality::Jerk)
        });
        let has_friend = males.values().any(|npc| {
            reg.core_personality(npc.core.personality) == Some(Personality::Friend)
        });
        assert!(has_romantic, "pool must contain a Romantic NPC");
        assert!(has_jerk, "pool must contain a Jerk NPC");
        assert!(has_friend, "pool must contain a Friend NPC");
    }

    #[test]
    fn spawn_is_deterministic_with_seed() {
        let config = NpcSpawnConfig { male_count: 5, female_count: 2 };

        let (names1, names2): (Vec<String>, Vec<String>) = {
            let mut reg1 = make_registry();
            let mut rng1 = rand::rngs::SmallRng::seed_from_u64(7);
            let (males1, _) = spawn_npcs(&config, &mut reg1, &mut rng1);
            let n1: Vec<String> = males1.values().map(|n| n.core.name.clone()).collect();

            let mut reg2 = make_registry();
            let mut rng2 = rand::rngs::SmallRng::seed_from_u64(7);
            let (males2, _) = spawn_npcs(&config, &mut reg2, &mut rng2);
            let n2: Vec<String> = males2.values().map(|n| n.core.name.clone()).collect();

            (n1, n2)
        };
        assert_eq!(names1, names2, "same seed must produce same names");
    }

    #[test]
    fn spawn_min_3_required_personalities_even_with_small_pool() {
        // With male_count=3, exactly one of each required type should be assigned.
        let mut reg = make_registry();
        let config = NpcSpawnConfig { male_count: 3, female_count: 0 };
        let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
        let (males, _) = spawn_npcs(&config, &mut reg, &mut rng);
        assert_eq!(males.len(), 3);
        // All three required personalities must be represented
        let has_romantic = males.values().any(|n| reg.core_personality(n.core.personality) == Some(Personality::Romantic));
        let has_jerk = males.values().any(|n| reg.core_personality(n.core.personality) == Some(Personality::Jerk));
        let has_friend = males.values().any(|n| reg.core_personality(n.core.personality) == Some(Personality::Friend));
        assert!(has_romantic && has_jerk && has_friend);
    }
}
```

**Step 3: Run tests to verify they fail (todo! panics)**

```bash
cargo test -p undone-packs -- spawner
```

Expected: tests panic with `not yet implemented`.

**Step 4: Implement spawn_npcs**

Replace `todo!()` with the full implementation:

```rust
use rand::seq::SliceRandom;
use rand::Rng;
use slotmap::SlotMap;
use std::collections::{HashMap, HashSet};
use undone_domain::{
    Age, AlcoholLevel, ArousalLevel, AttractionLevel, Behaviour, BreastSize,
    CharTypeId, FemaleClothing, FemaleNpc, FemaleNpcKey, LikingLevel, LoveLevel,
    MaleClothing, MaleFigure, MaleNpc, MaleNpcKey, NpcCore, NpcTraitId,
    Personality, PersonalityId, PlayerFigure, PregnancyState, RelationshipStatus,
};
use crate::PackRegistry;

// Constant appearance pools (hardcoded; can be moved to data later)
const MALE_FIGURES: &[MaleFigure] = &[
    MaleFigure::Average, MaleFigure::Skinny, MaleFigure::Toned,
    MaleFigure::Muscular, MaleFigure::Thickset, MaleFigure::Paunchy,
];
const FEMALE_FIGURES: &[PlayerFigure] = &[
    PlayerFigure::Slim, PlayerFigure::Toned, PlayerFigure::Womanly,
];
const BREAST_SIZES: &[BreastSize] = &[
    BreastSize::Small, BreastSize::MediumSmall, BreastSize::MediumLarge, BreastSize::Large,
];
const AGES: &[Age] = &[
    Age::EarlyTwenties, Age::Twenties, Age::LateTwenties, Age::Thirties,
];
const RACES: &[&str] = &["white", "black", "south_asian", "east_asian", "mixed"];
const EYE_COLOURS: &[&str] = &["brown", "blue", "green", "grey", "hazel"];
const HAIR_COLOURS: &[&str] = &["dark", "fair", "auburn", "black", "blonde"];
const CORE_PERSONALITIES: &[&str] = &["ROMANTIC", "JERK", "FRIEND", "INTELLECTUAL", "LAD"];
const REQUIRED_PERSONALITIES: &[&str] = &["ROMANTIC", "JERK", "FRIEND"];

pub struct NpcSpawnConfig {
    pub male_count: usize,
    pub female_count: usize,
}

impl Default for NpcSpawnConfig {
    fn default() -> Self {
        Self { male_count: 7, female_count: 2 }
    }
}

pub fn spawn_npcs<R: Rng>(
    config: &NpcSpawnConfig,
    registry: &mut PackRegistry,
    rng: &mut R,
) -> (SlotMap<MaleNpcKey, MaleNpc>, SlotMap<FemaleNpcKey, FemaleNpc>) {
    let mut males = SlotMap::with_key();
    let mut females = SlotMap::with_key();

    // Build the personality assignment list with diversity guarantees.
    // Required slots first, then random fills from the full core set.
    let mut personality_ids: Vec<PersonalityId> = REQUIRED_PERSONALITIES
        .iter()
        .map(|s| registry.intern_personality(s))
        .collect();
    while personality_ids.len() < config.male_count {
        let p = CORE_PERSONALITIES.choose(rng).unwrap();
        personality_ids.push(registry.intern_personality(p));
    }
    personality_ids.shuffle(rng);

    // Pre-intern male names, traits, char type
    let char_type_id = CharTypeId(registry.intern_personality("FRIEND").0); // reuse intern; CharTypeId wraps same Spur
    let npc_trait_ids: Vec<NpcTraitId> = registry
        .npc_trait_defs
        .keys()
        .copied()
        .collect();

    let male_names = registry.male_names().to_vec();
    let female_names = registry.female_names().to_vec();

    for i in 0..config.male_count {
        let name = male_names.choose(rng).cloned().unwrap_or_else(|| format!("NPC{}", i));
        let age = *AGES.choose(rng).unwrap();
        let race = RACES.choose(rng).unwrap().to_string();
        let eye_colour = EYE_COLOURS.choose(rng).unwrap().to_string();
        let hair_colour = HAIR_COLOURS.choose(rng).unwrap().to_string();
        let personality = personality_ids[i];
        let traits = pick_traits(&npc_trait_ids, 2, rng);
        let figure = *MALE_FIGURES.choose(rng).unwrap();

        let core = make_core(name, age, race, eye_colour, hair_colour, personality, traits);
        males.insert(MaleNpc {
            core,
            figure,
            clothing: MaleClothing::default(),
            had_orgasm: false,
            has_baby_with_pc: false,
        });
    }

    for i in 0..config.female_count {
        let name = female_names.choose(rng).cloned().unwrap_or_else(|| format!("FNPC{}", i));
        let age = *AGES.choose(rng).unwrap();
        let race = RACES.choose(rng).unwrap().to_string();
        let eye_colour = EYE_COLOURS.choose(rng).unwrap().to_string();
        let hair_colour = HAIR_COLOURS.choose(rng).unwrap().to_string();
        let personality = registry.intern_personality("FRIEND");
        let traits = pick_traits(&npc_trait_ids, 1, rng);
        let figure = *FEMALE_FIGURES.choose(rng).unwrap();
        let breasts = *BREAST_SIZES.choose(rng).unwrap();

        let core = make_core(name, age, race, eye_colour, hair_colour, personality, traits);
        females.insert(FemaleNpc {
            core,
            char_type: char_type_id,
            figure,
            breasts,
            clothing: FemaleClothing::default(),
            pregnancy: None,
            virgin: true,
        });
    }

    (males, females)
}

fn pick_traits<R: Rng>(pool: &[NpcTraitId], count: usize, rng: &mut R) -> HashSet<NpcTraitId> {
    let mut result = HashSet::new();
    if pool.is_empty() {
        return result;
    }
    let mut indices: Vec<usize> = (0..pool.len()).collect();
    indices.shuffle(rng);
    for &i in indices.iter().take(count.min(pool.len())) {
        result.insert(pool[i]);
    }
    result
}

fn make_core(
    name: String,
    age: Age,
    race: String,
    eye_colour: String,
    hair_colour: String,
    personality: PersonalityId,
    traits: HashSet<NpcTraitId>,
) -> NpcCore {
    NpcCore {
        name,
        age,
        race,
        eye_colour,
        hair_colour,
        personality,
        traits,
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
        contactable: false,
        arousal: ArousalLevel::Comfort,
        alcohol: AlcoholLevel::Sober,
    }
}
```

**Important note on CharTypeId:** `CharTypeId` wraps a `Spur`. The female NPC char type "FRIEND" can be interned the same way as a personality string — both use the same rodeo. The cleanest approach: add `intern_char_type(id: &str) -> CharTypeId` to the registry mirroring `intern_personality`. If that feels like over-engineering for a stub, directly construct `CharTypeId(registry.intern_personality("FRIEND").0)` — both `PersonalityId` and `CharTypeId` wrap `Spur`, and the FRIEND string is already interned for personalities. Use whichever compiles.

**Step 5: Export spawner from lib.rs**

Add to `crates/undone-packs/src/lib.rs`:
```rust
pub mod spawner;
pub use spawner::{NpcSpawnConfig, spawn_npcs};
```

**Step 6: Run tests**

```bash
cargo test -p undone-packs -- spawner
```

Expected: all 4 spawner tests pass.

**Step 7: Run full test suite**

```bash
cargo test
```

Expected: 70+ tests, all pass.

**Step 8: Diagnostics and format**

Use `mcp__rust__get_diagnostics` and `mcp__rust__format_code` on `spawner.rs`.

**Step 9: Commit**

```bash
git add crates/undone-packs/Cargo.toml crates/undone-packs/src/spawner.rs crates/undone-packs/src/lib.rs
git commit -m "feat(packs): NPC spawner with diversity guarantees"
```

---

## Task 6: Character creation — CharCreationConfig + new_game()

**Files:**
- Create: `crates/undone-packs/src/char_creation.rs`
- Modify: `crates/undone-packs/src/lib.rs` (export)

**Step 1: Write failing tests first in char_creation.rs**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use undone_domain::*;
    use crate::{load_packs, spawner::NpcSpawnConfig};
    use std::path::PathBuf;

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()
            .parent().unwrap()
            .join("packs")
    }

    fn base_config() -> CharCreationConfig {
        CharCreationConfig {
            name_fem: "Eva".into(),
            name_androg: "Ev".into(),
            name_masc: "Evan".into(),
            age: Age::EarlyTwenties,
            race: "white".into(),
            figure: PlayerFigure::Slim,
            breasts: BreastSize::MediumLarge,
            always_female: false,
            before_age: 28,
            before_race: "white".into(),
            before_sexuality: Sexuality::StraightMale,
            starting_traits: vec![],
            male_count: 7,
            female_count: 2,
        }
    }

    #[test]
    fn new_game_returns_world_with_player() {
        let (mut registry, _) = load_packs(&packs_dir()).unwrap();
        let config = base_config();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
        let world = new_game(config, &mut registry, &mut rng);

        assert_eq!(world.player.name_fem, "Eva");
        assert_eq!(world.player.before_age, 28);
        assert!(!world.player.always_female);
        assert_eq!(world.game_data.week, 0);
    }

    #[test]
    fn new_game_spawns_npc_pool() {
        let (mut registry, _) = load_packs(&packs_dir()).unwrap();
        let config = base_config();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(2);
        let world = new_game(config, &mut registry, &mut rng);

        assert_eq!(world.male_npcs.len(), 7);
        assert_eq!(world.female_npcs.len(), 2);
    }

    #[test]
    fn new_game_applies_starting_traits() {
        let (mut registry, _) = load_packs(&packs_dir()).unwrap();
        let shy = registry.resolve_trait("SHY").unwrap();
        let mut config = base_config();
        config.starting_traits = vec![shy];
        let mut rng = rand::rngs::SmallRng::seed_from_u64(3);
        let world = new_game(config, &mut registry, &mut rng);

        assert!(world.player.has_trait(shy), "player should have SHY trait");
    }

    #[test]
    fn new_game_always_female_sets_high_femininity() {
        let (mut registry, _) = load_packs(&packs_dir()).unwrap();
        let mut config = base_config();
        config.always_female = true;
        config.before_sexuality = Sexuality::AlwaysFemale;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(4);
        let world = new_game(config, &mut registry, &mut rng);

        assert!(
            world.player.femininity >= 70,
            "always-female PC should start with high femininity"
        );
    }
}
```

**Step 2: Run to verify they fail**

```bash
cargo test -p undone-packs -- char_creation
```

Expected: compile error (module doesn't exist yet) or `not yet implemented` panics.

**Step 3: Implement char_creation.rs**

```rust
use rand::Rng;
use std::collections::{HashMap, HashSet};
use undone_domain::{
    Age, AlcoholLevel, ArousalLevel, BreastSize, Player, PlayerFigure, Sexuality, TraitId,
};
use undone_world::{GameData, World};
use crate::{
    spawner::{spawn_npcs, NpcSpawnConfig},
    PackRegistry,
};

pub struct CharCreationConfig {
    /// Feminine display name (femininity 70+)
    pub name_fem: String,
    /// Androgynous display name (femininity 31–69)
    pub name_androg: String,
    /// Masculine display name (femininity 0–30)
    pub name_masc: String,
    pub age: Age,
    pub race: String,
    pub figure: PlayerFigure,
    pub breasts: BreastSize,
    /// true = PC was never transformed (always female)
    pub always_female: bool,
    pub before_age: u32,
    pub before_race: String,
    pub before_sexuality: Sexuality,
    /// Trait IDs (already resolved by the caller from registry)
    pub starting_traits: Vec<TraitId>,
    pub male_count: usize,
    pub female_count: usize,
}

/// Create a brand-new World from character creation choices.
///
/// Builds the Player, spawns the NPC pool, and returns a World ready for week 1.
pub fn new_game<R: Rng>(
    config: CharCreationConfig,
    registry: &mut PackRegistry,
    rng: &mut R,
) -> World {
    let starting_femininity = if config.always_female { 75 } else { 10 };

    let traits: HashSet<TraitId> = config.starting_traits.into_iter().collect();

    let player = Player {
        name_fem: config.name_fem,
        name_androg: config.name_androg,
        name_masc: config.name_masc,
        age: config.age,
        race: config.race,
        figure: config.figure,
        breasts: config.breasts,
        eye_colour: "brown".into(),
        hair_colour: "dark".into(),
        traits,
        skills: HashMap::new(),
        money: 500,
        stress: 0,
        anxiety: 0,
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
        always_female: config.always_female,
        femininity: starting_femininity,
        before_age: config.before_age,
        before_race: config.before_race,
        before_sexuality: config.before_sexuality,
    };

    let spawn_config = NpcSpawnConfig {
        male_count: config.male_count,
        female_count: config.female_count,
    };
    let (male_npcs, female_npcs) = spawn_npcs(&spawn_config, registry, rng);

    World {
        player,
        male_npcs,
        female_npcs,
        game_data: GameData::default(),
    }
}
```

**Step 4: Export from lib.rs**

Add to `crates/undone-packs/src/lib.rs`:
```rust
pub mod char_creation;
pub use char_creation::{CharCreationConfig, new_game};
```

**Step 5: Run tests**

```bash
cargo test -p undone-packs -- char_creation
```

Expected: all 4 tests pass.

**Step 6: Run full test suite**

```bash
cargo test
```

Expected: all tests pass, no warnings.

**Step 7: Diagnostics and format**

Use `mcp__rust__get_diagnostics` and `mcp__rust__format_code` on `char_creation.rs`.

**Step 8: Commit**

```bash
git add crates/undone-packs/src/char_creation.rs crates/undone-packs/src/lib.rs
git commit -m "feat(packs): CharCreationConfig + new_game() factory"
```

---

## Task 7: Final check and merge

**Step 1: Run clippy**

```bash
cargo clippy --all-targets -- -D warnings
```

Fix any warnings before continuing.

**Step 2: Run full test suite**

```bash
cargo test
```

Expected: 80+ tests, all pass, zero warnings.

**Step 3: Invoke finishing-a-development-branch skill**

Use `superpowers:finishing-a-development-branch` to determine merge/PR/cleanup approach.

---

## Notes for the implementing agent

- `CharTypeId` and `PersonalityId` both wrap `Spur`. Interning "FRIEND" for one also interns it for the other — they share the same rodeo.
- The save test `make_world()` in `undone-save/src/lib.rs` directly constructs `Player`. When `Player` gains new fields, that test will fail to compile. Fix it in Task 2 Step 6.
- The `rand` workspace dep has `features = ["small_rng"]`. SmallRng is imported as `rand::rngs::SmallRng` and seeded with `SeedableRng::seed_from_u64`.
- `personality_ids.shuffle(rng)` requires `rand::seq::SliceRandom` in scope.
- `names.choose(rng)` also requires `rand::seq::SliceRandom`.
- The spawner takes `&mut PackRegistry` to intern personality strings on first call. This is fine — new_game is called once per game start.
- Do not touch `undone-ui` or `undone-scene` — they are out of scope.
