use serde::Deserialize;

/// Raw TOML deserialization target for a scene file.
#[derive(Debug, Deserialize)]
pub struct SceneToml {
    pub scene: SceneMeta,
    pub intro: IntroDef,
    #[serde(default)]
    pub actions: Vec<ActionDef>,
    #[serde(default)]
    pub npc_actions: Vec<NpcActionDef>,
}

#[derive(Debug, Deserialize)]
pub struct SceneMeta {
    pub id: String,
    pub pack: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct IntroDef {
    pub prose: String,
}

#[derive(Debug, Deserialize)]
pub struct ActionDef {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub detail: String,
    pub condition: Option<String>,
    #[serde(default)]
    pub prose: String,
    #[serde(default)]
    pub allow_npc_actions: bool,
    #[serde(default)]
    pub effects: Vec<EffectDef>,
    #[serde(default)]
    pub next: Vec<NextBranchDef>,
}

#[derive(Debug, Deserialize)]
pub struct NpcActionDef {
    pub id: String,
    pub condition: Option<String>,
    #[serde(default)]
    pub prose: String,
    #[serde(default = "default_weight")]
    pub weight: u32,
    #[serde(default)]
    pub effects: Vec<EffectDef>,
}

fn default_weight() -> u32 {
    1
}

#[derive(Debug, Deserialize)]
pub struct NextBranchDef {
    #[serde(rename = "if")]
    pub condition: Option<String>,
    pub goto: Option<String>,
    #[serde(default)]
    pub finish: bool,
}

/// Typed effect, deserialised from `type = "..."` tagged TOML.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EffectDef {
    ChangeStress { amount: i32 },
    ChangeMoney { amount: i32 },
    ChangeAnxiety { amount: i32 },
    SetSceneFlag { flag: String },
    RemoveSceneFlag { flag: String },
    SetGameFlag { flag: String },
    RemoveGameFlag { flag: String },
    AddStat { stat: String, amount: i32 },
    SetStat { stat: String, value: i32 },
    SkillIncrease { skill: String, amount: i32 },
    AddTrait { trait_id: String },
    RemoveTrait { trait_id: String },
    AddArousal { delta: i8 },
    AddNpcLiking { npc: String, delta: i8 },
    AddNpcLove { npc: String, delta: i8 },
    AddWLiking { npc: String, delta: i8 },
    SetNpcFlag { npc: String, flag: String },
    AddNpcTrait { npc: String, trait_id: String },
    Transition { target: String },
}

// ---------------------------------------------------------------------------
// Resolved runtime types (after TOML structs)
// ---------------------------------------------------------------------------

use undone_expr::parser::Expr;

/// Resolved action — conditions parsed, ready for runtime evaluation.
#[derive(Debug, Clone)]
pub struct Action {
    pub id: String,
    pub label: String,
    pub detail: String,
    pub condition: Option<Expr>,
    pub prose: String,
    pub allow_npc_actions: bool,
    pub effects: Vec<EffectDef>,
    pub next: Vec<NextBranch>,
}

#[derive(Debug, Clone)]
pub struct NextBranch {
    pub condition: Option<Expr>,
    pub goto: Option<String>,
    pub finish: bool,
}

#[derive(Debug, Clone)]
pub struct NpcAction {
    pub id: String,
    pub condition: Option<Expr>,
    pub prose: String,
    pub weight: u32,
    pub effects: Vec<EffectDef>,
}

/// Immutable scene definition. Wrap in Arc for cheap cloning.
#[derive(Debug)]
pub struct SceneDefinition {
    pub id: String,
    pub pack: String,
    pub intro_prose: String,
    pub actions: Vec<Action>,
    pub npc_actions: Vec<NpcAction>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_SCENE: &str = r#"
[scene]
id = "test::minimal"
pack = "test"
description = "A minimal test scene."

[intro]
prose = "It begins."

[[actions]]
id = "wait"
label = "Wait"
detail = "Just wait."

[[actions]]
id = "leave"
label = "Leave"
condition = "!scene.hasFlag('blocked')"
prose = "You leave."

  [[actions.effects]]
  type = "change_stress"
  amount = -1

  [[actions.next]]
  finish = true
"#;

    #[test]
    fn parses_minimal_scene() {
        let raw: SceneToml = toml::from_str(MINIMAL_SCENE).unwrap();
        assert_eq!(raw.scene.id, "test::minimal");
        assert_eq!(raw.actions.len(), 2);
    }

    #[test]
    fn parses_action_effects() {
        let raw: SceneToml = toml::from_str(MINIMAL_SCENE).unwrap();
        let leave = raw.actions.iter().find(|a| a.id == "leave").unwrap();
        assert_eq!(leave.effects.len(), 1);
        assert!(matches!(
            leave.effects[0],
            EffectDef::ChangeStress { amount: -1 }
        ));
    }

    #[test]
    fn parses_action_next() {
        let raw: SceneToml = toml::from_str(MINIMAL_SCENE).unwrap();
        let leave = raw.actions.iter().find(|a| a.id == "leave").unwrap();
        assert_eq!(leave.next.len(), 1);
        assert!(leave.next[0].finish);
    }

    #[test]
    fn action_with_no_next_has_empty_vec() {
        let raw: SceneToml = toml::from_str(MINIMAL_SCENE).unwrap();
        let wait = raw.actions.iter().find(|a| a.id == "wait").unwrap();
        assert!(wait.next.is_empty());
    }
}
