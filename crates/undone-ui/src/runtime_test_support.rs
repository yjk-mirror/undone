#[cfg(test)]
mod tests {
    use crate::game_state::{load_world_from_save, start_game, GameState, PreGameState};
    use crate::runtime_controller::RuntimeController;
    use crate::runtime_snapshot::RuntimeSnapshot;
    use crate::AppSignals;
    use floem::prelude::SignalUpdate;
    use lasso::Key;
    use rand::{rngs::SmallRng, SeedableRng};
    use std::collections::{HashMap, HashSet};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};
    use undone_domain::{
        Age, AttractionLevel, Behaviour, LikingLevel, LoveLevel, MaleClothing, MaleFigure, MaleNpc,
        NpcCore, PersonalityId, RelationshipStatus,
    };
    use undone_scene::types::SceneDefinition;
    struct RuntimeHarness {
        gs: GameState,
        signals: AppSignals,
    }

    impl RuntimeHarness {
        fn controller(&mut self) -> RuntimeController<'_> {
            make_runtime_controller(&mut self.gs, self.signals)
        }

        fn snapshot(&self) -> RuntimeSnapshot {
            crate::runtime_snapshot::snapshot_runtime(self.signals, &self.gs)
        }
    }

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("packs")
    }

    pub fn make_test_pre_state() -> PreGameState {
        crate::game_state::test_pre_state_from_dir(&packs_dir())
    }

    pub fn make_test_game_state() -> GameState {
        let pre = make_test_pre_state();
        let config = crate::char_creation::robin_quick_config(&pre.registry);
        start_game(pre, config, true)
    }

    pub fn make_test_signals() -> AppSignals {
        AppSignals::new()
    }

    pub fn make_runtime_controller<'a>(
        gs: &'a mut GameState,
        signals: AppSignals,
    ) -> RuntimeController<'a> {
        RuntimeController::new(gs, signals)
    }

    pub fn snapshot(gs: &GameState, signals: AppSignals) -> RuntimeSnapshot {
        crate::runtime_snapshot::snapshot_runtime(signals, gs)
    }

    fn make_harness() -> RuntimeHarness {
        RuntimeHarness {
            gs: make_test_game_state(),
            signals: make_test_signals(),
        }
    }

    fn temp_save_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("undone_runtime_{name}_{unique}.json"))
    }

    fn copy_dir_recursive(src: &Path, dst: &Path) {
        fs::create_dir_all(dst).unwrap();
        for entry in fs::read_dir(src).unwrap() {
            let entry = entry.unwrap();
            let dst_path = dst.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_dir_recursive(&entry.path(), &dst_path);
            } else {
                fs::copy(entry.path(), dst_path).unwrap();
            }
        }
    }

    fn bad_runtime_fixture_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let fixture_root = std::env::temp_dir().join(format!("undone_runtime_fixture_{unique}"));
        let fixture_packs_dir = fixture_root.join("packs");
        copy_dir_recursive(&packs_dir(), &fixture_packs_dir);

        let skills_path = fixture_packs_dir
            .join("base")
            .join("data")
            .join("skills.toml");
        let skills = fs::read_to_string(&skills_path).unwrap();
        fs::write(
            &skills_path,
            skills.replace(
                "id          = \"FEMININITY\"",
                "id          = \"FEMININITY_MISSING\"",
            ),
        )
        .unwrap();

        fixture_packs_dir
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

    fn play_until_continue_with_last_action(
        harness: &mut RuntimeHarness,
    ) -> (RuntimeSnapshot, Option<String>) {
        let mut tried_actions = HashSet::new();
        let mut last_action_id = None;
        for _ in 0..32 {
            let current = harness.snapshot();
            if current.awaiting_continue || current.visible_actions.is_empty() {
                return (current, last_action_id);
            }

            let scene_id = current
                .current_scene_id
                .clone()
                .unwrap_or_else(|| "<no-scene>".to_string());
            let action_id = current
                .visible_actions
                .iter()
                .find(|action| tried_actions.insert((scene_id.clone(), action.id.clone())))
                .unwrap_or(&current.visible_actions[0])
                .id
                .clone();
            last_action_id = Some(action_id.clone());
            let mut controller = harness.controller();
            controller.choose_action(&action_id).unwrap();
        }

        (harness.snapshot(), last_action_id)
    }

    fn play_until_continue(harness: &mut RuntimeHarness) -> RuntimeSnapshot {
        play_until_continue_with_last_action(harness).0
    }

    fn settle_workplace_route(gs: &mut GameState) {
        gs.world.game_data.set_flag("ROUTE_WORKPLACE");
        gs.world
            .game_data
            .advance_arc("base::workplace_opening", "settled");
        gs.world.game_data.set_flag("MET_LANDLORD");
        gs.world.game_data.set_flag("FIRST_MEETING_DONE");
        gs.world.game_data.set_flag("ONCE_base::workplace_arrival");
        gs.world.game_data.set_flag("ONCE_base::workplace_landlord");
        gs.world
            .game_data
            .set_flag("ONCE_base::workplace_first_night");
        gs.world
            .game_data
            .set_flag("ONCE_base::workplace_first_clothes");
        gs.world
            .game_data
            .set_flag("ONCE_base::workplace_first_day");
        gs.world
            .game_data
            .set_flag("ONCE_base::workplace_work_meeting");
        gs.world.game_data.set_flag("ONCE_base::workplace_evening");
        for _ in 0..28 {
            gs.world.game_data.advance_time_slot();
        }
    }

    fn has_week_two_robin_content(snapshot: &RuntimeSnapshot) -> bool {
        matches!(
            snapshot.current_scene_id.as_deref(),
            Some("base::coffee_shop") | Some("base::plan_your_day")
        ) || snapshot.world.game_flags.iter().any(|flag| {
            matches!(
                flag.as_str(),
                "MET_JAKE" | "ONCE_base::coffee_shop" | "ONCE_base::plan_your_day"
            )
        })
    }

    #[test]
    fn acceptance_runtime_new_game_launch_exposes_visible_prose_and_choices() {
        let mut harness = make_harness();
        let mut controller = harness.controller();
        controller.continue_flow().unwrap();
        let snapshot = controller.snapshot();

        assert!(snapshot.current_scene_id.is_some());
        assert!(
            !snapshot.story_paragraphs.is_empty(),
            "initial runtime should expose visible prose"
        );
        assert!(
            !snapshot.visible_actions.is_empty(),
            "initial runtime should expose visible action choices"
        );
    }

    #[test]
    fn test_pre_state_uses_shared_loader_contract() {
        let pre = make_test_pre_state();

        assert!(pre.init_error.is_none());
        assert!(!pre.scenes.is_empty());
    }

    #[test]
    fn malformed_runtime_content_surfaces_error_state() {
        let fixture_packs_dir = bad_runtime_fixture_dir();
        let state = crate::game_state::init_game_from_dir(&fixture_packs_dir);

        assert!(state.init_error.is_some());
        assert!(state
            .init_error
            .as_deref()
            .is_some_and(|message| message.contains("FEMININITY")));

        fs::remove_dir_all(fixture_packs_dir.parent().unwrap()).unwrap();
    }

    #[test]
    fn acceptance_runtime_choosing_an_action_updates_story_and_progression_state() {
        let mut harness = make_harness();
        {
            let mut controller = harness.controller();
            controller.continue_flow().unwrap();
        }
        let before = harness.snapshot();
        let action_id = before.visible_actions[0].id.clone();

        {
            let mut controller = harness.controller();
            controller.choose_action(&action_id).unwrap();
        }
        let after = harness.snapshot();

        assert!(
            after.story_paragraphs.len() > before.story_paragraphs.len(),
            "choosing an action should append visible prose"
        );
        assert!(
            after.awaiting_continue || after.current_scene_id == before.current_scene_id,
            "action should advance progression within the current runtime flow"
        );
    }

    #[test]
    fn acceptance_runtime_continue_moves_to_the_next_scene_or_no_scene_state() {
        let mut harness = make_harness();
        {
            let mut controller = harness.controller();
            controller.continue_flow().unwrap();
        }
        let before = play_until_continue(&mut harness);
        assert!(
            before.awaiting_continue || before.visible_actions.is_empty(),
            "test fixture should reach a continue or exhausted scene state"
        );

        {
            let mut controller = harness.controller();
            controller.continue_flow().unwrap();
        }
        let after = harness.snapshot();

        assert!(
            after.current_scene_id != before.current_scene_id || after.current_scene_id.is_none(),
            "continue should advance to a new scene or to no-scene state"
        );
    }

    #[test]
    fn acceptance_runtime_continue_requires_explicit_continue_path() {
        let mut harness = make_harness();
        {
            let mut controller = harness.controller();
            controller.continue_flow().unwrap();
        }
        let (paused, last_action_id) = play_until_continue_with_last_action(&mut harness);

        assert!(
            paused.awaiting_continue,
            "runtime should reach an explicit continue state before advancing"
        );
        assert!(
            !paused.story_paragraphs.is_empty(),
            "continue state should preserve visible story context"
        );
        assert!(
            paused.visible_actions.is_empty(),
            "continue state must not expose stale action choices"
        );

        let stale_action_id = last_action_id.expect("fixture should choose at least one action");
        {
            let mut controller = harness.controller();
            let error = controller.choose_action(&stale_action_id).unwrap_err();
            assert!(
                error.contains("not currently visible"),
                "continue state must reject stale action ids, got: {error}"
            );
        }

        {
            let mut controller = harness.controller();
            controller.continue_flow().unwrap();
        }
        let resumed = harness.snapshot();

        assert!(
            resumed.current_scene_id != paused.current_scene_id
                || resumed.story_paragraphs != paused.story_paragraphs
                || resumed.visible_actions != paused.visible_actions,
            "only the explicit continue path should replace the paused runtime snapshot"
        );
    }

    #[test]
    fn acceptance_runtime_once_only_scene_does_not_repeat() {
        let mut harness = make_harness();
        {
            let mut controller = harness.controller();
            controller.continue_flow().unwrap();
        }
        let first_scene = harness.snapshot().current_scene_id.unwrap();
        assert!(
            harness
                .gs
                .world
                .game_data
                .has_flag(&format!("ONCE_{first_scene}")),
            "once-only scene flag should be set when the scene is served"
        );

        let before_continue = play_until_continue(&mut harness);
        assert!(before_continue.awaiting_continue);
        {
            let mut controller = harness.controller();
            controller.continue_flow().unwrap();
        }
        let after = harness.snapshot();

        assert_ne!(
            after.current_scene_id.as_deref(),
            Some(first_scene.as_str())
        );
    }

    #[test]
    fn acceptance_runtime_save_load_resume_clears_stale_runtime_state() {
        let mut harness = make_harness();
        {
            let mut controller = harness.controller();
            controller.continue_flow().unwrap();
        }
        let _ = play_until_continue(&mut harness);

        let save_path = temp_save_path("resume");
        undone_save::save_game(&harness.gs.world, &harness.gs.registry, &save_path).unwrap();

        harness.signals.story.set("stale prose".into());
        harness
            .signals
            .actions
            .set(vec![undone_scene::engine::ActionView {
                id: "stale".into(),
                label: "Stale".into(),
                detail: "stale".into(),
            }]);
        load_world_from_save(&mut harness.gs, &save_path).unwrap();
        {
            let mut controller = harness.controller();
            controller.resume_from_current_world().unwrap();
        }
        let resumed = harness.snapshot();

        assert!(
            resumed
                .story_paragraphs
                .iter()
                .all(|paragraph| !paragraph.contains("stale prose")),
            "resume should clear stale runtime story"
        );
        assert!(
            resumed
                .visible_actions
                .iter()
                .all(|action| action.id != "stale"),
            "resume should clear stale runtime actions"
        );
        assert_eq!(
            resumed.current_scene_id.as_deref(),
            Some("base::workplace_landlord")
        );

        std::fs::remove_file(save_path).unwrap();
    }

    #[test]
    fn acceptance_runtime_visible_errors_appear_in_story_paragraphs() {
        let scene = SceneDefinition {
            id: "test::acceptance_error".into(),
            pack: "test".into(),
            intro_prose: "{{ m.undefinedMethod() }}".into(),
            intro_variants: vec![],
            intro_thoughts: vec![],
            actions: vec![],
            npc_actions: vec![],
        };
        let mut scenes = HashMap::new();
        scenes.insert(scene.id.clone(), std::sync::Arc::new(scene));

        let mut registry = undone_packs::PackRegistry::new();
        let personality = registry.intern_personality("ROMANTIC");
        let mut world = undone_world::test_helpers::make_test_world();
        world.male_npcs.insert(test_male_npc(personality));
        let mut gs = GameState {
            world,
            registry,
            engine: undone_scene::engine::SceneEngine::new(scenes),
            scheduler: undone_scene::scheduler::Scheduler::empty(),
            rng: SmallRng::seed_from_u64(7),
            dev_mode: true,
            init_error: None,
            opening_scene: None,
            femininity_id: undone_domain::SkillId::from_spur(
                lasso::Spur::try_from_usize(0).unwrap(),
            ),
            current_scene_time_anchor: None,
        };
        let signals = make_test_signals();

        {
            let mut controller = make_runtime_controller(&mut gs, signals);
            controller.start_scene("test::acceptance_error").unwrap();
        }
        let snapshot = snapshot(&gs, signals);

        assert!(
            snapshot
                .story_paragraphs
                .iter()
                .any(|paragraph| paragraph.contains("[Scene error:")),
            "runtime errors should surface in visible story output: {:?}",
            snapshot.story_paragraphs
        );
    }

    #[test]
    fn acceptance_runtime_robin_route_reaches_week_two_without_dev_time_travel() {
        let mut harness = make_harness();
        let mut saw_week_two_content = false;
        {
            let mut controller = harness.controller();
            controller.continue_flow().unwrap();
        }
        let mut last_snapshot = harness.snapshot();

        for _ in 0..120 {
            let paused = play_until_continue(&mut harness);
            if has_week_two_robin_content(&paused) {
                saw_week_two_content = true;
            }
            if paused.world.week >= 2 && saw_week_two_content {
                last_snapshot = paused;
                break;
            }
            assert!(
                paused.awaiting_continue,
                "runtime should finish each scene into continue state while advancing naturally, got {:?}",
                paused
            );

            {
                let mut controller = harness.controller();
                controller.continue_flow().unwrap();
            }
            let snapshot = harness.snapshot();
            if has_week_two_robin_content(&snapshot) {
                saw_week_two_content = true;
            }
            if snapshot.world.week >= 2 && saw_week_two_content {
                last_snapshot = snapshot;
                break;
            }
            last_snapshot = snapshot;
        }

        assert!(
            last_snapshot.world.week >= 2,
            "runtime should reach week 2 without dev-only time travel, got week/day/slot = {:?}/{:?}/{:?}",
            last_snapshot.world.week,
            last_snapshot.world.day,
            last_snapshot.world.time_slot
        );
        assert!(
            saw_week_two_content,
            "runtime should naturally reach week-2 gated Robin content, final scene {:?}, flags {:?}",
            last_snapshot.current_scene_id,
            last_snapshot.world.game_flags
        );
    }

    #[test]
    fn acceptance_runtime_settled_slot_scene_consumes_one_time_slot() {
        let mut harness = make_harness();
        settle_workplace_route(&mut harness.gs);

        {
            let mut controller = harness.controller();
            controller.start_scene("base::plan_your_day").unwrap();
        }

        {
            let mut controller = harness.controller();
            controller.choose_action("go_out").unwrap();
        }

        let slot_scene = harness.snapshot();
        assert_ne!(
            slot_scene.current_scene_id.as_deref(),
            Some("base::plan_your_day"),
            "slot request should start a scheduled free_time scene"
        );

        let mut expected = harness.gs.world.game_data.clone();
        expected.advance_time_slot();
        let expected_time_slot = format!("{:?}", expected.time_slot);

        let paused = play_until_continue(&mut harness);
        assert!(
            paused.awaiting_continue,
            "free_time scene should finish into awaiting-continue state"
        );

        {
            let mut controller = harness.controller();
            controller.continue_flow().unwrap();
        }
        let after = harness.snapshot();

        assert_eq!(
            (
                after.world.week,
                after.world.day,
                after.world.time_slot.clone(),
            ),
            (expected.week, expected.day, expected_time_slot),
            "finishing a settled slot scene should consume exactly one time slot before the next global pick"
        );
    }

    #[test]
    fn acceptance_runtime_jake_explicit_scene_exposes_action_and_sets_progress_flag() {
        let mut harness = make_harness();

        {
            let mut controller = harness.controller();
            controller.start_scene("base::jake_apartment").unwrap();
        }
        let before = harness.snapshot();

        assert_eq!(
            before.current_scene_id.as_deref(),
            Some("base::jake_apartment")
        );
        assert!(
            before
                .visible_actions
                .iter()
                .any(|action| action.id == "let_him_lead"),
            "Jake explicit scene should expose its forward romantic action"
        );
        assert!(
            before
                .story_paragraphs
                .iter()
                .any(|paragraph| paragraph.contains("door is closed")),
            "Jake explicit scene should render visible romantic setup prose"
        );

        {
            let mut controller = harness.controller();
            controller.choose_action("let_him_lead").unwrap();
        }
        let after = harness.snapshot();

        assert!(
            after
                .world
                .game_flags
                .iter()
                .any(|flag| flag == "JAKE_INTIMATE"),
            "choosing the explicit Jake action should surface JAKE_INTIMATE in runtime state"
        );
    }

    #[test]
    fn acceptance_runtime_party_follow_up_exposes_action_and_sets_progress_flag() {
        let mut harness = make_harness();

        {
            let mut controller = harness.controller();
            controller
                .start_scene("base::party_stranger_after")
                .unwrap();
        }
        let before = harness.snapshot();

        assert_eq!(
            before.current_scene_id.as_deref(),
            Some("base::party_stranger_after")
        );
        assert!(
            before
                .visible_actions
                .iter()
                .any(|action| action.id == "go_with_him"),
            "party follow-up should expose its explicit forward action"
        );
        assert!(
            before
                .story_paragraphs
                .iter()
                .any(|paragraph| paragraph.contains("stairwell") || paragraph.contains("hallway")),
            "party follow-up should render visible post-party setup prose"
        );

        {
            let mut controller = harness.controller();
            controller.choose_action("go_with_him").unwrap();
        }
        let after = harness.snapshot();

        assert!(
            after
                .world
                .game_flags
                .iter()
                .any(|flag| flag == "PARTY_STRANGER_SLEPT"),
            "choosing the party follow-up action should surface PARTY_STRANGER_SLEPT in runtime state"
        );
    }

    #[test]
    fn acceptance_runtime_work_lunch_visibly_reflects_first_day_lunch_memory() {
        let mut desk_harness = make_harness();
        desk_harness
            .gs
            .world
            .game_data
            .set_flag("FIRST_DAY_LUNCH_DESK");
        {
            let mut controller = desk_harness.controller();
            controller.start_scene("base::work_lunch").unwrap();
        }
        let desk_snapshot = desk_harness.snapshot();

        let mut group_harness = make_harness();
        group_harness
            .gs
            .world
            .game_data
            .set_flag("FIRST_DAY_LUNCH_GROUP");
        {
            let mut controller = group_harness.controller();
            controller.start_scene("base::work_lunch").unwrap();
        }
        let group_snapshot = group_harness.snapshot();

        assert!(
            desk_snapshot
                .story_paragraphs
                .iter()
                .any(|paragraph| paragraph.contains("stayed at your desk")),
            "desk-lunch memory should be visible in later lunch prose"
        );
        assert!(
            group_snapshot
                .story_paragraphs
                .iter()
                .any(|paragraph| paragraph.contains("clusters already have names")),
            "group-lunch memory should be visible in later lunch prose"
        );
    }
}
