use floem::event::EventListener;
use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::RwSignal;

use crate::theme::ThemeColors;
use crate::AppSignals;
use crate::AppTab;

pub fn title_bar(signals: AppSignals) -> impl View {
    let tab = signals.tab;

    // Left zone: drag region with app name
    let left_zone = label(|| "UNDONE".to_string())
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.color(colors.lamp).font_size(11.0)
        })
        .container()
        .style(|s| {
            s.padding_left(16.0)
                .flex_grow(1.0)
                .height(40.0)
                .items_center()
        })
        .on_event_stop(EventListener::PointerDown, |_| {
            floem::action::drag_window();
        });

    // Center zone: tab buttons
    let tabs = h_stack((
        tab_button("Game", AppTab::Game, tab, signals),
        tab_button("Saves", AppTab::Saves, tab, signals),
        tab_button("Settings", AppTab::Settings, tab, signals),
    ));

    // Right zone: window control buttons
    let minimize_btn = label(|| "\u{2500}".to_string())
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.width(40.0)
                .height(40.0)
                .items_center()
                .justify_center()
                .color(colors.ink_ghost)
                .hover(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    s.color(colors.ink)
                })
        })
        .on_click_stop(|_| {
            floem::action::minimize_window();
        });

    let maximize_btn = label(|| "\u{25A1}".to_string())
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.width(40.0)
                .height(40.0)
                .items_center()
                .justify_center()
                .color(colors.ink_ghost)
                .hover(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    s.color(colors.ink)
                })
        })
        .on_click_stop(|_| {
            floem::action::toggle_window_maximized();
        });

    let close_btn = label(|| "\u{00D7}".to_string())
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.width(40.0)
                .height(40.0)
                .items_center()
                .justify_center()
                .color(colors.ink_ghost)
                .hover(|s| s.color(Color::rgb8(0xE0, 0x40, 0x40)))
        })
        .on_click_stop(|_| {
            floem::quit_app();
        });

    let right_zone = h_stack((minimize_btn, maximize_btn, close_btn));

    h_stack((left_zone, tabs, right_zone)).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.height(40.0)
            .width_full()
            .items_center()
            .border_bottom(1.0)
            .border_color(colors.seam)
            .background(colors.sidebar_ground)
    })
}

fn tab_button(
    name: &'static str,
    tab: AppTab,
    active: RwSignal<AppTab>,
    signals: AppSignals,
) -> impl View {
    label(move || name.to_string())
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            let is_active = active.get() == tab;
            let s = s
                .padding_horiz(16.0)
                .height(40.0)
                .items_center()
                .justify_center()
                .font_size(13.0)
                .hover(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    s.color(colors.ink_dim)
                });
            if is_active {
                s.color(colors.ink)
                    .border_bottom(2.0)
                    .border_color(colors.lamp)
            } else {
                s.color(colors.ink_ghost)
            }
        })
        .on_click_stop(move |_| {
            active.set(tab);
        })
}
