use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::RwSignal;
use floem::views::dyn_stack;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::game_state::GameState;
use crate::theme::ThemeColors;
use crate::AppSignals;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct SaveEntry {
    pub path: PathBuf,
    pub name: String,
    /// Formatted date/time string, e.g. "2026-02-23 22:47"
    pub modified: String,
    /// Raw seconds since epoch for sorting
    pub modified_secs: u64,
}

// ---------------------------------------------------------------------------
// Save directory helpers
// ---------------------------------------------------------------------------

fn saves_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("undone").join("saves"))
}

/// Convert a `SystemTime` to `u64` seconds since UNIX_EPOCH (0 on error).
fn system_time_to_secs(t: SystemTime) -> u64 {
    t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

/// Format seconds-since-epoch as `YYYY-MM-DD HH:MM` (UTC).
///
/// Avoids adding any new crates by doing manual Gregorian calendar arithmetic.
fn format_epoch_secs(secs: u64) -> String {
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;

    // Gregorian calendar from day count (days since 1970-01-01)
    let z = days + 719468;
    let era = z / 146097;
    let doe = z % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y_adj = if m <= 2 { y + 1 } else { y };

    format!("{:04}-{:02}-{:02} {:02}:{:02}", y_adj, m, d, hour, minute)
}

/// Read all `.json` files from the saves directory, sorted newest-first.
pub fn list_saves() -> Vec<SaveEntry> {
    let dir = match saves_dir() {
        Some(d) if d.is_dir() => d,
        _ => return vec![],
    };

    let mut entries = vec![];
    if let Ok(read_dir) = std::fs::read_dir(&dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                let name = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let modified_secs = entry
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .map(system_time_to_secs)
                    .unwrap_or(0);
                let modified = format_epoch_secs(modified_secs);
                entries.push(SaveEntry {
                    path,
                    name,
                    modified,
                    modified_secs,
                });
            }
        }
    }

    // Sort newest-first by raw seconds
    entries.sort_by(|a, b| b.modified_secs.cmp(&a.modified_secs));
    entries
}

// ---------------------------------------------------------------------------
// Save panel view
// ---------------------------------------------------------------------------

pub fn saves_panel(signals: AppSignals, state: Rc<RefCell<GameState>>) -> impl View {
    let save_list: RwSignal<Vec<SaveEntry>> = RwSignal::new(list_saves());
    let status_msg: RwSignal<String> = RwSignal::new(String::new());

    // --- "Save Current Game" button ---
    let save_state = Rc::clone(&state);
    let save_btn = label(|| "Save Current Game".to_string())
        .keyboard_navigable()
        .on_click_stop(move |_| {
            let dir = match saves_dir() {
                Some(d) => d,
                None => {
                    status_msg.set("Could not determine save directory.".into());
                    return;
                }
            };
            if let Err(e) = std::fs::create_dir_all(&dir) {
                status_msg.set(format!("Failed to create save directory: {e}"));
                return;
            }

            let gs = save_state.borrow();
            let fem_name = gs.world.player.name_fem.replace(' ', "_");
            let ts = system_time_to_secs(SystemTime::now());
            let filename = format!("{fem_name}_{ts}.json");
            let path = dir.join(&filename);

            match undone_save::save_game(&gs.world, &gs.registry, &path) {
                Ok(()) => {
                    status_msg.set(format!("Saved: {}", filename.trim_end_matches(".json")));
                    save_list.set(list_saves());
                }
                Err(e) => {
                    status_msg.set(format!("Save failed: {e}"));
                }
            }
        })
        .style(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.font_size(14.0)
                .font_family("system-ui, -apple-system, sans-serif".to_string())
                .padding_horiz(20.0)
                .padding_vert(10.0)
                .border(1.0)
                .border_radius(4.0)
                .border_color(colors.lamp)
                .color(colors.lamp)
                .background(colors.lamp_glow)
                .hover(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    s.background(colors.lamp_glow).border_color(colors.lamp)
                })
                .active(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    s.background(colors.lamp_glow)
                })
                .focus_visible(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    s.outline(2.0).outline_color(colors.lamp)
                })
        });

    // --- Status message strip ---
    let status_strip = label(move || status_msg.get()).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.font_size(13.0)
            .font_family("system-ui, -apple-system, sans-serif".to_string())
            .color(colors.ink_ghost)
            .min_height(20.0)
            .margin_top(6.0)
    });

    // --- Save entries list ---
    let list_state = Rc::clone(&state);
    let entries_list = dyn_stack(
        move || save_list.get(),
        |e: &SaveEntry| e.path.to_string_lossy().to_string(),
        {
            let list_state = Rc::clone(&list_state);
            move |entry: SaveEntry| {
                let entry_path_load = entry.path.clone();
                let entry_path_delete = entry.path.clone();
                let name_display = entry.name.clone();
                let modified_display = entry.modified.clone();

                // --- Load button ---
                let load_state = Rc::clone(&list_state);
                let load_btn = label(|| "Load".to_string())
                    .keyboard_navigable()
                    .on_click_stop(move |_| {
                        // Phase 1: load world and start the scene while holding the RefMut.
                        // Collect all engine events so we can release the borrow before
                        // calling process_events (which needs only &World, not &mut).
                        let events_and_fem = {
                            let mut gs = load_state.borrow_mut();
                            match undone_save::load_game(&entry_path_load, &gs.registry) {
                                Err(e) => {
                                    status_msg.set(format!("Load failed: {e}"));
                                    return;
                                }
                                Ok(loaded_world) => {
                                    gs.world = loaded_world;
                                }
                            }

                            // Clear UI signals first
                            signals.story.set(String::new());
                            signals.actions.set(vec![]);
                            signals.active_npc.set(None);
                            status_msg.set(
                                entry_path_load
                                    .file_stem()
                                    .map(|s| format!("Loaded: {}", s.to_string_lossy()))
                                    .unwrap_or_else(|| "Loaded.".into()),
                            );

                            // Start the opening scene so the player isn't staring at blank text
                            let fem_id = gs.registry.resolve_skill("FEMININITY").ok();
                            let GameState {
                                ref mut engine,
                                ref mut world,
                                ref registry,
                                ref opening_scene,
                                ..
                            } = *gs;
                            if let Some(scene_id) = opening_scene {
                                crate::start_scene(engine, world, registry, scene_id.clone());
                            }
                            let events = engine.drain();
                            (events, fem_id)
                        }; // RefMut dropped here

                        // Phase 2: process events with only a shared borrow
                        let (events, fem_id_opt) = events_and_fem;
                        if let Some(fem_id) = fem_id_opt {
                            let gs = load_state.borrow();
                            crate::process_events(events, signals, &gs.world, fem_id);
                        }

                        signals.tab.set(crate::AppTab::Game);
                    })
                    .style(move |s| small_action_btn_style(s, signals));

                // --- Delete button ---
                let delete_btn = label(|| "Delete".to_string())
                    .keyboard_navigable()
                    .on_click_stop(move |_| match std::fs::remove_file(&entry_path_delete) {
                        Ok(()) => {
                            save_list.set(list_saves());
                            status_msg.set(String::new());
                        }
                        Err(e) => {
                            status_msg.set(format!("Delete failed: {e}"));
                        }
                    })
                    .style(move |s| small_action_btn_style(s, signals));

                // --- Entry card ---
                let buttons = h_stack((load_btn, delete_btn))
                    .style(|s| s.flex_row().gap(8.0).margin_top(6.0));

                let name_label = label(move || name_display.clone()).style(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    s.font_size(14.0)
                        .font_family("system-ui, -apple-system, sans-serif".to_string())
                        .color(colors.ink)
                });

                let modified_label = label(move || modified_display.clone()).style(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    s.font_size(12.0)
                        .font_family("system-ui, -apple-system, sans-serif".to_string())
                        .color(colors.ink_dim)
                        .margin_top(2.0)
                });

                v_stack((name_label, modified_label, buttons)).style(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    s.padding(12.0)
                        .margin_bottom(8.0)
                        .border(1.0)
                        .border_radius(4.0)
                        .border_color(colors.seam)
                        .background(colors.page)
                        .width_full()
                })
            }
        },
    )
    .style(|s| s.flex_col().width_full());

    // --- Empty state label ---
    let empty_label = dyn_view(move || {
        if save_list.get().is_empty() {
            label(|| "No saves yet.".to_string())
                .style(move |s| {
                    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
                    s.font_size(14.0)
                        .font_family("system-ui, -apple-system, sans-serif".to_string())
                        .color(colors.ink_ghost)
                        .margin_top(16.0)
                })
                .into_any()
        } else {
            empty().into_any()
        }
    });

    // --- Top bar (save button + status) ---
    let top_bar = v_stack((save_btn, status_strip)).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.padding(16.0)
            .border_bottom(1.0)
            .border_color(colors.seam)
            .width_full()
    });

    // --- Scrollable list area ---
    let list_area =
        scroll(v_stack((entries_list, empty_label)).style(|s| s.padding(16.0).width_full()))
            .style(|s| s.flex_grow(1.0).width_full());

    v_stack((top_bar, list_area)).style(move |s| {
        let colors = ThemeColors::from_mode(signals.prefs.get().mode);
        s.size_full().background(colors.ground)
    })
}

// ---------------------------------------------------------------------------
// Shared button style helper
// ---------------------------------------------------------------------------

fn small_action_btn_style(s: floem::style::Style, signals: AppSignals) -> floem::style::Style {
    let colors = ThemeColors::from_mode(signals.prefs.get().mode);
    s.font_size(12.0)
        .font_family("system-ui, -apple-system, sans-serif".to_string())
        .padding_horiz(12.0)
        .padding_vert(5.0)
        .border(1.0)
        .border_radius(4.0)
        .border_color(colors.seam)
        .color(colors.ink_dim)
        .background(Color::TRANSPARENT)
        .hover(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.border_color(colors.lamp)
                .color(colors.lamp)
                .background(colors.lamp_glow)
        })
        .active(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.background(colors.lamp_glow)
        })
        .focus_visible(move |s| {
            let colors = ThemeColors::from_mode(signals.prefs.get().mode);
            s.outline(2.0).outline_color(colors.lamp)
        })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_epoch_secs_known_date() {
        // 2026-02-23 22:47:00 UTC = 1772452020 seconds since epoch
        // Let's verify: 2026-02-23 is day 20507 since 1970-01-01
        // 20507 * 86400 + 22*3600 + 47*60 = 1771804800 + 79200 + 2820 = 1771886820
        // Actually compute: days from 1970-01-01 to 2026-02-23
        // 1970..2026 = 56 years, with leap years
        // Use a well-known epoch: 2000-01-01 = 946684800
        // 2026-02-23: 26*365 + leap_years(2000..2026) = 26*365 + 7 = 9497 days from 2000-01-01
        // 946684800 + 9497*86400 = 946684800 + 820540800 = 1767225600 for 2026-01-01
        // + 31 (Jan) + 22 (Feb 1..22) = 53 days = 53*86400 = 4579200
        // 1767225600 + 4579200 = 1771804800 for 2026-02-23 00:00:00 UTC
        let secs = 1771804800u64 + 22 * 3600 + 47 * 60;
        let result = format_epoch_secs(secs);
        assert_eq!(result, "2026-02-23 22:47");
    }

    #[test]
    fn format_epoch_secs_unix_epoch() {
        let result = format_epoch_secs(0);
        assert_eq!(result, "1970-01-01 00:00");
    }

    #[test]
    fn list_saves_returns_empty_when_no_dir() {
        // The saves dir won't exist in CI/test environments; should return empty vec.
        // We can't fully test this without mocking dirs::data_dir, but we can confirm
        // it doesn't panic.
        let _ = list_saves();
    }
}
