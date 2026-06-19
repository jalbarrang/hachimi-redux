//! Layout debug instrumentation for the Control Center and tab bodies.
//!
//! Enable with `HACHIMI_GUI_DEBUG=1` to draw labeled colored borders around
//! taffy/egui boxes and log width readouts to stderr.

/// Whether the GUI layout debug overlay is enabled (set `HACHIMI_GUI_DEBUG=1`).
///
/// When on, the Control Center shell draws labeled colored borders around each
/// taffy/egui box (card, tab bar, body, footer, header-row children). Invaluable
/// for diagnosing taffy width/flex issues since the resolved rects are otherwise
/// invisible. Cheap runtime env check; works in-game too, not just the preview.
pub(crate) fn gui_debug_enabled() -> bool {
    std::env::var_os("HACHIMI_GUI_DEBUG").is_some()
}
