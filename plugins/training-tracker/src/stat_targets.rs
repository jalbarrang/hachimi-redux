//! User-configurable per-stat training targets for the cap/target warning.
//!
//! A target of `0` means "no target" — fall back to the live game cap. A positive
//! target gives an earlier warning (e.g. stop Stamina at 600 even though the cap is
//! higher).
//!
//! Persisted to `hachimi/training_config.json` (alongside Hachimi's own config):
//! [`load`] on plugin init, [`persist`] when the user commits an edit.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};

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

// ---------------------------------------------------------------------------
// Persistence (hachimi/training_config.json, next to Hachimi's config.json)
// ---------------------------------------------------------------------------

/// On-disk plugin config. Kept as a struct so more settings can be added later.
#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistedConfig {
    #[serde(default)]
    stat_targets: [i32; 5],
}

fn config_path() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|dir| dir.join("hachimi").join("training_config.json")))
        .unwrap_or_else(|| std::path::PathBuf::from("training_config.json"))
}

/// Load persisted targets into memory. Call once on plugin init. Missing/invalid
/// file is fine (targets stay all-zero).
pub fn load() {
    let path = config_path();
    let Ok(bytes) = std::fs::read(&path) else {
        return;
    };
    match serde_json::from_slice::<PersistedConfig>(&bytes) {
        Ok(cfg) => {
            set_targets(cfg.stat_targets);
            hlog_info!(target: "training-tracker", "stat targets loaded: {:?}", targets());
        }
        Err(e) => hlog_warn!(target: "training-tracker", "stat targets config parse failed: {e}"),
    }
}

/// Write the current targets to disk. Call when the user commits an edit.
pub fn persist() {
    let cfg = PersistedConfig {
        stat_targets: targets(),
    };
    let Ok(bytes) = serde_json::to_vec_pretty(&cfg) else {
        return;
    };
    let path = config_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = std::fs::write(&path, bytes) {
        hlog_warn!(target: "training-tracker", "stat targets persist failed: {e}");
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
