use crate::theme::ThemeColors;
use crate::{AppSignals, NpcSnapshot, PlayerSnapshot};
use floem::prelude::*;
use floem::reactive::RwSignal;

pub fn right_panel(signals: AppSignals) -> impl View {
    v_stack((
        stats_panel(signals.player, signals),
        npc_panel(signals.active_npc, signals),
    ))
    .style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.width(280.0)
            .height_full()
            .background(colors.sidebar_ground)
            .border_left(1.0)
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
            s.flex_grow(1.0).color(colors.ink_ghost).font_size(12.0)
        }),
        label(value_fn).style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(13.0).color(colors.ink)
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
        }),
        stat_row(
            "Femininity",
            move || player.get().femininity.to_string(),
            signals,
        ),
        stat_row("Money", move || format!("£{}", player.get().money), signals),
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
