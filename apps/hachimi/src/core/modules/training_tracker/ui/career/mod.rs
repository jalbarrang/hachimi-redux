//! Unified "Career" overlay panel — an egui port of the honse-tracker dashboard
//! `CareerPanel`: a single scrolling view stacking the trainee header, the
//! Training table, Bonds, Skills, and Conditions, styled with [`theme`].
//!
//! Sections are added incrementally (career-overlay-port t-005..t-008); for now
//! only the theme primitives exist.

mod bonds;
mod bonds_table;
mod header;
// Skills section is hidden for now (its draw call is disabled below); the module
// is kept compiling until it's re-enabled.
#[allow(dead_code)]
mod skills;
mod theme;
mod training;

use crate::core::modules::training_tracker::compat::egui;

use super::overlay;
use crate::core::modules::training_tracker::memory_reader::CareerSnapshot;
use crate::core::modules::training_tracker::overlay_cache;

/// Career tab entry point: refresh, then render the panel inside a scroll area,
/// or a waiting note when no career is active.
pub(super) fn draw_tab(ui: &mut egui::Ui) {
    overlay_cache::maybe_request_refresh();
    // The overlay is now height-capped (see `ui::mod`), so the Career body scrolls
    // internally within the remaining height instead of growing the host window.
    match overlay_cache::snapshot() {
        Some(s) if s.is_playing => overlay::scroll_list(ui, |ui| draw(ui, &s)),
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
    ui.add_space(8.0);

    training::draw(ui, snap);
    ui.add_space(8.0);

    bonds::draw(ui, snap);
    // ui.add_space(10.0);
    // skills::draw(ui, snap);
}
