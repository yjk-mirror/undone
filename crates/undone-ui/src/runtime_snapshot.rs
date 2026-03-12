use floem::prelude::SignalGet;
use serde::{Deserialize, Serialize};
use undone_scene::engine::BoundNpcData;

use crate::game_state::GameState;
use crate::{AppPhase, AppSignals, AppTab, NpcSnapshot, PlayerSnapshot};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeSnapshot {
    pub phase: String,
    pub tab: String,
    pub window_width: f64,
    pub window_height: f64,
    pub current_scene_id: Option<String>,
    pub awaiting_continue: bool,
    pub story_paragraphs: Vec<String>,
    pub visible_actions: Vec<VisibleActionSnapshot>,
    pub active_npc: Option<ActiveNpcSnapshot>,
    pub active_npcs: Vec<BoundActiveNpcSnapshot>,
    pub player: PlayerSummarySnapshot,
    pub world: WorldSummarySnapshot,
    pub init_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VisibleActionSnapshot {
    pub id: String,
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActiveNpcSnapshot {
    pub name: String,
    pub age: String,
    pub personality: String,
    pub relationship: String,
    pub pc_liking: String,
    pub pc_attraction: String,
    pub known: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoundActiveNpcSnapshot {
    pub binding: String,
    pub name: String,
    pub age: String,
    pub personality: String,
    pub relationship: String,
    pub pc_liking: String,
    pub pc_attraction: String,
    pub known: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PlayerSummarySnapshot {
    pub name: String,
    pub femininity: i32,
    pub money: i32,
    pub stress: i32,
    pub anxiety: i32,
    pub arousal: String,
    pub alcohol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct WorldSummarySnapshot {
    pub week: u32,
    pub day: u8,
    pub time_slot: String,
    pub game_flags: Vec<String>,
    pub arc_states: Vec<ArcStateSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArcStateSnapshot {
    pub id: String,
    pub state: String,
}

pub fn snapshot_runtime(signals: AppSignals, gs: &GameState) -> RuntimeSnapshot {
    let mut game_flags: Vec<String> = gs.world.game_data.flags.iter().cloned().collect();
    game_flags.sort();

    let mut arc_states: Vec<ArcStateSnapshot> = gs
        .world
        .game_data
        .arc_states
        .iter()
        .map(|(id, state)| ArcStateSnapshot {
            id: id.clone(),
            state: state.clone(),
        })
        .collect();
    arc_states.sort_by(|left, right| left.id.cmp(&right.id));

    let player = PlayerSnapshot::from_player(&gs.world.player, gs.femininity_id);
    let active_npcs = gs
        .engine
        .current_bound_npcs(&gs.world, &gs.registry)
        .into_iter()
        .map(bound_npc_snapshot)
        .collect();

    RuntimeSnapshot {
        phase: phase_name(signals.phase.get_untracked()).to_string(),
        tab: tab_name(signals.tab.get_untracked()).to_string(),
        window_width: signals.window_width.get_untracked(),
        window_height: signals.window_height.get_untracked(),
        current_scene_id: gs.engine.current_scene_id(),
        awaiting_continue: signals.awaiting_continue.get_untracked(),
        story_paragraphs: story_paragraphs(&signals.story.get_untracked()),
        visible_actions: signals
            .actions
            .get_untracked()
            .into_iter()
            .map(|action| VisibleActionSnapshot {
                id: action.id,
                label: action.label,
                detail: action.detail,
            })
            .collect(),
        active_npc: signals.active_npc.get_untracked().map(active_npc_snapshot),
        active_npcs,
        player: PlayerSummarySnapshot {
            name: player.name,
            femininity: player.femininity,
            money: player.money,
            stress: player.stress,
            anxiety: player.anxiety,
            arousal: player.arousal,
            alcohol: player.alcohol,
        },
        world: WorldSummarySnapshot {
            week: gs.world.game_data.week,
            day: gs.world.game_data.day,
            time_slot: format!("{:?}", gs.world.game_data.time_slot),
            game_flags,
            arc_states,
        },
        init_error: gs.init_error.clone(),
    }
}

fn story_paragraphs(story: &str) -> Vec<String> {
    story
        .split("\n\n")
        .filter(|paragraph| !paragraph.trim().is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn active_npc_snapshot(npc: NpcSnapshot) -> ActiveNpcSnapshot {
    let known = is_known_npc(&npc.relationship, &npc.pc_liking, &npc.pc_attraction);
    ActiveNpcSnapshot {
        name: npc.name,
        age: npc.age,
        personality: npc.personality,
        relationship: format!("{:?}", npc.relationship),
        pc_liking: format!("{:?}", npc.pc_liking),
        pc_attraction: format!("{:?}", npc.pc_attraction),
        known,
    }
}

fn bound_npc_snapshot(bound: BoundNpcData) -> BoundActiveNpcSnapshot {
    let known = is_known_npc(
        &bound.npc.relationship,
        &bound.npc.pc_liking,
        &bound.npc.pc_attraction,
    );
    BoundActiveNpcSnapshot {
        binding: bound.binding,
        name: bound.npc.name,
        age: format!("{:?}", bound.npc.age),
        personality: bound.npc.personality,
        relationship: format!("{:?}", bound.npc.relationship),
        pc_liking: format!("{:?}", bound.npc.pc_liking),
        pc_attraction: format!("{:?}", bound.npc.pc_attraction),
        known,
    }
}

fn is_known_npc(
    relationship: &undone_domain::RelationshipStatus,
    pc_liking: &undone_domain::LikingLevel,
    pc_attraction: &undone_domain::AttractionLevel,
) -> bool {
    !matches!(relationship, undone_domain::RelationshipStatus::Stranger)
        || !matches!(pc_liking, undone_domain::LikingLevel::Neutral)
        || !matches!(pc_attraction, undone_domain::AttractionLevel::Unattracted)
}

fn phase_name(phase: AppPhase) -> &'static str {
    match phase {
        AppPhase::Landing => "landing",
        AppPhase::BeforeCreation => "before_creation",
        AppPhase::TransformationIntro => "transformation_intro",
        AppPhase::FemCreation => "fem_creation",
        AppPhase::InGame => "in_game",
    }
}

fn tab_name(tab: AppTab) -> &'static str {
    match tab {
        AppTab::Game => "game",
        AppTab::Saves => "saves",
        AppTab::Settings => "settings",
        AppTab::Dev => "dev",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::{start_game, PreGameState};
    use crate::{AppPhase, AppTab, NpcSnapshot};
    use floem::prelude::SignalUpdate;
    use lasso::Key;
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;
    use std::sync::Arc;
    use undone_domain::{
        Age, AttractionLevel, Behaviour, LikingLevel, LoveLevel, MaleClothing, MaleFigure, MaleNpc,
        NpcCore, PersonalityId, RelationshipStatus,
    };
    use undone_scene::engine::{ActionView, EngineCommand};
    use undone_scene::{Action, EffectDef, NpcAction, SceneDefinition, SceneEngine, SceneNpcRef};

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("packs")
    }

    fn test_pre_state() -> PreGameState {
        crate::game_state::test_pre_state_from_dir(&packs_dir())
    }

    fn test_game_state() -> GameState {
        let pre = test_pre_state();
        let config = crate::char_creation::robin_quick_config(&pre.registry);
        start_game(pre, config, true)
    }

    fn test_male_npc(personality: PersonalityId) -> MaleNpc {
        MaleNpc {
            core: NpcCore {
                name: "Jake".into(),
                age: Age::MidLateTwenties,
                race: "white".into(),
                eye_colour: "blue".into(),
                hair_colour: "brown".into(),
                personality,
                traits: HashSet::new(),
                relationship: RelationshipStatus::Acquaintance,
                pc_liking: LikingLevel::Like,
                npc_liking: LikingLevel::Neutral,
                pc_love: LoveLevel::None,
                npc_love: LoveLevel::None,
                pc_attraction: AttractionLevel::Attracted,
                npc_attraction: AttractionLevel::Unattracted,
                behaviour: Behaviour::Neutral,
                relationship_flags: HashSet::new(),
                sexual_activities: HashSet::new(),
                custom_flags: HashMap::new(),
                custom_ints: HashMap::new(),
                knowledge: 0,
                contactable: true,
                arousal: undone_domain::ArousalLevel::Comfort,
                alcohol: undone_domain::AlcoholLevel::Sober,
                roles: HashSet::new(),
            },
            figure: MaleFigure::Average,
            clothing: MaleClothing::default(),
            had_orgasm: false,
            has_baby_with_pc: false,
        }
    }

    #[test]
    fn valid_runtime_content_still_reaches_character_creation() {
        let pre = crate::game_state::init_game_from_dir(&packs_dir());

        assert!(pre.init_error.is_none());
        assert!(!pre.scenes.is_empty());
    }

    #[test]
    fn runtime_snapshot_includes_player_visible_runtime_fields() {
        let mut gs = test_game_state();
        let signals = AppSignals::new();
        signals.phase.set(AppPhase::InGame);
        signals.tab.set(AppTab::Dev);
        signals.awaiting_continue.set(true);
        signals.window_width.set(1800.0);
        signals.window_height.set(1000.0);
        signals
            .story
            .set("First paragraph.\n\nSecond paragraph.".into());
        signals.actions.set(vec![ActionView {
            id: "wait".into(),
            label: "Wait".into(),
            detail: "Stay put.".into(),
        }]);
        signals.active_npc.set(Some(NpcSnapshot {
            name: "Jake".into(),
            age: "MidLateTwenties".into(),
            personality: "Romantic".into(),
            relationship: RelationshipStatus::Acquaintance,
            pc_liking: LikingLevel::Like,
            pc_attraction: AttractionLevel::Attracted,
        }));
        gs.world.game_data.set_flag("FLAG_ALPHA");
        gs.world
            .game_data
            .advance_arc("base::workplace_opening", "arrived");
        gs.engine.send(
            EngineCommand::StartScene("base::rain_shelter".into()),
            &mut gs.world,
            &gs.registry,
        );
        gs.engine.drain();

        let snapshot = snapshot_runtime(signals, &gs);

        assert_eq!(snapshot.phase, "in_game");
        assert_eq!(snapshot.tab, "dev");
        assert_eq!(
            snapshot.current_scene_id.as_deref(),
            Some("base::rain_shelter")
        );
        assert!(snapshot.awaiting_continue);
        assert_eq!(
            snapshot.story_paragraphs,
            vec![
                "First paragraph.".to_string(),
                "Second paragraph.".to_string()
            ]
        );
        assert_eq!(
            snapshot.visible_actions,
            vec![VisibleActionSnapshot {
                id: "wait".into(),
                label: "Wait".into(),
                detail: "Stay put.".into(),
            }]
        );
        assert_eq!(
            snapshot.active_npc,
            Some(ActiveNpcSnapshot {
                name: "Jake".into(),
                age: "MidLateTwenties".into(),
                personality: "Romantic".into(),
                relationship: "Acquaintance".into(),
                pc_liking: "Like".into(),
                pc_attraction: "Attracted".into(),
                known: true,
            })
        );
        assert!(snapshot.active_npcs.is_empty());
        assert_eq!(snapshot.player.name, "Robin");
        assert_eq!(snapshot.window_width, 1800.0);
        assert_eq!(snapshot.window_height, 1000.0);
        assert!(snapshot
            .world
            .game_flags
            .contains(&"FLAG_ALPHA".to_string()));
        assert!(snapshot
            .world
            .arc_states
            .iter()
            .any(|arc| arc.id == "base::workplace_opening" && arc.state == "arrived"));
    }

    #[test]
    fn runtime_snapshot_lists_multiple_active_npcs() {
        let mut gs = test_game_state();
        let signals = AppSignals::new();
        let romantic = gs.registry.intern_personality("ROMANTIC");
        let calm = gs.registry.intern_personality("CALM");
        let male_key = gs.world.male_npcs.insert(test_male_npc(romantic));
        let female_key = gs.world.female_npcs.insert(undone_domain::FemaleNpc {
            core: NpcCore {
                name: "Mia".into(),
                age: Age::MidLateTwenties,
                race: "white".into(),
                eye_colour: "green".into(),
                hair_colour: "black".into(),
                personality: calm,
                traits: HashSet::new(),
                relationship: RelationshipStatus::Friend,
                pc_liking: LikingLevel::Like,
                npc_liking: LikingLevel::Neutral,
                pc_love: LoveLevel::None,
                npc_love: LoveLevel::None,
                pc_attraction: AttractionLevel::Unattracted,
                npc_attraction: AttractionLevel::Unattracted,
                behaviour: Behaviour::Neutral,
                relationship_flags: HashSet::new(),
                sexual_activities: HashSet::new(),
                custom_flags: HashMap::new(),
                custom_ints: HashMap::new(),
                knowledge: 0,
                contactable: true,
                arousal: undone_domain::ArousalLevel::Comfort,
                alcohol: undone_domain::AlcoholLevel::Sober,
                roles: HashSet::new(),
            },
            char_type: undone_domain::CharTypeId::from_spur(
                lasso::Spur::try_from_usize(0).unwrap(),
            ),
            figure: undone_domain::PlayerFigure::Slim,
            breasts: undone_domain::BreastSize::Average,
            clothing: undone_domain::FemaleClothing::default(),
            pregnancy: None,
            virgin: true,
        });
        let mut role_bindings = HashMap::new();
        role_bindings.insert("ROLE_TEAM_LEAD".to_string(), SceneNpcRef::Male(male_key));
        role_bindings.insert("ROLE_DESIGNER".to_string(), SceneNpcRef::Female(female_key));

        gs.engine.start_scene_with_role_bindings(
            "base::rain_shelter".into(),
            None,
            None,
            role_bindings,
            &gs.world,
            &gs.registry,
        );
        gs.engine.drain();

        let snapshot = snapshot_runtime(signals, &gs);
        assert_eq!(
            snapshot.active_npcs,
            vec![
                BoundActiveNpcSnapshot {
                    binding: "ROLE_DESIGNER".into(),
                    name: "Mia".into(),
                    age: "MidLateTwenties".into(),
                    personality: "CALM".into(),
                    relationship: "Friend".into(),
                    pc_liking: "Like".into(),
                    pc_attraction: "Unattracted".into(),
                    known: true,
                },
                BoundActiveNpcSnapshot {
                    binding: "ROLE_TEAM_LEAD".into(),
                    name: "Jake".into(),
                    age: "MidLateTwenties".into(),
                    personality: "ROMANTIC".into(),
                    relationship: "Acquaintance".into(),
                    pc_liking: "Like".into(),
                    pc_attraction: "Attracted".into(),
                    known: true,
                }
            ]
        );
    }

    #[test]
    fn multi_npc_scene_contract_supports_prose_effects_and_snapshot() {
        let mut gs = test_game_state();
        let signals = AppSignals::new();
        let romantic = gs.registry.intern_personality("ROMANTIC");
        let calm = gs.registry.intern_personality("CALM");
        let male_key = gs.world.male_npcs.insert(test_male_npc(romantic));
        let female_key = gs.world.female_npcs.insert(undone_domain::FemaleNpc {
            core: NpcCore {
                name: "Mia".into(),
                age: Age::MidLateTwenties,
                race: "white".into(),
                eye_colour: "green".into(),
                hair_colour: "black".into(),
                personality: calm,
                traits: HashSet::new(),
                relationship: RelationshipStatus::Friend,
                pc_liking: LikingLevel::Like,
                npc_liking: LikingLevel::Neutral,
                pc_love: LoveLevel::None,
                npc_love: LoveLevel::None,
                pc_attraction: AttractionLevel::Unattracted,
                npc_attraction: AttractionLevel::Unattracted,
                behaviour: Behaviour::Neutral,
                relationship_flags: HashSet::new(),
                sexual_activities: HashSet::new(),
                custom_flags: HashMap::new(),
                custom_ints: HashMap::new(),
                knowledge: 0,
                contactable: true,
                arousal: undone_domain::ArousalLevel::Comfort,
                alcohol: undone_domain::AlcoholLevel::Sober,
                roles: HashSet::new(),
            },
            char_type: undone_domain::CharTypeId::from_spur(
                lasso::Spur::try_from_usize(0).unwrap(),
            ),
            figure: undone_domain::PlayerFigure::Slim,
            breasts: undone_domain::BreastSize::Average,
            clothing: undone_domain::FemaleClothing::default(),
            pregnancy: None,
            virgin: true,
        });
        let mut scenes = HashMap::new();
        scenes.insert(
            "test::multi_npc".to_string(),
            Arc::new(SceneDefinition {
                id: "test::multi_npc".into(),
                pack: "test".into(),
                intro_prose:
                    r#"{{ role.getName("ROLE_TEAM_LEAD") }} and {{ role.getName("ROLE_DESIGNER") }}"#
                        .into(),
                intro_variants: vec![],
                intro_thoughts: vec![],
                actions: vec![Action {
                    id: "approve".into(),
                    label: "Approve".into(),
                    detail: "Move things forward.".into(),
                    condition: None,
                    prose: String::new(),
                    allow_npc_actions: false,
                    effects: vec![EffectDef::AddNpcLiking {
                        npc: "ROLE_TEAM_LEAD".into(),
                        delta: 1,
                    }],
                    next: vec![],
                    thoughts: vec![],
                }],
                npc_actions: Vec::<NpcAction>::new(),
            }),
        );
        gs.engine = SceneEngine::new(scenes);

        let mut role_bindings = HashMap::new();
        role_bindings.insert("ROLE_TEAM_LEAD".to_string(), SceneNpcRef::Male(male_key));
        role_bindings.insert("ROLE_DESIGNER".to_string(), SceneNpcRef::Female(female_key));
        gs.engine.start_scene_with_role_bindings(
            "test::multi_npc".into(),
            None,
            None,
            role_bindings,
            &gs.world,
            &gs.registry,
        );
        let events = gs.engine.drain();
        let intro = events
            .iter()
            .find_map(|event| match event {
                undone_scene::engine::EngineEvent::ProseAdded(text) => Some(text.as_str()),
                _ => None,
            })
            .expect("intro prose should render");
        assert!(intro.contains("Jake"));
        assert!(intro.contains("Mia"));

        gs.engine
            .advance_with_action("approve", &mut gs.world, &gs.registry);
        assert_eq!(
            gs.world.male_npcs[male_key].core.pc_liking,
            LikingLevel::Close
        );

        let snapshot = snapshot_runtime(signals, &gs);
        assert_eq!(snapshot.active_npcs.len(), 2);
    }
}
