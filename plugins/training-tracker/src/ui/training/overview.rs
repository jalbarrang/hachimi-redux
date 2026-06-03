//! Training tab: turn/energy overview grid.

use hachimi_plugin_sdk::egui;

use crate::memory_reader;
use crate::rank_table;

use super::super::util::format_number;

pub(super) fn draw(ui: &mut egui::Ui, snap: &memory_reader::CareerSnapshot) {
    egui::Grid::new("tt_overview")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Turn");
            ui.label(format!("{} \u{2022} Month {}", snap.current_turn, snap.month));
            ui.end_row();

            let (mr, mg, mb) = memory_reader::motivation_color(snap.motivation);
            let mood = egui::Color32::from_rgb(mr, mg, mb);
            ui.strong("Energy");
            ui.colored_label(mood, format!("{}/{}", snap.hp, snap.max_hp));
            ui.end_row();

            ui.strong("Mood");
            ui.colored_label(mood, memory_reader::mood_label(snap.motivation));
            ui.end_row();

            ui.strong("Rank");
            match snap.evaluation_value {
                Some(value) => ui.strong(format!(
                    "{} \u{2022} {}",
                    rank_table::rank_label(value),
                    format_number(value)
                )),
                None => ui.weak("\u{2014}"),
            };
            ui.end_row();
        });
}
