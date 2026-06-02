//! Pre-migration divergence audit (design §8 step 0). Against the CURRENT two-impl
//! tree this surfaces every Rhai-vs-prose divergence as a failure to be decided.
//! After the migration (one shared accessor) it becomes the regression guard.
//!
//! The zero-arg `w`/`gd`/`scene` surface in `ZERO_ARG_WGD` is restricted to methods
//! BOTH backends already implement, so it should agree pre-migration (the genuine
//! divergence — NPC `getName`, spawn-name vs `effective_name()` — lives in the NPC
//! equivalence test below, which pins the unified decision).

use undone_packs::PackRegistry;
use undone_scene::scene_ctx::{SceneCtx, SceneNpcRef};
use undone_scene::script::engine::eval_string_for_test;
use undone_scene::template_ctx::render_prose;
use undone_world::test_helpers::{make_test_male_npc, make_test_world};
use undone_world::World;

/// A registry with the structural FEMININITY skill registered, so name/skill reads
/// resolve on both backends instead of spuriously erroring on an empty registry.
fn test_registry() -> PackRegistry {
    let mut r = PackRegistry::new();
    r.register_skills(vec![undone_packs::SkillDef {
        id: "FEMININITY".into(),
        name: "Femininity".into(),
        description: String::new(),
        min: 0,
        max: 100,
    }]);
    r
}

/// Render `expr` through prose; returns the rendered string.
fn via_prose(expr: &str) -> String {
    let world = make_test_world();
    let ctx = SceneCtx::new();
    let registry = test_registry();
    render_prose(&format!("{{{{ {expr} }}}}"), &world, &ctx, &registry)
        .unwrap_or_else(|e| format!("<<prose error: {e}>>"))
}

/// Evaluate `expr` through the Rhai condition/string path; returns the string form.
fn via_rhai(expr: &str) -> String {
    let world = make_test_world();
    let ctx = SceneCtx::new();
    let registry = test_registry();
    eval_string_for_test(expr, &world, &ctx, &registry)
        .unwrap_or_else(|e| format!("<<rhai error: {e}>>"))
}

/// The zero-arg read surface that BOTH backends should agree on.
const ZERO_ARG_WGD: &[&str] = &[
    "w.getHeight()",
    "w.getFigure()",
    "w.getArousal()",
    "w.getAlcohol()",
    "w.getName()",
    "w.getRace()",
    "w.getAge()",
    "w.pcOrigin()",
    "gd.timeSlot()", // string-returning
    // numeric (compared as strings to keep one assertion form):
    "w.getMoney()",
    "w.getStress()",
    "gd.week()",
    "gd.day()",
    "gd.desire()",
    // bool:
    "w.isVirgin()",
    "w.alwaysFemale()",
    "gd.isWeekday()",
    "gd.isWeekend()",
];

#[test]
fn w_gd_zero_arg_reads_agree_across_backends() {
    let mut mismatches = Vec::new();
    for expr in ZERO_ARG_WGD {
        let r = via_rhai(expr);
        let p = via_prose(expr);
        if r != p {
            mismatches.push(format!("  {expr}: rhai={r:?} prose={p:?}"));
        }
    }
    assert!(
        mismatches.is_empty(),
        "read/prose divergences (decide the unified value for each):\n{}",
        mismatches.join("\n")
    );
}

/// Build a world with an active male NPC whose spawn name and display name differ.
/// `make_test_male_npc` spawns `core.name = "Jake"`; we set `display_name = "Theo"`
/// so `effective_name()` ("Theo") differs from the raw spawn name ("Jake").
fn world_with_named_male() -> (World, SceneCtx, PackRegistry) {
    let mut registry = test_registry();
    let personality = registry.intern_personality("ROMANTIC");
    let mut world = make_test_world();
    let mut male = make_test_male_npc(personality);
    male.core.display_name = Some("Theo".to_string());
    let key = world.male_npcs.insert(male);
    let mut ctx = SceneCtx::new();
    ctx.active_male = Some(key);
    ctx.bind_role("ROLE_X", SceneNpcRef::Male(key));
    (world, ctx, registry)
}

/// The headline divergence (design §1): the unified `getName` accessor returns
/// `effective_name()` (the story-assigned display name), NOT the raw spawn name.
///
/// Prose ALREADY resolves `effective_name()` today (both `m.getName()` and
/// `role.getName(...)` read it), so this pins the decided post-migration value and
/// is green pre-migration on the prose side. The Rhai `role.getName` accessor returns
/// the raw spawn name today; Phase E unifies it onto `effective_name()`, and the
/// post-migration cross-backend equivalence is enforced in `template_ctx` tests.
#[test]
fn get_name_uses_effective_name_not_spawn_name() {
    let (world, ctx, registry) = world_with_named_male();

    let via_m =
        render_prose(r#"{{ m.getName() }}"#, &world, &ctx, &registry).expect("m.getName renders");
    assert_eq!(
        via_m.trim(),
        "Theo",
        "m.getName must be the effective (display) name"
    );

    let via_role = render_prose(r#"{{ role.getName("ROLE_X") }}"#, &world, &ctx, &registry)
        .expect("role.getName renders");
    assert_eq!(
        via_role.trim(),
        "Theo",
        "role.getName must be the effective (display) name, not the spawn name"
    );
}
