pub mod game_state;
pub mod left_panel;
pub mod right_panel;
pub mod theme;

use floem::prelude::*;
use floem::reactive::RwSignal;
use std::cell::RefCell;
use std::rc::Rc;
use undone_scene::engine::{ActionView, EngineCommand, EngineEvent};
use undone_world::World;

use crate::game_state::{init_game, GameState};
use crate::left_panel::left_panel;
use crate::right_panel::right_panel;
use crate::theme::{ThemeColors, UserPrefs};

/// All reactive signals used by the view tree.
#[derive(Clone, Copy)]
pub struct AppSignals {
    pub story: RwSignal<String>,
    pub actions: RwSignal<Vec<ActionView>>,
    pub player: RwSignal<PlayerSnapshot>,
    pub active_npc: RwSignal<Option<NpcSnapshot>>,
    pub prefs: RwSignal<UserPrefs>,
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
            prefs: RwSignal::new(UserPrefs::default()),
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

impl From<&undone_domain::Player> for PlayerSnapshot {
    fn from(p: &undone_domain::Player) -> Self {
        Self {
            name: p.active_name().to_owned(),
            femininity: p.femininity,
            money: p.money,
            stress: p.stress,
            anxiety: p.anxiety,
            arousal: format!("{:?}", p.arousal),
            alcohol: format!("{:?}", p.alcohol),
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
            age: format!("{:?}", npc.age),
            personality: format!("{:?}", npc.personality),
            relationship: format!("{:?}", npc.relationship),
            pc_liking: format!("{:?}", npc.pc_liking),
            pc_attraction: format!("{:?}", npc.pc_attraction),
        }
    }
}

pub fn app_view() -> impl View {
    let signals = AppSignals::new();

    let state = Rc::new(RefCell::new(init_game()));

    // Start opening scene on app launch
    {
        let mut gs = state.borrow_mut();
        let GameState {
            ref mut engine,
            ref mut world,
            ref registry,
        } = *gs;
        engine.send(
            EngineCommand::StartScene("base::rain_shelter".into()),
            world,
            registry,
        );
        let events = engine.drain();
        process_events(events, signals, world);
    }

    h_stack((right_panel(signals), left_panel(signals, Rc::clone(&state)))).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.size_full().background(colors.ground)
    })
}

pub fn process_events(events: Vec<EngineEvent>, signals: AppSignals, world: &World) {
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
            EngineEvent::SceneFinished => {
                signals.actions.set(vec![]);
                // Scheduler integration: future session will pick next scene
            }
        }
    }
    signals.player.set(PlayerSnapshot::from(&world.player));
}

#[cfg(test)]
mod tests {
    use super::*;
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
            femininity: 25,
        }
    }

    #[test]
    fn player_snapshot_name_uses_active_name() {
        let p = test_player();
        let snap = PlayerSnapshot::from(&p);
        assert_eq!(snap.name, "Evan"); // femininity=25 → masc
    }

    #[test]
    fn player_snapshot_captures_money() {
        let p = test_player();
        let snap = PlayerSnapshot::from(&p);
        assert_eq!(snap.money, 200);
    }
}
