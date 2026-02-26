use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use thiserror::Error;
use undone_packs::PackRegistry;

use crate::types::{
    Action, ActionDef, EffectDef, NarratorVariant, NarratorVariantDef, NextBranch, NextBranchDef,
    NpcAction, NpcActionDef, SceneDefinition, SceneToml, Thought, ThoughtDef,
};

#[derive(Debug, Error)]
pub enum SceneLoadError {
    #[error("io error reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("toml parse error in {path}: {message}")]
    Toml { path: PathBuf, message: String },
    #[error("parse error in condition '{expr}' in scene {scene_id}: {message}")]
    BadCondition {
        scene_id: String,
        expr: String,
        message: String,
    },
    #[error("unknown trait '{id}' in scene {scene_id}")]
    UnknownTrait { scene_id: String, id: String },
    #[error("unknown skill '{id}' in scene {scene_id}")]
    UnknownSkill { scene_id: String, id: String },
    #[error("scenes directory not found: {0}")]
    DirNotFound(PathBuf),
    #[error("unknown goto target '{target}' in scene {scene_id}, action {action_id}")]
    UnknownGotoTarget {
        scene_id: String,
        action_id: String,
        target: String,
    },
    #[error("unknown arc '{id}' in scene {scene_id}")]
    UnknownArc { scene_id: String, id: String },
    #[error("unknown arc state '{state}' for arc '{arc}' in scene {scene_id}")]
    UnknownArcState {
        scene_id: String,
        arc: String,
        state: String,
    },
    #[error("unknown stat '{id}' in scene {scene_id}")]
    UnknownStat { scene_id: String, id: String },
}

/// Load all `.toml` scene files from `scenes_dir`.
/// Each file is parsed, validated and resolved against the pack registry.
pub fn load_scenes(
    scenes_dir: &Path,
    registry: &PackRegistry,
) -> Result<HashMap<String, Arc<SceneDefinition>>, SceneLoadError> {
    if !scenes_dir.exists() {
        return Err(SceneLoadError::DirNotFound(scenes_dir.to_path_buf()));
    }

    let mut map: HashMap<String, Arc<SceneDefinition>> = HashMap::new();

    let entries = std::fs::read_dir(scenes_dir).map_err(|e| SceneLoadError::Io {
        path: scenes_dir.to_path_buf(),
        source: e,
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| SceneLoadError::Io {
            path: scenes_dir.to_path_buf(),
            source: e,
        })?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }

        let src = std::fs::read_to_string(&path).map_err(|e| SceneLoadError::Io {
            path: path.clone(),
            source: e,
        })?;

        let raw: SceneToml = toml::from_str(&src).map_err(|e| SceneLoadError::Toml {
            path: path.clone(),
            message: e.to_string(),
        })?;

        let scene_id = raw.scene.id.clone();
        let def = resolve_scene(raw, registry, &scene_id)?;
        map.insert(scene_id, Arc::new(def));
    }

    Ok(map)
}

/// Validate that all `goto` targets in all scenes reference existing scene IDs.
/// Call this after all packs' scenes have been loaded into the combined map.
pub fn validate_cross_references(
    scenes: &HashMap<String, Arc<SceneDefinition>>,
) -> Result<(), SceneLoadError> {
    for (scene_id, def) in scenes {
        for action in &def.actions {
            for branch in &action.next {
                if let Some(ref target) = branch.goto {
                    if !scenes.contains_key(target) {
                        return Err(SceneLoadError::UnknownGotoTarget {
                            scene_id: scene_id.clone(),
                            action_id: action.id.clone(),
                            target: target.clone(),
                        });
                    }
                }
            }
        }
    }
    Ok(())
}

fn resolve_scene(
    raw: SceneToml,
    registry: &PackRegistry,
    scene_id: &str,
) -> Result<SceneDefinition, SceneLoadError> {
    let mut intro_variants = Vec::with_capacity(raw.intro_variants.len());
    for v in raw.intro_variants {
        intro_variants.push(resolve_narrator_variant(v, scene_id)?);
    }

    let mut intro_thoughts = Vec::with_capacity(raw.thoughts.len());
    for t in raw.thoughts {
        intro_thoughts.push(resolve_thought(t, scene_id)?);
    }

    let mut actions = Vec::with_capacity(raw.actions.len());
    for a in raw.actions {
        actions.push(resolve_action(a, registry, scene_id)?);
    }

    let mut npc_actions = Vec::with_capacity(raw.npc_actions.len());
    for na in raw.npc_actions {
        npc_actions.push(resolve_npc_action(na, registry, scene_id)?);
    }

    Ok(SceneDefinition {
        id: raw.scene.id,
        pack: raw.scene.pack,
        intro_prose: raw.intro.prose,
        intro_variants,
        intro_thoughts,
        actions,
        npc_actions,
    })
}

fn resolve_thought(raw: ThoughtDef, scene_id: &str) -> Result<Thought, SceneLoadError> {
    let condition = raw
        .condition
        .as_deref()
        .map(|s| parse_condition(s, scene_id))
        .transpose()?;

    Ok(Thought {
        condition,
        prose: raw.prose,
        style: raw.style,
    })
}

fn resolve_narrator_variant(
    raw: NarratorVariantDef,
    scene_id: &str,
) -> Result<NarratorVariant, SceneLoadError> {
    let condition = parse_condition(&raw.condition, scene_id)?;
    Ok(NarratorVariant {
        condition,
        prose: raw.prose,
    })
}

fn parse_condition(
    expr_str: &str,
    scene_id: &str,
) -> Result<undone_expr::parser::Expr, SceneLoadError> {
    undone_expr::parse(expr_str).map_err(|e| SceneLoadError::BadCondition {
        scene_id: scene_id.to_string(),
        expr: expr_str.to_string(),
        message: e.to_string(),
    })
}

fn resolve_action(
    raw: ActionDef,
    registry: &PackRegistry,
    scene_id: &str,
) -> Result<Action, SceneLoadError> {
    let condition = raw
        .condition
        .as_deref()
        .map(|s| parse_condition(s, scene_id))
        .transpose()?;

    let mut next = Vec::with_capacity(raw.next.len());
    for nb in raw.next {
        next.push(resolve_next_branch(nb, scene_id)?);
    }

    // Validate effects at load time
    validate_effects(&raw.effects, registry, scene_id)?;

    let mut thoughts = Vec::with_capacity(raw.thoughts.len());
    for t in raw.thoughts {
        thoughts.push(resolve_thought(t, scene_id)?);
    }

    Ok(Action {
        id: raw.id,
        label: raw.label,
        detail: raw.detail,
        condition,
        prose: raw.prose,
        allow_npc_actions: raw.allow_npc_actions,
        effects: raw.effects,
        next,
        thoughts,
    })
}

fn resolve_npc_action(
    raw: NpcActionDef,
    registry: &PackRegistry,
    scene_id: &str,
) -> Result<NpcAction, SceneLoadError> {
    let condition = raw
        .condition
        .as_deref()
        .map(|s| parse_condition(s, scene_id))
        .transpose()?;

    validate_effects(&raw.effects, registry, scene_id)?;

    let next = raw
        .next
        .into_iter()
        .map(|n| resolve_next_branch(n, scene_id))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(NpcAction {
        id: raw.id,
        condition,
        prose: raw.prose,
        weight: raw.weight,
        effects: raw.effects,
        next,
    })
}

fn resolve_next_branch(raw: NextBranchDef, scene_id: &str) -> Result<NextBranch, SceneLoadError> {
    let condition = raw
        .condition
        .as_deref()
        .map(|s| parse_condition(s, scene_id))
        .transpose()?;

    Ok(NextBranch {
        condition,
        goto: raw.goto,
        slot: raw.slot,
        finish: raw.finish,
    })
}

/// Validate effect IDs against the registry at load time.
fn validate_effects(
    effects: &[EffectDef],
    registry: &PackRegistry,
    scene_id: &str,
) -> Result<(), SceneLoadError> {
    for effect in effects {
        match effect {
            EffectDef::AddTrait { trait_id } | EffectDef::RemoveTrait { trait_id } => {
                registry
                    .resolve_trait(trait_id)
                    .map_err(|_| SceneLoadError::UnknownTrait {
                        scene_id: scene_id.to_string(),
                        id: trait_id.clone(),
                    })?;
            }
            EffectDef::AddNpcTrait { trait_id, .. } => {
                registry
                    .resolve_npc_trait(trait_id)
                    .map_err(|_| SceneLoadError::UnknownTrait {
                        scene_id: scene_id.to_string(),
                        id: trait_id.clone(),
                    })?;
            }
            EffectDef::SkillIncrease { skill, .. } => {
                registry
                    .resolve_skill(skill)
                    .map_err(|_| SceneLoadError::UnknownSkill {
                        scene_id: scene_id.to_string(),
                        id: skill.clone(),
                    })?;
            }
            EffectDef::AddStat { stat, .. } | EffectDef::SetStat { stat, .. } => {
                if !registry.is_registered_stat(stat) {
                    return Err(SceneLoadError::UnknownStat {
                        scene_id: scene_id.to_string(),
                        id: stat.clone(),
                    });
                }
            }
            EffectDef::FailRedCheck { skill } => {
                registry
                    .resolve_skill(skill)
                    .map_err(|_| SceneLoadError::UnknownSkill {
                        scene_id: scene_id.to_string(),
                        id: skill.clone(),
                    })?;
            }
            EffectDef::AdvanceArc { arc, to_state } => {
                let arc_def = registry
                    .get_arc(arc)
                    .ok_or_else(|| SceneLoadError::UnknownArc {
                        scene_id: scene_id.to_string(),
                        id: arc.clone(),
                    })?;
                if !arc_def.states.contains(to_state) {
                    return Err(SceneLoadError::UnknownArcState {
                        scene_id: scene_id.to_string(),
                        arc: arc.clone(),
                        state: to_state.clone(),
                    });
                }
            }
            _ => {}
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("packs")
    }

    #[test]
    fn loads_rain_shelter_scene() {
        let (registry, _) = undone_packs::load_packs(&packs_dir()).unwrap();
        let scenes_dir = packs_dir().join("base").join("scenes");
        let scenes = load_scenes(&scenes_dir, &registry).unwrap();
        assert!(scenes.contains_key("base::rain_shelter"));
    }

    #[test]
    fn rain_shelter_has_expected_actions() {
        let (registry, _) = undone_packs::load_packs(&packs_dir()).unwrap();
        let scenes_dir = packs_dir().join("base").join("scenes");
        let scenes = load_scenes(&scenes_dir, &registry).unwrap();
        let scene = &scenes["base::rain_shelter"];
        let action_ids: Vec<&str> = scene.actions.iter().map(|a| a.id.as_str()).collect();
        assert!(action_ids.contains(&"main"));
        assert!(action_ids.contains(&"leave"));
        assert!(action_ids.contains(&"accept_umbrella"));
    }

    #[test]
    fn error_on_nonexistent_scenes_dir() {
        let registry = undone_packs::PackRegistry::new();
        let result = load_scenes(std::path::Path::new("/no/such/dir"), &registry);
        assert!(result.is_err());
    }

    #[test]
    fn validates_goto_cross_references() {
        use crate::types::{Action, NextBranch, SceneDefinition};
        use std::sync::Arc;

        let scene_a = Arc::new(SceneDefinition {
            id: "test::a".into(),
            pack: "test".into(),
            intro_prose: "A".into(),
            intro_variants: vec![],
            intro_thoughts: vec![],
            actions: vec![Action {
                id: "go".into(),
                label: "Go".into(),
                detail: String::new(),
                condition: None,
                prose: String::new(),
                allow_npc_actions: false,
                effects: vec![],
                next: vec![NextBranch {
                    condition: None,
                    goto: Some("test::nonexistent".into()),
                    slot: None,
                    finish: false,
                }],
                thoughts: vec![],
            }],
            npc_actions: vec![],
        });

        let mut scenes = HashMap::new();
        scenes.insert("test::a".into(), scene_a);

        let result = validate_cross_references(&scenes);
        assert!(result.is_err(), "should reject unknown goto target");
    }

    #[test]
    fn valid_goto_passes_cross_reference_check() {
        use crate::types::{Action, NextBranch, SceneDefinition};
        use std::sync::Arc;

        let scene_a = Arc::new(SceneDefinition {
            id: "test::a".into(),
            pack: "test".into(),
            intro_prose: "A".into(),
            intro_variants: vec![],
            intro_thoughts: vec![],
            actions: vec![Action {
                id: "go".into(),
                label: "Go".into(),
                detail: String::new(),
                condition: None,
                prose: String::new(),
                allow_npc_actions: false,
                effects: vec![],
                next: vec![NextBranch {
                    condition: None,
                    goto: Some("test::b".into()),
                    slot: None,
                    finish: false,
                }],
                thoughts: vec![],
            }],
            npc_actions: vec![],
        });

        let scene_b = Arc::new(SceneDefinition {
            id: "test::b".into(),
            pack: "test".into(),
            intro_prose: "B".into(),
            intro_variants: vec![],
            intro_thoughts: vec![],
            actions: vec![],
            npc_actions: vec![],
        });

        let mut scenes = HashMap::new();
        scenes.insert("test::a".into(), scene_a);
        scenes.insert("test::b".into(), scene_b);

        let result = validate_cross_references(&scenes);
        assert!(result.is_ok(), "valid goto should pass");
    }

    fn make_registry_with_stat(stat_id: &str) -> undone_packs::PackRegistry {
        let mut reg = undone_packs::PackRegistry::new();
        reg.register_stats(vec![undone_packs::data::StatDef {
            id: stat_id.into(),
            name: stat_id.into(),
            description: "test stat".into(),
        }]);
        reg
    }

    #[test]
    fn validate_effects_rejects_unknown_stat_in_add_stat() {
        let registry = undone_packs::PackRegistry::new(); // no stats registered
        let effects = vec![crate::types::EffectDef::AddStat {
            stat: "NONEXISTENT_STAT".into(),
            amount: 1,
        }];
        let result = validate_effects(&effects, &registry, "test::scene");
        assert!(
            matches!(result, Err(SceneLoadError::UnknownStat { .. })),
            "expected UnknownStat error, got: {:?}",
            result
        );
    }

    #[test]
    fn validate_effects_accepts_known_stat_in_add_stat() {
        let registry = make_registry_with_stat("TIMES_KISSED");
        let effects = vec![crate::types::EffectDef::AddStat {
            stat: "TIMES_KISSED".into(),
            amount: 1,
        }];
        let result = validate_effects(&effects, &registry, "test::scene");
        assert!(result.is_ok(), "known stat should pass validation");
    }

    #[test]
    fn validate_effects_rejects_unknown_skill_in_fail_red_check() {
        let registry = undone_packs::PackRegistry::new(); // no skills registered
        let effects = vec![crate::types::EffectDef::FailRedCheck {
            skill: "NONEXISTENT_SKILL".into(),
        }];
        let result = validate_effects(&effects, &registry, "test::scene");
        assert!(
            matches!(result, Err(SceneLoadError::UnknownSkill { .. })),
            "expected UnknownSkill error, got: {:?}",
            result
        );
    }
}
