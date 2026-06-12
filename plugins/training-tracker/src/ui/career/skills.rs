//! Career panel Skills section (acquired-skill cards with rarity rail + icon +
//! level) and the Conditions tag row. Mirrors the dashboard `CareerPanel` tail.

use hachimi_plugin_sdk::egui::{self, Align, Color32, CornerRadius, Layout, RichText, Stroke, Vec2};

use super::super::textures;
use super::theme;
use crate::chara_effects::{self, Polarity};
use crate::gametora_data;
use crate::memory_reader::CareerSnapshot;
use crate::overlay_cache;

pub(super) fn draw(ui: &mut egui::Ui, snap: &CareerSnapshot) {
    let skills = overlay_cache::skills();
    let trailing = match overlay_cache::skill_points() {
        Some(sp) => format!("{sp} SP  \u{b7} {}", skills.len()),
        None => format!("\u{b7} {}", skills.len()),
    };
    theme::section_strip(ui, "Skills", &trailing);
    ui.add_space(4.0);

    if skills.is_empty() {
        ui.label(RichText::new("No skills acquired yet").small().color(theme::FG_DIM));
    } else {
        let cols = (ui.available_width() / 200.0).floor().clamp(1.0, 3.0) as usize;
        let gap = 6.0;
        let cell_w = ((ui.available_width() - gap * (cols - 1) as f32) / cols as f32).max(120.0);
        for chunk in skills.chunks(cols) {
            ui.horizontal(|ui| {
                for (k, s) in chunk.iter().enumerate() {
                    if k > 0 {
                        ui.add_space(gap);
                    }
                    ui.allocate_ui_with_layout(Vec2::new(cell_w, 0.0), Layout::top_down(Align::Min), |ui| {
                        ui.set_width(cell_w);
                        skill_card(ui, s.master_id, s.level, &s.name);
                    });
                }
            });
            ui.add_space(gap);
        }
    }

    conditions(ui, snap);
}

fn skill_card(ui: &mut egui::Ui, master_id: i32, level: i32, name: &str) {
    let meta = gametora_data::skill(master_id as i64);
    let rarity = meta.and_then(|m| m.rarity).unwrap_or(1);
    let icon_id = meta.and_then(|m| m.iconid);

    egui::Frame::new()
        .inner_margin(egui::Margin {
            left: 0,
            right: 8,
            top: 5,
            bottom: 5,
        })
        .corner_radius(CornerRadius::same(8))
        .fill(theme::SURFACE_2)
        .stroke(Stroke::new(1.0, theme::LINE))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Rarity rail.
                let (rail, _) = ui.allocate_exact_size(Vec2::new(4.0, 24.0), egui::Sense::hover());
                ui.painter().rect_filled(
                    rail,
                    CornerRadius {
                        nw: 0,
                        ne: 2,
                        sw: 0,
                        se: 2,
                    },
                    rarity_color(rarity),
                );
                ui.add_space(6.0);
                if let Some(id) = icon_id {
                    textures::image_square(ui, &format!("{id}.png"), 24.0, Color32::WHITE);
                }
                let label = if name.is_empty() {
                    format!("#{master_id}")
                } else {
                    name.to_string()
                };
                // Name fills the row; Lv pill (when > 1) pinned right.
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if level > 1 {
                        theme::pill(ui, |ui| {
                            ui.label(
                                RichText::new(format!("Lv {level}"))
                                    .small()
                                    .strong()
                                    .color(theme::FG_MUTED),
                            );
                        });
                    }
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.add(egui::Label::new(RichText::new(label).small().strong().color(theme::FG)).truncate());
                    });
                });
            });
        });
}

/// Rarity rail color (uma-sim buckets): 1 white/silver, 2 gold, 3–5 unique
/// (rainbow → representative violet), 6 evolution pink.
fn rarity_color(rarity: i64) -> Color32 {
    match rarity {
        2 => Color32::from_rgb(0xff, 0xbe, 0x28),     // gold
        3..=5 => Color32::from_rgb(0xaa, 0xaa, 0xff), // unique (rainbow)
        6 => Color32::from_rgb(0xff, 0x9b, 0xd3),     // evolution pink
        _ => Color32::from_rgb(0xb5, 0xb2, 0xc6),     // white/silver
    }
}

fn conditions(ui: &mut egui::Ui, snap: &CareerSnapshot) {
    if snap.chara_effect_ids.is_empty() {
        return;
    }
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("CONDITIONS").small().strong().color(theme::FG_MUTED));
        for &id in &snap.chara_effect_ids {
            let (name, polarity) = chara_effects::lookup(id);
            // User convention: orange positive / blue negative.
            let color = match polarity {
                Polarity::Positive => theme::STAT_POWER,
                Polarity::Negative => Color32::from_rgb(0x4d, 0x9f, 0xff),
            };
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(8, 3))
                .corner_radius(CornerRadius::same(8))
                .fill(theme::SURFACE_2)
                .stroke(Stroke::new(1.0, color.gamma_multiply(0.6)))
                .show(ui, |ui| {
                    ui.label(RichText::new(name).small().strong().color(color));
                });
        }
    });
}
