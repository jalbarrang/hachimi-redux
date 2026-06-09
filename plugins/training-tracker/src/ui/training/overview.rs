//! Training tab: turn/energy overview grid + active conditions side panel.

use hachimi_plugin_sdk::egui;

use crate::chara_effects::{self, Polarity};
use crate::memory_reader;
use crate::rank_table;

use super::super::util::{format_number, rank_badge_segments};

pub(super) fn draw(ui: &mut egui::Ui, snap: &memory_reader::CareerSnapshot) {
    // Overview grid on the left, active conditions table on the right.
    ui.horizontal_top(|ui| {
        draw_overview_grid(ui, snap);
        ui.add_space(20.0);
        draw_conditions(ui, snap);
    });
}

fn draw_overview_grid(ui: &mut egui::Ui, snap: &memory_reader::CareerSnapshot) {
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
                Some(value) => {
                    let label = rank_table::rank_label(value);
                    let text_color = ui.style().visuals.text_color();
                    let font = egui::TextStyle::Body.resolve(ui.style());
                    let mut job = egui::text::LayoutJob::default();
                    // Two-tone badge: U/L prefix in its family color, base
                    // letter in its Table 1 color (single segment for G..SS+).
                    for (text, color) in rank_badge_segments(label) {
                        job.append(
                            &text,
                            0.0,
                            egui::TextFormat {
                                font_id: font.clone(),
                                color,
                                ..Default::default()
                            },
                        );
                    }
                    job.append(
                        &format!(" \u{2022} {}", format_number(value)),
                        0.0,
                        egui::TextFormat {
                            font_id: font,
                            color: text_color,
                            ..Default::default()
                        },
                    );
                    ui.label(job)
                }
                None => ui.weak("\u{2014}"),
            };
            ui.end_row();
        });
}

/// Active career conditions (状態), colored by polarity: positive = orange,
/// negative = blue (matching the in-game full-stats screen).
fn draw_conditions(ui: &mut egui::Ui, snap: &memory_reader::CareerSnapshot) {
    ui.vertical(|ui| {
        ui.strong("Conditions");
        if snap.chara_effect_ids.is_empty() {
            ui.weak("None");
            return;
        }
        egui::Grid::new("tt_conditions")
            .num_columns(1)
            .striped(true)
            .show(ui, |ui| {
                for &id in &snap.chara_effect_ids {
                    let (name, polarity) = chara_effects::lookup(id);
                    let color = match polarity {
                        Polarity::Positive => egui::Color32::from_rgb(255, 160, 40), // orange
                        Polarity::Negative => egui::Color32::from_rgb(100, 150, 255), // blue
                    };
                    ui.colored_label(color, name);
                    ui.end_row();
                }
            });
    });
}
