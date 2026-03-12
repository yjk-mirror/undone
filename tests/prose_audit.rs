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
fn validate_pack_reports_clean_results_for_touched_scene_cluster() {
    let report = undone::validate_pack::validate_repo_scenes_for_tests().expect("audit");

    assert!(
        report
            .findings_for_prefix("packs/base/scenes/campus_")
            .is_empty(),
        "expected cleaned campus cluster to be free of audit findings, got: {:?}",
        report.findings_for_prefix("packs/base/scenes/campus_")
    );
}
