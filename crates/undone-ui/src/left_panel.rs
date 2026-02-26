use crate::game_state::GameState;
use crate::theme::NumberKeyMode;
use crate::theme::ThemeColors;
use crate::AppSignals;
use floem::event::{Event, EventListener};
use floem::keyboard::{Key, NamedKey};
use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::{create_effect, create_rw_signal, RwSignal};
use floem::style::FlexWrap;
use floem::text::{
    Attrs, AttrsList, FamilyOwned, LineHeightValue, Style as TextStyle, TextLayout, Weight,
};
use floem::views::dyn_stack;
use pulldown_cmark::{Event as MdEvent, Options, Parser, Tag, TagEnd};
use std::cell::RefCell;
use std::rc::Rc;
use undone_scene::engine::{ActionView, EngineCommand};

/// Convert a markdown string into a floem `TextLayout` with styled spans.
///
/// Supports: bold (`**text**`), italic (`*text*`), bold-italic, headings (H1-H3).
/// Paragraphs are separated by a blank line. Block elements are joined with `\n\n`.
/// Inline code is rendered in normal weight (no monospace — we have no mono font embedded).
fn markdown_to_text_layout(
    markdown: &str,
    ink: Color,
    font_family: &str,
    font_size: f32,
    line_height: f32,
) -> TextLayout {
    let families: Vec<FamilyOwned> = FamilyOwned::parse_list(font_family).collect();

    // Walk the markdown event stream, accumulate flat text + span metadata.
    let mut text = String::new();
    // Each span: (byte_start, byte_end, is_bold, is_italic, size_override)
    let mut spans: Vec<(usize, usize, bool, bool, Option<f32>)> = Vec::new();

    let mut bold_depth: u32 = 0;
    let mut italic_depth: u32 = 0;
    let mut heading_level: u32 = 0;
    let mut span_start: Option<(usize, bool, bool, Option<f32>)> = None;

    // Flush the in-progress span up to `pos`, pushing it to `spans`.
    let flush = |pos: usize,
                 span_start: &mut Option<(usize, bool, bool, Option<f32>)>,
                 spans: &mut Vec<(usize, usize, bool, bool, Option<f32>)>| {
        if let Some((start, b, i, sz)) = span_start.take() {
            if pos > start {
                spans.push((start, pos, b, i, sz));
            }
        }
    };

    // Current style snapshot for starting a new span.
    let heading_sz =
        |hl: u32| -> Option<f32> { (hl > 0).then(|| heading_font_size(hl, font_size)) };

    for event in Parser::new_ext(markdown, Options::empty()) {
        match event {
            MdEvent::Start(Tag::Strong) => {
                flush(text.len(), &mut span_start, &mut spans);
                bold_depth += 1;
                span_start = Some((
                    text.len(),
                    bold_depth > 0,
                    italic_depth > 0,
                    heading_sz(heading_level),
                ));
            }
            MdEvent::End(TagEnd::Strong) => {
                flush(text.len(), &mut span_start, &mut spans);
                bold_depth = bold_depth.saturating_sub(1);
                span_start = Some((
                    text.len(),
                    bold_depth > 0,
                    italic_depth > 0,
                    heading_sz(heading_level),
                ));
            }
            MdEvent::Start(Tag::Emphasis) => {
                flush(text.len(), &mut span_start, &mut spans);
                italic_depth += 1;
                span_start = Some((
                    text.len(),
                    bold_depth > 0,
                    italic_depth > 0,
                    heading_sz(heading_level),
                ));
            }
            MdEvent::End(TagEnd::Emphasis) => {
                flush(text.len(), &mut span_start, &mut spans);
                italic_depth = italic_depth.saturating_sub(1);
                span_start = Some((
                    text.len(),
                    bold_depth > 0,
                    italic_depth > 0,
                    heading_sz(heading_level),
                ));
            }
            MdEvent::Start(Tag::Heading { level, .. }) => {
                flush(text.len(), &mut span_start, &mut spans);
                if !text.is_empty() {
                    text.push('\n');
                }
                heading_level = level as u32;
                span_start = Some((
                    text.len(),
                    true,
                    false,
                    Some(heading_font_size(heading_level, font_size)),
                ));
            }
            MdEvent::End(TagEnd::Heading(_)) => {
                flush(text.len(), &mut span_start, &mut spans);
                text.push('\n');
                heading_level = 0;
                span_start = Some((text.len(), false, false, None));
            }
            MdEvent::Start(Tag::Paragraph) => {
                flush(text.len(), &mut span_start, &mut spans);
                if !text.is_empty() {
                    text.push('\n');
                }
                span_start = Some((text.len(), false, false, None));
            }
            MdEvent::End(TagEnd::Paragraph) => {
                flush(text.len(), &mut span_start, &mut spans);
                text.push('\n');
                span_start = Some((text.len(), false, false, None));
            }
            MdEvent::Text(t) | MdEvent::Code(t) => text.push_str(&t),
            MdEvent::SoftBreak => text.push(' '),
            MdEvent::HardBreak => text.push('\n'),
            _ => {}
        }
    }

    // Flush any remaining span.
    flush(text.len(), &mut span_start, &mut spans);

    // Build AttrsList
    let lh = LineHeightValue::Normal(line_height);
    let default_attrs = Attrs::new()
        .color(ink)
        .family(&families)
        .font_size(font_size)
        .line_height(lh);
    let mut attrs_list = AttrsList::new(default_attrs);

    for (start, end, is_bold, is_italic, size_override) in spans {
        if start >= end {
            continue;
        }
        let sz = size_override.unwrap_or(font_size);
        let mut span_attrs = Attrs::new()
            .color(ink)
            .family(&families)
            .font_size(sz)
            .line_height(lh);
        if is_bold {
            span_attrs = span_attrs.weight(Weight::BOLD);
        }
        if is_italic {
            span_attrs = span_attrs.style(TextStyle::Italic);
        }
        attrs_list.add_span(start..end, span_attrs);
    }

    let mut layout = TextLayout::new();
    layout.set_text(&text, attrs_list);
    layout
}

/// Font size for headings: H1 = base * 1.6, H2 = base * 1.35, H3+ = base * 1.15
fn heading_font_size(level: u32, base: f32) -> f32 {
    match level {
        1 => base * 1.6,
        2 => base * 1.35,
        _ => base * 1.15,
    }
}

/// Send a `ChooseAction` command, drain events, and if the scene finished,
/// ask the scheduler to pick the next scene and start it.
fn dispatch_action(action_id: String, state: &Rc<RefCell<GameState>>, signals: AppSignals) {
    let mut gs = state.borrow_mut();
    let GameState {
        ref mut engine,
        ref mut world,
        ref registry,
        ref scheduler,
        ref mut rng,
        ..
    } = *gs;
    if let Ok(femininity_id) = registry.resolve_skill("FEMININITY") {
        let events = engine.advance_with_action(&action_id, world, registry);
        let finished = crate::process_events(events, signals, world, femininity_id);
        if finished {
            if signals.phase.get_untracked() == crate::AppPhase::TransformationIntro {
                // Transformation intro complete — move to female customisation.
                // (The throwaway world is discarded; FemCreation builds the real one.)
                signals.phase.set(crate::AppPhase::FemCreation);
            } else if let Some(result) = scheduler.pick_next(world, registry, rng) {
                if result.once_only {
                    world
                        .game_data
                        .set_flag(format!("ONCE_{}", result.scene_id));
                }
                engine.send(EngineCommand::StartScene(result.scene_id), world, registry);
                let events = engine.drain();
                crate::process_events(events, signals, world, femininity_id);
            }
        }
    }
}

pub fn story_panel(signals: AppSignals, state: Rc<RefCell<GameState>>) -> impl View {
    let story = signals.story;
    let actions = signals.actions;
    let state_clone = Rc::clone(&state);
    let hovered_detail = create_rw_signal(String::new());
    let highlighted_idx: RwSignal<Option<usize>> = RwSignal::new(None);

    // Reset highlight whenever actions change (new scene step).
    let hi_reset = highlighted_idx;
    create_effect(move |_| {
        let _ = actions.get(); // reactive dependency
        hi_reset.set(None);
    });

    let keyboard_handler = move |e: &Event| -> bool {
        if let Event::KeyDown(key_event) = e {
            let mode = signals.prefs.get().number_key_mode;
            let key = &key_event.key.logical_key;

            // Arrow navigation (always active regardless of mode).
            if key == &Key::Named(NamedKey::ArrowDown) {
                let len = actions.get().len();
                if len > 0 {
                    highlighted_idx.update(|h| {
                        *h = Some(match *h {
                            None => 0,
                            Some(i) => (i + 1) % len,
                        });
                    });
                }
                return true;
            }
            if key == &Key::Named(NamedKey::ArrowUp) {
                let len = actions.get().len();
                if len > 0 {
                    highlighted_idx.update(|h| {
                        *h = Some(match *h {
                            None => len.saturating_sub(1),
                            Some(i) => {
                                if i == 0 {
                                    len - 1
                                } else {
                                    i - 1
                                }
                            }
                        });
                    });
                }
                return true;
            }

            // Enter: confirm highlighted choice.
            if key == &Key::Named(NamedKey::Enter) {
                if let Some(idx) = highlighted_idx.get() {
                    let current_actions = actions.get();
                    if idx < current_actions.len() {
                        let action_id = current_actions[idx].id.clone();
                        drop(current_actions);
                        dispatch_action(action_id, &state_clone, signals);
                        return true;
                    }
                }
            }

            // Escape: clear highlight.
            if key == &Key::Named(NamedKey::Escape) {
                highlighted_idx.set(None);
                return true;
            }

            // Number keys 1–9.
            if let Key::Character(char_str) = key {
                if let Ok(num) = char_str.parse::<usize>() {
                    if num > 0 && num <= 9 {
                        let idx = num - 1;
                        let current_actions = actions.get();
                        if idx < current_actions.len() {
                            match mode {
                                NumberKeyMode::Instant => {
                                    let action_id = current_actions[idx].id.clone();
                                    drop(current_actions);
                                    dispatch_action(action_id, &state_clone, signals);
                                    return true;
                                }
                                NumberKeyMode::Confirm => {
                                    if highlighted_idx.get() == Some(idx) {
                                        // Already highlighted — confirm.
                                        let action_id = current_actions[idx].id.clone();
                                        drop(current_actions);
                                        dispatch_action(action_id, &state_clone, signals);
                                    } else {
                                        // Not highlighted — highlight it.
                                        highlighted_idx.set(Some(idx));
                                    }
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    };

    let scroll_gen = signals.scroll_gen;

    let prose_label = rich_text(move || {
        let prefs = signals.prefs.get();
        let colors = ThemeColors::from_mode(prefs.mode);
        markdown_to_text_layout(
            &story.get(),
            colors.ink,
            &prefs.font_family,
            prefs.font_size as f32,
            prefs.line_height,
        )
    })
    .style(|s| s.padding(24.0).max_width(680.0));

    let centered_prose = container(prose_label)
        .style(|s| s.width_full().flex_row().justify_center().padding_top(16.0));

    let scroll_area = scroll(centered_prose)
        .scroll_to(move || {
            let gen = scroll_gen.get();
            if gen > 0 {
                Some(floem::kurbo::Point::new(0.0, f64::MAX))
            } else {
                None
            }
        })
        .scroll_style(|s| s.shrink_to_fit())
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.flex_grow(1.0).flex_basis(0.0).background(colors.page)
        });

    let detail_strip = label(move || {
        // Show highlighted detail when a choice is keyboard-highlighted,
        // otherwise fall back to hovered detail.
        if let Some(idx) = highlighted_idx.get() {
            let acts = actions.get();
            if idx < acts.len() {
                return acts[idx].detail.clone();
            }
        }
        hovered_detail.get()
    })
    .style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.width_full()
            .min_height(28.0)
            .max_width(680.0)
            .padding_horiz(24.0)
            .padding_vert(6.0)
            .font_size(13.0)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
            .color(colors.ink_ghost)
            .background(colors.page)
            .hover(move |s| s.background(colors.page))
            .focus(move |s| s.background(colors.page))
    });

    // Wrap detail strip in a centering container to align with prose column
    let detail_strip = container(detail_strip).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.width_full()
            .flex_row()
            .justify_center()
            .background(colors.page)
            .border_top(1.0)
            .border_color(colors.seam)
    });

    v_stack((
        scroll_area,
        detail_strip,
        choices_bar(signals, state, hovered_detail, highlighted_idx),
    ))
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
    highlighted_idx: RwSignal<Option<usize>>,
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
                dispatch_action(action_id.clone(), &state_clone, signals_clone);
            };

            let exec_action_click = exec_action.clone();
            let exec_action_key = exec_action;
            let hovered = create_rw_signal(false);
            let is_highlighted = move || highlighted_idx.get() == Some(index);

            h_stack((
                label(move || format!("{}·", index + 1)).style(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    let ink = if hovered.get() || is_highlighted() {
                        colors.ink_dim
                    } else {
                        colors.ink_ghost
                    };
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
                let highlighted = is_highlighted();
                s.margin(4.0)
                    .padding_horiz(20.0)
                    .padding_vert(12.0)
                    .min_height(48.0)
                    .border(1.0)
                    .border_color(if highlighted {
                        colors.lamp
                    } else {
                        colors.seam
                    })
                    .border_radius(4.0)
                    .background(if highlighted {
                        colors.lamp_glow
                    } else {
                        Color::TRANSPARENT
                    })
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
    .style(|s| s.flex_row().flex_wrap(FlexWrap::Wrap).max_width(680.0));

    container(buttons).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.padding(12.0)
            .border_top(1.0)
            .border_color(colors.seam)
            .min_height(64.0)
            .width_full()
            .flex_row()
            .justify_center()
            .background(colors.page)
    })
}
