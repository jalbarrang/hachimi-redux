//! User-toggleable per-uma metrics, persisted to `hachimi/race_hud_config.json`
//! (next to Hachimi's own `config.json`).
//!
//! The visible-metric set is a small bitmask held in an atomic so the render
//! thread reads it lock-free; the L1 control page flips bits and persists.

use std::sync::atomic::{AtomicU8, Ordering};

use serde::{Deserialize, Serialize};

/// A toggleable metric shown inside each per-uma widget.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Metric {
    Hp = 0,
    Velocity = 1,
    Acceleration = 2,
    States = 3,
    Recoveries = 4,
}

impl Metric {
    /// All metrics in display order, paired with their control-page label.
    pub const ALL: [(Metric, &'static str); 5] = [
        (Metric::Hp, "HP"),
        (Metric::Velocity, "Velocity"),
        (Metric::Acceleration, "Acceleration"),
        (Metric::States, "States (kakari / blocked)"),
        (Metric::Recoveries, "Recoveries"),
    ];

    fn bit(self) -> u8 {
        1 << (self as u8)
    }
}

/// Mask with every metric shown (the default).
const ALL_MASK: u8 = 0b1_1111;

static SHOWN: AtomicU8 = AtomicU8::new(ALL_MASK);

/// Whether `metric` is currently shown in the widgets.
#[must_use]
pub fn is_shown(metric: Metric) -> bool {
    SHOWN.load(Ordering::Relaxed) & metric.bit() != 0
}

/// Show or hide `metric` (callers persist afterwards).
pub fn set_shown(metric: Metric, shown: bool) {
    let mut mask = SHOWN.load(Ordering::Relaxed);
    if shown {
        mask |= metric.bit();
    } else {
        mask &= !metric.bit();
    }
    SHOWN.store(mask & ALL_MASK, Ordering::Relaxed);
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedSettings {
    #[serde(default = "default_shown")]
    shown_metrics: u8,
}

fn default_shown() -> u8 {
    ALL_MASK
}

fn config_path() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|dir| dir.join("hachimi").join("race_hud_config.json")))
        .unwrap_or_else(|| std::path::PathBuf::from("race_hud_config.json"))
}

/// Load persisted settings. A missing/invalid file leaves the defaults intact.
pub fn load() {
    let path = config_path();
    let Ok(bytes) = std::fs::read(&path) else {
        return;
    };
    match serde_json::from_slice::<PersistedSettings>(&bytes) {
        Ok(cfg) => SHOWN.store(cfg.shown_metrics & ALL_MASK, Ordering::Relaxed),
        Err(e) => hlog_warn!(target: "race-hud", "config parse failed: {e}"),
    }
}

/// Persist the current visible-metric mask to disk. Call after a settings edit.
pub fn persist() {
    let cfg = PersistedSettings {
        shown_metrics: SHOWN.load(Ordering::Relaxed) & ALL_MASK,
    };
    let Ok(bytes) = serde_json::to_vec_pretty(&cfg) else {
        return;
    };
    let path = config_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = std::fs::write(&path, bytes) {
        hlog_warn!(target: "race-hud", "config persist failed: {e}");
    }
}
