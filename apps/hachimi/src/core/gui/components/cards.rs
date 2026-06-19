use super::super::theme::ThemeTokens;

pub(crate) fn card_frame(ui: &egui::Ui) -> egui::Frame {
    let tokens = ThemeTokens::from_ui(ui);

    egui::Frame::new()
        .fill(tokens.surface)
        .stroke(egui::Stroke::new(1.0, tokens.line))
        .inner_margin(egui::Margin::symmetric(12, 10))
}
