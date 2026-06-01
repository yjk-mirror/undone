use std::collections::{HashMap, HashSet};

use lasso::Key;
use undone_domain::*;

use super::*;
use crate::types::{SceneDefinition, Thought};
use undone_world::test_helpers::make_test_world as make_world;

/// Compile an effect call-list for tests (empty registry — flag/scene/npc only).
fn eff(src: &str) -> CompiledScript {
    crate::script::compile_effect(src, &PackRegistry::new(), "test").unwrap()
}

fn make_simple_scene() -> SceneDefinition {
    SceneDefinition {
        id: "test::simple".into(),
        pack: "test".into(),
        intro_prose: "It begins.".into(),
        intro_variants: vec![],
        intro_thoughts: vec![],
        actions: vec![
            Action {
                id: "wait".into(),
                label: "Wait".into(),
                detail: "Just wait.".into(),
                condition: None,
                prose: String::new(),
                allow_npc_actions: false,
                effect: None,
                next: vec![],
                thoughts: vec![],
            },
            Action {
                id: "leave".into(),
                label: "Leave".into(),
                detail: "Go.".into(),
                condition: None,
                prose: "You leave.".into(),
                allow_npc_actions: false,
                effect: Some(eff("w.changeStress(-1);")),
                next: vec![NextBranch {
                    condition: None,
                    goto: None,
                    slot: None,
                    finish: true,
                }],
                thoughts: vec![],
            },
        ],
        npc_actions: vec![],
    }
}

fn make_engine_with(scene: SceneDefinition) -> SceneEngine {
    let mut scenes = HashMap::new();
    scenes.insert(scene.id.clone(), Arc::new(scene));
    SceneEngine::new(scenes)
}

#[test]
fn start_scene_emits_prose_and_actions() {
    let mut engine = make_engine_with(make_simple_scene());
    let mut world = make_world();
    let registry = undone_packs::PackRegistry::new();

    engine.send(
        EngineCommand::StartScene("test::simple".into()),
        &mut world,
        &registry,
    );

    let events = engine.drain();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, EngineEvent::ProseAdded(_))),
        "expected ProseAdded"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, EngineEvent::ActionsAvailable(_))),
        "expected ActionsAvailable"
    );
}

#[test]
fn start_scene_with_role_bindings_exposes_bound_npcs() {
    let mut engine = make_engine_with(make_simple_scene());
    let mut world = make_world();
    let mut registry = undone_packs::PackRegistry::new();
    let romantic = registry.intern_personality("ROMANTIC");
    let calm = registry.intern_personality("CALM");
    let male_key = world.male_npcs.insert(MaleNpc {
        core: NpcCore {
            name: "Dan".into(),
            display_name: None,
            age: Age::MidLateTwenties,
            race: "white".into(),
            eye_colour: "blue".into(),
            hair_colour: "brown".into(),
            personality: romantic,
            traits: HashSet::new(),
            relationship: RelationshipStatus::Acquaintance,
            pc_liking: LikingLevel::Like,
            npc_liking: LikingLevel::Neutral,
            pc_love: LoveLevel::None,
            npc_love: LoveLevel::None,
            pc_attraction: AttractionLevel::Attracted,
            npc_attraction: AttractionLevel::Ok,
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
    });
    let female_key = world.female_npcs.insert(FemaleNpc {
        core: NpcCore {
            name: "Mia".into(),
            display_name: None,
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
        char_type: CharTypeId::from_spur(lasso::Spur::try_from_usize(0).unwrap()),
        figure: PlayerFigure::Slim,
        breasts: BreastSize::Average,
        clothing: FemaleClothing::default(),
        pregnancy: None,
        virgin: true,
    });
    let mut role_bindings = HashMap::new();
    role_bindings.insert("ROLE_TEAM_LEAD".to_string(), SceneNpcRef::Male(male_key));
    role_bindings.insert("ROLE_DESIGNER".to_string(), SceneNpcRef::Female(female_key));

    engine.start_scene_with_role_bindings(
        "test::simple".into(),
        None,
        None,
        role_bindings,
        &world,
        &registry,
    );
    engine.drain();

    let bound = engine.current_bound_npcs(&world, &registry);
    assert_eq!(bound.len(), 2);
    assert_eq!(bound[0].binding, "ROLE_DESIGNER");
    assert_eq!(bound[0].npc.name, "Mia");
    assert_eq!(bound[1].binding, "ROLE_TEAM_LEAD");
    assert_eq!(bound[1].npc.name, "Dan");
}

#[test]
fn choose_action_with_finish_emits_scene_finished() {
    let mut engine = make_engine_with(make_simple_scene());
    let mut world = make_world();
    let registry = undone_packs::PackRegistry::new();

    engine.send(
        EngineCommand::StartScene("test::simple".into()),
        &mut world,
        &registry,
    );
    engine.drain();

    engine.send(
        EngineCommand::ChooseAction("leave".into()),
        &mut world,
        &registry,
    );

    let events = engine.drain();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, EngineEvent::SceneFinished)),
        "expected SceneFinished"
    );
}

#[test]
fn choose_action_applies_effects() {
    let mut engine = make_engine_with(make_simple_scene());
    let mut world = make_world();
    let registry = undone_packs::PackRegistry::new();

    engine.send(
        EngineCommand::StartScene("test::simple".into()),
        &mut world,
        &registry,
    );
    engine.drain();

    let stress_before = world.player.stress;
    engine.send(
        EngineCommand::ChooseAction("leave".into()),
        &mut world,
        &registry,
    );

    assert_eq!(
        world.player.stress.get(),
        stress_before.get() - 1,
        "stress should have decreased by 1"
    );
}

#[test]
fn choose_loop_action_re_emits_actions_available() {
    let mut engine = make_engine_with(make_simple_scene());
    let mut world = make_world();
    let registry = undone_packs::PackRegistry::new();

    engine.send(
        EngineCommand::StartScene("test::simple".into()),
        &mut world,
        &registry,
    );
    engine.drain();

    // "wait" has no next branches → should re-emit actions
    engine.send(
        EngineCommand::ChooseAction("wait".into()),
        &mut world,
        &registry,
    );

    let events = engine.drain();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, EngineEvent::ActionsAvailable(_))),
        "expected ActionsAvailable after loop action"
    );
}

#[test]
fn condition_filters_actions() {
    // Build a scene with a conditional action
    let cond_expr = crate::script::compile_condition(
        r#"scene.hasFlag("special")"#,
        &PackRegistry::new(),
        "test",
    )
    .unwrap();
    let scene = SceneDefinition {
        id: "test::conditional".into(),
        pack: "test".into(),
        intro_prose: "Conditional test.".into(),
        intro_variants: vec![],
        intro_thoughts: vec![],
        actions: vec![
            Action {
                id: "always".into(),
                label: "Always".into(),
                detail: "Always visible.".into(),
                condition: None,
                prose: String::new(),
                allow_npc_actions: false,
                effect: None,
                next: vec![],
                thoughts: vec![],
            },
            Action {
                id: "special".into(),
                label: "Special".into(),
                detail: "Only when flag set.".into(),
                condition: Some(cond_expr),
                prose: String::new(),
                allow_npc_actions: false,
                effect: None,
                next: vec![],
                thoughts: vec![],
            },
        ],
        npc_actions: vec![],
    };

    let mut engine = make_engine_with(scene);
    let mut world = make_world();
    let registry = undone_packs::PackRegistry::new();

    engine.send(
        EngineCommand::StartScene("test::conditional".into()),
        &mut world,
        &registry,
    );

    let events = engine.drain();
    let actions_event = events
        .iter()
        .find_map(|e| {
            if let EngineEvent::ActionsAvailable(v) = e {
                Some(v)
            } else {
                None
            }
        })
        .expect("expected ActionsAvailable");

    // Without the flag, only "always" should be visible
    assert_eq!(actions_event.len(), 1, "expected 1 visible action");
    assert_eq!(actions_event[0].id, "always");
}

#[test]
fn set_active_male_emits_npc_activated() {
    let scene = make_simple_scene();
    let mut engine = make_engine_with(scene);
    let mut world = make_world();
    let mut registry = undone_packs::PackRegistry::new();
    let personality_id = registry.intern_personality("ROMANTIC");

    let npc = MaleNpc {
        core: NpcCore {
            name: "Jake".into(),
            display_name: None,
            age: Age::MidLateTwenties,
            race: "white".into(),
            eye_colour: "blue".into(),
            hair_colour: "brown".into(),
            personality: personality_id,
            traits: HashSet::new(),
            relationship: RelationshipStatus::Stranger,
            pc_liking: LikingLevel::Neutral,
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
            arousal: ArousalLevel::Comfort,
            alcohol: AlcoholLevel::Sober,
            roles: HashSet::new(),
        },
        figure: MaleFigure::Average,
        clothing: MaleClothing::default(),
        had_orgasm: false,
        has_baby_with_pc: false,
    };
    let key = world.male_npcs.insert(npc);

    // Need a scene on the stack for SetActiveMale to work
    engine.send(
        EngineCommand::StartScene("test::simple".into()),
        &mut world,
        &registry,
    );
    engine.drain();

    engine.send(EngineCommand::SetActiveMale(key), &mut world, &registry);
    let events = engine.drain();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, EngineEvent::NpcActivated(Some(_)))),
        "expected NpcActivated event with data"
    );
}

#[test]
fn npc_activated_uses_display_name_when_set() {
    // Regression guard: the People Here sidebar reads from NpcActivatedData.name.
    // After a first-meeting scene tags a roled NPC with set_npc_name, the
    // sidebar must show the story name ("Jake") instead of the random
    // spawn name ("Brian"). Without this, the sidebar shows whichever
    // random name the spawner chose and the story binding is invisible
    // to the player.
    use crate::script::apply_effect_script;
    use crate::SceneCtx;

    let scene = make_simple_scene();
    let mut engine = make_engine_with(scene);
    let mut world = make_world();
    let mut registry = undone_packs::PackRegistry::new();
    let personality_id = registry.intern_personality("ROMANTIC");

    let npc = MaleNpc {
        core: NpcCore {
            name: "Brian".into(),
            display_name: None,
            age: Age::MidLateTwenties,
            race: "white".into(),
            eye_colour: "blue".into(),
            hair_colour: "brown".into(),
            personality: personality_id,
            traits: HashSet::new(),
            relationship: RelationshipStatus::Stranger,
            pc_liking: LikingLevel::Neutral,
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
            arousal: ArousalLevel::Comfort,
            alcohol: AlcoholLevel::Sober,
            roles: HashSet::new(),
        },
        figure: MaleFigure::Average,
        clothing: MaleClothing::default(),
        had_orgasm: false,
        has_baby_with_pc: false,
    };
    let key = world.male_npcs.insert(npc);

    // Apply set_npc_name through the public effect path, like a scene
    // action would. Requires an active_male in the ctx.
    let mut ctx = SceneCtx::new();
    ctx.active_male = Some(key);
    let errors = apply_effect_script(
        &eff(r#"npc("m").setName("Jake");"#),
        &mut world,
        &mut ctx,
        &registry,
    );
    assert!(errors.is_empty(), "set_npc_name must succeed: {errors:?}");

    // Now drive the engine the way the runtime would: start a scene, bind
    // the male as active, and pull the NpcActivated event.
    engine.send(
        EngineCommand::StartScene("test::simple".into()),
        &mut world,
        &registry,
    );
    engine.drain();
    engine.send(EngineCommand::SetActiveMale(key), &mut world, &registry);
    let events = engine.drain();

    let activated = events
        .iter()
        .find_map(|e| match e {
            EngineEvent::NpcActivated(Some(data)) => Some(data),
            _ => None,
        })
        .expect("expected NpcActivated event after SetActiveMale");
    assert_eq!(
        activated.name, "Jake",
        "sidebar must read the story name, not the spawn name"
    );
    assert_eq!(
        world.male_npcs[key].core.name, "Brian",
        "underlying spawn name must be preserved"
    );
}

#[test]
fn scene_finished_clears_npc_activated() {
    let mut engine = make_engine_with(make_simple_scene());
    let mut world = make_world();
    let registry = undone_packs::PackRegistry::new();

    engine.send(
        EngineCommand::StartScene("test::simple".into()),
        &mut world,
        &registry,
    );
    engine.drain();

    engine.send(
        EngineCommand::ChooseAction("leave".into()),
        &mut world,
        &registry,
    );
    let events = engine.drain();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, EngineEvent::NpcActivated(None))),
        "expected NpcActivated(None) on scene finish"
    );
}

#[test]
fn goto_transition_works_normally() {
    // Verify that a single goto transition (the common case) works
    // correctly and is not blocked by the transition guard.
    let scene_a = SceneDefinition {
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
            effect: None,
            next: vec![NextBranch {
                condition: None,
                goto: Some("test::b".into()),
                slot: None,
                finish: false,
            }],
            thoughts: vec![],
        }],
        npc_actions: vec![],
    };
    let scene_b = SceneDefinition {
        id: "test::b".into(),
        pack: "test".into(),
        intro_prose: "B".into(),
        intro_variants: vec![],
        intro_thoughts: vec![],
        actions: vec![Action {
            id: "wait".into(),
            label: "Wait".into(),
            detail: String::new(),
            condition: None,
            prose: String::new(),
            allow_npc_actions: false,
            effect: None,
            next: vec![],
            thoughts: vec![],
        }],
        npc_actions: vec![],
    };

    let mut scenes = HashMap::new();
    scenes.insert("test::a".into(), Arc::new(scene_a));
    scenes.insert("test::b".into(), Arc::new(scene_b));
    let mut engine = SceneEngine::new(scenes);
    let mut world = make_world();
    let registry = PackRegistry::new();

    engine.send(
        EngineCommand::StartScene("test::a".into()),
        &mut world,
        &registry,
    );
    engine.drain();

    engine.send(
        EngineCommand::ChooseAction("go".into()),
        &mut world,
        &registry,
    );
    let events = engine.drain();

    // Should have transitioned to scene B: intro prose + actions
    assert!(
        events
            .iter()
            .any(|e| matches!(e, EngineEvent::ProseAdded(s) if s == "B")),
        "expected scene B intro prose"
    );
    assert!(
        events.iter().any(
            |e| matches!(e, EngineEvent::ActionsAvailable(v) if v.iter().any(|a| a.id == "wait"))
        ),
        "expected scene B actions"
    );
    // No error prose
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, EngineEvent::ProseAdded(s) if s.contains("exceeded"))),
        "normal goto should not trigger transition guard"
    );
}

#[test]
fn advance_with_action_returns_events() {
    let mut engine = make_engine_with(make_simple_scene());
    let mut world = make_world();
    let registry = PackRegistry::new();

    engine.send(
        EngineCommand::StartScene("test::simple".into()),
        &mut world,
        &registry,
    );
    engine.drain();

    // advance_with_action("leave") should produce ProseAdded + NpcActivated + SceneFinished
    let events = engine.advance_with_action("leave", &mut world, &registry);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, EngineEvent::SceneFinished)),
        "expected SceneFinished from advance_with_action"
    );
}

#[test]
fn reset_runtime_clears_stack_and_pending_events() {
    let mut engine = make_engine_with(make_simple_scene());
    let mut world = make_world();
    let registry = PackRegistry::new();

    engine.send(
        EngineCommand::StartScene("test::simple".into()),
        &mut world,
        &registry,
    );
    assert!(!engine.drain().is_empty());

    engine.send(
        EngineCommand::StartScene("test::simple".into()),
        &mut world,
        &registry,
    );
    engine.reset_runtime();
    assert!(engine.drain().is_empty());

    engine.send(
        EngineCommand::ChooseAction("leave".into()),
        &mut world,
        &registry,
    );
    assert!(engine.drain().is_empty());
}

#[test]
fn start_unknown_scene_emits_error_and_finishes() {
    let mut engine = SceneEngine::new(HashMap::new());
    let mut world = make_world();
    let registry = PackRegistry::new();

    engine.send(
        EngineCommand::StartScene("nonexistent::scene".into()),
        &mut world,
        &registry,
    );

    let events = engine.drain();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, EngineEvent::ProseAdded(s) if s.contains("not found"))),
        "expected error prose for unknown scene"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, EngineEvent::SceneFinished)),
        "expected SceneFinished for unknown scene"
    );
}

#[test]
fn slot_branch_emits_slot_requested() {
    // Build a scene with an action whose next branch has slot = Some("free_time")
    let scene = SceneDefinition {
        id: "test::hub".into(),
        pack: "test".into(),
        intro_prose: "Hub scene.".into(),
        intro_variants: vec![],
        intro_thoughts: vec![],
        actions: vec![Action {
            id: "go_free".into(),
            label: "Free Time".into(),
            detail: "Choose a free time activity.".into(),
            condition: None,
            prose: String::new(),
            allow_npc_actions: false,
            effect: None,
            next: vec![NextBranch {
                condition: None,
                goto: None,
                slot: Some("free_time".into()),
                finish: false,
            }],
            thoughts: vec![],
        }],
        npc_actions: vec![],
    };

    let mut engine = make_engine_with(scene);
    let mut world = make_world();
    let registry = PackRegistry::new();

    engine.send(
        EngineCommand::StartScene("test::hub".into()),
        &mut world,
        &registry,
    );
    engine.drain();

    engine.send(
        EngineCommand::ChooseAction("go_free".into()),
        &mut world,
        &registry,
    );
    let events = engine.drain();

    assert!(
        events
            .iter()
            .any(|e| matches!(e, EngineEvent::SlotRequested(s) if s == "free_time")),
        "expected SlotRequested(\"free_time\") after choosing a slot branch"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, EngineEvent::NpcActivated(None))),
        "expected NpcActivated(None) when slot branch fires"
    );
}

#[test]
fn intro_thought_emits_thought_added() {
    let thought = Thought {
        condition: None, // unconditional — always fires
        prose: "You feel a pang of unease.".into(),
        style: "inner_voice".into(),
    };
    let scene = SceneDefinition {
        id: "test::thought".into(),
        pack: "test".into(),
        intro_prose: "The rain hammers the shelter roof.".into(),
        intro_variants: vec![],
        intro_thoughts: vec![thought],
        actions: vec![],
        npc_actions: vec![],
    };

    let mut engine = make_engine_with(scene);
    let mut world = make_world();
    let registry = PackRegistry::new();

    engine.send(
        EngineCommand::StartScene("test::thought".into()),
        &mut world,
        &registry,
    );
    let events = engine.drain();

    assert!(
        events.iter().any(|e| matches!(
            e,
            EngineEvent::ThoughtAdded { text, style }
                if text.contains("unease") && style == "inner_voice"
        )),
        "expected ThoughtAdded with 'unease' prose"
    );
}

#[test]
fn action_thought_emits_after_action_prose() {
    let thought = Thought {
        condition: None,
        prose: "Was that really the right call?".into(),
        style: "anxiety".into(),
    };
    let scene = SceneDefinition {
        id: "test::action_thought".into(),
        pack: "test".into(),
        intro_prose: "Shelter.".into(),
        intro_variants: vec![],
        intro_thoughts: vec![],
        actions: vec![Action {
            id: "decide".into(),
            label: "Decide".into(),
            detail: String::new(),
            condition: None,
            prose: "You make a decision.".into(),
            allow_npc_actions: false,
            effect: None,
            next: vec![NextBranch {
                condition: None,
                goto: None,
                slot: None,
                finish: true,
            }],
            thoughts: vec![thought],
        }],
        npc_actions: vec![],
    };

    let mut engine = make_engine_with(scene);
    let mut world = make_world();
    let registry = PackRegistry::new();

    engine.send(
        EngineCommand::StartScene("test::action_thought".into()),
        &mut world,
        &registry,
    );
    engine.drain();

    engine.send(
        EngineCommand::ChooseAction("decide".into()),
        &mut world,
        &registry,
    );
    let events = engine.drain();

    assert!(
        events.iter().any(|e| matches!(
            e,
            EngineEvent::ThoughtAdded { text, style }
                if text.contains("right call") && style == "anxiety"
        )),
        "expected ThoughtAdded with 'right call' prose after action"
    );
}

#[test]
fn effect_error_emits_error_occurred_event() {
    // An effect targeting an unbound npc ref ("male" is not "m"/"f"/a bound
    // role) — the mutator records an error. The engine must emit ErrorOccurred
    // rather than silently eprintln.
    let scene = SceneDefinition {
        id: "test::effect_error".into(),
        pack: "test".into(),
        intro_prose: "Something happens.".into(),
        intro_variants: vec![],
        intro_thoughts: vec![],
        actions: vec![Action {
            id: "go".into(),
            label: "Go".into(),
            detail: String::new(),
            condition: None,
            prose: String::new(),
            allow_npc_actions: false,
            effect: Some(eff(r#"npc("male").addLiking(1);"#)),
            next: vec![NextBranch {
                condition: None,
                goto: None,
                slot: None,
                finish: true,
            }],
            thoughts: vec![],
        }],
        npc_actions: vec![],
    };

    let mut engine = make_engine_with(scene);
    let mut world = make_world(); // no active male NPC
    let registry = undone_packs::PackRegistry::new();

    engine.send(
        EngineCommand::StartScene("test::effect_error".into()),
        &mut world,
        &registry,
    );
    engine.drain();

    engine.send(
        EngineCommand::ChooseAction("go".into()),
        &mut world,
        &registry,
    );
    let events = engine.drain();

    assert!(
        events
            .iter()
            .any(|e| matches!(e, EngineEvent::ErrorOccurred(_))),
        "expected ErrorOccurred event when effect fails; got: {:?}",
        events
    );
}

#[test]
fn action_condition_error_emits_error_occurred_and_hides_action() {
    let cond =
        crate::script::compile_condition(r#"m.hasFlag("READY")"#, &PackRegistry::new(), "test")
            .unwrap();
    let scene = SceneDefinition {
        id: "test::condition_error".into(),
        pack: "test".into(),
        intro_prose: "Condition test.".into(),
        intro_variants: vec![],
        intro_thoughts: vec![],
        actions: vec![
            Action {
                id: "safe".into(),
                label: "Safe".into(),
                detail: String::new(),
                condition: None,
                prose: String::new(),
                allow_npc_actions: false,
                effect: None,
                next: vec![],
                thoughts: vec![],
            },
            Action {
                id: "broken".into(),
                label: "Broken".into(),
                detail: String::new(),
                condition: Some(cond),
                prose: String::new(),
                allow_npc_actions: false,
                effect: None,
                next: vec![],
                thoughts: vec![],
            },
        ],
        npc_actions: vec![],
    };

    let mut engine = make_engine_with(scene);
    let mut world = make_world();
    let registry = PackRegistry::new();

    engine.send(
        EngineCommand::StartScene("test::condition_error".into()),
        &mut world,
        &registry,
    );
    let events = engine.drain();

    assert!(
        events.iter().any(|event| matches!(
            event,
            EngineEvent::ErrorOccurred(msg) if msg.contains("condition error")
        )),
        "expected visible diagnostic for action condition error, got {:?}",
        events
    );

    let action_ids = events
        .iter()
        .find_map(|event| {
            if let EngineEvent::ActionsAvailable(actions) = event {
                Some(
                    actions
                        .iter()
                        .map(|action| action.id.as_str())
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            }
        })
        .expect("scene should emit actions");
    assert_eq!(action_ids, vec!["safe"]);
}

#[test]
fn intro_template_error_emits_error_occurred() {
    let scene = SceneDefinition {
        id: "test::template_error".into(),
        pack: "test".into(),
        intro_prose: "{{ m.getLiking() }}".into(),
        intro_variants: vec![],
        intro_thoughts: vec![],
        actions: vec![],
        npc_actions: vec![],
    };

    let mut engine = make_engine_with(scene);
    let mut world = make_world();
    let registry = PackRegistry::new();

    engine.send(
        EngineCommand::StartScene("test::template_error".into()),
        &mut world,
        &registry,
    );
    let events = engine.drain();

    assert!(
        events.iter().any(|event| matches!(
            event,
            EngineEvent::ErrorOccurred(msg) if msg.contains("template error")
        )),
        "expected visible diagnostic for template error, got {:?}",
        events
    );
    assert!(
            !events
                .iter()
                .any(|event| matches!(event, EngineEvent::ProseAdded(text) if text.contains("template error"))),
            "template failures should surface through ErrorOccurred, not ad-hoc prose: {:?}",
            events
        );
}
