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
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use undone_domain::{
        Age, AttractionLevel, Behaviour, LikingLevel, LoveLevel, MaleClothing, MaleFigure, MaleNpc,
        NpcCore, PersonalityId, RelationshipStatus,
    };
    use undone_packs::load_packs;
    use undone_scene::loader::load_scenes;
    use undone_scene::scheduler::{load_schedule, validate_entry_scene_references};
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
        let packs_dir = packs_dir();
        let (registry, metas) = load_packs(&packs_dir).unwrap();

        let mut scenes: HashMap<String, std::sync::Arc<SceneDefinition>> = HashMap::new();
        let mut scene_sources: HashMap<String, String> = HashMap::new();
        for meta in &metas {
            let scene_dir = meta.pack_dir.join(&meta.manifest.content.scenes_dir);
            for (scene_id, scene) in load_scenes(&scene_dir, &registry).unwrap() {
                scene_sources.insert(scene_id.clone(), meta.manifest.pack.id.clone());
                scenes.insert(scene_id, scene);
            }
        }
        undone_scene::loader::validate_cross_references(&scenes).unwrap();

        let scheduler = load_schedule(&metas, &registry).unwrap();
        scheduler.validate_scene_references(&scenes).unwrap();
        validate_entry_scene_references(
            &scenes,
            registry.opening_scene(),
            registry.transformation_scene(),
        )
        .unwrap();

        PreGameState {
            registry,
            scenes,
            scheduler,
            rng: SmallRng::seed_from_u64(7),
            init_error: None,
        }
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

    fn play_until_continue(harness: &mut RuntimeHarness) -> RuntimeSnapshot {
        for _ in 0..8 {
            let current = harness.snapshot();
            if current.awaiting_continue || current.visible_actions.is_empty() {
                return current;
            }

            let action_id = current.visible_actions[0].id.clone();
            let mut controller = harness.controller();
            controller.choose_action(&action_id).unwrap();
        }

        harness.snapshot()
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
}
