//! Reactive form-signal state + pre-game-state IO helpers for character creation.
use floem::prelude::*;
use floem::reactive::RwSignal;
use rand::SeedableRng;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use undone_domain::{Age, Appearance, BeforeSexuality, BreastSize, PlayerFigure};
use undone_packs::PackRegistry;
use undone_scene::scheduler::Scheduler;

use crate::game_state::{GameState, PreGameState};
use crate::{AppPhase, AppSignals};

use super::config::FemFormDefaults;

/// Read the race list from the pack registry, falling back to `["White"]` if empty.
pub(crate) fn read_races(pre_state: &Rc<RefCell<Option<PreGameState>>>) -> Vec<String> {
    if let Some(ref pre) = *pre_state.borrow() {
        if !pre.registry.races().is_empty() {
            return pre.registry.races().to_vec();
        }
    }
    vec!["White".to_string()]
}

/// Read the male names list from the pack registry, falling back to a minimal set.
pub(crate) fn read_male_names(pre_state: &Rc<RefCell<Option<PreGameState>>>) -> Vec<String> {
    if let Some(ref pre) = *pre_state.borrow() {
        if !pre.registry.male_names().is_empty() {
            return pre.registry.male_names().to_vec();
        }
    }
    vec!["Matt".to_string(), "Ryan".to_string(), "David".to_string()]
}

pub(crate) fn store_runtime_init_error(
    pre_state: &Rc<RefCell<Option<PreGameState>>>,
    message: String,
) {
    let mut pre_mut = pre_state.borrow_mut();
    if let Some(ref mut pre) = *pre_mut {
        pre.init_error = Some(message);
        return;
    }

    *pre_mut = Some(PreGameState {
        registry: PackRegistry::new(),
        scenes: HashMap::new(),
        scheduler: Scheduler::empty(),
        rng: rand::rngs::SmallRng::from_entropy(),
        init_error: Some(message),
    });
}

pub(crate) fn surface_runtime_init_error(
    pre_state: &Rc<RefCell<Option<PreGameState>>>,
    game_state: &Rc<RefCell<Option<GameState>>>,
    signals: AppSignals,
    message: String,
) {
    store_runtime_init_error(pre_state, message);
    *game_state.borrow_mut() = None;
    signals.tab.set(crate::AppTab::Game);
    // Defer phase transition — see build_begin_button comment.
    floem::action::exec_after(std::time::Duration::ZERO, move |_| {
        signals.phase.set(AppPhase::InGame);
    });
}

// ── BeforeCreation form signals ───────────────────────────────────────────────

#[derive(Clone, Copy)]
pub(crate) struct BeforeFormSignals {
    pub(crate) origin_idx: RwSignal<u8>,
    pub(crate) before_name: RwSignal<String>,
    pub(crate) before_age: RwSignal<Age>,
    pub(crate) before_sexuality: RwSignal<BeforeSexuality>,
    pub(crate) before_race: RwSignal<String>,
    // personality
    pub(crate) trait_shy: RwSignal<bool>,
    pub(crate) trait_cute: RwSignal<bool>,
    pub(crate) trait_posh: RwSignal<bool>,
    pub(crate) trait_sultry: RwSignal<bool>,
    pub(crate) trait_down_to_earth: RwSignal<bool>,
    pub(crate) trait_bitchy: RwSignal<bool>,
    pub(crate) trait_refined: RwSignal<bool>,
    pub(crate) trait_romantic: RwSignal<bool>,
    pub(crate) trait_flirty: RwSignal<bool>,
    pub(crate) trait_ambitious: RwSignal<bool>,
    pub(crate) trait_outgoing: RwSignal<bool>,
    pub(crate) trait_overactive_imagination: RwSignal<bool>,
    pub(crate) trait_analytical: RwSignal<bool>,
    pub(crate) trait_confident: RwSignal<bool>,
    // attitude traits
    pub(crate) trait_sexist: RwSignal<bool>,
    pub(crate) trait_homophobic: RwSignal<bool>,
    pub(crate) trait_objectifying: RwSignal<bool>,
    pub(crate) appearance: RwSignal<Appearance>,
    // content prefs
    pub(crate) include_rough: RwSignal<bool>,
    pub(crate) likes_rough: RwSignal<bool>,
    // mode: 0=Robin preset, 1=Raul preset, 2=Custom
    pub(crate) char_mode: RwSignal<u8>,
}

impl BeforeFormSignals {
    pub(crate) fn new() -> Self {
        Self {
            origin_idx: RwSignal::new(0),
            before_name: RwSignal::new(String::new()),
            before_age: RwSignal::new(Age::EarlyTwenties),
            before_sexuality: RwSignal::new(BeforeSexuality::AttractedToWomen),
            before_race: RwSignal::new(String::new()),
            trait_shy: RwSignal::new(false),
            trait_cute: RwSignal::new(false),
            trait_posh: RwSignal::new(false),
            trait_sultry: RwSignal::new(false),
            trait_down_to_earth: RwSignal::new(false),
            trait_bitchy: RwSignal::new(false),
            trait_refined: RwSignal::new(false),
            trait_romantic: RwSignal::new(false),
            trait_flirty: RwSignal::new(false),
            trait_ambitious: RwSignal::new(false),
            trait_outgoing: RwSignal::new(false),
            trait_overactive_imagination: RwSignal::new(false),
            trait_analytical: RwSignal::new(false),
            trait_confident: RwSignal::new(false),
            trait_sexist: RwSignal::new(false),
            trait_homophobic: RwSignal::new(false),
            trait_objectifying: RwSignal::new(false),
            appearance: RwSignal::new(Appearance::Average),
            include_rough: RwSignal::new(false),
            likes_rough: RwSignal::new(false),
            char_mode: RwSignal::new(0u8),
        }
    }
}

// ── FemCreation form signals ──────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub(crate) struct FemFormSignals {
    pub(crate) name_fem: RwSignal<String>,
    pub(crate) age: RwSignal<Age>,
    pub(crate) figure: RwSignal<PlayerFigure>,
    pub(crate) breasts: RwSignal<BreastSize>,
    pub(crate) race: RwSignal<String>,
}

impl FemFormSignals {
    pub(crate) fn from_defaults(defaults: &FemFormDefaults) -> Self {
        Self {
            name_fem: RwSignal::new(defaults.name_fem.clone()),
            age: RwSignal::new(defaults.age),
            figure: RwSignal::new(defaults.figure),
            breasts: RwSignal::new(defaults.breasts),
            race: RwSignal::new(defaults.race.clone()),
        }
    }
}
