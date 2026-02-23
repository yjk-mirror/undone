use floem::prelude::*;
use floem::peniko::Color;
use floem::reactive::RwSignal;
use crate::{AppSignals, PlayerSnapshot, NpcSnapshot};

// Design Tokens (Warm Paper)
const SIDEBAR_GROUND: Color = Color::rgb8(237, 232, 220); // #EDE8DC
const INK: Color = Color::rgb8(28, 24, 20);           // #1C1814
const INK_GHOST: Color = Color::rgb8(140, 128, 120);     // #8C8078
const SEAM: Color = Color::rgba8(28, 24, 20, 25);        // #1C18141A (10%)

pub fn right_panel(signals: AppSignals) -> impl View {
    v_stack((
        stats_panel(signals.player),
        npc_panel(signals.active_npc),
    ))
    .style(|s| s
        .width(280.0)
        .height_full()
        .background(SIDEBAR_GROUND)
        .border_left(1.0)
        .border_color(SEAM)
    )
}

fn stat_row(label_text: &'static str, value_fn: impl Fn() -> String + 'static) -> impl View {
    h_stack((
        label(move || label_text.to_uppercase())
            .style(|s| s
                .flex_grow(1.0)
                .color(INK_GHOST)
                .font_size(12.0)
            ),
        label(value_fn)
            .style(|s| s
                .font_size(13.0)
                .color(INK)
            ),
    ))
    .style(|s| s.height(28.0).items_center())
}

fn stats_panel(player: RwSignal<PlayerSnapshot>) -> impl View {
    v_stack((
        // Signature Element — PC Name Display
        label(move || player.get().name)
            .style(|s| s
                .font_size(18.0)
                .font_weight(floem::text::Weight::LIGHT)
                .color(INK)
                .margin_bottom(16.0)
            ),
        
        stat_row("Femininity", move || player.get().femininity.to_string()),
        stat_row("Money",      move || format!("£{}", player.get().money)),
        stat_row("Stress",     move || player.get().stress.to_string()),
        stat_row("Anxiety",    move || player.get().anxiety.to_string()),
        
        // State section gap
        empty().style(|s| s.height(8.0)),
        
        stat_row("Arousal",    move || player.get().arousal.clone()),
        stat_row("Alcohol",    move || player.get().alcohol.clone()),
    ))
    .style(|s| s.padding(16.0))
}

fn npc_panel(active_npc: RwSignal<Option<NpcSnapshot>>) -> impl View {
    container(
        dyn_view(move || {
            if let Some(npc) = active_npc.get() {
                v_stack((
                    label(move || format!("── {} ──", npc.name.clone()))
                        .style(|s| s
                            .font_weight(floem::text::Weight::BOLD)
                            .font_size(13.0)
                            .color(INK)
                            .padding_bottom(8.0)
                            .items_center()
                            .width_full()
                        ),
                    stat_row("Age",          move || npc.age.clone()),
                    stat_row("Personality",  move || npc.personality.clone()),
                    stat_row("Relationship", move || npc.relationship.clone()),
                    stat_row("Liking",       move || npc.pc_liking.clone()),
                    stat_row("Attraction",   move || npc.pc_attraction.clone()),
                ))
                .into_any()
            } else {
                empty().into_any()
            }
        })
    )
    .style(|s| s
        .padding(16.0)
        .border_top(1.0)
        .border_color(SEAM)
    )
}
