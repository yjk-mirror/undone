//! Phase 1 acceptance — Rhai conditions/effects behave identically to the legacy
//! `undone-expr`/`EffectDef` engine, and the load-time fail-fast guarantee
//! (unknown content id rejected at LOAD) is preserved.
//!
//! These exercise the engine from the OUTSIDE: load the real base pack, run a
//! representative condition + effect, and prove a typo'd content id fails the
//! loader rather than slipping through to runtime.

use std::path::PathBuf;

use undone_packs::load_packs;
use undone_scene::loader::load_scenes;
use undone_scene::{apply_effect_script, compile_condition, compile_effect, eval_bool, SceneCtx};
use undone_world::test_helpers::make_test_world;

fn packs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("packs")
}

/// The real base pack loads cleanly — every authored condition + effect compiles
/// through the Rhai gate. (If any scene's script had a syntax error, unknown
/// method, or unknown content id, this would fail at load.)
#[test]
fn base_pack_loads_through_rhai() {
    let (registry, _metas) = load_packs(&packs_dir()).unwrap();
    let scenes_dir = packs_dir().join("base").join("scenes");
    let scenes = load_scenes(&scenes_dir, &registry).expect("base pack must load through Rhai");
    assert!(scenes.len() >= 60, "expected the full base pack, got {}", scenes.len());
    assert!(scenes.contains_key("base::coffee_shop"));
}

/// A representative condition evaluates through Rhai with the real registry —
/// recorded expectation, not inline-constructed to match the implementation.
#[test]
fn representative_condition_evaluates() {
    let (registry, _metas) = load_packs(&packs_dir()).unwrap();
    let mut world = make_test_world();
    let ctx = SceneCtx::new();

    // A fresh test world has no ROUTE_WORKPLACE flag set.
    let script = compile_condition(
        r#"gd.hasGameFlag("ROUTE_WORKPLACE")"#,
        &registry,
        "acceptance",
    )
    .unwrap();
    assert!(!eval_bool(&script, &world, &ctx, &registry).unwrap());

    // Set it; the same condition now passes.
    world.game_data.set_flag("ROUTE_WORKPLACE");
    assert!(eval_bool(&script, &world, &ctx, &registry).unwrap());
}

/// A representative effect applies through Rhai and mutates persistent state.
#[test]
fn representative_effect_applies() {
    let (registry, _metas) = load_packs(&packs_dir()).unwrap();
    let mut world = make_test_world();
    let mut ctx = SceneCtx::new();

    let before = world.player.money;
    let script = compile_effect(
        r#"w.changeMoney(-5); gd.setGameFlag("COFFEE_SHOP_VISITED");"#,
        &registry,
        "acceptance",
    )
    .unwrap();
    let errors = apply_effect_script(&script, &mut world, &mut ctx, &registry);
    assert!(errors.is_empty(), "effect should apply cleanly: {errors:?}");
    assert_eq!(world.player.money, before - 5);
    assert!(world.game_data.has_flag("COFFEE_SHOP_VISITED"));
}

/// FAIL-FAST: a scene whose condition references an unknown trait id must be
/// rejected at LOAD by `load_scenes`, not deferred to runtime. This is the
/// §4.4 guarantee the whole cutover hinges on.
#[test]
fn typod_content_id_fails_at_load() {
    let dir = std::env::temp_dir().join(format!(
        "undone_rhai_parity_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("bad.toml"),
        r#"
[scene]
id = "test::bad"
pack = "test"
description = "Scene with a typo'd trait id."

[intro]
prose = "It begins."

[[actions]]
id = "go"
label = "Go"
condition = 'w.hasTrait("TYPO_NONEXISTENT_TRAIT")'
"#,
    )
    .unwrap();

    // Empty registry — TYPO_NONEXISTENT_TRAIT cannot resolve.
    let result = load_scenes(&dir, &undone_packs::PackRegistry::new());
    std::fs::remove_dir_all(&dir).ok();

    assert!(
        matches!(result, Err(undone_scene::SceneLoadError::UnknownTrait { .. })),
        "a typo'd trait id must fail at LOAD, got: {result:?}"
    );
}

/// FAIL-FAST: an unknown effect mutator id (bad skill in skillIncrease) is also
/// rejected at load.
#[test]
fn typod_effect_id_fails_at_load() {
    let dir = std::env::temp_dir().join(format!(
        "undone_rhai_parity_eff_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("bad.toml"),
        r#"
[scene]
id = "test::bad_effect"
pack = "test"
description = "Scene with a typo'd skill id in an effect."

[intro]
prose = "It begins."

[[actions]]
id = "go"
label = "Go"
effect = 'w.skillIncrease("NONEXISTENT_SKILL", 5);'
"#,
    )
    .unwrap();

    let result = load_scenes(&dir, &undone_packs::PackRegistry::new());
    std::fs::remove_dir_all(&dir).ok();

    assert!(
        matches!(result, Err(undone_scene::SceneLoadError::UnknownSkill { .. })),
        "a typo'd skill id in an effect must fail at LOAD, got: {result:?}"
    );
}
