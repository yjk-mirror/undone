//! BeforeCreation "Next" button + FemCreation "Begin" button (validation + game start).
use floem::prelude::*;
use floem::reactive::RwSignal;
use std::cell::RefCell;
use std::rc::Rc;
use undone_domain::{
    Age, Appearance, BeforeIdentity, BeforeSexuality, BeforeVoice, BreastSize, ButtSize,
    ClitSensitivity, Complexion, EyeColour, HairColour, HairLength, Height, InnerLabiaSize,
    LipShape, MaleFigure, NaturalPubicHair, NippleSensitivity, PcOrigin, PenisSize, PlayerFigure,
    PubicHairStyle, SkinTone, WaistSize, WetnessBaseline,
};
use undone_packs::{char_creation::CharCreationConfig, PresetData};

use crate::game_state::{build_throwaway_game_state, start_game_checked, GameState, PreGameState};
use crate::theme::{ThemeColors, UI_FONT_FAMILY};
use crate::{AppPhase, AppSignals, PartialCharState};

use super::config::*;
use super::contracts::*;
use super::signals::*;

// ── Next button ───────────────────────────────────────────────────────────────

pub(crate) fn build_next_button(
    signals: AppSignals,
    form: BeforeFormSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
    partial_char: RwSignal<Option<PartialCharState>>,
) -> impl View {
    label(|| "Next \u{2192}".to_string())
        .keyboard_navigable()
        .on_click_stop(move |_| {
            let char_mode = form.char_mode.get_untracked();

            let origin: PcOrigin;
            let before_name: String;
            let before_age: Age;
            let before_race: String;
            let before_sexuality: BeforeSexuality;
            let trait_names: Vec<String>;

            // Clone the preset (if any) from the registry so it outlives the borrow.
            let preset_owned: Option<PresetData> = {
                let pre_borrow = pre_state.borrow();
                pre_borrow
                    .as_ref()
                    .and_then(|pre| pre.registry.presets().get(char_mode as usize).cloned())
            };
            // char_mode < number-of-presets means preset mode; otherwise custom.
            let num_presets = {
                let pre_borrow = pre_state.borrow();
                pre_borrow
                    .as_ref()
                    .map(|pre| pre.registry.presets().len())
                    .unwrap_or(0)
            };

            if (char_mode as usize) < num_presets {
                // Preset mode
                let preset = preset_owned.as_ref().unwrap();
                origin = preset.origin;
                before_name = preset.before_name.clone();
                before_age = preset.before_age;
                before_race = preset.before_race.clone();
                before_sexuality = preset.before_sexuality;
                trait_names = preset.trait_ids.iter().map(|s| s.to_string()).collect();
            } else {
                // Custom mode
                let origin_idx = form.origin_idx.get_untracked();
                if origin_idx != 3 && form.before_name.get_untracked().trim().is_empty() {
                    return;
                }
                origin = origin_from_idx(origin_idx);
                before_name = form.before_name.get_untracked();
                before_age = form.before_age.get_untracked();
                before_race = form.before_race.get_untracked();
                before_sexuality = form.before_sexuality.get_untracked();

                let mut tn: Vec<String> = Vec::new();
                if form.trait_shy.get_untracked() {
                    tn.push("SHY".into());
                }
                if form.trait_cute.get_untracked() {
                    tn.push("CUTE".into());
                }
                if form.trait_posh.get_untracked() {
                    tn.push("POSH".into());
                }
                if form.trait_sultry.get_untracked() {
                    tn.push("SULTRY".into());
                }
                if form.trait_down_to_earth.get_untracked() {
                    tn.push("DOWN_TO_EARTH".into());
                }
                if form.trait_bitchy.get_untracked() {
                    tn.push("BITCHY".into());
                }
                if form.trait_refined.get_untracked() {
                    tn.push("REFINED".into());
                }
                if form.trait_romantic.get_untracked() {
                    tn.push("ROMANTIC".into());
                }
                if form.trait_flirty.get_untracked() {
                    tn.push("FLIRTY".into());
                }
                if form.trait_ambitious.get_untracked() {
                    tn.push("AMBITIOUS".into());
                }
                if form.trait_outgoing.get_untracked() {
                    tn.push("OUTGOING".into());
                }
                if form.trait_overactive_imagination.get_untracked() {
                    tn.push("OVERACTIVE_IMAGINATION".into());
                }
                if form.trait_analytical.get_untracked() {
                    tn.push("ANALYTICAL".into());
                }
                if form.trait_confident.get_untracked() {
                    tn.push("CONFIDENT".into());
                }
                if form.trait_sexist.get_untracked() {
                    tn.push("SEXIST".into());
                }
                if form.trait_homophobic.get_untracked() {
                    tn.push("HOMOPHOBIC".into());
                }
                if form.trait_objectifying.get_untracked() {
                    tn.push("OBJECTIFYING".into());
                }
                trait_names = tn;
            }

            let trait_name_strs: Vec<&str> = trait_names.iter().map(|s| s.as_str()).collect();
            let starting_traits = {
                let pre_borrow = pre_state.borrow();
                if let Some(ref pre) = *pre_borrow {
                    match resolve_starting_traits(
                        &pre.registry,
                        &trait_name_strs,
                        form.include_rough.get_untracked(),
                        form.likes_rough.get_untracked(),
                    ) {
                        Ok(traits) => traits,
                        Err(message) => {
                            drop(pre_borrow);
                            surface_runtime_init_error(&pre_state, &game_state, signals, message);
                            return;
                        }
                    }
                } else {
                    vec![]
                }
            };

            // Presets declare their own starting game flags; custom players
            // start freeform with no preset routing flags.
            let starting_flags = preset_owned
                .as_ref()
                .map(|preset| preset.starting_flags.clone())
                .unwrap_or_default();

            let appearance = if let Some(ref p) = preset_owned {
                p.appearance
            } else {
                form.appearance.get_untracked()
            };
            let partial = PartialCharState {
                origin,
                before_name: before_name.clone(),
                before_age,
                before_race: before_race.clone(),
                before_sexuality,
                starting_traits,
                starting_flags,
                preset_idx: preset_owned.as_ref().map(|_| char_mode),
                appearance,
            };
            partial_char.set(Some(partial.clone()));

            if origin == PcOrigin::AlwaysFemale {
                // Skip transformation intro — go straight to fem creation.
                signals.phase.set(AppPhase::FemCreation);
            } else {
                // Create a throwaway world for the transformation intro scene.
                // This world is discarded after the intro — the real world is
                // created at FemCreation submit via new_game().
                let before_identity = if let Some(ref p) = preset_owned {
                    Some(BeforeIdentity {
                        name: partial.before_name.clone(),
                        age: partial.before_age,
                        race: partial.before_race.clone(),
                        sexuality: partial.before_sexuality,
                        figure: p.before_figure,
                        height: p.before_height,
                        hair_colour: p.before_hair_colour,
                        eye_colour: p.before_eye_colour,
                        skin_tone: p.before_skin_tone,
                        penis_size: p.before_penis_size,
                        voice: p.before_voice,
                        traits: std::collections::HashSet::new(),
                    })
                } else {
                    Some(BeforeIdentity {
                        name: partial.before_name.clone(),
                        age: partial.before_age,
                        race: partial.before_race.clone(),
                        sexuality: partial.before_sexuality,
                        figure: MaleFigure::Average,
                        height: Height::Average,
                        hair_colour: HairColour::DarkBrown,
                        eye_colour: EyeColour::Brown,
                        skin_tone: SkinTone::Medium,
                        penis_size: PenisSize::Average,
                        voice: BeforeVoice::Average,
                        traits: std::collections::HashSet::new(),
                    })
                };
                let throwaway_config = if let Some(ref p) = preset_owned {
                    CharCreationConfig {
                        name_fem: p.name_fem.clone(),
                        name_masc: p.name_masc.clone(),
                        age: p.age,
                        race: p.race.clone(),
                        figure: p.figure,
                        breasts: p.breasts,
                        origin,
                        before: before_identity,
                        starting_traits: partial.starting_traits.clone(),
                        male_count: 0,
                        female_count: 0,
                        starting_flags: partial.starting_flags.iter().cloned().collect(),
                        starting_arc_states: std::collections::HashMap::new(),
                        height: p.height,
                        butt: p.butt,
                        waist: p.waist,
                        lips: p.lips,
                        hair_colour: p.hair_colour,
                        hair_length: p.hair_length,
                        eye_colour: p.eye_colour,
                        skin_tone: p.skin_tone,
                        complexion: p.complexion,
                        appearance: p.appearance,
                        pubic_hair: p.pubic_hair,
                        natural_pubic_hair: p.natural_pubic_hair,
                        nipple_sensitivity: p.nipple_sensitivity,
                        clit_sensitivity: p.clit_sensitivity,
                        inner_labia: p.inner_labia,
                        wetness_baseline: p.wetness_baseline,
                    }
                } else {
                    CharCreationConfig {
                        name_fem: String::new(),
                        name_masc: partial.before_name.clone(),
                        age: partial.before_age,
                        race: partial.before_race.clone(),
                        figure: PlayerFigure::Slim,
                        breasts: BreastSize::Full,
                        origin,
                        before: before_identity,
                        starting_traits: partial.starting_traits.clone(),
                        male_count: 0,
                        female_count: 0,
                        starting_flags: partial.starting_flags.iter().cloned().collect(),
                        starting_arc_states: std::collections::HashMap::new(),
                        height: Height::Average,
                        butt: ButtSize::Round,
                        waist: WaistSize::Average,
                        lips: LipShape::Average,
                        hair_colour: HairColour::DarkBrown,
                        hair_length: HairLength::Shoulder,
                        eye_colour: EyeColour::Brown,
                        skin_tone: SkinTone::Medium,
                        complexion: Complexion::Normal,
                        appearance: Appearance::Average,
                        pubic_hair: PubicHairStyle::Trimmed,
                        natural_pubic_hair: NaturalPubicHair::Full,
                        nipple_sensitivity: NippleSensitivity::Normal,
                        clit_sensitivity: ClitSensitivity::Normal,
                        inner_labia: InnerLabiaSize::Average,
                        wetness_baseline: WetnessBaseline::Normal,
                    }
                };

                {
                    let mut pre_mut = pre_state.borrow_mut();
                    if let Some(ref mut pre) = *pre_mut {
                        match build_throwaway_game_state(pre, throwaway_config, false) {
                            Ok(throwaway_gs) => {
                                *game_state.borrow_mut() = Some(throwaway_gs);
                            }
                            Err(message) => {
                                drop(pre_mut);
                                surface_runtime_init_error(
                                    &pre_state,
                                    &game_state,
                                    signals,
                                    message,
                                );
                                return;
                            }
                        }
                    }
                }

                signals.phase.set(AppPhase::TransformationIntro);
            }
        })
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.margin_top(24.0)
                .padding_horiz(40.0)
                .padding_vert(16.0)
                .font_size(16.0)
                .font_family(UI_FONT_FAMILY.to_string())
                .border(1.5)
                .border_color(colors.lamp)
                .border_radius(6.0)
                .color(colors.lamp)
                .background(colors.lamp_glow)
                .hover(|s| s.background(colors.lamp_glow))
                .focus_visible(|s| s.outline(2.0).outline_color(colors.lamp))
                .active(|s| s.background(colors.lamp_glow))
        })
}

// ── Begin Your Story button (FemCreation) ─────────────────────────────────────

pub(crate) fn build_begin_button(
    signals: AppSignals,
    form: FemFormSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
    partial_char: RwSignal<Option<PartialCharState>>,
    dev_mode: bool,
) -> impl View {
    label(|| "Begin Your Story".to_string())
        .keyboard_navigable()
        .on_click_stop(move |_| {
            let partial = partial_char.get_untracked().unwrap_or_else(|| {
                // AlwaysFemale path: partial may not carry before-data.
                PartialCharState {
                    origin: PcOrigin::AlwaysFemale,
                    before_name: String::new(),
                    before_age: form.age.get_untracked(),
                    before_race: form.race.get_untracked(),
                    before_sexuality: BeforeSexuality::AttractedToWomen,
                    starting_traits: vec![],
                    starting_flags: vec![],
                    preset_idx: None,
                    appearance: Appearance::Average,
                }
            });

            let origin = partial.origin;

            // Clone the preset (if any) from the registry so it outlives the borrow.
            let preset_owned: Option<PresetData> = {
                let pre_borrow = pre_state.borrow();
                pre_borrow
                    .as_ref()
                    .and_then(|pre| preset_by_idx(&pre.registry, partial.preset_idx).cloned())
            };

            let config = if let Some(ref p) = preset_owned {
                // Preset mode: all physical/sexual attributes come from PresetData
                let before = if origin.was_transformed() {
                    Some(BeforeIdentity {
                        name: partial.before_name.clone(),
                        age: partial.before_age,
                        race: partial.before_race.clone(),
                        sexuality: partial.before_sexuality,
                        figure: p.before_figure,
                        height: p.before_height,
                        hair_colour: p.before_hair_colour,
                        eye_colour: p.before_eye_colour,
                        skin_tone: p.before_skin_tone,
                        penis_size: p.before_penis_size,
                        voice: p.before_voice,
                        traits: std::collections::HashSet::new(),
                    })
                } else {
                    None
                };
                CharCreationConfig {
                    name_fem: p.name_fem.clone(),
                    name_masc: p.name_masc.clone(),
                    age: p.age,
                    race: p.race.clone(),
                    figure: p.figure,
                    breasts: p.breasts,
                    origin,
                    before,
                    starting_traits: partial.starting_traits,
                    male_count: 6,
                    female_count: 2,
                    starting_flags: partial.starting_flags.into_iter().collect(),
                    starting_arc_states: std::collections::HashMap::new(),
                    height: p.height,
                    butt: p.butt,
                    waist: p.waist,
                    lips: p.lips,
                    hair_colour: p.hair_colour,
                    hair_length: p.hair_length,
                    eye_colour: p.eye_colour,
                    skin_tone: p.skin_tone,
                    complexion: p.complexion,
                    appearance: p.appearance,
                    pubic_hair: p.pubic_hair,
                    natural_pubic_hair: p.natural_pubic_hair,
                    nipple_sensitivity: p.nipple_sensitivity,
                    clit_sensitivity: p.clit_sensitivity,
                    inner_labia: p.inner_labia,
                    wetness_baseline: p.wetness_baseline,
                }
            } else {
                // Custom mode: form signals + defaults for unexposed fields
                let fem_race = form.race.get_untracked();
                let before = if origin.was_transformed() {
                    Some(BeforeIdentity {
                        name: partial.before_name.clone(),
                        age: partial.before_age,
                        race: partial.before_race.clone(),
                        sexuality: partial.before_sexuality,
                        figure: MaleFigure::Average,
                        height: Height::Average,
                        hair_colour: HairColour::DarkBrown,
                        eye_colour: EyeColour::Brown,
                        skin_tone: SkinTone::Medium,
                        penis_size: PenisSize::Average,
                        voice: BeforeVoice::Average,
                        traits: std::collections::HashSet::new(),
                    })
                } else {
                    None
                };
                let pc_age = if origin == PcOrigin::AlwaysFemale {
                    form.age.get_untracked()
                } else {
                    partial.before_age
                };
                CharCreationConfig {
                    name_fem: form.name_fem.get_untracked(),
                    name_masc: partial.before_name.clone(),
                    age: pc_age,
                    race: fem_race,
                    figure: form.figure.get_untracked(),
                    breasts: form.breasts.get_untracked(),
                    origin,
                    before,
                    starting_traits: partial.starting_traits,
                    male_count: 6,
                    female_count: 2,
                    starting_flags: partial.starting_flags.into_iter().collect(),
                    starting_arc_states: std::collections::HashMap::new(),
                    height: Height::Average,
                    butt: ButtSize::Round,
                    waist: WaistSize::Average,
                    lips: LipShape::Average,
                    hair_colour: HairColour::DarkBrown,
                    hair_length: HairLength::Shoulder,
                    eye_colour: EyeColour::Brown,
                    skin_tone: SkinTone::Medium,
                    complexion: Complexion::Normal,
                    appearance: partial.appearance,
                    pubic_hair: PubicHairStyle::Trimmed,
                    natural_pubic_hair: NaturalPubicHair::Full,
                    nipple_sensitivity: NippleSensitivity::Normal,
                    clit_sensitivity: ClitSensitivity::Normal,
                    inner_labia: InnerLabiaSize::Average,
                    wetness_baseline: WetnessBaseline::Normal,
                }
            };

            if let Some(ref pre) = *pre_state.borrow() {
                let startup_errors = validate_startup_contract(&pre.registry, config.origin);
                if !startup_errors.is_empty() {
                    surface_runtime_init_error(
                        &pre_state,
                        &game_state,
                        signals,
                        format!(
                            "Character creation contract error(s):\n{}",
                            startup_errors.join("\n")
                        ),
                    );
                    return;
                }
            }

            let pre = match pre_state.borrow_mut().take() {
                Some(p) => p,
                None => return, // already started
            };

            match start_game_checked(pre, config, dev_mode) {
                Ok(gs) => {
                    *game_state.borrow_mut() = Some(gs);
                    signals.tab.set(crate::AppTab::Game);
                    // Defer the InGame phase transition to the next frame.
                    // Setting it synchronously inside on_click_stop causes a
                    // floem reactive panic: the dyn_container rebuild enters
                    // the InGame branch whose style closures call .get() on
                    // signals, creating nested scopes inside the consumed
                    // click-handler context.
                    floem::action::exec_after(std::time::Duration::ZERO, move |_| {
                        signals.phase.set(AppPhase::InGame);
                    });
                }
                Err(message) => {
                    surface_runtime_init_error(&pre_state, &game_state, signals, message);
                }
            }
        })
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.margin_top(24.0)
                .padding_horiz(40.0)
                .padding_vert(16.0)
                .font_size(16.0)
                .font_family(UI_FONT_FAMILY.to_string())
                .border(1.5)
                .border_color(colors.lamp)
                .border_radius(6.0)
                .color(colors.lamp)
                .background(colors.lamp_glow)
                .hover(|s| s.background(colors.lamp_glow))
                .focus_visible(|s| s.outline(2.0).outline_color(colors.lamp))
                .active(|s| s.background(colors.lamp_glow))
        })
}
