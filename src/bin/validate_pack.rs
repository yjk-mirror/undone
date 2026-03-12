use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let simulate = args.iter().any(|arg| arg == "--simulate");
    let weeks: u32 = args
        .iter()
        .position(|arg| arg == "--weeks")
        .and_then(|index| args.get(index + 1))
        .and_then(|value| value.parse().ok())
        .unwrap_or(52);
    let runs: u32 = args
        .iter()
        .position(|arg| arg == "--runs")
        .and_then(|index| args.get(index + 1))
        .and_then(|value| value.parse().ok())
        .unwrap_or(1000);

    let packs_dir = undone::validate_pack::default_packs_dir();
    println!("Loading packs from {:?}", packs_dir);

    let report = match undone::validate_pack::validate_pack_dir(&packs_dir) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    };

    println!("Packs loaded. Loading scenes...");
    for warning in &report.warnings {
        eprintln!("WARN  {warning}");
    }
    for finding in &report.prose_findings {
        let location = finding
            .line
            .map(|line| format!(":{}", line))
            .unwrap_or_default();
        eprintln!(
            "WARN  [prose:{}] {}{}: {}",
            finding.kind, finding.file_path, location, finding.message
        );
    }
    for error in &report.errors {
        eprintln!("{error}");
    }

    if report.has_errors() {
        eprintln!("\n{} error(s) found.", report.error_count());
        process::exit(1);
    }

    println!(
        "\nAll checks passed. {} total scene(s).",
        report.total_scenes
    );

    if !simulate {
        return;
    }

    println!("\nRunning runtime-driven distribution simulation ({weeks} weeks x {runs} runs)...\n");
    let result = match undone::validate_pack::run_simulation_from_dir(&packs_dir, weeks, runs) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    };

    println!("Runtime Scene Distribution ({weeks} weeks x {runs} runs):");
    for stat in result.stats() {
        let warning = stat
            .warning
            .as_ref()
            .map(|value| format!("  ! {value}"))
            .unwrap_or_default();
        println!(
            "  {:<40} - {:>5.1}% (avg {:.1}/run){}",
            stat.scene_id, stat.percentage, stat.avg_per_run, warning
        );
    }
}
