//! Career panel Bonds section: a two-column list of supports/guests with their
//! card type, bond value, and the facility they trained on this turn — with the
//! rainbow-ready highlight when a card can friendship-train. Mirrors the
//! dashboard `CareerPanel` Bonds grid.

use hachimi_plugin_sdk::egui::{self, Align, Layout, RichText, Vec2};

use super::theme;
use crate::gametora_data;
use crate::memory_reader::CareerSnapshot;
use crate::overlay_cache;

/// One resolved bond row.
struct Bond {
    name: String,
    /// Specialty facility (0..4) when the card is a stat card; `None` for
    /// guests / pal-friend / uncatalogued.
    specialty: Option<usize>,
    /// `true` for a pal/friend card (emoji glyph), `false` for `group`.
    is_friend: bool,
    has_type: bool,
    value: i32,
    /// Facility trained on this turn (0..4), from partner placements.
    on_facility: Option<usize>,
    is_support: bool,
    rainbow_ready: bool,
}

pub(super) fn draw(ui: &mut egui::Ui, snap: &CareerSnapshot) {
    theme::section_strip(ui, "Bonds", "");
    ui.add_space(4.0);

    let mut bonds = collect(snap);
    if bonds.is_empty() {
        ui.label(RichText::new("No bond data yet").small().color(theme::FG_DIM));
        return;
    }
    // Supports before guests, then highest bond first.
    bonds.sort_by(|a, b| (a.is_support.cmp(&b.is_support).reverse()).then(b.value.cmp(&a.value)));

    // Single full-width column at the overlay's narrow width.
    let w = super::super::overlay::content_width();
    for bond in &bonds {
        row(ui, bond, w);
        ui.add_space(4.0);
    }
}

fn collect(snap: &CareerSnapshot) -> Vec<Bond> {
    let evals = overlay_cache::evaluations();
    let deck = overlay_cache::equipped_support_ids();
    evals
        .iter()
        .filter(|e| e.is_appear || e.value > 0)
        .map(|e| {
            let support_id = deck
                .iter()
                .find(|(slot, _)| *slot == e.target_id)
                .map(|(_, id)| *id as i64)
                .filter(|id| *id > 0);

            let card = support_id.and_then(gametora_data::support_card);
            let type_str = card.and_then(|c| c.r#type.as_deref());
            let specialty = support_id.and_then(gametora_data::support_specialty_facility);
            let on_facility = snap.partner_placements.get(&e.target_id).map(|(f, _)| *f);

            let name = support_id
                .and_then(gametora_data::support_card_name)
                .map(str::to_owned)
                .filter(|n| !n.is_empty())
                .or_else(|| (!e.name.is_empty()).then(|| e.name.clone()))
                .or_else(|| scenario_npc_name(snap.scenario_id, e.target_id).map(str::to_owned))
                .unwrap_or_else(|| format!("#{}", e.target_id));

            let is_support = e.guest_chara_id <= 0 && (support_id.is_some() || (1..=6).contains(&e.target_id));
            // Rainbow fires only on the card's own specialty facility at bond >= 80.
            let rainbow_ready = specialty.is_some() && e.value >= 80 && on_facility == specialty;

            Bond {
                name,
                specialty,
                is_friend: type_str == Some("friend"),
                has_type: type_str.is_some(),
                value: e.value,
                on_facility,
                is_support,
                rainbow_ready,
            }
        })
        .collect()
}

/// A bond row: `[name (fills, truncates) | type chip | bond value | On chip]`.
/// Deterministic widths only — no right_to_left/left_to_right nesting, which broke
/// (rows escaped the column) under the auto_sized window.
fn row(ui: &mut egui::Ui, bond: &Bond, w: f32) {
    theme::row_frame(bond.rainbow_ready).show(ui, |ui| {
        // Fill the column (minus the frame's horizontal inner margin).
        ui.set_width((w - 20.0).max(40.0));
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            // Reserve the right cluster's budget; the name takes the rest and truncates.
            let right = 96.0;
            let name_w = (ui.available_width() - right).max(28.0);
            ui.allocate_ui_with_layout(Vec2::new(name_w, 18.0), Layout::left_to_right(Align::Center), |ui| {
                ui.add(egui::Label::new(RichText::new(&bond.name).small().strong().color(theme::FG)).truncate());
            });
            type_chip(ui, bond);
            bond_value(ui, bond.value);
            on_chip(ui, bond.on_facility);
        });
    });
}

/// Card specialty: stat chip for trainable types, glyph for pal/friend/group.
fn type_chip(ui: &mut egui::Ui, bond: &Bond) {
    if !bond.has_type {
        ui.label(RichText::new("\u{2013}").small().color(theme::FG_DIM));
        return;
    }
    match bond.specialty {
        Some(f) => {
            theme::stat_chip(ui, f, 16.0);
        }
        None => {
            let glyph = if bond.is_friend { "\u{1f91d}" } else { "\u{1f465}" }; // 🤝 / 👥
            ui.label(RichText::new(glyph).small());
        }
    }
}

fn bond_value(ui: &mut egui::Ui, value: i32) {
    let color = if value >= 80 {
        theme::STAT_POWER
    } else if value >= 60 {
        theme::UMA_400
    } else {
        theme::FG
    };
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label(RichText::new(value.to_string()).small().strong().color(color));
        ui.label(RichText::new("/100").small().color(theme::FG_DIM));
    });
}

/// The facility this card trained on this turn (stat chip), or a dash.
fn on_chip(ui: &mut egui::Ui, facility: Option<usize>) {
    match facility {
        Some(f) => {
            theme::stat_chip(ui, f, 16.0);
        }
        None => {
            ui.label(RichText::new("\u{2013}").small().color(theme::FG_DIM));
        }
    }
}

/// Scenario NPC names (not real support cards), keyed by scenario + target id.
fn scenario_npc_name(scenario_id: i32, target_id: i32) -> Option<&'static str> {
    match (scenario_id, target_id) {
        (4, 102) => Some("Director Akikawa"),
        (4, 103) => Some("Etsuko Otonashi"),
        _ => None,
    }
}
