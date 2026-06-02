//! Smart training recommendation: score each facility by its projected 評価点
//! (evaluation-point) gain this turn, failure-adjusted and clamped at the player's
//! per-stat targets/caps. Pure logic — no IL2CPP, safe on the render thread.
//!
//! Model (agreed with the user):
//! ```text
//! eval_delta = Σ_stat [ stat_score(min(cur + gain, ceiling)) − stat_score(cur) ]
//!   ceiling = effective_threshold(target, cap)   (0 ⇒ no clamp)
//! p = failure_rate / 100
//! score = eval_delta × (1 − p)                         # EV of the gains
//! if failure_rate > risk_threshold_pct:                # extra risk penalty
//!     loss = eval_cost_of_losing failure_stat_loss pts + mood_drop_penalty
//!     score −= p × loss
//! ```
//!
//! The thresholds/penalties are user-tunable via [`RecommendParams`] (surfaced in
//! the L1 settings page and persisted by `crate::config`); the scoring functions
//! take them explicitly so the logic stays pure and deterministically testable.
//!
//! Known v1 limitation: greedy per-turn. It does not value building bonds early for
//! later rainbow payoff, nor cross-turn energy management.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::evaluation::stat_score;
use crate::stat_targets;

/// Default failure % above which the extra downside penalty applies.
pub const DEFAULT_RISK_THRESHOLD_PCT: i32 = 25;
/// Default: if EVERY available facility's failure % exceeds this, training is a bad
/// turn — suggest resting (or racing on race-encouraged scenarios) instead.
pub const DEFAULT_ALL_RISKY_PCT: i32 = 30;
/// Default eval-point cost of one motivation-level drop on failure.
pub const DEFAULT_MOOD_DROP_PENALTY: i32 = 30;
/// Default stat points lost on a failed training (applied to the trained stats).
pub const DEFAULT_FAILURE_STAT_LOSS: i32 = 5;

/// User-tunable weights for the recommendation model. Persisted in
/// `training_config.json` (see `crate::config`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RecommendParams {
    /// Failure % above which a facility gets the extra downside penalty.
    #[serde(default = "default_risk_threshold")]
    pub risk_threshold_pct: i32,
    /// If every facility's failure % exceeds this, suggest Rest/Race.
    #[serde(default = "default_all_risky")]
    pub all_risky_pct: i32,
    /// Eval-point cost charged for the mood drop on a failed training.
    #[serde(default = "default_mood_penalty")]
    pub mood_drop_penalty: i32,
    /// Modeled stat points lost on a failed training.
    #[serde(default = "default_failure_stat_loss")]
    pub failure_stat_loss: i32,
}

impl RecommendParams {
    /// The built-in defaults (a `const` so it can seed a `static` store).
    pub const DEFAULT: RecommendParams = RecommendParams {
        risk_threshold_pct: DEFAULT_RISK_THRESHOLD_PCT,
        all_risky_pct: DEFAULT_ALL_RISKY_PCT,
        mood_drop_penalty: DEFAULT_MOOD_DROP_PENALTY,
        failure_stat_loss: DEFAULT_FAILURE_STAT_LOSS,
    };

    /// Clamp to sane ranges (percentages `0..=100`, penalties non-negative).
    fn clamped(self) -> Self {
        Self {
            risk_threshold_pct: self.risk_threshold_pct.clamp(0, 100),
            all_risky_pct: self.all_risky_pct.clamp(0, 100),
            mood_drop_penalty: self.mood_drop_penalty.max(0),
            failure_stat_loss: self.failure_stat_loss.max(0),
        }
    }
}

impl Default for RecommendParams {
    fn default() -> Self {
        Self::DEFAULT
    }
}

fn default_risk_threshold() -> i32 {
    DEFAULT_RISK_THRESHOLD_PCT
}
fn default_all_risky() -> i32 {
    DEFAULT_ALL_RISKY_PCT
}
fn default_mood_penalty() -> i32 {
    DEFAULT_MOOD_DROP_PENALTY
}
fn default_failure_stat_loss() -> i32 {
    DEFAULT_FAILURE_STAT_LOSS
}

/// Live, user-tunable parameters (defaults until config loads).
static PARAMS: Mutex<RecommendParams> = Mutex::new(RecommendParams::DEFAULT);

/// Current recommendation parameters.
pub fn params() -> RecommendParams {
    PARAMS.lock().map(|g| *g).unwrap_or(RecommendParams::DEFAULT)
}

/// Replace the parameters (clamped to sane ranges). Call [`crate::config::persist`]
/// to write them to disk.
pub fn set_params(p: RecommendParams) {
    if let Ok(mut g) = PARAMS.lock() {
        *g = p.clamped();
    }
}

/// Scenario training-set bases (Speed-slot command id) where racing is the better
/// fallback when all trainings are too risky. Trackblazer (Make a New Track, base
/// 1101) rewards racing; URA (101) and Unity Cup (601) do not.
const RACE_ENCOURAGED_BASES: &[i32] = &[1101];

/// Whether the active scenario (identified by its Speed-slot command base) rewards
/// racing enough to prefer it over resting when every facility is too risky.
#[must_use]
pub fn scenario_encourages_racing(scenario_command_base: i32) -> bool {
    RACE_ENCOURAGED_BASES.contains(&scenario_command_base)
}

/// The overall suggestion for the turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnSuggestion {
    /// Train the given facility slot (the best by projected score).
    Train(usize),
    /// All facilities are too risky — rest to recover energy.
    Rest,
    /// All facilities are too risky, but this scenario rewards racing.
    Race,
}

/// Decide the turn suggestion: if every facility with live data exceeds
/// `params.all_risky_pct` failure, suggest Rest (or Race when `race_encouraged`);
/// otherwise train the best-scoring facility.
#[must_use]
pub fn turn_suggestion(
    scores: &[FacilityScore; 5],
    failure_rates: [i32; 5],
    race_encouraged: bool,
    params: &RecommendParams,
) -> TurnSuggestion {
    let known: Vec<usize> = (0..5).filter(|&i| scores[i].known).collect();
    let all_risky = !known.is_empty() && known.iter().all(|&i| failure_rates[i] > params.all_risky_pct);
    if all_risky {
        return if race_encouraged {
            TurnSuggestion::Race
        } else {
            TurnSuggestion::Rest
        };
    }
    match scores.iter().position(|f| f.is_best) {
        Some(i) => TurnSuggestion::Train(i),
        None => TurnSuggestion::Rest,
    }
}

/// Per-facility recommendation result.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FacilityScore {
    /// Projected 評価点 gain this turn after failure/risk adjustment (can be negative).
    pub score: i32,
    /// `true` for the single best facility (highest score among known facilities).
    pub is_best: bool,
    /// Whether live command info was available for this facility this turn.
    pub known: bool,
}

/// Inputs for the recommendation, all per facility-slot [Speed, Stamina, Power,
/// Guts, Wisdom] (outer index = facility, matching the snapshot layout).
pub struct Inputs<'a> {
    /// Current stat values [Speed, Stamina, Power, Guts, Wisdom].
    pub current: [i32; 5],
    /// Per-facility, per-stat gain (facility × stat).
    pub per_stat_gains: &'a [[i32; 5]; 5],
    /// Per-stat caps [Speed..Wisdom]; `0` ⇒ unknown.
    pub caps: [i32; 5],
    /// Per-stat targets [Speed..Wisdom]; `0` ⇒ use cap.
    pub targets: [i32; 5],
    /// Per-facility failure %; `< 0` ⇒ unknown (treated as 0% for scoring).
    pub failure_rates: [i32; 5],
}

/// Score all five facilities and flag the best. Facilities with no live gain data
/// (all-zero row) are marked `known: false` and excluded from the best pick.
#[must_use]
pub fn score_facilities(input: &Inputs, params: &RecommendParams) -> [FacilityScore; 5] {
    let mut out = [FacilityScore::default(); 5];
    let mut best: Option<(usize, i32)> = None;

    for (i, slot) in out.iter_mut().enumerate() {
        let gains = input.per_stat_gains[i];
        let known = gains.iter().any(|&g| g != 0);
        let fail = input.failure_rates[i].max(0);
        let score = facility_score(input.current, gains, input.caps, input.targets, fail, params);
        *slot = FacilityScore {
            score,
            is_best: false,
            known,
        };
        if known && best.is_none_or(|(_, bs)| score > bs) {
            best = Some((i, score));
        }
    }

    if let Some((b, _)) = best {
        out[b].is_best = true;
    }
    out
}

/// Score a single facility: failure-adjusted projected 評価点 delta.
fn facility_score(
    current: [i32; 5],
    gains: [i32; 5],
    caps: [i32; 5],
    targets: [i32; 5],
    fail_pct: i32,
    params: &RecommendParams,
) -> i32 {
    let eval_delta = projected_eval_delta(current, gains, caps, targets);
    let p = fail_pct as f32 / 100.0;
    let mut score = eval_delta as f32 * (1.0 - p);

    if fail_pct > params.risk_threshold_pct {
        score -= p * failure_loss(current, gains, params) as f32;
    }
    score.round() as i32
}

/// Projected 評価点 gain from a facility's per-stat gains, clamping useful gains at
/// `min(target, cap)` (gains past the ceiling earn no evaluation).
fn projected_eval_delta(current: [i32; 5], gains: [i32; 5], caps: [i32; 5], targets: [i32; 5]) -> i32 {
    let mut delta = 0;
    for s in 0..5 {
        if gains[s] == 0 {
            continue;
        }
        let ceiling = stat_targets::effective_threshold(targets[s], caps[s]);
        let raised = current[s] + gains[s];
        let capped = if ceiling > 0 { raised.min(ceiling) } else { raised };
        delta += stat_score(capped) - stat_score(current[s]);
    }
    delta
}

/// Eval-point cost of a failed training: losing `params.failure_stat_loss` on each
/// stat the facility would have raised, plus the mood-drop penalty.
fn failure_loss(current: [i32; 5], gains: [i32; 5], params: &RecommendParams) -> i32 {
    let mut loss = params.mood_drop_penalty;
    for s in 0..5 {
        if gains[s] == 0 {
            continue;
        }
        let dropped = (current[s] - params.failure_stat_loss).max(0);
        loss += stat_score(current[s]) - stat_score(dropped);
    }
    loss
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Default params for tests (the scoring logic is independent of the live store).
    fn p() -> RecommendParams {
        RecommendParams::default()
    }

    fn one_facility(slot: usize, gains: [i32; 5]) -> [[i32; 5]; 5] {
        let mut m = [[0i32; 5]; 5];
        m[slot] = gains;
        m
    }

    #[test]
    fn unknown_facilities_are_not_best() {
        let input = Inputs {
            current: [100; 5],
            per_stat_gains: &[[0i32; 5]; 5],
            caps: [0; 5],
            targets: [0; 5],
            failure_rates: [-1; 5],
        };
        let out = score_facilities(&input, &p());
        assert!(out.iter().all(|f| !f.known));
        assert!(out.iter().all(|f| !f.is_best));
    }

    #[test]
    fn best_is_highest_score() {
        // Speed facility gains 20 on Speed; Guts facility gains 20 on Guts but at 40% fail.
        let mut gains = [[0i32; 5]; 5];
        gains[0] = [20, 0, 0, 0, 0];
        gains[3] = [0, 0, 0, 20, 0];
        let input = Inputs {
            current: [300; 5],
            per_stat_gains: &gains,
            caps: [0; 5],
            targets: [0; 5],
            failure_rates: [0, -1, -1, 40, -1],
        };
        let out = score_facilities(&input, &p());
        assert!(out[0].is_best, "safe Speed should beat risky Guts");
        assert!(out[0].score > out[3].score);
        assert!(out[3].known);
    }

    #[test]
    fn higher_failure_lowers_score() {
        let gains = one_facility(0, [20, 0, 0, 0, 0]);
        let mk = |fail: i32| {
            let input = Inputs {
                current: [300; 5],
                per_stat_gains: &gains,
                caps: [0; 5],
                targets: [0; 5],
                failure_rates: [fail, -1, -1, -1, -1],
            };
            score_facilities(&input, &p())[0].score
        };
        assert!(mk(0) > mk(20));
        assert!(mk(20) > mk(50)); // risk penalty kicks in above 25%
    }

    #[test]
    fn gains_past_ceiling_earn_nothing() {
        // Stat at its target already → projected delta is ~0 regardless of gain.
        let gains = one_facility(0, [50, 0, 0, 0, 0]);
        let at_target = Inputs {
            current: [1200, 0, 0, 0, 0],
            per_stat_gains: &gains,
            caps: [1200, 0, 0, 0, 0],
            targets: [1200, 0, 0, 0, 0],
            failure_rates: [0, -1, -1, -1, -1],
        };
        assert_eq!(score_facilities(&at_target, &p())[0].score, 0);

        // Same gain with headroom scores positive.
        let with_room = Inputs {
            current: [800, 0, 0, 0, 0],
            per_stat_gains: &gains,
            caps: [1200, 0, 0, 0, 0],
            targets: [0, 0, 0, 0, 0],
            failure_rates: [0, -1, -1, -1, -1],
        };
        assert!(score_facilities(&with_room, &p())[0].score > 0);
    }

    #[test]
    fn suggest_rest_when_all_risky() {
        let mut gains = [[0i32; 5]; 5];
        for (i, g) in gains.iter_mut().enumerate() {
            g[i] = 10; // every facility raises one stat
        }
        let input = Inputs {
            current: [300; 5],
            per_stat_gains: &gains,
            caps: [0; 5],
            targets: [0; 5],
            failure_rates: [35, 40, 31, 50, 33], // all > 30%
        };
        let scores = score_facilities(&input, &p());
        assert_eq!(
            turn_suggestion(&scores, input.failure_rates, false, &p()),
            TurnSuggestion::Rest
        );
        assert_eq!(
            turn_suggestion(&scores, input.failure_rates, true, &p()),
            TurnSuggestion::Race
        );
    }

    #[test]
    fn suggest_train_when_one_is_safe() {
        let mut gains = [[0i32; 5]; 5];
        for (i, g) in gains.iter_mut().enumerate() {
            g[i] = 10;
        }
        let input = Inputs {
            current: [300; 5],
            per_stat_gains: &gains,
            caps: [0; 5],
            targets: [0; 5],
            failure_rates: [35, 40, 5, 50, 33], // Power is safe
        };
        let scores = score_facilities(&input, &p());
        assert_eq!(
            turn_suggestion(&scores, input.failure_rates, false, &p()),
            TurnSuggestion::Train(2)
        );
    }

    #[test]
    fn no_data_suggests_rest() {
        let input = Inputs {
            current: [300; 5],
            per_stat_gains: &[[0i32; 5]; 5],
            caps: [0; 5],
            targets: [0; 5],
            failure_rates: [-1; 5],
        };
        let scores = score_facilities(&input, &p());
        assert_eq!(
            turn_suggestion(&scores, input.failure_rates, false, &p()),
            TurnSuggestion::Rest
        );
    }

    #[test]
    fn target_clamps_before_cap() {
        // Target 900 below cap 1200: gain past 900 is wasted even though under cap.
        let gains = one_facility(0, [40, 0, 0, 0, 0]);
        let capped_at_target = projected_eval_delta([880, 0, 0, 0, 0], gains[0], [1200, 0, 0, 0, 0], [900, 0, 0, 0, 0]);
        let capped_at_cap = projected_eval_delta([880, 0, 0, 0, 0], gains[0], [1200, 0, 0, 0, 0], [0, 0, 0, 0, 0]);
        assert!(capped_at_target < capped_at_cap);
    }

    #[test]
    fn params_clamp_to_sane_ranges() {
        set_params(RecommendParams {
            risk_threshold_pct: 250,
            all_risky_pct: -5,
            mood_drop_penalty: -100,
            failure_stat_loss: -1,
        });
        let got = params();
        assert_eq!(got.risk_threshold_pct, 100);
        assert_eq!(got.all_risky_pct, 0);
        assert_eq!(got.mood_drop_penalty, 0);
        assert_eq!(got.failure_stat_loss, 0);
        set_params(RecommendParams::default());
    }
}
