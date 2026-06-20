//! Shared UI sizing and overlay identifiers.

/// Default overlay font size.
pub(super) const OVERLAY_FONT_SIZE: f32 = 12.0;
/// Fixed overlay content width at zoom 1.0 (points). The panel is a fixed-width
/// column with auto height; the zoom slider scales this (and the fonts/spacing).
pub(super) const OVERLAY_BASE_WIDTH: f32 = 500.0;
/// Minimum height for scrollable lists (Skills/Shop tabs, Bonds section) when the
/// panel is small, in points. Lists otherwise grow to fill the resizable panel.
pub(super) const MIN_LIST_HEIGHT: f32 = 400.0;
/// Maximum overlay height at zoom 1.0 (points). The panel caps here instead of
/// growing unbounded with content (which made the host window scroll the whole
/// overlay); tab bodies scroll internally within the remaining space.
pub(super) const OVERLAY_MAX_HEIGHT: f32 = 720.0;

/// The overlay ID used during registration — must match for show/hide calls.
pub(super) const OVERLAY_ID: &str = "training_tracker_overlay";
