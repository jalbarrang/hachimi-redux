//! Placeholder tab bodies for off-game preview.

pub(crate) fn stub_tab(ui: &mut egui::Ui, title: &str, note: &str) {
    ui.add_space(12.0);
    ui.heading(title);
    ui.add_space(6.0);
    ui.weak(note);
}
