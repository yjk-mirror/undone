use crate::game_state::GameState;
use crate::runtime_controller::RuntimeController;
use crate::signal_utils::{get_or, get_or_default};
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
use undone_scene::engine::ActionView;

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
            MdEvent::Rule => {
                // Thematic break (---): render as a visual separator line.
                flush(text.len(), &mut span_start, &mut spans);
                if !text.is_empty() {
                    text.push('\n');
                }
                text.push_str("——————————");
                text.push('\n');
                span_start = Some((text.len(), false, false, None));
            }
            MdEvent::Start(Tag::BlockQuote(_)) => {
                // Blockquote: just pass through; the inner paragraph
                // handles newlines and the bold/italic inside renders.
            }
            MdEvent::End(TagEnd::BlockQuote(_)) => {}
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
    let mut controller = RuntimeController::new(&mut gs, signals);
    let _ = controller.choose_action(&action_id);
}

/// Called when the player clicks "Continue" after reading action prose.
/// Picks the next scene from the scheduler and starts it.
fn continue_to_next_scene(state: &Rc<RefCell<GameState>>, signals: AppSignals) {
    let mut gs = state.borrow_mut();
    let mut controller = RuntimeController::new(&mut gs, signals);
    let _ = controller.continue_flow();
}

pub fn story_panel(signals: AppSignals, state: Rc<RefCell<GameState>>) -> impl View {
    let story = signals.story;
    let actions = signals.actions;
    let state_clone = Rc::clone(&state);
    let hovered_detail = create_rw_signal(String::new());
    let highlighted_idx: RwSignal<Option<usize>> = RwSignal::new(None);

    // Reset highlight and hovered detail whenever actions change (new scene step).
    let hi_reset = highlighted_idx;
    let detail_reset = hovered_detail;
    create_effect(move |_| {
        let _ = actions.get(); // reactive dependency
        hi_reset.set(None);
        detail_reset.set(String::new());
    });

    let state_for_continue = Rc::clone(&state);
    let keyboard_handler = move |e: &Event| -> bool {
        if let Event::KeyDown(key_event) = e {
            let mode = signals.prefs.get().number_key_mode;
            let key = &key_event.key.logical_key;

            // When awaiting continue, Enter/Space advances to next scene.
            // All other keys are consumed to prevent stale action navigation.
            if signals.awaiting_continue.get_untracked() {
                if key == &Key::Named(NamedKey::Enter) || key == &Key::Named(NamedKey::Space) {
                    continue_to_next_scene(&state_for_continue, signals);
                }
                return true;
            }

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
                if let Some(idx) = get_or_default(highlighted_idx) {
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
                                    if get_or_default(highlighted_idx) == Some(idx) {
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

    // Track actual window height via WindowResized events. Initialized to
    // the default window height (800) minus the custom title bar (40).
    let panel_height = create_rw_signal(760.0f64);

    // Reactive max_height for the scroll area. Floem's taffy integration
    // hardcodes overflow:visible on every node (to_taffy_style uses
    // ..Default::default()), which prevents flex_grow+flex_basis(0) from
    // shrinking the scroll below its content height. max_height is the one
    // constraint that taffy reliably respects. We compute it from the
    // actual panel height and the number of action buttons.
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
            // Estimate the bottom bar height from the action count.
            // Each button is ~56px (48 min-height + 8 margin). Use worst case
            // (1 button per row) — long labels can prevent wrapping 2-per-row.
            // Detail strip is ~41px.
            let n = actions.get().len().max(1) as f64;
            let choices_height = n * 56.0 + 25.0; // padding_vert(12)*2 + border(1)
            let detail_height = 41.0;
            let bottom_height = choices_height + detail_height;
            let available = get_or(panel_height, 760.0);
            let max_h = (available - bottom_height - 8.0).max(200.0);
            s.max_height(max_h as f32)
                .flex_grow(1.0)
                .flex_basis(0.0)
                .min_height(0.0)
                .background(colors.page)
        });

    let detail_strip = label(move || {
        // Show highlighted detail when a choice is keyboard-highlighted,
        // otherwise fall back to hovered detail.
        if let Some(idx) = get_or_default(highlighted_idx) {
            let acts = actions.get();
            if idx < acts.len() {
                return acts[idx].detail.clone();
            }
        }
        get_or_default(hovered_detail)
    })
    .style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.width_full()
            .min_height(40.0)
            .max_width(680.0)
            .padding_horiz(24.0)
            .padding_vert(10.0)
            .font_size(14.0)
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
            .flex_shrink(0.0)
            .justify_center()
            .background(colors.page)
            .border_top(1.0)
            .border_color(colors.seam)
    });

    let state_for_bar = Rc::clone(&state);
    let state_for_cont = Rc::clone(&state);
    let awaiting = signals.awaiting_continue;
    let action_bar = dyn_container(
        move || awaiting.get(),
        move |is_waiting| {
            if is_waiting {
                continue_button(signals, Rc::clone(&state_for_cont)).into_any()
            } else {
                choices_bar(
                    signals,
                    Rc::clone(&state_for_bar),
                    hovered_detail,
                    highlighted_idx,
                )
                .into_any()
            }
        },
    );

    v_stack((scroll_area, detail_strip, action_bar))
        .keyboard_navigable()
        .on_event_stop(EventListener::KeyDown, move |e| {
            keyboard_handler(e);
        })
        .on_event_cont(EventListener::WindowResized, move |e| {
            if let Event::WindowResized(size) = e {
                // Window height minus title bar (40px).
                panel_height.set(size.height - 40.0);
            }
        })
        .style(|s| s.flex_grow(1.0).min_height(0.0).height_full())
}

fn continue_button(signals: AppSignals, state: Rc<RefCell<GameState>>) -> impl View {
    let btn = label(move || "Continue".to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.padding_vert(12.0)
            .padding_horiz(24.0)
            .border_radius(4.0)
            .border(1.0)
            .border_color(colors.seam)
            .color(colors.ink)
            .font_size(15.0)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
            .cursor(floem::style::CursorStyle::Pointer)
            .hover(|s| s.background(colors.lamp_glow).border_color(colors.lamp))
    });

    container(btn)
        .on_click_stop(move |_| {
            continue_to_next_scene(&state, signals);
        })
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.width_full()
                .flex_row()
                .justify_center()
                .padding_vert(12.0)
                .min_height(64.0)
                .flex_shrink(0.0)
                .border_top(1.0)
                .border_color(colors.seam)
                .background(colors.page)
        })
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
            let is_highlighted = move || get_or_default(highlighted_idx) == Some(index);

            h_stack((
                label(move || format!("{}·", index + 1)).style(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    let ink = if get_or_default(hovered) || is_highlighted() {
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
    .style(|s| {
        s.flex_row()
            .flex_wrap(FlexWrap::Wrap)
            .max_width(680.0)
            .padding_horiz(24.0)
    });

    container(buttons).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.padding_vert(12.0)
            .border_top(1.0)
            .border_color(colors.seam)
            .min_height(64.0)
            .flex_shrink(0.0)
            .width_full()
            .flex_row()
            .justify_center()
            .background(colors.page)
    })
}
