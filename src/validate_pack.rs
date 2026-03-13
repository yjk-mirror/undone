use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rand::{rngs::SmallRng, SeedableRng};
use toml::Value;
use undone_packs::{load_packs, LoadedPackMeta, PackRegistry};
use undone_scene::scheduler::Scheduler;
use undone_scene::simulator::{SimulationConfig, SimulationResult};
use undone_scene::types::SceneDefinition;
use undone_scene::{
    load_scenes, load_schedule, validate_cross_references, validate_entry_scene_references,
};
use undone_world::World;

pub struct ValidationReport {
    pub packs_dir: PathBuf,
    pub total_scenes: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub prose_findings: Vec<ProseFinding>,
}

impl ValidationReport {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn has_finding(&self, file_path: &str, kind: &str) -> bool {
        self.prose_findings
            .iter()
            .any(|finding| finding.file_path == file_path && finding.kind == kind)
    }

    pub fn findings_for_prefix(&self, prefix: &str) -> Vec<&ProseFinding> {
        self.prose_findings
            .iter()
            .filter(|finding| finding.file_path.starts_with(prefix))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProseFinding {
    pub file_path: String,
    pub kind: String,
    pub line: Option<usize>,
    pub message: String,
}

struct LoadedValidationContext {
    registry: PackRegistry,
    scenes: HashMap<String, Arc<SceneDefinition>>,
    scheduler: Option<Scheduler>,
}

pub fn validate_pack_dir(packs_dir: impl AsRef<Path>) -> Result<ValidationReport, String> {
    let (report, _) = collect_validation(packs_dir.as_ref())?;
    Ok(report)
}

pub fn run_simulation_for_tests(weeks: u32, runs: u32) -> Result<SimulationResult, String> {
    run_simulation_from_dir(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("packs"),
        weeks,
        runs,
    )
}

pub fn validate_repo_scenes_for_tests() -> Result<ValidationReport, String> {
    validate_pack_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("packs"))
}

pub fn run_simulation_from_dir(
    packs_dir: impl AsRef<Path>,
    weeks: u32,
    runs: u32,
) -> Result<SimulationResult, String> {
    let (report, context) = collect_validation(packs_dir.as_ref())?;
    if report.has_errors() {
        return Err(report.errors.join("\n"));
    }

    let context =
        context.ok_or_else(|| "Simulation skipped: scheduler not available.".to_string())?;
    let world = build_simulation_world(&context.registry)?;
    Ok(undone_scene::simulator::simulate(
        context
            .scheduler
            .as_ref()
            .expect("validated simulation context should have a scheduler"),
        &context.scenes,
        &context.registry,
        &world,
        SimulationConfig {
            weeks,
            runs,
            seed: 42,
        },
    ))
}

pub fn default_packs_dir() -> PathBuf {
    PathBuf::from("packs")
}

pub fn audit_scene_text(file_path: &str, scene_text: &str) -> Vec<ProseFinding> {
    let mut findings = Vec::new();

    for (index, line) in scene_text.lines().enumerate() {
        let trimmed = line.trim_start();
        let lowercase = line.to_ascii_lowercase();

        let inline_prose_starts_with_third_person = trimmed
            .strip_prefix("prose")
            .and_then(|rest| rest.split_once('"'))
            .is_some_and(|(_, prose_start)| {
                prose_start.starts_with("She ")
                    || prose_start.starts_with("She's")
                    || prose_start.starts_with("She'd")
                    || prose_start.starts_with("She'll")
            });

        if trimmed.starts_with("She ")
            || trimmed.starts_with("She's")
            || trimmed.starts_with("She'd")
            || trimmed.starts_with("She'll")
            || inline_prose_starts_with_third_person
        {
            findings.push(ProseFinding {
                file_path: file_path.to_string(),
                kind: "third_person_player_narration".to_string(),
                line: Some(index + 1),
                message: "player-facing prose should stay in second-person present tense"
                    .to_string(),
            });
        }
        if lowercase.contains("alwaysfemale(") {
            findings.push(ProseFinding {
                file_path: file_path.to_string(),
                kind: "unnecessary_always_female_guard".to_string(),
                line: Some(index + 1),
                message: "avoid `alwaysFemale()` guards in current scene prose unless the branch is genuinely required".to_string(),
            });
        }
        if lowercase.contains("check your phone")
            || lowercase.contains("checking your phone")
            || lowercase.contains("wait for something to happen")
            || lowercase.contains("universal coffee-shop acknowledgment")
        {
            findings.push(ProseFinding {
                file_path: file_path.to_string(),
                kind: "filler_action".to_string(),
                line: Some(index + 1),
                message: "replace filler beats with concrete progression or texture".to_string(),
            });
        }
        if lowercase.contains("none of this was conscious")
            || lowercase.contains("you used to do this")
        {
            findings.push(ProseFinding {
                file_path: file_path.to_string(),
                kind: "meta_analysis".to_string(),
                line: Some(index + 1),
                message: "cut meta-analysis and describe the body-first experience directly"
                    .to_string(),
            });
        }
        if lowercase.contains("completely fine")
            || lowercase.contains("went exactly the way it was supposed to go")
        {
            findings.push(ProseFinding {
                file_path: file_path.to_string(),
                kind: "fine_test_failure".to_string(),
                line: Some(index + 1),
                message: "replace flat 'fine' beats with something earned on the page".to_string(),
            });
        }
    }

    // Intro-specific player agency checks (requires TOML parsing)
    findings.extend(audit_intro_agency(file_path, scene_text));

    findings
}

// Verbs that represent deliberate player decisions — choosing to act.
// Involuntary/experiential verbs (feel, notice, hear, see, smell, know, remember,
// realize, sense, wonder, recognize) are intentionally excluded.
const DELIBERATE_ACTION_VERBS: &[&str] = &[
    "accept", "add", "carry", "change", "check", "choose", "close", "decide", "dress", "drink",
    "drop", "eat", "file", "follow", "get", "grab", "hang", "head", "lean", "lift", "lock", "look",
    "move", "nod", "open", "order", "pay", "pick", "place", "pull", "push", "put", "reach", "set",
    "shake", "shift", "sip", "sit", "slip", "smile", "stand", "step", "stretch", "swap", "take",
    "text", "turn", "type", "walk", "wave", "wear",
];

fn audit_intro_agency(file_path: &str, scene_text: &str) -> Vec<ProseFinding> {
    let Ok(doc) = scene_text.parse::<Value>() else {
        return Vec::new();
    };

    let mut intro_prose_list: Vec<&str> = Vec::new();

    // [intro].prose
    if let Some(prose) = doc
        .get("intro")
        .and_then(|v| v.get("prose"))
        .and_then(|v| v.as_str())
    {
        intro_prose_list.push(prose);
    }

    // [[intro_variants]][].prose
    if let Some(variants) = doc.get("intro_variants").and_then(|v| v.as_array()) {
        for variant in variants {
            if let Some(prose) = variant.get("prose").and_then(|v| v.as_str()) {
                intro_prose_list.push(prose);
            }
        }
    }

    // [[thoughts]][].prose
    if let Some(thoughts) = doc.get("thoughts").and_then(|v| v.as_array()) {
        for thought in thoughts {
            if let Some(prose) = thought.get("prose").and_then(|v| v.as_str()) {
                intro_prose_list.push(prose);
            }
        }
    }

    let mut findings = Vec::new();
    for prose in intro_prose_list {
        for line in prose.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("{%") || trimmed.starts_with("{#") {
                continue;
            }

            if detect_player_speech(trimmed) {
                findings.push(ProseFinding {
                    file_path: file_path.to_string(),
                    kind: "player_speech_in_intro".to_string(),
                    line: find_line_in_source(scene_text, trimmed),
                    message: format!("player speech in intro: {}", truncate_prose(trimmed, 100)),
                });
            }

            if detect_player_deliberate_action(trimmed) {
                findings.push(ProseFinding {
                    file_path: file_path.to_string(),
                    kind: "player_action_in_intro".to_string(),
                    line: find_line_in_source(scene_text, trimmed),
                    message: format!(
                        "player takes deliberate action in intro: {}",
                        truncate_prose(trimmed, 100)
                    ),
                });
            }
        }
    }

    findings
}

/// Detects player speech in intro prose.
///
/// Pattern 1: `"Thanks." You take the bag` — quoted text followed by `You` acting.
/// Pattern 2: `You say/tell/ask` at sentence boundary — player speech verbs.
fn detect_player_speech(line: &str) -> bool {
    // Pattern 1: line starts with quoted text, closing quote followed by " You "
    if line.starts_with('"') {
        if let Some(close_offset) = line[1..].find('"') {
            let after_close = &line[close_offset + 2..];
            let after_trimmed = after_close.trim_start();
            if after_trimmed.starts_with("You ") {
                return true;
            }
        }
    }

    // Pattern 2: "you say/tell/ask" at sentence start
    let lower = line.to_ascii_lowercase();
    for verb in ["you say ", "you tell ", "you ask "] {
        if lower.starts_with(verb) {
            return true;
        }
        // Mid-sentence after period
        if lower.contains(&format!(". {verb}")) {
            return true;
        }
    }

    false
}

/// Detects deliberate player actions in intro prose.
///
/// Matches `You [deliberate_verb]` at sentence boundaries. Excludes involuntary
/// body responses (feel, notice, hear, etc.) which are acceptable in intros.
fn detect_player_deliberate_action(line: &str) -> bool {
    for (pos, _) in line.match_indices("You ") {
        if pos == 0 || is_at_sentence_start(line, pos) {
            let rest = &line[pos + 4..];
            let verb = rest
                .split(|c: char| !c.is_ascii_alphabetic())
                .next()
                .unwrap_or("");
            if DELIBERATE_ACTION_VERBS.contains(&verb.to_ascii_lowercase().as_str()) {
                return true;
            }
        }
    }
    false
}

fn is_at_sentence_start(line: &str, pos: usize) -> bool {
    let before = line[..pos].trim_end();
    before.ends_with('.')
        || before.ends_with('!')
        || before.ends_with('?')
        || before.ends_with('"')
        || before.ends_with('\u{2014}') // em-dash
        || before.ends_with(':')
}

fn find_line_in_source(scene_text: &str, needle: &str) -> Option<usize> {
    scene_text
        .lines()
        .enumerate()
        .find(|(_, line)| line.contains(needle))
        .map(|(i, _)| i + 1)
}

fn truncate_prose(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let end = s.char_indices().nth(max).map(|(i, _)| i).unwrap_or(s.len());
        format!("{}...", &s[..end])
    }
}

fn collect_validation(
    packs_dir: &Path,
) -> Result<(ValidationReport, Option<LoadedValidationContext>), String> {
    let packs_dir = packs_dir.to_path_buf();
    let (registry, pack_metas) =
        load_packs(&packs_dir).map_err(|err| format!("FATAL: pack load failed: {err}"))?;

    let mut report = ValidationReport {
        packs_dir,
        total_scenes: 0,
        warnings: Vec::new(),
        errors: Vec::new(),
        prose_findings: Vec::new(),
    };

    let conflict_errors = registry.validate_trait_conflicts();
    report.errors.extend(
        conflict_errors
            .into_iter()
            .map(|error| format!("trait conflict: {error}")),
    );

    let mut all_scenes = HashMap::new();
    let mut scene_sources = HashMap::new();
    for meta in &pack_metas {
        let scenes_dir = meta.pack_dir.join(&meta.manifest.content.scenes_dir);
        match load_scenes(&scenes_dir, &registry) {
            Ok(scenes) => {
                for (id, scene) in &scenes {
                    if !scene.has_persistent_world_mutation() {
                        report.warnings.push(format!(
                            "[{id}] no persistent world mutation (scene-local flags and navigation do not count)"
                        ));
                    }
                }
                if let Err(error) = extend_scenes_checked(
                    &mut all_scenes,
                    &mut scene_sources,
                    scenes,
                    &meta.manifest.pack.id,
                ) {
                    report.errors.push(error);
                }
            }
            Err(error) => {
                report.errors.push(format!(
                    "ERROR loading scenes for '{}': {error}",
                    meta.manifest.pack.id
                ));
            }
        }
    }

    report.total_scenes = all_scenes.len();

    if let Err(error) = validate_cross_references(&all_scenes) {
        report
            .errors
            .push(format!("ERROR cross-reference: {error}"));
    }

    let scheduler = load_scheduler_report(&registry, &pack_metas, &all_scenes, &mut report);
    if let Some(ref scheduler) = scheduler {
        let warnings = undone_scene::reachability::check_reachability(
            &scheduler.all_conditions(),
            &all_scenes,
        );
        report.warnings.extend(
            warnings
                .into_iter()
                .map(|warning| format!("[reachability] {}: {}", warning.context, warning.message)),
        );

        report.errors.extend(
            undone_ui::char_creation::validate_runtime_contract(&registry, scheduler)
                .into_iter()
                .map(|error| format!("ERROR char creation contract: {error}")),
        );
    }

    report.prose_findings = collect_prose_findings(&report.packs_dir, &pack_metas);

    let context = Some(LoadedValidationContext {
        registry,
        scenes: all_scenes,
        scheduler,
    });
    Ok((report, context))
}

fn load_scheduler_report(
    registry: &PackRegistry,
    pack_metas: &[LoadedPackMeta],
    all_scenes: &HashMap<String, Arc<SceneDefinition>>,
    report: &mut ValidationReport,
) -> Option<Scheduler> {
    match load_schedule(pack_metas, registry) {
        Ok(scheduler) => {
            if let Err(error) = scheduler.validate_scene_references(all_scenes) {
                report
                    .errors
                    .push(format!("ERROR schedule validation: {error}"));
            }
            if let Err(error) = validate_entry_scene_references(
                all_scenes,
                registry.opening_scene(),
                registry.transformation_scene(),
            ) {
                report
                    .errors
                    .push(format!("ERROR entry scene validation: {error}"));
            }
            Some(scheduler)
        }
        Err(error) => {
            report
                .errors
                .push(format!("ERROR loading schedule: {error}"));
            None
        }
    }
}

fn build_simulation_world(registry: &PackRegistry) -> Result<World, String> {
    let mut sim_registry = registry.clone();
    let config = undone_ui::char_creation::robin_quick_config(&sim_registry);
    let mut sim_rng = SmallRng::seed_from_u64(42);
    Ok(undone_packs::char_creation::new_game(
        config,
        &mut sim_registry,
        &mut sim_rng,
    ))
}

fn collect_prose_findings(packs_dir: &Path, pack_metas: &[LoadedPackMeta]) -> Vec<ProseFinding> {
    let mut findings = Vec::new();
    for meta in pack_metas {
        let scene_dir = meta.pack_dir.join(&meta.manifest.content.scenes_dir);
        let mut scene_files = Vec::new();
        collect_scene_files(&scene_dir, &mut scene_files);
        for scene_file in scene_files {
            let Ok(scene_text) = fs::read_to_string(&scene_file) else {
                continue;
            };
            let relative_path = scene_file
                .strip_prefix(packs_dir)
                .ok()
                .map(normalize_pack_relative_path)
                .unwrap_or_else(|| scene_file.to_string_lossy().replace('\\', "/"));
            findings.extend(audit_scene_text(&relative_path, &scene_text));
        }
    }
    findings
}

fn collect_scene_files(scene_dir: &Path, scene_files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(scene_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            collect_scene_files(&path, scene_files);
        } else if path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("toml"))
        {
            scene_files.push(path);
        }
    }
}

fn normalize_pack_relative_path(path: &Path) -> String {
    format!("packs/{}", path.to_string_lossy().replace('\\', "/"))
}

fn extend_scenes_checked(
    all_scenes: &mut HashMap<String, Arc<SceneDefinition>>,
    scene_sources: &mut HashMap<String, String>,
    incoming: HashMap<String, Arc<SceneDefinition>>,
    source: &str,
) -> Result<(), String> {
    for (scene_id, scene) in incoming {
        if let Some(first_source) = scene_sources.insert(scene_id.clone(), source.to_string()) {
            return Err(format!(
                "ERROR loading scenes for '{source}': duplicate scene id '{scene_id}': '{source}' conflicts with already-loaded '{first_source}'"
            ));
        }
        all_scenes.insert(scene_id, scene);
    }
    Ok(())
}
