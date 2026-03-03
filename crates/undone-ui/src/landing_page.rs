use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::dyn_stack;
use std::cell::RefCell;
use std::rc::Rc;

use crate::game_state::{load_game_state_from_save, GameState, PreGameState};
use crate::saves_panel::{list_saves, SaveEntry};
use crate::theme::ThemeColors;
use crate::{AppPhase, AppSignals, AppTab};

fn try_load_entry(
    entry: &SaveEntry,
    signals: AppSignals,
    pre_state: &Rc<RefCell<Option<PreGameState>>>,
    game_state: &Rc<RefCell<Option<GameState>>>,
    status_msg: RwSignal<String>,
) {
    let pre = match pre_state.borrow_mut().take() {
        Some(pre) => pre,
        None => {
            status_msg.set("Game is already in progress.".to_string());
            return;
        }
    };

    match load_game_state_from_save(pre, &entry.path) {
        Ok(loaded_game) => {
            *game_state.borrow_mut() = Some(loaded_game);
            signals.story.set(String::new());
            signals.actions.set(vec![]);
            signals.active_npc.set(None);
            signals.tab.set(AppTab::Game);
            signals.phase.set(AppPhase::InGame);
            status_msg.set(format!("Loaded: {}", entry.name));
        }
        Err(err) => {
            status_msg.set(err);
        }
    }
}

pub fn landing_view(
    signals: AppSignals,
    pre_state: Rc<RefCell<Option<PreGameState>>>,
    game_state: Rc<RefCell<Option<GameState>>>,
) -> impl View {
    let status_msg: RwSignal<String> = RwSignal::new(String::new());
    let show_load_list: RwSignal<bool> = RwSignal::new(false);
    let save_list: RwSignal<Vec<SaveEntry>> = RwSignal::new(list_saves());

    let new_game_btn = label(|| "New Game".to_string())
        .keyboard_navigable()
        .on_click_stop(move |_| {
            signals.tab.set(AppTab::Game);
            signals.phase.set(AppPhase::BeforeCreation);
        })
        .style(move |s| primary_btn_style(s, signals));

    let continue_btn = label(|| "Continue".to_string())
        .keyboard_navigable()
        .on_click_stop({
            let pre_state = Rc::clone(&pre_state);
            let game_state = Rc::clone(&game_state);
            move |_| {
                let saves = save_list.get_untracked();
                let Some(entry) = saves.first() else {
                    status_msg.set("No saves found. Start a new game.".to_string());
                    return;
                };
                try_load_entry(entry, signals, &pre_state, &game_state, status_msg);
            }
        })
        .style(move |s| primary_btn_style(s, signals));

    let load_btn = label(|| "Load".to_string())
        .keyboard_navigable()
        .on_click_stop(move |_| {
            save_list.set(list_saves());
            show_load_list.update(|v| *v = !*v);
        })
        .style(move |s| primary_btn_style(s, signals));

    let settings_btn = label(|| "Settings".to_string())
        .keyboard_navigable()
        .on_click_stop(move |_| {
            signals.tab.set(AppTab::Settings);
        })
        .style(move |s| primary_btn_style(s, signals));

    let status = label(move || status_msg.get()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.font_size(13.0)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
            .color(colors.ink_ghost)
            .min_height(18.0)
            .margin_top(10.0)
    });

    let save_section = dyn_view(move || {
        if !show_load_list.get() {
            return empty().into_any();
        }

        let content = if save_list.get().is_empty() {
            label(|| "No saves yet.".to_string())
                .style(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    s.font_size(14.0)
                        .font_family("system-ui, -apple-system, sans-serif".to_string())
                        .color(colors.ink_ghost)
                })
                .into_any()
        } else {
            dyn_stack(
                move || save_list.get(),
                |entry: &SaveEntry| entry.path.to_string_lossy().to_string(),
                {
                    let pre_state = Rc::clone(&pre_state);
                    let game_state = Rc::clone(&game_state);
                    move |entry: SaveEntry| {
                        let load_entry = entry.clone();
                        let delete_entry = entry.clone();

                        let load_btn = label(|| "Load".to_string())
                            .keyboard_navigable()
                            .on_click_stop({
                                let pre_state = Rc::clone(&pre_state);
                                let game_state = Rc::clone(&game_state);
                                move |_| {
                                    try_load_entry(
                                        &load_entry,
                                        signals,
                                        &pre_state,
                                        &game_state,
                                        status_msg,
                                    );
                                }
                            })
                            .style(move |s| small_btn_style(s, signals));

                        let delete_btn = label(|| "Delete".to_string())
                            .keyboard_navigable()
                            .on_click_stop(move |_| {
                                match std::fs::remove_file(&delete_entry.path) {
                                    Ok(()) => {
                                        save_list.set(list_saves());
                                        status_msg.set(String::new());
                                    }
                                    Err(err) => {
                                        status_msg.set(format!("Delete failed: {err}"));
                                    }
                                }
                            })
                            .style(move |s| small_btn_style(s, signals));

                        let controls =
                            h_stack((load_btn, delete_btn)).style(|s| s.flex_row().gap(8.0));
                        let name = label(move || entry.name.clone()).style(move |s| {
                            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                            s.font_size(14.0)
                                .font_family("system-ui, -apple-system, sans-serif".to_string())
                                .color(colors.ink)
                        });
                        let modified = label(move || entry.modified.clone()).style(move |s| {
                            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                            s.font_size(12.0)
                                .font_family("system-ui, -apple-system, sans-serif".to_string())
                                .color(colors.ink_dim)
                        });

                        v_stack((name, modified, controls)).style(move |s| {
                            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                            s.width_full()
                                .padding(12.0)
                                .margin_bottom(8.0)
                                .border(1.0)
                                .border_radius(4.0)
                                .border_color(colors.seam)
                                .background(colors.page_raised)
                        })
                    }
                },
            )
            .style(|s| s.width_full().flex_col())
            .into_any()
        };

        container(content)
            .style(move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                s.width_full()
                    .max_width(680.0)
                    .margin_top(20.0)
                    .padding(16.0)
                    .border(1.0)
                    .border_radius(6.0)
                    .border_color(colors.seam)
                    .background(colors.page)
            })
            .into_any()
    });

    let body = v_stack((
        label(|| "Your Story Begins".to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(34.0)
                .font_weight(floem::text::Weight::LIGHT)
                .color(colors.ink)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
        label(|| {
            "Choose how you want to start: begin a new life, continue where you left off, or load a specific save."
                .to_string()
        })
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.max_width(720.0)
                .margin_top(10.0)
                .font_size(15.0)
                .line_height(1.45)
                .color(colors.ink_dim)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
        h_stack((new_game_btn, continue_btn, load_btn, settings_btn))
            .style(|s| s.gap(12.0).margin_top(26.0).flex_wrap(floem::style::FlexWrap::Wrap)),
        status,
        save_section,
    ))
    .style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.width_full()
            .padding_horiz(32.0)
            .padding_vert(36.0)
            .items_center()
            .background(colors.ground)
    });

    scroll(body)
        .scroll_style(|s| s.shrink_to_fit())
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.size_full().background(colors.ground)
        })
}

fn primary_btn_style(s: floem::style::Style, signals: AppSignals) -> floem::style::Style {
    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
    s.padding_horiz(20.0)
        .padding_vert(10.0)
        .font_size(15.0)
        .font_family("system-ui, -apple-system, sans-serif".to_string())
        .border(1.5)
        .border_radius(6.0)
        .border_color(colors.lamp)
        .color(colors.lamp)
        .background(colors.lamp_glow)
        .hover(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.background(colors.lamp_glow)
        })
        .focus_visible(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.outline(2.0).outline_color(colors.lamp)
        })
}

fn small_btn_style(s: floem::style::Style, signals: AppSignals) -> floem::style::Style {
    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
    s.font_size(12.0)
        .font_family("system-ui, -apple-system, sans-serif".to_string())
        .padding_horiz(10.0)
        .padding_vert(5.0)
        .border(1.0)
        .border_radius(4.0)
        .border_color(colors.seam)
        .color(colors.ink_dim)
        .background(Color::TRANSPARENT)
        .hover(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.border_color(colors.lamp)
                .color(colors.lamp)
                .background(colors.lamp_glow)
        })
}
