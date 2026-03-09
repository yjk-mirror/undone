use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use undone_expr::{Call, Expr, Receiver, Value};

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
    schedule_conditions: &[(String, Expr)],
    scenes: &HashMap<String, Arc<SceneDefinition>>,
) -> Vec<ReachabilityWarning> {
    let facts = collect_effect_facts(scenes);
    let mut warnings = Vec::new();
    let mut seen = HashSet::new();

    for (context, expr) in schedule_conditions {
        inspect_expr(expr, context, &facts, &mut warnings, &mut seen, false);
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

fn inspect_expr(
    expr: &Expr,
    context: &str,
    facts: &EffectFacts,
    warnings: &mut Vec<ReachabilityWarning>,
    seen: &mut HashSet<(String, String)>,
    negated: bool,
) {
    match expr {
        // When negated, a hasGameFlag check means "absence is intended" — skip the warning.
        Expr::Call(call) if !negated => inspect_call(call, context, facts, warnings, seen),
        Expr::Call(_) => {}
        Expr::Not(inner) => inspect_expr(inner, context, facts, warnings, seen, !negated),
        Expr::And(left, right)
        | Expr::Or(left, right)
        | Expr::Ne(left, right)
        | Expr::Lt(left, right)
        | Expr::Gt(left, right)
        | Expr::Le(left, right)
        | Expr::Ge(left, right) => {
            inspect_expr(left, context, facts, warnings, seen, negated);
            inspect_expr(right, context, facts, warnings, seen, negated);
        }
        Expr::Eq(left, right) => {
            inspect_expr(left, context, facts, warnings, seen, negated);
            inspect_expr(right, context, facts, warnings, seen, negated);
            inspect_arc_state_eq(left, right, context, facts, warnings, seen);
            inspect_npc_liking_eq(left, right, context, facts, warnings, seen);
        }
        Expr::Lit(_) => {}
    }
}

fn inspect_call(
    call: &Call,
    context: &str,
    facts: &EffectFacts,
    warnings: &mut Vec<ReachabilityWarning>,
    seen: &mut HashSet<(String, String)>,
) {
    if call.receiver == Receiver::GameData && call.method == "hasGameFlag" {
        if let Some(Value::Str(flag)) = call.args.first() {
            if !facts.set_game_flags.contains(flag) {
                push_warning(
                    context,
                    format!("game flag '{flag}' is required but no scene effect sets it"),
                    warnings,
                    seen,
                );
            }
        }
    }
}

fn inspect_arc_state_eq(
    left: &Expr,
    right: &Expr,
    context: &str,
    facts: &EffectFacts,
    warnings: &mut Vec<ReachabilityWarning>,
    seen: &mut HashSet<(String, String)>,
) {
    if let Some((arc, state)) = extract_arc_state_eq(left, right) {
        let reachable = facts
            .reachable_arc_states
            .get(arc)
            .is_some_and(|states| states.contains(state));
        if !reachable {
            push_warning(
                context,
                format!("arc state '{arc} == {state}' is required but no scene advances that arc to '{state}'"),
                warnings,
                seen,
            );
        }
    }
}

fn inspect_npc_liking_eq(
    left: &Expr,
    right: &Expr,
    context: &str,
    facts: &EffectFacts,
    warnings: &mut Vec<ReachabilityWarning>,
    seen: &mut HashSet<(String, String)>,
) {
    if !facts.npc_liking_can_overshoot {
        return;
    }

    if let Some((role, level)) = extract_npc_liking_eq(left, right) {
        push_warning(
            context,
            format!("exact npc liking check '{role} == {level}' may be skipped by AddNpcLiking deltas larger than 1"),
            warnings,
            seen,
        );
    }
}

fn extract_arc_state_eq<'a>(left: &'a Expr, right: &'a Expr) -> Option<(&'a str, &'a str)> {
    match (extract_arc_state_call(left), extract_string_lit(right)) {
        (Some(arc), Some(state)) => Some((arc, state)),
        _ => match (extract_arc_state_call(right), extract_string_lit(left)) {
            (Some(arc), Some(state)) => Some((arc, state)),
            _ => None,
        },
    }
}

fn extract_npc_liking_eq<'a>(left: &'a Expr, right: &'a Expr) -> Option<(&'a str, &'a str)> {
    match (extract_npc_liking_call(left), extract_string_lit(right)) {
        (Some(role), Some(level)) => Some((role, level)),
        _ => match (extract_npc_liking_call(right), extract_string_lit(left)) {
            (Some(role), Some(level)) => Some((role, level)),
            _ => None,
        },
    }
}

fn extract_arc_state_call(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Call(call)
            if call.receiver == Receiver::GameData
                && call.method == "arcState"
                && matches!(call.args.first(), Some(Value::Str(_))) =>
        {
            match call.args.first() {
                Some(Value::Str(arc)) => Some(arc.as_str()),
                _ => None,
            }
        }
        _ => None,
    }
}

fn extract_npc_liking_call(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Call(call)
            if call.receiver == Receiver::GameData
                && call.method == "npcLiking"
                && matches!(call.args.first(), Some(Value::Str(_))) =>
        {
            match call.args.first() {
                Some(Value::Str(role)) => Some(role.as_str()),
                _ => None,
            }
        }
        _ => None,
    }
}

fn extract_string_lit(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Lit(Value::Str(value)) => Some(value.as_str()),
        _ => None,
    }
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
                undone_expr::parse("gd.hasGameFlag('JAKE_MET')").unwrap(),
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
                undone_expr::parse("!gd.hasGameFlag('JAKE_REJECTED')").unwrap(),
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
                undone_expr::parse("gd.hasGameFlag('JAKE_MET')").unwrap(),
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
                undone_expr::parse("gd.npcLiking('ROLE_JAKE') == 'Like'").unwrap(),
            )],
            &scenes,
        );

        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("ROLE_JAKE"));
    }
}
