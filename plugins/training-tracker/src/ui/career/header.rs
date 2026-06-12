//! Career panel header: trainee identity (portrait + rank badge + eval value,
//! name, outfit, stars) and the condition cluster (year·date·turn, energy, mood).
//! Mirrors the top row of the dashboard `CareerPanel`.

use hachimi_plugin_sdk::egui::{self, Color32, CornerRadius, Pos2, Rect, RichText, Stroke, StrokeKind, Vec2};

use super::super::textures;
use super::theme;
use crate::career_meta;
use crate::gametora_data;
use crate::memory_reader::{self, CareerSnapshot};
use crate::rank_table;

const PORTRAIT: f32 = 72.0;

pub(super) fn draw(ui: &mut egui::Ui, snap: &CareerSnapshot) {
    ui.horizontal_top(|ui| {
        identity(ui, snap);
        // Push the condition cluster to the right edge.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            condition(ui, snap);
        });
    });
}

fn identity(ui: &mut egui::Ui, snap: &CareerSnapshot) {
    ui.vertical(|ui| {
        portrait_with_badge(ui, snap);
        if let Some(ev) = snap.evaluation_value {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(group_thousands(ev)).strong().color(theme::FG_MUTED));
            });
        }
    });
    ui.add_space(10.0);
    ui.vertical(|ui| {
        ui.add_space(6.0);
        let card = gametora_data::character_card(snap.card_id as i64);
        let name = card
            .and_then(|c| c.name_en.clone().or_else(|| c.name_jp.clone()))
            .unwrap_or_else(|| format!("#{}", snap.card_id));
        ui.label(RichText::new(name).size(18.0).strong().color(theme::FG));
        ui.horizontal(|ui| {
            if let Some(outfit) = card.and_then(|c| c.title_en_gl.clone().or_else(|| c.title_jp.clone())) {
                if !outfit.is_empty() {
                    ui.label(RichText::new(outfit).size(12.0).strong().color(theme::FG_MUTED));
                }
            }
            stars(ui, snap.star.clamp(0, 5));
        });
    });
}

/// Portrait square with the overlapping circular rank badge at the top-right.
fn portrait_with_badge(ui: &mut egui::Ui, snap: &CareerSnapshot) {
    let badge = 30.0;
    let region = Vec2::new(PORTRAIT + badge * 0.4, PORTRAIT + badge * 0.35);
    let (rect, _) = ui.allocate_exact_size(region, egui::Sense::hover());
    let p_rect = Rect::from_min_size(Pos2::new(rect.left(), rect.bottom() - PORTRAIT), Vec2::splat(PORTRAIT));

    // Portrait image (or placeholder), with a rounded border.
    let drawn = career_meta::trainee_portrait_path(snap.card_id)
        .and_then(|path| textures::texture(ui.ctx(), &path))
        .map(|tex| {
            ui.painter().image(
                tex.id(),
                p_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        })
        .is_some();
    if !drawn {
        ui.painter()
            .rect_filled(p_rect, CornerRadius::same(10), theme::SURFACE_3);
    }
    ui.painter().rect_stroke(
        p_rect,
        CornerRadius::same(10),
        Stroke::new(1.0, theme::LINE),
        StrokeKind::Inside,
    );

    // Rank badge: gold-ringed dark medallion with the rank sprite, top-right.
    if let Some(ev) = snap.evaluation_value {
        let label = rank_table::rank_label(ev);
        let center = Pos2::new(p_rect.right() - 2.0, p_rect.top() + 2.0);
        let r = badge * 0.5;
        ui.painter().circle_filled(center, r, theme::SURFACE_1);
        ui.painter().circle_stroke(center, r, Stroke::new(2.0, theme::GOLD));
        let drew = career_meta::rank_label_sprite(label)
            .and_then(|path| textures::texture(ui.ctx(), &path))
            .map(|tex| {
                let s = badge * 0.74;
                let ir = Rect::from_center_size(center, Vec2::splat(s));
                ui.painter().image(
                    tex.id(),
                    ir,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            })
            .is_some();
        if !drew {
            ui.painter().text(
                center,
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(badge * 0.5),
                theme::GOLD,
            );
        }
    }
}

fn stars(ui: &mut egui::Ui, value: i32) {
    let mut s = String::new();
    for i in 0..5 {
        s.push(if i < value { '\u{2605}' } else { '\u{2606}' }); // ★ / ☆
    }
    ui.label(RichText::new(s).size(13.0).color(theme::GOLD));
}

fn condition(ui: &mut egui::Ui, snap: &CareerSnapshot) {
    // right_to_left layout: add from the right; reverse visual order by stacking
    // vertically instead so the cluster reads top-down like the web.
    ui.vertical(|ui| {
        let (year, date) = career_meta::turn_date(snap.current_turn, snap.scenario_id);
        theme::pill(ui, |ui| {
            ui.label(RichText::new(year).strong().color(theme::UMA_300));
            ui.label(RichText::new("·").color(theme::FG_DIM));
            ui.label(RichText::new(date).strong().color(theme::FG));
            ui.label(RichText::new("·").color(theme::FG_DIM));
            ui.label(
                RichText::new(format!("T{}", snap.current_turn))
                    .strong()
                    .color(theme::FG_MUTED),
            );
        });
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            energy_pill(ui, snap);
            ui.add_space(6.0);
            mood_pill(ui, snap);
        });
    });
}

fn energy_pill(ui: &mut egui::Ui, snap: &CareerSnapshot) {
    let pct = if snap.max_hp > 0 {
        (snap.hp as f32 / snap.max_hp as f32 * 100.0).round() as i32
    } else {
        0
    };
    let hp_color = if pct <= 25 {
        theme::GRADE_A
    } else if pct <= 50 {
        theme::STAT_POWER
    } else {
        theme::UMA_300
    };
    theme::pill(ui, |ui| {
        ui.label(RichText::new("Energy").strong().color(theme::FG_MUTED));
        ui.label(RichText::new(snap.hp.to_string()).strong().color(hp_color));
        ui.label(RichText::new(format!("/{}", snap.max_hp)).color(theme::FG_DIM));
    });
}

fn mood_pill(ui: &mut egui::Ui, snap: &CareerSnapshot) {
    let label = memory_reader::mood_label(snap.motivation);
    theme::pill(ui, |ui| {
        ui.label(
            RichText::new(label.to_uppercase())
                .strong()
                .color(theme::mood_color(snap.motivation)),
        );
    });
}

/// Thousands-separated integer ("7,002").
fn group_thousands(n: i32) -> String {
    let neg = n < 0;
    let digits: Vec<char> = n.unsigned_abs().to_string().chars().collect();
    let mut out = String::new();
    for (i, c) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*c);
    }
    if neg {
        format!("-{out}")
    } else {
        out
    }
}
