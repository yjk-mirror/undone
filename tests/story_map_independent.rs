//! Independent acceptance tests for the `story-map` authoring tool.
//!
//! Author did NOT write the tool. These tests verify the INTENDED behavior of
//! `undone::story_map` against the REAL base pack, picking concrete examples by
//! reading the real scene/preset data. They do not reuse the in-crate unit-test
//! fixtures or the existing `tests/story_map_acceptance.rs` assertions.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use undone::story_map::{build_story_map, is_up_to_date, render_json, render_markdown, StoryMap};

fn packs_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("packs")
}

fn build() -> StoryMap {
    build_story_map(&packs_dir()).expect("build_story_map must succeed on the real base pack")
}

// ---- Criterion 1: builds successfully and returns at least one thread ----
#[test]
fn builds_on_real_pack_with_threads() {
    // BREAKS IF: the tool fails to load the real base pack, or returns no threads
    // (writers get an empty map and learn nothing about what to write).
    let map = build();
    assert!(
        !map.threads.is_empty(),
        "real base pack must produce at least one thread"
    );
    // No-op litmus: a `return StoryMap::default()` would have zero threads.
    assert!(
        map.threads.len() >= 5,
        "expected the several roadmap threads, got {}",
        map.threads.len()
    );
}

// ---- Criterion 2: total partition (no orphans, no double-claim) ----
#[test]
fn every_scene_claimed_by_exactly_one_thread() {
    // BREAKS IF: a shipping scene is orphaned (claimed by no thread) or appears in
    // two threads — either way the "what to write next" partition is wrong.
    let map = build();
    assert!(
        map.orphans.is_empty(),
        "orphans must be empty; found: {:?}",
        map.orphans
    );

    let mut seen: HashSet<String> = HashSet::new();
    let mut dupes: Vec<String> = Vec::new();
    for t in &map.threads {
        for s in &t.scenes {
            if !seen.insert(s.id.clone()) {
                dupes.push(s.id.clone());
            }
        }
    }
    assert!(
        dupes.is_empty(),
        "no scene id may appear in two threads; duplicates: {dupes:?}"
    );
    // No-op litmus: an empty map would trivially pass the dupe check, so also
    // assert real scenes were partitioned.
    assert!(seen.len() >= 20, "expected many scenes, got {}", seen.len());
}

// ---- Criterion 3: produces — setGameFlag / advanceArc surface in `produces` ----
#[test]
fn produces_lists_flag_and_arc_signals() {
    // BREAKS IF: produced-signal extraction misses setGameFlag/advanceArc. Verified
    // against real data: workplace_first_day sets STARTED_JOB and advances
    // base::workplace_opening=working (read directly from the scene file).
    let map = build();
    let node = map
        .threads
        .iter()
        .flat_map(|t| &t.scenes)
        .find(|s| s.id == "workplace_first_day")
        .expect("workplace_first_day must be present in some thread");

    assert!(
        node.produces.contains(&"STARTED_JOB".to_string()),
        "workplace_first_day must list STARTED_JOB in produces; got {:?}",
        node.produces
    );
    assert!(
        node.produces
            .contains(&"base::workplace_opening=working".to_string()),
        "workplace_first_day must list the advanced arc signal; got {:?}",
        node.produces
    );
}

// ---- Criterion 4: dangling correctness — action-condition consumption counts ----
#[test]
fn flag_read_only_by_action_condition_is_not_dangling() {
    // BREAKS IF: STARTED_JOB (set by workplace_first_day, read by an ACTION-level
    // choice condition in plan_your_day — not a scene-entry gate) is mis-reported as
    // dangling because only entry gates were scanned for consumers. A false dangling
    // tells a writer to write a follow-up scene that already exists.
    let map = build();

    let dangling_signals: Vec<&str> = map
        .threads
        .iter()
        .flat_map(|t| &t.dangling)
        .map(|d| d.signal.as_str())
        .collect();
    assert!(
        !dangling_signals.contains(&"STARTED_JOB"),
        "STARTED_JOB is consumed by an action condition in plan_your_day and must \
         NOT be dangling; dangling list: {dangling_signals:?}"
    );

    let write_next_dangling: Vec<&str> = map
        .write_next
        .iter()
        .filter(|w| w.kind == "dangling")
        .map(|w| w.detail.as_str())
        .collect();
    assert!(
        !write_next_dangling.iter().any(|d| d.contains("STARTED_JOB")),
        "STARTED_JOB must not appear as a dangling write-next item; got {write_next_dangling:?}"
    );
}

// ---- Criterion 5: broken-gate correctness — preset starting flags are producible ----
#[test]
fn gates_on_preset_starting_flags_are_not_broken() {
    // BREAKS IF: a gate on a preset-seeded starting flag (ROUTE_WORKPLACE from
    // 01-robin.toml, ROUTE_CAMPUS from 02-camila.toml) is reported broken because
    // preset starting_flags were not folded into the producible set. That would
    // flood the map with false "write a producer" findings.
    let map = build();

    let broken_route: Vec<String> = map
        .threads
        .iter()
        .flat_map(|t| &t.broken)
        .filter(|b| b.missing.starts_with("ROUTE_"))
        .map(|b| format!("{} gates on {}", b.scene, b.missing))
        .collect();
    assert!(
        broken_route.is_empty(),
        "no gate on a ROUTE_* preset starting flag may be broken; found: {broken_route:?}"
    );
}

// ---- Criterion 6: explicit scenes-list wins over earlier flag_prefix ----
#[test]
fn explicit_scene_listing_wins_over_earlier_gym_prefix() {
    // BREAKS IF: gym_changing_room (produces GYM_CHANGING_ROOM, explicitly listed
    // under "Ambient life") is stolen by the earlier "Cal / gym" thread's
    // flag_prefix = "GYM_". Author intent in the `scenes` list must win.
    let map = build();

    let gym_thread = map
        .threads
        .iter()
        .find(|t| t.name == "Cal / gym")
        .expect("'Cal / gym' thread must exist");
    let ambient_thread = map
        .threads
        .iter()
        .find(|t| t.name == "Ambient life")
        .expect("'Ambient life' thread must exist");

    assert!(
        !gym_thread
            .scenes
            .iter()
            .any(|s| s.id == "gym_changing_room"),
        "gym_changing_room must NOT be claimed by 'Cal / gym' via the GYM_ prefix"
    );
    assert!(
        ambient_thread
            .scenes
            .iter()
            .any(|s| s.id == "gym_changing_room"),
        "gym_changing_room must be claimed by 'Ambient life' (its explicit scenes list)"
    );
}

// ---- Criterion 7: JSON validity ----
#[test]
fn json_is_valid_and_exposes_thread_names() {
    // BREAKS IF: the JSON sidecar is malformed or the thread-name path moves, so
    // downstream agents can't read it.
    let map = build();
    let json = render_json(&map).expect("render_json must succeed");
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("render_json output must be valid JSON");

    let first_name = parsed["threads"][0]["name"]
        .as_str()
        .expect("threads[0].name must be a string");
    assert!(
        !first_name.is_empty(),
        "first thread name must be non-empty, got {first_name:?}"
    );
    // No-op litmus: cross-check the JSON name against the in-memory map so an empty
    // or hardcoded JSON would fail.
    assert_eq!(
        first_name, map.threads[0].name,
        "JSON thread name must match the in-memory map"
    );
}

// ---- Criterion 8a: committed files are up to date ----
#[test]
fn committed_outputs_match_freshly_generated() {
    // BREAKS IF: the checked-in docs/story-map.{md,json} drift from what the tool
    // generates today — the staleness guarantee (and `--check`) is then a lie.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let md = root.join("docs").join("story-map.md");
    let json = root.join("docs").join("story-map.json");
    let up = is_up_to_date(&packs_dir(), &md, &json)
        .expect("is_up_to_date must succeed on the real pack");
    assert!(
        up,
        "committed docs/story-map.{{md,json}} are STALE vs freshly-generated output. \
         Run `cargo run --bin story-map` and commit."
    );
}

// ---- Criterion 8b: is_up_to_date detects a mutated committed file ----
#[test]
fn is_up_to_date_detects_mismatch() {
    // BREAKS IF: is_up_to_date returns true regardless of file content (a no-op that
    // always passes would make `--check` worthless). We point it at a nonexistent
    // path (reads as empty), which cannot equal the non-empty generated markdown.
    let bogus = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("__nonexistent_story_map__.md");
    let json = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("story-map.json");
    let up = is_up_to_date(&packs_dir(), &bogus, &json)
        .expect("is_up_to_date must succeed even when a file is missing");
    assert!(
        !up,
        "is_up_to_date must report stale when the markdown file is missing/empty"
    );
}

// ---- Negative: render_markdown is non-trivial and reflects real content ----
#[test]
fn markdown_contains_real_thread_headers() {
    // BREAKS IF: the human-facing report renders empty or drops thread sections, so
    // a writer opening story-map.md sees nothing actionable.
    let map = build();
    let md = render_markdown(&map);
    assert!(md.contains("# Story Map"), "missing top header");
    assert!(md.contains("## Write Next"), "missing Write Next digest");
    // Real roadmap threads must appear as section headers.
    assert!(
        md.contains("## Jake romance"),
        "expected the 'Jake romance' thread section in the markdown"
    );
    assert!(
        md.contains("## Ambient life"),
        "expected the 'Ambient life' thread section in the markdown"
    );
}
