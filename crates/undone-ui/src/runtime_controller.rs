use floem::prelude::{SignalGet, SignalUpdate};

use crate::game_state::{GameState, SceneTimeAnchor};
use crate::runtime_snapshot::{snapshot_runtime, RuntimeSnapshot};
use crate::{
    process_events, reset_scene_ui_state, start_scene, AppPhase, AppSignals, AppTab, PlayerSnapshot,
};
use undone_scene::engine::EngineEvent;
use undone_scene::scheduler::PickResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCommandOutcome {
    pub started_scene_id: Option<String>,
    pub current_scene_id: Option<String>,
    pub scene_finished: bool,
}

pub type RuntimeCommandResult = Result<RuntimeCommandOutcome, String>;

pub struct RuntimeController<'a> {
    pub gs: &'a mut GameState,
    pub signals: AppSignals,
}

impl<'a> RuntimeController<'a> {
    pub fn new(gs: &'a mut GameState, signals: AppSignals) -> Self {
        Self { gs, signals }
    }

    pub fn start_scene(&mut self, scene_id: impl Into<String>) -> RuntimeCommandResult {
        let scene_id = scene_id.into();
        if !self.gs.engine.has_scene(&scene_id) {
            return Err(format!("Unknown scene '{scene_id}'"));
        }

        self.start_scene_internal(scene_id, None, None)
    }

    pub fn choose_action(&mut self, action_id: &str) -> RuntimeCommandResult {
        let chosen = self
            .signals
            .actions
            .get_untracked()
            .into_iter()
            .find(|action| action.id == action_id)
            .ok_or_else(|| format!("Action '{action_id}' is not currently visible"))?;

        self.echo_choice(&chosen.label);

        let events =
            self.gs
                .engine
                .advance_with_action(action_id, &mut self.gs.world, &self.gs.registry);
        let requested_slot = events.iter().find_map(|event| {
            if let EngineEvent::SlotRequested(slot) = event {
                Some(slot.clone())
            } else {
                None
            }
        });
        let scene_finished =
            process_events(events, self.signals, &self.gs.world, self.gs.femininity_id);

        if let Some(slot_name) = requested_slot {
            return self.start_requested_slot(&slot_name);
        }

        if scene_finished {
            if self.signals.phase.get_untracked() == AppPhase::TransformationIntro {
                self.signals.phase.set(AppPhase::FemCreation);
            } else {
                self.signals.awaiting_continue.set(true);
            }
        }

        Ok(self.outcome(None, scene_finished))
    }

    pub fn continue_flow(&mut self) -> RuntimeCommandResult {
        let can_launch_initial = self.gs.engine.current_scene_id().is_none();
        if !self.signals.awaiting_continue.get_untracked() && !can_launch_initial {
            return Err("Runtime is not awaiting continue".to_string());
        }

        if self.signals.awaiting_continue.get_untracked() {
            self.consume_pending_scene_time();
        }

        self.start_next_scene(true)
    }

    pub fn jump_to_scene(&mut self, scene_id: &str) -> RuntimeCommandResult {
        let outcome = self.start_scene(scene_id.to_string())?;
        self.signals.tab.set(AppTab::Game);
        Ok(outcome)
    }

    pub fn resume_from_current_world(&mut self) -> RuntimeCommandResult {
        self.gs.engine.reset_runtime();
        self.gs.opening_scene = None;
        self.gs.current_scene_time_anchor = None;
        self.start_next_scene(false)
    }

    pub fn snapshot(&self) -> RuntimeSnapshot {
        snapshot_runtime(self.signals, self.gs)
    }

    fn start_scene_internal(
        &mut self,
        scene_id: String,
        scene_time_anchor: Option<SceneTimeAnchor>,
        npc_role: Option<&str>,
    ) -> RuntimeCommandResult {
        if !self.gs.engine.has_scene(&scene_id) {
            return Err(format!("Unknown scene '{scene_id}'"));
        }

        self.gs.current_scene_time_anchor = scene_time_anchor;
        reset_scene_ui_state(self.signals);
        start_scene(
            &mut self.gs.engine,
            &self.gs.world,
            &self.gs.registry,
            scene_id.clone(),
            npc_role,
        );
        let events = self.gs.engine.drain();
        let scene_finished =
            process_events(events, self.signals, &self.gs.world, self.gs.femininity_id);
        if scene_finished {
            self.signals.awaiting_continue.set(true);
        }

        Ok(self.outcome(Some(scene_id), scene_finished))
    }

    fn start_next_scene(&mut self, allow_opening_scene: bool) -> RuntimeCommandResult {
        if let Some(result) =
            self.gs
                .scheduler
                .pick_next(&self.gs.world, &self.gs.registry, &mut self.gs.rng)
        {
            let _ = self.gs.opening_scene.take();
            return self.start_scheduled_scene(result);
        }

        if allow_opening_scene {
            if let Some(scene_id) = self.gs.opening_scene.take() {
                return self.start_scene_internal(scene_id, None, None);
            }
        }

        Ok(self.show_no_scene_available())
    }

    fn start_requested_slot(&mut self, slot_name: &str) -> RuntimeCommandResult {
        if let Some(result) = self.gs.scheduler.pick(
            slot_name,
            &self.gs.world,
            &self.gs.registry,
            &mut self.gs.rng,
        ) {
            return self.start_scheduled_scene(result);
        }

        Ok(self.show_no_scene_available())
    }

    fn start_scheduled_scene(&mut self, result: PickResult) -> RuntimeCommandResult {
        if !self.gs.engine.has_scene(&result.scene_id) {
            return Err(format!("Unknown scene '{}'", result.scene_id));
        }

        if result.once_only {
            self.gs
                .world
                .game_data
                .set_flag(format!("ONCE_{}", result.scene_id));
        }
        let scene_time_anchor = result
            .consumes_time
            .then(|| SceneTimeAnchor::capture(&self.gs.world));
        self.start_scene_internal(
            result.scene_id,
            scene_time_anchor,
            result.npc_role.as_deref(),
        )
    }

    fn show_no_scene_available(&mut self) -> RuntimeCommandOutcome {
        self.gs.current_scene_time_anchor = None;
        reset_scene_ui_state(self.signals);
        self.signals
            .story
            .set("[No eligible scene is currently available.]".to_string());
        self.signals.actions.set(vec![]);
        self.signals.player.set(PlayerSnapshot::from_player(
            &self.gs.world.player,
            self.gs.femininity_id,
        ));

        self.outcome(None, false)
    }

    fn outcome(
        &self,
        started_scene_id: Option<String>,
        scene_finished: bool,
    ) -> RuntimeCommandOutcome {
        RuntimeCommandOutcome {
            started_scene_id,
            current_scene_id: self.gs.engine.current_scene_id(),
            scene_finished,
        }
    }

    fn consume_pending_scene_time(&mut self) {
        let should_advance = self
            .gs
            .current_scene_time_anchor
            .is_some_and(|anchor| anchor.matches_world(&self.gs.world));
        self.gs.current_scene_time_anchor = None;
        if should_advance {
            self.gs.world.game_data.advance_time_slot();
        }
    }

    fn echo_choice(&self, label: &str) {
        let echo = format!("\n\n---\n\n> **{}**", label);
        self.signals.story.update(|story| story.push_str(&echo));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::{start_game, GameState, PreGameState};
    use crate::NpcSnapshot;
    use lasso::Key;
    use rand::{rngs::SmallRng, SeedableRng};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};
    use undone_domain::{AttractionLevel, LikingLevel, RelationshipStatus, SkillId};
    use undone_packs::{LoadedPackMeta, PackContent, PackManifest, PackMeta, PackRegistry};
    use undone_scene::engine::{ActionView, SceneEngine};
    use undone_scene::scheduler::{load_schedule, Scheduler};
    use undone_scene::types::{Action, NextBranch, SceneDefinition};
    use undone_world::test_helpers::{make_test_male_npc, make_test_world as test_world};

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("packs")
    }

    fn temp_test_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("undone_ui_{prefix}_{unique}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn test_pre_state() -> PreGameState {
        crate::game_state::test_pre_state_from_dir(&packs_dir())
    }

    fn test_game_state() -> GameState {
        let pre = test_pre_state();
        let config = crate::char_creation::robin_quick_config(&pre.registry);
        start_game(pre, config, true)
    }

    fn action_ids(snapshot: &RuntimeSnapshot) -> Vec<&str> {
        snapshot
            .visible_actions
            .iter()
            .map(|action| action.id.as_str())
            .collect()
    }

    fn custom_game_state(scene: SceneDefinition) -> GameState {
        let mut scenes = HashMap::new();
        scenes.insert(scene.id.clone(), Arc::new(scene));

        let mut registry = PackRegistry::new();
        let personality = registry.intern_personality("ROMANTIC");
        let mut world = test_world();
        world.male_npcs.insert(make_test_male_npc(personality));

        GameState {
            world,
            registry,
            engine: SceneEngine::new(scenes),
            scheduler: Scheduler::empty(),
            rng: SmallRng::seed_from_u64(7),
            dev_mode: true,
            init_error: None,
            opening_scene: None,
            femininity_id: SkillId::from_spur(lasso::Spur::try_from_usize(0).unwrap()),
            current_scene_time_anchor: None,
        }
    }

    fn scheduler_with_event(slot_name: &str, scene_id: &str) -> Scheduler {
        let pack_dir = temp_test_dir("runtime_requested_slot");
        let schedule_path = pack_dir.join("schedule.toml");
        std::fs::write(
            &schedule_path,
            format!(
                r#"
                [[slot]]
                name = "{slot_name}"

                [[slot.events]]
                scene = "{scene_id}"
                weight = 10
            "#
            ),
        )
        .unwrap();

        let meta = LoadedPackMeta {
            pack_dir,
            manifest: PackManifest {
                pack: PackMeta {
                    id: "test".into(),
                    name: "Test".into(),
                    version: "0.1.0".into(),
                    author: "Undone tests".into(),
                    requires: vec![],
                    opening_scene: None,
                    transformation_scene: None,
                },
                content: PackContent {
                    traits: String::new(),
                    npc_traits: String::new(),
                    skills: String::new(),
                    scenes_dir: String::new(),
                    schedule_file: Some("schedule.toml".into()),
                    names_file: None,
                    stats_file: None,
                    races_file: None,
                    categories_file: None,
                    arcs_file: None,
                },
            },
        };
        load_schedule(&[meta], &PackRegistry::new()).unwrap()
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

    fn play_first_visible_action_until_pause(
        controller: &mut RuntimeController<'_>,
    ) -> RuntimeSnapshot {
        for _ in 0..32 {
            let snapshot = controller.snapshot();
            if snapshot.awaiting_continue || snapshot.visible_actions.is_empty() {
                return snapshot;
            }

            let action_id = snapshot.visible_actions[0].id.clone();
            controller.choose_action(&action_id).unwrap();
        }

        controller.snapshot()
    }

    #[test]
    fn runtime_controller_start_scene_clears_stale_scene_ui_state() {
        let scene = SceneDefinition {
            id: "test::clear_state".into(),
            pack: "test".into(),
            intro_prose: "Fresh intro.".into(),
            intro_variants: vec![],
            intro_thoughts: vec![],
            actions: vec![Action {
                id: "wait".into(),
                label: "Wait".into(),
                detail: "Stay here.".into(),
                condition: None,
                prose: String::new(),
                allow_npc_actions: false,
                effect: None,
                next: vec![NextBranch {
                    condition: None,
                    goto: None,
                    slot: None,
                    finish: false,
                }],
                thoughts: vec![],
            }],
            npc_actions: vec![],
        };

        let mut gs = custom_game_state(scene);
        let signals = AppSignals::new();
        signals.story.set("stale".into());
        signals.actions.set(vec![ActionView {
            id: "stale".into(),
            label: "Stale".into(),
            detail: "old".into(),
        }]);
        signals.active_npc.set(Some(NpcSnapshot {
            name: "Old".into(),
            age: "Old".into(),
            personality: "Old".into(),
            relationship: RelationshipStatus::Acquaintance,
            pc_liking: LikingLevel::Like,
            pc_attraction: AttractionLevel::Attracted,
        }));
        signals.awaiting_continue.set(true);
        signals.scroll_gen.set(3);

        let mut controller = RuntimeController::new(&mut gs, signals);
        let outcome = controller.start_scene("test::clear_state").unwrap();

        assert_eq!(
            outcome.started_scene_id.as_deref(),
            Some("test::clear_state")
        );
        assert_eq!(signals.scroll_gen.get(), 0);
        assert!(!signals.awaiting_continue.get());
        assert!(signals.story.get().contains("Fresh intro."));
        assert_eq!(signals.actions.get().len(), 1);
        assert_ne!(signals.story.get(), "stale");
    }

    #[test]
    fn runtime_controller_start_scene_binds_fallback_npcs_before_intro_render() {
        let scene = SceneDefinition {
            id: "test::intro_time_npc".into(),
            pack: "test".into(),
            intro_prose: "{{ m.getLiking() }}".into(),
            intro_variants: vec![],
            intro_thoughts: vec![],
            actions: vec![],
            npc_actions: vec![],
        };

        let mut gs = custom_game_state(scene);
        let signals = AppSignals::new();

        let mut controller = RuntimeController::new(&mut gs, signals);
        controller.start_scene("test::intro_time_npc").unwrap();
        let snapshot = controller.snapshot();

        assert!(
            !snapshot
                .story_paragraphs
                .iter()
                .any(|paragraph| paragraph.contains("[Scene error:")),
            "intro-time NPC access must be valid during scene start: {:?}",
            snapshot.story_paragraphs
        );
    }

    #[test]
    fn runtime_controller_continue_flow_starts_next_scene_and_applies_once_only() {
        let mut gs = test_game_state();
        let signals = AppSignals::new();
        signals.awaiting_continue.set(true);

        let mut probe_rng = gs.rng.clone();
        let expected = gs
            .scheduler
            .pick_next(&gs.world, &gs.registry, &mut probe_rng)
            .expect("test game should have an eligible scene");

        let mut controller = RuntimeController::new(&mut gs, signals);
        let outcome = controller.continue_flow().unwrap();

        assert_eq!(
            outcome.started_scene_id.as_deref(),
            Some(expected.scene_id.as_str())
        );
        assert_eq!(
            controller.gs.engine.current_scene_id().as_deref(),
            Some(expected.scene_id.as_str())
        );
        if expected.once_only {
            assert!(controller
                .gs
                .world
                .game_data
                .has_flag(&format!("ONCE_{}", expected.scene_id)));
        }
    }

    #[test]
    fn runtime_controller_jump_to_scene_reuses_shared_scene_start_path() {
        let scene = SceneDefinition {
            id: "test::jump_target".into(),
            pack: "test".into(),
            intro_prose: "{{ m.getLiking() }}".into(),
            intro_variants: vec![],
            intro_thoughts: vec![],
            actions: vec![],
            npc_actions: vec![],
        };

        let mut gs = custom_game_state(scene);
        let signals = AppSignals::new();
        signals.tab.set(AppTab::Dev);
        signals.story.set("stale".into());

        let mut controller = RuntimeController::new(&mut gs, signals);
        let outcome = controller.jump_to_scene("test::jump_target").unwrap();
        let snapshot = controller.snapshot();

        assert_eq!(
            outcome.started_scene_id.as_deref(),
            Some("test::jump_target")
        );
        assert_eq!(signals.tab.get(), AppTab::Game);
        assert!(
            !snapshot
                .story_paragraphs
                .iter()
                .any(|paragraph| paragraph.contains("[Scene error:")),
            "jump_to_scene should reuse start-scene semantics: {:?}",
            snapshot.story_paragraphs
        );
        assert_ne!(signals.story.get(), "stale");
    }

    #[test]
    fn runtime_controller_choose_action_advances_slot_requests_into_new_scene() {
        let mut gs = test_game_state();
        let signals = AppSignals::new();

        let mut controller = RuntimeController::new(&mut gs, signals);
        controller.start_scene("base::plan_your_day").unwrap();
        for _ in 0..28 {
            controller.gs.world.game_data.advance_time_slot();
        }

        let before = controller.snapshot();
        assert_eq!(
            before.current_scene_id.as_deref(),
            Some("base::plan_your_day")
        );
        assert_eq!(
            action_ids(&before),
            vec!["go_out", "run_errands", "stay_in"]
        );

        let mut probe_rng = controller.gs.rng.clone();
        let expected = controller
            .gs
            .scheduler
            .pick(
                "free_time",
                &controller.gs.world,
                &controller.gs.registry,
                &mut probe_rng,
            )
            .expect("free_time slot should have at least one eligible scene");

        let outcome = controller.choose_action("go_out").unwrap();
        let after = controller.snapshot();

        assert_eq!(
            outcome.current_scene_id.as_deref(),
            Some(expected.scene_id.as_str())
        );
        assert_eq!(
            after.current_scene_id.as_deref(),
            Some(expected.scene_id.as_str())
        );
        assert_ne!(
            action_ids(&after),
            vec!["go_out", "run_errands", "stay_in"],
            "slot-based scene transitions must replace stale hub actions"
        );
        assert!(
            after
                .story_paragraphs
                .iter()
                .all(|paragraph| !paragraph.contains("Coffee, window, list.")),
            "slot-based scene transitions must replace stale hub prose: {:?}",
            after.story_paragraphs
        );
    }

    #[test]
    fn runtime_controller_choose_action_returns_error_when_requested_slot_scene_is_missing() {
        let scene = SceneDefinition {
            id: "test::hub".into(),
            pack: "test".into(),
            intro_prose: "Hub.".into(),
            intro_variants: vec![],
            intro_thoughts: vec![],
            actions: vec![Action {
                id: "go".into(),
                label: "Go".into(),
                detail: "Request a slot.".into(),
                condition: None,
                prose: String::new(),
                allow_npc_actions: false,
                effect: None,
                next: vec![NextBranch {
                    condition: None,
                    goto: None,
                    slot: Some("test_slot".into()),
                    finish: false,
                }],
                thoughts: vec![],
            }],
            npc_actions: vec![],
        };
        let mut gs = custom_game_state(scene);
        gs.scheduler = scheduler_with_event("test_slot", "test::missing_scene");
        let signals = AppSignals::new();

        let mut controller = RuntimeController::new(&mut gs, signals);
        controller.start_scene("test::hub").unwrap();

        let error = controller.choose_action("go").unwrap_err();
        assert!(
            error.contains("Unknown scene 'test::missing_scene'"),
            "requested-slot content mismatch should be a recoverable command error, got: {error}"
        );
    }

    #[test]
    fn runtime_controller_continue_consumes_time_after_free_time_scene() {
        let mut gs = test_game_state();
        settle_workplace_route(&mut gs);
        let signals = AppSignals::new();

        let mut controller = RuntimeController::new(&mut gs, signals);
        controller.start_scene("base::plan_your_day").unwrap();
        controller.choose_action("go_out").unwrap();

        assert!(
            controller.gs.current_scene_time_anchor.is_some(),
            "scheduled free_time scenes should remember their cadence anchor"
        );

        let mut expected = controller.gs.world.game_data.clone();
        expected.advance_time_slot();

        let paused = play_first_visible_action_until_pause(&mut controller);
        assert!(
            paused.awaiting_continue,
            "free_time scene should finish into awaiting-continue state"
        );

        controller.continue_flow().unwrap();

        assert_eq!(
            (
                controller.gs.world.game_data.week,
                controller.gs.world.game_data.day,
                format!("{:?}", controller.gs.world.game_data.time_slot),
            ),
            (
                expected.week,
                expected.day,
                format!("{:?}", expected.time_slot)
            ),
            "continue_flow should consume exactly one time slot after a settled free_time scene"
        );
    }

    #[test]
    fn runtime_controller_opening_arc_scene_does_not_double_advance_time() {
        let mut gs = test_game_state();
        gs.world.game_data.set_flag("ROUTE_WORKPLACE");
        gs.world
            .game_data
            .advance_arc("base::workplace_opening", "working");
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
        let signals = AppSignals::new();
        signals.awaiting_continue.set(true);

        let mut controller = RuntimeController::new(&mut gs, signals);
        let outcome = controller.continue_flow().unwrap();

        assert_eq!(
            outcome.started_scene_id.as_deref(),
            Some("base::workplace_evening")
        );
        assert!(
            controller.gs.current_scene_time_anchor.is_none(),
            "opening-arc scenes should not be treated as cadence-consuming slot scenes"
        );

        let _ = play_first_visible_action_until_pause(&mut controller);
        let after_scene = controller.gs.world.game_data.clone();

        controller.continue_flow().unwrap();

        assert_eq!(
            (
                controller.gs.world.game_data.week,
                controller.gs.world.game_data.day,
                controller.gs.world.game_data.time_slot,
            ),
            (after_scene.week, after_scene.day, after_scene.time_slot),
            "continue_flow must not add an extra time step after explicit advance_time scenes"
        );
    }
}
