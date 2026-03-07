use serde::Deserialize;

/// Raw TOML deserialization target for a scene file.
#[derive(Debug, Deserialize)]
pub struct SceneToml {
    pub scene: SceneMeta,
    pub intro: IntroDef,
    /// Optional narrator variants. First passing condition replaces the base intro.
    #[serde(default)]
    pub intro_variants: Vec<NarratorVariantDef>,
    /// Thoughts that fire automatically after the intro, based on conditions.
    #[serde(default)]
    pub thoughts: Vec<ThoughtDef>,
    #[serde(default)]
    pub actions: Vec<ActionDef>,
    #[serde(default)]
    pub npc_actions: Vec<NpcActionDef>,
}

/// A narrator variant: replaces the base intro when its condition passes.
#[derive(Debug, Deserialize, Clone)]
pub struct NarratorVariantDef {
    pub condition: String,
    pub prose: String,
}

/// A thought block: fires automatically (optionally conditioned) with a style tag.
#[derive(Debug, Deserialize, Clone)]
pub struct ThoughtDef {
    pub condition: Option<String>,
    pub prose: String,
    /// Visual style tag for the UI. "inner_voice" = italicised inner monologue.
    #[serde(default = "default_thought_style")]
    pub style: String,
}

fn default_thought_style() -> String {
    "inner_voice".to_string()
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
    /// Thoughts fired after the action prose is displayed.
    #[serde(default)]
    pub thoughts: Vec<ThoughtDef>,
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
    #[serde(default)]
    pub next: Vec<NextBranchDef>,
}

fn default_weight() -> u32 {
    1
}

#[derive(Debug, Deserialize)]
pub struct NextBranchDef {
    #[serde(rename = "if")]
    pub condition: Option<String>,
    pub goto: Option<String>,
    pub slot: Option<String>,
    #[serde(default)]
    pub finish: bool,
}

/// Typed effect, deserialised from `type = "..."` tagged TOML.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EffectDef {
    ChangeStress {
        amount: i32,
    },
    ChangeMoney {
        amount: i32,
    },
    ChangeAnxiety {
        amount: i32,
    },
    SetSceneFlag {
        flag: String,
    },
    RemoveSceneFlag {
        flag: String,
    },
    SetGameFlag {
        flag: String,
    },
    RemoveGameFlag {
        flag: String,
    },
    AddStat {
        stat: String,
        amount: i32,
    },
    SetStat {
        stat: String,
        value: i32,
    },
    SkillIncrease {
        skill: String,
        amount: i32,
    },
    AddTrait {
        trait_id: String,
    },
    RemoveTrait {
        trait_id: String,
    },
    AddArousal {
        delta: i8,
    },
    AddNpcLiking {
        npc: String,
        delta: i8,
    },
    AddNpcLove {
        npc: String,
        delta: i8,
    },
    AddWLiking {
        npc: String,
        delta: i8,
    },
    SetNpcFlag {
        npc: String,
        flag: String,
    },
    AddNpcTrait {
        npc: String,
        trait_id: String,
    },
    Transition {
        target: String,
    },
    AddStuff {
        item: String,
    },
    RemoveStuff {
        item: String,
    },
    SetRelationship {
        npc: String,
        status: String,
    },
    SetNpcAttraction {
        npc: String,
        delta: i8,
    },
    SetNpcBehaviour {
        npc: String,
        behaviour: String,
    },
    SetContactable {
        npc: String,
        value: bool,
    },
    AddSexualActivity {
        npc: String,
        activity: String,
    },
    SetPlayerPartner {
        npc: String,
    },
    AddPlayerFriend {
        npc: String,
    },
    SetJobTitle {
        title: String,
    },
    ChangeAlcohol {
        delta: i8,
    },
    SetVirgin {
        value: bool,
        virgin_type: Option<String>,
    },
    AdvanceTime {
        slots: u32,
    },
    AdvanceArc {
        arc: String,
        to_state: String,
    },
    SetNpcRole {
        /// "m" or "f"
        npc: String,
        role: String,
    },
    FailRedCheck {
        skill: String,
    },
}

impl EffectDef {
    /// Returns true when the effect mutates persistent world state rather than
    /// only the current scene frame or immediate control flow.
    pub fn mutates_persistent_world(&self) -> bool {
        matches!(
            self,
            Self::ChangeStress { .. }
                | Self::ChangeMoney { .. }
                | Self::ChangeAnxiety { .. }
                | Self::SetGameFlag { .. }
                | Self::RemoveGameFlag { .. }
                | Self::AddStat { .. }
                | Self::SetStat { .. }
                | Self::SkillIncrease { .. }
                | Self::AddTrait { .. }
                | Self::RemoveTrait { .. }
                | Self::AddArousal { .. }
                | Self::AddNpcLiking { .. }
                | Self::AddNpcLove { .. }
                | Self::AddWLiking { .. }
                | Self::SetNpcFlag { .. }
                | Self::AddNpcTrait { .. }
                | Self::AddStuff { .. }
                | Self::RemoveStuff { .. }
                | Self::SetRelationship { .. }
                | Self::SetNpcAttraction { .. }
                | Self::SetNpcBehaviour { .. }
                | Self::SetContactable { .. }
                | Self::AddSexualActivity { .. }
                | Self::SetPlayerPartner { .. }
                | Self::AddPlayerFriend { .. }
                | Self::SetJobTitle { .. }
                | Self::ChangeAlcohol { .. }
                | Self::SetVirgin { .. }
                | Self::AdvanceTime { .. }
                | Self::AdvanceArc { .. }
                | Self::SetNpcRole { .. }
                | Self::FailRedCheck { .. }
        )
    }
}

// ---------------------------------------------------------------------------
// Resolved runtime types (after TOML structs)
// ---------------------------------------------------------------------------

use undone_expr::parser::Expr;

/// Resolved thought — condition pre-parsed, ready for runtime evaluation.
#[derive(Debug, Clone)]
pub struct Thought {
    pub condition: Option<Expr>,
    pub prose: String,
    pub style: String,
}

/// Resolved narrator variant — condition pre-parsed.
#[derive(Debug, Clone)]
pub struct NarratorVariant {
    pub condition: Expr,
    pub prose: String,
}

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
    /// Thoughts displayed after the action prose.
    pub thoughts: Vec<Thought>,
}

#[derive(Debug, Clone)]
pub struct NextBranch {
    pub condition: Option<Expr>,
    pub goto: Option<String>,
    pub slot: Option<String>,
    pub finish: bool,
}

#[derive(Debug, Clone)]
pub struct NpcAction {
    pub id: String,
    pub condition: Option<Expr>,
    pub prose: String,
    pub weight: u32,
    pub effects: Vec<EffectDef>,
    pub next: Vec<NextBranch>,
}

/// Immutable scene definition. Wrap in Arc for cheap cloning.
#[derive(Debug)]
pub struct SceneDefinition {
    pub id: String,
    pub pack: String,
    pub intro_prose: String,
    /// Narrator variants evaluated at scene start; first match replaces intro_prose.
    pub intro_variants: Vec<NarratorVariant>,
    /// Thoughts fired after intro prose (before actions are shown).
    pub intro_thoughts: Vec<Thought>,
    pub actions: Vec<Action>,
    pub npc_actions: Vec<NpcAction>,
}

impl SceneDefinition {
    /// Returns true when any player or NPC action in the scene can mutate
    /// persistent world state.
    pub fn has_persistent_world_mutation(&self) -> bool {
        self.actions
            .iter()
            .flat_map(|action| action.effects.iter())
            .chain(
                self.npc_actions
                    .iter()
                    .flat_map(|action| action.effects.iter()),
            )
            .any(EffectDef::mutates_persistent_world)
    }
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

    #[test]
    fn persistent_world_mutation_excludes_scene_local_and_navigation_effects() {
        assert!(EffectDef::ChangeStress { amount: 1 }.mutates_persistent_world());
        assert!(EffectDef::SetGameFlag {
            flag: "SEEN".into()
        }
        .mutates_persistent_world());
        assert!(EffectDef::SetNpcRole {
            npc: "m".into(),
            role: "ROLE_TEST".into(),
        }
        .mutates_persistent_world());
        assert!(!EffectDef::SetSceneFlag {
            flag: "local".into()
        }
        .mutates_persistent_world());
        assert!(!EffectDef::RemoveSceneFlag {
            flag: "local".into()
        }
        .mutates_persistent_world());
        assert!(!EffectDef::Transition {
            target: "base::next".into()
        }
        .mutates_persistent_world());
    }

    #[test]
    fn scene_persistent_world_mutation_counts_player_and_npc_actions() {
        let scene = SceneDefinition {
            id: "test::scene".into(),
            pack: "test".into(),
            intro_prose: "Intro.".into(),
            intro_variants: vec![],
            intro_thoughts: vec![],
            actions: vec![Action {
                id: "look".into(),
                label: "Look".into(),
                detail: String::new(),
                condition: None,
                prose: String::new(),
                allow_npc_actions: false,
                effects: vec![EffectDef::SetSceneFlag {
                    flag: "local".into(),
                }],
                next: vec![],
                thoughts: vec![],
            }],
            npc_actions: vec![NpcAction {
                id: "answer".into(),
                condition: None,
                prose: String::new(),
                weight: 1,
                effects: vec![EffectDef::AddNpcLiking {
                    npc: "male".into(),
                    delta: 1,
                }],
                next: vec![],
            }],
        };

        assert!(scene.has_persistent_world_mutation());
    }

    #[test]
    fn scene_without_persistent_mutation_returns_false() {
        let scene = SceneDefinition {
            id: "test::scene".into(),
            pack: "test".into(),
            intro_prose: "Intro.".into(),
            intro_variants: vec![],
            intro_thoughts: vec![],
            actions: vec![Action {
                id: "move".into(),
                label: "Move".into(),
                detail: String::new(),
                condition: None,
                prose: String::new(),
                allow_npc_actions: false,
                effects: vec![
                    EffectDef::SetSceneFlag {
                        flag: "local".into(),
                    },
                    EffectDef::Transition {
                        target: "base::next".into(),
                    },
                ],
                next: vec![],
                thoughts: vec![],
            }],
            npc_actions: vec![],
        };

        assert!(!scene.has_persistent_world_mutation());
    }
}
