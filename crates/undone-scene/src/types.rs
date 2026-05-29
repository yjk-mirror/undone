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
    /// Rhai effect call-list, e.g. `effect = 'w.addArousal(1); gd.setGameFlag("X");'`.
    #[serde(default)]
    pub effect: Option<String>,
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
    /// Rhai effect call-list (see [`ActionDef::effect`]).
    #[serde(default)]
    pub effect: Option<String>,
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

// ---------------------------------------------------------------------------
// Resolved runtime types (after TOML structs)
// ---------------------------------------------------------------------------

use crate::script::CompiledScript;

/// Resolved thought — condition compiled, ready for runtime evaluation.
#[derive(Debug, Clone)]
pub struct Thought {
    pub condition: Option<CompiledScript>,
    pub prose: String,
    pub style: String,
}

/// Resolved narrator variant — condition compiled.
#[derive(Debug, Clone)]
pub struct NarratorVariant {
    pub condition: CompiledScript,
    pub prose: String,
}

/// Resolved action — conditions compiled, ready for runtime evaluation.
#[derive(Debug, Clone)]
pub struct Action {
    pub id: String,
    pub label: String,
    pub detail: String,
    pub condition: Option<CompiledScript>,
    pub prose: String,
    pub allow_npc_actions: bool,
    /// Compiled effect call-list (applied via `apply_effect_script`).
    pub effect: Option<CompiledScript>,
    pub next: Vec<NextBranch>,
    /// Thoughts displayed after the action prose.
    pub thoughts: Vec<Thought>,
}

#[derive(Debug, Clone)]
pub struct NextBranch {
    pub condition: Option<CompiledScript>,
    pub goto: Option<String>,
    pub slot: Option<String>,
    pub finish: bool,
}

#[derive(Debug, Clone)]
pub struct NpcAction {
    pub id: String,
    pub condition: Option<CompiledScript>,
    pub prose: String,
    pub weight: u32,
    /// Compiled effect call-list (applied via `apply_effect_script`).
    pub effect: Option<CompiledScript>,
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
    /// persistent world state. Scans the compiled effect call-lists' source for
    /// any non-scene-local mutator (reconstructs the legacy `EffectDef` walk).
    pub fn has_persistent_world_mutation(&self) -> bool {
        self.actions
            .iter()
            .filter_map(|action| action.effect.as_ref())
            .chain(
                self.npc_actions
                    .iter()
                    .filter_map(|action| action.effect.as_ref()),
            )
            .any(|script| crate::script::source_has_persistent_mutation(&script.source))
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
condition = '!scene.hasFlag("blocked")'
prose = "You leave."
effect = "w.changeStress(-1);"

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
    fn parses_action_effect() {
        let raw: SceneToml = toml::from_str(MINIMAL_SCENE).unwrap();
        let leave = raw.actions.iter().find(|a| a.id == "leave").unwrap();
        assert_eq!(leave.effect.as_deref(), Some("w.changeStress(-1);"));
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

    /// Build a compiled effect for tests (empty registry — flags/scene only).
    fn effect(src: &str) -> CompiledScript {
        crate::script::compile_effect(src, &undone_packs::PackRegistry::new(), "test").unwrap()
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
                // scene-local only — not persistent on its own
                effect: Some(effect(r#"scene.setFlag("local");"#)),
                next: vec![],
                thoughts: vec![],
            }],
            npc_actions: vec![NpcAction {
                id: "answer".into(),
                condition: None,
                prose: String::new(),
                weight: 1,
                // persistent NPC mutation
                effect: Some(effect(r#"npc("m").addLiking(1);"#)),
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
                effect: Some(effect(r#"scene.setFlag("local");"#)),
                next: vec![],
                thoughts: vec![],
            }],
            npc_actions: vec![],
        };

        assert!(!scene.has_persistent_world_mutation());
    }
}
