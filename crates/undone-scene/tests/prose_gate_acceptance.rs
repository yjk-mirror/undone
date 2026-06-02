//! Acceptance tests for the prose load gate (design §5.4).
//!
//! These exercise the gate from the OUTSIDE, through the two public entry points a
//! content author / loader actually reaches:
//!   - `validate_prose` — the static load-time gate run on every prose field.
//!   - `render_prose`    — the runtime Minijinja render path.
//!   - `load_packs` + `load_scenes` — the real loader, which runs the gate on every
//!     prose field of every scene in the base pack.
//!
//! The gate's contract: a prose template may only call methods that exist on a known
//! receiver AND are flagged prose-callable, with any string-literal content-id arg
//! resolving against the registry. Read methods barred from prose (condition-only,
//! e.g. `checkSkill`) and write mutators must be rejected at load — not discovered at
//! render time when the page is already on screen.
//!
//! Anti-circular-validation: criterion 6 uses the REAL base pack on disk (not a
//! hand-built fixture). Unit-style registries register only the ids referenced.

use std::path::{Path, PathBuf};

use undone_packs::PackRegistry;
use undone_scene::scene_ctx::SceneCtx;
use undone_scene::script::api::prose_validate::validate_prose;
use undone_scene::template_ctx::render_prose;
use undone_world::test_helpers::{make_test_male_npc, make_test_world};

// ---------------------------------------------------------------------------
// Test-registry helpers — register ONLY the ids each test references.
// ---------------------------------------------------------------------------

fn registry_with(traits: &[&str], skills: &[&str]) -> PackRegistry {
    let mut r = PackRegistry::new();
    r.register_traits(
        traits
            .iter()
            .map(|id| undone_packs::TraitDef {
                id: (*id).into(),
                name: (*id).into(),
                description: String::new(),
                hidden: false,
                group: None,
                conflicts: vec![],
            })
            .collect(),
    );
    r.register_skills(
        skills
            .iter()
            .map(|id| undone_packs::SkillDef {
                id: (*id).into(),
                name: (*id).into(),
                description: String::new(),
                min: 0,
                max: 100,
            })
            .collect(),
    );
    r
}

/// Path to the real base pack on disk: `<crate_manifest>/../../packs`.
fn packs_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("packs")
}

// ===========================================================================
// Criterion 1 — VALID read method + content id passes the gate.
// ===========================================================================

#[test]
fn criterion1_valid_trait_read_passes() {
    // ASSERTS: an author writing `{% if w.hasTrait("SHY") %}` against a registry that
    // has SHY loads without error (the gate does not false-reject legitimate prose).
    let r = registry_with(&["SHY"], &[]);
    let result = validate_prose(
        r#"{% if w.hasTrait("SHY") %}x{% endif %}"#,
        &r,
        "criterion1",
    );
    assert!(
        result.is_ok(),
        "valid hasTrait read must pass the gate, got: {result:?}"
    );
}

#[test]
fn criterion1_valid_name_read_passes() {
    // ASSERTS: `{{ w.getName() }}` — a zero-arg prose read — passes the gate.
    let r = registry_with(&[], &[]);
    let result = validate_prose(r#"{{ w.getName() }}"#, &r, "criterion1");
    assert!(
        result.is_ok(),
        "valid getName read must pass the gate, got: {result:?}"
    );
}

// ===========================================================================
// Criterion 2 — UNKNOWN method fails the gate.
// ===========================================================================

#[test]
fn criterion2_unknown_method_fails() {
    // ASSERTS: a typo'd / nonexistent method `w.notAReal()` is caught at LOAD, not
    // silently rendered as blank prose for the player.
    let r = registry_with(&[], &[]);
    let result = validate_prose(r#"{{ w.notAReal() }}"#, &r, "criterion2");
    assert!(
        result.is_err(),
        "unknown method w.notAReal() must be rejected, got Ok"
    );
    // No-op litmus: assert the error names the offending method, not just "is_err".
    let msg = format!("{:?}", result.unwrap_err());
    assert!(
        msg.contains("notAReal"),
        "error should name the unknown method, got: {msg}"
    );
}

#[test]
fn criterion2_unknown_content_id_fails() {
    // ASSERTS: a real method with a string-literal id that is NOT in the registry
    // (`w.hasTrait("NOPE")`) fails the gate — content-id resolution is enforced.
    let r = registry_with(&["SHY"], &[]);
    let result = validate_prose(
        r#"{% if w.hasTrait("NOPE") %}x{% endif %}"#,
        &r,
        "criterion2",
    );
    assert!(
        result.is_err(),
        "unknown trait id NOPE must be rejected, got Ok"
    );
}

// ===========================================================================
// Criterion 3 — checkSkill (condition-only) is barred from prose.
// ===========================================================================

#[test]
fn criterion3_checkskill_barred_from_prose() {
    // ASSERTS: `w.checkSkill("CHARM", 10)` — which has an RNG side effect on the roll
    // cache and is condition-only — cannot leak into prose, where it would re-roll on
    // every render.
    let r = registry_with(&[], &["CHARM"]);
    let result = validate_prose(
        r#"{% if w.checkSkill("CHARM", 10) %}x{% endif %}"#,
        &r,
        "criterion3",
    );
    assert!(
        result.is_err(),
        "checkSkill must be barred from prose (condition-only), got Ok"
    );
    // No-op litmus: the rejection must be about THIS method (CHARM is registered, so
    // this is not an id-resolution failure — it's a context failure).
    let msg = format!("{:?}", result.unwrap_err());
    assert!(
        msg.contains("checkSkill"),
        "error should name checkSkill, got: {msg}"
    );
}

// ===========================================================================
// Criterion 4 — WRITE method fails the gate.
// ===========================================================================

#[test]
fn criterion4_write_method_barred_from_prose() {
    // ASSERTS: a mutator `w.changeMoney(5)` cannot be embedded in prose — prose must
    // be side-effect-free; a render must never change the player's money.
    let r = registry_with(&[], &[]);
    let result = validate_prose(r#"{{ w.changeMoney(5) }}"#, &r, "criterion4");
    assert!(
        result.is_err(),
        "write method changeMoney must be rejected in prose, got Ok"
    );
    let msg = format!("{:?}", result.unwrap_err());
    assert!(
        msg.contains("changeMoney"),
        "error should name changeMoney, got: {msg}"
    );
}

// ===========================================================================
// Criterion 5 — m.getName() renders the DISPLAY name, not the spawn name.
// ===========================================================================

#[test]
fn criterion5_npc_getname_renders_display_name() {
    // ASSERTS: when a story assigns an NPC a display name ("Theo"), prose that prints
    // the NPC's name shows "Theo" — NOT the raw spawn name ("Jake"). A regression here
    // means the player sees the wrong character name on screen.
    let mut registry = registry_with(&[], &[]);
    let personality = registry.intern_personality("ROMANTIC");

    let mut world = make_test_world();
    let mut male = make_test_male_npc(personality);
    // make_test_male_npc spawns core.name = "Jake"; the story renames the display name.
    assert_eq!(
        male.core.name, "Jake",
        "fixture precondition: spawn name is Jake"
    );
    male.core.display_name = Some("Theo".to_string());
    let key = world.male_npcs.insert(male);

    let mut ctx = SceneCtx::new();
    ctx.active_male = Some(key);

    let rendered =
        render_prose(r#"{{ m.getName() }}"#, &world, &ctx, &registry).expect("m.getName renders");

    assert_eq!(
        rendered.trim(),
        "Theo",
        "prose must show the effective (display) name, not the spawn name"
    );
    // No-op litmus: explicitly prove the raw spawn name does NOT leak through.
    assert_ne!(
        rendered.trim(),
        "Jake",
        "prose must NOT show the raw spawn name"
    );
}

// ===========================================================================
// Criterion 6 — the entire base pack loads cleanly through the real loader,
// which runs the prose gate on every prose field; the full scene set comes back.
// ===========================================================================

#[test]
fn criterion6_base_pack_loads_cleanly_with_full_scene_set() {
    // ASSERTS: the shipped base pack passes the prose gate end-to-end — no authored
    // scene contains a prose call the gate rejects, and the loader returns the whole
    // scene corpus (not an empty / partial map). If the gate over-rejects real content
    // or the loader silently drops scenes, this breaks.
    let packs = packs_dir();
    assert!(
        packs.join("base").exists(),
        "base pack must exist on disk at {}",
        packs.display()
    );

    let (registry, _meta) = undone_packs::load_packs(&packs).expect("base pack registry must load");

    let scenes_dir = packs.join("base").join("scenes");
    let scenes = undone_scene::loader::load_scenes(&scenes_dir, &registry)
        .expect("base pack scenes must load cleanly through the prose gate");

    // The full scene set: the scenes dir has 75 .toml files on disk; the loader keys
    // by scene id, so we assert a non-trivial corpus came back rather than an empty or
    // single-scene map (which a broken loader could return with Ok).
    let toml_count = std::fs::read_dir(&scenes_dir)
        .expect("read scenes dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("toml"))
        .count();
    assert!(
        toml_count > 0,
        "precondition: base pack must have scene .toml files on disk"
    );
    assert_eq!(
        scenes.len(),
        toml_count,
        "loader must return every scene file as a loaded scene (got {} from {} files)",
        scenes.len(),
        toml_count
    );
}
