use crate::game_state::GameState;
use crate::layout::{
    story_panel_max_height, story_region_width_for_window, ACTION_BUTTON_MIN_WIDTH,
};
use crate::runtime_controller::RuntimeController;
use crate::signal_utils::get_or_default;
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

fn action_feedback_reset_generation(
    scene_epoch: u64,
    actions: &[ActionView],
) -> (u64, Vec<String>) {
    (
        scene_epoch,
        actions.iter().map(|action| action.id.clone()).collect(),
    )
}

fn reset_action_feedback_state(
    hovered_detail: floem::reactive::RwSignal<String>,
    highlighted_idx: RwSignal<Option<usize>>,
) {
    highlighted_idx.set(None);
    hovered_detail.set(String::new());
}

#[cfg(test)]
fn centered_action_hitbox_contains(bar_width: f64, control_width: f64, point_x: f64) -> bool {
    if bar_width <= 0.0 || point_x < 0.0 || point_x > bar_width {
        return false;
    }

    let visible_width = control_width.clamp(0.0, bar_width);
    let inset = (bar_width - visible_width) / 2.0;
    point_x >= inset && point_x <= inset + visible_width
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
        let reset_generation =
            action_feedback_reset_generation(signals.scene_epoch.get(), &actions.get());
        let _ = reset_generation;
        reset_action_feedback_state(detail_reset, hi_reset);
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

    // Reactive max_height for the scroll area. Floem's taffy integration
    // hardcodes overflow:visible on every node (to_taffy_style uses
    // ..Default::default()), which prevents flex_grow+flex_basis(0) from
    // shrinking the scroll below its content height. max_height is the one
    // constraint that taffy reliably respects. We compute it from the
    // live window metrics so the layout survives resizes and scene changes.
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
            let max_h = story_panel_max_height(
                signals.window_width.get(),
                signals.window_height.get(),
                actions.get().len(),
            );
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
        .style(|s| s.flex_grow(1.0).min_height(0.0).height_full())
}

fn continue_button(signals: AppSignals, state: Rc<RefCell<GameState>>) -> impl View {
    let btn = label(move || "Continue".to_string())
        .on_click_stop(move |_| {
            continue_to_next_scene(&state, signals);
        })
        .style(move |s| {
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

    container(btn).style(move |s| {
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
                    .min_width(ACTION_BUTTON_MIN_WIDTH as f32)
                    .flex_grow(1.0)
                    .flex_basis(ACTION_BUTTON_MIN_WIDTH as f32)
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
    .style(move |s| {
        let max_width = story_region_width_for_window(signals.window_width.get()) as f32;
        s.flex_row()
            .flex_wrap(FlexWrap::Wrap)
            .width_full()
            .max_width(max_width)
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

#[cfg(test)]
mod tests {
    use super::{
        action_feedback_reset_generation, centered_action_hitbox_contains, markdown_to_text_layout,
    };
    use crate::layout::{
        action_button_columns_for_window, action_button_rows_for_window, sidebar_width_for_window,
        story_region_width_for_window, ACTION_BUTTON_MIN_WIDTH,
    };
    use floem::peniko::Color;
    use floem::text::Weight;
    use undone_scene::engine::ActionView;

    fn markdown_layout_lines(markdown: &str) -> Vec<String> {
        markdown_to_text_layout(markdown, Color::rgb8(0, 0, 0), "system-ui", 16.0, 1.4)
            .lines()
            .iter()
            .map(|line| line.text().to_string())
            .collect()
    }

    fn markdown_layout_line_heights(markdown: &str) -> Vec<f32> {
        let layout =
            markdown_to_text_layout(markdown, Color::rgb8(0, 0, 0), "system-ui", 16.0, 1.4);
        let mut heights = Vec::new();
        let mut last_line = None;
        for run in layout.layout_runs() {
            if last_line != Some(run.line_i) {
                heights.push(run.line_height);
                last_line = Some(run.line_i);
            }
        }
        heights
    }

    #[test]
    fn sidebar_width_shrinks_before_story_column_becomes_unusable() {
        assert_eq!(sidebar_width_for_window(1200.0), 280.0);
        assert_eq!(sidebar_width_for_window(920.0), 220.0);
        assert_eq!(sidebar_width_for_window(650.0), 180.0);
    }

    #[test]
    fn action_button_columns_collapse_as_window_narrows() {
        assert_eq!(action_button_columns_for_window(1200.0), 3);
        assert_eq!(action_button_columns_for_window(900.0), 2);
        assert_eq!(action_button_columns_for_window(650.0), 1);
    }

    #[test]
    fn action_button_rows_follow_the_responsive_column_count() {
        assert_eq!(action_button_rows_for_window(1200.0, 5), 2);
        assert_eq!(action_button_rows_for_window(900.0, 5), 3);
        assert_eq!(action_button_rows_for_window(650.0, 2), 2);
    }

    #[test]
    fn continue_hitbox_is_limited_to_visible_chrome() {
        assert!(centered_action_hitbox_contains(480.0, 132.0, 240.0));
        assert!(!centered_action_hitbox_contains(480.0, 132.0, 48.0));
        assert!(!centered_action_hitbox_contains(480.0, 132.0, 432.0));
    }

    #[test]
    fn action_bar_dead_space_is_not_owned_by_centered_controls() {
        assert!(centered_action_hitbox_contains(960.0, 720.0, 480.0));
        assert!(!centered_action_hitbox_contains(960.0, 720.0, 80.0));
        assert!(!centered_action_hitbox_contains(960.0, 720.0, 920.0));
    }

    #[test]
    fn action_button_columns_account_for_action_bar_padding() {
        assert_eq!(action_button_columns_for_window(1200.0), 3);
        assert_eq!(action_button_columns_for_window(900.0), 2);
        assert_eq!(action_button_columns_for_window(760.0), 1);
    }

    #[test]
    fn action_button_rows_recompute_after_width_changes() {
        assert_eq!(action_button_rows_for_window(1200.0, 4), 2);
        assert_eq!(action_button_rows_for_window(760.0, 4), 4);
        assert_eq!(action_button_rows_for_window(1600.0, 7), 2);
    }

    #[test]
    fn story_region_width_stays_usable_and_grows_on_wide_windows() {
        assert_eq!(
            story_region_width_for_window(320.0),
            ACTION_BUTTON_MIN_WIDTH
        );
        assert!(story_region_width_for_window(1600.0) > story_region_width_for_window(1200.0));
    }

    #[test]
    fn markdown_to_text_layout_preserves_paragraph_breaks() {
        let lines = markdown_layout_lines("First paragraph.\n\nSecond paragraph.");

        assert_eq!(lines, vec!["First paragraph.", "", "Second paragraph."]);
    }

    #[test]
    fn markdown_to_text_layout_makes_headings_larger_than_body_copy() {
        let layout = markdown_to_text_layout(
            "# Heading\n\nBody copy.",
            Color::rgb8(0, 0, 0),
            "system-ui",
            16.0,
            1.4,
        );
        let heading_attrs = layout.lines()[0].attrs_list().get_span(0);
        let line_heights = markdown_layout_line_heights("# Heading\n\nBody copy.");

        assert_eq!(heading_attrs.weight, Weight::BOLD);
        assert!(line_heights[0] > line_heights[2]);
    }

    #[test]
    fn markdown_to_text_layout_renders_horizontal_rules_as_separator_lines() {
        let lines = markdown_layout_lines("Before\n\n---\n\nAfter");

        assert_eq!(lines, vec!["Before", "", "——————————", "", "After"]);
    }

    #[test]
    fn markdown_to_text_layout_keeps_soft_and_hard_breaks_readable() {
        let lines = markdown_layout_lines("Soft\nbreak\n\nHard\\\nbreak");

        assert_eq!(lines, vec!["Soft break", "", "Hard", "break"]);
    }

    #[test]
    fn action_feedback_reset_generation_changes_when_scene_epoch_changes() {
        let repeated_actions = vec![ActionView {
            id: "wait".into(),
            label: "Wait".into(),
            detail: "Hold steady.".into(),
        }];

        assert_ne!(
            action_feedback_reset_generation(1, &repeated_actions),
            action_feedback_reset_generation(2, &repeated_actions)
        );
    }

    #[test]
    fn action_feedback_reset_generation_changes_when_action_ids_change() {
        let first_actions = vec![ActionView {
            id: "wait".into(),
            label: "Wait".into(),
            detail: "Hold steady.".into(),
        }];
        let second_actions = vec![ActionView {
            id: "leave".into(),
            label: "Leave".into(),
            detail: "Move on.".into(),
        }];

        assert_ne!(
            action_feedback_reset_generation(3, &first_actions),
            action_feedback_reset_generation(3, &second_actions)
        );
    }
}
