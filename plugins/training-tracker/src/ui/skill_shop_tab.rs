//! Skill Shop tab: SP + filters + purchasable list (scrollable).

use hachimi_plugin_sdk::egui;

use crate::overlay_cache;
use crate::skill_shop;
use crate::skill_shop_prefs::{cycle_sort_mode, prefs, set_prefs, sort_mode_label, DistanceFilter, StyleFilter};

use super::overlay;

pub(super) fn draw(ui: &mut egui::Ui) {
    overlay_cache::maybe_request_refresh();
    if overlay_cache::snapshot().is_none() {
        ui.small("Loading shop data…");
        return;
    }

    draw_header(ui);
    draw_controls(ui);
    ui.separator();
    overlay::scroll_list(ui, draw_list);
}

fn draw_header(ui: &mut egui::Ui) {
    if let Some(sp) = overlay_cache::skill_points() {
        ui.strong(format!("SP: {}", sp));
    }
}

fn draw_controls(ui: &mut egui::Ui) {
    let p = prefs();

    if ui
        .small_button(format!("Sort: {}", sort_mode_label(p.sort_mode)))
        .clicked()
    {
        cycle_sort_mode();
    }

    ui.horizontal_wrapped(|ui| {
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
    });

    ui.horizontal_wrapped(|ui| {
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
    });

    let mut show_hintless = p.show_hintless;
    if ui.checkbox(&mut show_hintless, "Show full-price (no hint)").changed() {
        set_prefs(|prefs| prefs.show_hintless = show_hintless);
    }
    if show_hintless {
        ui.small("Open the in-game skill shop once to capture purchasable rows.");
    }
}

fn draw_list(ui: &mut egui::Ui) {
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
