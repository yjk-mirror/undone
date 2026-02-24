use std::path::PathBuf;

use undone_packs::load_packs;
use undone_scene::{
    loader::{load_scenes, validate_cross_references},
    types::EffectDef,
};

fn main() {
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
                    // Warn: scene has no lasting effects
                    let has_lasting = scene.actions.iter().any(|a| {
                        a.effects.iter().any(|e| {
                            matches!(
                                e,
                                EffectDef::SetGameFlag { .. }
                                    | EffectDef::AddNpcLiking { .. }
                                    | EffectDef::AdvanceArc { .. }
                            )
                        })
                    });
                    if !has_lasting {
                        eprintln!(
                            "WARN  [{}] no lasting effects (game flag, NPC liking, or arc advance)",
                            id
                        );
                    }
                }
                all_scenes.extend(scenes);
            }
            Err(e) => {
                eprintln!("ERROR loading scenes for '{}': {e}", meta.manifest.pack.id);
                error_count += 1;
            }
        }
    }

    // Cross-reference check: all goto targets must exist
    if let Err(e) = validate_cross_references(&all_scenes) {
        eprintln!("ERROR cross-reference: {e}");
        error_count += 1;
    }

    if error_count > 0 {
        eprintln!("\n{error_count} error(s) found.");
        std::process::exit(1);
    } else {
        println!("\nAll checks passed. {} total scene(s).", all_scenes.len());
    }
}
