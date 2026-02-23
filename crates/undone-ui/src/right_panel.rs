use crate::theme::{ThemeColors, ThemeMode};
use crate::{AppSignals, NpcSnapshot, PlayerSnapshot};
use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::RwSignal;

pub fn sidebar_panel(signals: AppSignals) -> impl View {
    v_stack((
        stats_panel(signals.player, signals),
        npc_panel(signals.active_npc, signals),
        mode_toggle(signals),
    ))
    .style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.width(280.0)
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
        // Signature Element — PC Name Display
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
        // State section gap
        empty().style(|s| s.height(8.0)),
        stat_row("Arousal", move || player.get().arousal.clone(), signals),
        stat_row("Alcohol", move || player.get().alcohol.clone(), signals),
    ))
    .style(|s| s.padding(16.0))
}

fn npc_panel(active_npc: RwSignal<Option<NpcSnapshot>>, signals: AppSignals) -> impl View {
    container(dyn_view(move || {
        if let Some(npc) = active_npc.get() {
            v_stack((
                label(move || format!("── {} ──", npc.name.clone())).style(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    s.font_weight(floem::text::Weight::BOLD)
                        .font_size(13.0)
                        .color(colors.ink)
                        .padding_bottom(8.0)
                        .items_center()
                        .width_full()
                        .font_family("system-ui, -apple-system, sans-serif".to_string())
                }),
                stat_row("Age", move || npc.age.clone(), signals),
                stat_row("Personality", move || npc.personality.clone(), signals),
                stat_row("Relationship", move || npc.relationship.clone(), signals),
                stat_row("Liking", move || npc.pc_liking.clone(), signals),
                stat_row("Attraction", move || npc.pc_attraction.clone(), signals),
            ))
            .into_any()
        } else {
            empty().into_any()
        }
    }))
    .style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.padding(16.0).border_top(1.0).border_color(colors.seam)
    })
}

fn mode_toggle(signals: AppSignals) -> impl View {
    let make_btn = move |text: &'static str, mode: ThemeMode| {
        label(move || text)
            .keyboard_navigable()
            .on_click_stop(move |_| {
                signals.prefs.update(|p| p.mode = mode);
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
                    .color(if is_active { colors.lamp } else { colors.ink_ghost })
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
        s.padding(12.0).border_top(1.0).border_color(colors.seam)
    })
}
