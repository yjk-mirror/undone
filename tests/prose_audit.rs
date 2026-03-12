use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn packs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("packs")
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let dst_path = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir_recursive(&entry.path(), &dst_path);
        } else {
            fs::copy(entry.path(), dst_path).unwrap();
        }
    }
}

fn invalid_prose_pack_dir() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let fixture_root = std::env::temp_dir().join(format!("undone_invalid_prose_pack_{unique}"));
    let fixture_packs_dir = fixture_root.join("packs");
    copy_dir_recursive(&packs_dir(), &fixture_packs_dir);

    let scene_path = fixture_packs_dir
        .join("base")
        .join("scenes")
        .join("weekend_morning.toml");
    let scene = fs::read_to_string(&scene_path).unwrap();
    fs::write(
        &scene_path,
        scene.replace(
            "You stretch beneath the blankets for another minute, eyes closed.",
            "You check your phone and wait for something to happen.",
        ),
    )
    .unwrap();

    fixture_packs_dir
}

#[test]
fn prose_audit_flags_third_person_player_narration() {
    let scene = r#"[scene]
id = "test::scene"
[intro]
prose = "She walks into the room.""#;

    let findings = undone::validate_pack::audit_scene_text("test.toml", scene);
    assert!(findings
        .iter()
        .any(|finding| finding.kind == "third_person_player_narration"));
}

#[test]
fn prose_audit_flags_unnecessary_always_female_guard() {
    let scene = r#"{% if w.alwaysFemale() %}You smooth your skirt.{% endif %}"#;

    let findings = undone::validate_pack::audit_scene_text("guard.toml", scene);
    assert!(findings
        .iter()
        .any(|finding| finding.kind == "unnecessary_always_female_guard"));
}

#[test]
fn prose_audit_flags_filler_action_phrasing() {
    let scene = r#"You check your phone and wait for something to happen."#;

    let findings = undone::validate_pack::audit_scene_text("filler.toml", scene);
    assert!(findings
        .iter()
        .any(|finding| finding.kind == "filler_action"));
}

#[test]
fn prose_audit_flags_meta_analysis_phrasing() {
    let scene = r#"None of this was conscious. You used to do this without thinking."#;

    let findings = undone::validate_pack::audit_scene_text("meta.toml", scene);
    assert!(findings
        .iter()
        .any(|finding| finding.kind == "meta_analysis"));
}

#[test]
fn prose_audit_flags_player_speech_in_intro() {
    let scene = r#"[scene]
id = "test::scene"
pack = "test"
description = "test"

[intro]
prose = """
The man hands you the bag.

"Thanks." You take it and keep moving.
""""#;

    let findings = undone::validate_pack::audit_scene_text("test.toml", scene);
    assert!(
        findings
            .iter()
            .any(|finding| finding.kind == "player_speech_in_intro"),
        "expected player_speech_in_intro finding, got: {:?}",
        findings
    );
}

#[test]
fn prose_audit_flags_player_deliberate_action_in_intro() {
    let scene = r#"[scene]
id = "test::scene"
pack = "test"
description = "test"

[intro]
prose = """
The coffee shop is warm.

You sit down at the counter and order a drink.
""""#;

    let findings = undone::validate_pack::audit_scene_text("test.toml", scene);
    assert!(
        findings
            .iter()
            .any(|finding| finding.kind == "player_action_in_intro"),
        "expected player_action_in_intro finding, got: {:?}",
        findings
    );
}

#[test]
fn prose_audit_does_not_flag_involuntary_body_response_in_intro() {
    let scene = r#"[scene]
id = "test::scene"
pack = "test"
description = "test"

[intro]
prose = """
The room is cold.

Your hands go numb. You feel the weight of it.
""""#;

    let findings = undone::validate_pack::audit_scene_text("test.toml", scene);
    assert!(
        !findings
            .iter()
            .any(|finding| finding.kind == "player_action_in_intro"),
        "involuntary body response should not be flagged, got: {:?}",
        findings
    );
}

#[test]
fn prose_audit_does_not_flag_player_speech_in_action_prose() {
    let scene = r#"[scene]
id = "test::scene"
pack = "test"
description = "test"

[intro]
prose = "The man is waiting."

[[actions]]
id = "greet"
label = "Say hello"
prose = """
"Hey there." You smile.
""""#;

    let findings = undone::validate_pack::audit_scene_text("test.toml", scene);
    assert!(
        !findings
            .iter()
            .any(|finding| finding.kind == "player_speech_in_intro"),
        "player speech in action prose should not be flagged as intro violation, got: {:?}",
        findings
    );
}

#[test]
fn prose_audit_flags_player_action_in_thought_prose() {
    let scene = r#"[scene]
id = "test::scene"
pack = "test"
description = "test"

[intro]
prose = "The room is quiet."

[[thoughts]]
condition = "true"
style = "inner_voice"
prose = "You grab the notebook from the shelf."
"#;

    let findings = undone::validate_pack::audit_scene_text("test.toml", scene);
    assert!(
        findings
            .iter()
            .any(|finding| finding.kind == "player_action_in_intro"),
        "player action in thought prose should be flagged, got: {:?}",
        findings
    );
}

#[test]
fn validate_pack_report_includes_prose_findings() {
    let fixture_packs_dir = invalid_prose_pack_dir();
    let report = undone::validate_pack::validate_pack_dir(&fixture_packs_dir).expect("report");

    assert!(report
        .prose_findings
        .iter()
        .any(|finding| finding.kind == "filler_action"));

    fs::remove_dir_all(fixture_packs_dir.parent().unwrap()).unwrap();
}

#[test]
fn campus_cluster_has_no_third_person_or_unnecessary_guard_findings() {
    let report = undone::validate_pack::validate_repo_scenes_for_tests().expect("audit");

    assert!(!report.has_finding(
        "packs/base/scenes/campus_arrival.toml",
        "third_person_player_narration"
    ));
    assert!(!report.has_finding(
        "packs/base/scenes/campus_call_home.toml",
        "third_person_player_narration"
    ));
    assert!(!report.has_finding(
        "packs/base/scenes/campus_dorm.toml",
        "unnecessary_always_female_guard"
    ));
    assert!(!report.has_finding(
        "packs/base/scenes/campus_orientation.toml",
        "unnecessary_always_female_guard"
    ));
}

#[test]
fn filler_cleanup_cluster_has_no_known_fine_test_phrases() {
    let report = undone::validate_pack::validate_repo_scenes_for_tests().expect("audit");

    assert!(!report.has_finding("packs/base/scenes/weekend_morning.toml", "filler_action"));
    assert!(!report.has_finding("packs/base/scenes/coffee_shop.toml", "filler_action"));
    assert!(!report.has_finding("packs/base/scenes/bookstore.toml", "fine_test_failure"));
    assert!(!report.has_finding("packs/base/scenes/work_friday.toml", "fine_test_failure"));
}

#[test]
fn player_agency_audit_report() {
    let report = undone::validate_pack::validate_repo_scenes_for_tests().expect("audit");
    let agency_findings: Vec<_> = report
        .prose_findings
        .iter()
        .filter(|f| f.kind == "player_speech_in_intro" || f.kind == "player_action_in_intro")
        .collect();

    // Print the report for human review
    for f in &agency_findings {
        eprintln!(
            "[{}] {} (line {:?}): {}",
            f.kind, f.file_path, f.line, f.message
        );
    }

    eprintln!("\nTotal player-agency findings: {}", agency_findings.len());
}

// These tests scope to finding types that were cleaned in prior sessions.
// Player agency findings (player_speech_in_intro, player_action_in_intro)
// are expected to exist until Phase 2 rewrites land.
const PROSE_QUALITY_KINDS: &[&str] = &[
    "third_person_player_narration",
    "unnecessary_always_female_guard",
    "filler_action",
    "meta_analysis",
    "fine_test_failure",
];

#[test]
fn validate_pack_reports_clean_results_for_touched_scene_cluster() {
    let report = undone::validate_pack::validate_repo_scenes_for_tests().expect("audit");

    let findings: Vec<_> = report
        .findings_for_prefix("packs/base/scenes/campus_")
        .into_iter()
        .filter(|f| PROSE_QUALITY_KINDS.contains(&f.kind.as_str()))
        .collect();

    assert!(
        findings.is_empty(),
        "expected cleaned campus cluster to be free of prose quality findings, got: {:?}",
        findings
    );
}

#[test]
fn workplace_opening_spine_has_no_prose_quality_findings() {
    let report = undone::validate_pack::validate_repo_scenes_for_tests().expect("audit");
    let files = [
        "packs/base/scenes/workplace_arrival.toml",
        "packs/base/scenes/workplace_landlord.toml",
        "packs/base/scenes/workplace_first_night.toml",
        "packs/base/scenes/workplace_first_clothes.toml",
        "packs/base/scenes/workplace_first_day.toml",
        "packs/base/scenes/workplace_work_meeting.toml",
        "packs/base/scenes/workplace_evening.toml",
    ];

    let mut findings = Vec::new();
    for file in files {
        findings.extend(
            report
                .prose_findings
                .iter()
                .filter(|f| f.file_path == file && PROSE_QUALITY_KINDS.contains(&f.kind.as_str()))
                .cloned(),
        );
    }

    assert!(
        findings.is_empty(),
        "expected workplace opening spine to be free of prose quality findings, got: {:?}",
        findings
    );
}
