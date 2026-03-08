pub mod char_creation;
pub mod game_state;
pub mod landing_page;
pub mod left_panel;
pub mod right_panel;
pub mod saves_panel;
pub mod settings_panel;
pub mod theme;
pub mod title_bar;

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::style::Position;
use floem::views::drag_resize_window_area;
use floem::window::ResizeDirection;
use std::cell::RefCell;
use std::rc::Rc;
use undone_domain::SkillId;
use undone_scene::engine::{ActionView, EngineCommand, EngineEvent};
use undone_world::World;

use crate::char_creation::{char_creation_view, fem_creation_view};
use crate::game_state::{init_game, GameState, PreGameState};
use crate::landing_page::landing_view;
use crate::left_panel::story_panel;
use crate::right_panel::sidebar_panel;
use crate::saves_panel::saves_panel;
use crate::settings_panel::settings_view;
use crate::theme::{ThemeColors, UserPrefs};
use crate::title_bar::title_bar;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppTab {
    Game,
    Saves,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppPhase {
    Landing,
    BeforeCreation,
    TransformationIntro,
    FemCreation,
    InGame,
}

/// Accumulated choices from BeforeCreation, passed forward to FemCreation.
#[derive(Clone)]
pub struct PartialCharState {
    pub origin: undone_domain::PcOrigin,
    pub before_name: String,
    pub before_age: undone_domain::Age,
    pub before_race: String,
    pub before_sexuality: undone_domain::BeforeSexuality,
    pub starting_traits: Vec<undone_domain::TraitId>,
    /// Starting game flags to seed at game start. Presets currently use this to
    /// opt into a route; custom players start freeform with no preset flags.
    pub starting_flags: Vec<String>,
    /// Preset index: 0=Robin, 1=Raul, None=custom.
    /// When set, FemCreation uses preset data for all physical attributes.
    pub preset_idx: Option<u8>,
    /// Appearance level selected in character creation.
    pub appearance: undone_domain::Appearance,
}

/// All reactive signals used by the view tree.
#[derive(Clone, Copy)]
pub struct AppSignals {
    pub story: RwSignal<String>,
    pub actions: RwSignal<Vec<ActionView>>,
    pub player: RwSignal<PlayerSnapshot>,
    pub active_npc: RwSignal<Option<NpcSnapshot>>,
    pub prefs: RwSignal<UserPrefs>,
    pub tab: RwSignal<AppTab>,
    pub phase: RwSignal<AppPhase>,
    pub scroll_gen: RwSignal<u64>,
}

impl Default for AppSignals {
    fn default() -> Self {
        Self::new()
    }
}

impl AppSignals {
    pub fn new() -> Self {
        Self {
            story: RwSignal::new(String::new()),
            actions: RwSignal::new(Vec::new()),
            player: RwSignal::new(PlayerSnapshot::default()),
            active_npc: RwSignal::new(None),
            prefs: RwSignal::new(crate::theme::load_prefs()),
            tab: RwSignal::new(AppTab::Game),
            phase: RwSignal::new(AppPhase::Landing),
            scroll_gen: RwSignal::new(0),
        }
    }
}

/// Display-ready snapshot of the player for the stats sidebar.
#[derive(Clone, Default)]
pub struct PlayerSnapshot {
    pub name: String,
    pub femininity: i32,
    pub money: i32,
    pub stress: i32,
    pub anxiety: i32,
    pub arousal: String, // e.g. "Comfort", "Enjoy"
    pub alcohol: String, // e.g. "Sober", "Tipsy"
}

impl PlayerSnapshot {
    /// Build a display snapshot from the player, reading FEMININITY from the skills map.
    pub fn from_player(p: &undone_domain::Player, femininity_id: SkillId) -> Self {
        Self {
            name: p.active_name(femininity_id).to_owned(),
            femininity: p.skill(femininity_id),
            money: p.money,
            stress: p.stress,
            anxiety: p.anxiety,
            arousal: format!("{}", p.arousal),
            alcohol: format!("{}", p.alcohol),
        }
    }
}

/// Display-ready snapshot of an active NPC.
#[derive(Clone)]
pub struct NpcSnapshot {
    pub name: String,
    pub age: String,
    pub personality: String,
    pub relationship: undone_domain::RelationshipStatus,
    pub pc_liking: undone_domain::LikingLevel,
    pub pc_attraction: undone_domain::AttractionLevel,
}

impl NpcSnapshot {
    /// Returns true once the NPC has moved beyond an unknown stranger state.
    pub fn is_known(&self) -> bool {
        !matches!(
            self.relationship,
            undone_domain::RelationshipStatus::Stranger
        ) || !matches!(self.pc_liking, undone_domain::LikingLevel::Neutral)
            || !matches!(
                self.pc_attraction,
                undone_domain::AttractionLevel::Unattracted
            )
    }
}

pub fn app_view() -> impl View {
    let signals = AppSignals::new();

    // Load packs (no world yet — waits for char creation)
    let pre_state: Rc<RefCell<Option<PreGameState>>> = Rc::new(RefCell::new(Some(init_game())));
    let game_state: Rc<RefCell<Option<GameState>>> = Rc::new(RefCell::new(None));

    // Accumulates choices across the three-phase creation flow.
    let partial_char: RwSignal<Option<PartialCharState>> = RwSignal::new(None);

    // Surface pack-load errors in the story panel immediately (shown when we transition to InGame).
    {
        let ps = pre_state.borrow();
        if let Some(ref pre) = *ps {
            if let Some(ref err) = pre.init_error {
                signals.story.set(err.clone());
            }
        }
    }

    let pre_state_cc = Rc::clone(&pre_state);
    let game_state_cc = Rc::clone(&game_state);
    let pre_state_lp = Rc::clone(&pre_state);
    let game_state_lp = Rc::clone(&game_state);
    let game_state_ig = Rc::clone(&game_state);

    let phase = signals.phase;

    let content = dyn_container(
        move || phase.get(),
        move |current_phase| match current_phase {
            AppPhase::Landing => dyn_container(move || signals.tab.get(), {
                let pre_state_lp = Rc::clone(&pre_state_lp);
                let game_state_lp = Rc::clone(&game_state_lp);
                move |tab| match tab {
                    AppTab::Settings => settings_view(signals).into_any(),
                    AppTab::Game | AppTab::Saves => {
                        landing_view(signals, Rc::clone(&pre_state_lp), Rc::clone(&game_state_lp))
                            .into_any()
                    }
                }
            })
            .style(|s| s.size_full())
            .into_any(),
            AppPhase::BeforeCreation => dyn_container(move || signals.tab.get(), {
                let pre_state_cc = Rc::clone(&pre_state_cc);
                let game_state_cc = Rc::clone(&game_state_cc);
                move |tab| match tab {
                    AppTab::Settings => settings_view(signals).into_any(),
                    _ => char_creation_view(
                        signals,
                        Rc::clone(&pre_state_cc),
                        Rc::clone(&game_state_cc),
                        partial_char,
                    )
                    .into_any(),
                }
            })
            .style(|s| s.size_full())
            .into_any(),
            AppPhase::TransformationIntro => {
                // Start the transformation scene against the throwaway world
                // (created in the "Next" button handler in char_creation.rs).
                let gs_ref = Rc::clone(&game_state_ig);
                {
                    let mut gs_opt = gs_ref.borrow_mut();
                    if let Some(ref mut gs) = *gs_opt {
                        let fem_id = gs.femininity_id;
                        let GameState {
                            ref mut engine,
                            ref mut world,
                            ref registry,
                            ..
                        } = *gs;
                        if let Some(scene_id) = registry.transformation_scene() {
                            let scene_id = scene_id.to_owned();
                            start_scene(engine, world, registry, scene_id);
                        }
                        let events = engine.drain();
                        process_events(events, signals, world, fem_id);
                    }
                }

                let inner_gs: GameState = match gs_ref.borrow_mut().take() {
                    Some(gs) => gs,
                    None => {
                        return placeholder_panel(
                            "Transformation intro: game state missing",
                            signals,
                        )
                        .into_any();
                    }
                };
                let gs_cell: Rc<RefCell<GameState>> = Rc::new(RefCell::new(inner_gs));

                dyn_container(move || signals.tab.get(), {
                    let gs_cell = Rc::clone(&gs_cell);
                    move |tab| match tab {
                        AppTab::Settings => settings_view(signals).into_any(),
                        _ => h_stack((
                            sidebar_panel(signals),
                            story_panel(signals, Rc::clone(&gs_cell)),
                        ))
                        .style(|s| s.size_full())
                        .into_any(),
                    }
                })
                .style(|s| s.size_full())
                .into_any()
            }
            AppPhase::FemCreation => dyn_container(move || signals.tab.get(), {
                let pre_state_cc = Rc::clone(&pre_state_cc);
                let game_state_cc = Rc::clone(&game_state_cc);
                move |tab| match tab {
                    AppTab::Settings => settings_view(signals).into_any(),
                    _ => fem_creation_view(
                        signals,
                        Rc::clone(&pre_state_cc),
                        Rc::clone(&game_state_cc),
                        partial_char,
                    )
                    .into_any(),
                }
            })
            .style(|s| s.size_full())
            .into_any(),
            AppPhase::InGame => {
                // On first transition to InGame, start either opening scene (new game)
                // or the next eligible scheduled scene (loaded save).
                let gs_ref = Rc::clone(&game_state_ig);
                {
                    let mut gs_opt = gs_ref.borrow_mut();
                    if let Some(ref mut gs) = *gs_opt {
                        if gs.init_error.is_none() {
                            let fem_id = gs.femininity_id;
                            let GameState {
                                ref mut engine,
                                ref mut world,
                                ref registry,
                                ref scheduler,
                                ref mut rng,
                                ref mut opening_scene,
                                ..
                            } = *gs;

                            // Clear leftover prose from previous phases
                            // (e.g. TransformationIntro text surviving into InGame).
                            signals.story.set(String::new());

                            // Scheduler takes priority: arc triggers (e.g.
                            // workplace_arrival for ROUTE_WORKPLACE) must fire before
                            // the generic opening_scene fallback from pack.toml.
                            let mut started_scene = false;
                            if let Some(result) = scheduler.pick_next(world, registry, rng) {
                                // Scheduler found an eligible scene — discard opening_scene.
                                let _ = opening_scene.take();
                                if result.once_only {
                                    world
                                        .game_data
                                        .set_flag(format!("ONCE_{}", result.scene_id));
                                }
                                start_scene(engine, world, registry, result.scene_id);
                                started_scene = true;
                            } else if let Some(scene_id) = opening_scene.take() {
                                // No scheduled scene — use pack's opening_scene
                                // (custom route with no arc flags).
                                start_scene(engine, world, registry, scene_id);
                                started_scene = true;
                            }

                            if started_scene {
                                let events = engine.drain();
                                let finished = process_events(events, signals, world, fem_id);
                                if finished {
                                    // Clear story for the next scene — clean page turn.
                                    signals.story.set(String::new());
                                    if let Some(result) = scheduler.pick_next(world, registry, rng)
                                    {
                                        if result.once_only {
                                            world
                                                .game_data
                                                .set_flag(format!("ONCE_{}", result.scene_id));
                                        }
                                        start_scene(engine, world, registry, result.scene_id);
                                        let events = engine.drain();
                                        process_events(events, signals, world, fem_id);
                                    }
                                }
                            } else {
                                signals
                                    .story
                                    .set("[No eligible scene is currently available.]".to_string());
                                signals.actions.set(vec![]);
                                signals
                                    .player
                                    .set(PlayerSnapshot::from_player(&world.player, fem_id));
                            }
                        }
                    }
                }

                // Convert Rc<RefCell<Option<GameState>>> into Rc<RefCell<GameState>>
                // by extracting the value once at transition time.
                let inner_gs: GameState = match gs_ref.borrow_mut().take() {
                    Some(gs) => gs,
                    None => {
                        return placeholder_panel("Game state missing", signals).into_any();
                    }
                };
                let gs_cell: Rc<RefCell<GameState>> = Rc::new(RefCell::new(inner_gs));

                dyn_container(move || signals.tab.get(), {
                    let gs_cell = Rc::clone(&gs_cell);
                    move |tab| match tab {
                        AppTab::Settings => settings_view(signals).into_any(),
                        AppTab::Saves => saves_panel(signals, Rc::clone(&gs_cell)).into_any(),
                        AppTab::Game => h_stack((
                            sidebar_panel(signals),
                            story_panel(signals, Rc::clone(&gs_cell)),
                        ))
                        .style(|s| s.size_full())
                        .into_any(),
                    }
                })
                .style(|s| s.flex_grow(1.0))
                .into_any()
            }
        },
    )
    .style(|s| s.flex_grow(1.0).flex_basis(0.0).min_height(0.0));

    // Title bar is always visible (both CharCreation and InGame phases).
    let body = v_stack((title_bar(signals), content)).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.size_full().background(colors.ground)
    });

    let main_column = body.style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.size_full().background(colors.ground)
    });

    // Resize grips — thin invisible strips on all edges/corners.
    // Required because show_titlebar(false) removes the OS resize borders.
    let grip = 5.0; // px, invisible hit area

    let top = drag_resize_window_area(ResizeDirection::North, empty())
        .style(move |s| s.width_full().height(grip).position(Position::Absolute));

    let bottom = drag_resize_window_area(ResizeDirection::South, empty()).style(move |s| {
        s.width_full()
            .height(grip)
            .position(Position::Absolute)
            .inset_bottom(0.0)
    });

    let left = drag_resize_window_area(ResizeDirection::West, empty()).style(move |s| {
        s.width(grip)
            .height_full()
            .position(Position::Absolute)
            .inset_left(0.0)
    });

    let right = drag_resize_window_area(ResizeDirection::East, empty()).style(move |s| {
        s.width(grip)
            .height_full()
            .position(Position::Absolute)
            .inset_right(0.0)
    });

    let top_left = drag_resize_window_area(ResizeDirection::NorthWest, empty()).style(move |s| {
        s.width(grip)
            .height(grip)
            .position(Position::Absolute)
            .inset_top(0.0)
            .inset_left(0.0)
    });

    let top_right = drag_resize_window_area(ResizeDirection::NorthEast, empty()).style(move |s| {
        s.width(grip)
            .height(grip)
            .position(Position::Absolute)
            .inset_top(0.0)
            .inset_right(0.0)
    });

    let bottom_left =
        drag_resize_window_area(ResizeDirection::SouthWest, empty()).style(move |s| {
            s.width(grip)
                .height(grip)
                .position(Position::Absolute)
                .inset_bottom(0.0)
                .inset_left(0.0)
        });

    let bottom_right =
        drag_resize_window_area(ResizeDirection::SouthEast, empty()).style(move |s| {
            s.width(grip)
                .height(grip)
                .position(Position::Absolute)
                .inset_bottom(0.0)
                .inset_right(0.0)
        });

    (
        main_column,
        top,
        bottom,
        left,
        right,
        top_left,
        top_right,
        bottom_left,
        bottom_right,
    )
        .style(|s| s.size_full())
}

/// Start a scene and wire in the active NPCs so effects like `set_npc_role` and
/// `add_npc_liking` can resolve their `npc = "m"` / `npc = "f"` references.
///
/// The game loop is responsible for this — the engine only stores the active NPC
/// keys, it doesn't pick them. For now we activate the first male and first female
/// NPC in the world's slotmaps (the spawner guarantees at least one of each).
pub fn start_scene(
    engine: &mut undone_scene::engine::SceneEngine,
    world: &mut World,
    registry: &undone_packs::PackRegistry,
    scene_id: String,
) {
    engine.send(EngineCommand::StartScene(scene_id), world, registry);
    if let Some((key, _)) = world.male_npcs.iter().next() {
        engine.send(EngineCommand::SetActiveMale(key), world, registry);
    }
    if let Some((key, _)) = world.female_npcs.iter().next() {
        engine.send(EngineCommand::SetActiveFemale(key), world, registry);
    }
}

const MAX_STORY_PARAGRAPHS: usize = 200;

fn trim_story_paragraphs(story: &mut String) {
    let para_count = story.split("\n\n").count();
    if para_count <= MAX_STORY_PARAGRAPHS {
        return;
    }

    let to_drop = para_count - MAX_STORY_PARAGRAPHS;
    let mut remaining = to_drop;
    let mut byte_offset = 0;
    for (i, _) in story.match_indices("\n\n") {
        remaining -= 1;
        if remaining == 0 {
            byte_offset = i + 2;
            break;
        }
    }
    if byte_offset > 0 {
        *story = story[byte_offset..].to_string();
    }
}

fn append_story_paragraph(story: &mut String, text: &str) {
    if !story.is_empty() {
        story.push_str("\n\n");
    }
    story.push_str(text);
    trim_story_paragraphs(story);
}

/// Process engine events, updating signals. Returns `true` if `SceneFinished` was among them.
pub fn process_events(
    events: Vec<EngineEvent>,
    signals: AppSignals,
    world: &World,
    femininity_id: SkillId,
) -> bool {
    let mut scene_finished = false;
    for event in events {
        match event {
            EngineEvent::ProseAdded(text) => {
                // Only scroll to bottom when appending to existing prose (i.e.
                // after the player picks an action). On the first prose of a new
                // scene the story starts empty — we want the viewport at the top.
                let should_scroll = !signals.story.get_untracked().is_empty();
                signals.story.update(|s| {
                    append_story_paragraph(s, &text);
                });
                if should_scroll {
                    // Delay the scroll-to-bottom by one frame. The scroll_to
                    // effect uses update_state_deferred which fires after layout
                    // in the same frame — but the content rebuild (from the story
                    // signal) may not have its new taffy size computed yet in that
                    // same layout pass. Deferring to the next frame via exec_after
                    // ensures layout has fully settled with the new content.
                    let sg = signals.scroll_gen;
                    floem::action::exec_after(std::time::Duration::ZERO, move |_| {
                        sg.update(|n| *n += 1);
                    });
                }
            }
            EngineEvent::ActionsAvailable(actions) => {
                signals.actions.set(actions);
            }
            EngineEvent::NpcActivated(data) => {
                if let Some(d) = data.as_ref() {
                    let next = NpcSnapshot {
                        name: d.name.clone(),
                        age: format!("{}", d.age),
                        personality: d.personality.clone(),
                        relationship: d.relationship.clone(),
                        pc_liking: d.pc_liking,
                        pc_attraction: d.pc_attraction,
                    };
                    signals.active_npc.update(|slot| match slot {
                        Some(current) if current.is_known() && !next.is_known() => {
                            // Keep meaningful context if a placeholder stranger activation
                            // event arrives after a known NPC in the same event burst.
                        }
                        _ => *slot = Some(next),
                    });
                } else {
                    signals.active_npc.set(None);
                }
            }
            EngineEvent::SceneFinished => {
                signals.actions.set(vec![]);
                scene_finished = true;
            }
            EngineEvent::ThoughtAdded { text, .. } => {
                // Thoughts append to the story text. Style differentiation (italic,
                // anxiety register, etc.) is deferred to the UI design session.
                signals.story.update(|s| {
                    append_story_paragraph(s, &text);
                });
                let sg = signals.scroll_gen;
                floem::action::exec_after(std::time::Duration::ZERO, move |_| {
                    sg.update(|n| *n += 1);
                });
            }
            EngineEvent::SlotRequested(_slot) => {
                // Slot routing is handled by the caller (left_panel dispatch);
                // the UI event processor ignores it here.
            }
            EngineEvent::ErrorOccurred(msg) => {
                signals.story.update(|s| {
                    append_story_paragraph(s, &format!("[Scene error: {}]", msg));
                });
                let sg = signals.scroll_gen;
                floem::action::exec_after(std::time::Duration::ZERO, move |_| {
                    sg.update(|n| *n += 1);
                });
            }
        }
    }
    signals
        .player
        .set(PlayerSnapshot::from_player(&world.player, femininity_id));
    scene_finished
}

fn placeholder_panel(msg: &'static str, signals: AppSignals) -> impl View {
    container(label(move || msg.to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.color(colors.ink_dim).font_size(16.0)
    }))
    .style(|s| s.size_full().items_center().justify_center())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lasso::Key;
    use slotmap::SlotMap;
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;
    use undone_domain::*;
    use undone_packs::PackRegistry;
    use undone_scene::engine::{EngineCommand, SceneEngine};
    use undone_scene::types::{Action, EffectDef, NextBranch, SceneDefinition};
    use undone_world::{GameData, World};

    fn test_player() -> Player {
        Player {
            name_fem: "Eva".into(),
            name_masc: "Evan".into(),
            before: Some(BeforeIdentity {
                name: "Evan".into(),
                age: Age::MidLateTwenties,
                race: "white".into(),
                sexuality: BeforeSexuality::AttractedToWomen,
                figure: MaleFigure::Average,
                height: Height::Average,
                hair_colour: HairColour::DarkBrown,
                eye_colour: EyeColour::Brown,
                skin_tone: SkinTone::Medium,
                penis_size: PenisSize::Average,
                voice: BeforeVoice::Average,
                traits: HashSet::new(),
            }),
            age: Age::LateTeen,
            race: "white".into(),
            figure: PlayerFigure::Slim,
            breasts: BreastSize::Full,
            eye_colour: EyeColour::Blue,
            hair_colour: HairColour::Blonde,
            height: Height::Average,
            hair_length: HairLength::Shoulder,
            skin_tone: SkinTone::Medium,
            complexion: Complexion::Normal,
            appearance: Appearance::Average,
            butt: ButtSize::Round,
            waist: WaistSize::Average,
            lips: LipShape::Average,
            nipple_sensitivity: NippleSensitivity::Normal,
            clit_sensitivity: ClitSensitivity::Normal,
            pubic_hair: PubicHairStyle::Trimmed,
            natural_pubic_hair: NaturalPubicHair::Full,
            inner_labia: InnerLabiaSize::Average,
            wetness_baseline: WetnessBaseline::Normal,
            traits: HashSet::new(),
            skills: HashMap::new(),
            money: 200,
            stress: 5,
            anxiety: 2,
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
        }
    }

    fn test_world() -> World {
        World {
            player: test_player(),
            male_npcs: SlotMap::with_key(),
            female_npcs: SlotMap::with_key(),
            game_data: GameData::default(),
        }
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

    #[test]
    fn player_snapshot_name_uses_active_name() {
        // femininity=25 → masculine name; set via skills map
        let fem_id = SkillId(lasso::Spur::try_from_usize(0).unwrap());
        let mut p = test_player();
        p.skills.insert(
            fem_id,
            undone_domain::SkillValue {
                value: 25,
                modifier: 0,
            },
        );
        let snap = PlayerSnapshot::from_player(&p, fem_id);
        assert_eq!(snap.name, "Evan"); // femininity=25 → masc
    }

    #[test]
    fn player_snapshot_captures_money() {
        let fem_id = SkillId(lasso::Spur::try_from_usize(0).unwrap());
        let p = test_player();
        let snap = PlayerSnapshot::from_player(&p, fem_id);
        assert_eq!(snap.money, 200);
    }

    #[test]
    fn append_story_paragraph_trims_to_latest_limit() {
        let mut story = String::new();
        for i in 0..205 {
            append_story_paragraph(&mut story, &format!("p{i}"));
        }

        let paragraphs: Vec<&str> = story.split("\n\n").collect();
        assert_eq!(paragraphs.len(), MAX_STORY_PARAGRAPHS);
        assert_eq!(paragraphs.first().copied(), Some("p5"));
        assert_eq!(paragraphs.last().copied(), Some("p204"));
    }

    #[test]
    fn append_story_paragraph_separates_entries() {
        let mut story = String::new();
        append_story_paragraph(&mut story, "one");
        append_story_paragraph(&mut story, "two");
        assert_eq!(story, "one\n\ntwo");
    }

    #[test]
    fn npc_snapshot_is_known_false_for_stranger_defaults() {
        let npc = NpcSnapshot {
            name: "Alex".to_string(),
            age: "Twenty".to_string(),
            personality: "Calm".to_string(),
            relationship: RelationshipStatus::Stranger,
            pc_liking: LikingLevel::Neutral,
            pc_attraction: AttractionLevel::Unattracted,
        };
        assert!(!npc.is_known());
    }

    #[test]
    fn npc_snapshot_is_known_true_after_relationship_progress() {
        let npc = NpcSnapshot {
            name: "Alex".to_string(),
            age: "Twenty".to_string(),
            personality: "Calm".to_string(),
            relationship: RelationshipStatus::Acquaintance,
            pc_liking: LikingLevel::Neutral,
            pc_attraction: AttractionLevel::Unattracted,
        };
        assert!(npc.is_known());
    }

    #[test]
    fn process_events_appends_error_occurred_to_story_output() {
        let signals = AppSignals::new();
        let fem_id = SkillId(lasso::Spur::try_from_usize(0).unwrap());
        let world = test_world();

        let finished = process_events(
            vec![EngineEvent::ErrorOccurred(
                "[scene-engine] template error in scene 'test::scene' (intro prose): boom".into(),
            )],
            signals,
            &world,
            fem_id,
        );

        assert!(!finished);
        assert!(signals.story.get().contains("[Scene error:"));
        assert!(signals.story.get().contains("template error"));
    }

    #[test]
    fn start_scene_binds_first_male_for_followup_action_effects() {
        let scene = SceneDefinition {
            id: "test::npc_binding".into(),
            pack: "test".into(),
            intro_prose: "Intro.".into(),
            intro_variants: vec![],
            intro_thoughts: vec![],
            actions: vec![Action {
                id: "go".into(),
                label: "Go".into(),
                detail: String::new(),
                condition: None,
                prose: String::new(),
                allow_npc_actions: false,
                effects: vec![EffectDef::AddNpcLiking {
                    npc: "m".into(),
                    delta: 1,
                }],
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

        let mut scenes = HashMap::new();
        scenes.insert(scene.id.clone(), Arc::new(scene));

        let mut engine = SceneEngine::new(scenes);
        let mut world = test_world();
        let mut registry = PackRegistry::new();
        let personality = registry.intern_personality("ROMANTIC");
        let male_key = world.male_npcs.insert(test_male_npc(personality));

        start_scene(
            &mut engine,
            &mut world,
            &registry,
            "test::npc_binding".into(),
        );
        engine.drain();

        engine.send(
            EngineCommand::ChooseAction("go".into()),
            &mut world,
            &registry,
        );
        let events = engine.drain();

        assert_eq!(world.male_npcs[male_key].core.pc_liking, LikingLevel::Ok);
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, EngineEvent::ErrorOccurred(_))),
            "fallback binding should make active-male effects safe after scene start: {:?}",
            events
        );
    }
}
