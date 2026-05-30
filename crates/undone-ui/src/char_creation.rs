use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::dropdown::Dropdown;
use rand::SeedableRng;
use std::cell::RefCell;
use std::rc::Rc;
use undone_domain::{Age, BreastSize, PcOrigin, PlayerFigure};
use undone_packs::{PackRegistry, PresetData};

use crate::game_state::{GameState, PreGameState};
use crate::theme::ThemeColors;
use crate::{AppSignals, PartialCharState};

mod buttons;
mod config;
mod contracts;
mod sections;
mod signals;
mod widgets;
use buttons::*;
use config::*;
pub use config::{resolve_starting_traits, robin_quick_config};
pub use contracts::*;
use sections::*;
use signals::*;
use widgets::*;

// ── public entry points ───────────────────────────────────────────────────────

/// BeforeCreation phase: who you were before the transformation.
pub fn char_creation_view(
    signals: AppSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
    partial_char: RwSignal<Option<PartialCharState>>,
) -> impl View {
    let form = BeforeFormSignals::new();
    let char_mode = form.char_mode;

    let races_list = read_races(&pre_state);
    if let Some(first) = races_list.first() {
        form.before_race.set(first.clone());
    }
    let male_names = read_male_names(&pre_state);

    // Clone presets from registry so closures can own them.
    let presets: Vec<PresetData> = {
        let pre_borrow = pre_state.borrow();
        pre_borrow
            .as_ref()
            .map(|pre| pre.registry.presets().to_vec())
            .unwrap_or_default()
    };

    let next_btn = build_next_button(signals, form, pre_state, game_state, partial_char);

    let races_for_dyn = races_list;
    let names_for_dyn = male_names;
    let main_section = dyn_container(
        move || char_mode.get(),
        move |mode| {
            if mode == 2 {
                v_stack((
                    section_your_past(signals, form, races_for_dyn.clone(), names_for_dyn.clone()),
                    section_personality(signals, form),
                    section_content_prefs(signals, form),
                ))
                .into_any()
            } else if let Some(preset) = presets.get(mode as usize) {
                section_preset_detail(signals, preset).into_any()
            } else {
                empty().into_any()
            }
        },
    );

    let content = v_stack((
        heading("Your Story Begins", signals),
        section_preset_select(signals, char_mode),
        main_section,
        next_btn,
        empty().style(|s| s.height(40.0)),
    ))
    .style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.width_full()
            .max_width(640.0)
            .padding_horiz(40.0)
            .padding_vert(32.0)
            .color(colors.ink)
    });

    let centered = container(content).style(|s| s.width_full().flex_row().justify_center());

    scroll(centered)
        .scroll_style(|s| s.shrink_to_fit())
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.size_full().background(colors.page)
        })
}

/// FemCreation phase: who you are now.
///
/// For presets with `discovery_beats`, renders an interactive step-driven flow:
/// each beat shows prose, optionally reveals attribute groups, and optionally
/// presents reaction choices that set game flags. For presets without beats
/// (or custom characters), falls back to the flat form layout.
pub fn fem_creation_view(
    signals: AppSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
    partial_char: RwSignal<Option<PartialCharState>>,
    dev_mode: bool,
) -> impl View {
    let races_list = read_races(&pre_state);
    let partial = partial_char.get_untracked();
    // Clone the preset (if any) so it outlives the borrow.
    let preset_owned: Option<PresetData> = {
        let pre_borrow = pre_state.borrow();
        pre_borrow.as_ref().and_then(|pre| {
            partial
                .as_ref()
                .and_then(|partial| preset_by_idx(&pre.registry, partial.preset_idx))
                .cloned()
        })
    };

    let has_discovery = preset_owned
        .as_ref()
        .map(|p| !p.discovery_beats.is_empty())
        .unwrap_or(false);

    if has_discovery {
        fem_creation_discovery_view(
            signals,
            pre_state,
            game_state,
            partial_char,
            dev_mode,
            preset_owned.unwrap(),
        )
        .into_any()
    } else {
        fem_creation_flat_view(
            signals,
            pre_state,
            game_state,
            partial_char,
            dev_mode,
            preset_owned,
            races_list,
        )
        .into_any()
    }
}

/// Flat FemCreation layout (custom characters or presets without discovery beats).
fn fem_creation_flat_view(
    signals: AppSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
    partial_char: RwSignal<Option<PartialCharState>>,
    dev_mode: bool,
    preset_owned: Option<PresetData>,
    races_list: Vec<String>,
) -> impl View {
    let partial = partial_char.get_untracked();
    let preset_ref = preset_owned.as_ref();
    let defaults = {
        let pre_borrow = pre_state.borrow();
        let registry = pre_borrow.as_ref().map(|pre| &pre.registry);
        if let Some(reg) = registry {
            fem_form_defaults(
                reg,
                partial.as_ref(),
                races_list.first().map(|race| race.as_str()),
            )
        } else {
            fem_form_defaults(
                &PackRegistry::new(),
                partial.as_ref(),
                races_list.first().map(|race| race.as_str()),
            )
        }
    };
    let form = FemFormSignals::from_defaults(&defaults);

    let is_always_female = partial
        .as_ref()
        .map(|p| p.origin == PcOrigin::AlwaysFemale)
        .unwrap_or(false);

    let begin_btn =
        build_begin_button(signals, form, pre_state, game_state, partial_char, dev_mode);

    let age_row: Box<dyn View> = if let Some(preset) = preset_ref {
        Box::new(read_only_row("Age", preset.age.to_string(), signals))
    } else if is_always_female {
        Box::new(form_row(
            "Age",
            signals,
            Dropdown::new_rw(
                form.age,
                vec![
                    Age::LateTeen,
                    Age::EarlyTwenties,
                    Age::MidLateTwenties,
                    Age::LateTwenties,
                    Age::Thirties,
                    Age::Forties,
                    Age::Fifties,
                    Age::Old,
                ],
            )
            .main_view(themed_trigger::<Age>(signals))
            .list_item_view(themed_item::<Age>(signals))
            .style(field_style(signals)),
        ))
    } else {
        Box::new(empty())
    };

    let names_section: Box<dyn View> = if let Some(preset) = preset_ref {
        Box::new(
            v_stack((
                section_title("Your Name", signals),
                read_only_row("Name", preset.name_fem.to_string(), signals),
            ))
            .style(section_style()),
        )
    } else {
        Box::new(
            v_stack((
                section_title("Your Name", signals),
                form_row(
                    "Name",
                    signals,
                    text_input(form.name_fem)
                        .placeholder("e.g. Eva")
                        .style(field_style(signals)),
                ),
            ))
            .style(section_style()),
        )
    };

    let body_section: Box<dyn View> = if let Some(preset) = preset_ref {
        let physical_traits: Vec<String> = preset
            .trait_ids
            .iter()
            .filter(|id| {
                let s = id.as_str();
                !PERSONALITY_TRAIT_IDS.contains(&s) && BODY_APPEARANCE_TRAIT_IDS.contains(&s)
            })
            .map(|id| trait_id_to_display(id))
            .collect();
        let sexual_traits: Vec<String> = preset
            .trait_ids
            .iter()
            .filter(|id| {
                let s = id.as_str();
                !PERSONALITY_TRAIT_IDS.contains(&s) && !BODY_APPEARANCE_TRAIT_IDS.contains(&s)
            })
            .map(|id| trait_id_to_display(id))
            .collect();
        let physical_row: Box<dyn View> = if physical_traits.is_empty() {
            Box::new(empty())
        } else {
            Box::new(trait_chips("Physical", physical_traits, signals))
        };
        let sexual_row: Box<dyn View> = if sexual_traits.is_empty() {
            Box::new(empty())
        } else {
            Box::new(trait_chips("Sexual", sexual_traits, signals))
        };
        Box::new(
            v_stack((
                section_title("Your Body", signals),
                read_only_row("Figure", preset.figure.to_string(), signals),
                read_only_row("Breasts", preset.breasts.to_string(), signals),
                physical_row,
                sexual_row,
            ))
            .style(section_style()),
        )
    } else {
        Box::new(
            v_stack((
                section_title("Your Body", signals),
                form_row(
                    "Figure",
                    signals,
                    Dropdown::new_rw(
                        form.figure,
                        vec![
                            PlayerFigure::Petite,
                            PlayerFigure::Slim,
                            PlayerFigure::Athletic,
                            PlayerFigure::Hourglass,
                            PlayerFigure::Curvy,
                            PlayerFigure::Thick,
                            PlayerFigure::Plus,
                        ],
                    )
                    .main_view(themed_trigger::<PlayerFigure>(signals))
                    .list_item_view(themed_item::<PlayerFigure>(signals))
                    .style(field_style(signals)),
                ),
                form_row(
                    "Breasts",
                    signals,
                    Dropdown::new_rw(
                        form.breasts,
                        vec![
                            BreastSize::Flat,
                            BreastSize::Perky,
                            BreastSize::Handful,
                            BreastSize::Average,
                            BreastSize::Full,
                            BreastSize::Big,
                            BreastSize::Huge,
                        ],
                    )
                    .main_view(themed_trigger::<BreastSize>(signals))
                    .list_item_view(themed_item::<BreastSize>(signals))
                    .style(field_style(signals)),
                ),
            ))
            .style(section_style()),
        )
    };

    let background_section: Box<dyn View> = if let Some(preset) = preset_ref {
        Box::new(
            v_stack((
                section_title("Background", signals),
                read_only_row("Race", preset.race.to_string(), signals),
                age_row,
            ))
            .style(section_style()),
        )
    } else {
        Box::new(
            v_stack((
                section_title("Background", signals),
                form_row("Race", signals, race_picker(form.race, races_list, signals)),
                age_row,
            ))
            .style(section_style()),
        )
    };

    let bridge_copy = fem_creation_bridge_copy(partial.as_ref());
    let framing_prose = label(move || bridge_copy.clone()).style(move |s| {
        let prefs = signals.prefs.get();
        let colors = ThemeColors::from_mode(prefs.mode);
        s.width_full()
            .padding_vert(16.0)
            .padding_horiz(4.0)
            .color(colors.ink_dim)
            .font_size(prefs.font_size as f32 * 0.95)
            .line_height(1.6)
    });

    let content = v_stack((
        heading("Who Are You Now?", signals),
        framing_prose,
        names_section,
        body_section,
        background_section,
        begin_btn,
        empty().style(|s| s.height(40.0)),
    ))
    .style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.width_full()
            .max_width(640.0)
            .padding_horiz(40.0)
            .padding_vert(32.0)
            .color(colors.ink)
    });

    let centered = container(content).style(|s| s.width_full().flex_row().justify_center());

    scroll(centered)
        .scroll_style(|s| s.shrink_to_fit())
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.size_full().background(colors.page)
        })
}

/// Discovery-beat FemCreation layout (presets with interactive beats).
///
/// Renders one beat at a time. Each beat shows prose, reveals attribute groups
/// as read-only chips, and optionally presents reaction choice buttons. The
/// player advances through beats via choices or a Continue button.
fn fem_creation_discovery_view(
    signals: AppSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
    partial_char: RwSignal<Option<PartialCharState>>,
    dev_mode: bool,
    preset: PresetData,
) -> impl View {
    let partial = partial_char.get_untracked();
    let defaults = {
        let pre_borrow = pre_state.borrow();
        let registry = pre_borrow.as_ref().map(|pre| &pre.registry);
        if let Some(reg) = registry {
            fem_form_defaults(reg, partial.as_ref(), None)
        } else {
            fem_form_defaults(&PackRegistry::new(), partial.as_ref(), None)
        }
    };
    let form = FemFormSignals::from_defaults(&defaults);

    // Current beat index — drives the dyn_container.
    let beat_idx = RwSignal::new(0usize);
    let beat_count = preset.discovery_beats.len();

    let preset_rc = Rc::new(preset);
    let preset_for_dyn = preset_rc.clone();

    let begin_btn = build_begin_button(
        signals,
        form,
        pre_state.clone(),
        game_state.clone(),
        partial_char,
        dev_mode,
    );

    // Build a throwaway world from the full preset so we can render
    // discovery prose through minijinja with trait/skill branches.
    let rendered_beats: Vec<String> = {
        let pre_borrow = pre_state.borrow();
        if let Some(pre) = pre_borrow.as_ref() {
            if let Some(idx) = partial.as_ref().and_then(|p| p.preset_idx) {
                let cfg = config_from_preset(&pre.registry, idx as usize);
                let mut reg_clone = pre.registry.clone();
                let mut rng = rand::rngs::SmallRng::from_entropy();
                let world = undone_packs::new_game(cfg, &mut reg_clone, &mut rng);
                let empty_ctx = undone_scene::SceneCtx::new();
                preset_rc
                    .discovery_beats
                    .iter()
                    .map(|beat| {
                        undone_scene::template_ctx::render_prose(
                            &beat.prose,
                            &world,
                            &empty_ctx,
                            &reg_clone,
                        )
                        .unwrap_or_else(|_| beat.prose.clone())
                    })
                    .collect()
            } else {
                preset_rc
                    .discovery_beats
                    .iter()
                    .map(|beat| beat.prose.clone())
                    .collect()
            }
        } else {
            preset_rc
                .discovery_beats
                .iter()
                .map(|beat| beat.prose.clone())
                .collect()
        }
    };
    let rendered_beats = Rc::new(rendered_beats);
    let rendered_for_dyn = rendered_beats.clone();

    let beat_view = dyn_container(
        move || beat_idx.get(),
        move |idx| {
            if idx >= beat_count {
                return empty().into_any();
            }
            let prose_text = rendered_for_dyn[idx].clone();

            // ── Prose (rendered through minijinja with trait branches) ─
            let prose = label(move || prose_text.clone()).style(move |s| {
                let prefs = signals.prefs.get();
                let colors = ThemeColors::from_mode(prefs.mode);
                s.width_full()
                    .padding_vert(16.0)
                    .padding_horiz(4.0)
                    .color(colors.ink_dim)
                    .font_size(prefs.font_size as f32 * 0.95)
                    .line_height(1.6)
            });

            // ── Reveals ──────────────────────────────────────────────
            let reveals = build_discovery_reveals(
                signals,
                &preset_for_dyn,
                &preset_for_dyn.discovery_beats[idx].reveals,
                form,
            );

            // ── Continue button ──────────────────────────────────────
            let continue_btn = label(|| "Continue".to_string())
                .keyboard_navigable()
                .on_click_stop(move |_| {
                    beat_idx.set(idx + 1);
                })
                .style(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    s.padding_vert(10.0)
                        .padding_horiz(24.0)
                        .border(1.0)
                        .border_radius(6.0)
                        .border_color(colors.ink_dim.multiply_alpha(0.3))
                        .color(colors.ink)
                        .margin_top(16.0)
                        .cursor(floem::style::CursorStyle::Pointer)
                        .hover(|s| s.background(colors.ink_dim.multiply_alpha(0.08)))
                });

            v_stack((prose, reveals, continue_btn))
                .style(|s| s.width_full().margin_bottom(16.0))
                .into_any()
        },
    )
    .style(|s| s.size_full());

    // The begin_btn (from build_begin_button) handles full game creation.
    // Only show it after all beats have been advanced through.
    let begin_visible = begin_btn.style(move |s| {
        if beat_idx.get() < beat_count {
            s.display(floem::style::Display::None)
        } else {
            s
        }
    });

    let content =
        v_stack((beat_view, begin_visible, empty().style(|s| s.height(40.0)))).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.width_full()
                .max_width(640.0)
                .padding_horiz(40.0)
                .padding_vert(32.0)
                .color(colors.ink)
        });

    let centered = container(content).style(|s| s.width_full().flex_row().justify_center());

    scroll(centered)
        .scroll_style(|s| s.shrink_to_fit())
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.size_full().background(colors.page)
        })
}

/// Build reveal views for a discovery beat based on which attribute groups are listed.
fn build_discovery_reveals(
    signals: AppSignals,
    preset: &PresetData,
    reveals: &[undone_packs::RevealGroup],
    form: FemFormSignals,
) -> Box<dyn View> {
    use undone_packs::RevealGroup;

    let mut rows: Vec<Box<dyn View>> = Vec::new();

    for group in reveals {
        match group {
            RevealGroup::Scale => {
                rows.push(Box::new(read_only_row(
                    "Figure",
                    preset.figure.to_string(),
                    signals,
                )));
                rows.push(Box::new(read_only_row(
                    "Height",
                    preset.height.to_string(),
                    signals,
                )));
            }
            RevealGroup::Body => {
                rows.push(Box::new(read_only_row(
                    "Breasts",
                    preset.breasts.to_string(),
                    signals,
                )));
                rows.push(Box::new(read_only_row(
                    "Butt",
                    preset.butt.to_string(),
                    signals,
                )));
                rows.push(Box::new(read_only_row(
                    "Waist",
                    preset.waist.to_string(),
                    signals,
                )));
                rows.push(Box::new(read_only_row(
                    "Lips",
                    preset.lips.to_string(),
                    signals,
                )));
            }
            RevealGroup::Face => {
                rows.push(Box::new(read_only_row(
                    "Hair",
                    format!("{} {}", preset.hair_colour, preset.hair_length),
                    signals,
                )));
                rows.push(Box::new(read_only_row(
                    "Eyes",
                    preset.eye_colour.to_string(),
                    signals,
                )));
                rows.push(Box::new(read_only_row(
                    "Skin",
                    preset.skin_tone.to_string(),
                    signals,
                )));
                rows.push(Box::new(read_only_row(
                    "Race",
                    preset.race.clone(),
                    signals,
                )));
                rows.push(Box::new(read_only_row(
                    "Appearance",
                    preset.appearance.to_string(),
                    signals,
                )));
            }
            RevealGroup::Name => {
                rows.push(Box::new(
                    v_stack((
                        section_title("Your Name", signals),
                        form_row(
                            "Name",
                            signals,
                            text_input(form.name_fem)
                                .placeholder(&preset.name_fem)
                                .style(field_style(signals)),
                        ),
                    ))
                    .style(section_style()),
                ));
            }
            RevealGroup::Sexual => {
                let sexual_traits: Vec<String> = preset
                    .trait_ids
                    .iter()
                    .filter(|id| {
                        let s = id.as_str();
                        !PERSONALITY_TRAIT_IDS.contains(&s)
                            && !BODY_APPEARANCE_TRAIT_IDS.contains(&s)
                    })
                    .map(|id| trait_id_to_display(id))
                    .collect();
                if !sexual_traits.is_empty() {
                    rows.push(Box::new(trait_chips("Sexual", sexual_traits, signals)));
                }
            }
            RevealGroup::Begin => {
                // Begin is handled separately — the Begin button shows after all beats.
            }
        }
    }

    if rows.is_empty() {
        Box::new(empty())
    } else {
        Box::new(v_stack_from_iter(rows).style(|s| s.width_full().gap(4.0).margin_top(12.0)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use undone_domain::{Appearance, BeforeSexuality};
    use undone_packs::load_packs;
    use undone_scene::scheduler::load_schedule;

    fn packs_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("packs")
    }

    /// Helper: find the Robin preset by before_name from the loaded registry.
    fn robin_preset(registry: &PackRegistry) -> &PresetData {
        registry
            .presets()
            .iter()
            .find(|p| p.before_name == "Robin")
            .expect("Robin preset should be loaded from pack data")
    }

    /// Helper: find the Camila/Raul preset by before_name from the loaded registry.
    fn camila_preset(registry: &PackRegistry) -> &PresetData {
        registry
            .presets()
            .iter()
            .find(|p| p.before_name == "Raul")
            .expect("Camila/Raul preset should be loaded from pack data")
    }

    #[test]
    fn validate_registry_contract_reports_missing_traits() {
        let registry = PackRegistry::new();
        let errors = validate_registry_contract(&registry);
        assert!(
            errors.iter().any(|error| error.contains("BLOCK_ROUGH")),
            "expected missing trait error, got: {:?}",
            errors
        );
    }

    #[test]
    fn fem_form_defaults_use_preset_values_when_present() {
        let (registry, _) = load_packs(&packs_dir()).unwrap();
        let robin = robin_preset(&registry);
        let robin_idx = registry
            .presets()
            .iter()
            .position(|p| p.before_name == "Robin")
            .unwrap() as u8;

        let partial = PartialCharState {
            origin: PcOrigin::CisMaleTransformed,
            before_name: "Robin".into(),
            before_age: Age::Thirties,
            before_race: "White".into(),
            before_sexuality: BeforeSexuality::AttractedToWomen,
            starting_traits: vec![],
            starting_flags: vec!["ROUTE_WORKPLACE".into()],
            preset_idx: Some(robin_idx),
            appearance: Appearance::Average,
        };

        let defaults = fem_form_defaults(&registry, Some(&partial), Some("White"));
        assert_eq!(defaults.name_fem, "Robin");
        assert_eq!(defaults.figure, robin.figure);
        assert_eq!(defaults.breasts, robin.breasts);
        assert_eq!(defaults.race, robin.race);
    }

    #[test]
    fn fem_form_defaults_use_camila_name_for_raul_preset() {
        let (registry, _) = load_packs(&packs_dir()).unwrap();
        let camila = camila_preset(&registry);
        let camila_idx = registry
            .presets()
            .iter()
            .position(|p| p.before_name == "Raul")
            .unwrap() as u8;

        let partial = PartialCharState {
            origin: PcOrigin::CisMaleTransformed,
            before_name: "Raul".into(),
            before_age: Age::LateTeen,
            before_race: "Latina".into(),
            before_sexuality: BeforeSexuality::AttractedToWomen,
            starting_traits: vec![],
            starting_flags: vec!["ROUTE_CAMPUS".into()],
            preset_idx: Some(camila_idx),
            appearance: Appearance::Average,
        };

        let defaults = fem_form_defaults(&registry, Some(&partial), Some("White"));
        assert_eq!(defaults.name_fem, "Camila");
        assert_eq!(defaults.figure, camila.figure);
        assert_eq!(defaults.breasts, camila.breasts);
        assert_eq!(defaults.race, camila.race);
    }

    #[test]
    fn fem_form_defaults_fall_back_to_before_race_for_custom_mode() {
        let (registry, _) = load_packs(&packs_dir()).unwrap();

        let partial = PartialCharState {
            origin: PcOrigin::CisMaleTransformed,
            before_name: "Evan".into(),
            before_age: Age::EarlyTwenties,
            before_race: "Latina".into(),
            before_sexuality: BeforeSexuality::AttractedToWomen,
            starting_traits: vec![],
            starting_flags: vec![],
            preset_idx: None,
            appearance: Appearance::Average,
        };

        let defaults = fem_form_defaults(&registry, Some(&partial), Some("White"));
        assert_eq!(defaults.name_fem, "Eva");
        assert_eq!(defaults.race, "Latina");
        assert_eq!(defaults.age, Age::EarlyTwenties);
    }

    #[test]
    fn validate_runtime_contract_accepts_base_pack_routes() {
        let (registry, metas) = load_packs(&packs_dir()).unwrap();
        let scheduler = load_schedule(&metas, &registry).unwrap();

        let errors = validate_runtime_contract(&registry, &scheduler);

        assert!(
            errors.is_empty(),
            "expected no runtime contract errors, got: {:?}",
            errors
        );
    }

    #[test]
    fn robin_quick_config_builds_workplace_preset() {
        let (registry, _) = load_packs(&packs_dir()).unwrap();
        let robin = robin_preset(&registry);

        let config = robin_quick_config(&registry);

        assert_eq!(config.name_fem, robin.name_fem);
        assert_eq!(config.name_masc, robin.name_masc);
        assert!(config.starting_flags.contains("ROUTE_WORKPLACE"));
        assert_eq!(config.male_count, 6);
        assert_eq!(config.female_count, 3);
        assert_eq!(config.appearance, robin.appearance);
        assert_eq!(config.starting_traits.len(), robin.trait_ids.len());
    }

    #[test]
    fn fem_creation_bridge_copy_is_route_aware_for_workplace_preset() {
        let partial = PartialCharState {
            origin: PcOrigin::CisMaleTransformed,
            before_name: "Robin".into(),
            before_age: Age::Thirties,
            before_race: "White".into(),
            before_sexuality: BeforeSexuality::AttractedToWomen,
            starting_traits: vec![],
            starting_flags: vec!["ROUTE_WORKPLACE".into()],
            preset_idx: Some(0),
            appearance: Appearance::Average,
        };

        let prose = fem_creation_bridge_copy(Some(&partial));
        let paragraphs: Vec<&str> = prose.split("\n\n").collect();

        assert!(
            paragraphs.len() >= 2,
            "workplace bridge should feel like a real beat, not a one-line placeholder: {prose}"
        );
        assert!(
            prose.contains("mirror") || prose.contains("bathroom"),
            "bridge should include physical discovery: {prose}"
        );
        assert!(
            prose.contains("job") || prose.contains("Monday"),
            "workplace bridge should stay tied to the route pressure: {prose}"
        );
    }

    /// Physical/body traits from presets must NOT appear in BeforeCreation's
    /// personality display — they are post-transformation attributes.
    #[test]
    fn before_creation_personality_display_excludes_physical_traits() {
        let physical_traits = [
            "STRAIGHT_HAIR",
            "SWEET_VOICE",
            "ALMOND_EYES",
            "WIDE_HIPS",
            "NARROW_WAIST",
            "SMALL_HANDS",
        ];
        for id in physical_traits {
            assert!(
                !PERSONALITY_TRAIT_IDS.contains(&id),
                "Physical trait '{}' must not be in PERSONALITY_TRAIT_IDS",
                id
            );
        }
    }

    /// Robin's personality display (BeforeCreation) must only contain personality traits —
    /// filtering on PERSONALITY_TRAIT_IDS must exclude all physical/sexual traits.
    #[test]
    fn robin_preset_personality_display_excludes_body_and_sexual_traits() {
        let (registry, _) = load_packs(&packs_dir()).unwrap();
        let robin = robin_preset(&registry);

        let displayed: Vec<&str> = robin
            .trait_ids
            .iter()
            .map(|s| s.as_str())
            .filter(|id| PERSONALITY_TRAIT_IDS.contains(id))
            .collect();

        assert!(
            !displayed.is_empty(),
            "Robin should have personality traits to display"
        );

        let non_personality = [
            "STRAIGHT_HAIR",
            "SWEET_VOICE",
            "ALMOND_EYES",
            "WIDE_HIPS",
            "NARROW_WAIST",
            "SMALL_HANDS",
            "HAIR_TRIGGER",
            "HEAVY_SQUIRTER",
            "MULTI_ORGASMIC",
            "REGULAR_PERIODS",
        ];
        for id in displayed {
            assert!(
                !non_personality.contains(&id),
                "Non-personality trait '{}' incorrectly appears in BeforeCreation display",
                id
            );
        }
    }
}
