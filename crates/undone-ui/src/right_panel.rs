use crate::layout::sidebar_width_for_window;
use crate::theme::{ThemeColors, ThemeMode};
use crate::{AppSignals, NpcSnapshot, PlayerSnapshot};
use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::style::FlexWrap;
use undone_domain::{AttractionLevel, LikingLevel};

pub fn sidebar_panel(signals: AppSignals) -> impl View {
    v_stack((
        stats_panel(signals.player, signals),
        people_panel(signals.active_npc, signals),
        mode_toggle(signals),
    ))
    .style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        let sidebar_width = sidebar_width_for_window(signals.window_width.get()) as f32;
        s.width(sidebar_width)
            .min_width(sidebar_width)
            .height_full()
            .background(colors.sidebar_ground)
            .border_right(1.0)
            .border_color(colors.seam)
    })
}

fn stat_row(
    label_text: &'static str,
    value_fn: impl Fn() -> String + 'static,
    signals: AppSignals,
) -> impl View {
    h_stack((
        label(move || label_text.to_uppercase()).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.flex_grow(1.0)
                .color(colors.ink_ghost)
                .font_size(12.0)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
        label(value_fn).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(13.0)
                .color(colors.ink)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
    ))
    .style(|s| s.height(28.0).items_center())
}

fn stats_panel(player: RwSignal<PlayerSnapshot>, signals: AppSignals) -> impl View {
    v_stack((
        label(move || player.get().name).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(18.0)
                .font_weight(floem::text::Weight::LIGHT)
                .color(colors.ink)
                .margin_bottom(16.0)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
        }),
        stat_row(
            "Femininity",
            move || player.get().femininity.to_string(),
            signals,
        ),
        stat_row("Money", move || format!("${}", player.get().money), signals),
        stat_row("Stress", move || player.get().stress.to_string(), signals),
        stat_row("Anxiety", move || player.get().anxiety.to_string(), signals),
        empty().style(|s| s.height(8.0)),
        stat_row("Arousal", move || player.get().arousal.clone(), signals),
        stat_row("Alcohol", move || player.get().alcohol.clone(), signals),
    ))
    .style(|s| s.padding(16.0))
}

fn people_panel(active_npc: RwSignal<Option<NpcSnapshot>>, signals: AppSignals) -> impl View {
    let title = label(|| "People Here".to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.font_size(12.0)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
            .color(colors.ink_ghost)
            .margin_bottom(8.0)
    });

    let content = dyn_view(move || {
        let Some(npc) = active_npc.get() else {
            return people_empty_state(signals).into_any();
        };
        if !npc.is_known() {
            return people_empty_state(signals).into_any();
        }

        let name = npc.name;
        let age = npc.age;
        let personality = npc.personality;
        let relationship = format!("{}", npc.relationship);
        let liking = liking_band(npc.pc_liking);
        let attraction = attraction_band(npc.pc_attraction);

        v_stack((
            label(move || name.clone()).style(move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                s.font_size(15.0)
                    .font_weight(floem::text::Weight::MEDIUM)
                    .color(colors.ink)
                    .font_family("system-ui, -apple-system, sans-serif".to_string())
            }),
            label(move || format!("{} - {}", age, personality)).style(move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                s.font_size(12.0)
                    .color(colors.ink_dim)
                    .margin_top(2.0)
                    .font_family("system-ui, -apple-system, sans-serif".to_string())
            }),
            label(move || format!("Relationship: {}", relationship)).style(move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                s.font_size(12.0)
                    .color(colors.ink_dim)
                    .margin_top(8.0)
                    .font_family("system-ui, -apple-system, sans-serif".to_string())
            }),
            label(move || format!("Liking: {}", liking)).style(move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                s.font_size(12.0)
                    .color(colors.ink_dim)
                    .margin_top(2.0)
                    .font_family("system-ui, -apple-system, sans-serif".to_string())
            }),
            label(move || format!("Attraction: {}", attraction)).style(move |s| {
                let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                s.font_size(12.0)
                    .color(colors.ink_dim)
                    .margin_top(2.0)
                    .font_family("system-ui, -apple-system, sans-serif".to_string())
            }),
        ))
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.width_full()
                .padding(10.0)
                .border(1.0)
                .border_radius(4.0)
                .border_color(colors.seam)
                .background(colors.page_raised)
        })
        .into_any()
    });

    v_stack((title, content)).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.padding_horiz(16.0)
            .padding_vert(12.0)
            .border_top(1.0)
            .border_color(colors.seam)
            .width_full()
    })
}

fn people_empty_state(signals: AppSignals) -> impl View {
    label(|| "No one else is in focus yet.".to_string()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.font_size(12.0)
            .color(colors.ink_ghost)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
    })
}

fn liking_band(level: LikingLevel) -> &'static str {
    match level {
        LikingLevel::Neutral => "Neutral",
        LikingLevel::Ok => "Open",
        LikingLevel::Like => "Warm",
        LikingLevel::Close => "Very Warm",
    }
}

fn attraction_band(level: AttractionLevel) -> &'static str {
    match level {
        AttractionLevel::Unattracted => "Uninterested",
        AttractionLevel::Ok => "Curious",
        AttractionLevel::Attracted => "Interested",
        AttractionLevel::Lust => "Intense",
    }
}

fn mode_toggle(signals: AppSignals) -> impl View {
    let make_btn = move |text: &'static str, mode: ThemeMode| {
        label(move || text)
            .keyboard_navigable()
            .on_click_stop(move |_| {
                signals.prefs.update(|p| p.mode = mode);
                let prefs = signals.prefs.get_untracked();
                crate::theme::save_prefs(&prefs);
            })
            .style(move |s| {
                let prefs = signals.prefs.get();
                let colors = ThemeColors::from_mode(prefs.mode);
                let is_active = prefs.mode == mode;
                s.font_size(11.0)
                    .padding_horiz(10.0)
                    .padding_vert(5.0)
                    .border(1.0)
                    .border_radius(4.0)
                    .margin_right(6.0)
                    .font_family("system-ui, -apple-system, sans-serif".to_string())
                    .border_color(if is_active { colors.lamp } else { colors.seam })
                    .color(if is_active {
                        colors.lamp
                    } else {
                        colors.ink_ghost
                    })
                    .background(if is_active {
                        colors.lamp_glow
                    } else {
                        Color::TRANSPARENT
                    })
            })
    };

    h_stack((
        make_btn("Warm", ThemeMode::Light),
        make_btn("Sepia", ThemeMode::Sepia),
        make_btn("Night", ThemeMode::Dark),
    ))
    .style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.padding(12.0)
            .border_top(1.0)
            .border_color(colors.seam)
            .flex_wrap(FlexWrap::Wrap)
            .width_full()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn liking_band_mapping_is_player_readable() {
        assert_eq!(liking_band(LikingLevel::Neutral), "Neutral");
        assert_eq!(liking_band(LikingLevel::Ok), "Open");
        assert_eq!(liking_band(LikingLevel::Like), "Warm");
        assert_eq!(liking_band(LikingLevel::Close), "Very Warm");
    }

    #[test]
    fn attraction_band_mapping_is_player_readable() {
        assert_eq!(
            attraction_band(AttractionLevel::Unattracted),
            "Uninterested"
        );
        assert_eq!(attraction_band(AttractionLevel::Ok), "Curious");
        assert_eq!(attraction_band(AttractionLevel::Attracted), "Interested");
        assert_eq!(attraction_band(AttractionLevel::Lust), "Intense");
    }
}
