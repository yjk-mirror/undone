#[test]
fn runtime_simulation_reports_reachable_and_unreachable_rows() {
    let result = undone::validate_pack::run_simulation_for_tests(4, 3).expect("simulation");
    let stats = result.stats();
    let landlord = stats
        .iter()
        .find(|stat| stat.scene_id == "base::workplace_landlord")
        .expect("simulation should include workplace_landlord");
    assert!(
        landlord.warning.as_deref() != Some("NEVER FIRES"),
        "workplace_landlord should be reachable in runtime-driven simulation: {:?}",
        landlord.warning
    );

    let campus = stats
        .iter()
        .find(|stat| stat.scene_id == "base::campus_arrival")
        .expect("simulation should keep zero-count rows for unreachable scenes");
    assert!(
        campus.warning.as_deref() == Some("NEVER FIRES"),
        "campus_arrival should remain a zero-count unreachable row in the Robin horizon: {:?}",
        campus.warning
    );
}
