use std::cell::RefCell;
use std::rc::Rc;

use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::dyn_stack;

use crate::dev_ipc::{execute_command, game_state_snapshot, DevCommand};
use crate::game_state::GameState;
use crate::theme::ThemeColors;
use crate::AppSignals;

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
}

impl DevContext {
    fn run(&self, command: DevCommand) {
        let response = {
            let mut gs_ref = self.gs.borrow_mut();
            execute_command(&mut gs_ref, self.signals, command)
        };
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
    };

    let filter = RwSignal::new(String::new());
    let flag_input = RwSignal::new(String::new());

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
                            filter_scene_ids(ids, &filter.get())
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

    let inspector_section = section_card(
        "State Inspector",
        scroll(
            label({
                let ctx = ctx.clone();
                move || {
                    let _ = ctx.signals.dev_tick.get();
                    let snapshot = {
                        let gs_ref = ctx.gs.borrow();
                        game_state_snapshot(&gs_ref)
                    };
                    serde_json::to_string_pretty(&snapshot)
                        .unwrap_or_else(|err| format!("{{\"error\":\"{err}\"}}"))
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
        label(move || status.get()).style(move |s| {
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
    label(move || text.to_string())
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
    use super::filter_scene_ids;

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
}
