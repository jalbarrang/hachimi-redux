//! Tiny egui-native widget helpers for plugins.
//!
//! Immediate-mode `fn(&mut egui::Ui, ...)` draws — no retained tree. Plugins cast
//! the host `Ui` pointer with [`crate::ui_from_ptr`] and call these directly.

/// Design tokens as `egui::Color32` constants so plugins style egui draws
/// without parsing hex strings.
pub mod theme {
    use egui::Color32;

    /// App background (`--color-bg`).
    pub const BG: Color32 = Color32::from_rgb(0x0b, 0x0e, 0x13);
    /// Raised panel surface (`--color-surface-1`).
    pub const SURFACE_1: Color32 = Color32::from_rgb(0x15, 0x1a, 0x23);
    /// Nested panel / chip surface (`--color-surface-2`).
    pub const SURFACE_2: Color32 = Color32::from_rgb(0x1c, 0x22, 0x30);
    /// Divider / border line.
    pub const LINE: Color32 = Color32::from_rgb(0x2c, 0x36, 0x48);

    /// Primary foreground text.
    pub const FG: Color32 = Color32::from_rgb(0xea, 0xef, 0xf6);
    /// Muted foreground text.
    pub const FG_MUTED: Color32 = Color32::from_rgb(0xa3, 0xb1, 0xc4);
    /// Dim foreground text.
    pub const FG_DIM: Color32 = Color32::from_rgb(0x6e, 0x7d, 0x92);

    /// Blue accent.
    pub const ACCENT: Color32 = Color32::from_rgb(0x5f, 0xb2, 0xff);
    /// Green success / primary CTA.
    pub const GOOD: Color32 = Color32::from_rgb(0x4f, 0xbb, 0x4f);
    /// Amber warning.
    pub const WARN: Color32 = Color32::from_rgb(0xff, 0xb0, 0x4d);
    /// Red danger.
    pub const BAD: Color32 = Color32::from_rgb(0xff, 0x7a, 0x6b);
}

/// A labeled toggle (checkbox + label). Returns `Some(new_state)` only when the
/// user flipped it this frame, so the caller can persist on change:
///
/// ```ignore
/// if let Some(on) = widgets::toggle(ui, "Show HP", shown) {
///     set_shown(on);
/// }
/// ```
pub fn toggle(ui: &mut egui::Ui, label: &str, checked: bool) -> Option<bool> {
    let mut value = checked;
    let response = ui.checkbox(&mut value, egui::RichText::new(label).color(theme::FG).size(14.0));
    response.changed().then_some(value)
}

/// A full-width 1px divider line in the theme line colour.
pub fn separator(ui: &mut egui::Ui) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, theme::LINE);
}

/// A titled, framed panel: surface fill + line border + bold title, with the body
/// drawn by `add_body`. Mirrors the old window-chrome component.
pub fn window_chrome(ui: &mut egui::Ui, title: &str, add_body: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(theme::SURFACE_1)
        .stroke(egui::Stroke::new(1.0, theme::LINE))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(title).color(theme::FG).strong());
            ui.add_space(4.0);
            add_body(ui);
        });
}
