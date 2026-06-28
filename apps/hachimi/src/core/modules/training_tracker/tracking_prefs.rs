//! Tracking lifecycle preferences.
//!
//! `auto_track_careers` lets the tracker follow the game's career lifecycle: start
//! once the game reports a career, and stop once it leaves career mode. The manual
//! button remains an override; this flag controls only the automatic handlers.

use std::sync::atomic::{AtomicBool, Ordering};

const DEFAULT_AUTO_TRACK_CAREERS: bool = true;

static AUTO_TRACK_CAREERS: AtomicBool = AtomicBool::new(DEFAULT_AUTO_TRACK_CAREERS);

pub(crate) fn auto_track_careers() -> bool {
    AUTO_TRACK_CAREERS.load(Ordering::Relaxed)
}

pub(crate) fn set_auto_track_careers(value: bool) {
    AUTO_TRACK_CAREERS.store(value, Ordering::Relaxed);
}

pub(crate) fn default_auto_track_careers() -> bool {
    DEFAULT_AUTO_TRACK_CAREERS
}
