use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use thiserror::Error;
use undone_packs::PackRegistry;

use crate::types::{
    Action, ActionDef, EffectDef, NextBranch, NextBranchDef, NpcAction, NpcActionDef,
    SceneDefinition, SceneToml,
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

fn resolve_scene(
    raw: SceneToml,
    registry: &PackRegistry,
    scene_id: &str,
) -> Result<SceneDefinition, SceneLoadError> {
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
        actions,
        npc_actions,
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

    Ok(Action {
        id: raw.id,
        label: raw.label,
        detail: raw.detail,
        condition,
        prose: raw.prose,
        allow_npc_actions: raw.allow_npc_actions,
        effects: raw.effects,
        next,
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

    Ok(NpcAction {
        id: raw.id,
        condition,
        prose: raw.prose,
        weight: raw.weight,
        effects: raw.effects,
    })
}

fn resolve_next_branch(
    raw: NextBranchDef,
    scene_id: &str,
) -> Result<NextBranch, SceneLoadError> {
    let condition = raw
        .condition
        .as_deref()
        .map(|s| parse_condition(s, scene_id))
        .transpose()?;

    Ok(NextBranch {
        condition,
        goto: raw.goto,
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
}
