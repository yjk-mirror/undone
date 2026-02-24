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
    types::{Action, NarratorVariant, NextBranch, SceneDefinition, Thought},
};

/// Maximum scene transitions per command. Prevents both deep sub-scene stacks
/// and flat goto cycles (where the stack stays at depth 1 but transitions loop).
const MAX_TRANSITIONS_PER_COMMAND: usize = 32;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

pub struct SceneEngine {
    scenes: HashMap<String, Arc<SceneDefinition>>,
    stack: Vec<SceneFrame>,
    events: VecDeque<EngineEvent>,
    rng: SmallRng,
    /// Counts scene transitions within a single `send()` call.
    /// Reset at the start of each command. Guards against goto cycles.
    transition_count: usize,
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

#[derive(Debug, Clone)]
pub enum EngineEvent {
    ProseAdded(String),
    /// An inner thought or emotional aside. The UI renders this in a distinct style.
    /// `style` is a hint: "inner_voice" = italic, "anxiety" = anxious register, etc.
    ThoughtAdded {
        text: String,
        style: String,
    },
    ActionsAvailable(Vec<ActionView>),
    NpcActivated(Option<NpcActivatedData>),
    SceneFinished,
    /// Hub scene chose a scheduler slot — UI should run the scheduler for this slot.
    SlotRequested(String),
    ErrorOccurred(String),
}

#[derive(Debug, Clone)]
pub struct NpcActivatedData {
    pub name: String,
    pub age: undone_domain::Age,
    pub personality: String,
    pub relationship: undone_domain::RelationshipStatus,
    pub pc_liking: undone_domain::LikingLevel,
    pub pc_attraction: undone_domain::AttractionLevel,
}

impl NpcActivatedData {
    pub fn from_npc(npc: &undone_domain::NpcCore, registry: &PackRegistry) -> Self {
        Self {
            name: npc.name.clone(),
            age: npc.age,
            personality: registry.personality_name(npc.personality).to_owned(),
            relationship: npc.relationship.clone(),
            pc_liking: npc.pc_liking,
            pc_attraction: npc.pc_attraction,
        }
    }
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
            transition_count: 0,
        }
    }

    /// Dispatch a command. The engine may push zero or more events.
    pub fn send(&mut self, cmd: EngineCommand, world: &mut World, registry: &PackRegistry) {
        self.transition_count = 0;
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
                if let Some(npc) = world.male_npc(key) {
                    self.events.push_back(EngineEvent::NpcActivated(Some(
                        NpcActivatedData::from_npc(&npc.core, registry),
                    )));
                }
            }
            EngineCommand::SetActiveFemale(key) => {
                if let Some(frame) = self.stack.last_mut() {
                    frame.ctx.active_female = Some(key);
                }
                if let Some(npc) = world.female_npc(key) {
                    self.events.push_back(EngineEvent::NpcActivated(Some(
                        NpcActivatedData::from_npc(&npc.core, registry),
                    )));
                }
            }
        }
    }

    /// Drain all pending events, returning them in order.
    pub fn drain(&mut self) -> Vec<EngineEvent> {
        self.events.drain(..).collect()
    }

    /// Convenience: send a ChooseAction command and immediately drain events.
    /// Use this from the UI instead of calling send() + drain() separately.
    pub fn advance_with_action(
        &mut self,
        action_id: &str,
        world: &mut World,
        registry: &PackRegistry,
    ) -> Vec<EngineEvent> {
        self.send(
            EngineCommand::ChooseAction(action_id.to_string()),
            world,
            registry,
        );
        self.drain()
    }

    // -----------------------------------------------------------------------
    // Private: condition evaluation helper
    // -----------------------------------------------------------------------

    /// Evaluate a condition expression, logging errors and defaulting to false.
    fn eval_condition(
        expr: &undone_expr::parser::Expr,
        world: &World,
        ctx: &SceneCtx,
        registry: &PackRegistry,
        scene_id: &str,
        context: &str,
    ) -> bool {
        match eval(expr, world, ctx, registry) {
            Ok(val) => val,
            Err(e) => {
                eprintln!(
                    "[scene-engine] condition error in scene '{}' ({}): {}",
                    scene_id, context, e
                );
                false
            }
        }
    }

    // -----------------------------------------------------------------------
    // Private: scene lifecycle
    // -----------------------------------------------------------------------

    fn start_scene(&mut self, id: String, world: &World, registry: &PackRegistry) {
        self.transition_count += 1;
        if self.transition_count > MAX_TRANSITIONS_PER_COMMAND {
            eprintln!(
                "[scene-engine] transition limit: {} transitions reached starting '{id}'",
                self.transition_count
            );
            self.events.push_back(EngineEvent::ProseAdded(format!(
                "[Engine error: exceeded {} scene transitions — possible cycle involving '{id}']",
                MAX_TRANSITIONS_PER_COMMAND
            )));
            self.stack.clear();
            self.events.push_back(EngineEvent::NpcActivated(None));
            self.events.push_back(EngineEvent::SceneFinished);
            return;
        }

        let def = match self.scenes.get(&id) {
            Some(d) => Arc::clone(d),
            None => {
                eprintln!("[scene-engine] unknown scene: {id}");
                self.events.push_back(EngineEvent::ProseAdded(format!(
                    "[Error: scene not found: '{id}']"
                )));
                self.events.push_back(EngineEvent::SceneFinished);
                return;
            }
        };

        let mut ctx = SceneCtx::new();
        ctx.scene_id = Some(def.id.clone());

        // Select intro prose: use first passing variant, fall back to base intro
        let intro_prose =
            Self::select_intro_prose(&def.intro_variants, &def.intro_prose, world, &ctx, registry);

        // Render intro prose
        match render_prose(intro_prose, world, &ctx, registry) {
            Ok(prose) => self.events.push_back(EngineEvent::ProseAdded(prose)),
            Err(e) => self
                .events
                .push_back(EngineEvent::ProseAdded(format!("[template error: {e}]"))),
        }

        // Render intro thoughts
        Self::render_thoughts(
            &def.intro_thoughts,
            world,
            &ctx,
            registry,
            &mut self.events,
            &def.id,
        );

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
            let frame = self.stack.last().expect("engine stack must not be empty");
            match render_prose(&action.prose, world, &frame.ctx, registry) {
                Ok(prose) => self.events.push_back(EngineEvent::ProseAdded(prose)),
                Err(e) => self
                    .events
                    .push_back(EngineEvent::ProseAdded(format!("[template error: {e}]"))),
            }
        }

        // Render action thoughts (after prose, before effects)
        {
            let frame = self.stack.last().expect("engine stack must not be empty");
            let thoughts = action.thoughts.clone();
            let scene_id = frame.def.id.clone();
            Self::render_thoughts(
                &thoughts,
                world,
                &frame.ctx,
                registry,
                &mut self.events,
                &scene_id,
            );
        }

        // Apply effects
        {
            let frame = self
                .stack
                .last_mut()
                .expect("engine stack must not be empty");
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

    // -----------------------------------------------------------------------
    // Private: thought and narrator variant helpers
    // -----------------------------------------------------------------------

    /// Select the first narrator variant whose condition passes, or fall back to `base`.
    fn select_intro_prose<'a>(
        variants: &'a [NarratorVariant],
        base: &'a str,
        world: &World,
        ctx: &SceneCtx,
        registry: &PackRegistry,
    ) -> &'a str {
        for variant in variants {
            if Self::eval_condition(
                &variant.condition,
                world,
                ctx,
                registry,
                "variant",
                "intro_variant",
            ) {
                return &variant.prose;
            }
        }
        base
    }

    /// Evaluate and emit thought events for all thoughts whose conditions pass.
    fn render_thoughts(
        thoughts: &[Thought],
        world: &World,
        ctx: &SceneCtx,
        registry: &PackRegistry,
        events: &mut VecDeque<EngineEvent>,
        scene_id: &str,
    ) {
        for thought in thoughts {
            let passes = match &thought.condition {
                None => true,
                Some(expr) => Self::eval_condition(expr, world, ctx, registry, scene_id, "thought"),
            };
            if passes {
                match render_prose(&thought.prose, world, ctx, registry) {
                    Ok(text) if !text.trim().is_empty() => {
                        events.push_back(EngineEvent::ThoughtAdded {
                            text,
                            style: thought.style.clone(),
                        });
                    }
                    Ok(_) => {}
                    Err(e) => events.push_back(EngineEvent::ErrorOccurred(format!(
                        "thought prose error in scene '{scene_id}': {e}"
                    ))),
                }
            }
        }
    }

    fn emit_actions(&mut self, world: &World, registry: &PackRegistry) {
        let frame = match self.stack.last() {
            Some(f) => f,
            None => return,
        };

        let mut views = Vec::new();
        for action in &frame.def.actions {
            let visible = match &action.condition {
                Some(expr) => Self::eval_condition(
                    expr,
                    world,
                    &frame.ctx,
                    registry,
                    &frame.def.id,
                    &format!("action '{}'", action.id),
                ),
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

        self.events.push_back(EngineEvent::ActionsAvailable(views));
    }

    fn run_npc_actions(&mut self, world: &mut World, registry: &PackRegistry) {
        // Collect eligible NPC actions (condition passes) with their weights
        let npc_actions: Vec<(usize, u32)> = {
            let frame = self.stack.last().expect("engine stack must not be empty");
            frame
                .def
                .npc_actions
                .iter()
                .enumerate()
                .filter_map(|(i, na)| {
                    let eligible = match &na.condition {
                        Some(expr) => Self::eval_condition(
                            expr,
                            world,
                            &frame.ctx,
                            registry,
                            &frame.def.id,
                            &format!("npc_action '{}'", na.id),
                        ),
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
            let frame = self.stack.last().expect("engine stack must not be empty");
            let na = &frame.def.npc_actions[idx];
            (na.prose.clone(), na.effects.clone())
        };

        // Render NPC prose
        if !prose.is_empty() {
            let frame = self.stack.last().expect("engine stack must not be empty");
            match render_prose(&prose, world, &frame.ctx, registry) {
                Ok(rendered) => self.events.push_back(EngineEvent::ProseAdded(rendered)),
                Err(e) => self
                    .events
                    .push_back(EngineEvent::ProseAdded(format!("[template error: {e}]"))),
            }
        }

        // Apply NPC action effects
        {
            let frame = self
                .stack
                .last_mut()
                .expect("engine stack must not be empty");
            for effect in &effects {
                if let Err(e) = apply_effect(effect, world, &mut frame.ctx, registry) {
                    eprintln!("[scene-engine] npc effect error: {e}");
                }
            }
        }
    }

    fn evaluate_next(&mut self, branches: Vec<NextBranch>, world: &World, registry: &PackRegistry) {
        if branches.is_empty() {
            // No next branches — re-emit actions (loop)
            self.emit_actions(world, registry);
            return;
        }

        for branch in &branches {
            let condition_passes = match &branch.condition {
                Some(expr) => {
                    let frame = self.stack.last().expect("engine stack must not be empty");
                    Self::eval_condition(
                        expr,
                        world,
                        &frame.ctx,
                        registry,
                        &frame.def.id,
                        "next branch",
                    )
                }
                None => true,
            };

            if !condition_passes {
                continue;
            }

            if branch.finish {
                self.stack.pop();
                self.events.push_back(EngineEvent::NpcActivated(None));
                self.events.push_back(EngineEvent::SceneFinished);
                return;
            }

            if let Some(goto) = &branch.goto {
                let target = goto.clone();
                self.stack.pop();
                self.start_scene(target, world, registry);
                return;
            }

            if let Some(slot_name) = &branch.slot {
                let slot = slot_name.clone();
                self.stack.pop();
                self.events.push_back(EngineEvent::NpcActivated(None));
                self.events.push_back(EngineEvent::SlotRequested(slot));
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

    use crate::types::{EffectDef, SceneDefinition, Thought};

    fn make_world() -> World {
        World {
            player: Player {
                name_fem: "Eva".into(),
                name_androg: "Ev".into(),
                name_masc: "Evan".into(),
                before: Some(BeforeIdentity {
                    name: "Evan".into(),
                    age: Age::Twenties,
                    race: "white".into(),
                    sexuality: BeforeSexuality::AttractedToWomen,
                    figure: MaleFigure::Average,
                    traits: HashSet::new(),
                }),
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
                origin: PcOrigin::CisMaleTransformed,
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
                    effects: vec![],
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
                    effects: vec![EffectDef::ChangeStress { amount: -1 }],
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
            world.player.stress,
            stress_before - 1,
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
        use undone_expr::parse;

        // Build a scene with a conditional action
        let cond_expr = parse("scene.hasFlag('special')").unwrap();
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
                    effects: vec![],
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
                    effects: vec![],
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
                age: Age::Twenties,
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
                effects: vec![],
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
                effects: vec![],
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
            events
                .iter()
                .any(|e| matches!(e, EngineEvent::ActionsAvailable(v) if v.iter().any(|a| a.id == "wait"))),
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
    fn transition_guard_constant_is_reasonable() {
        // Verify the transition guard constant exists and is bounded.
        // The guard protects against future code paths that could cause
        // recursive transitions (currently the engine architecture only
        // allows one transition per command via goto). This is defensive
        // programming per engineering principle #5 (bounded resources).
        assert!(
            MAX_TRANSITIONS_PER_COMMAND >= 8,
            "limit too low — would block legitimate scene chains"
        );
        assert!(
            MAX_TRANSITIONS_PER_COMMAND <= 128,
            "limit too high — would allow runaway before tripping"
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
                effects: vec![],
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
                effects: vec![],
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
}
