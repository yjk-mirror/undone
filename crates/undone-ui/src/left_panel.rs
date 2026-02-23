use crate::game_state::GameState;
use crate::theme::ThemeColors;
use crate::AppSignals;
use floem::event::{Event, EventListener};
use floem::keyboard::{Key, NamedKey};
use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::create_rw_signal;
use floem::style::FlexWrap;
use floem::views::dyn_stack;
use std::cell::RefCell;
use std::rc::Rc;
use undone_scene::engine::ActionView;

pub fn story_panel(signals: AppSignals, state: Rc<RefCell<GameState>>) -> impl View {
    let story = signals.story;
    let actions = signals.actions;
    let state_clone = Rc::clone(&state);
    let hovered_detail = create_rw_signal(String::new());

    // Global keyboard shortcut listener for left panel (1-9 to select choices)
    let keyboard_handler = move |e: &Event| {
        if let Event::KeyDown(key_event) = e {
            if let Key::Character(char_str) = &key_event.key.logical_key {
                if let Ok(num) = char_str.parse::<usize>() {
                    if num > 0 && num <= 9 {
                        let idx = num - 1;
                        let current_actions = actions.get();
                        if idx < current_actions.len() {
                            let action_id = current_actions[idx].id.clone();
                            let mut gs = state_clone.borrow_mut();
                            let GameState {
                                ref mut engine,
                                ref mut world,
                                ref registry,
                            } = *gs;
                            engine.send(
                                undone_scene::engine::EngineCommand::ChooseAction(action_id),
                                world,
                                registry,
                            );
                            let events = engine.drain();
                            crate::process_events(events, signals, world);
                            return true; // handled
                        }
                    }
                }
            }
        }
        false
    };

    let prose_label = label(move || story.get()).style(move |s| {
        let prefs = signals.prefs.get();
        let colors = ThemeColors::from_mode(prefs.mode);
        s.font_family(prefs.font_family)
            .font_size(prefs.font_size as f32)
            .line_height(prefs.line_height)
            .padding(24.0)
            .max_width(680.0) // ~65 chars
            .color(colors.ink)
    });

    let centered_prose =
        container(prose_label).style(|s| s.width_full().justify_center());

    let scroll_area = scroll(centered_prose).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.flex_grow(1.0).background(colors.page)
    });

    let detail_strip = label(move || hovered_detail.get()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.width_full()
            .min_height(28.0)
            .padding_horiz(24.0)
            .padding_vert(6.0)
            .font_size(13.0)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
            .color(colors.ink_ghost)
            .background(colors.page)
            .border_top(1.0)
            .border_color(colors.seam)
    });

    v_stack((scroll_area, detail_strip, choices_bar(signals, state, hovered_detail)))
        .keyboard_navigable()
        .on_event_stop(EventListener::KeyDown, move |e| {
            keyboard_handler(e);
        })
        .style(|s| s.flex_grow(1.0))
}

fn choices_bar(
    signals: AppSignals,
    state: Rc<RefCell<GameState>>,
    hovered_detail: floem::reactive::RwSignal<String>,
) -> impl View {
    let actions = signals.actions;

    let buttons = dyn_stack(
        move || actions.get(),
        |a: &ActionView| a.id.clone(),
        move |action| {
            let index = actions
                .get()
                .iter()
                .position(|a| a.id == action.id)
                .unwrap_or(0);
            let action_id = action.id.clone();
            let label_text = action.label.clone();
            let detail_text = action.detail.clone();
            let detail_text_enter = detail_text.clone();
            let state_clone = Rc::clone(&state);
            let signals_clone = signals;

            let exec_action = move || {
                let mut gs = state_clone.borrow_mut();
                let GameState {
                    ref mut engine,
                    ref mut world,
                    ref registry,
                } = *gs;
                engine.send(
                    undone_scene::engine::EngineCommand::ChooseAction(action_id.clone()),
                    world,
                    registry,
                );
                let events = engine.drain();
                crate::process_events(events, signals_clone, world);
            };

            let exec_action_click = exec_action.clone();
            let exec_action_key = exec_action;
            let hovered = create_rw_signal(false);

            h_stack((
                label(move || format!("{}·", index + 1)).style(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    let ink = if hovered.get() { colors.ink_dim } else { colors.ink_ghost };
                    s.padding_right(8.0)
                        .color(ink)
                        .font_size(15.0)
                        .font_family("system-ui, -apple-system, sans-serif".to_string())
                }),
                label(move || label_text.clone()).style(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    s.color(colors.ink)
                        .font_size(15.0)
                        .font_family("system-ui, -apple-system, sans-serif".to_string())
                }),
            ))
            .keyboard_navigable()
            .on_click_stop(move |_| {
                exec_action_click();
            })
            .on_event_cont(EventListener::PointerEnter, move |_| {
                hovered.set(true);
                hovered_detail.set(detail_text_enter.clone());
            })
            .on_event_cont(EventListener::PointerLeave, move |_| {
                hovered.set(false);
                hovered_detail.set(String::new());
            })
            .on_event_stop(EventListener::KeyDown, move |e| {
                if let Event::KeyDown(key_event) = e {
                    let key = &key_event.key.logical_key;
                    if key == &Key::Named(NamedKey::Enter) || key == &Key::Named(NamedKey::Space) {
                        exec_action_key();
                    }
                }
            })
            .style(move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                s.margin(4.0)
                    .padding_horiz(20.0)
                    .padding_vert(12.0)
                    .min_height(48.0)
                    .border(1.0)
                    .border_color(colors.seam)
                    .border_radius(4.0)
                    .background(Color::TRANSPARENT)
                    .items_center()
                    .hover(|s| s.background(colors.lamp_glow).border_color(colors.lamp))
                    .focus_visible(|s| {
                        s.background(colors.lamp_glow)
                            .border_color(colors.lamp)
                            .outline_color(colors.lamp)
                            .outline(2.0)
                    })
                    .active(|s| s.background(colors.lamp_glow).border_color(colors.lamp))
                    .disabled(|s| s.border_color(colors.seam).color(colors.ink_ghost))
            })
        },
    )
    .style(|s| s.flex_row().flex_wrap(FlexWrap::Wrap));

    container(buttons).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.padding(12.0)
            .border_top(1.0)
            .border_color(colors.seam)
            .min_height(64.0)
            .width_full()
            .background(colors.page)
    })
}
