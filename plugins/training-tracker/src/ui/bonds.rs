//! Bonds tab: bond names + progress (scrollable).

use hachimi_plugin_sdk::egui;

use crate::overlay_cache;

use super::overlay;
use super::util::bond_color;

pub(super) fn draw(ui: &mut egui::Ui) {
    overlay_cache::maybe_request_refresh();
    overlay::scroll_list(ui, draw_panel);
}

fn draw_panel(ui: &mut egui::Ui) {
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
