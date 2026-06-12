//! Career panel Training section: the Speed/Stamina/Power/Guts/Wit table with
//! per-facility level, stat value + rank sprite, single/total gains, and failure
//! rate. Mirrors the dashboard `CareerPanel` Training grid.

use hachimi_plugin_sdk::egui::{self, Color32, CornerRadius, RichText, Stroke};

use super::super::textures;
use super::theme;
use crate::career_meta;
use crate::memory_reader::CareerSnapshot;

const FACILITIES: [&str; 5] = ["Speed", "Stamina", "Power", "Guts", "Wit"];

pub(super) fn draw(ui: &mut egui::Ui, snap: &CareerSnapshot) {
    theme::section_strip(ui, "Training", "");
    ui.add_space(4.0);

    let stats = [snap.speed, snap.stamina, snap.power, snap.guts, snap.wiz];
    egui::Frame::new()
        .inner_margin(egui::Margin::same(8))
        .corner_radius(CornerRadius::same(8))
        .fill(theme::SURFACE_2)
        .stroke(Stroke::new(1.0, theme::LINE))
        .show(ui, |ui| {
            egui::Grid::new("tt_career_training")
                .num_columns(6)
                .spacing([6.0, 5.0])
                .min_col_width(26.0)
                .show(ui, |ui| {
                    // Header: blank corner + facility columns (stat icon + level).
                    // The icon identifies the stat — names don't fit a 250px column.
                    ui.label("");
                    for (i, name) in FACILITIES.iter().enumerate() {
                        let resp = ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            textures::image_square(ui, &career_meta::stat_icon_path(i), 13.0, Color32::WHITE);
                            ui.label(
                                RichText::new(format!("L{}", snap.training_levels[i]))
                                    .small()
                                    .strong()
                                    .color(theme::UMA_400),
                            );
                        });
                        resp.response.on_hover_text(*name);
                    }
                    ui.end_row();

                    // Stat: rank sprite + value.
                    ui.label(RichText::new("Stat").strong().color(theme::FG));
                    for &v in &stats {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            textures::image_square(ui, &career_meta::stat_rank_sprite(v), 16.0, Color32::WHITE);
                            ui.label(RichText::new(v.to_string()).strong().color(theme::FG));
                        });
                    }
                    ui.end_row();

                    // Single: gain to the trained (own) stat.
                    ui.label(RichText::new("Single").strong().color(theme::FG));
                    for i in 0..5 {
                        gain_cell(ui, snap.per_stat_gains[i][i]);
                    }
                    ui.end_row();

                    // Total: sum of all gains from that facility.
                    ui.label(RichText::new("Total").strong().color(theme::FG));
                    for i in 0..5 {
                        gain_cell(ui, snap.stat_gains[i]);
                    }
                    ui.end_row();

                    // Failure %.
                    ui.label(RichText::new("Failure").strong().color(theme::FG));
                    for i in 0..5 {
                        fail_cell(ui, snap.failure_rates[i]);
                    }
                    ui.end_row();
                });
        });
}

fn gain_cell(ui: &mut egui::Ui, gain: i32) {
    if gain > 0 {
        ui.label(RichText::new(format!("+{gain}")).strong().color(theme::STAT_SPEED));
    } else {
        ui.label(RichText::new("\u{2013}").color(theme::FG_DIM));
    }
}

fn fail_cell(ui: &mut egui::Ui, rate: i32) {
    if rate < 0 {
        ui.label(RichText::new("\u{2013}").color(theme::FG_MUTED));
        return;
    }
    let color = if rate < 20 {
        theme::UMA_400
    } else if rate < 40 {
        theme::STAT_POWER
    } else if rate < 60 {
        theme::STAT_GUTS
    } else {
        theme::GRADE_A
    };
    ui.label(RichText::new(format!("{rate}%")).strong().color(color));
}
