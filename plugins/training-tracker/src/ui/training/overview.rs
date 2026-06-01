//! Training tab: turn/energy overview grid.

use hachimi_plugin_sdk::egui;

use crate::memory_reader;

pub(super) fn draw(ui: &mut egui::Ui, snap: &memory_reader::CareerSnapshot) {
    egui::Grid::new("tt_overview")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.label("Turn");
            ui.label(format!("{} \u{2022} Month {}", snap.current_turn, snap.month));
            ui.end_row();

            let (mr, mg, mb) = memory_reader::motivation_color(snap.motivation);
            ui.label("Energy");
            ui.colored_label(
                egui::Color32::from_rgb(mr, mg, mb),
                format!(
                    "{}/{}  {}",
                    snap.hp,
                    snap.max_hp,
                    memory_reader::mood_label(snap.motivation)
                ),
            );
            ui.end_row();
        });

    ui.add_space(4.0);
}
