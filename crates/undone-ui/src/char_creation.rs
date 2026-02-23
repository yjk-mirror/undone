use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::dropdown::Dropdown;
use floem::views::Checkbox;
use std::cell::RefCell;
use std::rc::Rc;
use undone_domain::{Age, BreastSize, PlayerFigure, Sexuality};
use undone_packs::char_creation::CharCreationConfig;

use crate::game_state::{error_game_state, start_game, GameState, PreGameState};
use crate::theme::ThemeColors;
use crate::{AppPhase, AppSignals};

// ── form-level signals ────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct CharFormSignals {
    name_fem: RwSignal<String>,
    name_androg: RwSignal<String>,
    name_masc: RwSignal<String>,
    age: RwSignal<Age>,
    figure: RwSignal<PlayerFigure>,
    breasts: RwSignal<BreastSize>,
    always_female: RwSignal<bool>,
    before_age_str: RwSignal<String>,
    sexuality: RwSignal<Sexuality>,
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
    trait_beautiful: RwSignal<bool>,
    trait_plain: RwSignal<bool>,
    // content
    include_rough: RwSignal<bool>,
    likes_rough: RwSignal<bool>,
}

impl CharFormSignals {
    fn new() -> Self {
        Self {
            name_fem: RwSignal::new("Eva".to_string()),
            name_androg: RwSignal::new("Ev".to_string()),
            name_masc: RwSignal::new("Evan".to_string()),
            age: RwSignal::new(Age::EarlyTwenties),
            figure: RwSignal::new(PlayerFigure::Slim),
            breasts: RwSignal::new(BreastSize::MediumLarge),
            always_female: RwSignal::new(false),
            before_age_str: RwSignal::new("28".to_string()),
            sexuality: RwSignal::new(Sexuality::StraightMale),
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
            trait_beautiful: RwSignal::new(false),
            trait_plain: RwSignal::new(false),
            include_rough: RwSignal::new(false),
            likes_rough: RwSignal::new(false),
        }
    }
}

// ── public entry point ────────────────────────────────────────────────────────

pub fn char_creation_view(
    signals: AppSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
) -> impl View {
    let form = CharFormSignals::new();

    let begin_btn = build_begin_button(signals, form, pre_state, game_state);

    let content = v_stack((
        heading(signals),
        section_who_you_are(signals, form),
        section_your_past(signals, form),
        section_personality(signals, form),
        section_content_prefs(signals, form),
        begin_btn,
        // bottom padding so the button isn't flush with bottom
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

fn heading(signals: AppSignals) -> impl View {
    label(|| "Create Your Character".to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.font_size(28.0)
            .font_weight(floem::text::Weight::LIGHT)
            .color(colors.ink)
            .margin_bottom(32.0)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
    })
}

// ── section: Who You Are Now ──────────────────────────────────────────────────

fn section_who_you_are(signals: AppSignals, form: CharFormSignals) -> impl View {
    v_stack((
        section_title("Who You Are Now", signals),
        form_row(
            "Feminine name",
            signals,
            text_input(form.name_fem)
                .placeholder("e.g. Eva")
                .style(input_style(signals)),
        ),
        form_row(
            "Androgynous name",
            signals,
            text_input(form.name_androg)
                .placeholder("e.g. Ev")
                .style(input_style(signals)),
        ),
        form_row(
            "Masculine name",
            signals,
            text_input(form.name_masc)
                .placeholder("e.g. Evan")
                .style(input_style(signals)),
        ),
        form_row(
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
            .style(dropdown_style(signals)),
        ),
        form_row(
            "Figure",
            signals,
            Dropdown::new_rw(
                form.figure,
                vec![PlayerFigure::Slim, PlayerFigure::Toned, PlayerFigure::Womanly],
            )
            .style(dropdown_style(signals)),
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
            .style(dropdown_style(signals)),
        ),
    ))
    .style(section_style())
}

// ── section: Your Past ────────────────────────────────────────────────────────

fn section_your_past(signals: AppSignals, form: CharFormSignals) -> impl View {
    let always_female = form.always_female;

    let past_fields = dyn_container(
        move || always_female.get(),
        move |is_always| {
            if is_always {
                empty().into_any()
            } else {
                v_stack((
                    form_row(
                        "Age before transition",
                        signals,
                        text_input(form.before_age_str)
                            .placeholder("28")
                            .style(input_style(signals)),
                    ),
                    form_row(
                        "Before sexuality",
                        signals,
                        Dropdown::new_rw(
                            form.sexuality,
                            vec![
                                Sexuality::StraightMale,
                                Sexuality::GayMale,
                                Sexuality::BiMale,
                            ],
                        )
                        .style(dropdown_style(signals)),
                    ),
                ))
                .into_any()
            }
        },
    );

    v_stack((
        section_title("Your Past", signals),
        h_stack((
            Checkbox::labeled_rw(always_female, || "I was always female")
                .style(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    s.items_center()
                        .gap(8.0)
                        .font_size(14.0)
                        .color(colors.ink)
                        .font_family("system-ui, -apple-system, sans-serif".to_string())
                }),
        ))
        .style(|s| s.margin_bottom(12.0)),
        past_fields,
    ))
    .style(section_style())
}

// ── section: Personality ──────────────────────────────────────────────────────

fn section_personality(signals: AppSignals, form: CharFormSignals) -> impl View {
    let beautiful = form.trait_beautiful;
    let plain = form.trait_plain;

    // Mutual exclusion: toggling beautiful clears plain and vice versa.
    // We use on_click_stop inside custom checkbox wrappers.
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
        .style(|s| s.gap(16.0).margin_bottom(8.0).flex_wrap(floem::style::FlexWrap::Wrap)),
        h_stack((
            trait_checkbox("Bitchy", form.trait_bitchy, signals),
            trait_checkbox("Refined", form.trait_refined, signals),
            trait_checkbox("Romantic", form.trait_romantic, signals),
            trait_checkbox("Flirty", form.trait_flirty, signals),
            trait_checkbox("Ambitious", form.trait_ambitious, signals),
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

fn section_content_prefs(signals: AppSignals, form: CharFormSignals) -> impl View {
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
        Checkbox::labeled_rw(form.likes_rough, || {
            "I enjoy rougher content".to_string()
        })
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.items_center()
                .gap(8.0)
                .font_size(14.0)
                .color(colors.ink)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
    ))
    .style(section_style())
}

// ── Begin button ──────────────────────────────────────────────────────────────

fn build_begin_button(
    signals: AppSignals,
    form: CharFormSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
) -> impl View {
    label(|| "Begin Your Story".to_string())
        .keyboard_navigable()
        .on_click_stop(move |_| {
            // Take the PreGameState out of the cell
            let pre = pre_state.borrow_mut().take();
            let pre = match pre {
                Some(p) => p,
                None => return, // already started
            };

            // If pack loading failed, build error state and go to in-game
            if let Some(ref err) = pre.init_error {
                let err_msg = err.clone();
                signals
                    .story
                    .set(format!("Error loading packs: {}", err_msg));
                let gs = error_game_state(err_msg);
                *game_state.borrow_mut() = Some(gs);
                signals.phase.set(AppPhase::InGame);
                return;
            }

            let is_always_female = form.always_female.get_untracked();

            // Resolve before_age
            let before_age: u32 = form
                .before_age_str
                .get_untracked()
                .parse()
                .unwrap_or(28);

            // Collect sexuality
            let before_sexuality = if is_always_female {
                Sexuality::AlwaysFemale
            } else {
                form.sexuality.get_untracked()
            };

            // Collect starting traits (string names)
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
            if is_always_female {
                trait_names.push("ALWAYS_FEMALE");
                trait_names.push("NOT_TRANSFORMED");
            }
            if !form.include_rough.get_untracked() {
                trait_names.push("BLOCK_ROUGH");
            }
            if form.likes_rough.get_untracked() {
                trait_names.push("LIKES_ROUGH");
            }
            if form.trait_beautiful.get_untracked() {
                trait_names.push("BEAUTIFUL");
            }
            if form.trait_plain.get_untracked() {
                trait_names.push("PLAIN");
            }

            // Resolve trait IDs from registry
            let starting_traits: Vec<_> = trait_names
                .iter()
                .filter_map(|name| pre.registry.resolve_trait(name).ok())
                .collect();

            let config = CharCreationConfig {
                name_fem: form.name_fem.get_untracked(),
                name_androg: form.name_androg.get_untracked(),
                name_masc: form.name_masc.get_untracked(),
                age: form.age.get_untracked(),
                race: "white".to_string(),
                figure: form.figure.get_untracked(),
                breasts: form.breasts.get_untracked(),
                always_female: is_always_female,
                before_age,
                before_race: "white".to_string(),
                before_sexuality,
                starting_traits,
                male_count: 6,
                female_count: 2,
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

fn section_style() -> impl Fn(floem::style::Style) -> floem::style::Style {
    |s| {
        s.flex_col()
            .width_full()
            .margin_bottom(32.0)
            .padding_bottom(24.0)
    }
}

fn input_style(signals: AppSignals) -> impl Fn(floem::style::Style) -> floem::style::Style {
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

fn dropdown_style(signals: AppSignals) -> impl Fn(floem::style::Style) -> floem::style::Style {
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
