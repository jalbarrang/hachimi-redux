//! Tiny egui-native widget helpers for plugins.
//!
//! Immediate-mode `fn(&mut egui::Ui, ...)` draws — no retained tree. Plugins cast
//! the host `Ui` pointer with [`crate::ui_from_ptr`] and call these directly.

/// Design tokens as `egui::Color32` constants so plugins style egui draws
/// without parsing hex strings.
pub mod theme {
    use egui::Color32;

    const TOKENS: honse_ui::theme::Tokens = honse_ui::theme::Tokens::DEFAULT;

    /// App background (`--color-bg`).
    pub const BG: Color32 = TOKENS.bg;
    /// Raised panel surface (`--color-surface-1`).
    pub const SURFACE_1: Color32 = TOKENS.surface_1;
    /// Nested panel / chip surface (`--color-surface-2`).
    pub const SURFACE_2: Color32 = TOKENS.surface_2;
    /// Divider / border line.
    pub const LINE: Color32 = TOKENS.line;

    /// Primary foreground text.
    pub const FG: Color32 = TOKENS.fg;
    /// Muted foreground text.
    pub const FG_MUTED: Color32 = TOKENS.fg_muted;
    /// Dim foreground text.
    pub const FG_DIM: Color32 = TOKENS.fg_dim;

    /// Blue accent.
    pub const ACCENT: Color32 = TOKENS.accent;
    /// Green success / primary CTA.
    pub const GOOD: Color32 = TOKENS.good;
    /// Amber warning.
    pub const WARN: Color32 = TOKENS.warn;
    /// Red danger.
    pub const BAD: Color32 = TOKENS.bad;

    pub use honse_ui::theme::{mood_color, stat_color, Tokens};
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
    honse_ui::components::toggle(ui, label, checked)
}

/// A full-width 1px divider line in the theme line colour.
pub fn separator(ui: &mut egui::Ui) {
    let _ = honse_ui::components::separator(ui);
}

/// A titled, framed panel: surface fill + line border + bold title, with the body
/// drawn by `add_body`. Mirrors the old window-chrome component.
pub fn window_chrome(ui: &mut egui::Ui, title: &str, add_body: impl FnOnce(&mut egui::Ui)) {
    honse_ui::components::window_chrome(ui, title, add_body);
}
