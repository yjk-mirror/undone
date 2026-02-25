use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use undone_packs::PackRegistry;
use undone_world::World;

/// Increment this whenever the save format changes in a breaking way.
pub const SAVE_VERSION: u32 = 4;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum SaveError {
    #[error("io error with {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("save version {saved} is not compatible with current version {expected}")]
    VersionMismatch { saved: u32, expected: u32 },

    /// The interned ID at `index` was `saved` when the file was written, but is
    /// `current` now. This means pack content changed (or load order differs) and
    /// the save file is no longer valid.
    #[error("interned ID mismatch at index {index}: saved '{saved}', current '{current}'")]
    IdMismatch {
        index: usize,
        saved: String,
        current: String,
    },

    /// The save file references more interned IDs than the current registry has.
    /// A pack was probably removed.
    #[error(
        "save file has {saved_count} interned IDs but registry only has {registry_count}; \
         a pack may have been removed"
    )]
    TooManyIds {
        saved_count: usize,
        registry_count: usize,
    },
}

// ---------------------------------------------------------------------------
// Save file format
// ---------------------------------------------------------------------------

/// The on-disk save format. Versioned for future migration support.
///
/// # ID stability
///
/// Interned IDs (`TraitId`, `SkillId`, etc.) are stored as raw `u32` values
/// (lasso `Spur` indices) inside `world`. These values are only valid when the
/// registry has the same set of strings at the same indices.
///
/// `id_strings` records all interned strings in Spur-index order at save time.
/// On load, this is validated against the current registry to ensure the IDs
/// in `world` still refer to the correct strings.
#[derive(Serialize, Deserialize)]
pub struct SaveFile {
    pub version: u32,
    /// All strings interned by the PackRegistry, in Spur-index order.
    pub id_strings: Vec<String>,
    pub world: World,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Serialize `world` to a JSON save file at `path`.
///
/// The save file embeds all interned ID strings so that `load_game` can
/// validate them against the current pack state on load.
pub fn save_game(world: &World, registry: &PackRegistry, path: &Path) -> Result<(), SaveError> {
    let id_strings = registry.all_interned_strings();
    let file = SaveFile {
        version: SAVE_VERSION,
        id_strings,
        world: world.clone(),
    };
    let json = serde_json::to_string_pretty(&file)?;
    std::fs::write(path, &json).map_err(|e| SaveError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;
    Ok(())
}

/// Deserialize a save file from `path`, validating it against the current registry.
///
/// Automatically migrates older saves to the current format:
///   - v1 → v2: `always_female: bool` + old `Sexuality` variants → `origin: PcOrigin`
///     + `before_sexuality: Option<BeforeSexuality>`
///   - v2 → v3: flat `before_age`, `before_race`, `before_sexuality` fields on player
///     → `before: Option<BeforeIdentity>`; adds `day` and `time_slot` to
///     `game_data` if missing.
///   - v3 → v4: renames `Age::Twenties` → `Age::MidLateTwenties` in player.age
///     and player.before.age.
///
/// # Errors
///
/// Returns `SaveError::VersionMismatch` if the save version is unknown. Returns
/// `SaveError::IdMismatch` or `SaveError::TooManyIds` if the pack content or load
/// order has changed since the file was written.
pub fn load_game(path: &Path, registry: &PackRegistry) -> Result<World, SaveError> {
    let json = std::fs::read_to_string(path).map_err(|e| SaveError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    // Parse to a raw Value first so we can inspect the version and migrate if needed.
    let mut raw: serde_json::Value = serde_json::from_str(&json)?;

    let version = raw
        .get("version")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .unwrap_or(0);

    if version == 1 {
        // v1 → v2 → v3 → v4
        raw = migrate_v1_to_v2(raw);
        raw = migrate_v2_to_v3(raw);
        raw = migrate_v3_to_v4(raw);
        raw["version"] = serde_json::Value::Number(4u32.into());
    } else if version == 2 {
        // v2 → v3 → v4
        raw = migrate_v2_to_v3(raw);
        raw = migrate_v3_to_v4(raw);
        raw["version"] = serde_json::Value::Number(4u32.into());
    } else if version == 3 {
        // v3 → v4: rename Age::Twenties → Age::MidLateTwenties
        raw = migrate_v3_to_v4(raw);
        raw["version"] = serde_json::Value::Number(4u32.into());
    } else if version != SAVE_VERSION {
        return Err(SaveError::VersionMismatch {
            saved: version,
            expected: SAVE_VERSION,
        });
    }

    let file: SaveFile = serde_json::from_value(raw)?;

    validate_ids(&file.id_strings, registry)?;

    Ok(file.world)
}

// ---------------------------------------------------------------------------
// Migration helpers
// ---------------------------------------------------------------------------

/// Transform a v1 save JSON into a v2-compatible JSON structure.
///
/// v1 differences in `world.player`:
///   - `always_female: bool` (replaced by `origin: PcOrigin` string)
///   - `before_sexuality: string` (variants: StraightMale, GayMale, BiMale, AlwaysFemale)
///     (replaced by `before_sexuality: Option<BeforeSexuality>` string | null)
///
/// Because traits are stored as interned integer IDs in v1 saves, we cannot
/// distinguish `AlwaysFemale` from `CisFemaleTransformed` via the NOT_TRANSFORMED
/// trait at migration time. We therefore map `always_female: true` to `"AlwaysFemale"`
/// as the safe default — callers who need the distinction can update via the UI.
fn migrate_v1_to_v2(mut save_json: serde_json::Value) -> serde_json::Value {
    if let Some(player) = save_json.get_mut("world").and_then(|w| w.get_mut("player")) {
        let always_female = player
            .get("always_female")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let old_sexuality = player
            .get("before_sexuality")
            .and_then(|v| v.as_str())
            .unwrap_or("StraightMale")
            .to_string();

        // Map always_female bool → PcOrigin string.
        // We cannot distinguish AlwaysFemale from CisFemaleTransformed without the
        // NOT_TRANSFORMED trait (which is stored as an interned integer in the save),
        // so we use AlwaysFemale as the safe default for always_female=true saves.
        let origin = if always_female {
            "AlwaysFemale"
        } else {
            "CisMaleTransformed"
        };

        // Map old Sexuality variants → new BeforeSexuality JSON representation.
        // Some(BeforeSexuality::X) serialises as the string "X"; None serialises as null.
        let new_sexuality = match old_sexuality.as_str() {
            "StraightMale" => serde_json::Value::String("AttractedToWomen".to_string()),
            "GayMale" => serde_json::Value::String("AttractedToMen".to_string()),
            "BiMale" => serde_json::Value::String("AttractedToBoth".to_string()),
            // "AlwaysFemale" variant meant no meaningful pre-transformation sexuality
            _ => serde_json::Value::Null,
        };

        if let Some(obj) = player.as_object_mut() {
            obj.remove("always_female");
            obj.insert(
                "origin".to_string(),
                serde_json::Value::String(origin.to_string()),
            );
            obj.insert("before_sexuality".to_string(), new_sexuality);
        }
    }
    save_json
}

/// Transform a v2 save JSON into a v3-compatible JSON structure.
///
/// v2 differences in `world.player`:
///   - `before_age: u32` (a raw integer, not an Age enum string)
///   - `before_race: String`
///   - `before_sexuality: Option<BeforeSexuality>` (string or null)
///
/// v3 replaces those three flat fields with:
///   - `before: Option<BeforeIdentity>` (an object or null)
///
/// Additionally, v2 `world.game_data` may be missing `day` and `time_slot`
/// (they are new in v3). Those fields have serde defaults so they are handled
/// automatically by deserialization, but we explicitly insert them here for
/// clarity and to keep the raw JSON valid before any further processing.
fn migrate_v2_to_v3(mut save_json: serde_json::Value) -> serde_json::Value {
    if let Some(player) = save_json.get_mut("world").and_then(|w| w.get_mut("player")) {
        // Extract old flat fields before mutating the object.
        let before_race = player
            .get("before_race")
            .and_then(|v| v.as_str())
            .unwrap_or("white")
            .to_string();

        let before_sexuality = player
            .get("before_sexuality")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        let before_age_num = player
            .get("before_age")
            .and_then(|v| v.as_u64())
            .unwrap_or(25);

        // Map numeric age to Age enum string.
        let before_age = match before_age_num {
            0..=19 => "LateTeen",
            20..=22 => "EarlyTwenties",
            23..=26 => "Twenties",
            27..=29 => "LateTwenties",
            30..=39 => "Thirties",
            40..=49 => "Forties",
            50..=59 => "Fifties",
            _ => "Old",
        };

        // Use name_masc for the before identity name, falling back to name_androg.
        let before_name = player
            .get("name_masc")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        // AlwaysFemale PCs have no pre-transformation identity.
        let origin = player
            .get("origin")
            .and_then(|v| v.as_str())
            .unwrap_or("CisMaleTransformed");

        let before = if origin == "AlwaysFemale" {
            serde_json::Value::Null
        } else {
            let mut before_obj = serde_json::Map::new();
            before_obj.insert("name".into(), serde_json::Value::String(before_name));
            before_obj.insert(
                "age".into(),
                serde_json::Value::String(before_age.to_string()),
            );
            before_obj.insert("race".into(), serde_json::Value::String(before_race));
            // Keep before_sexuality as-is if present; default to AttractedToWomen.
            if before_sexuality.is_null() {
                before_obj.insert(
                    "sexuality".into(),
                    serde_json::Value::String("AttractedToWomen".into()),
                );
            } else {
                before_obj.insert("sexuality".into(), before_sexuality);
            }
            before_obj.insert("figure".into(), serde_json::Value::String("Average".into()));
            before_obj.insert("traits".into(), serde_json::Value::Array(vec![]));
            serde_json::Value::Object(before_obj)
        };

        if let Some(obj) = player.as_object_mut() {
            // Remove the three flat v2 fields.
            obj.remove("before_age");
            obj.remove("before_race");
            obj.remove("before_sexuality");
            // Insert the new nested before field.
            obj.insert("before".into(), before);
        }
    }

    // GameData: insert day and time_slot if absent (they have serde defaults but
    // explicit insertion keeps the migrated JSON self-consistent).
    if let Some(game_data) = save_json
        .get_mut("world")
        .and_then(|w| w.get_mut("game_data"))
    {
        if let Some(gd) = game_data.as_object_mut() {
            gd.entry("day")
                .or_insert(serde_json::Value::Number(0.into()));
            gd.entry("time_slot")
                .or_insert(serde_json::Value::String("Morning".into()));
        }
    }

    save_json
}

/// Transform a v3 save JSON into a v4-compatible JSON structure.
///
/// v4 renames the `Age::Twenties` variant to `Age::MidLateTwenties`. Saves that
/// contain the old string `"Twenties"` in `world.player.age` or
/// `world.player.before.age` are updated to `"MidLateTwenties"`.
fn migrate_v3_to_v4(mut save_json: serde_json::Value) -> serde_json::Value {
    fn rename_age(val: &mut serde_json::Value) {
        if val.as_str() == Some("Twenties") {
            *val = serde_json::Value::String("MidLateTwenties".to_string());
        }
    }

    if let Some(world) = save_json.get_mut("world") {
        if let Some(player) = world.get_mut("player") {
            if let Some(age) = player.get_mut("age") {
                rename_age(age);
            }
            if let Some(before) = player.get_mut("before") {
                if let Some(age) = before.get_mut("age") {
                    rename_age(age);
                }
            }
        }
    }

    save_json
}

/// Verify that all IDs recorded in the save file still map to the same strings
/// in the current registry.
fn validate_ids(saved: &[String], registry: &PackRegistry) -> Result<(), SaveError> {
    let current = registry.all_interned_strings();

    // If the save references IDs beyond the registry's range, a pack was removed.
    if saved.len() > current.len() {
        return Err(SaveError::TooManyIds {
            saved_count: saved.len(),
            registry_count: current.len(),
        });
    }

    // Every ID in the save must match the same-index ID in the current registry.
    for (i, (s, c)) in saved.iter().zip(current.iter()).enumerate() {
        if s != c {
            return Err(SaveError::IdMismatch {
                index: i,
                saved: s.clone(),
                current: c.clone(),
            });
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    use slotmap::SlotMap;
    use undone_domain::{BeforeIdentity, BeforeSexuality, MaleFigure, PcOrigin, *};
    use undone_world::{GameData, World};

    use super::*;

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap() // exits crates/undone-save/
            .parent()
            .unwrap() // exits crates/
            .join("packs")
    }

    fn make_world(registry: &PackRegistry) -> World {
        let shy_id = registry.resolve_trait("SHY").unwrap();
        let mut traits = HashSet::new();
        traits.insert(shy_id);

        World {
            player: Player {
                name_fem: "Eva".into(),
                name_androg: "Ev".into(),
                name_masc: "Evan".into(),
                before: Some(BeforeIdentity {
                    name: "Evan".into(),
                    age: Age::MidLateTwenties,
                    race: "white".into(),
                    sexuality: BeforeSexuality::AttractedToWomen,
                    figure: MaleFigure::Average,
                    traits: HashSet::new(),
                }),
                age: Age::LateTeen,
                race: "east_asian".into(),
                figure: PlayerFigure::Slim,
                breasts: BreastSize::Large,
                eye_colour: "brown".into(),
                hair_colour: "dark".into(),
                traits,
                skills: HashMap::new(),
                money: 500,
                stress: 10,
                anxiety: 3,
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
                origin: PcOrigin::CisMaleTransformed,
            },
            male_npcs: SlotMap::with_key(),
            female_npcs: SlotMap::with_key(),
            game_data: GameData::default(),
        }
    }

    #[test]
    fn round_trip_save_and_load() {
        let (registry, _) = undone_packs::load_packs(&packs_dir()).unwrap();
        let world = make_world(&registry);

        let dir = tempfile_dir();
        let path = dir.join("test_save.json");

        save_game(&world, &registry, &path).expect("save should succeed");
        assert!(path.exists(), "save file should exist");

        let loaded = load_game(&path, &registry).expect("load should succeed");
        assert_eq!(loaded.player.name_fem, world.player.name_fem);
        assert_eq!(loaded.player.stress, world.player.stress);
        assert_eq!(loaded.player.money, world.player.money);
        assert_eq!(loaded.player.skills, world.player.skills);
        assert_eq!(loaded.player.traits, world.player.traits);
        assert_eq!(loaded.game_data.week, world.game_data.week);
    }

    #[test]
    fn save_file_is_valid_json() {
        let (registry, _) = undone_packs::load_packs(&packs_dir()).unwrap();
        let world = make_world(&registry);

        let dir = tempfile_dir();
        let path = dir.join("json_test.json");
        save_game(&world, &registry, &path).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["version"], SAVE_VERSION);
        assert!(parsed["id_strings"].is_array());
        assert!(parsed["world"].is_object());
    }

    #[test]
    fn load_fails_on_version_mismatch() {
        let (registry, _) = undone_packs::load_packs(&packs_dir()).unwrap();
        let world = make_world(&registry);

        let dir = tempfile_dir();
        let path = dir.join("version_mismatch.json");
        save_game(&world, &registry, &path).unwrap();

        // Patch the version field to something unknown (> current)
        let content = std::fs::read_to_string(&path).unwrap();
        let mut parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        parsed["version"] = serde_json::Value::Number((SAVE_VERSION + 1).into());
        std::fs::write(&path, serde_json::to_string_pretty(&parsed).unwrap()).unwrap();

        let result = load_game(&path, &registry);
        assert!(
            matches!(result, Err(SaveError::VersionMismatch { .. })),
            "should fail with VersionMismatch"
        );
    }

    #[test]
    fn validate_ids_detects_mismatch() {
        let mut registry = PackRegistry::new();
        registry.register_traits(vec![undone_packs::TraitDef {
            id: "ALPHA".into(),
            name: "Alpha".into(),
            description: "".into(),
            hidden: false,
            group: None,
            conflicts: vec![],
        }]);

        // "ALPHA" → index 0; save claims index 0 = "BETA"
        let saved = vec!["BETA".to_string()];
        let result = validate_ids(&saved, &registry);
        assert!(
            matches!(result, Err(SaveError::IdMismatch { index: 0, .. })),
            "should detect ID mismatch at index 0"
        );
    }

    #[test]
    fn validate_ids_detects_too_many_saved_ids() {
        let registry = PackRegistry::new(); // empty registry
        let saved = vec!["SOMETHING".to_string()]; // saved has 1 ID, registry has 0
        let result = validate_ids(&saved, &registry);
        assert!(
            matches!(result, Err(SaveError::TooManyIds { .. })),
            "should detect too many saved IDs"
        );
    }

    #[test]
    fn validate_ids_passes_when_registry_has_more() {
        let mut registry = PackRegistry::new();
        registry.register_traits(vec![
            undone_packs::TraitDef {
                id: "SHY".into(),
                name: "Shy".into(),
                description: "".into(),
                hidden: false,
                group: None,
                conflicts: vec![],
            },
            undone_packs::TraitDef {
                id: "POSH".into(),
                name: "Posh".into(),
                description: "".into(),
                hidden: false,
                group: None,
                conflicts: vec![],
            },
        ]);

        // Saved only knew about "SHY" (first registered); registry also has POSH
        let saved = vec!["SHY".to_string()];
        let result = validate_ids(&saved, &registry);
        assert!(result.is_ok(), "registry having more IDs than save is OK");
    }

    /// Test that a v1-format save is correctly migrated through v2 and v3 on load.
    ///
    /// v1 used `always_female: bool` and `before_sexuality: "StraightMale"` (etc.).
    /// After the full v1→v2→v3 migration chain the player should have:
    ///   - `origin: CisMaleTransformed`
    ///   - `before: Some(BeforeIdentity { sexuality: AttractedToWomen, ... })`
    #[test]
    fn migrate_v1_save_to_v3() {
        let (registry, _) = undone_packs::load_packs(&packs_dir()).unwrap();

        // Build a v1-format JSON manually.  We need real interned IDs in id_strings
        // and a real SHY trait in the world so the validator passes.
        let id_strings = registry.all_interned_strings();

        // Build a minimal v1 world JSON by constructing from the current world
        // and then patching it to look like v1.
        let world = make_world(&registry);
        let mut save_json = serde_json::to_value(&SaveFile {
            version: 1, // lie about the version
            id_strings,
            world,
        })
        .unwrap();

        // Patch the version field to 1.
        save_json["version"] = serde_json::Value::Number(1u32.into());

        // Patch the player to v1 shape: remove `origin`, remove `before`,
        // add `always_female`. The v1→v2 migration will add back `origin` and
        // `before_sexuality` as flat fields; v2→v3 will then collapse them into
        // `before: Option<BeforeIdentity>`.
        {
            let player = save_json["world"]["player"].as_object_mut().unwrap();
            player.remove("origin");
            player.remove("before");
            player.insert("always_female".to_string(), serde_json::Value::Bool(false));
            // before_sexuality in v1 was a bare string like "StraightMale"
            player.insert(
                "before_sexuality".to_string(),
                serde_json::Value::String("StraightMale".to_string()),
            );
        }

        // Write the patched JSON to a temp file.
        let dir = tempfile_dir();
        let path = dir.join("v1_migration_test.json");
        std::fs::write(&path, serde_json::to_string_pretty(&save_json).unwrap()).unwrap();

        // Load it — should migrate v1→v2→v3 transparently.
        let loaded = load_game(&path, &registry).expect("v1 load should succeed");

        assert_eq!(
            loaded.player.origin,
            PcOrigin::CisMaleTransformed,
            "always_female=false should migrate to CisMaleTransformed"
        );
        // After the full v1→v2→v3 chain, before_sexuality="AttractedToWomen" (mapped
        // from "StraightMale") is assembled into a BeforeIdentity object.
        assert!(
            loaded.player.before.is_some(),
            "before should be Some after v1→v2→v3 migration"
        );
        assert_eq!(
            loaded.player.before.as_ref().unwrap().sexuality,
            BeforeSexuality::AttractedToWomen,
            "StraightMale v1 sexuality should map to AttractedToWomen"
        );
    }

    /// Test that a v2-format save (flat before_age / before_race / before_sexuality)
    /// is correctly migrated to v3 (nested before: Option<BeforeIdentity>) on load.
    #[test]
    fn migrate_v2_save_to_v3() {
        let (registry, _) = undone_packs::load_packs(&packs_dir()).unwrap();

        let id_strings = registry.all_interned_strings();

        // Build a v2-format world from the current world, then patch it.
        let world = make_world(&registry);
        let mut save_json = serde_json::to_value(&SaveFile {
            version: 2,
            id_strings,
            world,
        })
        .unwrap();

        // Patch to v2 player shape: remove `before`, add flat before_* fields.
        {
            let player = save_json["world"]["player"].as_object_mut().unwrap();
            player.remove("before");
            player.insert(
                "before_age".to_string(),
                serde_json::Value::Number(30u32.into()),
            );
            player.insert(
                "before_race".to_string(),
                serde_json::Value::String("white".to_string()),
            );
            player.insert(
                "before_sexuality".to_string(),
                serde_json::Value::String("AttractedToWomen".to_string()),
            );
        }

        // Also remove day/time_slot from game_data to simulate a v2 save that
        // predates those fields.
        {
            let gd = save_json["world"]["game_data"].as_object_mut().unwrap();
            gd.remove("day");
            gd.remove("time_slot");
        }

        let dir = tempfile_dir();
        let path = dir.join("v2_migration_test.json");
        std::fs::write(&path, serde_json::to_string_pretty(&save_json).unwrap()).unwrap();

        // Load — should migrate v2→v3 transparently.
        let loaded = load_game(&path, &registry).expect("v2→v3 load should succeed");

        assert!(
            loaded.player.before.is_some(),
            "before should be Some after v2→v3 migration"
        );
        assert_eq!(
            loaded.player.before.as_ref().unwrap().race,
            "white",
            "before.race should be migrated from before_race"
        );
        assert_eq!(
            loaded.game_data.time_slot,
            undone_domain::TimeSlot::Morning,
            "time_slot should default to Morning when absent in v2 save"
        );
    }

    /// Create a temp dir under the OS temp location for test files.
    fn tempfile_dir() -> PathBuf {
        let dir = std::env::temp_dir().join("undone_save_tests");
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }
}
