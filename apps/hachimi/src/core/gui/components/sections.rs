use crate::core::gui::scale::get_scale;

/// Section title used across the native Control Center tabs. Renders a left-aligned
/// bold heading (h3-style) — no full-width fill, so it never overflows the shell the
/// way a stretched banner did. Returns the label `Response`.
pub(crate) fn section_banner(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    let scale = get_scale(ui.ctx());
    ui.add(egui::Label::new(egui::RichText::new(text).strong().size(15.0 * scale)))
}

/// Consistent section header used across the Control Center tabs.
pub(crate) fn section_header(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.add_space(8.0);
    section_banner(ui, text);
    ui.add_space(4.0);
}
