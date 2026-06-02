//! Independent acceptance test for criterion 9: a save written at the PREVIOUS
//! format version (6) loads under the CURRENT version (7) with desire defaulting
//! to 0 — no error.
//!
//! Written from the criterion alone. Exercises the public `save_game` /
//! `load_game` API from the outside against the REAL base pack registry (so the
//! interned-id validation table is real, not a fixture the implementer hand-fit).

use std::path::PathBuf;

use undone_save::{load_game, save_game, SAVE_VERSION};

fn packs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("packs")
}

fn tempfile_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("undone_desire_save_compat");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// BREAKS IF: an old (v6) save — which has NO `desire` field in `game_data` —
/// fails to load under v7, or loads with garbage instead of desire == 0. A real
/// player upgrading the game would lose their save.
#[test]
fn v6_save_loads_under_v7_with_desire_zero() {
    // The current save format must actually be 7 for this test to mean anything.
    assert_eq!(
        SAVE_VERSION, 7,
        "this back-compat test targets v6→v7; SAVE_VERSION changed to {SAVE_VERSION}"
    );

    let (mut registry, _) = undone_packs::load_packs(&packs_dir()).unwrap();

    // Build a real, current-format world and save it.
    let mut world = undone_world::test_helpers::make_test_world();
    // Give the live world a non-zero desire so we can prove the v6 file (which
    // lacks the field) loads as 0 and not as "whatever was in memory".
    world.game_data.set_desire(77);

    let dir = tempfile_dir();
    let path = dir.join("v6_compat.json");
    save_game(&world, &registry, &path).expect("save should succeed");

    // Downgrade the on-disk file to look like a v6 save: stamp version = 6 and
    // delete the `desire` field that v6 never wrote.
    let content = std::fs::read_to_string(&path).unwrap();
    let mut parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    parsed["version"] = serde_json::Value::Number(6u32.into());
    let removed = parsed["world"]["game_data"]
        .as_object_mut()
        .unwrap()
        .remove("desire");
    assert!(
        removed.is_some(),
        "precondition: current save wrote a `desire` field we could strip to \
         simulate v6 (the field must exist to begin with)"
    );
    std::fs::write(&path, serde_json::to_string_pretty(&parsed).unwrap()).unwrap();

    // Load the v6 file under the current code — must migrate transparently.
    let loaded = load_game(&path, &mut registry).expect("v6 save must load under v7");

    assert_eq!(
        loaded.game_data.desire(),
        0,
        "a v6 save with no desire field must load with desire == 0, got {}",
        loaded.game_data.desire()
    );

    // And the rest of the world still round-trips (the migration is otherwise a
    // no-op, not a wipe).
    assert_eq!(loaded.player.name_fem, world.player.name_fem);
    assert_eq!(loaded.game_data.week, world.game_data.week);
}

/// BREAKS IF: a v6 save that DID carry a desire value (e.g. a hand-edited or
/// forward-compatible field) is rejected. The field is serde(default), so a
/// present value should be honoured on load, not refused.
#[test]
fn v6_save_with_present_desire_value_is_honoured() {
    let (mut registry, _) = undone_packs::load_packs(&packs_dir()).unwrap();
    let world = undone_world::test_helpers::make_test_world();

    let dir = tempfile_dir();
    let path = dir.join("v6_with_desire.json");
    save_game(&world, &registry, &path).unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    let mut parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    parsed["version"] = serde_json::Value::Number(6u32.into());
    parsed["world"]["game_data"]["desire"] = serde_json::Value::Number(42u32.into());
    std::fs::write(&path, serde_json::to_string_pretty(&parsed).unwrap()).unwrap();

    let loaded = load_game(&path, &mut registry).expect("v6 save must load under v7");
    assert_eq!(
        loaded.game_data.desire(),
        42,
        "a present desire value in a v6 save must survive the migration"
    );
}
