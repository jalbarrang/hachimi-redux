//! Shared widget helpers for the Control Center's egui-native tabs.

use crate::core::gui::scale::get_scale;
use crate::core::gui::theme::ThemeTokens;

/// Toggle switch (checkbox-style). Returns `true` when the value changed.
pub(crate) fn toggle(ui: &mut egui::Ui, label: &str, value: &mut bool) -> bool {
    ui.checkbox(value, label).changed()
}

/// Labelled slider. Returns `true` when the value changed.
pub(crate) fn slider_f32(ui: &mut egui::Ui, value: &mut f32, range: std::ops::RangeInclusive<f32>, step: f64) -> bool {
    ui.add(egui::Slider::new(value, range).step_by(step).trailing_fill(true))
        .changed()
}

/// Ghost button (borderless, icon/text only).
pub(crate) fn ghost_button(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    let tokens = ThemeTokens::from_ui(ui);
    ui.add(egui::Button::new(egui::RichText::new(text).color(tokens.text_dim)).frame(false))
}

/// Begin a two-column settings grid (label + control). Call `end_row()` after
/// each label+control pair.
pub(crate) fn settings_grid(ui: &mut egui::Ui, id: impl std::hash::Hash, add_body: impl FnOnce(&mut egui::Ui)) {
    let scale = get_scale(ui.ctx());
    egui::Grid::new(id)
        .num_columns(2)
        .spacing([8.0 * scale, 6.0 * scale])
        .min_col_width(140.0 * scale)
        .striped(false)
        .show(ui, add_body);
}

/// Muted label for the left column of a settings grid.
pub(crate) fn settings_label(ui: &mut egui::Ui, text: &str) {
    let tokens = ThemeTokens::from_ui(ui);
    ui.label(egui::RichText::new(text).color(tokens.text_dim));
}

/// Accent-colored section heading (bold, with spacing).
pub(crate) fn settings_section(ui: &mut egui::Ui, text: &str) {
    let tokens = ThemeTokens::from_ui(ui);
    let scale = get_scale(ui.ctx());
    ui.add_space(8.0 * scale);
    ui.label(
        egui::RichText::new(text)
            .color(tokens.accent)
            .strong()
            .size(15.0 * scale),
    );
    ui.add_space(4.0 * scale);
}

/// Combo box backed by a `&[(T, &str)]` choice list. Returns `true` when the
/// selection changed.
pub(crate) fn combo<T: PartialEq + Copy>(
    ui: &mut egui::Ui,
    id_salt: impl std::hash::Hash,
    value: &mut T,
    choices: &[(T, &str)],
) -> bool {
    let selected = choices.iter().find(|(v, _)| v == value).map_or("Unknown", |(_, s)| s);

    let mut changed = false;
    egui::ComboBox::new(ui.id().with(id_salt), "")
        .wrap_mode(egui::TextWrapMode::Wrap)
        .selected_text(selected)
        .show_ui(ui, |ui| {
            for (v, label) in choices {
                changed |= ui.selectable_value(value, *v, *label).changed();
            }
        });
    changed
}
