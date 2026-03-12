pub mod effects;
pub mod engine;
pub mod loader;
pub mod reachability;
pub mod scheduler;
pub mod simulator;
pub mod template_ctx;
pub mod types;

pub use effects::{apply_effect, EffectError};
pub use engine::{ActionView, EngineCommand, EngineEvent, NpcActivatedData, SceneEngine};
pub use loader::{load_scenes, validate_cross_references, SceneLoadError};
pub use scheduler::{
    load_schedule, validate_entry_scene_references, PickResult, Scheduler, SchedulerError,
};
pub use types::{Action, EffectDef, NextBranch, NpcAction, SceneDefinition, SceneMeta, SceneToml};
pub use undone_expr::SceneNpcRef;

#[cfg(test)]
mod integration_tests {
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;
    use std::sync::Arc;

    use lasso::Key;
    use undone_domain::*;
    use undone_packs::load_packs;
    use undone_world::World;

    use crate::engine::{EngineCommand, EngineEvent, SceneEngine};
    use crate::loader::load_scenes;
    use crate::EffectDef;
    use undone_world::test_helpers::make_test_world;

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("packs")
    }

    fn make_world_with_shy(registry: &undone_packs::PackRegistry) -> World {
        let shy_id = registry.resolve_trait("SHY").unwrap();
        let mut world = make_test_world();
        world.player.traits.insert(shy_id);
        world
    }

    fn make_male_npc() -> MaleNpc {
        MaleNpc {
            core: NpcCore {
                name: "Stranger".into(),
                age: Age::Thirties,
                race: "white".into(),
                eye_colour: "grey".into(),
                hair_colour: "brown".into(),
                personality: PersonalityId::from_spur(lasso::Spur::try_from_usize(0).unwrap()),
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
        }
    }

    fn load_base_content() -> (
        undone_packs::PackRegistry,
        crate::scheduler::Scheduler,
        HashMap<String, Arc<crate::types::SceneDefinition>>,
    ) {
        let (registry, metas) = load_packs(&packs_dir()).unwrap();
        let scheduler = crate::scheduler::load_schedule(&metas, &registry).unwrap();
        let scenes_dir = packs_dir().join("base").join("scenes");
        let scenes = load_scenes(&scenes_dir, &registry).unwrap();
        (registry, scheduler, scenes)
    }

    fn make_robin_world(registry: &undone_packs::PackRegistry) -> World {
        make_world_with_shy(registry)
    }

    fn start_scene_with_male_binding(
        scene_id: &str,
    ) -> (
        undone_packs::PackRegistry,
        World,
        SceneEngine,
        MaleNpcKey,
    ) {
        let (registry, _scheduler, scenes) = load_base_content();
        let mut world = make_robin_world(&registry);
        let male_npc_key = world.male_npcs.insert(make_male_npc());
        let mut engine = SceneEngine::new(scenes);
        engine.send(
            EngineCommand::StartScene(scene_id.to_string()),
            &mut world,
            &registry,
        );
        engine.send(
            EngineCommand::SetActiveMale(male_npc_key),
            &mut world,
            &registry,
        );
        engine.drain();
        (registry, world, engine, male_npc_key)
    }

    fn render_scene_intro(scene_id: &str, world: &mut World) -> String {
        let (registry, _scheduler, scenes) = load_base_content();
        let mut engine = SceneEngine::new(scenes);
        engine.send(
            EngineCommand::StartScene(scene_id.to_string()),
            world,
            &registry,
        );
        let events = engine.drain();
        events
            .into_iter()
            .filter_map(|event| {
                if let EngineEvent::ProseAdded(text) = event {
                    Some(text)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn rain_shelter_full_flow() {
        // 1. Load packs
        let (registry, metas) = undone_packs::load_packs(&packs_dir()).unwrap();
        assert!(!metas.is_empty());

        // 2. Load scenes
        let scenes_dir = packs_dir().join("base").join("scenes");
        let scenes = load_scenes(&scenes_dir, &registry).unwrap();
        assert!(scenes.contains_key("base::rain_shelter"));

        // 3. Create world with SHY player
        let mut world = make_world_with_shy(&registry);

        // 4. Build engine
        let mut engine = SceneEngine::new(scenes);

        // 5. Start scene
        engine.send(
            EngineCommand::StartScene("base::rain_shelter".into()),
            &mut world,
            &registry,
        );
        let events = engine.drain();

        // 6. Assert intro prose renders (non-BEAUTIFUL default branch)
        let prose_events: Vec<&str> = events
            .iter()
            .filter_map(|e| {
                if let EngineEvent::ProseAdded(p) = e {
                    Some(p.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert!(!prose_events.is_empty(), "intro prose should be emitted");
        let all_prose = prose_events.join("\n");
        assert!(
            all_prose.contains("Quick read"),
            "non-BEAUTIFUL branch should appear in intro"
        );

        // 7. Assert initial actions (main + leave, NOT accept_umbrella yet)
        let actions_event = events
            .iter()
            .find_map(|e| {
                if let EngineEvent::ActionsAvailable(a) = e {
                    Some(a)
                } else {
                    None
                }
            })
            .unwrap();
        let ids: Vec<&str> = actions_event.iter().map(|a| a.id.as_str()).collect();
        assert!(ids.contains(&"main"), "main should be available");
        assert!(ids.contains(&"leave"), "leave should be available");
        assert!(
            !ids.contains(&"accept_umbrella"),
            "accept_umbrella not available yet"
        );
    }

    #[test]
    fn workplace_arc_full_playthrough() {
        use crate::scheduler::load_schedule;
        use rand::rngs::SmallRng;
        use rand::SeedableRng;

        // Load packs + schedule + scenes
        let (registry, metas) = undone_packs::load_packs(&packs_dir()).unwrap();
        let scheduler = load_schedule(&metas, &registry).unwrap();
        let scenes_dir = packs_dir().join("base").join("scenes");
        let scenes = load_scenes(&scenes_dir, &registry).unwrap();

        // Start world with ROUTE_WORKPLACE flag (what the Robin preset provides)
        let mut world = make_world_with_shy(&registry);
        world.game_data.set_flag("ROUTE_WORKPLACE");
        // Spawn a male NPC (Marcus stand-in) so scenes with set_npc_role / add_npc_liking work
        let male_npc_key = world.male_npcs.insert(make_male_npc());

        let mut engine = SceneEngine::new(scenes);
        let mut rng = SmallRng::seed_from_u64(42);

        let mut visited: Vec<String> = Vec::new();
        let mut all_errors: Vec<String> = Vec::new();

        // Simulate the arc from start to settled
        'arc: for _ in 0..30 {
            let Some(pick) = scheduler.pick_next(&world, &registry, &mut rng) else {
                break;
            };

            let scene_id = pick.scene_id.clone();

            // Game loop responsibility: mark once-only scenes as played
            if pick.once_only {
                world.game_data.set_flag(format!("ONCE_{}", scene_id));
            }

            visited.push(scene_id.clone());

            engine.send(
                EngineCommand::StartScene(scene_id.clone()),
                &mut world,
                &registry,
            );
            // Wire in the male NPC so effects like set_npc_role and add_npc_liking work
            engine.send(
                EngineCommand::SetActiveMale(male_npc_key),
                &mut world,
                &registry,
            );

            // Play through the scene until SceneFinished
            for _ in 0..10 {
                let events = engine.drain();

                for e in &events {
                    if let EngineEvent::ErrorOccurred(msg) = e {
                        all_errors.push(format!("[{}] {}", scene_id, msg));
                    }
                }

                if events
                    .iter()
                    .any(|e| matches!(e, EngineEvent::SceneFinished))
                {
                    break;
                }

                let available = events.iter().find_map(|e| {
                    if let EngineEvent::ActionsAvailable(a) = e {
                        Some(a.clone())
                    } else {
                        None
                    }
                });

                match available {
                    Some(actions) if !actions.is_empty() => {
                        engine.send(
                            EngineCommand::ChooseAction(actions[0].id.clone()),
                            &mut world,
                            &registry,
                        );
                    }
                    _ => break, // no actions and not finished
                }
            }

            if world.game_data.arc_state("base::workplace_opening") == Some("settled") {
                break 'arc;
            }
        }

        let expected = [
            "base::workplace_arrival",
            "base::workplace_landlord",
            "base::workplace_first_night",
            "base::workplace_first_clothes",
            "base::workplace_first_day",
            "base::workplace_work_meeting",
            "base::workplace_evening",
        ];
        for scene in &expected {
            assert!(
                visited.iter().any(|v| v == scene),
                "scene '{}' was never visited; visited order: {:?}",
                scene,
                visited
            );
        }
        assert!(
            all_errors.is_empty(),
            "unexpected errors during arc playthrough: {:?}",
            all_errors
        );
    }

    #[test]
    fn femininity_reaches_25_by_workplace_arc_end() {
        use crate::scheduler::load_schedule;
        use rand::rngs::SmallRng;
        use rand::SeedableRng;

        let (registry, metas) = undone_packs::load_packs(&packs_dir()).unwrap();
        let scheduler = load_schedule(&metas, &registry).unwrap();
        let scenes_dir = packs_dir().join("base").join("scenes");
        let scenes = load_scenes(&scenes_dir, &registry).unwrap();

        let mut world = make_world_with_shy(&registry);
        world.game_data.set_flag("ROUTE_WORKPLACE");

        // Set FEMININITY to 10 — CisMaleTransformed starting value (new_game() does this)
        let fem_id = registry
            .resolve_skill("FEMININITY")
            .expect("FEMININITY skill must be registered");
        world.player.skills.insert(
            fem_id,
            SkillValue {
                value: 10,
                modifier: 0,
            },
        );

        let mut engine = SceneEngine::new(scenes);
        let mut rng = SmallRng::seed_from_u64(42);

        // Simulate full workplace arc
        'arc: for _ in 0..30 {
            let Some(pick) = scheduler.pick_next(&world, &registry, &mut rng) else {
                break;
            };
            let scene_id = pick.scene_id.clone();
            if pick.once_only {
                world.game_data.set_flag(format!("ONCE_{}", scene_id));
            }

            engine.send(
                EngineCommand::StartScene(scene_id.clone()),
                &mut world,
                &registry,
            );

            for _ in 0..10 {
                let events = engine.drain();
                if events
                    .iter()
                    .any(|e| matches!(e, EngineEvent::SceneFinished))
                {
                    break;
                }
                let available = events.iter().find_map(|e| {
                    if let EngineEvent::ActionsAvailable(a) = e {
                        Some(a.clone())
                    } else {
                        None
                    }
                });
                match available {
                    Some(actions) if !actions.is_empty() => {
                        engine.send(
                            EngineCommand::ChooseAction(actions[0].id.clone()),
                            &mut world,
                            &registry,
                        );
                    }
                    _ => break,
                }
            }

            if world.game_data.arc_state("base::workplace_opening") == Some("settled") {
                break 'arc;
            }
        }

        assert_eq!(
            world.game_data.arc_state("base::workplace_opening"),
            Some("settled"),
            "arc should have reached settled"
        );

        let femininity = world.player.skill(fem_id);
        assert!(
            femininity >= 25,
            "FEMININITY should be >= 25 after arc completion, got {}",
            femininity
        );
    }

    #[test]
    fn work_slot_fires_when_settled() {
        use crate::scheduler::load_schedule;
        use rand::rngs::SmallRng;
        use rand::SeedableRng;

        let (registry, metas) = undone_packs::load_packs(&packs_dir()).unwrap();
        let scheduler = load_schedule(&metas, &registry).unwrap();

        let mut world = make_world_with_shy(&registry);
        // Set arc to settled state (post-arc)
        world.game_data.set_flag("ROUTE_WORKPLACE");
        world
            .game_data
            .arc_states
            .insert("base::workplace_opening".to_string(), "settled".to_string());
        world.game_data.week = 3;

        // The work slot should produce at least one eligible event with the settled arc state.
        let mut rng = SmallRng::seed_from_u64(42);
        let result = scheduler.pick("work", &world, &registry, &mut rng);
        assert!(
            result.is_some(),
            "No work scene scheduled in settled state — work slot missing or conditions wrong"
        );
        assert!(
            result.unwrap().scene_id.starts_with("base::work_"),
            "Scheduled scene should start with 'base::work_'"
        );
    }

    #[test]
    fn rain_shelter_npc_fires_and_umbrella_becomes_available() {
        let (registry, _) = undone_packs::load_packs(&packs_dir()).unwrap();
        let scenes_dir = packs_dir().join("base").join("scenes");
        let scenes = load_scenes(&scenes_dir, &registry).unwrap();

        let mut world = make_world_with_shy(&registry);
        let npc_key = world.male_npcs.insert(make_male_npc());
        assert_eq!(
            world.male_npcs[npc_key].core.pc_liking,
            LikingLevel::Neutral
        );

        let mut engine = SceneEngine::new(scenes);

        // Start scene + wire NPC
        engine.send(
            EngineCommand::StartScene("base::rain_shelter".into()),
            &mut world,
            &registry,
        );
        engine.send(EngineCommand::SetActiveMale(npc_key), &mut world, &registry);
        engine.drain();

        // Pick "main" (allow_npc_actions = true) — NPC should fire and set umbrella_offered.
        // The NPC action always fires when condition passes (umbrella not yet offered, weight=10),
        // so after choosing "main" accept_umbrella should be visible.
        engine.send(
            EngineCommand::ChooseAction("main".into()),
            &mut world,
            &registry,
        );
        let events = engine.drain();

        // accept_umbrella should now be visible
        let actions = events
            .iter()
            .find_map(|e| {
                if let EngineEvent::ActionsAvailable(a) = e {
                    Some(a)
                } else {
                    None
                }
            })
            .unwrap();
        let ids: Vec<&str> = actions.iter().map(|a| a.id.as_str()).collect();
        assert!(
            ids.contains(&"accept_umbrella"),
            "accept_umbrella should be visible after NPC fires"
        );

        // Accept umbrella — finish scene
        engine.send(
            EngineCommand::ChooseAction("accept_umbrella".into()),
            &mut world,
            &registry,
        );
        let events = engine.drain();
        assert!(events
            .iter()
            .any(|e| matches!(e, EngineEvent::SceneFinished)));

        // NPC pc_liking should have increased by 1 step (Neutral → Ok)
        assert_eq!(world.male_npcs[npc_key].core.pc_liking, LikingLevel::Ok);
    }

    #[test]
    fn gd_npc_liking_returns_liking_for_npc_with_role() {
        use undone_expr::{eval, parser::parse, SceneCtx};
        let (registry, _metas) = undone_packs::load_packs(&packs_dir()).unwrap();
        let mut world = make_world_with_shy(&registry);
        let ctx = SceneCtx::new();

        // Assign ROLE_TEST to one male NPC and set its liking to Ok
        let key = world.male_npcs.insert(make_male_npc());
        world.male_npcs[key].core.pc_liking = LikingLevel::Ok;
        world.male_npcs[key]
            .core
            .roles
            .insert("ROLE_TEST".to_string());

        let expr = parse("gd.npcLiking('ROLE_TEST') == 'Ok'").unwrap();
        assert!(eval(&expr, &world, &ctx, &registry).unwrap());
    }

    #[test]
    fn workplace_opening_branch_contract_exposes_memory_flags_and_callbacks() {
        use crate::scheduler::load_schedule;

        let (registry, metas) = undone_packs::load_packs(&packs_dir()).unwrap();
        let scheduler = load_schedule(&metas, &registry).unwrap();
        let scenes_dir = packs_dir().join("base").join("scenes");
        let scenes = load_scenes(&scenes_dir, &registry).unwrap();

        let landlord = &scenes["base::workplace_landlord"];
        let landlord_actions: Vec<&str> =
            landlord.actions.iter().map(|action| action.id.as_str()).collect();
        assert!(landlord_actions.contains(&"keep_it_transactional"));
        assert!(landlord
            .actions
            .iter()
            .find(|action| action.id == "wait_him_out")
            .unwrap()
            .effects
            .contains(&EffectDef::SetGameFlag {
                flag: "LANDLORD_WAITED_HIM_OUT".into(),
            }));
        assert!(landlord
            .actions
            .iter()
            .find(|action| action.id == "explain_briefly")
            .unwrap()
            .effects
            .contains(&EffectDef::SetGameFlag {
                flag: "LANDLORD_EXPLAINED_BRIEFLY".into(),
            }));

        let first_night = &scenes["base::workplace_first_night"];
        let first_night_actions: Vec<&str> = first_night
            .actions
            .iter()
            .map(|action| action.id.as_str())
            .collect();
        assert!(first_night_actions.contains(&"unpack_and_stage_tomorrow"));
        assert!(first_night
            .actions
            .iter()
            .find(|action| action.id == "order_food_sleep")
            .unwrap()
            .effects
            .contains(&EffectDef::SetGameFlag {
                flag: "FIRST_NIGHT_CRASHED".into(),
            }));

        let first_clothes = &scenes["base::workplace_first_clothes"];
        let first_clothes_actions: Vec<&str> = first_clothes
            .actions
            .iter()
            .map(|action| action.id.as_str())
            .collect();
        assert!(first_clothes_actions.contains(&"ask_for_help_outright"));
        assert!(first_clothes_actions.contains(&"buy_minimum_and_leave"));

        let first_day = &scenes["base::workplace_first_day"];
        let first_day_actions: Vec<&str> = first_day
            .actions
            .iter()
            .map(|action| action.id.as_str())
            .collect();
        assert!(first_day_actions.contains(&"redirect_to_work"));
        assert!(first_day
            .actions
            .iter()
            .find(|action| action.id == "assert_expertise")
            .unwrap()
            .effects
            .contains(&EffectDef::SetGameFlag {
                flag: "FIRST_DAY_ASSERTED_STATUS".into(),
            }));
        assert!(first_day
            .actions
            .iter()
            .find(|action| action.id == "lunch_with_group")
            .unwrap()
            .effects
            .contains(&EffectDef::SetGameFlag {
                flag: "FIRST_DAY_LUNCH_GROUP".into(),
            }));

        assert!(scenes.contains_key("base::opening_callback_status_assertion"));
        assert!(scenes.contains_key("base::opening_callback_mirror_afterglow"));
        assert!(scenes.contains_key("base::opening_callback_first_week_solitude"));
        assert!(scenes.contains_key("base::opening_callback_transactional_defense"));

        let scheduled_scene_ids = scheduler.all_scene_ids();
        assert!(scheduled_scene_ids.contains(&"base::opening_callback_status_assertion".into()));
        assert!(scheduled_scene_ids.contains(&"base::opening_callback_mirror_afterglow".into()));
        assert!(scheduled_scene_ids.contains(&"base::opening_callback_first_week_solitude".into()));
        assert!(scheduled_scene_ids.contains(&"base::opening_callback_transactional_defense".into()));
    }

    #[test]
    fn jake_apartment_explicit_path_updates_persistent_sexual_state() {
        let (registry, mut world, mut engine, male_npc_key) =
            start_scene_with_male_binding("base::jake_apartment");

        let events = engine.advance_with_action("let_him_lead", &mut world, &registry);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, EngineEvent::SceneFinished)),
            "explicit scene should finish after selecting the primary action"
        );

        assert!(
            !world.player.virgin,
            "jake_apartment should clear player virginity on the explicit path"
        );
        assert_eq!(
            world.player.partner,
            Some(NpcKey::Male(male_npc_key)),
            "Jake route should preserve romantic continuity through player partner"
        );
        let npc = world
            .male_npc(male_npc_key)
            .expect("active male npc should still exist");
        assert!(
            npc.core.sexual_activities.contains("vaginal"),
            "jake_apartment should record vaginal sexual activity"
        );
    }

    #[test]
    fn jake_kiss_and_see_path_updates_full_romantic_intimacy_state() {
        let (registry, mut world, mut engine, male_npc_key) =
            start_scene_with_male_binding("base::jake_apartment");

        let events = engine.advance_with_action("kiss_and_see", &mut world, &registry);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, EngineEvent::SceneFinished)),
            "kiss_and_see should finish after the forward romantic action"
        );

        assert!(
            !world.player.virgin,
            "kiss_and_see should clear player virginity once the route commits to full sex"
        );
        assert_eq!(
            world.player.partner,
            Some(NpcKey::Male(male_npc_key)),
            "kiss_and_see should establish the same romantic continuity as the other Jake payoff paths"
        );
        let npc = world
            .male_npc(male_npc_key)
            .expect("active male npc should still exist");
        assert!(
            npc.core.sexual_activities.contains("vaginal"),
            "kiss_and_see should record vaginal sexual activity"
        );
    }

    #[test]
    fn marcus_closet_explicit_path_updates_persistent_sexual_state() {
        let (registry, mut world, mut engine, male_npc_key) =
            start_scene_with_male_binding("base::work_marcus_closet");

        let events = engine.advance_with_action("close_the_door", &mut world, &registry);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, EngineEvent::SceneFinished)),
            "Marcus explicit scene should finish after choosing the forward action"
        );

        assert!(
            !world.player.virgin,
            "work_marcus_closet should clear player virginity on the explicit path"
        );
        let npc = world
            .male_npc(male_npc_key)
            .expect("active male npc should still exist");
        assert!(
            npc.core.sexual_activities.contains("vaginal"),
            "work_marcus_closet should record vaginal sexual activity"
        );
    }

    #[test]
    fn bar_stranger_explicit_path_updates_persistent_sexual_state() {
        let (registry, mut world, mut engine, male_npc_key) =
            start_scene_with_male_binding("base::bar_stranger_night");

        let events = engine.advance_with_action("wait_let_him_move", &mut world, &registry);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, EngineEvent::SceneFinished)),
            "bar stranger scene should finish after the explicit forward action"
        );

        assert!(
            !world.player.virgin,
            "bar_stranger_night should clear player virginity on the explicit path"
        );
        let npc = world
            .male_npc(male_npc_key)
            .expect("active male npc should still exist");
        assert!(
            npc.core.sexual_activities.contains("vaginal"),
            "bar_stranger_night should record vaginal sexual activity"
        );
    }

    #[test]
    fn jake_apartment_can_trigger_in_week_four_once_second_date_is_done() {
        let (registry, scheduler, _scenes) = load_base_content();
        let mut world = make_robin_world(&registry);
        world.game_data.week = 4;
        world.game_data.set_flag("MET_JAKE");
        world.game_data.set_flag("JAKE_FIRST_DATE");
        world.game_data.set_flag("JAKE_SECOND_DATE");
        world.game_data.set_flag("ONCE_base::coffee_shop");
        world.game_data.set_flag("ONCE_base::plan_your_day");
        world.game_data.set_flag("ONCE_base::neighborhood_bar");

        let pick = scheduler.check_triggers("free_time", &world, &registry);
        assert!(
            matches!(pick, Some(ref result) if result.scene_id == "base::jake_apartment"),
            "week-4 Jake timing should allow jake_apartment, got {:?}",
            pick.map(|result| result.scene_id)
        );
    }

    #[test]
    fn party_stranger_outside_has_follow_up_trigger() {
        let (registry, scheduler, _scenes) = load_base_content();
        let mut world = make_robin_world(&registry);
        world.game_data.week = 4;
        world.game_data.set_flag("PARTY_STRANGER_OUTSIDE");
        world.game_data.set_flag("ONCE_base::coffee_shop");
        world.game_data.set_flag("ONCE_base::plan_your_day");
        world.game_data.set_flag("ONCE_base::neighborhood_bar");

        let pick = scheduler.check_triggers("free_time", &world, &registry);
        assert!(
            matches!(pick, Some(ref result) if result.scene_id == "base::party_stranger_after"),
            "party outside branch should schedule a dedicated follow-up, got {:?}",
            pick.map(|result| result.scene_id)
        );
    }

    #[test]
    fn party_stranger_after_explicit_path_updates_persistent_state() {
        let (registry, mut world, mut engine, male_npc_key) =
            start_scene_with_male_binding("base::party_stranger_after");

        let events = engine.advance_with_action("go_with_him", &mut world, &registry);
        assert!(
            events
                .iter()
                .any(|event| matches!(event, EngineEvent::SceneFinished)),
            "party stranger follow-up should finish after the explicit forward action"
        );

        assert!(
            !world.player.virgin,
            "party stranger follow-up should clear player virginity on first-time play"
        );
        let npc = world
            .male_npc(male_npc_key)
            .expect("active male npc should still exist");
        assert!(
            npc.core.sexual_activities.contains("vaginal"),
            "party stranger follow-up should record vaginal sexual activity"
        );
    }

    #[test]
    fn transactional_defense_callback_is_scheduled_for_transactional_opening_branch() {
        let (_registry, scheduler, _scenes) = load_base_content();
        assert!(
            scheduler
                .all_scene_ids()
                .contains(&"base::opening_callback_transactional_defense".to_string()),
            "transactional landlord branch should have a scheduled callback scene"
        );
        assert!(
            scheduler.references_game_flag("LANDLORD_KEPT_TRANSACTIONAL"),
            "schedule should reference LANDLORD_KEPT_TRANSACTIONAL once the callback is live"
        );
    }

    #[test]
    fn mirror_afterglow_handles_functional_clothes_branch() {
        let (registry, _scheduler, _scenes) = load_base_content();
        let mut world = make_robin_world(&registry);
        world.game_data.set_flag("FIRST_CLOTHES_FUNCTIONAL");

        let prose = render_scene_intro("base::opening_callback_mirror_afterglow", &mut world);
        assert!(
            prose.contains("efficiently") || prose.contains("impersonally"),
            "functional first-clothes branch should have dedicated callback prose, got: {prose}"
        );
    }

    #[test]
    fn work_lunch_changes_based_on_first_day_lunch_memory() {
        let (registry, _scheduler, _scenes) = load_base_content();
        let mut desk_world = make_robin_world(&registry);
        desk_world.game_data.set_flag("FIRST_DAY_LUNCH_DESK");
        let desk_prose = render_scene_intro("base::work_lunch", &mut desk_world);

        let mut group_world = make_robin_world(&registry);
        group_world.game_data.set_flag("FIRST_DAY_LUNCH_GROUP");
        let group_prose = render_scene_intro("base::work_lunch", &mut group_world);

        assert_ne!(
            desk_prose, group_prose,
            "later work lunch should materially differ based on first-day lunch posture"
        );
    }

    #[test]
    fn mirror_afterglow_is_not_trigger_first_once_week_two_opens_up() {
        let (registry, scheduler, _scenes) = load_base_content();
        let mut world = make_robin_world(&registry);
        world.game_data.week = 2;
        world.game_data.set_flag("ROUTE_WORKPLACE");
        world
            .game_data
            .arc_states
            .insert("base::workplace_opening".into(), "settled".into());
        world.game_data.set_flag("FIRST_CLOTHES_MIRROR");
        world.game_data.set_flag("ONCE_base::coffee_shop");
        world.game_data.set_flag("ONCE_base::plan_your_day");
        world.game_data.set_flag("ONCE_base::neighborhood_bar");

        let pick = scheduler.check_triggers("free_time", &world, &registry);
        assert!(
            !matches!(pick, Some(ref result) if result.scene_id == "base::opening_callback_mirror_afterglow"),
            "mirror afterglow should blend into weighted free-time content, got {:?}",
            pick.map(|result| result.scene_id)
        );
    }

    #[test]
    fn first_week_solitude_is_not_trigger_first_once_week_two_opens_up() {
        let (registry, scheduler, _scenes) = load_base_content();
        let mut world = make_robin_world(&registry);
        world.game_data.week = 2;
        world.game_data.set_flag("ROUTE_WORKPLACE");
        world
            .game_data
            .arc_states
            .insert("base::workplace_opening".into(), "settled".into());
        world.game_data.set_flag("FIRST_NIGHT_RESEARCHED");
        world.game_data.set_flag("ONCE_base::coffee_shop");
        world.game_data.set_flag("ONCE_base::plan_your_day");
        world.game_data.set_flag("ONCE_base::neighborhood_bar");

        let pick = scheduler.check_triggers("free_time", &world, &registry);
        assert!(
            !matches!(pick, Some(ref result) if result.scene_id == "base::opening_callback_first_week_solitude"),
            "first-week solitude should blend into weighted free-time content, got {:?}",
            pick.map(|result| result.scene_id)
        );
    }
}
