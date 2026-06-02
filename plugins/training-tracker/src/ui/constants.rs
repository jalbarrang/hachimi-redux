//! Shared UI sizing and overlay identifiers.

/// Default overlay font size.
pub(super) const OVERLAY_FONT_SIZE: f32 = 12.0;
pub(super) const OVERLAY_MIN_WIDTH: f32 = 340.0;
/// Minimum height for scrollable lists (Skills/Shop tabs, Bonds section) when the
/// panel is small, in points. Lists otherwise grow to fill the resizable panel.
pub(super) const MIN_LIST_HEIGHT: f32 = 80.0;

/// The overlay ID used during registration — must match for show/hide calls.
pub(super) const OVERLAY_ID: &str = "training_tracker_overlay";
