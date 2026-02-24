pub mod char_creation;
pub mod game_state;
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

use crate::char_creation::char_creation_view;
use crate::game_state::{init_game, GameState, PreGameState};
use crate::left_panel::story_panel;
use crate::right_panel::sidebar_panel;
use crate::saves_panel::saves_panel;
use crate::settings_panel::settings_view;
use crate::theme::{ThemeColors, UserPrefs};
use crate::title_bar::title_bar;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    Game,
    Saves,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppPhase {
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
            phase: RwSignal::new(AppPhase::BeforeCreation),
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
    pub relationship: String,
    pub pc_liking: String,
    pub pc_attraction: String,
}

impl From<&undone_domain::NpcCore> for NpcSnapshot {
    fn from(npc: &undone_domain::NpcCore) -> Self {
        Self {
            name: npc.name.clone(),
            age: format!("{}", npc.age),
            personality: format!("{:?}", npc.personality),
            relationship: format!("{}", npc.relationship),
            pc_liking: format!("{}", npc.pc_liking),
            pc_attraction: format!("{}", npc.pc_attraction),
        }
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
    let game_state_ig = Rc::clone(&game_state);

    let phase = signals.phase;

    let content = dyn_container(
        move || phase.get(),
        move |current_phase| match current_phase {
            AppPhase::BeforeCreation => {
                char_creation_view(
                    signals,
                    Rc::clone(&pre_state_cc),
                    Rc::clone(&game_state_cc),
                    partial_char,
                )
                .into_any()
            }
            AppPhase::TransformationIntro => {
                // TODO Task 5: wire transformation intro scene
                placeholder_panel("Transformation intro — coming soon", signals).into_any()
            }
            AppPhase::FemCreation => {
                // TODO Task 7: wire fem creation form
                placeholder_panel("Fem creation — coming soon", signals).into_any()
            }
            AppPhase::InGame => {
                // On first transition to InGame, start the opening scene.
                let gs_ref = Rc::clone(&game_state_ig);
                {
                    let mut gs_opt = gs_ref.borrow_mut();
                    if let Some(ref mut gs) = *gs_opt {
                        if gs.init_error.is_none() {
                            if let Ok(fem_id) = gs.registry.resolve_skill("FEMININITY") {
                                let GameState {
                                    ref mut engine,
                                    ref mut world,
                                    ref registry,
                                    ref scheduler,
                                    ref mut rng,
                                    ref opening_scene,
                                    ref default_slot,
                                    ..
                                } = *gs;
                                if let Some(scene_id) = opening_scene {
                                    engine.send(
                                        EngineCommand::StartScene(scene_id.clone()),
                                        world,
                                        registry,
                                    );
                                }
                                let events = engine.drain();
                                let finished = process_events(events, signals, world, fem_id);
                                if finished {
                                    if let Some(slot) = default_slot.as_deref() {
                                        if let Some(result) =
                                            scheduler.pick(slot, world, registry, rng)
                                        {
                                            engine.send(
                                                EngineCommand::StartScene(result.scene_id),
                                                world,
                                                registry,
                                            );
                                            let events = engine.drain();
                                            process_events(events, signals, world, fem_id);
                                        }
                                    }
                                }
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
                        AppTab::Game => h_stack((
                            sidebar_panel(signals),
                            story_panel(signals, Rc::clone(&gs_cell)),
                        ))
                        .style(|s| s.size_full())
                        .into_any(),
                        AppTab::Saves => saves_panel(signals, Rc::clone(&gs_cell)).into_any(),
                        AppTab::Settings => settings_view(signals).into_any(),
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
                    if !s.is_empty() {
                        s.push_str("\n\n");
                    }
                    s.push_str(&text);
                    // Cap at 200 paragraphs to prevent unbounded growth.
                    const MAX_PARAGRAPHS: usize = 200;
                    let para_count = s.split("\n\n").count();
                    if para_count > MAX_PARAGRAPHS {
                        let to_drop = para_count - MAX_PARAGRAPHS;
                        let mut remaining = to_drop;
                        let mut byte_offset = 0;
                        for (i, _) in s.match_indices("\n\n") {
                            remaining -= 1;
                            if remaining == 0 {
                                byte_offset = i + 2;
                                break;
                            }
                        }
                        if byte_offset > 0 {
                            *s = s[byte_offset..].to_string();
                        }
                    }
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
                signals.active_npc.set(data.as_ref().map(|d| NpcSnapshot {
                    name: d.name.clone(),
                    age: format!("{}", d.age),
                    personality: d.personality.clone(),
                    relationship: format!("{}", d.relationship),
                    pc_liking: format!("{}", d.pc_liking),
                    pc_attraction: format!("{}", d.pc_attraction),
                }));
            }
            EngineEvent::SceneFinished => {
                signals.actions.set(vec![]);
                scene_finished = true;
            }
            EngineEvent::ThoughtAdded { text, .. } => {
                // Thoughts append to the story text. Style differentiation (italic,
                // anxiety register, etc.) is deferred to the UI design session.
                signals.story.update(|s| {
                    if !s.is_empty() {
                        s.push_str("\n\n");
                    }
                    s.push_str(&text);
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
                    if !s.is_empty() {
                        s.push_str("\n\n");
                    }
                    s.push_str(&format!("[Scene error: {}]", msg));
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
    use std::collections::{HashMap, HashSet};
    use undone_domain::*;

    fn test_player() -> Player {
        Player {
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
            race: "white".into(),
            figure: PlayerFigure::Slim,
            breasts: BreastSize::MediumLarge,
            eye_colour: "blue".into(),
            hair_colour: "blonde".into(),
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
}
