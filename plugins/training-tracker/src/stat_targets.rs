//! User-configurable per-stat training targets for the cap/target warning.
//!
//! A target of `0` means "no target" — fall back to the live game cap. A positive
//! target gives an earlier warning (e.g. stop Stamina at 600 even though the cap is
//! higher).
//!
//! In-memory state only; persistence to `training_config.json` is owned by
//! [`crate::config`].

use std::sync::Mutex;

/// Stat order: [Speed, Stamina, Power, Guts, Wit].
pub const LABELS: [&str; 5] = ["Speed", "Stamina", "Power", "Guts", "Wit"];

/// Upper bound for a target (matches the highest reachable stat cap).
pub const MAX_TARGET: i32 = 2000;

static TARGETS: Mutex<[i32; 5]> = Mutex::new([0; 5]);

/// Current per-stat targets (`0` = use game cap).
pub fn targets() -> [i32; 5] {
    TARGETS.lock().ok().map(|g| *g).unwrap_or([0; 5])
}

/// Replace all targets (values are clamped to `0..=MAX_TARGET`).
pub fn set_targets(new: [i32; 5]) {
    if let Ok(mut g) = TARGETS.lock() {
        for (slot, v) in g.iter_mut().zip(new) {
            *slot = v.clamp(0, MAX_TARGET);
        }
    }
}

/// Effective warning threshold for a stat: the target when set, else the game cap.
pub fn effective_threshold(target: i32, cap: i32) -> i32 {
    if target > 0 {
        target
    } else {
        cap
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_threshold_prefers_target() {
        assert_eq!(effective_threshold(600, 1200), 600); // target set
        assert_eq!(effective_threshold(0, 1200), 1200); // unset → cap
        assert_eq!(effective_threshold(0, 0), 0); // both unknown
        assert_eq!(effective_threshold(900, 0), 900); // target even with unknown cap
    }

    #[test]
    fn set_targets_clamps() {
        set_targets([5000, -10, 1200, 0, 600]);
        let t = targets();
        assert_eq!(t[0], MAX_TARGET); // clamped down
        assert_eq!(t[1], 0); // clamped up from negative
        assert_eq!(t[2], 1200);
        assert_eq!(t[4], 600);
        set_targets([0; 5]); // reset for other tests
    }
}
