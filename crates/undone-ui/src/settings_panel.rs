use floem::peniko::Color;
use floem::prelude::*;

use crate::theme::{save_prefs, NumberKeyMode, ThemeColors, ThemeMode};
use crate::AppSignals;

pub fn settings_view(signals: AppSignals) -> impl View {
    let content = v_stack((
        settings_section_label("Theme", signals),
        theme_row(signals),
        settings_section_label("Font Size", signals),
        font_size_row(signals),
        settings_section_label("Line Height", signals),
        line_height_row(signals),
        settings_section_label("Number Key Mode", signals),
        number_key_mode_row(signals),
    ))
    .style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.width_full()
            .max_width(480.0)
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

fn settings_section_label(text: &'static str, signals: AppSignals) -> impl View {
    label(move || text.to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.font_size(12.0)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
            .color(colors.ink_ghost)
            .margin_top(16.0)
            .margin_bottom(4.0)
    })
}

fn theme_row(signals: AppSignals) -> impl View {
    let btn = |mode: ThemeMode, label_text: &'static str| {
        let is_active = move || signals.prefs.get().mode == mode;
        label(move || label_text.to_string())
            .on_click_stop(move |_| {
                signals.prefs.update(|p| p.mode = mode);
                save_prefs(&signals.prefs.get());
            })
            .style(move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                let active = is_active();
                s.padding_horiz(16.0)
                    .padding_vert(8.0)
                    .margin_right(4.0)
                    .border(1.0)
                    .border_radius(4.0)
                    .font_size(14.0)
                    .font_family("system-ui, -apple-system, sans-serif".to_string())
                    .cursor(floem::style::CursorStyle::Pointer)
                    .border_color(if active { colors.lamp } else { colors.seam })
                    .color(if active { colors.lamp } else { colors.ink })
                    .background(if active {
                        colors.lamp_glow
                    } else {
                        Color::TRANSPARENT
                    })
                    .hover(|s| s.background(colors.lamp_glow).border_color(colors.lamp))
            })
    };

    h_stack((
        btn(ThemeMode::Light, "Warm"),
        btn(ThemeMode::Sepia, "Sepia"),
        btn(ThemeMode::Dark, "Night"),
    ))
}

fn font_size_row(signals: AppSignals) -> impl View {
    let dec = move || {
        signals.prefs.update(|p| {
            if p.font_size > 14 {
                p.font_size -= 1;
            }
        });
        save_prefs(&signals.prefs.get());
    };
    let inc = move || {
        signals.prefs.update(|p| {
            if p.font_size < 24 {
                p.font_size += 1;
            }
        });
        save_prefs(&signals.prefs.get());
    };

    h_stack((
        stepper_button("\u{2212}", dec, signals),
        label(move || format!("{}", signals.prefs.get().font_size)).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.width(40.0)
                .font_size(15.0)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
                .color(colors.ink)
                .items_center()
                .justify_center()
        }),
        stepper_button("+", inc, signals),
    ))
    .style(|s| s.items_center())
}

fn line_height_row(signals: AppSignals) -> impl View {
    let dec = move || {
        signals.prefs.update(|p| {
            let next = (p.line_height * 10.0 - 1.0).round() / 10.0;
            if next >= 1.2 {
                p.line_height = next;
            }
        });
        save_prefs(&signals.prefs.get());
    };
    let inc = move || {
        signals.prefs.update(|p| {
            let next = (p.line_height * 10.0 + 1.0).round() / 10.0;
            if next <= 2.0 {
                p.line_height = next;
            }
        });
        save_prefs(&signals.prefs.get());
    };

    h_stack((
        stepper_button("\u{2212}", dec, signals),
        label(move || format!("{:.1}", signals.prefs.get().line_height)).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.width(40.0)
                .font_size(15.0)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
                .color(colors.ink)
                .items_center()
                .justify_center()
        }),
        stepper_button("+", inc, signals),
    ))
    .style(|s| s.items_center())
}

fn number_key_mode_row(signals: AppSignals) -> impl View {
    let btn = |mode: NumberKeyMode, label_text: &'static str| {
        let is_active = move || signals.prefs.get().number_key_mode == mode;
        label(move || label_text.to_string())
            .on_click_stop(move |_| {
                signals.prefs.update(|p| p.number_key_mode = mode);
                save_prefs(&signals.prefs.get());
            })
            .style(move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                let active = is_active();
                s.padding_horiz(16.0)
                    .padding_vert(8.0)
                    .margin_right(4.0)
                    .border(1.0)
                    .border_radius(4.0)
                    .font_size(14.0)
                    .font_family("system-ui, -apple-system, sans-serif".to_string())
                    .cursor(floem::style::CursorStyle::Pointer)
                    .border_color(if active { colors.lamp } else { colors.seam })
                    .color(if active { colors.lamp } else { colors.ink })
                    .background(if active {
                        colors.lamp_glow
                    } else {
                        Color::TRANSPARENT
                    })
                    .hover(|s| s.background(colors.lamp_glow).border_color(colors.lamp))
            })
    };

    h_stack((
        btn(NumberKeyMode::Instant, "Instant"),
        btn(NumberKeyMode::Confirm, "Confirm"),
    ))
}

fn stepper_button(
    text: &'static str,
    action: impl Fn() + 'static,
    signals: AppSignals,
) -> impl View {
    label(move || text.to_string())
        .on_click_stop(move |_| action())
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.width(32.0)
                .height(32.0)
                .items_center()
                .justify_center()
                .border(1.0)
                .border_radius(4.0)
                .font_size(16.0)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
                .border_color(colors.seam)
                .color(colors.ink)
                .cursor(floem::style::CursorStyle::Pointer)
                .hover(|s| s.background(colors.lamp_glow).border_color(colors.lamp))
        })
}
