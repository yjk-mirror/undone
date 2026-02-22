use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use rand::{rngs::SmallRng, Rng, SeedableRng};
use undone_domain::{FemaleNpcKey, MaleNpcKey};
use undone_expr::{eval, SceneCtx};
use undone_packs::PackRegistry;
use undone_world::World;

use crate::{
    effects::apply_effect,
    template_ctx::render_prose,
    types::{Action, NextBranch, SceneDefinition},
};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

pub struct SceneEngine {
    scenes: HashMap<String, Arc<SceneDefinition>>,
    stack: Vec<SceneFrame>,
    events: VecDeque<EngineEvent>,
    rng: SmallRng,
}

struct SceneFrame {
    def: Arc<SceneDefinition>,
    ctx: SceneCtx,
}

#[derive(Debug)]
pub enum EngineCommand {
    StartScene(String),
    ChooseAction(String),
    SetActiveMale(MaleNpcKey),
    SetActiveFemale(FemaleNpcKey),
}

#[derive(Debug)]
pub enum EngineEvent {
    ProseAdded(String),
    ActionsAvailable(Vec<ActionView>),
    SceneFinished,
}

#[derive(Debug, Clone)]
pub struct ActionView {
    pub id: String,
    pub label: String,
    pub detail: String,
}

// ---------------------------------------------------------------------------
// Implementation
// ---------------------------------------------------------------------------

impl SceneEngine {
    pub fn new(scenes: HashMap<String, Arc<SceneDefinition>>) -> Self {
        Self {
            scenes,
            stack: Vec::new(),
            events: VecDeque::new(),
            rng: SmallRng::from_entropy(),
        }
    }

    /// Dispatch a command. The engine may push zero or more events.
    pub fn send(
        &mut self,
        cmd: EngineCommand,
        world: &mut World,
        registry: &PackRegistry,
    ) {
        match cmd {
            EngineCommand::StartScene(id) => {
                self.start_scene(id, world, registry);
            }
            EngineCommand::ChooseAction(action_id) => {
                self.choose_action(action_id, world, registry);
            }
            EngineCommand::SetActiveMale(key) => {
                if let Some(frame) = self.stack.last_mut() {
                    frame.ctx.active_male = Some(key);
                }
            }
            EngineCommand::SetActiveFemale(key) => {
                if let Some(frame) = self.stack.last_mut() {
                    frame.ctx.active_female = Some(key);
                }
            }
        }
    }

    /// Drain all pending events, returning them in order.
    pub fn drain(&mut self) -> Vec<EngineEvent> {
        self.events.drain(..).collect()
    }

    // -----------------------------------------------------------------------
    // Private: scene lifecycle
    // -----------------------------------------------------------------------

    fn start_scene(&mut self, id: String, world: &World, registry: &PackRegistry) {
        let def = match self.scenes.get(&id) {
            Some(d) => Arc::clone(d),
            None => {
                eprintln!("[scene-engine] unknown scene: {id}");
                return;
            }
        };

        let ctx = SceneCtx::new();

        // Render intro prose
        match render_prose(&def.intro_prose, world, &ctx, registry) {
            Ok(prose) => self.events.push_back(EngineEvent::ProseAdded(prose)),
            Err(e) => self
                .events
                .push_back(EngineEvent::ProseAdded(format!("[template error: {e}]"))),
        }

        self.stack.push(SceneFrame { def, ctx });

        self.emit_actions(world, registry);
    }

    fn choose_action(&mut self, action_id: String, world: &mut World, registry: &PackRegistry) {
        // Find action in current frame
        let frame = match self.stack.last() {
            Some(f) => f,
            None => return,
        };

        let action: Action = match frame.def.actions.iter().find(|a| a.id == action_id) {
            Some(a) => a.clone(),
            None => return,
        };

        let allow_npc = action.allow_npc_actions;

        // Render action prose (if non-empty)
        if !action.prose.is_empty() {
            let frame = self.stack.last().unwrap();
            match render_prose(&action.prose, world, &frame.ctx, registry) {
                Ok(prose) => self.events.push_back(EngineEvent::ProseAdded(prose)),
                Err(e) => self
                    .events
                    .push_back(EngineEvent::ProseAdded(format!("[template error: {e}]"))),
            }
        }

        // Apply effects
        {
            let frame = self.stack.last_mut().unwrap();
            for effect in &action.effects {
                if let Err(e) = apply_effect(effect, world, &mut frame.ctx, registry) {
                    eprintln!("[scene-engine] effect error: {e}");
                }
            }
        }

        // Run NPC actions if allowed
        if allow_npc {
            self.run_npc_actions(world, registry);
        }

        // Evaluate next branches
        let next_branches = action.next.clone();
        self.evaluate_next(next_branches, world, registry);
    }

    fn emit_actions(&mut self, world: &World, registry: &PackRegistry) {
        let frame = match self.stack.last() {
            Some(f) => f,
            None => return,
        };

        let mut views = Vec::new();
        for action in &frame.def.actions {
            let visible = match &action.condition {
                Some(expr) => eval(expr, world, &frame.ctx, registry).unwrap_or(false),
                None => true,
            };
            if visible {
                views.push(ActionView {
                    id: action.id.clone(),
                    label: action.label.clone(),
                    detail: action.detail.clone(),
                });
            }
        }

        self.events
            .push_back(EngineEvent::ActionsAvailable(views));
    }

    fn run_npc_actions(&mut self, world: &mut World, registry: &PackRegistry) {
        // Collect eligible NPC actions (condition passes) with their weights
        let npc_actions: Vec<(usize, u32)> = {
            let frame = self.stack.last().unwrap();
            frame
                .def
                .npc_actions
                .iter()
                .enumerate()
                .filter_map(|(i, na)| {
                    let eligible = match &na.condition {
                        Some(expr) => eval(expr, world, &frame.ctx, registry).unwrap_or(false),
                        None => true,
                    };
                    if eligible {
                        Some((i, na.weight))
                    } else {
                        None
                    }
                })
                .collect()
        };

        if npc_actions.is_empty() {
            return;
        }

        // Weighted random selection
        let total_weight: u32 = npc_actions.iter().map(|(_, w)| w).sum();
        if total_weight == 0 {
            return;
        }
        let mut roll = self.rng.gen_range(0..total_weight);
        let selected_idx = npc_actions
            .iter()
            .find(|(_, w)| {
                if roll < *w {
                    true
                } else {
                    roll -= w;
                    false
                }
            })
            .map(|(i, _)| *i);

        let Some(idx) = selected_idx else { return };

        // Clone data we need before borrowing mutably
        let (prose, effects): (String, Vec<_>) = {
            let frame = self.stack.last().unwrap();
            let na = &frame.def.npc_actions[idx];
            (na.prose.clone(), na.effects.clone())
        };

        // Render NPC prose
        if !prose.is_empty() {
            let frame = self.stack.last().unwrap();
            match render_prose(&prose, world, &frame.ctx, registry) {
                Ok(rendered) => self.events.push_back(EngineEvent::ProseAdded(rendered)),
                Err(e) => self
                    .events
                    .push_back(EngineEvent::ProseAdded(format!("[template error: {e}]"))),
            }
        }

        // Apply NPC action effects
        {
            let frame = self.stack.last_mut().unwrap();
            for effect in &effects {
                if let Err(e) = apply_effect(effect, world, &mut frame.ctx, registry) {
                    eprintln!("[scene-engine] npc effect error: {e}");
                }
            }
        }
    }

    fn evaluate_next(
        &mut self,
        branches: Vec<NextBranch>,
        world: &World,
        registry: &PackRegistry,
    ) {
        if branches.is_empty() {
            // No next branches — re-emit actions (loop)
            self.emit_actions(world, registry);
            return;
        }

        for branch in &branches {
            let condition_passes = match &branch.condition {
                Some(expr) => {
                    let frame = self.stack.last().unwrap();
                    eval(expr, world, &frame.ctx, registry).unwrap_or(false)
                }
                None => true,
            };

            if !condition_passes {
                continue;
            }

            if branch.finish {
                self.stack.pop();
                self.events.push_back(EngineEvent::SceneFinished);
                return;
            }

            if let Some(goto) = &branch.goto {
                let target = goto.clone();
                self.stack.pop();
                self.start_scene(target, world, registry);
                return;
            }
        }

        // No branch matched — re-emit actions
        self.emit_actions(world, registry);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    use slotmap::SlotMap;
    use undone_domain::*;
    use undone_world::{GameData, World};

    use crate::types::{EffectDef, SceneDefinition};

    fn make_world() -> World {
        World {
            player: Player {
                name: "Eva".into(),
                age: Age::LateTeen,
                race: "east_asian".into(),
                figure: PlayerFigure::Slim,
                breasts: BreastSize::Large,
                eye_colour: "brown".into(),
                hair_colour: "dark".into(),
                traits: HashSet::new(),
                skills: HashMap::new(),
                money: 100,
                stress: 10,
                anxiety: 5,
                arousal: ArousalLevel::Comfort,
                alcohol: AlcoholLevel::Sober,
                partner: None,
                friends: vec![],
                virgin: true,
                anal_virgin: true,
                lesbian_virgin: true,
                on_pill: false,
                pregnancy: None,
                stuff: HashSet::new(),
                custom_flags: HashMap::new(),
                custom_ints: HashMap::new(),
                always_female: false,
                femininity: 10,
            },
            male_npcs: SlotMap::with_key(),
            female_npcs: SlotMap::with_key(),
            game_data: GameData::default(),
        }
    }

    fn make_simple_scene() -> SceneDefinition {
        SceneDefinition {
            id: "test::simple".into(),
            pack: "test".into(),
            intro_prose: "It begins.".into(),
            actions: vec![
                Action {
                    id: "wait".into(),
                    label: "Wait".into(),
                    detail: "Just wait.".into(),
                    condition: None,
                    prose: String::new(),
                    allow_npc_actions: false,
                    effects: vec![],
                    next: vec![],
                },
                Action {
                    id: "leave".into(),
                    label: "Leave".into(),
                    detail: "Go.".into(),
                    condition: None,
                    prose: "You leave.".into(),
                    allow_npc_actions: false,
                    effects: vec![EffectDef::ChangeStress { amount: -1 }],
                    next: vec![NextBranch {
                        condition: None,
                        goto: None,
                        finish: true,
                    }],
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

        assert_eq!(world.player.stress, stress_before - 1, "stress should have decreased by 1");
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
        use undone_expr::parse;

        // Build a scene with a conditional action
        let cond_expr = parse("scene.hasFlag('special')").unwrap();
        let scene = SceneDefinition {
            id: "test::conditional".into(),
            pack: "test".into(),
            intro_prose: "Conditional test.".into(),
            actions: vec![
                Action {
                    id: "always".into(),
                    label: "Always".into(),
                    detail: "Always visible.".into(),
                    condition: None,
                    prose: String::new(),
                    allow_npc_actions: false,
                    effects: vec![],
                    next: vec![],
                },
                Action {
                    id: "special".into(),
                    label: "Special".into(),
                    detail: "Only when flag set.".into(),
                    condition: Some(cond_expr),
                    prose: String::new(),
                    allow_npc_actions: false,
                    effects: vec![],
                    next: vec![],
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
}
