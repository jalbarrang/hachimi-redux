//! Career panel Bonds section: a two-column list of supports/guests with their
//! card type, bond value, and the facility they trained on this turn — with the
//! rainbow-ready highlight when a card can friendship-train. Mirrors the
//! dashboard `CareerPanel` Bonds grid.

use egui_taffy::taffy::prelude::{auto, fr, length};
use egui_taffy::{taffy, tui, TuiBuilderLogic, TuiContainerResponse};
use hachimi_plugin_sdk::egui::{self, Align, Layout, RichText, Vec2, Vec2b};

use super::super::dimens;
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

    // Two side-by-side columns, filled column-major: the first half goes down the
    // left column, the rest down the right.
    let w = super::super::overlay::content_width();
    let gap = dimens::z(dimens::GAP_MD);
    let col_w = ((w - gap) / 2.0).max(60.0);
    let mid = bonds.len().div_ceil(2);
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        bond_column(ui, &bonds, 0..mid, col_w);
        bond_column(ui, &bonds, mid..bonds.len(), col_w);
    });
}

/// One vertical column of bond rows at a pinned width.
fn bond_column(ui: &mut egui::Ui, bonds: &[Bond], range: std::ops::Range<usize>, col_w: f32) {
    ui.allocate_ui_with_layout(Vec2::new(col_w, 0.0), Layout::top_down(Align::Min), |ui| {
        ui.set_width(col_w);
        for idx in range {
            row(ui, &bonds[idx], col_w, idx);
            ui.add_space(4.0);
        }
    });
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
/// Laid out with an egui_taffy grid: the name column is the single `fr(1.)`
/// flexible track and truncates, while the right cluster uses `auto()` tracks
/// sized to their content.
fn row(ui: &mut egui::Ui, bond: &Bond, w: f32, idx: usize) {
    theme::row_frame(bond.rainbow_ready).show(ui, |ui| {
        // Fill the column (minus the frame's symmetric 10px horizontal margin).
        let inner = (w - dimens::z(dimens::ROW_FRAME_MARGIN)).max(40.0);
        ui.set_width(inner);
        // Cells get ~0 measured width during taffy's layout pass; force text to
        // extend so non-truncating labels don't wrap one glyph per line.
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
        // Unique per row: a shared id would make every row collide on one
        // persistent Taffy state and thrash it dirty every frame.
        tui(ui, ui.id().with("bond_row").with(idx))
            .reserve_width(inner)
            .style(taffy::Style {
                display: taffy::Display::Grid,
                // Name flexes; the three right-cluster cells size to content.
                grid_template_columns: vec![fr(1.), auto(), auto(), auto()],
                gap: taffy::Size {
                    width: length(dimens::z(dimens::GAP_MD)),
                    height: length(0.0),
                },
                align_items: Some(taffy::AlignItems::Center),
                size: taffy::Size {
                    width: length(inner),
                    height: auto(),
                },
                ..Default::default()
            })
            .show(|tui| {
                // Name cell: left-aligned, takes the flexible track, truncates.
                // Report a constant, width-independent size via `ui_manual`: a
                // truncating label fills whatever width Taffy assigns it, so
                // reporting `ui.min_size()` (what `.ui()` does) feeds the assigned
                // width back into fr-track sizing and makes Taffy recompute every
                // frame (the `request_discard` spam) while the track collapses to
                // `...`. min/max width 0 + `infinite.x` lets it grow into the
                // flexible track without the feedback loop.
                tui.style(cell(taffy::JustifyContent::Start)).add(|tui| {
                    tui.ui_manual(|ui, _| {
                        ui.add(
                            egui::Label::new(RichText::new(&bond.name).small().strong().color(theme::FG)).truncate(),
                        );
                        let h = ui.min_size().y;
                        TuiContainerResponse {
                            inner: (),
                            min_size: Vec2::new(0.0, h),
                            intrinsic_size: None,
                            max_size: Vec2::new(0.0, h),
                            infinite: Vec2b::new(true, false),
                        }
                    });
                });
                tui.style(cell(taffy::JustifyContent::Center))
                    .add(|tui| tui.ui(|ui| type_chip(ui, bond)));
                tui.style(cell(taffy::JustifyContent::Center))
                    .add(|tui| tui.ui(|ui| bond_value(ui, bond.value)));
                tui.style(cell(taffy::JustifyContent::Center))
                    .add(|tui| tui.ui(|ui| on_chip(ui, bond.on_facility)));
            });
    });
}

/// A grid cell: a flex row with the given main-axis justification, vertically
/// centered. `min_size.width = 0` lets the name track shrink so it truncates
/// instead of forcing the grid wider than its column.
fn cell(justify: taffy::JustifyContent) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Row,
        align_items: Some(taffy::AlignItems::Center),
        justify_content: Some(justify),
        min_size: taffy::Size {
            width: length(0.0),
            height: auto(),
        },
        ..Default::default()
    }
}

/// Card specialty: stat chip for trainable types, glyph for pal/friend/group.
fn type_chip(ui: &mut egui::Ui, bond: &Bond) {
    if !bond.has_type {
        ui.label(RichText::new("\u{2013}").small().color(theme::FG_DIM));
        return;
    }
    match bond.specialty {
        Some(f) => {
            theme::stat_chip(ui, f, dimens::z(dimens::ICON_MD));
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
            theme::stat_chip(ui, f, dimens::z(dimens::ICON_MD));
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
