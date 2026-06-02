use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::script::{
    source_advance_arcs, source_has_liking_overshoot, source_set_game_flags, CompiledScript,
};
use crate::types::SceneDefinition;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReachabilityWarning {
    pub context: String,
    pub message: String,
}

#[derive(Default)]
struct EffectFacts {
    set_game_flags: HashSet<String>,
    reachable_arc_states: HashMap<String, HashSet<String>>,
    npc_liking_can_overshoot: bool,
}

pub fn check_reachability(
    schedule_conditions: &[(String, CompiledScript)],
    scenes: &HashMap<String, Arc<SceneDefinition>>,
    preset_starting_flags: &HashSet<String>,
) -> Vec<ReachabilityWarning> {
    let mut facts = collect_effect_facts(scenes);
    // A flag a preset declares as a starting flag is present from game start, so
    // a `hasGameFlag` gate on it is reachable even if no scene effect sets it
    // (e.g. ROUTE_CAMPUS, seeded by the Camila preset). Fold them into the
    // "can be present" set so they don't read as unreachable.
    facts
        .set_game_flags
        .extend(preset_starting_flags.iter().cloned());
    let mut warnings = Vec::new();
    let mut seen = HashSet::new();

    for (context, script) in schedule_conditions {
        inspect_source(&script.source, context, &facts, &mut warnings, &mut seen);
    }

    warnings
}

fn collect_effect_facts(scenes: &HashMap<String, Arc<SceneDefinition>>) -> EffectFacts {
    let mut facts = EffectFacts::default();

    for scene in scenes.values() {
        let effect_sources = scene
            .actions
            .iter()
            .filter_map(|a| a.effect.as_ref())
            .chain(scene.npc_actions.iter().filter_map(|a| a.effect.as_ref()))
            .map(|script| script.source.as_str());

        for src in effect_sources {
            for flag in source_set_game_flags(src) {
                facts.set_game_flags.insert(flag);
            }
            for (arc, state) in source_advance_arcs(src) {
                facts
                    .reachable_arc_states
                    .entry(arc)
                    .or_default()
                    .insert(state);
            }
            if source_has_liking_overshoot(src) {
                facts.npc_liking_can_overshoot = true;
            }
        }
    }

    facts
}

/// Scan a compiled condition's source for the three reachability patterns. This
/// reconstructs the legacy `Expr` walk over the authored Rhai source — sound
/// because flag / arc / liking args are string literals (the design's rule).
fn inspect_source(
    src: &str,
    context: &str,
    facts: &EffectFacts,
    warnings: &mut Vec<ReachabilityWarning>,
    seen: &mut HashSet<(String, String)>,
) {
    // hasGameFlag("X") — warn if no scene sets X. A negated check (`!gd.hasGameFlag`)
    // means "absence is intended", so skip it.
    for (flag, negated) in find_hasgameflag(src) {
        if !negated && !facts.set_game_flags.contains(&flag) {
            push_warning(
                context,
                format!("game flag '{flag}' is required but no scene effect sets it"),
                warnings,
                seen,
            );
        }
    }

    // arcState("ARC") == "STATE" — warn if no scene advances ARC to STATE.
    for (arc, state) in find_eq_call(src, "arcState") {
        let reachable = facts
            .reachable_arc_states
            .get(&arc)
            .is_some_and(|states| states.contains(&state));
        if !reachable {
            push_warning(
                context,
                format!("arc state '{arc} == {state}' is required but no scene advances that arc to '{state}'"),
                warnings,
                seen,
            );
        }
    }

    // npcLiking("ROLE") == "LEVEL" — warn if an AddNpcLiking delta > 1 can overshoot it.
    if facts.npc_liking_can_overshoot {
        for (role, level) in find_eq_call(src, "npcLiking") {
            push_warning(
                context,
                format!("exact npc liking check '{role} == {level}' may be skipped by AddNpcLiking deltas larger than 1"),
                warnings,
                seen,
            );
        }
    }
}

/// All `hasGameFlag("FLAG")` references in a condition source, each paired with
/// whether it is logically negated (`!gd.hasGameFlag(...)`). A negated reference
/// is an anti-requirement (the flag must be ABSENT), not a dependency.
pub fn required_game_flags(src: &str) -> Vec<(String, bool)> {
    find_hasgameflag(src)
}

/// All `arcState("ARC") == "STATE"` equality references in a condition source,
/// returned as `(arc, state)` pairs.
pub fn arc_state_eqs(src: &str) -> Vec<(String, String)> {
    find_eq_call(src, "arcState")
}

/// Find every `hasGameFlag("FLAG")` and whether it is logically negated (the
/// nearest non-space char before `gd.hasGameFlag` is `!`).
fn find_hasgameflag(src: &str) -> Vec<(String, bool)> {
    const NEEDLE: &str = "hasGameFlag(";
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    for (idx, _) in src.match_indices(NEEDLE) {
        if let Some(flag) = string_arg_at(src, idx + NEEDLE.len()) {
            // Walk back past `gd.hasGameFlag` to the receiver start, then skip
            // whitespace; a leading `!` (possibly with spaces) means negated.
            let mut j = idx;
            // step back over "gd." or any "recv."
            while j > 0
                && (bytes[j - 1].is_ascii_alphanumeric()
                    || bytes[j - 1] == b'.'
                    || bytes[j - 1] == b'_')
            {
                j -= 1;
            }
            while j > 0 && bytes[j - 1].is_ascii_whitespace() {
                j -= 1;
            }
            let negated = j > 0 && bytes[j - 1] == b'!';
            out.push((flag, negated));
        }
    }
    out
}

/// Find `method("ARG") == "VALUE"` (and the reversed `"VALUE" == method("ARG")`)
/// occurrences, returning `(arg, value)` pairs.
fn find_eq_call(src: &str, method: &str) -> Vec<(String, String)> {
    let needle = format!("{method}(");
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    for (idx, _) in src.match_indices(&needle) {
        let arg_start = idx + needle.len();
        let Some(arg) = string_arg_at(src, arg_start) else {
            continue;
        };
        // advance past the call's closing ')'
        let mut k = arg_start;
        while k < bytes.len() && bytes[k] != b')' {
            k += 1;
        }
        k += 1; // past ')'
                // method-first order: `) == "value"`
        if let Some(value) = eq_string_after(src, k) {
            out.push((arg.clone(), value));
            continue;
        }
        // value-first order: `"value" == method(...)` — look before the receiver.
        if let Some(value) = eq_string_before(src, idx) {
            out.push((arg, value));
        }
    }
    out
}

/// If `src[from..]` begins (after optional whitespace) with a `"..."` literal,
/// return its contents.
fn string_arg_at(src: &str, from: usize) -> Option<String> {
    let s = &src[from..];
    let s = s.trim_start();
    let rest = s.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// After position `from`, expect `== "value"` (whitespace-tolerant); return value.
fn eq_string_after(src: &str, from: usize) -> Option<String> {
    let s = src.get(from..)?.trim_start();
    let s = s.strip_prefix("==")?.trim_start();
    let rest = s.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Before position `to`, expect `"value" ==` (whitespace-tolerant); return value.
fn eq_string_before(src: &str, to: usize) -> Option<String> {
    let s = src.get(..to)?.trim_end();
    let s = s.strip_suffix("==")?.trim_end();
    let inner = s.strip_suffix('"')?;
    let start = inner.rfind('"')?;
    Some(inner[start + 1..].to_string())
}

fn push_warning(
    context: &str,
    message: String,
    warnings: &mut Vec<ReachabilityWarning>,
    seen: &mut HashSet<(String, String)>,
) {
    let key = (context.to_string(), message.clone());
    if seen.insert(key) {
        warnings.push(ReachabilityWarning {
            context: context.to_string(),
            message,
        });
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::types::{Action, NextBranch, SceneDefinition};

    use super::*;

    fn cond(src: &str) -> CompiledScript {
        crate::script::compile_condition(src, &undone_packs::PackRegistry::new(), "test").unwrap()
    }

    #[test]
    fn required_game_flags_reports_flag_and_negation() {
        // BREAKS IF: positive vs negated flag refs stop being distinguished —
        // story-map would treat `!hasGameFlag` as a dependency.
        let got = required_game_flags(r#"gd.hasGameFlag("A") && !gd.hasGameFlag("B")"#);
        assert!(got.contains(&("A".to_string(), false)));
        assert!(got.contains(&("B".to_string(), true)));
    }

    #[test]
    fn arc_state_eqs_reports_arc_and_state() {
        // BREAKS IF: arcState equality extraction breaks — story-map loses arc edges.
        let got = arc_state_eqs(r#"gd.arcState("base::workplace_opening") == "settled""#);
        assert_eq!(
            got,
            vec![("base::workplace_opening".to_string(), "settled".to_string())]
        );
    }

    fn scene_with_effect(effect_src: &str) -> Arc<SceneDefinition> {
        let effect =
            crate::script::compile_effect(effect_src, &undone_packs::PackRegistry::new(), "test")
                .unwrap();
        Arc::new(SceneDefinition {
            id: "test::scene".into(),
            pack: "test".into(),
            intro_prose: "Intro.".into(),
            intro_variants: vec![],
            intro_thoughts: vec![],
            actions: vec![Action {
                id: "go".into(),
                label: "Go".into(),
                detail: String::new(),
                condition: None,
                prose: String::new(),
                allow_npc_actions: false,
                effect: Some(effect),
                next: vec![NextBranch {
                    condition: None,
                    goto: None,
                    slot: None,
                    finish: true,
                }],
                thoughts: vec![],
            }],
            npc_actions: vec![],
        })
    }

    #[test]
    fn flag_required_but_never_set_is_warned() {
        let warnings = check_reachability(
            &[(
                "slot 'free_time', scene 'base::jake_first_date'".to_string(),
                cond(r#"gd.hasGameFlag("JAKE_MET")"#),
            )],
            &HashMap::new(),
            &HashSet::new(),
        );

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("JAKE_MET"));
    }

    #[test]
    fn flag_satisfied_by_preset_starting_flag_passes() {
        // ROUTE_CAMPUS is set by no scene effect — only the Camila preset declares
        // it as a starting flag. The campus gates must NOT read as unreachable.
        let starting: HashSet<String> = ["ROUTE_CAMPUS".to_string()].into();
        let warnings = check_reachability(
            &[(
                "slot 'campus_opening', scene 'base::campus_library'".to_string(),
                cond(r#"gd.hasGameFlag("ROUTE_CAMPUS")"#),
            )],
            &HashMap::new(),
            &starting,
        );

        assert!(
            warnings.is_empty(),
            "preset starting flag should satisfy reachability: {warnings:?}"
        );
    }

    #[test]
    fn negated_flag_check_does_not_warn_when_flag_never_set() {
        // !gd.hasGameFlag('X') means "fire when flag is absent" — absence is the default state.
        // No warning should be emitted even though no scene sets the flag.
        let warnings = check_reachability(
            &[(
                "slot 'free_time', scene 'base::jake_first_date'".to_string(),
                cond(r#"!gd.hasGameFlag("JAKE_REJECTED")"#),
            )],
            &HashMap::new(),
            &HashSet::new(),
        );

        assert!(
            warnings.is_empty(),
            "negated flag check should not warn: {warnings:?}"
        );
    }

    #[test]
    fn flag_required_and_set_by_effect_passes() {
        let scenes = HashMap::from([(
            "test::scene".to_string(),
            scene_with_effect(r#"gd.setGameFlag("JAKE_MET");"#),
        )]);

        let warnings = check_reachability(
            &[(
                "slot 'free_time', scene 'base::jake_first_date'".to_string(),
                cond(r#"gd.hasGameFlag("JAKE_MET")"#),
            )],
            &scenes,
            &HashSet::new(),
        );

        assert!(warnings.is_empty());
    }

    #[test]
    fn exact_equality_liking_check_warns_when_overshoot_possible() {
        let scenes = HashMap::from([(
            "test::scene".to_string(),
            scene_with_effect(r#"npc("m").addLiking(2);"#),
        )]);

        let warnings = check_reachability(
            &[(
                "slot 'free_time', scene 'base::jake_first_date'".to_string(),
                cond(r#"gd.npcLiking("ROLE_JAKE") == "Like""#),
            )],
            &scenes,
            &HashSet::new(),
        );

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("ROLE_JAKE"));
    }
}
