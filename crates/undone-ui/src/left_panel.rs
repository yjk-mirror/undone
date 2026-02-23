use std::rc::Rc;
use std::cell::RefCell;
use floem::prelude::*;
use floem::peniko::Color;
use floem::views::dyn_stack;
use floem::style::FlexWrap;
use crate::AppSignals;
use crate::game_state::GameState;
use undone_scene::engine::ActionView;

// Design Tokens (Warm Paper)
const PAGE: Color = Color::rgb8(253, 250, 244);     // #FDFAF4
const INK: Color = Color::rgb8(28, 24, 20);           // #1C1814
const INK_GHOST: Color = Color::rgb8(140, 128, 120);     // #8C8078
const SEAM: Color = Color::rgba8(28, 24, 20, 25);        // #1C18141A (10%)
const LAMP: Color = Color::rgb8(176, 112, 48);        // #B07030
const LAMP_GLOW: Color = Color::rgba8(176, 112, 48, 30); // #B070301F (12%)

pub fn left_panel(signals: AppSignals, state: Rc<RefCell<GameState>>) -> impl View {
    let story = signals.story;

    v_stack((
        scroll(
            label(move || story.get())
                .style(|s| s
                    .font_family("Georgia, 'Times New Roman', serif".to_string())
                    .font_size(17.0)
                    .line_height(1.5)
                    .padding(24.0)
                    .max_width(680.0)
                    .color(INK)
                )
        )
        .style(|s| s.flex_grow(1.0).background(PAGE)),

        choices_bar(signals, state),
    ))
    .style(|s| s
        .flex_grow(1.0)
        .border_right(1.0)
        .border_color(SEAM)
    )
}

fn choices_bar(signals: AppSignals, state: Rc<RefCell<GameState>>) -> impl View {
    let actions = signals.actions;

    let buttons = dyn_stack(
        move || actions.get(),
        |a: &ActionView| a.id.clone(),
        move |action| {
            // Find index for number prefix
            let index = actions.get().iter().position(|a| a.id == action.id).unwrap_or(0);
            let action_id = action.id.clone();
            let label_text = action.label.clone();
            let state_clone = Rc::clone(&state);
            let signals_clone = signals;

            h_stack((
                label(move || format!("{}·", index + 1))
                    .style(|s| s
                        .padding_right(8.0)
                        .color(INK_GHOST)
                        .font_size(15.0)
                    ),
                label(move || label_text.clone())
                    .style(|s| s
                        .color(INK)
                        .font_size(15.0)
                    ),
            ))
            .on_click_stop(move |_| {
                let mut gs = state_clone.borrow_mut();
                let GameState { ref mut engine, ref mut world, ref registry } = *gs;
                engine.send(
                    undone_scene::engine::EngineCommand::ChooseAction(action_id.clone()),
                    world,
                    registry,
                );
                let events = engine.drain();
                crate::process_events(events, signals_clone, world);
            })
            .style(|s| s
                .margin(4.0)
                .padding_horiz(20.0)
                .padding_vert(12.0)
                .min_height(48.0)
                .border(1.0)
                .border_color(SEAM)
                .border_radius(4.0)
                .background(Color::TRANSPARENT)
                .items_center()
                .hover(|s| s
                    .background(LAMP_GLOW)
                    .border_color(LAMP)
                )
            )
        },
    )
    .style(|s| s.flex_row().flex_wrap(FlexWrap::Wrap));

    container(buttons)
        .style(|s| s
            .padding(12.0)
            .border_top(1.0)
            .border_color(SEAM)
            .min_height(64.0)
            .width_full()
            .background(PAGE)
        )
}
