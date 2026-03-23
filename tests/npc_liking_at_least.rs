// Independent tests for the npcLikingAtLeast feature, bounds-checked NPC action
// indexing, and prose audit "She" fix.
//
// Written by independent test author — not the implementer.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use lasso::Key;

// ---------------------------------------------------------------------------
// 1. npcLikingAtLeast expression evaluation
// ---------------------------------------------------------------------------

/// Helper: parse + eval a boolean expression against a world + registry.
fn eval_bool(expr_str: &str, world: &undone_world::World, reg: &undone_packs::PackRegistry) -> bool {
    let expr = undone_expr::parse(expr_str).expect("parse should succeed");
    let ctx = undone_expr::SceneCtx::new();
    undone_expr::eval(&expr, world, &ctx, reg).expect("eval should succeed")
}

/// Build a minimal MaleNpc with a given role and pc_liking level.
fn male_npc_with_role_and_liking(
    name: &str,
    role: &str,
    pc_liking: undone_domain::LikingLevel,
    reg: &mut undone_packs::PackRegistry,
) -> undone_domain::MaleNpc {
    let personality = reg.intern_personality("STUBBORN");
    undone_domain::MaleNpc {
        core: undone_domain::NpcCore {
            name: name.into(),
            age: undone_domain::Age::Thirties,
            race: "white".into(),
            eye_colour: "grey".into(),
            hair_colour: "black".into(),
            personality,
            traits: HashSet::new(),
            relationship: undone_domain::RelationshipStatus::Acquaintance,
            pc_liking,
            npc_liking: undone_domain::LikingLevel::Neutral,
            pc_love: undone_domain::LoveLevel::None,
            npc_love: undone_domain::LoveLevel::None,
            pc_attraction: undone_domain::AttractionLevel::Unattracted,
            npc_attraction: undone_domain::AttractionLevel::Unattracted,
            behaviour: undone_domain::Behaviour::Neutral,
            relationship_flags: HashSet::new(),
            sexual_activities: HashSet::new(),
            custom_flags: HashMap::new(),
            custom_ints: HashMap::new(),
            knowledge: 0,
            roles: HashSet::from([role.to_string()]),
            contactable: false,
            arousal: undone_domain::ArousalLevel::Comfort,
            alcohol: undone_domain::AlcoholLevel::Sober,
        },
        figure: undone_domain::MaleFigure::Average,
        clothing: undone_domain::MaleClothing::default(),
        had_orgasm: false,
        has_baby_with_pc: false,
    }
}

/// Build a minimal FemaleNpc with a given role and pc_liking level.
fn female_npc_with_role_and_liking(
    name: &str,
    role: &str,
    pc_liking: undone_domain::LikingLevel,
    reg: &mut undone_packs::PackRegistry,
) -> undone_domain::FemaleNpc {
    let personality = reg.intern_personality("CARING");
    undone_domain::FemaleNpc {
        core: undone_domain::NpcCore {
            name: name.into(),
            age: undone_domain::Age::EarlyTwenties,
            race: "white".into(),
            eye_colour: "green".into(),
            hair_colour: "blonde".into(),
            personality,
            traits: HashSet::new(),
            relationship: undone_domain::RelationshipStatus::Acquaintance,
            pc_liking,
            npc_liking: undone_domain::LikingLevel::Neutral,
            pc_love: undone_domain::LoveLevel::None,
            npc_love: undone_domain::LoveLevel::None,
            pc_attraction: undone_domain::AttractionLevel::Unattracted,
            npc_attraction: undone_domain::AttractionLevel::Unattracted,
            behaviour: undone_domain::Behaviour::Neutral,
            relationship_flags: HashSet::new(),
            sexual_activities: HashSet::new(),
            custom_flags: HashMap::new(),
            custom_ints: HashMap::new(),
            knowledge: 0,
            roles: HashSet::from([role.to_string()]),
            contactable: false,
            arousal: undone_domain::ArousalLevel::Comfort,
            alcohol: undone_domain::AlcoholLevel::Sober,
        },
        char_type: undone_domain::CharTypeId::from_spur(lasso::Spur::try_from_usize(0).unwrap()),
        figure: undone_domain::PlayerFigure::Slim,
        breasts: undone_domain::BreastSize::Average,
        clothing: undone_domain::FemaleClothing::default(),
        pregnancy: None,
        virgin: true,
    }
}

// ── Happy path: NPC at each liking level ──────────────────────────────────

#[test]
// BREAKS IF: npcLikingAtLeast returns wrong result for Neutral-level NPC checked against all thresholds
fn npc_at_neutral_passes_neutral_fails_all_higher() {
    let mut reg = undone_packs::PackRegistry::new();
    let mut world = undone_world::test_helpers::make_test_world();
    let npc = male_npc_with_role_and_liking("Hank", "ROLE_HANK", undone_domain::LikingLevel::Neutral, &mut reg);
    world.male_npcs.insert(npc);

    assert!(eval_bool("gd.npcLikingAtLeast('ROLE_HANK', 'Neutral')", &world, &reg));
    assert!(!eval_bool("gd.npcLikingAtLeast('ROLE_HANK', 'Ok')", &world, &reg));
    assert!(!eval_bool("gd.npcLikingAtLeast('ROLE_HANK', 'Like')", &world, &reg));
    assert!(!eval_bool("gd.npcLikingAtLeast('ROLE_HANK', 'Close')", &world, &reg));
}

#[test]
// BREAKS IF: npcLikingAtLeast returns wrong result for Ok-level NPC
fn npc_at_ok_passes_neutral_and_ok_fails_higher() {
    let mut reg = undone_packs::PackRegistry::new();
    let mut world = undone_world::test_helpers::make_test_world();
    let npc = male_npc_with_role_and_liking("Sam", "ROLE_SAM", undone_domain::LikingLevel::Ok, &mut reg);
    world.male_npcs.insert(npc);

    assert!(eval_bool("gd.npcLikingAtLeast('ROLE_SAM', 'Neutral')", &world, &reg));
    assert!(eval_bool("gd.npcLikingAtLeast('ROLE_SAM', 'Ok')", &world, &reg));
    assert!(!eval_bool("gd.npcLikingAtLeast('ROLE_SAM', 'Like')", &world, &reg));
    assert!(!eval_bool("gd.npcLikingAtLeast('ROLE_SAM', 'Close')", &world, &reg));
}

#[test]
// BREAKS IF: npcLikingAtLeast returns wrong result for Like-level NPC
fn npc_at_like_passes_neutral_ok_like_fails_close() {
    let mut reg = undone_packs::PackRegistry::new();
    let mut world = undone_world::test_helpers::make_test_world();
    let npc = male_npc_with_role_and_liking("Tim", "ROLE_TIM", undone_domain::LikingLevel::Like, &mut reg);
    world.male_npcs.insert(npc);

    assert!(eval_bool("gd.npcLikingAtLeast('ROLE_TIM', 'Neutral')", &world, &reg));
    assert!(eval_bool("gd.npcLikingAtLeast('ROLE_TIM', 'Ok')", &world, &reg));
    assert!(eval_bool("gd.npcLikingAtLeast('ROLE_TIM', 'Like')", &world, &reg));
    assert!(!eval_bool("gd.npcLikingAtLeast('ROLE_TIM', 'Close')", &world, &reg));
}

#[test]
// BREAKS IF: npcLikingAtLeast returns false for Close-level NPC even at highest threshold
fn npc_at_close_passes_all_levels() {
    let mut reg = undone_packs::PackRegistry::new();
    let mut world = undone_world::test_helpers::make_test_world();
    let npc = male_npc_with_role_and_liking("Val", "ROLE_VAL", undone_domain::LikingLevel::Close, &mut reg);
    world.male_npcs.insert(npc);

    assert!(eval_bool("gd.npcLikingAtLeast('ROLE_VAL', 'Neutral')", &world, &reg));
    assert!(eval_bool("gd.npcLikingAtLeast('ROLE_VAL', 'Ok')", &world, &reg));
    assert!(eval_bool("gd.npcLikingAtLeast('ROLE_VAL', 'Like')", &world, &reg));
    assert!(eval_bool("gd.npcLikingAtLeast('ROLE_VAL', 'Close')", &world, &reg));
}

// ── Missing role defaults to Neutral ──────────────────────────────────────

#[test]
// BREAKS IF: missing NPC role does not default to Neutral — schedule conditions for
// unknown roles would return wrong results
fn missing_role_defaults_to_neutral() {
    let reg = undone_packs::PackRegistry::new();
    let world = undone_world::test_helpers::make_test_world();

    // Neutral >= Neutral is true
    assert!(eval_bool("gd.npcLikingAtLeast('ROLE_GHOST', 'Neutral')", &world, &reg));
    // Neutral >= Ok is false
    assert!(!eval_bool("gd.npcLikingAtLeast('ROLE_GHOST', 'Ok')", &world, &reg));
    // Neutral >= Like is false
    assert!(!eval_bool("gd.npcLikingAtLeast('ROLE_GHOST', 'Like')", &world, &reg));
    // Neutral >= Close is false
    assert!(!eval_bool("gd.npcLikingAtLeast('ROLE_GHOST', 'Close')", &world, &reg));
}

// ── Female NPC role lookup ────────────────────────────────────────────────

#[test]
// BREAKS IF: find_npc_liking_by_role only searches male NPCs and misses female NPCs
fn female_npc_role_is_found_by_npc_liking_at_least() {
    let mut reg = undone_packs::PackRegistry::new();
    let mut world = undone_world::test_helpers::make_test_world();
    let npc = female_npc_with_role_and_liking("Lisa", "ROLE_LISA", undone_domain::LikingLevel::Like, &mut reg);
    world.female_npcs.insert(npc);

    assert!(eval_bool("gd.npcLikingAtLeast('ROLE_LISA', 'Ok')", &world, &reg));
    assert!(eval_bool("gd.npcLikingAtLeast('ROLE_LISA', 'Like')", &world, &reg));
    assert!(!eval_bool("gd.npcLikingAtLeast('ROLE_LISA', 'Close')", &world, &reg));
}

// ── Error paths ───────────────────────────────────────────────────────────

#[test]
// BREAKS IF: invalid liking level string is silently accepted instead of returning error
fn invalid_liking_level_returns_error() {
    let reg = undone_packs::PackRegistry::new();
    let world = undone_world::test_helpers::make_test_world();
    let ctx = undone_expr::SceneCtx::new();

    // "close" lowercase should fail — only "Close" is valid
    let expr = undone_expr::parse("gd.npcLikingAtLeast('ROLE_X', 'close')").unwrap();
    let result = undone_expr::eval(&expr, &world, &ctx, &reg);
    assert!(result.is_err(), "lowercase 'close' should be rejected as an invalid liking level");

    // Completely bogus level
    let expr2 = undone_expr::parse("gd.npcLikingAtLeast('ROLE_X', 'BestFriends')").unwrap();
    let result2 = undone_expr::eval(&expr2, &world, &ctx, &reg);
    assert!(result2.is_err(), "'BestFriends' should be rejected as an invalid liking level");
}

#[test]
// BREAKS IF: empty string liking level silently evaluates instead of returning error
fn empty_liking_level_returns_error() {
    let reg = undone_packs::PackRegistry::new();
    let world = undone_world::test_helpers::make_test_world();
    let ctx = undone_expr::SceneCtx::new();

    let expr = undone_expr::parse("gd.npcLikingAtLeast('ROLE_X', '')").unwrap();
    let result = undone_expr::eval(&expr, &world, &ctx, &reg);
    assert!(result.is_err(), "empty string should be rejected as an invalid liking level");
}

// ---------------------------------------------------------------------------
// 2. Condition validator: npcLikingAtLeast arity checking
// ---------------------------------------------------------------------------

fn packs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("packs")
}

fn temp_scene_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("undone_npcliking_test_{prefix}_{unique}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_scene_with_condition(dir: &PathBuf, scene_id: &str, filename: &str, condition: &str) {
    let toml_content = format!(
        r#"[scene]
id = "{scene_id}"
pack = "test"
description = "test scene"

[intro]
prose = "Test prose."

[[actions]]
id = "act"
label = "Do something"
condition = "{condition}"
prose = "You do it."
"#
    );
    fs::write(dir.join(format!("{filename}.toml")), toml_content).unwrap();
}

#[test]
// BREAKS IF: condition validator rejects valid npcLikingAtLeast with 2 string args
fn validator_accepts_npc_liking_at_least_with_two_string_args() {
    let dir = temp_scene_dir("valid_2args");
    let (registry, _) = undone_packs::load_packs(&packs_dir()).unwrap();
    write_scene_with_condition(&dir, "test::valid_2args", "valid_2args", "gd.npcLikingAtLeast('ROLE_MARCUS', 'Ok')");

    let result = undone_scene::load_scenes(&dir, &registry);
    assert!(result.is_ok(), "npcLikingAtLeast with 2 string args should be accepted, got: {:?}", result.err());

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
// BREAKS IF: condition validator accepts npcLikingAtLeast with 0 args (wrong arity)
fn validator_rejects_npc_liking_at_least_with_zero_args() {
    let dir = temp_scene_dir("invalid_0args");
    let (registry, _) = undone_packs::load_packs(&packs_dir()).unwrap();
    write_scene_with_condition(&dir, "test::invalid_0", "invalid_0", "gd.npcLikingAtLeast()");

    let result = undone_scene::load_scenes(&dir, &registry);
    assert!(result.is_err(), "npcLikingAtLeast with 0 args should be rejected");

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
// BREAKS IF: condition validator accepts npcLikingAtLeast with 1 arg (wrong arity)
fn validator_rejects_npc_liking_at_least_with_one_arg() {
    let dir = temp_scene_dir("invalid_1arg");
    let (registry, _) = undone_packs::load_packs(&packs_dir()).unwrap();
    write_scene_with_condition(&dir, "test::invalid_1", "invalid_1", "gd.npcLikingAtLeast('ROLE_MARCUS')");

    let result = undone_scene::load_scenes(&dir, &registry);
    assert!(result.is_err(), "npcLikingAtLeast with 1 arg should be rejected");

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
// BREAKS IF: condition validator accepts npcLikingAtLeast with 3 args (wrong arity)
fn validator_rejects_npc_liking_at_least_with_three_args() {
    let dir = temp_scene_dir("invalid_3args");
    let (registry, _) = undone_packs::load_packs(&packs_dir()).unwrap();
    write_scene_with_condition(&dir, "test::invalid_3", "invalid_3", "gd.npcLikingAtLeast('ROLE_MARCUS', 'Ok', 'extra')");

    let result = undone_scene::load_scenes(&dir, &registry);
    assert!(result.is_err(), "npcLikingAtLeast with 3 args should be rejected");

    fs::remove_dir_all(&dir).unwrap();
}

// ---------------------------------------------------------------------------
// 3. Prose audit: "She" at line start NOT flagged
// ---------------------------------------------------------------------------

#[test]
// BREAKS IF: prose audit wrongly flags NPC "She" references as third-person player narration
fn audit_does_not_flag_she_at_line_start_as_third_person() {
    let scene = r#"[scene]
id = "test::she_npc"
[intro]
prose = """
She walks into the room.
She smiles at you from across the bar.
She picks up the glass and hands it to you.
""""#;

    let findings = undone::validate_pack::audit_scene_text("test_she.toml", scene);
    let third_person_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.kind == "third_person_player_narration")
        .collect();
    assert!(
        third_person_findings.is_empty(),
        "Lines starting with 'She' are NPC references — should not be flagged as third-person player narration, got: {:?}",
        third_person_findings
    );
}

#[test]
// BREAKS IF: prose audit stops detecting filler_action after the She fix
fn audit_still_detects_filler_action_after_she_fix() {
    let scene = r#"You check your phone and wait for something to happen."#;

    let findings = undone::validate_pack::audit_scene_text("filler_check.toml", scene);
    assert!(
        findings.iter().any(|f| f.kind == "filler_action"),
        "filler_action detection should still work, got: {:?}",
        findings
    );
}

#[test]
// BREAKS IF: prose audit stops detecting unnecessary_always_female_guard after the She fix
fn audit_still_detects_always_female_guard_after_she_fix() {
    let scene = r#"{% if w.alwaysFemale() %}She smooths her skirt.{% endif %}"#;

    let findings = undone::validate_pack::audit_scene_text("guard_check.toml", scene);
    assert!(
        findings.iter().any(|f| f.kind == "unnecessary_always_female_guard"),
        "unnecessary_always_female_guard detection should still work, got: {:?}",
        findings
    );
}

#[test]
// BREAKS IF: prose audit stops detecting meta_analysis patterns after the She fix
fn audit_still_detects_meta_analysis_after_she_fix() {
    let scene = r#"None of this was conscious. You used to do this every morning."#;

    let findings = undone::validate_pack::audit_scene_text("meta_check.toml", scene);
    assert!(
        findings.iter().any(|f| f.kind == "meta_analysis"),
        "meta_analysis detection should still work, got: {:?}",
        findings
    );
}

#[test]
// BREAKS IF: audit produces false positives on scene with only NPC "She" lines and no issues
fn audit_scene_with_only_she_npc_lines_is_clean() {
    let scene = r#"[scene]
id = "test::clean_she"
pack = "test"
description = "test"

[intro]
prose = """
She reaches across the table.
She's already ordered.
She turns the page.
""""#;

    let findings = undone::validate_pack::audit_scene_text("clean_she.toml", scene);
    // Filter out player_action_in_intro which is a separate check
    let non_agency_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.kind != "player_action_in_intro" && f.kind != "player_speech_in_intro")
        .collect();
    assert!(
        non_agency_findings.is_empty(),
        "scene with only NPC 'She' lines should have no prose quality findings, got: {:?}",
        non_agency_findings
    );
}

// ---------------------------------------------------------------------------
// 4. Content: schedule and scenes use npcLikingAtLeast correctly
// ---------------------------------------------------------------------------

#[test]
// BREAKS IF: validate-pack reports errors — meaning content uses invalid expressions
fn validate_pack_reports_zero_errors() {
    let report = undone::validate_pack::validate_repo_scenes_for_tests().expect("validation report");
    assert!(
        !report.has_errors(),
        "validate-pack should report 0 errors, got {} errors: {:?}",
        report.error_count(),
        report.errors
    );
}

#[test]
// BREAKS IF: schedule still uses old == 'Ok' / == 'Like' checks instead of npcLikingAtLeast
fn schedule_uses_npc_liking_at_least_for_marcus_and_jake() {
    let schedule_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("packs")
        .join("base")
        .join("data")
        .join("schedule.toml");
    let schedule_text = fs::read_to_string(&schedule_path).expect("schedule.toml should exist");

    // work_marcus_favor and work_marcus_late should use npcLikingAtLeast('ROLE_MARCUS', 'Ok')
    assert!(
        schedule_text.contains("npcLikingAtLeast('ROLE_MARCUS', 'Ok')"),
        "schedule should contain npcLikingAtLeast('ROLE_MARCUS', 'Ok') for Marcus favor/late scenes"
    );

    // work_marcus_drinks should use npcLikingAtLeast('ROLE_MARCUS', 'Like')
    assert!(
        schedule_text.contains("npcLikingAtLeast('ROLE_MARCUS', 'Like')"),
        "schedule should contain npcLikingAtLeast('ROLE_MARCUS', 'Like') for Marcus drinks scene"
    );

    // jake_first_date should use npcLikingAtLeast('ROLE_JAKE', 'Like')
    assert!(
        schedule_text.contains("npcLikingAtLeast('ROLE_JAKE', 'Like')"),
        "schedule should contain npcLikingAtLeast('ROLE_JAKE', 'Like') for Jake first date"
    );
}

#[test]
// BREAKS IF: schedule still has old-style exact equality checks for NPC liking
fn schedule_does_not_use_exact_equality_for_npc_liking() {
    let schedule_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("packs")
        .join("base")
        .join("data")
        .join("schedule.toml");
    let schedule_text = fs::read_to_string(&schedule_path).expect("schedule.toml should exist");

    // Check that old patterns like npcLiking('ROLE_MARCUS') == 'Ok' are not present.
    // The npcLiking('role') function still exists for equality checks, but schedule
    // conditions that gate on minimum liking should use npcLikingAtLeast instead.
    let has_old_marcus_equality = schedule_text.contains("npcLiking('ROLE_MARCUS') == 'Ok'")
        || schedule_text.contains("npcLiking('ROLE_MARCUS') == 'Like'");
    assert!(
        !has_old_marcus_equality,
        "schedule should not have old-style npcLiking() == equality checks for Marcus"
    );

    let has_old_jake_equality = schedule_text.contains("npcLiking('ROLE_JAKE') == 'Like'");
    assert!(
        !has_old_jake_equality,
        "schedule should not have old-style npcLiking() == equality checks for Jake"
    );
}

#[test]
// BREAKS IF: scene files that use npcLikingAtLeast in action conditions fail to load
fn scene_files_with_npc_liking_at_least_load_successfully() {
    let (registry, _) = undone_packs::load_packs(&packs_dir()).unwrap();
    let scenes_dir = packs_dir().join("base").join("scenes");
    let scenes = undone_scene::load_scenes(&scenes_dir, &registry)
        .expect("all scenes including those with npcLikingAtLeast should load");

    // Verify specific scenes that use npcLikingAtLeast exist and loaded
    assert!(
        scenes.contains_key("base::work_friday"),
        "work_friday scene should load successfully"
    );
    assert!(
        scenes.contains_key("base::coffee_shop_return"),
        "coffee_shop_return scene should load successfully"
    );
    assert!(
        scenes.contains_key("base::jake_outside"),
        "jake_outside scene should load successfully"
    );
}

// ---------------------------------------------------------------------------
// 5. LikingLevel ordering verification (structural)
// ---------------------------------------------------------------------------

#[test]
// BREAKS IF: LikingLevel enum ordering does not follow Neutral < Ok < Like < Close
fn liking_level_ordering_is_neutral_ok_like_close() {
    use undone_domain::LikingLevel;

    // Verify the ordering explicitly — this is the foundation of npcLikingAtLeast
    assert!(LikingLevel::Neutral < LikingLevel::Ok);
    assert!(LikingLevel::Ok < LikingLevel::Like);
    assert!(LikingLevel::Like < LikingLevel::Close);

    // Verify >= semantics used by npcLikingAtLeast
    assert!(LikingLevel::Like >= LikingLevel::Ok);
    assert!(LikingLevel::Like >= LikingLevel::Neutral);
    assert!(LikingLevel::Like >= LikingLevel::Like);
    assert!(!(LikingLevel::Like >= LikingLevel::Close));
}
