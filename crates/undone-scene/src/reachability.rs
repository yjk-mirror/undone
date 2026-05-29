use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::script::CompiledScript;
use crate::types::{EffectDef, SceneDefinition};

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
) -> Vec<ReachabilityWarning> {
    let facts = collect_effect_facts(scenes);
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
        for effect in scene
            .actions
            .iter()
            .flat_map(|action| action.effects.iter())
            .chain(
                scene
                    .npc_actions
                    .iter()
                    .flat_map(|action| action.effects.iter()),
            )
        {
            match effect {
                EffectDef::SetGameFlag { flag } => {
                    facts.set_game_flags.insert(flag.clone());
                }
                EffectDef::AdvanceArc { arc, to_state } => {
                    facts
                        .reachable_arc_states
                        .entry(arc.clone())
                        .or_default()
                        .insert(to_state.clone());
                }
                EffectDef::AddNpcLiking { delta, .. } if delta.abs() > 1 => {
                    facts.npc_liking_can_overshoot = true;
                }
                _ => {}
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

    fn scene_with_effect(effect: EffectDef) -> Arc<SceneDefinition> {
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
                effects: vec![effect],
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
        );

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("JAKE_MET"));
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
            scene_with_effect(EffectDef::SetGameFlag {
                flag: "JAKE_MET".into(),
            }),
        )]);

        let warnings = check_reachability(
            &[(
                "slot 'free_time', scene 'base::jake_first_date'".to_string(),
                cond(r#"gd.hasGameFlag("JAKE_MET")"#),
            )],
            &scenes,
        );

        assert!(warnings.is_empty());
    }

    #[test]
    fn exact_equality_liking_check_warns_when_overshoot_possible() {
        let scenes = HashMap::from([(
            "test::scene".to_string(),
            scene_with_effect(EffectDef::AddNpcLiking {
                npc: "m".into(),
                delta: 2,
            }),
        )]);

        let warnings = check_reachability(
            &[(
                "slot 'free_time', scene 'base::jake_first_date'".to_string(),
                cond(r#"gd.npcLiking("ROLE_JAKE") == "Like""#),
            )],
            &scenes,
        );

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("ROLE_JAKE"));
    }
}
