use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use undone_packs::PackRegistry;
use undone_world::World;

/// Increment this whenever the save format changes in a breaking way.
pub const SAVE_VERSION: u32 = 1;

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
/// # Errors
///
/// Returns `SaveError::VersionMismatch` if the save was written with a different
/// format version. Returns `SaveError::IdMismatch` or `SaveError::TooManyIds` if
/// the pack content or load order has changed since the file was written.
pub fn load_game(path: &Path, registry: &PackRegistry) -> Result<World, SaveError> {
    let json = std::fs::read_to_string(path).map_err(|e| SaveError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    let file: SaveFile = serde_json::from_str(&json)?;

    if file.version != SAVE_VERSION {
        return Err(SaveError::VersionMismatch {
            saved: file.version,
            expected: SAVE_VERSION,
        });
    }

    validate_ids(&file.id_strings, registry)?;

    Ok(file.world)
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
    use undone_domain::*;
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
                before_age: 30,
                before_race: "white".into(),
                before_sexuality: Sexuality::StraightMale,
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
                always_female: false,
                femininity: 15,
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
        assert_eq!(loaded.player.femininity, world.player.femininity);
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

        // Patch the version field
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
            },
            undone_packs::TraitDef {
                id: "POSH".into(),
                name: "Posh".into(),
                description: "".into(),
                hidden: false,
            },
        ]);

        // Saved only knew about "SHY" (first registered); registry also has POSH
        let saved = vec!["SHY".to_string()];
        let result = validate_ids(&saved, &registry);
        assert!(result.is_ok(), "registry having more IDs than save is OK");
    }

    /// Create a temp dir under the OS temp location for test files.
    fn tempfile_dir() -> PathBuf {
        let dir = std::env::temp_dir().join("undone_save_tests");
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }
}
