use floem::event::EventListener;
use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::RwSignal;

use crate::theme::ThemeColors;
use crate::AppSignals;
use crate::AppTab;

pub fn title_bar(signals: AppSignals, dev_mode: bool) -> impl View {
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

    // Center zone: tab buttons (dev_mode is static — no reactivity needed)
    let tabs = if dev_mode {
        h_stack((
            tab_button("Game", AppTab::Game, tab, signals, dev_mode),
            tab_button("Saves", AppTab::Saves, tab, signals, dev_mode),
            tab_button("Settings", AppTab::Settings, tab, signals, dev_mode),
            tab_button("Dev", AppTab::Dev, tab, signals, dev_mode),
        ))
        .into_any()
    } else {
        h_stack((
            tab_button("Game", AppTab::Game, tab, signals, dev_mode),
            tab_button("Saves", AppTab::Saves, tab, signals, dev_mode),
            tab_button("Settings", AppTab::Settings, tab, signals, dev_mode),
        ))
        .into_any()
    };

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
    dev_mode: bool,
) -> impl View {
    label(move || name.to_string())
        .keyboard_navigable()
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            let is_active = active.get() == tab;
            let phase = signals.phase.get();
            let tabs_enabled = match tab {
                AppTab::Settings => true,
                AppTab::Game | AppTab::Saves => {
                    phase == crate::AppPhase::Landing || phase == crate::AppPhase::InGame
                }
                AppTab::Dev => dev_mode && phase == crate::AppPhase::InGame,
            };
            let s = s
                .padding_horiz(16.0)
                .height(40.0)
                .items_center()
                .justify_center()
                .font_size(13.0);

            if !tabs_enabled {
                s.color(colors.ink_ghost)
            } else if is_active {
                s.color(colors.ink)
                    .border_bottom(2.0)
                    .border_color(colors.lamp)
            } else {
                s.color(colors.ink_ghost).hover(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    s.color(colors.ink_dim)
                })
            }
        })
        .on_click_stop(move |_| {
            let phase = signals.phase.get_untracked();
            let tab_enabled = match tab {
                AppTab::Settings => true,
                AppTab::Game | AppTab::Saves => {
                    phase == crate::AppPhase::Landing || phase == crate::AppPhase::InGame
                }
                AppTab::Dev => dev_mode && phase == crate::AppPhase::InGame,
            };
            if tab_enabled {
                active.set(tab);
            }
        })
}
