//! Independent acceptance test for criterion 10: COMPOSURE is a REQUIRED
//! structural skill. A pack/registry that lacks a COMPOSURE skill definition
//! must fail the required-id validation AT LOAD — the same way a missing
//! FEMININITY does.
//!
//! Written from the criterion alone. Exercises the public `load_packs` entry
//! point against a real-but-mutated copy of the base pack: we clone the base
//! pack to a temp dir and strip exactly one skill from its skills file, then
//! assert load fails with the precise MissingRequiredId for that skill.

use std::path::{Path, PathBuf};

use undone_packs::{load_packs, PackLoadError};

fn base_pack_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("packs")
}

/// Recursively copy a directory tree.
fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir(&from, &to);
        } else {
            std::fs::copy(&from, &to).unwrap();
        }
    }
}

/// Clone the base packs dir into a temp location, then remove the `[[skill]]`
/// block whose `id` == `skill_id` from base/data/skills.toml. Returns the temp
/// packs dir root (containing a single `base/` pack).
fn base_pack_without_skill(skill_id: &str, tag: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("undone_req_{tag}_{unique}"));
    copy_dir(&base_pack_src(), &root);

    let skills_path = root.join("base").join("data").join("skills.toml");
    let src = std::fs::read_to_string(&skills_path).unwrap();
    let mut doc: toml::Value = toml::from_str(&src).unwrap();

    let skills = doc
        .get_mut("skill")
        .and_then(|v| v.as_array_mut())
        .expect("skills.toml must have a [[skill]] array");
    let before = skills.len();
    skills.retain(|s| s.get("id").and_then(|v| v.as_str()) != Some(skill_id));
    let after = skills.len();
    assert_eq!(
        before - after,
        1,
        "expected to strip exactly one '{skill_id}' skill (before={before}, after={after})"
    );

    std::fs::write(&skills_path, toml::to_string(&doc).unwrap()).unwrap();
    root
}

/// BREAKS IF: a pack missing the COMPOSURE skill loads anyway. The whole
/// need-state layer would then resolve COMPOSURE to nothing at runtime and the
/// engine would have no facade axis — a structural requirement silently unmet.
#[test]
fn missing_composure_fails_required_id_validation_at_load() {
    let root = base_pack_without_skill("COMPOSURE", "composure");
    // Discard the Ok payload (PackRegistry isn't Debug) — we only care about the error.
    let result = load_packs(&root).map(|_| ());
    std::fs::remove_dir_all(&root).ok();

    match result {
        Err(PackLoadError::MissingRequiredId { kind, id }) => {
            assert_eq!(kind, "skill", "COMPOSURE is a skill-kind required id");
            assert_eq!(
                id, "COMPOSURE",
                "the missing-required-id error must name COMPOSURE specifically"
            );
        }
        other => panic!(
            "loading a pack without COMPOSURE must fail with \
             MissingRequiredId{{id:\"COMPOSURE\"}}, got: {other:?}"
        ),
    }
}

/// Control / parity proof: the SAME load path rejects a missing FEMININITY the
/// SAME way. This is the "(the same way a missing FEMININITY does)" half of the
/// criterion — confirming COMPOSURE was elevated to a structural required id on
/// equal footing, not handled by some weaker bespoke path.
#[test]
fn missing_femininity_fails_required_id_validation_at_load() {
    let root = base_pack_without_skill("FEMININITY", "femininity");
    let result = load_packs(&root).map(|_| ());
    std::fs::remove_dir_all(&root).ok();

    assert!(
        matches!(
            result,
            Err(PackLoadError::MissingRequiredId {
                kind: "skill",
                id: "FEMININITY"
            })
        ),
        "loading a pack without FEMININITY must fail the same required-id gate, got: {result:?}"
    );
}

/// Sanity floor: the UNMUTATED base pack loads cleanly (so the failures above
/// are caused by the missing skill, not by the copy/strip harness itself).
#[test]
fn untouched_base_pack_loads_clean() {
    let result = load_packs(&base_pack_src());
    assert!(
        result.is_ok(),
        "the real base pack must load cleanly (control for the strip tests): {:?}",
        result.err()
    );
}
