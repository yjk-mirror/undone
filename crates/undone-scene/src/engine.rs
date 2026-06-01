use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use crate::scene_ctx::{SceneCtx, SceneNpcRef};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use undone_domain::{FemaleNpcKey, MaleNpcKey};
use undone_packs::PackRegistry;
use undone_world::World;

use crate::{
    script::{apply_effect_script, eval_bool, CompiledScript},
    template_ctx::render_prose,
    types::{Action, NarratorVariant, NextBranch, SceneDefinition, Thought},
};

/// Maximum scene transitions per command. Prevents both deep sub-scene stacks
/// and flat goto cycles (where the stack stays at depth 1 but transitions loop).
const MAX_TRANSITIONS_PER_COMMAND: usize = 32;

const _: () = {
    assert!(
        MAX_TRANSITIONS_PER_COMMAND >= 8,
        "transition guard too low for legitimate scene chains"
    );
    assert!(
        MAX_TRANSITIONS_PER_COMMAND <= 128,
        "transition guard too high for runaway protection"
    );
};

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
            name: npc.effective_name().to_string(),
            age: npc.age,
            personality: registry.personality_name(npc.personality).to_owned(),
            relationship: npc.relationship.clone(),
            pc_liking: npc.pc_liking,
            pc_attraction: npc.pc_attraction,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoundNpcData {
    pub binding: String,
    pub npc: NpcActivatedData,
}

#[derive(Debug, Clone)]
pub struct ActionView {
    pub id: String,
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SceneSummary {
    pub id: String,
    pub pack: String,
    pub description: String,
    pub action_count: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SceneInfo {
    pub id: String,
    pub pack: String,
    pub actions: Vec<SceneActionInfo>,
    pub npc_action_count: usize,
    pub has_intro_variants: bool,
    pub has_thoughts: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SceneActionInfo {
    pub id: String,
    pub label: String,
    pub detail: String,
    pub has_condition: bool,
    pub has_next: bool,
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
                self.start_scene(id, world, registry, None, None, HashMap::new());
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

    pub fn scene_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.scenes.keys().cloned().collect();
        ids.sort();
        ids
    }

    pub fn has_scene(&self, scene_id: &str) -> bool {
        self.scenes.contains_key(scene_id)
    }

    /// Return a summary of every loaded scene (id, pack, description, action count).
    pub fn all_scene_summaries(&self) -> Vec<SceneSummary> {
        let mut summaries: Vec<SceneSummary> = self
            .scenes
            .values()
            .map(|def| SceneSummary {
                id: def.id.clone(),
                pack: def.pack.clone(),
                description: def.intro_prose.chars().take(120).collect::<String>()
                    + if def.intro_prose.len() > 120 {
                        "..."
                    } else {
                        ""
                    },
                action_count: def.actions.len(),
            })
            .collect();
        summaries.sort_by(|a, b| a.id.cmp(&b.id));
        summaries
    }

    /// Return detailed info about a single scene.
    pub fn scene_info(&self, scene_id: &str) -> Option<SceneInfo> {
        let def = self.scenes.get(scene_id)?;
        Some(SceneInfo {
            id: def.id.clone(),
            pack: def.pack.clone(),
            actions: def
                .actions
                .iter()
                .map(|a| SceneActionInfo {
                    id: a.id.clone(),
                    label: a.label.clone(),
                    detail: a.detail.clone(),
                    has_condition: a.condition.is_some(),
                    has_next: !a.next.is_empty(),
                })
                .collect(),
            npc_action_count: def.npc_actions.len(),
            has_intro_variants: !def.intro_variants.is_empty(),
            has_thoughts: !def.intro_thoughts.is_empty(),
        })
    }

    pub fn current_scene_id(&self) -> Option<String> {
        self.stack.last().map(|frame| frame.def.id.clone())
    }

    pub fn current_bound_npcs(&self, world: &World, registry: &PackRegistry) -> Vec<BoundNpcData> {
        let Some(frame) = self.stack.last() else {
            return Vec::new();
        };

        let mut bound = Vec::new();
        if let Some(key) = frame.ctx.active_male {
            if let Some(npc) = world.male_npc(key) {
                bound.push(BoundNpcData {
                    binding: "m".into(),
                    npc: NpcActivatedData::from_npc(&npc.core, registry),
                });
            }
        }
        if let Some(key) = frame.ctx.active_female {
            if let Some(npc) = world.female_npc(key) {
                bound.push(BoundNpcData {
                    binding: "f".into(),
                    npc: NpcActivatedData::from_npc(&npc.core, registry),
                });
            }
        }

        let mut roles: Vec<_> = frame.ctx.role_bindings.iter().collect();
        roles.sort_by(|left, right| left.0.cmp(right.0));
        for (role, npc_ref) in roles {
            match npc_ref {
                SceneNpcRef::Male(key) => {
                    if let Some(npc) = world.male_npc(*key) {
                        bound.push(BoundNpcData {
                            binding: role.clone(),
                            npc: NpcActivatedData::from_npc(&npc.core, registry),
                        });
                    }
                }
                SceneNpcRef::Female(key) => {
                    if let Some(npc) = world.female_npc(*key) {
                        bound.push(BoundNpcData {
                            binding: role.clone(),
                            npc: NpcActivatedData::from_npc(&npc.core, registry),
                        });
                    }
                }
            }
        }

        bound
    }

    pub fn start_scene_with_bindings(
        &mut self,
        scene_id: String,
        active_male: Option<MaleNpcKey>,
        active_female: Option<FemaleNpcKey>,
        world: &World,
        registry: &PackRegistry,
    ) {
        self.transition_count = 0;
        self.start_scene(
            scene_id,
            world,
            registry,
            active_male,
            active_female,
            HashMap::new(),
        );
    }

    pub fn start_scene_with_role_bindings(
        &mut self,
        scene_id: String,
        active_male: Option<MaleNpcKey>,
        active_female: Option<FemaleNpcKey>,
        role_bindings: HashMap<String, SceneNpcRef>,
        world: &World,
        registry: &PackRegistry,
    ) {
        self.transition_count = 0;
        self.start_scene(
            scene_id,
            world,
            registry,
            active_male,
            active_female,
            role_bindings,
        );
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

    /// Clear runtime state before starting a fresh flow (for example after loading a save).
    /// Scene definitions are kept; only stack, queued events, and transition counter are reset.
    pub fn reset_runtime(&mut self) {
        self.stack.clear();
        self.events.clear();
        self.transition_count = 0;
    }

    // -----------------------------------------------------------------------
    // Private: condition evaluation helper
    // -----------------------------------------------------------------------

    /// Evaluate a condition expression, logging errors and defaulting to false.
    fn eval_condition(
        expr: &CompiledScript,
        world: &World,
        ctx: &SceneCtx,
        registry: &PackRegistry,
        scene_id: &str,
        context: &str,
        events: Option<&mut VecDeque<EngineEvent>>,
    ) -> bool {
        match eval_bool(expr, world, ctx, registry) {
            Ok(val) => val,
            Err(e) => {
                let msg = format!(
                    "[scene-engine] condition error in scene '{}' ({}): {}",
                    scene_id, context, e
                );
                log::warn!("{msg}");
                if let Some(events) = events {
                    events.push_back(EngineEvent::ErrorOccurred(msg));
                }
                false
            }
        }
    }

    fn emit_template_error(
        events: &mut VecDeque<EngineEvent>,
        scene_id: &str,
        context: &str,
        err: &impl std::fmt::Display,
    ) {
        let msg = format!(
            "[scene-engine] template error in scene '{}' ({}): {}",
            scene_id, context, err
        );
        log::warn!("{msg}");
        events.push_back(EngineEvent::ErrorOccurred(msg));
    }

    // -----------------------------------------------------------------------
    // Private: scene lifecycle
    // -----------------------------------------------------------------------

    fn start_scene(
        &mut self,
        id: String,
        world: &World,
        registry: &PackRegistry,
        active_male: Option<MaleNpcKey>,
        active_female: Option<FemaleNpcKey>,
        role_bindings: HashMap<String, SceneNpcRef>,
    ) {
        self.transition_count += 1;
        if self.transition_count > MAX_TRANSITIONS_PER_COMMAND {
            log::error!(
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
                log::error!("[scene-engine] unknown scene: {id}");
                self.events.push_back(EngineEvent::ProseAdded(format!(
                    "[Error: scene not found: '{id}']"
                )));
                self.events.push_back(EngineEvent::SceneFinished);
                return;
            }
        };

        let mut ctx = SceneCtx::new();
        ctx.scene_id = Some(def.id.clone());
        ctx.active_male = active_male;
        ctx.active_female = active_female;
        ctx.role_bindings = role_bindings;

        let intro_prose = Self::select_intro_prose(
            &def.intro_variants,
            &def.intro_prose,
            world,
            &ctx,
            registry,
            &def.id,
            &mut self.events,
        );

        match render_prose(intro_prose, world, &ctx, registry) {
            Ok(prose) => self.events.push_back(EngineEvent::ProseAdded(prose)),
            Err(e) => Self::emit_template_error(&mut self.events, &def.id, "intro prose", &e),
        }

        Self::render_thoughts(
            &def.intro_thoughts,
            world,
            &ctx,
            registry,
            &mut self.events,
            &def.id,
        );

        self.stack.push(SceneFrame { def, ctx });
        if let Some(key) = active_male {
            if let Some(npc) = world.male_npc(key) {
                self.events
                    .push_back(EngineEvent::NpcActivated(Some(NpcActivatedData::from_npc(
                        &npc.core, registry,
                    ))));
            }
        }
        if let Some(key) = active_female {
            if let Some(npc) = world.female_npc(key) {
                self.events
                    .push_back(EngineEvent::NpcActivated(Some(NpcActivatedData::from_npc(
                        &npc.core, registry,
                    ))));
            }
        }

        self.emit_actions(world, registry);
    }

    fn choose_action(&mut self, action_id: String, world: &mut World, registry: &PackRegistry) {
        let frame = match self.stack.last() {
            Some(f) => f,
            None => return,
        };

        let action: Action = match frame.def.actions.iter().find(|a| a.id == action_id) {
            Some(a) => a.clone(),
            None => return,
        };

        // Re-check action condition — it may have become invalid since actions were displayed
        if let Some(ref expr) = action.condition {
            if !Self::eval_condition(
                expr,
                world,
                &frame.ctx,
                registry,
                &frame.def.id,
                &format!("action '{}'", action.id),
                Some(&mut self.events),
            ) {
                // Condition no longer passes — silently ignore the stale click.
                // Re-emit current actions so the UI refreshes.
                self.emit_actions(world, registry);
                return;
            }
        }

        let allow_npc = action.allow_npc_actions;

        if !action.prose.is_empty() {
            let frame = self.stack.last().expect("engine stack must not be empty");
            match render_prose(&action.prose, world, &frame.ctx, registry) {
                Ok(prose) => self.events.push_back(EngineEvent::ProseAdded(prose)),
                Err(e) => Self::emit_template_error(
                    &mut self.events,
                    &frame.def.id,
                    &format!("action '{}'", action.id),
                    &e,
                ),
            }
        }

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

        let effect_errors: Vec<String> = {
            let frame = self
                .stack
                .last_mut()
                .expect("engine stack must not be empty");
            match &action.effect {
                Some(script) => apply_effect_script(script, world, &mut frame.ctx, registry),
                None => Vec::new(),
            }
        };
        for msg in &effect_errors {
            log::warn!("{msg}");
        }
        for msg in effect_errors {
            self.events.push_back(EngineEvent::ErrorOccurred(msg));
        }

        if allow_npc {
            self.run_npc_actions(world, registry);
        }

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
        scene_id: &str,
        events: &mut VecDeque<EngineEvent>,
    ) -> &'a str {
        for variant in variants {
            if Self::eval_condition(
                &variant.condition,
                world,
                ctx,
                registry,
                scene_id,
                "intro_variant",
                Some(events),
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
                Some(expr) => Self::eval_condition(
                    expr,
                    world,
                    ctx,
                    registry,
                    scene_id,
                    "thought",
                    Some(events),
                ),
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
                    Err(e) => Self::emit_template_error(events, scene_id, "thought prose", &e),
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
                    Some(&mut self.events),
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
                            Some(&mut self.events),
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

        let (prose, effect, next_branches): (String, Option<_>, Vec<_>) = {
            let frame = self.stack.last().expect("engine stack must not be empty");
            let Some(na) = frame.def.npc_actions.get(idx) else {
                log::error!(
                    "NPC action index {} out of bounds (len {}) in scene '{}'",
                    idx,
                    frame.def.npc_actions.len(),
                    frame.def.id
                );
                return;
            };
            (na.prose.clone(), na.effect.clone(), na.next.clone())
        };

        if !prose.is_empty() {
            let frame = self.stack.last().expect("engine stack must not be empty");
            match render_prose(&prose, world, &frame.ctx, registry) {
                Ok(rendered) => self.events.push_back(EngineEvent::ProseAdded(rendered)),
                Err(e) => Self::emit_template_error(
                    &mut self.events,
                    &frame.def.id,
                    "npc action prose",
                    &e,
                ),
            }
        }

        let npc_effect_errors: Vec<String> = {
            let frame = self
                .stack
                .last_mut()
                .expect("engine stack must not be empty");
            match &effect {
                Some(script) => apply_effect_script(script, world, &mut frame.ctx, registry),
                None => Vec::new(),
            }
        };
        for msg in &npc_effect_errors {
            log::warn!("{msg}");
        }
        for msg in npc_effect_errors {
            self.events.push_back(EngineEvent::ErrorOccurred(msg));
        }

        if !next_branches.is_empty() {
            self.evaluate_next(next_branches, world, registry);
        }
    }

    fn evaluate_next(&mut self, branches: Vec<NextBranch>, world: &World, registry: &PackRegistry) {
        if branches.is_empty() {
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
                        Some(&mut self.events),
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
                self.start_scene(target, world, registry, None, None, HashMap::new());
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

        self.emit_actions(world, registry);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

// Hardcoded content-ID audit: no runtime content ID literals in engine.rs.
// Test code below uses IDs like "ROMANTIC" as fixture data — acceptable.
#[cfg(test)]
#[allow(non_snake_case)]
mod tests;
