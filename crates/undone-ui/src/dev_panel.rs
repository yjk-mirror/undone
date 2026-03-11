use std::cell::RefCell;
use std::rc::Rc;

use floem::prelude::*;
use floem::reactive::{create_effect, RwSignal};
use floem::views::dyn_stack;

use crate::dev_ipc::{execute_command, game_state_snapshot, runtime_state_snapshot, DevCommand};
use crate::game_state::GameState;
use crate::layout::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH};
use crate::runtime_snapshot::RuntimeSnapshot;
use crate::signal_utils::get_or_default;
use crate::theme::ThemeColors;
use crate::AppSignals;

const WINDOW_SIZE_PRESETS: [(f64, f64); 3] = [(1200.0, 800.0), (1400.0, 900.0), (1800.0, 1000.0)];

/// Bundles the shared state that every dev panel widget needs.
#[derive(Clone)]
struct DevContext {
    gs: Rc<RefCell<GameState>>,
    signals: AppSignals,
    status: RwSignal<String>,
    money: RwSignal<String>,
    stress: RwSignal<String>,
    anxiety: RwSignal<String>,
    femininity: RwSignal<String>,
    window_width: RwSignal<String>,
    window_height: RwSignal<String>,
}

impl DevContext {
    fn run(&self, command: DevCommand) {
        let response = {
            let mut gs_ref = self.gs.borrow_mut();
            execute_command(&mut gs_ref, self.signals, command)
        };
        if response.success {
            self.signals.dev_tick.update(|tick| *tick += 1);
        }
        self.status.set(response.message);
        self.sync_inputs();
    }

    fn sync_inputs(&self) {
        let snapshot = {
            let gs_ref = self.gs.borrow();
            game_state_snapshot(&gs_ref)
        };
        self.money.set(snapshot.money.to_string());
        self.stress.set(snapshot.stress.to_string());
        self.anxiety.set(snapshot.anxiety.to_string());
        self.femininity.set(snapshot.femininity.to_string());
        self.window_width
            .set(self.signals.window_width.get_untracked().to_string());
        self.window_height
            .set(self.signals.window_height.get_untracked().to_string());
    }
}

pub fn dev_panel(signals: AppSignals, gs: Rc<RefCell<GameState>>) -> impl View {
    let snapshot = {
        let gs_ref = gs.borrow();
        game_state_snapshot(&gs_ref)
    };

    let ctx = DevContext {
        gs,
        signals,
        status: RwSignal::new(String::new()),
        money: RwSignal::new(snapshot.money.to_string()),
        stress: RwSignal::new(snapshot.stress.to_string()),
        anxiety: RwSignal::new(snapshot.anxiety.to_string()),
        femininity: RwSignal::new(snapshot.femininity.to_string()),
        window_width: RwSignal::new(signals.window_width.get_untracked().to_string()),
        window_height: RwSignal::new(signals.window_height.get_untracked().to_string()),
    };

    let filter = RwSignal::new(String::new());
    let flag_input = RwSignal::new(String::new());

    create_effect({
        let window_width = ctx.window_width;
        let window_height = ctx.window_height;
        move |_| {
            let width_text = signals.window_width.get().to_string();
            let height_text = signals.window_height.get().to_string();

            if window_width.get_untracked() != width_text {
                window_width.set(width_text);
            }
            if window_height.get_untracked() != height_text {
                window_height.set(height_text);
            }
        }
    });

    let scene_section = section_card(
        "Scene Jumper",
        v_stack((
            text_input(filter)
                .placeholder("Filter scenes")
                .style(input_style(signals)),
            scroll(
                dyn_stack(
                    {
                        let ctx = ctx.clone();
                        move || {
                            let _ = ctx.signals.dev_tick.get();
                            let ids = {
                                let gs_ref = ctx.gs.borrow();
                                gs_ref.engine.scene_ids()
                            };
                            filter_scene_ids(ids, &get_or_default(filter))
                        }
                    },
                    |scene_id: &String| scene_id.clone(),
                    {
                        let ctx = ctx.clone();
                        move |scene_id: String| {
                            let click_scene = scene_id.clone();
                            label(move || scene_id.clone())
                                .on_click_stop({
                                    let ctx = ctx.clone();
                                    move |_| {
                                        ctx.run(DevCommand::JumpToScene {
                                            scene_id: click_scene.clone(),
                                        });
                                    }
                                })
                                .style(move |s| {
                                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                                    s.width_full()
                                        .padding_horiz(12.0)
                                        .padding_vert(8.0)
                                        .border_bottom(1.0)
                                        .border_color(colors.seam)
                                        .color(colors.ink)
                                        .font_family(
                                            "system-ui, -apple-system, sans-serif".to_string(),
                                        )
                                        .hover(|s| s.background(colors.lamp_glow))
                                })
                        }
                    },
                )
                .style(|s| s.width_full().flex_col()),
            )
            .scroll_style(|s| s.shrink_to_fit())
            .style(|s| s.width_full().max_height(260.0)),
        )),
        signals,
    );

    let stats_section = section_card(
        "Stat Editors",
        v_stack((
            stat_editor_row("Money", ctx.money, "money", ctx.clone()),
            stat_editor_row("Stress", ctx.stress, "stress", ctx.clone()),
            stat_editor_row("Anxiety", ctx.anxiety, "anxiety", ctx.clone()),
            stat_editor_row("Femininity", ctx.femininity, "femininity", ctx.clone()),
        )),
        signals,
    );

    let flag_section = section_card(
        "Flags",
        v_stack((
            h_stack((
                text_input(flag_input)
                    .placeholder("GAME_FLAG")
                    .style(input_style(signals)),
                action_button("Set", signals, {
                    let ctx = ctx.clone();
                    move || {
                        ctx.run(DevCommand::SetFlag {
                            flag: flag_input.get_untracked(),
                        });
                    }
                }),
                action_button("Remove", signals, {
                    let ctx = ctx.clone();
                    move || {
                        ctx.run(DevCommand::RemoveFlag {
                            flag: flag_input.get_untracked(),
                        });
                    }
                }),
            ))
            .style(|s| s.gap(8.0).items_center()),
            dyn_stack(
                {
                    let ctx = ctx.clone();
                    move || {
                        let _ = ctx.signals.dev_tick.get();
                        let gs_ref = ctx.gs.borrow();
                        game_state_snapshot(&gs_ref).game_flags
                    }
                },
                |flag: &String| flag.clone(),
                move |flag: String| {
                    label(move || flag.clone()).style(move |s| {
                        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                        s.padding_horiz(8.0)
                            .padding_vert(4.0)
                            .margin_right(6.0)
                            .margin_bottom(6.0)
                            .border(1.0)
                            .border_color(colors.seam)
                            .border_radius(4.0)
                            .color(colors.ink_dim)
                            .font_size(12.0)
                            .font_family("system-ui, -apple-system, sans-serif".to_string())
                    })
                },
            )
            .style(|s| s.flex_row().flex_wrap(floem::style::FlexWrap::Wrap)),
        )),
        signals,
    );

    let quick_section = section_card(
        "Quick Actions",
        h_stack((
            action_button("Advance 1 Week", signals, {
                let ctx = ctx.clone();
                move || {
                    ctx.run(DevCommand::AdvanceTime { weeks: 1 });
                }
            }),
            action_button("All NPC → Close", signals, {
                let ctx = ctx.clone();
                move || {
                    ctx.run(DevCommand::SetAllNpcLiking {
                        level: "Close".to_string(),
                    });
                }
            }),
        ))
        .style(|s| s.gap(8.0).flex_wrap(floem::style::FlexWrap::Wrap)),
        signals,
    );

    let window_section = section_card(
        "Window",
        v_stack((
            label(move || {
                format!(
                    "Current content size: {:.0}x{:.0}",
                    signals.window_width.get(),
                    signals.window_height.get()
                )
            })
            .style(move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                s.color(colors.ink_dim)
                    .font_size(13.0)
                    .font_family("system-ui, -apple-system, sans-serif".to_string())
            }),
            h_stack((
                text_input(ctx.window_width)
                    .placeholder("Width")
                    .style(input_style(signals)),
                text_input(ctx.window_height)
                    .placeholder("Height")
                    .style(input_style(signals)),
                action_button("Apply Size", signals, {
                    let ctx = ctx.clone();
                    move || match parse_window_size_inputs(
                        &ctx.window_width.get_untracked(),
                        &ctx.window_height.get_untracked(),
                    ) {
                        Some((width, height)) => {
                            ctx.run(DevCommand::SetWindowSize { width, height })
                        }
                        None => ctx
                            .status
                            .set("Window size must be positive numbers".to_string()),
                    }
                }),
                action_button("Default", signals, {
                    let ctx = ctx.clone();
                    move || {
                        let (width, height) = default_window_size();
                        ctx.window_width.set(width.to_string());
                        ctx.window_height.set(height.to_string());
                        ctx.run(DevCommand::SetWindowSize { width, height });
                    }
                }),
            ))
            .style(|s| s.gap(8.0).flex_wrap(floem::style::FlexWrap::Wrap)),
            h_stack((
                window_size_button(
                    WINDOW_SIZE_PRESETS[0].0,
                    WINDOW_SIZE_PRESETS[0].1,
                    ctx.clone(),
                    signals,
                ),
                window_size_button(
                    WINDOW_SIZE_PRESETS[1].0,
                    WINDOW_SIZE_PRESETS[1].1,
                    ctx.clone(),
                    signals,
                ),
                window_size_button(
                    WINDOW_SIZE_PRESETS[2].0,
                    WINDOW_SIZE_PRESETS[2].1,
                    ctx.clone(),
                    signals,
                ),
            ))
            .style(|s| s.gap(8.0).flex_wrap(floem::style::FlexWrap::Wrap)),
        )),
        signals,
    );

    let inspector_section = section_card(
        "State Inspector",
        scroll(
            label({
                let ctx = ctx.clone();
                move || {
                    let _ = ctx.signals.dev_tick.get();
                    let snapshot = {
                        let gs_ref = ctx.gs.borrow();
                        runtime_state_snapshot(&gs_ref, ctx.signals)
                    };
                    format_runtime_snapshot_json(&snapshot)
                }
            })
            .style(move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                s.width_full()
                    .color(colors.ink)
                    .font_size(12.0)
                    .font_family("Consolas, Menlo, monospace".to_string())
            }),
        )
        .scroll_style(|s| s.shrink_to_fit())
        .style(|s| s.width_full().max_height(280.0)),
        signals,
    );

    let status = ctx.status;
    scroll(v_stack((
        heading("Dev Tools", signals),
        label(move || get_or_default(status)).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.min_height(18.0)
                .font_size(13.0)
                .color(colors.ink_dim)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
        scene_section,
        stats_section,
        flag_section,
        quick_section,
        window_section,
        inspector_section,
    )))
    .scroll_style(|s| s.shrink_to_fit())
    .style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.size_full()
            .padding(20.0)
            .gap(16.0)
            .background(colors.page)
    })
}

fn filter_scene_ids(scene_ids: Vec<String>, filter_text: &str) -> Vec<String> {
    if filter_text.trim().is_empty() {
        return scene_ids;
    }

    let needle = filter_text.to_lowercase();
    scene_ids
        .into_iter()
        .filter(|scene_id| scene_id.to_lowercase().contains(&needle))
        .collect()
}

fn format_runtime_snapshot_json(snapshot: &RuntimeSnapshot) -> String {
    serde_json::to_string_pretty(snapshot).unwrap_or_else(|err| format!("{{\"error\":\"{err}\"}}"))
}

fn parse_window_size_inputs(width: &str, height: &str) -> Option<(f64, f64)> {
    let width = width.trim().parse::<f64>().ok()?;
    let height = height.trim().parse::<f64>().ok()?;

    if !(width.is_finite() && height.is_finite()) || width <= 0.0 || height <= 0.0 {
        return None;
    }

    Some((width, height))
}

fn default_window_size() -> (f64, f64) {
    (DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
}

fn window_size_button(width: f64, height: f64, ctx: DevContext, signals: AppSignals) -> impl View {
    let label_text = format!("{width:.0}x{height:.0}");
    action_button_owned(label_text, signals, move || {
        ctx.run(DevCommand::SetWindowSize { width, height });
    })
}

fn stat_editor_row(
    label_text: &'static str,
    input: RwSignal<String>,
    stat_key: &'static str,
    ctx: DevContext,
) -> impl View {
    let signals = ctx.signals;
    h_stack((
        label(move || label_text.to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.width(100.0)
                .color(colors.ink_dim)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
        text_input(input).style(input_style(signals)),
        action_button("Apply", signals, move || {
            match input.get_untracked().trim().parse::<i32>() {
                Ok(value) => ctx.run(DevCommand::SetStat {
                    stat: stat_key.to_string(),
                    value,
                }),
                Err(err) => ctx.status.set(format!("Invalid {label_text} value: {err}")),
            }
        }),
    ))
    .style(|s| s.gap(8.0).items_center().margin_bottom(10.0))
}

fn section_card(title: &'static str, content: impl IntoView, signals: AppSignals) -> impl View {
    v_stack((
        label(move || title.to_string()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(14.0)
                .font_weight(floem::text::Weight::SEMIBOLD)
                .color(colors.ink)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
        content.into_view(),
    ))
    .style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.width_full()
            .padding(16.0)
            .gap(12.0)
            .border(1.0)
            .border_radius(6.0)
            .border_color(colors.seam)
            .background(colors.page_raised)
    })
}

fn heading(text: &'static str, signals: AppSignals) -> impl View {
    label(move || text.to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.font_size(22.0)
            .font_weight(floem::text::Weight::LIGHT)
            .color(colors.ink)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
    })
}

fn input_style(signals: AppSignals) -> impl Fn(floem::style::Style) -> floem::style::Style {
    move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.height(32.0)
            .min_width(180.0)
            .padding_horiz(10.0)
            .font_size(14.0)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
            .color(colors.ink)
            .background(colors.page)
            .border(1.0)
            .border_color(colors.seam)
            .border_radius(4.0)
    }
}

fn action_button(
    text: &'static str,
    signals: AppSignals,
    on_click: impl Fn() + 'static,
) -> impl View {
    action_button_owned(text.to_string(), signals, on_click)
}

fn action_button_owned(
    text: String,
    signals: AppSignals,
    on_click: impl Fn() + 'static,
) -> impl View {
    label(move || text.clone())
        .keyboard_navigable()
        .on_click_stop(move |_| on_click())
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.padding_horiz(12.0)
                .padding_vert(8.0)
                .font_size(13.0)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
                .border(1.0)
                .border_color(colors.seam)
                .border_radius(4.0)
                .color(colors.ink)
                .background(colors.page)
                .hover(|s| s.border_color(colors.lamp).background(colors.lamp_glow))
        })
}

#[cfg(test)]
mod tests {
    use super::{
        default_window_size, filter_scene_ids, format_runtime_snapshot_json,
        parse_window_size_inputs, WINDOW_SIZE_PRESETS,
    };
    use crate::layout::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH};
    use crate::runtime_snapshot::{
        ActiveNpcSnapshot, ArcStateSnapshot, PlayerSummarySnapshot, RuntimeSnapshot,
        VisibleActionSnapshot, WorldSummarySnapshot,
    };

    #[test]
    fn filter_scene_ids_matches_case_insensitively() {
        let filtered = filter_scene_ids(
            vec![
                "base::coffee_shop".to_string(),
                "base::rain_shelter".to_string(),
            ],
            "COFFEE",
        );

        assert_eq!(filtered, vec!["base::coffee_shop".to_string()]);
    }

    #[test]
    fn format_runtime_snapshot_json_includes_runtime_fields() {
        let formatted = format_runtime_snapshot_json(&RuntimeSnapshot {
            phase: "in_game".into(),
            tab: "dev".into(),
            window_width: 1800.0,
            window_height: 1000.0,
            current_scene_id: Some("base::coffee_shop".into()),
            awaiting_continue: false,
            story_paragraphs: vec!["Hello".into()],
            visible_actions: vec![VisibleActionSnapshot {
                id: "wait".into(),
                label: "Wait".into(),
                detail: "Stay".into(),
            }],
            active_npc: Some(ActiveNpcSnapshot {
                name: "Jake".into(),
                age: "MidLateTwenties".into(),
                personality: "Romantic".into(),
                relationship: "Acquaintance".into(),
                pc_liking: "Like".into(),
                pc_attraction: "Attracted".into(),
                known: true,
            }),
            player: PlayerSummarySnapshot {
                name: "Robin".into(),
                femininity: 10,
                money: 500,
                stress: 0,
                anxiety: 0,
                arousal: "Comfort".into(),
                alcohol: "Sober".into(),
            },
            world: WorldSummarySnapshot {
                week: 0,
                day: 0,
                time_slot: "Morning".into(),
                game_flags: vec!["ROUTE_WORKPLACE".into()],
                arc_states: vec![ArcStateSnapshot {
                    id: "base::workplace_opening".into(),
                    state: "arrived".into(),
                }],
            },
            init_error: None,
        });

        assert!(formatted.contains("\"story_paragraphs\""));
        assert!(formatted.contains("\"visible_actions\""));
        assert!(formatted.contains("\"current_scene_id\""));
    }

    #[test]
    fn window_size_presets_include_wide_layout_target() {
        assert!(WINDOW_SIZE_PRESETS.contains(&(1800.0, 1000.0)));
    }

    #[test]
    fn parse_window_size_inputs_accepts_positive_numbers() {
        assert_eq!(
            parse_window_size_inputs("1800", "1000"),
            Some((1800.0, 1000.0))
        );
    }

    #[test]
    fn parse_window_size_inputs_rejects_empty_or_non_positive_values() {
        assert_eq!(parse_window_size_inputs("", "1000"), None);
        assert_eq!(parse_window_size_inputs("0", "1000"), None);
        assert_eq!(parse_window_size_inputs("1800", "-1"), None);
    }

    #[test]
    fn default_window_size_matches_shared_layout_defaults() {
        assert_eq!(
            default_window_size(),
            (DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
        );
    }
}
