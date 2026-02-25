use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::dropdown::Dropdown;
use floem::views::Checkbox;
use rand::SeedableRng;
use std::cell::RefCell;
use std::rc::Rc;
use undone_domain::{
    Age, BeforeIdentity, BeforeSexuality, BreastSize, MaleFigure, PcOrigin, PlayerFigure,
};
use undone_packs::char_creation::CharCreationConfig;

use crate::game_state::{start_game, GameState, PreGameState};
use crate::theme::ThemeColors;
use crate::{AppPhase, AppSignals, PartialCharState};

// ── PC origin helpers ─────────────────────────────────────────────────────────

fn origin_from_idx(idx: u8) -> PcOrigin {
    match idx {
        0 => PcOrigin::CisMaleTransformed,
        1 => PcOrigin::TransWomanTransformed,
        2 => PcOrigin::CisFemaleTransformed,
        _ => PcOrigin::AlwaysFemale,
    }
}

/// Read the race list from the pack registry, falling back to `["White"]` if empty.
fn read_races(pre_state: &Rc<RefCell<Option<PreGameState>>>) -> Vec<String> {
    if let Some(ref pre) = *pre_state.borrow() {
        if !pre.registry.races().is_empty() {
            return pre.registry.races().to_vec();
        }
    }
    vec!["White".to_string()]
}

// ── BeforeCreation form signals ───────────────────────────────────────────────

#[derive(Clone, Copy)]
struct BeforeFormSignals {
    origin_idx: RwSignal<u8>,
    before_name: RwSignal<String>,
    before_age: RwSignal<Age>,
    before_sexuality: RwSignal<BeforeSexuality>,
    before_race: RwSignal<String>,
    // personality
    trait_shy: RwSignal<bool>,
    trait_cute: RwSignal<bool>,
    trait_posh: RwSignal<bool>,
    trait_sultry: RwSignal<bool>,
    trait_down_to_earth: RwSignal<bool>,
    trait_bitchy: RwSignal<bool>,
    trait_refined: RwSignal<bool>,
    trait_romantic: RwSignal<bool>,
    trait_flirty: RwSignal<bool>,
    trait_ambitious: RwSignal<bool>,
    trait_outgoing: RwSignal<bool>,
    trait_overactive_imagination: RwSignal<bool>,
    trait_beautiful: RwSignal<bool>,
    trait_plain: RwSignal<bool>,
    // content prefs
    include_rough: RwSignal<bool>,
    likes_rough: RwSignal<bool>,
}

impl BeforeFormSignals {
    fn new() -> Self {
        Self {
            origin_idx: RwSignal::new(0),
            before_name: RwSignal::new("Evan".to_string()),
            before_age: RwSignal::new(Age::Twenties),
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
            trait_beautiful: RwSignal::new(false),
            trait_plain: RwSignal::new(false),
            include_rough: RwSignal::new(false),
            likes_rough: RwSignal::new(false),
        }
    }
}

// ── FemCreation form signals ──────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct FemFormSignals {
    name_fem: RwSignal<String>,
    name_androg: RwSignal<String>,
    age: RwSignal<Age>,
    figure: RwSignal<PlayerFigure>,
    breasts: RwSignal<BreastSize>,
    race: RwSignal<String>,
}

impl FemFormSignals {
    fn new() -> Self {
        Self {
            name_fem: RwSignal::new("Eva".to_string()),
            name_androg: RwSignal::new("Ev".to_string()),
            age: RwSignal::new(Age::EarlyTwenties),
            figure: RwSignal::new(PlayerFigure::Slim),
            breasts: RwSignal::new(BreastSize::MediumLarge),
            race: RwSignal::new(String::new()),
        }
    }
}

// ── public entry points ───────────────────────────────────────────────────────

/// BeforeCreation phase: who you were before the transformation.
pub fn char_creation_view(
    signals: AppSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
    partial_char: RwSignal<Option<PartialCharState>>,
) -> impl View {
    let form = BeforeFormSignals::new();

    let races_list = read_races(&pre_state);
    if let Some(first) = races_list.first() {
        form.before_race.set(first.clone());
    }

    let next_btn = build_next_button(signals, form, pre_state, game_state, partial_char);

    let content = v_stack((
        heading("Your Story Begins", signals),
        section_your_past(signals, form, races_list),
        section_personality(signals, form),
        section_content_prefs(signals, form),
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
pub fn fem_creation_view(
    signals: AppSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
    partial_char: RwSignal<Option<PartialCharState>>,
) -> impl View {
    let form = FemFormSignals::new();

    // Determine if AlwaysFemale (so we show the Age field).
    let is_always_female = partial_char
        .get_untracked()
        .map(|p| p.origin == PcOrigin::AlwaysFemale)
        .unwrap_or(false);

    // Default race to first available; override with before_race if the player set one.
    let races_list = read_races(&pre_state);
    if let Some(first) = races_list.first() {
        form.race.set(first.clone());
    }
    if let Some(ref partial) = partial_char.get_untracked() {
        if !partial.before_race.is_empty() {
            form.race.set(partial.before_race.clone());
        }
    }

    let begin_btn = build_begin_button(signals, form, pre_state, game_state, partial_char);

    let age_row: Box<dyn View> = if is_always_female {
        Box::new(form_row(
            "Age",
            signals,
            Dropdown::new_rw(
                form.age,
                vec![
                    Age::LateTeen,
                    Age::EarlyTwenties,
                    Age::Twenties,
                    Age::LateTwenties,
                    Age::Thirties,
                    Age::Forties,
                    Age::Fifties,
                    Age::Old,
                ],
            )
            .style(field_style(signals)),
        ))
    } else {
        Box::new(empty())
    };

    let content = v_stack((
        heading("Who Are You Now?", signals),
        // Names section
        v_stack((
            section_title("Your Name", signals),
            form_row(
                "Feminine name",
                signals,
                text_input(form.name_fem)
                    .placeholder("e.g. Eva")
                    .style(field_style(signals)),
            ),
            form_row(
                "Androgynous name",
                signals,
                text_input(form.name_androg)
                    .placeholder("e.g. Ev")
                    .style(field_style(signals)),
            ),
        ))
        .style(section_style()),
        // Body section
        v_stack((
            section_title("Your Body", signals),
            form_row(
                "Figure",
                signals,
                Dropdown::new_rw(
                    form.figure,
                    vec![
                        PlayerFigure::Slim,
                        PlayerFigure::Toned,
                        PlayerFigure::Womanly,
                    ],
                )
                .style(field_style(signals)),
            ),
            form_row(
                "Breasts",
                signals,
                Dropdown::new_rw(
                    form.breasts,
                    vec![
                        BreastSize::Small,
                        BreastSize::MediumSmall,
                        BreastSize::MediumLarge,
                        BreastSize::Large,
                    ],
                )
                .style(field_style(signals)),
            ),
        ))
        .style(section_style()),
        // Background section
        v_stack((
            section_title("Background", signals),
            form_row("Race", signals, race_picker(form.race, races_list, signals)),
            age_row,
        ))
        .style(section_style()),
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

// ── heading ───────────────────────────────────────────────────────────────────

fn heading(title: &'static str, signals: AppSignals) -> impl View {
    label(move || title.to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.font_size(28.0)
            .font_weight(floem::text::Weight::LIGHT)
            .color(colors.ink)
            .margin_bottom(32.0)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
    })
}

// ── section: Your Past ────────────────────────────────────────────────────────

fn section_your_past(
    signals: AppSignals,
    form: BeforeFormSignals,
    races: Vec<String>,
) -> impl View {
    let origin_idx = form.origin_idx;

    let origin_radios = v_stack((
        radio_opt(
            "Something happened to me \u{2014} I was a man",
            move || origin_idx.get() == 0,
            move || origin_idx.set(0),
            signals,
        ),
        radio_opt(
            "Something happened to me \u{2014} I was a trans woman",
            move || origin_idx.get() == 1,
            move || origin_idx.set(1),
            signals,
        ),
        radio_opt(
            "Something happened to me \u{2014} I was a woman",
            move || origin_idx.get() == 2,
            move || origin_idx.set(2),
            signals,
        ),
        radio_opt(
            "I was always a woman",
            move || origin_idx.get() == 3,
            move || origin_idx.set(3),
            signals,
        ),
    ))
    .style(|s| s.margin_bottom(16.0));

    let before_fields = dyn_container(
        move || origin_idx.get(),
        move |idx| {
            if idx == 3 {
                // AlwaysFemale — no before-fields
                return empty().into_any();
            }

            let origin = origin_from_idx(idx);
            let br = races.clone();

            let name_row = form_row(
                "Name before",
                signals,
                text_input(form.before_name)
                    .placeholder("e.g. Evan")
                    .style(field_style(signals)),
            );

            let age_row = form_row(
                "Age before",
                signals,
                Dropdown::new_rw(
                    form.before_age,
                    vec![
                        Age::LateTeen,
                        Age::EarlyTwenties,
                        Age::Twenties,
                        Age::LateTwenties,
                        Age::Thirties,
                        Age::Forties,
                        Age::Fifties,
                        Age::Old,
                    ],
                )
                .style(field_style(signals)),
            );

            let race_row = form_row(
                "Race before",
                signals,
                race_picker(form.before_race, br, signals),
            );

            if origin.was_male_bodied() {
                let sexuality_row = form_row(
                    "Before sexuality",
                    signals,
                    Dropdown::new_rw(
                        form.before_sexuality,
                        vec![
                            BeforeSexuality::AttractedToWomen,
                            BeforeSexuality::AttractedToMen,
                            BeforeSexuality::AttractedToBoth,
                        ],
                    )
                    .style(field_style(signals)),
                );
                v_stack((name_row, age_row, sexuality_row, race_row)).into_any()
            } else {
                v_stack((name_row, age_row, race_row)).into_any()
            }
        },
    );

    v_stack((
        section_title("Your Past", signals),
        origin_radios,
        before_fields,
    ))
    .style(section_style())
}

// ── section: Personality ──────────────────────────────────────────────────────

fn section_personality(signals: AppSignals, form: BeforeFormSignals) -> impl View {
    let beautiful = form.trait_beautiful;
    let plain = form.trait_plain;

    let beautiful_cb = h_stack((
        Checkbox::new_rw(beautiful).style(|s| s.margin_right(8.0)),
        label(|| "Beautiful").style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(14.0)
                .color(colors.ink)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
    ))
    .style(|s| s.items_center().cursor(floem::style::CursorStyle::Pointer))
    .on_click_stop(move |_| {
        let new_val = !beautiful.get();
        beautiful.set(new_val);
        if new_val {
            plain.set(false);
        }
    });

    let plain_cb = h_stack((
        Checkbox::new_rw(plain).style(|s| s.margin_right(8.0)),
        label(|| "Plain").style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(14.0)
                .color(colors.ink)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
    ))
    .style(|s| s.items_center().cursor(floem::style::CursorStyle::Pointer))
    .on_click_stop(move |_| {
        let new_val = !plain.get();
        plain.set(new_val);
        if new_val {
            beautiful.set(false);
        }
    });

    let trait_grid = v_stack((
        h_stack((
            trait_checkbox("Shy", form.trait_shy, signals),
            trait_checkbox("Cute", form.trait_cute, signals),
            trait_checkbox("Posh", form.trait_posh, signals),
            trait_checkbox("Sultry", form.trait_sultry, signals),
            trait_checkbox("Down to Earth", form.trait_down_to_earth, signals),
        ))
        .style(|s| {
            s.gap(16.0)
                .margin_bottom(8.0)
                .flex_wrap(floem::style::FlexWrap::Wrap)
        }),
        h_stack((
            trait_checkbox("Bitchy", form.trait_bitchy, signals),
            trait_checkbox("Refined", form.trait_refined, signals),
            trait_checkbox("Romantic", form.trait_romantic, signals),
            trait_checkbox("Flirty", form.trait_flirty, signals),
            trait_checkbox("Ambitious", form.trait_ambitious, signals),
        ))
        .style(|s| {
            s.gap(16.0)
                .margin_bottom(8.0)
                .flex_wrap(floem::style::FlexWrap::Wrap)
        }),
        h_stack((
            trait_checkbox("Outgoing", form.trait_outgoing, signals),
            trait_checkbox(
                "Overactive Imagination",
                form.trait_overactive_imagination,
                signals,
            ),
        ))
        .style(|s| s.gap(16.0).flex_wrap(floem::style::FlexWrap::Wrap)),
    ));

    let divider = empty().style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.height(1.0)
            .width_full()
            .background(colors.seam)
            .margin_vert(12.0)
    });

    let appearance_row = h_stack((beautiful_cb, plain_cb)).style(|s| s.gap(24.0).items_center());

    v_stack((
        section_title("Personality", signals),
        hint_label("Pick 2\u{2013}3 traits:", signals),
        trait_grid,
        divider,
        appearance_row,
    ))
    .style(section_style())
}

// ── section: Content Preferences ─────────────────────────────────────────────

fn section_content_prefs(signals: AppSignals, form: BeforeFormSignals) -> impl View {
    v_stack((
        section_title("Content Preferences", signals),
        Checkbox::labeled_rw(form.include_rough, || {
            "Include rough / non-con content".to_string()
        })
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.items_center()
                .gap(8.0)
                .font_size(14.0)
                .color(colors.ink)
                .margin_bottom(10.0)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
        Checkbox::labeled_rw(form.likes_rough, || "I enjoy rougher content".to_string()).style(
            move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                s.items_center()
                    .gap(8.0)
                    .font_size(14.0)
                    .color(colors.ink)
                    .font_family("system-ui, -apple-system, sans-serif".to_string())
            },
        ),
    ))
    .style(section_style())
}

// ── Next button ───────────────────────────────────────────────────────────────

fn build_next_button(
    signals: AppSignals,
    form: BeforeFormSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
    partial_char: RwSignal<Option<PartialCharState>>,
) -> impl View {
    label(|| "Next \u{2192}".to_string())
        .keyboard_navigable()
        .on_click_stop(move |_| {
            let origin_idx = form.origin_idx.get_untracked();
            // Guard: non-AlwaysFemale origins require a name before proceeding.
            if origin_idx != 3 && form.before_name.get_untracked().trim().is_empty() {
                return;
            }
            let origin = origin_from_idx(origin_idx);

            // Collect starting traits
            let mut trait_names: Vec<&'static str> = Vec::new();
            if form.trait_shy.get_untracked() {
                trait_names.push("SHY");
            }
            if form.trait_cute.get_untracked() {
                trait_names.push("CUTE");
            }
            if form.trait_posh.get_untracked() {
                trait_names.push("POSH");
            }
            if form.trait_sultry.get_untracked() {
                trait_names.push("SULTRY");
            }
            if form.trait_down_to_earth.get_untracked() {
                trait_names.push("DOWN_TO_EARTH");
            }
            if form.trait_bitchy.get_untracked() {
                trait_names.push("BITCHY");
            }
            if form.trait_refined.get_untracked() {
                trait_names.push("REFINED");
            }
            if form.trait_romantic.get_untracked() {
                trait_names.push("ROMANTIC");
            }
            if form.trait_flirty.get_untracked() {
                trait_names.push("FLIRTY");
            }
            if form.trait_ambitious.get_untracked() {
                trait_names.push("AMBITIOUS");
            }
            if form.trait_outgoing.get_untracked() {
                trait_names.push("OUTGOING");
            }
            if form.trait_overactive_imagination.get_untracked() {
                trait_names.push("OVERACTIVE_IMAGINATION");
            }
            if form.trait_beautiful.get_untracked() {
                trait_names.push("BEAUTIFUL");
            }
            if form.trait_plain.get_untracked() {
                trait_names.push("PLAIN");
            }
            if !form.include_rough.get_untracked() {
                trait_names.push("BLOCK_ROUGH");
            }
            if form.likes_rough.get_untracked() {
                trait_names.push("LIKES_ROUGH");
            }

            // Resolve trait IDs from registry
            let starting_traits: Vec<_> = {
                let pre_borrow = pre_state.borrow();
                if let Some(ref pre) = *pre_borrow {
                    trait_names
                        .iter()
                        .filter_map(|name| pre.registry.resolve_trait(name).ok())
                        .collect()
                } else {
                    vec![]
                }
            };

            let partial = PartialCharState {
                origin,
                before_name: form.before_name.get_untracked(),
                before_age: form.before_age.get_untracked(),
                before_race: form.before_race.get_untracked(),
                before_sexuality: form.before_sexuality.get_untracked(),
                starting_traits,
            };
            partial_char.set(Some(partial.clone()));

            if origin == PcOrigin::AlwaysFemale {
                // Skip transformation intro — go straight to fem creation.
                signals.phase.set(AppPhase::FemCreation);
            } else {
                // Create a throwaway world for the transformation intro scene.
                // This world is discarded after the intro — the real world is
                // created at FemCreation submit via new_game().
                let before_identity = Some(BeforeIdentity {
                    name: partial.before_name.clone(),
                    age: partial.before_age,
                    race: partial.before_race.clone(),
                    sexuality: partial.before_sexuality,
                    figure: MaleFigure::Average,
                    traits: std::collections::HashSet::new(),
                });
                let throwaway_config = CharCreationConfig {
                    name_fem: String::new(),
                    name_androg: String::new(),
                    name_masc: partial.before_name.clone(),
                    age: partial.before_age,
                    race: partial.before_race.clone(),
                    figure: PlayerFigure::Slim,
                    breasts: BreastSize::MediumLarge,
                    origin,
                    before: before_identity,
                    starting_traits: partial.starting_traits.clone(),
                    male_count: 0,
                    female_count: 0,
                    starting_flags: std::collections::HashSet::new(),
                    starting_arc_states: std::collections::HashMap::new(),
                };

                {
                    let mut pre_mut = pre_state.borrow_mut();
                    if let Some(ref mut pre) = *pre_mut {
                        let throwaway_world = undone_packs::char_creation::new_game(
                            throwaway_config,
                            &mut pre.registry,
                            &mut pre.rng,
                        );
                        let engine = undone_scene::engine::SceneEngine::new(pre.scenes.clone());
                        let throwaway_gs = GameState {
                            world: throwaway_world,
                            registry: pre.registry.clone(),
                            engine,
                            scheduler: pre.scheduler.clone(),
                            rng: rand::rngs::SmallRng::from_entropy(),
                            init_error: None,
                            opening_scene: pre.registry.opening_scene().map(|s| s.to_owned()),
                            default_slot: pre.registry.default_slot().map(|s| s.to_owned()),
                        };
                        *game_state.borrow_mut() = Some(throwaway_gs);
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
                .font_family("system-ui, -apple-system, sans-serif".to_string())
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

fn build_begin_button(
    signals: AppSignals,
    form: FemFormSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
    partial_char: RwSignal<Option<PartialCharState>>,
) -> impl View {
    label(|| "Begin Your Story".to_string())
        .keyboard_navigable()
        .on_click_stop(move |_| {
            let pre = match pre_state.borrow_mut().take() {
                Some(p) => p,
                None => return, // already started
            };

            let partial = partial_char.get_untracked().unwrap_or_else(|| {
                // AlwaysFemale path: partial may not carry before-data.
                PartialCharState {
                    origin: PcOrigin::AlwaysFemale,
                    before_name: String::new(),
                    before_age: form.age.get_untracked(),
                    before_race: form.race.get_untracked(),
                    before_sexuality: BeforeSexuality::AttractedToWomen,
                    starting_traits: vec![],
                }
            });

            let origin = partial.origin;
            let fem_race = form.race.get_untracked();

            let before = if origin.has_before_life() {
                Some(BeforeIdentity {
                    name: partial.before_name.clone(),
                    age: partial.before_age,
                    race: partial.before_race.clone(),
                    sexuality: partial.before_sexuality,
                    figure: MaleFigure::Average,
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

            let config = CharCreationConfig {
                name_fem: form.name_fem.get_untracked(),
                name_androg: form.name_androg.get_untracked(),
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
                starting_flags: std::collections::HashSet::new(),
                starting_arc_states: std::collections::HashMap::new(),
            };

            let gs = start_game(pre, config);
            *game_state.borrow_mut() = Some(gs);
            signals.phase.set(AppPhase::InGame);
        })
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.margin_top(24.0)
                .padding_horiz(40.0)
                .padding_vert(16.0)
                .font_size(16.0)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
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

// ── helpers ───────────────────────────────────────────────────────────────────

fn section_title(title: &'static str, signals: AppSignals) -> impl View {
    label(move || title.to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.font_size(13.0)
            .font_weight(floem::text::Weight::SEMIBOLD)
            .color(colors.ink_ghost)
            .margin_bottom(16.0)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
    })
}

fn hint_label(text: &'static str, signals: AppSignals) -> impl View {
    label(move || text.to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.font_size(13.0)
            .color(colors.ink_dim)
            .margin_bottom(12.0)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
    })
}

fn form_row(label_text: &'static str, signals: AppSignals, input: impl IntoView) -> impl View {
    h_stack((
        label(move || label_text.to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.width(180.0)
                .font_size(14.0)
                .color(colors.ink_dim)
                .items_center()
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
        input.into_view(),
    ))
    .style(|s| s.items_center().margin_bottom(12.0))
}

fn trait_checkbox(name: &'static str, sig: RwSignal<bool>, signals: AppSignals) -> impl View {
    Checkbox::labeled_rw(sig, move || name.to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.items_center()
            .gap(6.0)
            .font_size(14.0)
            .color(colors.ink)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
    })
}

fn radio_opt(
    opt_label: &'static str,
    is_active: impl Fn() -> bool + Copy + 'static,
    on_select: impl Fn() + Copy + 'static,
    signals: AppSignals,
) -> impl View {
    let indicator = empty().style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        let bg = if is_active() {
            colors.lamp
        } else {
            colors.page_raised
        };
        let border_col = if is_active() {
            colors.lamp
        } else {
            colors.seam
        };
        s.width(13.0)
            .height(13.0)
            .border_radius(7.0)
            .border(1.5)
            .border_color(border_col)
            .background(bg)
            .margin_right(8.0)
    });
    h_stack((
        indicator,
        label(move || opt_label.to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(14.0)
                .color(colors.ink)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
    ))
    .style(|s| {
        s.items_center()
            .cursor(floem::style::CursorStyle::Pointer)
            .margin_bottom(8.0)
    })
    .on_click_stop(move |_| on_select())
}

fn race_picker(selection: RwSignal<String>, races: Vec<String>, signals: AppSignals) -> impl View {
    let races_signal = RwSignal::new(races);
    dyn_stack(
        move || races_signal.get(),
        |r| r.clone(),
        move |race| {
            let race_for_cmp = race.clone();
            let race_for_set = race.clone();
            let is_sel = move || selection.get() == race_for_cmp;
            let set_race = move || selection.set(race_for_set.clone());
            label(move || race.clone())
                .on_click_stop(move |_| set_race())
                .style(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    let selected = is_sel();
                    s.padding_horiz(12.0)
                        .padding_vert(6.0)
                        .margin_right(4.0)
                        .margin_bottom(4.0)
                        .border(1.0)
                        .border_radius(4.0)
                        .font_size(14.0)
                        .font_family("system-ui, -apple-system, sans-serif".to_string())
                        .cursor(floem::style::CursorStyle::Pointer)
                        .border_color(if selected { colors.lamp } else { colors.seam })
                        .color(if selected { colors.lamp } else { colors.ink })
                        .background(if selected {
                            colors.lamp_glow
                        } else {
                            floem::peniko::Color::TRANSPARENT
                        })
                })
        },
    )
    .style(|s| s.flex_row().flex_wrap(floem::style::FlexWrap::Wrap))
}

fn section_style() -> impl Fn(floem::style::Style) -> floem::style::Style {
    |s| {
        s.flex_col()
            .width_full()
            .margin_bottom(32.0)
            .padding_bottom(24.0)
    }
}

/// Shared style for text inputs and dropdowns — same dimensions, border, and font.
fn field_style(signals: AppSignals) -> impl Fn(floem::style::Style) -> floem::style::Style {
    move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.width(220.0)
            .height(32.0)
            .padding_horiz(10.0)
            .font_size(14.0)
            .color(colors.ink)
            .background(colors.page_raised)
            .border(1.0)
            .border_color(colors.seam)
            .border_radius(4.0)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
    }
}
