//! Scenario tab: scenario-specific readout.

mod trackblazer;

use crate::core::modules::training_tracker::compat::egui;

use crate::core::modules::training_tracker::memory_reader;

use super::snapshot;

pub(super) fn draw(ui: &mut egui::Ui) {
    let Some(snap) = snapshot::current_snapshot(ui) else {
        return;
    };
    match snap.scenario_state {
        Some(memory_reader::ScenarioState::Trackblazer(shop)) => trackblazer::draw(ui, &shop),
        None => {
            ui.small("No scenario-specific data for this run.");
            ui.small("(Supported: Trackblazer / Make a New Track.)");
        }
    }
}
