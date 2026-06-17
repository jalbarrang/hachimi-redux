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

/// Debug-only: outline an egui `Ui`'s `max_rect` and label it with its width.
/// Gated behind [`gui_debug_enabled`].
pub(crate) fn dbg_outline(ui: &egui::Ui, color: egui::Color32, label: &str) {
    if !gui_debug_enabled() {
        return;
    }
    let r = ui.max_rect();
    eprintln!(
        "[gui-debug] {label}: max_rect=[{:.1}..{:.1}] w={:.1} avail_w={:.1} clip_w={:.1}",
        r.left(),
        r.right(),
        r.width(),
        ui.available_width(),
        ui.clip_rect().width(),
    );
    ui.painter()
        .rect_stroke(r, 0.0, egui::Stroke::new(1.5, color), egui::StrokeKind::Inside);
    ui.painter().text(
        r.left_top() + egui::vec2(1.0, 1.0),
        egui::Align2::LEFT_TOP,
        format!("{label} {:.0}", r.width()),
        egui::FontId::monospace(10.0),
        color,
    );
}
