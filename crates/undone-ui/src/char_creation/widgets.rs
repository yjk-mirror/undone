//! Shared themed widgets for character creation (labels, rows, chips, pickers, dropdown styling).
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::Checkbox;

use crate::theme::{ThemeColors, UI_FONT_FAMILY};
use crate::AppSignals;

// ── helpers ───────────────────────────────────────────────────────────────────

pub(crate) fn section_title(title: &'static str, signals: AppSignals) -> impl View {
    label(move || title.to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.font_size(13.0)
            .font_weight(floem::text::Weight::SEMIBOLD)
            .color(colors.ink_ghost)
            .margin_bottom(16.0)
            .font_family(UI_FONT_FAMILY.to_string())
    })
}

pub(crate) fn hint_label(text: &'static str, signals: AppSignals) -> impl View {
    label(move || text.to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.font_size(13.0)
            .color(colors.ink_dim)
            .margin_bottom(12.0)
            .font_family(UI_FONT_FAMILY.to_string())
    })
}

pub(crate) fn form_row(
    label_text: &'static str,
    signals: AppSignals,
    input: impl IntoView,
) -> impl View {
    h_stack((
        label(move || label_text.to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.width(180.0)
                .font_size(14.0)
                .color(colors.ink_dim)
                .items_center()
                .font_family(UI_FONT_FAMILY.to_string())
        }),
        input.into_view(),
    ))
    .style(|s| s.items_center().margin_bottom(12.0))
}

pub(crate) fn read_only_row(
    label_text: &'static str,
    value: String,
    signals: AppSignals,
) -> impl View {
    h_stack((
        label(move || label_text.to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.min_width(180.0)
                .width(180.0)
                .font_size(14.0)
                .color(colors.ink_dim)
                .items_center()
                .font_family(UI_FONT_FAMILY.to_string())
        }),
        label(move || value.clone()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(14.0)
                .flex_basis(0.0)
                .flex_grow(1.0)
                .flex_shrink(1.0)
                .max_width(400.0)
                .color(colors.ink)
                .font_family(UI_FONT_FAMILY.to_string())
        }),
    ))
    .style(|s| s.items_start().margin_bottom(12.0).max_width(600.0))
}

pub(crate) fn trait_id_to_display(id: &str) -> String {
    let s = id.to_lowercase().replace('_', " ");
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub(crate) fn trait_chips(
    label_text: &'static str,
    traits: Vec<String>,
    signals: AppSignals,
) -> impl View {
    let traits_sig = RwSignal::new(traits);
    let chips = dyn_stack(
        move || traits_sig.get(),
        |t| t.clone(),
        move |t| {
            label(move || t.clone()).style(move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                s.padding_horiz(8.0)
                    .padding_vert(3.0)
                    .margin_right(4.0)
                    .margin_bottom(4.0)
                    .border(1.0)
                    .border_radius(4.0)
                    .border_color(colors.seam)
                    .color(colors.ink_ghost)
                    .font_size(12.0)
                    .font_family(UI_FONT_FAMILY.to_string())
            })
        },
    )
    .style(|s| {
        s.flex_row()
            .flex_wrap(floem::style::FlexWrap::Wrap)
            .flex_basis(0.0)
            .flex_grow(1.0)
    });
    h_stack((
        label(move || label_text.to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.min_width(180.0)
                .width(180.0)
                .font_size(14.0)
                .color(colors.ink_dim)
                .font_family(UI_FONT_FAMILY.to_string())
        }),
        chips,
    ))
    .style(|s| s.items_start().margin_bottom(12.0).max_width(600.0))
}

pub(crate) fn trait_checkbox(
    name: &'static str,
    sig: RwSignal<bool>,
    signals: AppSignals,
) -> impl View {
    Checkbox::labeled_rw(sig, move || name.to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.items_center()
            .gap(6.0)
            .font_size(14.0)
            .color(colors.ink)
            .font_family(UI_FONT_FAMILY.to_string())
    })
}

pub(crate) fn radio_opt(
    opt_label: &'static str,
    subtitle: &'static str,
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
            .margin_top(3.0)
            .flex_shrink(0.0)
    });
    let text_col = v_stack((
        label(move || opt_label.to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(14.0)
                .color(colors.ink)
                .font_family(UI_FONT_FAMILY.to_string())
        }),
        label(move || subtitle.to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(12.0)
                .color(colors.ink_dim)
                .margin_top(2.0)
                .font_family(UI_FONT_FAMILY.to_string())
        }),
    ));
    h_stack((indicator, text_col))
        .style(|s| {
            s.items_start()
                .cursor(floem::style::CursorStyle::Pointer)
                .margin_bottom(10.0)
        })
        .on_click_stop(move |_| on_select())
}

pub(crate) fn race_picker(
    selection: RwSignal<String>,
    races: Vec<String>,
    signals: AppSignals,
) -> impl View {
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
                        .font_family(UI_FONT_FAMILY.to_string())
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

pub(crate) fn section_style() -> impl Fn(floem::style::Style) -> floem::style::Style {
    |s| {
        s.flex_col()
            .width_full()
            .margin_bottom(32.0)
            .padding_bottom(24.0)
    }
}

/// Returns a closure suitable for `Dropdown::list_item_view` that renders each item
/// with the current theme's ink color and page_raised background.
pub(crate) fn themed_item<T: std::fmt::Display + 'static>(
    signals: AppSignals,
) -> impl Fn(T) -> floem::AnyView {
    move |item| {
        let s = item.to_string();
        label(move || s.clone())
            .style(move |style| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                style
                    .color(colors.ink)
                    .background(colors.page_raised)
                    .padding_horiz(10.0)
                    .padding_vert(6.0)
                    .font_size(14.0)
                    .font_family(UI_FONT_FAMILY.to_string())
            })
            .into_any()
    }
}

/// Returns a closure suitable for `Dropdown::main_view` that renders the selected item
/// with the current theme's ink color — used to fix text-invisible bug in Night theme
/// (floem's default_main_view uses unstyled `text()` which doesn't inherit ink color).
pub(crate) fn themed_trigger<T: std::fmt::Display + 'static>(
    signals: AppSignals,
) -> impl Fn(T) -> floem::AnyView {
    move |item| {
        let s = item.to_string();
        label(move || s.clone())
            .style(move |style| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                style
                    .color(colors.ink)
                    .font_size(14.0)
                    .font_family(UI_FONT_FAMILY.to_string())
            })
            .into_any()
    }
}

/// Shared style for text inputs and dropdowns — same dimensions, border, and font.
pub(crate) fn field_style(
    signals: AppSignals,
) -> impl Fn(floem::style::Style) -> floem::style::Style {
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
            .font_family(UI_FONT_FAMILY.to_string())
    }
}
