//! Training tab: cap warning, turn suggestion, career summary.

use hachimi_plugin_sdk::egui;

use crate::memory_reader;
use crate::recommend;

use super::super::util::{format_number, rank_text};
use super::stats_grid::StatRow;

pub(super) fn draw(
    ui: &mut egui::Ui,
    snap: &memory_reader::CareerSnapshot,
    stats: &[StatRow; 5],
    rec: &[recommend::FacilityScore; 5],
    any_capped: bool,
) {
    if any_capped {
        ui.small("\u{26a0} target/cap reached — further training wasted");
    }

    let race_encouraged = recommend::scenario_encourages_racing(snap.scenario_command_base);

    match recommend::turn_suggestion(rec, snap.failure_rates, race_encouraged) {
        recommend::TurnSuggestion::Train(best) => {
            ui.small(format!(
                "\u{2605} best: {} — projected score {}",
                stats[best].0, rec[best].score
            ));
        }
        recommend::TurnSuggestion::Rest => {
            ui.colored_label(egui::Color32::from_rgb(120, 200, 255), "\u{1f4a4} Rest");
        }
        recommend::TurnSuggestion::Race => {
            ui.colored_label(egui::Color32::from_rgb(255, 200, 50), "\u{1f3c1} Race");
        }
    }

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.strong(format!("Total {}", snap.total_stats));
        ui.separator();
        ui.label(rank_text(snap));
    });
    ui.small(format!(
        "Fans {}  Races {}/{}W",
        format_number(snap.fan_count),
        snap.total_races,
        snap.win_count
    ));
}
