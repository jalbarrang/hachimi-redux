use super::super::theme::ThemeTokens;
use super::cards::card_frame;

pub(crate) fn empty_state(ui: &mut egui::Ui, text: impl Into<String>) {
    let tokens = ThemeTokens::from_ui(ui);
    card_frame(ui).show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new(text.into()).color(tokens.text_dim));
        });
    });
}
