//! Independent acceptance tests for the scene condition/effect language cutover.
//!
//! These verify the engine's *guarantees* from the OUTSIDE — public API + real
//! pack data — independent of which expression language backs them:
//!   1. The real base pack (scenes + schedule) loads end-to-end.
//!   2. Conditions evaluate against world state and flip with it.
//!   3. Effects mutate observable persistent state.
//!   4. FAIL-FAST AT LOAD: an unknown content id is rejected by `load_scenes`,
//!      while a VALID (registered) id loads cleanly. Both directions proven.
//!   5. A mutating call used in a condition (read-only context) is rejected.
//!
//! Real pack data is used for the "loads + representative behavior" criteria.
//! Small temp-dir fixtures are used for the fail-fast criteria, with registries
//! built from the public `register_traits`/`register_skills` API — NOT data that
//! mirrors the implementation.

use std::path::PathBuf;

use undone_packs::{load_packs, PackRegistry, SkillDef, TraitDef};
use undone_scene::{
    apply_effect_script, compile_condition, compile_effect, eval_bool, load_schedule, load_scenes,
    SceneCtx, SceneLoadError,
};
use undone_world::test_helpers::make_test_world;

fn packs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("packs")
}

/// Creates a fresh temp dir for a single fixture scene and returns its path.
fn temp_scene_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "undone_expr_accept_{tag}_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// ============================================================================
// Criterion 1 — Pack loads end-to-end (scenes + schedule).
// ============================================================================

/// BREAKS IF: the real base pack no longer loads — scenes fail to compile
/// through the expression gate, or the full set (60+, incl. coffee_shop) is
/// not produced. A user would get an empty/broken game on launch.
#[test]
fn real_base_pack_scenes_load_with_full_set() {
    let (registry, _metas) = load_packs(&packs_dir()).expect("load_packs must succeed");
    let scenes_dir = packs_dir().join("base").join("scenes");
    let scenes = load_scenes(&scenes_dir, &registry).expect("load_scenes must succeed");

    assert!(
        scenes.len() >= 60,
        "expected the full base scene set (>=60), got {}",
        scenes.len()
    );
    assert!(
        scenes.contains_key("base::coffee_shop"),
        "base::coffee_shop must be present in the loaded scene set"
    );
}

/// BREAKS IF: the schedule (which uses conditions to pick scenes) fails to load
/// against the real pack — the game would have no scheduler and never advance.
#[test]
fn real_base_pack_schedule_loads() {
    let (registry, metas) = load_packs(&packs_dir()).expect("load_packs must succeed");
    let scheduler = load_schedule(&metas, &registry);
    assert!(
        scheduler.is_ok(),
        "load_schedule must succeed against the real pack, got: {:?}",
        scheduler.err()
    );
    // No-op litmus: a scheduler with zero scenes would technically be Ok but
    // useless. Assert it actually references real scheduled content.
    let scheduler = scheduler.unwrap();
    assert!(
        !scheduler.all_scene_ids().is_empty(),
        "scheduler loaded but references no scenes — schedule is empty"
    );
}

// ============================================================================
// Criterion 2 — Conditions gate correctly at runtime and flip with state.
// ============================================================================

/// BREAKS IF: a flag condition does not track world state — e.g. always returns
/// false (gate never opens) or always true (gate never closes). User-visible:
/// route-gated content would be permanently unreachable or always shown.
#[test]
fn flag_condition_flips_with_world_state() {
    let (registry, _metas) = load_packs(&packs_dir()).unwrap();
    let mut world = make_test_world();
    let ctx = SceneCtx::new();

    let script = compile_condition(r#"gd.hasGameFlag("ROUTE_WORKPLACE")"#, &registry, "accept")
        .expect("valid flag condition must compile");

    // Fresh world: flag unset → false.
    assert!(
        !eval_bool(&script, &world, &ctx, &registry).unwrap(),
        "ROUTE_WORKPLACE must be false on a fresh world"
    );

    // After setting the flag → true. Same compiled script, only state changed.
    world.game_data.set_flag("ROUTE_WORKPLACE");
    assert!(
        eval_bool(&script, &world, &ctx, &registry).unwrap(),
        "ROUTE_WORKPLACE must be true after set_flag"
    );

    // And flips back when removed — proves it reads live state, not a cache.
    world.game_data.remove_flag("ROUTE_WORKPLACE");
    assert!(
        !eval_bool(&script, &world, &ctx, &registry).unwrap(),
        "ROUTE_WORKPLACE must be false again after remove_flag"
    );
}

/// BREAKS IF: boolean composition (negation / and / or) is mis-evaluated — a
/// composed gate would let through or block the wrong states.
#[test]
fn composed_boolean_condition_evaluates_correctly() {
    let (registry, _metas) = load_packs(&packs_dir()).unwrap();
    let mut world = make_test_world();
    let ctx = SceneCtx::new();

    // "flag A set AND flag B not set"
    let script = compile_condition(
        r#"gd.hasGameFlag("ALPHA") && !gd.hasGameFlag("BETA")"#,
        &registry,
        "accept",
    )
    .expect("composed condition must compile");

    // Neither set: A is false → whole thing false.
    assert!(!eval_bool(&script, &world, &ctx, &registry).unwrap());

    // A set, B unset → true.
    world.game_data.set_flag("ALPHA");
    assert!(eval_bool(&script, &world, &ctx, &registry).unwrap());

    // Both set → the !B clause makes it false.
    world.game_data.set_flag("BETA");
    assert!(!eval_bool(&script, &world, &ctx, &registry).unwrap());
}

// ============================================================================
// Criterion 3 — Effects mutate observable persistent state.
// ============================================================================

/// BREAKS IF: an effect reports success (empty error vec) but does not actually
/// mutate state — money unchanged, flag not set. User-visible: choices would
/// have no consequences (purchases free, flags never recorded).
#[test]
fn effect_mutates_money_and_flag() {
    let (registry, _metas) = load_packs(&packs_dir()).unwrap();
    let mut world = make_test_world();
    let mut ctx = SceneCtx::new();

    let before = world.player.money;
    let script = compile_effect(
        r#"w.changeMoney(-5); gd.setGameFlag("ACCEPT_TEST_FLAG");"#,
        &registry,
        "accept",
    )
    .expect("valid effect must compile");

    let errors = apply_effect_script(&script, &mut world, &mut ctx, &registry);
    assert!(
        errors.is_empty(),
        "effect should apply with no errors, got: {errors:?}"
    );

    // Strong value assertions — not "not null".
    assert_eq!(
        world.player.money,
        before - 5,
        "changeMoney(-5) must decrement money by exactly 5"
    );
    assert!(
        world.game_data.has_flag("ACCEPT_TEST_FLAG"),
        "setGameFlag must persist the flag"
    );
}

// ============================================================================
// Criterion 4 — FAIL-FAST at LOAD (the key guarantee), BOTH directions.
// ============================================================================

/// BREAKS IF: a typo'd trait id in a condition slips past load and is deferred
/// to runtime (or silently treated as false). The whole "validated at load
/// time" contract collapses — content authors ship broken gates undetected.
#[test]
fn unknown_trait_in_condition_rejected_at_load() {
    let dir = temp_scene_dir("badtrait");
    std::fs::write(
        dir.join("bad.toml"),
        r#"
[scene]
id = "test::bad_trait"
pack = "test"
description = "Scene whose action condition names a nonexistent trait."

[intro]
prose = "It begins."

[[actions]]
id = "go"
label = "Go"
condition = 'w.hasTrait("TYPO_NONEXISTENT")'
"#,
    )
    .unwrap();

    // Empty registry — TYPO_NONEXISTENT cannot resolve.
    let result = load_scenes(&dir, &PackRegistry::new());
    std::fs::remove_dir_all(&dir).ok();

    assert!(
        matches!(result, Err(SceneLoadError::UnknownTrait { .. })),
        "unknown trait id must fail load with UnknownTrait, got: {result:?}"
    );
}

/// BREAKS IF: a typo'd skill id inside an EFFECT slips past load. Effects are
/// validated too, not just conditions.
#[test]
fn unknown_skill_in_effect_rejected_at_load() {
    let dir = temp_scene_dir("badskill");
    std::fs::write(
        dir.join("bad.toml"),
        r#"
[scene]
id = "test::bad_skill"
pack = "test"
description = "Scene whose action effect names a nonexistent skill."

[intro]
prose = "It begins."

[[actions]]
id = "go"
label = "Go"
effect = 'w.skillIncrease("NONEXISTENT_SKILL", 5);'
"#,
    )
    .unwrap();

    let result = load_scenes(&dir, &PackRegistry::new());
    std::fs::remove_dir_all(&dir).ok();

    assert!(
        matches!(result, Err(SceneLoadError::UnknownSkill { .. })),
        "unknown skill id in an effect must fail load with UnknownSkill, got: {result:?}"
    );
}

/// BREAKS IF: the loader is OVER-strict and rejects VALID content — a real,
/// registered trait/skill should load cleanly. Without this direction, a loader
/// that rejects *everything* would pass the negative tests above while being
/// completely broken. This is the no-op litmus for the fail-fast guarantee.
#[test]
fn valid_registered_trait_and_skill_load_ok() {
    // Build a registry from the public registration API with exactly the ids
    // the fixture uses. This data does NOT mirror the implementation — it is the
    // documented way a pack declares its content.
    let mut registry = PackRegistry::new();
    registry.register_traits(vec![TraitDef {
        id: "ACCEPT_REAL_TRAIT".to_string(),
        name: "Acceptance Trait".to_string(),
        description: "A trait registered for the positive load-direction test.".to_string(),
        hidden: false,
        group: None,
        conflicts: vec![],
    }]);
    registry.register_skills(vec![SkillDef {
        id: "ACCEPT_REAL_SKILL".to_string(),
        name: "Acceptance Skill".to_string(),
        description: "A skill registered for the positive load-direction test.".to_string(),
        min: 0,
        max: 100,
    }]);

    let dir = temp_scene_dir("goodids");
    std::fs::write(
        dir.join("good.toml"),
        r#"
[scene]
id = "test::good_ids"
pack = "test"
description = "Scene referencing a registered trait and skill."

[intro]
prose = "It begins."

[[actions]]
id = "go"
label = "Go"
condition = 'w.hasTrait("ACCEPT_REAL_TRAIT")'
effect = 'w.skillIncrease("ACCEPT_REAL_SKILL", 5);'
"#,
    )
    .unwrap();

    let result = load_scenes(&dir, &registry);
    std::fs::remove_dir_all(&dir).ok();

    let scenes = result.expect("scene with VALID registered ids must load Ok");
    assert!(
        scenes.contains_key("test::good_ids"),
        "the valid fixture scene must be in the loaded set"
    );
}

// ============================================================================
// Criterion 5 — A mutating call used in a condition is rejected (read-only).
// ============================================================================

/// BREAKS IF: a mutating method compiles as a CONDITION. Conditions are
/// evaluated to decide availability, possibly many times, and must be
/// side-effect free. If `addArousal` were allowed in a condition, merely
/// checking whether an action is available would silently change game state.
#[test]
fn mutating_call_rejected_in_condition() {
    let (registry, _metas) = load_packs(&packs_dir()).unwrap();

    let result = compile_condition(r#"w.addArousal(1)"#, &registry, "accept");
    assert!(
        result.is_err(),
        "a mutating call (addArousal) must be rejected when compiled as a read-only condition"
    );
}

/// Sanity counterpart: the SAME mutating call IS valid as an effect. Proves the
/// rejection above is about read-only context, not a broken/unknown method.
/// BREAKS IF: addArousal is wholesale unknown (then the criterion-5 test would
/// pass for the wrong reason).
#[test]
fn mutating_call_is_valid_as_effect() {
    let (registry, _metas) = load_packs(&packs_dir()).unwrap();

    let result = compile_effect(r#"w.addArousal(1);"#, &registry, "accept");
    assert!(
        result.is_ok(),
        "addArousal must be a valid EFFECT (proving criterion-5 rejection is context-based, not unknown-method): {:?}",
        result.err()
    );
}
