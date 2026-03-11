use floem::prelude::{SignalGet, SignalUpdate};

use crate::game_state::GameState;
use crate::runtime_snapshot::{snapshot_runtime, RuntimeSnapshot};
use crate::{
    process_events, reset_scene_ui_state, start_scene, AppPhase, AppSignals, AppTab,
    PlayerSnapshot,
};
use undone_scene::engine::EngineEvent;

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

        self.start_scene_internal(scene_id)
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
            return Ok(self.start_requested_slot(&slot_name));
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
        self.start_next_scene(false)
    }

    pub fn snapshot(&self) -> RuntimeSnapshot {
        snapshot_runtime(self.signals, self.gs)
    }

    fn start_scene_internal(&mut self, scene_id: String) -> RuntimeCommandResult {
        reset_scene_ui_state(self.signals);
        start_scene(
            &mut self.gs.engine,
            &self.gs.world,
            &self.gs.registry,
            scene_id.clone(),
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
            if result.once_only {
                self.gs
                    .world
                    .game_data
                    .set_flag(format!("ONCE_{}", result.scene_id));
            }
            return self.start_scene_internal(result.scene_id);
        }

        if allow_opening_scene {
            if let Some(scene_id) = self.gs.opening_scene.take() {
                return self.start_scene_internal(scene_id);
            }
        }

        Ok(self.show_no_scene_available())
    }

    fn start_requested_slot(&mut self, slot_name: &str) -> RuntimeCommandOutcome {
        if let Some(result) =
            self.gs
                .scheduler
                .pick(slot_name, &self.gs.world, &self.gs.registry, &mut self.gs.rng)
        {
            if result.once_only {
                self.gs
                    .world
                    .game_data
                    .set_flag(format!("ONCE_{}", result.scene_id));
            }
            return self
                .start_scene_internal(result.scene_id)
                .expect("scheduler returned a known scene id");
        }

        self.show_no_scene_available()
    }

    fn show_no_scene_available(&mut self) -> RuntimeCommandOutcome {
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
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;
    use std::sync::Arc;
    use undone_domain::{
        Age, AttractionLevel, Behaviour, LikingLevel, LoveLevel, MaleClothing, MaleFigure, MaleNpc,
        NpcCore, PersonalityId, RelationshipStatus, SkillId,
    };
    use undone_packs::{load_packs, PackRegistry};
    use undone_scene::engine::{ActionView, SceneEngine};
    use undone_scene::loader::load_scenes;
    use undone_scene::scheduler::{load_schedule, validate_entry_scene_references, Scheduler};
    use undone_scene::types::{Action, NextBranch, SceneDefinition};
    use undone_world::test_helpers::make_test_world as test_world;

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("packs")
    }

    fn test_pre_state() -> PreGameState {
        let packs_dir = packs_dir();
        let (registry, metas) = load_packs(&packs_dir).unwrap();

        let mut scenes: HashMap<String, Arc<SceneDefinition>> = HashMap::new();
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

    fn custom_game_state(scene: SceneDefinition) -> GameState {
        let mut scenes = HashMap::new();
        scenes.insert(scene.id.clone(), Arc::new(scene));

        let mut registry = PackRegistry::new();
        let personality = registry.intern_personality("ROMANTIC");
        let mut world = test_world();
        world.male_npcs.insert(test_male_npc(personality));

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
        }
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
                effects: vec![],
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
        assert_eq!(before.current_scene_id.as_deref(), Some("base::plan_your_day"));
        assert_eq!(action_ids(&before), vec!["go_out", "run_errands", "stay_in"]);

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
}
