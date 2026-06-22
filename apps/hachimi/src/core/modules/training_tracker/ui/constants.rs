//! Shared UI sizing and overlay identifiers.

/// Default overlay font size.
pub(super) const OVERLAY_FONT_SIZE: f32 = 16.0;
/// Default fixed overlay content width at zoom 1.0 (points). Each panel is a
/// fixed-width column with auto height; the zoom slider scales this.
pub(super) const OVERLAY_BASE_WIDTH: f32 = 500.0;
pub(super) const ENERGY_BASE_WIDTH: f32 = 140.0;
pub(super) const TRAINING_BASE_WIDTH: f32 = 500.0;
pub(super) const BONDS_BASE_WIDTH: f32 = 500.0;
pub(super) const SCENARIO_BASE_WIDTH: f32 = 500.0;
pub(super) const SHOP_BASE_WIDTH: f32 = 500.0;
/// Minimum height for scrollable lists (Skills/Shop tabs, Bonds section) when the
/// panel is small, in points. Lists otherwise grow to fill the resizable panel.
pub(super) const MIN_LIST_HEIGHT: f32 = 400.0;
/// Maximum overlay height at zoom 1.0 (points). The panel caps here instead of
/// growing unbounded with content (which made the host window scroll the whole
/// overlay); tab bodies scroll internally within the remaining space.
pub(super) const OVERLAY_MAX_HEIGHT: f32 = 900.0;

pub(super) const ENERGY_OVERLAY_ID: &str = "training_tracker_overlay_energy";
pub(super) const TRAINING_OVERLAY_ID: &str = "training_tracker_overlay_training";
pub(super) const BONDS_OVERLAY_ID: &str = "training_tracker_overlay_bonds";
pub(super) const SCENARIO_OVERLAY_ID: &str = "training_tracker_overlay_scenario";
pub(super) const SHOP_OVERLAY_ID: &str = "training_tracker_overlay_shop";

pub(super) const PANEL_IDS: [&str; 5] = [
    ENERGY_OVERLAY_ID,
    TRAINING_OVERLAY_ID,
    BONDS_OVERLAY_ID,
    SCENARIO_OVERLAY_ID,
    SHOP_OVERLAY_ID,
];
