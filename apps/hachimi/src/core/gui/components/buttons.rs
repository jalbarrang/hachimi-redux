use super::super::theme::ThemeTokens;

#[derive(Clone, Copy)]
pub(crate) enum PillButtonKind {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

pub(crate) fn pill_button(ui: &mut egui::Ui, text: impl Into<String>, kind: PillButtonKind) -> egui::Response {
    let tokens = ThemeTokens::from_ui(ui);
    let (fill, stroke, text_color) = match kind {
        PillButtonKind::Primary => (
            tokens.accent,
            egui::Stroke::new(1.0, tokens.accent_2),
            tokens.accent_ink,
        ),
        PillButtonKind::Secondary => (tokens.surface, egui::Stroke::new(1.0, tokens.line), tokens.text),
        PillButtonKind::Ghost => (
            egui::Color32::TRANSPARENT,
            egui::Stroke::new(1.0, tokens.line),
            tokens.text_dim,
        ),
        PillButtonKind::Danger => (
            tokens.crit,
            egui::Stroke::new(1.0, tokens.crit.linear_multiply(0.8)),
            egui::Color32::WHITE,
        ),
    };

    ui.add(
        egui::Button::new(egui::RichText::new(text.into()).color(text_color).strong())
            .fill(fill)
            .stroke(stroke)
            .corner_radius(tokens.pill_radius)
            .min_size(egui::vec2(0.0, 26.0)),
    )
}

pub(crate) fn primary_button(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    pill_button(ui, text, PillButtonKind::Primary)
}

pub(crate) fn secondary_button(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    pill_button(ui, text, PillButtonKind::Secondary)
}

pub(crate) fn ghost_button(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    pill_button(ui, text, PillButtonKind::Ghost)
}

pub(crate) fn danger_button(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    pill_button(ui, text, PillButtonKind::Danger)
}
