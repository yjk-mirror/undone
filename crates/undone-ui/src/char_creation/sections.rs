//! Character-creation form sections (BeforeCreation past/personality/prefs, preset select/detail).
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::dropdown::Dropdown;
use floem::views::Checkbox;
use undone_domain::{Age, Appearance, BeforeSexuality};
use undone_packs::PresetData;

use crate::theme::{ThemeColors, UI_FONT_FAMILY};
use crate::AppSignals;

use super::config::*;
use super::signals::*;
use super::widgets::*;

// ── heading ───────────────────────────────────────────────────────────────────

pub(crate) fn heading(title: &'static str, signals: AppSignals) -> impl View {
    label(move || title.to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.font_size(28.0)
            .font_weight(floem::text::Weight::LIGHT)
            .color(colors.ink)
            .margin_bottom(32.0)
            .font_family(UI_FONT_FAMILY.to_string())
    })
}

// ── section: Your Past ────────────────────────────────────────────────────────

pub(crate) fn section_your_past(
    signals: AppSignals,
    form: BeforeFormSignals,
    races: Vec<String>,
    male_names: Vec<String>,
) -> impl View {
    let origin_idx = form.origin_idx;

    let origin_radios = v_stack((
        radio_opt(
            "Something happened to me \u{2014} I was a man",
            "Transformed from male. The core experience.",
            move || origin_idx.get() == 0,
            move || origin_idx.set(0),
            signals,
        ),
        radio_opt(
            "Something happened to me \u{2014} I was a trans woman",
            "Already knew yourself. The transformation was recognition.",
            move || origin_idx.get() == 1,
            move || origin_idx.set(1),
            signals,
        ),
        radio_opt(
            "Something happened to me \u{2014} I was a woman",
            "You were female. Something still changed.",
            move || origin_idx.get() == 2,
            move || origin_idx.set(2),
            signals,
        ),
        radio_opt(
            "I was always a woman",
            "No transformation. Play as yourself.",
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
            let mn = male_names.clone();
            let hint = mn.first().cloned().unwrap_or_else(|| "Matt".to_string());

            // Name field with Randomize button
            let name_row = {
                let mn_click = mn.clone();
                h_stack((
                    label(move || "Name before".to_string()).style(move |s| {
                        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                        s.width(180.0)
                            .font_size(14.0)
                            .color(colors.ink_dim)
                            .items_center()
                            .font_family(UI_FONT_FAMILY.to_string())
                    }),
                    text_input(form.before_name)
                        .placeholder(hint)
                        .style(move |s| {
                            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                            s.width(178.0)
                                .height(32.0)
                                .padding_horiz(10.0)
                                .font_size(14.0)
                                .color(colors.ink)
                                .background(colors.page_raised)
                                .border(1.0)
                                .border_color(colors.seam)
                                .border_radius(4.0)
                                .font_family(UI_FONT_FAMILY.to_string())
                        }),
                    label(|| "Rand".to_string())
                        .keyboard_navigable()
                        .on_click_stop(move |_| {
                            if !mn_click.is_empty() {
                                let idx = rand::random::<usize>() % mn_click.len();
                                form.before_name.set(mn_click[idx].clone());
                            }
                        })
                        .style(move |s| {
                            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                            s.width(40.0)
                                .height(32.0)
                                .margin_left(4.0)
                                .font_size(11.0)
                                .color(colors.ink_dim)
                                .border(1.0)
                                .border_color(colors.seam)
                                .border_radius(4.0)
                                .items_center()
                                .justify_center()
                                .cursor(floem::style::CursorStyle::Pointer)
                        }),
                ))
                .style(|s| s.items_center().margin_bottom(12.0))
            };

            let age_row = form_row(
                "Age before",
                signals,
                Dropdown::new_rw(
                    form.before_age,
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
                    .main_view(themed_trigger::<BeforeSexuality>(signals))
                    .list_item_view(themed_item::<BeforeSexuality>(signals))
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

pub(crate) fn section_personality(signals: AppSignals, form: BeforeFormSignals) -> impl View {
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
            trait_checkbox("Analytical", form.trait_analytical, signals),
            trait_checkbox("Confident", form.trait_confident, signals),
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

    let divider2 = empty().style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.height(1.0)
            .width_full()
            .background(colors.seam)
            .margin_vert(12.0)
    });

    let appearance_row = form_row(
        "Appearance",
        signals,
        Dropdown::new_rw(
            form.appearance,
            vec![
                Appearance::Plain,
                Appearance::Average,
                Appearance::Attractive,
                Appearance::Beautiful,
                Appearance::Stunning,
                Appearance::Devastating,
            ],
        )
        .main_view(themed_trigger::<Appearance>(signals))
        .list_item_view(themed_item::<Appearance>(signals))
        .style(field_style(signals)),
    );

    let attitude_grid = h_stack((
        trait_checkbox("Sexist", form.trait_sexist, signals),
        trait_checkbox("Homophobic", form.trait_homophobic, signals),
        trait_checkbox("Objectifying", form.trait_objectifying, signals),
    ))
    .style(|s| s.gap(16.0).flex_wrap(floem::style::FlexWrap::Wrap));

    v_stack((
        section_title("Personality", signals),
        hint_label("Pick 2\u{2013}3 traits:", signals),
        trait_grid,
        divider,
        appearance_row,
        divider2,
        hint_label("Former attitudes (optional):", signals),
        attitude_grid,
    ))
    .style(section_style())
}

// ── section: Content Preferences ─────────────────────────────────────────────

pub(crate) fn section_content_prefs(signals: AppSignals, form: BeforeFormSignals) -> impl View {
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
                .font_family(UI_FONT_FAMILY.to_string())
        }),
        Checkbox::labeled_rw(form.likes_rough, || "I enjoy rougher content".to_string()).style(
            move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                s.items_center()
                    .gap(8.0)
                    .font_size(14.0)
                    .color(colors.ink)
                    .font_family(UI_FONT_FAMILY.to_string())
            },
        ),
    ))
    .style(section_style())
}

// ── section: Preset select ────────────────────────────────────────────────────

pub(crate) fn section_preset_select(signals: AppSignals, char_mode: RwSignal<u8>) -> impl View {
    v_stack((
        section_title("Who Were You?", signals),
        radio_opt(
            "Robin",
            "Thirty-two. Software engineer, a decade in. Moved to a city you didn't know.",
            move || char_mode.get() == 0,
            move || char_mode.set(0),
            signals,
        ),
        radio_opt(
            "Raul",
            "Eighteen. Starting university. Expectations carefully calibrated.",
            move || char_mode.get() == 1,
            move || char_mode.set(1),
            signals,
        ),
        radio_opt(
            "Create your own",
            "Customize your origin, name, traits, and background.",
            move || char_mode.get() == 2,
            move || char_mode.set(2),
            signals,
        ),
    ))
    .style(section_style())
}

// ── section: Preset detail ────────────────────────────────────────────────────

/// Trait IDs that describe the character's personality (shown on the "Who Were You?" screen).
/// Body, sexual, arousal, and other post-transformation traits are NOT shown here.
pub(crate) const PERSONALITY_TRAIT_IDS: &[&str] = &[
    "SHY",
    "CUTE",
    "POSH",
    "SULTRY",
    "DOWN_TO_EARTH",
    "BITCHY",
    "REFINED",
    "ROMANTIC",
    "FLIRTY",
    "AMBITIOUS",
    "OUTGOING",
    "OVERACTIVE_IMAGINATION",
    "ANALYTICAL",
    "CONFIDENT",
    "SEXIST",
    "HOMOPHOBIC",
    "OBJECTIFYING",
];

/// Physical appearance trait IDs shown under "Physical" in the body section.
/// Non-personality traits not in this list appear under "Sexual".
pub(crate) const BODY_APPEARANCE_TRAIT_IDS: &[&str] = &[
    "STRAIGHT_HAIR",
    "WAVY_HAIR",
    "CURLY_HAIR",
    "SWEET_VOICE",
    "SMOKY_VOICE",
    "HUSKY_VOICE",
    "ALMOND_EYES",
    "WIDE_EYES",
    "HOODED_EYES",
    "WIDE_HIPS",
    "NARROW_HIPS",
    "NARROW_WAIST",
    "SMALL_HANDS",
    "LONG_LEGS",
    "LONG_NECK",
    "PRONOUNCED_COLLARBONES",
    "THIGH_GAP",
    "SOFT_SKIN",
    "NATURALLY_SMOOTH",
    "INTOXICATING_SCENT",
    "REGULAR_PERIODS",
    "IRREGULAR_PERIODS",
];

pub(crate) fn section_preset_detail(signals: AppSignals, preset: &PresetData) -> impl View {
    let personality_traits: Vec<String> = preset
        .trait_ids
        .iter()
        .filter(|id| PERSONALITY_TRAIT_IDS.contains(&id.as_str()))
        .map(|id| trait_id_to_display(id))
        .collect();

    let blurb = preset.blurb.clone();
    let name = preset.before_name.clone();
    let age = preset.before_age.to_string();
    let race = preset.before_race.clone();
    let build = preset.before_figure.to_string();
    let height = preset.before_height.to_string();
    let hair = preset.before_hair_colour.to_string();
    let eyes = preset.before_eye_colour.to_string();
    let skin = preset.before_skin_tone.to_string();
    let voice = preset.before_voice.to_string();
    let penis = preset.before_penis_size.to_string();

    v_stack((
        label(move || blurb.clone()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(14.0)
                .color(colors.ink)
                .margin_bottom(20.0)
                .font_family(UI_FONT_FAMILY.to_string())
        }),
        read_only_row("Name", name, signals),
        read_only_row("Age", age, signals),
        read_only_row("Race", race, signals),
        read_only_row("Build", build, signals),
        read_only_row("Height", height, signals),
        read_only_row("Hair", hair, signals),
        read_only_row("Eyes", eyes, signals),
        read_only_row("Skin tone", skin, signals),
        read_only_row("Voice", voice, signals),
        read_only_row("Penis size", penis, signals),
        trait_chips("Personality", personality_traits, signals),
    ))
    .style(|s| s.flex_col().width_full().margin_bottom(24.0))
}
