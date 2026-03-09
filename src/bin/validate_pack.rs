use std::path::PathBuf;

use rand::{rngs::SmallRng, SeedableRng};
use undone_packs::load_packs;
use undone_scene::{
    loader::{load_scenes, validate_cross_references},
    scheduler::validate_entry_scene_references,
};

fn extend_scenes_checked(
    all_scenes: &mut std::collections::HashMap<
        String,
        std::sync::Arc<undone_scene::types::SceneDefinition>,
    >,
    scene_sources: &mut std::collections::HashMap<String, String>,
    incoming: std::collections::HashMap<
        String,
        std::sync::Arc<undone_scene::types::SceneDefinition>,
    >,
    source: &str,
) -> Result<(), String> {
    for (scene_id, scene) in incoming {
        if let Some(first_source) = scene_sources.insert(scene_id.clone(), source.to_string()) {
            return Err(format!(
                "duplicate scene id '{scene_id}': '{source}' conflicts with already-loaded '{first_source}'"
            ));
        }
        all_scenes.insert(scene_id, scene);
    }
    Ok(())
}

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

    let packs_dir = PathBuf::from("packs");
    println!("Loading packs from {:?}", packs_dir);

    let (registry, pack_metas) = match load_packs(&packs_dir) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("FATAL: pack load failed: {e}");
            std::process::exit(1);
        }
    };

    println!("Packs loaded. Loading scenes...");
    let mut error_count = 0;
    let mut all_scenes = std::collections::HashMap::new();
    let mut scene_sources = std::collections::HashMap::new();

    for meta in &pack_metas {
        let scenes_dir = meta.pack_dir.join(&meta.manifest.content.scenes_dir);
        match load_scenes(&scenes_dir, &registry) {
            Ok(scenes) => {
                println!(
                    "  {} scene(s) loaded from pack '{}'",
                    scenes.len(),
                    meta.manifest.pack.id
                );
                for (id, scene) in &scenes {
                    if !scene.has_persistent_world_mutation() {
                        eprintln!(
                            "WARN  [{}] no persistent world mutation (scene-local flags and navigation do not count)",
                            id
                        );
                    }
                }
                if let Err(e) = extend_scenes_checked(
                    &mut all_scenes,
                    &mut scene_sources,
                    scenes,
                    &meta.manifest.pack.id,
                ) {
                    eprintln!("ERROR loading scenes for '{}': {e}", meta.manifest.pack.id);
                    error_count += 1;
                }
            }
            Err(e) => {
                eprintln!("ERROR loading scenes for '{}': {e}", meta.manifest.pack.id);
                error_count += 1;
            }
        }
    }

    let scheduler = match undone_scene::load_schedule(&pack_metas, &registry) {
        Ok(scheduler) => {
            if let Err(e) = scheduler.validate_scene_references(&all_scenes) {
                eprintln!("ERROR schedule validation: {e}");
                error_count += 1;
            }
            if let Err(e) = validate_entry_scene_references(
                &all_scenes,
                registry.opening_scene(),
                registry.transformation_scene(),
            ) {
                eprintln!("ERROR entry scene validation: {e}");
                error_count += 1;
            }
            Some(scheduler)
        }
        Err(e) => {
            eprintln!("ERROR loading schedule: {e}");
            error_count += 1;
            None
        }
    };

    // Trait conflict validation
    let conflict_errors = registry.validate_trait_conflicts();
    if !conflict_errors.is_empty() {
        for e in &conflict_errors {
            eprintln!("  ERROR: {e}");
        }
        error_count += conflict_errors.len();
    }

    // Cross-reference check: all goto targets must exist
    if let Err(e) = validate_cross_references(&all_scenes) {
        eprintln!("ERROR cross-reference: {e}");
        error_count += 1;
    }

    if let Some(ref scheduler) = scheduler {
        let warnings = undone_scene::reachability::check_reachability(
            &scheduler.all_conditions(),
            &all_scenes,
        );
        for warning in warnings {
            eprintln!(
                "WARN  [reachability] {}: {}",
                warning.context, warning.message
            );
        }

        let char_creation_errors =
            undone_ui::char_creation::validate_runtime_contract(&registry, scheduler);
        for error in char_creation_errors {
            eprintln!("ERROR char creation contract: {error}");
            error_count += 1;
        }
    }

    if error_count > 0 {
        eprintln!("\n{error_count} error(s) found.");
        std::process::exit(1);
    } else {
        println!("\nAll checks passed. {} total scene(s).", all_scenes.len());
    }

    if simulate {
        let Some(ref scheduler) = scheduler else {
            eprintln!("Simulation skipped: scheduler not available.");
            std::process::exit(1);
        };

        println!("\nRunning distribution simulation ({weeks} weeks x {runs} runs)...\n");

        let mut sim_registry = registry.clone();
        let config = undone_ui::char_creation::robin_quick_config(&sim_registry);
        let mut sim_rng = SmallRng::seed_from_u64(42);
        let world = undone_packs::char_creation::new_game(config, &mut sim_registry, &mut sim_rng);

        let result = undone_scene::simulator::simulate(
            scheduler,
            &registry,
            &world,
            undone_scene::simulator::SimulationConfig {
                weeks,
                runs,
                seed: 42,
            },
        );

        println!("Scene Distribution ({weeks} weeks x {runs} runs):");
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
}
