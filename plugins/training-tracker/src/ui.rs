//! GUI rendering via the Hachimi plugin menu system.
//!
//! With API v9 the host hands plugins the real `egui::Ui`, so we draw with egui
//! directly. Registers a menu section and an overlay that display:
//! - Live career stats read directly from game memory (memory-read mode)
//! - Training facility visit counts from hooks (complementary)

use std::ffi::c_void;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::Ordering;

use hachimi_plugin_sdk::{egui, ui_from_ptr, Sdk};

use crate::class_dump;
use crate::memory_reader;
use crate::overlay_cache;
use crate::skill_shop;
use crate::skill_shop_prefs::{cycle_sort_mode, prefs, set_prefs, sort_mode_label, DistanceFilter, StyleFilter};
use crate::tracker::{Facility, TRACKER};

/// Default overlay font size.
const OVERLAY_FONT_SIZE: f32 = 12.0;
const OVERLAY_MIN_WIDTH: f32 = 300.0;

/// The overlay ID used during registration — must match for show/hide calls.
const OVERLAY_ID: &str = "training_tracker_overlay";

/// Register the plugin's UI components with the Hachimi GUI.
pub fn register_ui() {
    let sdk = Sdk::get();

    sdk.register_menu_section(draw_menu_section, std::ptr::null_mut());

    if sdk.register_overlay(OVERLAY_ID, draw_overlay, std::ptr::null_mut()) != 0 {
        hlog_info!(target: "training-tracker", "UI registered (menu + overlay)");
    } else {
        hlog_warn!(
            target: "training-tracker",
            "Menu registered; overlay registration declined by host"
        );
    }
}

// ===========================================================================
// Menu section (drawn inside Hachimi menu panel)
// ===========================================================================

extern "C" fn draw_menu_section(ui: *mut c_void, _userdata: *mut c_void) {
    // SAFETY: host passes its live `&mut egui::Ui` for this callback.
    let ui = unsafe { ui_from_ptr(ui) };
    if panic::catch_unwind(AssertUnwindSafe(|| draw_menu_section_inner(ui))).is_err() {
        hlog_error!("draw_menu_section PANICKED");
    }
}

fn draw_menu_section_inner(ui: &mut egui::Ui) {
    let sdk = Sdk::get();

    ui.heading("\u{1f3cb} Training Tracker");

    draw_tracking_controls(ui);
    draw_hook_status(ui);

    if ui.button("\u{1f4ca} Show Training Overlay").clicked() {
        if sdk.overlay_set_visible(OVERLAY_ID, true) {
            sdk.show_notification("Training overlay shown");
        } else {
            hlog_warn!(target: "training-tracker", "Host declined overlay_set_visible");
        }
    }

    if ui.button("\u{1f4cb} Dump All IL2CPP Classes").clicked() {
        class_dump::dump_all_classes();
        sdk.show_notification("Class dump complete — see il2cpp_classes.txt");
    }
}

/// Draw start/stop button and brief status in the menu.
fn draw_tracking_controls(ui: &mut egui::Ui) {
    let sdk = Sdk::get();
    let tracking = memory_reader::TRACKING.load(Ordering::Relaxed);

    if !tracking {
        if ui.button("\u{25b6} Start Memory Tracking").clicked() {
            match memory_reader::start_tracking() {
                Ok(()) => sdk.show_notification("Memory tracking started!"),
                Err(e) => {
                    sdk.show_notification(&format!("Failed: {}", e));
                    hlog_error!("start_tracking failed: {}", e);
                    false
                }
            };
        }
        ui.small("Reads stats directly from game memory via IL2CPP");
        return;
    }

    if ui.button("\u{23f9} Stop Memory Tracking").clicked() {
        memory_reader::stop_tracking();
        sdk.show_notification("Memory tracking stopped");
        return;
    }

    overlay_cache::maybe_request_refresh();
    let status = match overlay_cache::snapshot() {
        Some(snap) if snap.is_playing => format!(
            "\u{2705} Tracking • Turn {} • Total {}",
            snap.current_turn, snap.total_stats
        ),
        Some(_) => "\u{23f8} No active career".to_owned(),
        None => "\u{26a0} Waiting for data…".to_owned(),
    };
    ui.small(status);
}

/// Draw a compact hook-counts status line with a reset button.
fn draw_hook_status(ui: &mut egui::Ui) {
    let sdk = Sdk::get();

    let tracker = match TRACKER.lock() {
        Ok(t) => t,
        Err(_) => return,
    };

    let total = tracker.total();
    if !tracker.active && total == 0 {
        return;
    }
    drop(tracker);

    ui.small(format!("Hook events: {} trainings recorded", total));

    if ui.small_button("Reset Counts").clicked() {
        if let Ok(mut t) = TRACKER.lock() {
            t.counts = [0; 5];
            sdk.show_notification("Training counts reset!");
        }
    }
}

// ===========================================================================
// Overlay (always-on-screen HUD)
// ===========================================================================

extern "C" fn draw_overlay(ui: *mut c_void, _userdata: *mut c_void) {
    // SAFETY: host passes its live `&mut egui::Ui` for this callback.
    let ui = unsafe { ui_from_ptr(ui) };
    if panic::catch_unwind(AssertUnwindSafe(|| draw_overlay_inner(ui))).is_err() {
        hlog_error!("draw_overlay PANICKED");
    }
}

fn draw_overlay_inner(ui: &mut egui::Ui) {
    let tracking = memory_reader::TRACKING.load(Ordering::Relaxed);

    ui.style_mut().override_font_id = Some(egui::FontId::proportional(OVERLAY_FONT_SIZE));
    ui.set_min_width(OVERLAY_MIN_WIDTH);

    if tracking {
        draw_overlay_memory(ui);
    } else {
        draw_overlay_hooks(ui);
    }
}

/// Overlay: memory-read live stats.
fn draw_overlay_memory(ui: &mut egui::Ui) {
    overlay_cache::maybe_request_refresh();

    let snap = match overlay_cache::snapshot() {
        Some(s) if s.is_playing => s,
        Some(_) => {
            ui.small("\u{1f3cb} No active career");
            return;
        }
        None => {
            ui.small("\u{1f3cb} Loading career data…");
            return;
        }
    };

    ui.small(format!(
        "\u{1f3cb} Turn {} \u{2022} Month {}",
        snap.current_turn, snap.month
    ));

    let lv = &snap.training_levels;
    ui.small(format!(
        "Speed {:>4}(L{})  Stamina {:>4}(L{})  Power {:>4}(L{})",
        snap.speed, lv[0], snap.stamina, lv[1], snap.power, lv[2]
    ));
    ui.small(format!(
        "Guts {:>4}(L{})  Wit {:>4}(L{})  Total {:>4}",
        snap.guts, lv[3], snap.wiz, lv[4], snap.total_stats
    ));

    let (mr, mg, mb) = memory_reader::motivation_color(snap.motivation);
    ui.colored_label(
        egui::Color32::from_rgb(mr, mg, mb),
        format!(
            "Energy {}/{}  Mood: {}",
            snap.hp,
            snap.max_hp,
            memory_reader::mood_label(snap.motivation)
        ),
    );

    ui.small(format!(
        "Fans {}  Races {}/{}W",
        format_number(snap.fan_count),
        snap.total_races,
        snap.win_count
    ));

    egui::CollapsingHeader::new("\u{1f4d6} Skills")
        .default_open(false)
        .show(ui, draw_skills_panel);
    egui::CollapsingHeader::new("\u{1f91d} Bonds")
        .default_open(false)
        .show(ui, draw_bonds_panel);
    egui::CollapsingHeader::new("\u{1f6d2} Skill Shop")
        .default_open(false)
        .show(ui, draw_skill_shop_panel);
}

/// Draw the skills panel inside a collapsing header.
fn draw_skills_panel(ui: &mut egui::Ui) {
    let skills = overlay_cache::skills();

    if skills.is_empty() {
        ui.small("No skills acquired yet");
        return;
    }

    for skill in &skills {
        let label = if skill.name.is_empty() {
            format!("Lv.{} \u{2022} Skill #{}", skill.level, skill.master_id)
        } else {
            format!("Lv.{} \u{2022} {}", skill.level, skill.name)
        };
        ui.small(label);
    }

    ui.colored_label(
        egui::Color32::from_rgb(150, 150, 150),
        format!("{} skills", skills.len()),
    );
}

/// Draw the bonds/friendship panel inside a collapsing header.
fn draw_bonds_panel(ui: &mut egui::Ui) {
    let evals = overlay_cache::evaluations();

    if evals.is_empty() {
        ui.small("No bond data available");
        return;
    }

    for eval in &evals {
        if !eval.is_appear {
            continue;
        }

        let (r, g, b) = bond_color(eval.value);
        let name = if eval.name.is_empty() {
            format!("#{}", eval.target_id)
        } else {
            eval.name.clone()
        };
        ui.colored_label(
            egui::Color32::from_rgb(r, g, b),
            format!("{} - {}/100", name, eval.value),
        );
    }
}

/// Draw the skill shop panel inside a collapsing header.
fn draw_skill_shop_panel(ui: &mut egui::Ui) {
    if overlay_cache::snapshot().is_none() {
        ui.small("Loading shop data\u{2026}");
        return;
    }

    if let Some(sp) = overlay_cache::skill_points() {
        ui.small(format!("SP: {}", sp));
    }

    draw_skill_shop_controls(ui);

    let entries = skill_shop::prepare_entries_for_display(overlay_cache::skill_shop(), &prefs());
    if entries.is_empty() {
        ui.small("No shop skills match filters");
        return;
    }

    for entry in &entries {
        let icon = skill_shop::rarity_label(entry.rarity);
        let discount = skill_shop::discount_pct(entry.hint_level, false);
        let color = if entry.rarity >= 2 {
            egui::Color32::from_rgb(255, 200, 50)
        } else {
            egui::Color32::from_rgb(220, 220, 220)
        };

        let name = if entry.name.is_empty() {
            format!("#{}", entry.group_id)
        } else {
            entry.name.clone()
        };

        let cost_str = if entry.base_cost > 0 {
            let discounted = skill_shop::discounted_cost(entry.base_cost, entry.hint_level, false);
            format!(" {}pt", discounted)
        } else {
            String::new()
        };

        let prefix = if !entry.has_hint { "[full] " } else { "" };

        let label = if discount > 0 {
            format!("{}{} {} (-{}%{})", prefix, icon, name, discount, cost_str)
        } else if cost_str.is_empty() {
            format!("{}{} {}", prefix, icon, name)
        } else {
            format!("{}{} {} ({})", prefix, icon, name, cost_str.trim())
        };
        ui.colored_label(color, label);
    }
}

fn draw_skill_shop_controls(ui: &mut egui::Ui) {
    let p = prefs();

    if ui
        .small_button(format!("Sort: {}", sort_mode_label(p.sort_mode)))
        .clicked()
    {
        cycle_sort_mode();
    }

    ui.small("Style:");
    for &(label, filter) in StyleFilter::LABELS {
        let selected = p.style_filter == filter;
        if ui
            .small_button(format!("{}{}", if selected { "*" } else { "" }, label))
            .clicked()
        {
            set_prefs(|prefs| prefs.style_filter = filter);
        }
    }

    ui.small("Dist:");
    for &(label, filter) in DistanceFilter::LABELS {
        let selected = p.distance_filter == filter;
        if ui
            .small_button(format!("{}{}", if selected { "*" } else { "" }, label))
            .clicked()
        {
            set_prefs(|prefs| prefs.distance_filter = filter);
        }
    }

    let mut show_hintless = p.show_hintless;
    if ui.checkbox(&mut show_hintless, "Show full-price (no hint)").changed() {
        set_prefs(|prefs| prefs.show_hintless = show_hintless);
    }
    if show_hintless {
        ui.small("Open the in-game skill shop once to capture purchasable rows.");
    }
}

/// Color for bond/friendship value: blue → green → orange → gold (max).
pub fn bond_color(value: i32) -> (u8, u8, u8) {
    if value >= 100 {
        (255, 200, 50) // Gold - maxed
    } else if value >= 80 {
        (255, 160, 40) // Orange - high
    } else if value >= 40 {
        (100, 220, 100) // Green - medium
    } else {
        (100, 150, 255) // Blue - low
    }
}

/// Overlay: hook-based training counts (fallback when not memory-tracking).
fn draw_overlay_hooks(ui: &mut egui::Ui) {
    let tracker = match TRACKER.lock() {
        Ok(t) => t,
        Err(_) => return,
    };

    if !tracker.active && tracker.total() == 0 {
        return;
    }

    let counts = tracker.counts;
    let total = tracker.total();
    drop(tracker);

    ui.small("\u{1f3cb} Training");

    for facility in Facility::ALL {
        let count = counts[facility as usize];
        let (r, g, b) = facility_color(facility);
        ui.colored_label(
            egui::Color32::from_rgb(r, g, b),
            format!("{}: {}", facility.name(), count),
        );
    }

    ui.small(format!("Total: {}", total));
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Color per facility (matching common game UI colors).
pub(crate) fn facility_color(facility: Facility) -> (u8, u8, u8) {
    match facility {
        Facility::Speed => (70, 130, 255),   // Blue
        Facility::Stamina => (255, 70, 70),  // Red
        Facility::Power => (255, 140, 40),   // Orange
        Facility::Guts => (255, 130, 180),   // Pink
        Facility::Wisdom => (100, 220, 100), // Green
    }
}

/// Format a number with comma separators.
pub(crate) fn format_number(n: i32) -> String {
    if n < 0 {
        return format!("-{}", format_number(-n));
    }
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- bond_color ----

    #[test]
    fn bond_color_thresholds() {
        // Maxed (gold)
        assert_eq!(bond_color(100), (255, 200, 50));
        assert_eq!(bond_color(150), (255, 200, 50));
        // High (orange)
        assert_eq!(bond_color(80), (255, 160, 40));
        assert_eq!(bond_color(99), (255, 160, 40));
        // Medium (green)
        assert_eq!(bond_color(40), (100, 220, 100));
        assert_eq!(bond_color(79), (100, 220, 100));
        // Low (blue)
        assert_eq!(bond_color(0), (100, 150, 255));
        assert_eq!(bond_color(39), (100, 150, 255));
    }

    // ---- format_number ----

    #[test]
    fn format_number_basic() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
    }

    #[test]
    fn format_number_negative() {
        assert_eq!(format_number(-1000), "-1,000");
        assert_eq!(format_number(-42), "-42");
    }

    // ---- facility_color ----

    #[test]
    fn facility_colors_distinct() {
        let colors: Vec<_> = Facility::ALL.iter().map(|f| facility_color(*f)).collect();
        // All 5 should be different
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(
                    colors[i],
                    colors[j],
                    "Facilities {:?} and {:?} share color",
                    Facility::ALL[i],
                    Facility::ALL[j]
                );
            }
        }
    }

    // ---- motivation helpers (from memory_reader, tested here for convenience) ----

    #[test]
    fn mood_labels() {
        assert!(memory_reader::mood_label(5).contains("Great"));
        assert!(memory_reader::mood_label(3).contains("Normal"));
        assert!(memory_reader::mood_label(1).contains("Terrible"));
        assert_eq!(memory_reader::mood_label(0), "???");
    }

    #[test]
    fn motivation_colors_distinct() {
        let colors: Vec<_> = (1..=5).map(memory_reader::motivation_color).collect();
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(colors[i], colors[j]);
            }
        }
    }
}
