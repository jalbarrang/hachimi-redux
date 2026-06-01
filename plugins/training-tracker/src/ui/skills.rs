//! Skills tab: acquired-skills list (scrollable).

use hachimi_plugin_sdk::egui;

use crate::overlay_cache;

use super::overlay;

pub(super) fn draw(ui: &mut egui::Ui) {
    overlay_cache::maybe_request_refresh();
    overlay::scroll_list(ui, draw_panel);
}

fn draw_panel(ui: &mut egui::Ui) {
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
