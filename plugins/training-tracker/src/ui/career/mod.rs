//! Unified "Career" overlay panel — an egui port of the honse-tracker dashboard
//! `CareerPanel`: a single scrolling view stacking the trainee header, the
//! Training table, Bonds, Skills, and Conditions, styled with [`theme`].
//!
//! Sections are added incrementally (career-overlay-port t-005..t-008); for now
//! only the theme primitives exist.

mod bonds;
mod header;
mod skills;
mod theme;
mod training;

use hachimi_plugin_sdk::egui;

use crate::memory_reader::CareerSnapshot;
use crate::overlay_cache;

/// Career tab entry point: refresh, then render the panel inside a scroll area,
/// or a waiting note when no career is active.
pub(super) fn draw_tab(ui: &mut egui::Ui) {
    overlay_cache::maybe_request_refresh();
    // No scroll area: the overlay is fixed-width / auto-height, so the panel just
    // grows to fit. A ScrollArea here fills the available width and defeats the
    // width cap (the window then grows without bound).
    match overlay_cache::snapshot() {
        Some(s) if s.is_playing => draw(ui, &s),
        _ => {
            ui.label(
                egui::RichText::new("Waiting for an active career\u{2026}")
                    .italics()
                    .color(theme::FG_MUTED),
            );
        }
    }
}

/// Draw the unified Career panel for an active career snapshot. The overlay's own
/// background frame is the panel face, so sections are drawn directly (no card).
fn draw(ui: &mut egui::Ui, snap: &CareerSnapshot) {
    header::draw(ui, snap);
    ui.add_space(10.0);
    training::draw(ui, snap);
    ui.add_space(10.0);
    bonds::draw(ui, snap);
    ui.add_space(10.0);
    skills::draw(ui, snap);
}
