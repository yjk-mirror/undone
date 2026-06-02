use std::path::PathBuf;
use std::process;

use undone::story_map::{build_story_map, is_up_to_date, render_json, render_markdown};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let check = args.iter().any(|a| a == "--check");

    let packs_dir = PathBuf::from("packs");
    let md_path = PathBuf::from("docs/story-map.md");
    let json_path = PathBuf::from("docs/story-map.json");

    if check {
        match is_up_to_date(&packs_dir, &md_path, &json_path) {
            Ok(true) => {
                println!("story-map: up to date.");
            }
            Ok(false) => {
                eprintln!(
                    "story-map: STALE. Run `cargo run --bin story-map` and commit the result."
                );
                process::exit(1);
            }
            Err(e) => {
                eprintln!("story-map: {e}");
                process::exit(1);
            }
        }
        return;
    }

    let map = match build_story_map(&packs_dir) {
        Ok(map) => map,
        Err(e) => {
            eprintln!("story-map: {e}");
            process::exit(1);
        }
    };

    let md = render_markdown(&map);
    let json = match render_json(&map) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("story-map: {e}");
            process::exit(1);
        }
    };

    if let Err(e) = std::fs::write(&md_path, md) {
        eprintln!("story-map: write {}: {e}", md_path.display());
        process::exit(1);
    }
    if let Err(e) = std::fs::write(&json_path, json) {
        eprintln!("story-map: write {}: {e}", json_path.display());
        process::exit(1);
    }

    let threads = map.threads.len();
    let orphans = map.orphans.len();
    let writes = map.write_next.len();
    println!(
        "story-map: wrote {} and {} ({threads} threads, {writes} write-next items, {orphans} orphans).",
        md_path.display(),
        json_path.display()
    );
    if orphans > 0 {
        println!(
            "  ⚠ {orphans} orphan scene(s) — add them to a thread in packs/base/roadmap.toml."
        );
    }
}
