use std::process::Command;

#[test]
fn validate_pack_runtime_simulation_reports_reachable_and_unreachable_rows() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "validate-pack",
            "--",
            "--simulate",
            "--weeks",
            "4",
            "--runs",
            "3",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("validate-pack simulation should run");

    assert!(
        output.status.success(),
        "validate-pack simulation failed:\nstdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let landlord_line = stdout
        .lines()
        .find(|line| line.contains("base::workplace_landlord"))
        .expect("output should include workplace_landlord");
    assert!(
        !landlord_line.contains("NEVER FIRES"),
        "workplace_landlord should be reachable in runtime-driven simulation:\n{landlord_line}\n\nfull output:\n{stdout}"
    );

    let campus_line = stdout
        .lines()
        .find(|line| line.contains("base::campus_arrival"))
        .expect("output should keep zero-count rows for unreachable scenes");
    assert!(
        campus_line.contains("NEVER FIRES"),
        "campus_arrival should remain a zero-count unreachable row in the Robin horizon:\n{campus_line}\n\nfull output:\n{stdout}"
    );
}
