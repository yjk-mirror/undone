use std::collections::HashSet;
use std::path::{Path, PathBuf};

use undone::story_map::{build_story_map, is_up_to_date, render_json};

fn packs_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("packs")
}

#[test]
fn builds_on_real_base_pack() {
    // BREAKS IF: the tool can't process the shipping content.
    let map = build_story_map(&packs_dir()).expect("story map should build on base pack");
    assert!(!map.threads.is_empty(), "expected declared threads");
}

#[test]
fn thread_assignment_is_a_true_partition() {
    // BREAKS IF: a scene gets claimed by two threads (assignment double-counts).
    let map = build_story_map(&packs_dir()).unwrap();
    let mut seen: HashSet<String> = HashSet::new();
    for thread in &map.threads {
        for node in &thread.scenes {
            assert!(
                seen.insert(node.id.clone()),
                "scene '{}' appears in more than one thread",
                node.id
            );
        }
    }
}

#[test]
fn every_scene_is_claimed_no_orphans() {
    // BREAKS IF: a non-archived scene is in no roadmap thread. FIX: add its short
    // id to the right thread's `scenes` list in packs/base/roadmap.toml.
    let map = build_story_map(&packs_dir()).unwrap();
    assert!(
        map.orphans.is_empty(),
        "unclaimed scenes (add to packs/base/roadmap.toml): {:?}",
        map.orphans
    );
}

#[test]
fn json_sidecar_roundtrips() {
    // BREAKS IF: the JSON schema stops being valid/parseable for agents.
    let map = build_story_map(&packs_dir()).unwrap();
    let json = render_json(&map).unwrap();
    let _: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
}

#[test]
fn committed_map_is_up_to_date() {
    // BREAKS IF: docs/story-map.{md,json} drift from the content. FIX: rerun
    // `cargo run --bin story-map` and commit.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fresh = is_up_to_date(
        &packs_dir(),
        &root.join("docs/story-map.md"),
        &root.join("docs/story-map.json"),
    )
    .unwrap();
    assert!(
        fresh,
        "regenerate with `cargo run --bin story-map` and commit"
    );
}
