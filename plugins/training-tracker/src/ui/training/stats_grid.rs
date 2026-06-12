//! Training tab: stat columns, gains, failure rates, recommendations.

use hachimi_plugin_sdk::egui;

use crate::build_profile;
use crate::course_data;
use crate::gametora_data;
use crate::memory_reader;
use crate::overlay_cache;
use crate::planner;
use crate::recommend;
use crate::stat_targets;

use super::super::util::{cap_level, failure_rate_color, stat_rank_color, CapLevel};

/// Per-stat display row: name, value, training level, effective cap threshold.
pub(super) type StatRow = (&'static str, i32, i32, i32);

pub(super) fn build_stats(snap: &memory_reader::CareerSnapshot) -> [StatRow; 5] {
    let lv = &snap.training_levels;
    let caps = &snap.stat_caps;
    let tgt = stat_targets::targets();
    let thr = |i: usize, cap: i32| stat_targets::effective_threshold(tgt[i], cap);
    [
        ("Speed", snap.speed, lv[0], thr(0, caps[0])),
        ("Stamina", snap.stamina, lv[1], thr(1, caps[1])),
        ("Power", snap.power, lv[2], thr(2, caps[2])),
        ("Guts", snap.guts, lv[3], thr(3, caps[3])),
        ("Wit", snap.wiz, lv[4], thr(4, caps[4])),
    ]
}

/// Build the multi-turn planner context from the live snapshot + active targets.
/// `bond_pressure` comes from each facility's present supports (their bond gauge
/// vs the rainbow threshold), read live into the snapshot.
pub(super) fn plan_context(snap: &memory_reader::CareerSnapshot) -> planner::PlannerContext {
    let current = [snap.speed, snap.stamina, snap.power, snap.guts, snap.wiz];
    planner::PlannerContext {
        hp: snap.hp,
        max_hp: snap.max_hp,
        motivation: snap.motivation,
        current_turn: snap.current_turn,
        failure_rates: snap.failure_rates,
        stat_deficit: planner::stat_deficits(current, stat_targets::targets(), snap.stat_caps),
        bond_pressure: Some(if planner::params().specialty_rainbow_gating {
            specialty_gated_bond_pressure(snap)
        } else {
            snap.per_facility_bond_pressure
        }),
    }
}

/// Re-aggregate per-facility near-rainbow pressure, counting a support only on its
/// **own specialty** facility (where a rainbow can actually fire). Same soft-OR as
/// the ungated value (`1 − ∏(1 − p_k)`), but restricted to on-specialty deck
/// cards. Guests and pal/friend/group cards drop out (no deck-map / no specialty).
/// Used only when `PlannerParams::specialty_rainbow_gating` is on.
fn specialty_gated_bond_pressure(snap: &memory_reader::CareerSnapshot) -> [f32; 5] {
    // `target_id` (deck slot 1..6) -> equipped support-card id.
    let deck: std::collections::HashMap<i32, i32> = overlay_cache::equipped_support_ids().into_iter().collect();
    let mut not_p = [1.0f32; 5];
    for (&target_id, &(facility, pressure)) in &snap.partner_placements {
        if facility >= 5 {
            continue;
        }
        let Some(&card_id) = deck.get(&target_id) else {
            continue; // guest / unknown deck slot
        };
        // Only the card's own specialty facility can rainbow it.
        if gametora_data::support_specialty_facility(card_id as i64) != Some(facility) {
            continue;
        }
        not_p[facility] *= 1.0 - pressure.clamp(0.0, 1.0);
    }
    std::array::from_fn(|i| (1.0 - not_p[i]).clamp(0.0, 1.0))
}

/// Build the objective/CM scoring context from the active build profile + target
/// course data. Missing course params degrade gracefully to the Rank objective.
pub(super) fn scoring_context(snap: &memory_reader::CareerSnapshot) -> recommend::ScoringContext<'static> {
    let profile = build_profile::active();
    let course = course_data::course_params(profile.target_course_id);
    let aptitudes = course
        .map(|c| recommend::cm_aptitudes_for_course(&snap.aptitudes, c))
        .unwrap_or_default();
    recommend::ScoringContext {
        objective: profile.objective,
        stat_weights: profile.stat_weights,
        course,
        aptitudes,
        strategy: profile.strategy,
        ground_condition: profile.ground_condition,
        // Planned (inherited/acquired) recoveries + the trained outfit's own
        // built-in recoveries (full value), so the model never asks you to train
        // stamina the Uma already covers.
        recovery_heal_bp: (crate::gametora_data::recovery_heal_bp_total(&profile.recovery_skill_ids)
            + crate::gametora_data::card_recovery_bp_total(snap.card_id as i64)) as f64,
        // Career races add a flat per-scenario stat line in-race; fold it into the
        // curve-position math so Stamina isn't over-valued mid-career.
        scenario_race_bonus: crate::cm_model::scenario_race_bonus(snap.scenario_id),
    }
}

pub(super) fn score_facilities(
    snap: &memory_reader::CareerSnapshot,
    ctx: &recommend::ScoringContext,
) -> [recommend::FacilityScore; 5] {
    recommend::score_facilities(
        &recommend::Inputs {
            current: [snap.speed, snap.stamina, snap.power, snap.guts, snap.wiz],
            per_stat_gains: &snap.per_stat_gains,
            caps: snap.stat_caps,
            targets: stat_targets::targets(),
            failure_rates: snap.failure_rates,
            ctx: *ctx,
        },
        &recommend::params(),
    )
}

pub(super) fn draw(
    ui: &mut egui::Ui,
    snap: &memory_reader::CareerSnapshot,
    stats: &[StatRow; 5],
    rec: &[recommend::FacilityScore; 5],
    show_scores: bool,
) -> bool {
    let mut any_capped = false;
    egui::Grid::new("tt_stats")
        .num_columns(stats.len() + 1)
        .striped(true)
        .show(ui, |ui| {
            // Top-left corner is blank; stat names act as the column header.
            ui.label("");
            for (name, _, level, _) in stats {
                ui.label(format!("{} (L{})", name, level));
            }
            ui.end_row();

            ui.strong("Stat");
            for (_, value, _, cap) in stats {
                // Color is keyed off the stat's letter rank; the cap warning
                // (⚠) is preserved as a marker when the stat is at its cap.
                let color = stat_rank_color(*value);
                let text = match cap_level(*value, *cap) {
                    CapLevel::AtCap => {
                        any_capped = true;
                        format!("{}\u{26a0}", value)
                    }
                    CapLevel::Near | CapLevel::Normal => value.to_string(),
                };
                ui.colored_label(color, text);
            }
            ui.end_row();

            // Single: gain to the trained (own) stat only.
            ui.strong("Single");
            for (i, _) in stats.iter().enumerate() {
                let single = snap.per_stat_gains[i][i];
                if single > 0 {
                    ui.colored_label(egui::Color32::from_rgb(120, 200, 255), format!("+{}", single));
                } else {
                    ui.weak("—");
                }
            }
            ui.end_row();

            // Total: sum of all stat gains from that facility.
            ui.strong("Total");
            for gain in &snap.stat_gains {
                if *gain > 0 {
                    ui.colored_label(egui::Color32::from_rgb(120, 200, 255), format!("+{}", gain));
                } else {
                    ui.weak("—");
                }
            }
            ui.end_row();

            ui.strong("Failure");
            for fail in &snap.failure_rates {
                if *fail >= 0 {
                    let (r, g, b) = failure_rate_color(*fail);
                    ui.colored_label(egui::Color32::from_rgb(r, g, b), format!("{}%", fail));
                } else {
                    ui.weak("—");
                }
            }
            ui.end_row();

            // The Score row is the recommendation surface; hidden when the
            // objective is `Off`.
            if show_scores {
                ui.strong("Score");
                for fs in rec {
                    if fs.known {
                        if fs.is_best {
                            ui.colored_label(egui::Color32::from_rgb(120, 220, 120), format!("\u{2605}{}", fs.score));
                        } else {
                            ui.weak(fs.score.to_string());
                        }
                    } else {
                        ui.weak("—");
                    }
                }
                ui.end_row();
            }
        });
    any_capped
}
