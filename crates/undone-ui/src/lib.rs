pub mod game_state;
pub mod left_panel;
pub mod right_panel;
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

use crate::game_state::{init_game, GameState};
use crate::left_panel::story_panel;
use crate::right_panel::sidebar_panel;
use crate::theme::{ThemeColors, UserPrefs};
use crate::title_bar::title_bar;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    Game,
    Saves,
    Settings,
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

    let state = Rc::new(RefCell::new(init_game()));

    // Surface pack-load errors in the story panel.
    {
        let gs = state.borrow();
        if let Some(ref err) = gs.init_error {
            signals.story.set(err.clone());
        }
    }

    // Resolve FEMININITY skill id once — used to build PlayerSnapshot.
    // Only used in the non-error path (process_events is never called when init_error is set).
    let femininity_id: Option<SkillId> = {
        let gs = state.borrow();
        gs.registry.resolve_skill("FEMININITY").ok()
    };

    // Start opening scene on app launch (only when packs loaded successfully).
    if let Some(fem_id) = femininity_id {
        let mut gs = state.borrow_mut();
        if gs.init_error.is_none() {
            let GameState {
                ref mut engine,
                ref mut world,
                ref registry,
                ref scheduler,
                ref mut rng,
                ..
            } = *gs;
            engine.send(
                EngineCommand::StartScene("base::rain_shelter".into()),
                world,
                registry,
            );
            let events = engine.drain();
            let finished = process_events(events, signals, world, fem_id);
            if finished {
                if let Some(scene_id) = scheduler.pick("free_time", world, registry, rng) {
                    engine.send(EngineCommand::StartScene(scene_id), world, registry);
                    let events = engine.drain();
                    process_events(events, signals, world, fem_id);
                }
            }
        }
    }

    let content = dyn_container(
        move || signals.tab.get(),
        move |tab| match tab {
            AppTab::Game => h_stack((
                sidebar_panel(signals),
                story_panel(signals, Rc::clone(&state)),
            ))
            .style(|s| s.size_full())
            .into_any(),
            AppTab::Saves => placeholder_panel("Saves \u{2014} coming soon", signals).into_any(),
            AppTab::Settings => {
                placeholder_panel("Settings \u{2014} coming soon", signals).into_any()
            }
        },
    )
    .style(|s| s.flex_grow(1.0));

    let main_column = v_stack((title_bar(signals), content)).style(move |s| {
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
                signals.story.update(|s| {
                    if !s.is_empty() {
                        s.push_str("\n\n");
                    }
                    s.push_str(&text);
                });
            }
            EngineEvent::ActionsAvailable(actions) => {
                signals.actions.set(actions);
            }
            EngineEvent::NpcActivated(data) => {
                signals.active_npc.set(data.as_ref().map(|d| NpcSnapshot {
                    name: d.name.clone(),
                    age: format!("{}", d.age),
                    personality: format!("{:?}", d.personality),
                    relationship: format!("{}", d.relationship),
                    pc_liking: format!("{}", d.pc_liking),
                    pc_attraction: format!("{}", d.pc_attraction),
                }));
            }
            EngineEvent::SceneFinished => {
                signals.actions.set(vec![]);
                scene_finished = true;
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
            before_age: 30,
            before_race: "white".into(),
            before_sexuality: Sexuality::StraightMale,
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
            always_female: false,
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
