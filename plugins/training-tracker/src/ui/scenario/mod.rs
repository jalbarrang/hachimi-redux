//! Scenario tab: scenario-specific readout.

mod trackblazer;

use hachimi_plugin_sdk::egui;

use crate::memory_reader;

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
