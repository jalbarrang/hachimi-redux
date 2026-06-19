use super::super::theme::ThemeTokens;

#[allow(dead_code)]
pub(crate) fn stat_chip(ui: &mut egui::Ui, label: &str, value: impl ToString, delta: Option<&str>) -> egui::Response {
    let tokens = ThemeTokens::from_ui(ui);
    let desired = egui::vec2(72.0, 48.0);
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, tokens.small_radius, tokens.surface);
        ui.painter().rect_stroke(
            rect,
            tokens.small_radius,
            egui::Stroke::new(1.0, tokens.line),
            egui::epaint::StrokeKind::Inside,
        );
        let label_galley = ui.painter().layout_no_wrap(
            label.to_owned(),
            egui::TextStyle::Small.resolve(ui.style()),
            tokens.text_dim,
        );
        let value_galley = ui.painter().layout_no_wrap(
            value.to_string(),
            egui::TextStyle::Button.resolve(ui.style()),
            tokens.text,
        );
        ui.painter().galley(
            egui::pos2(rect.center().x - label_galley.size().x / 2.0, rect.top() + 6.0),
            label_galley,
            tokens.text_dim,
        );
        ui.painter().galley(
            egui::pos2(rect.center().x - value_galley.size().x / 2.0, rect.top() + 20.0),
            value_galley,
            tokens.text,
        );
        if let Some(delta) = delta {
            let delta_galley = ui.painter().layout_no_wrap(
                delta.to_owned(),
                egui::TextStyle::Small.resolve(ui.style()),
                tokens.accent,
            );
            ui.painter().galley(
                egui::pos2(rect.center().x - delta_galley.size().x / 2.0, rect.bottom() - 14.0),
                delta_galley,
                tokens.accent,
            );
        }
    }
    response
}

#[allow(dead_code)]
pub(crate) fn icon_tile(
    ui: &mut egui::Ui,
    icon: impl Into<String>,
    label: impl Into<String>,
    badge: Option<&str>,
) -> egui::Response {
    let tokens = ThemeTokens::from_ui(ui);
    let desired = egui::vec2(160.0, 54.0);
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click());
    let fill = if response.hovered() {
        tokens.surface_hi
    } else {
        tokens.surface
    };

    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, tokens.card_radius, fill);
        ui.painter().rect_stroke(
            rect,
            tokens.card_radius,
            egui::Stroke::new(1.0, if response.hovered() { tokens.accent } else { tokens.line }),
            egui::epaint::StrokeKind::Inside,
        );

        let icon_rect = egui::Rect::from_min_size(rect.min + egui::vec2(10.0, 8.0), egui::vec2(38.0, 38.0));
        ui.painter()
            .rect_filled(icon_rect, tokens.small_radius, tokens.surface_hi);
        ui.painter().rect_stroke(
            icon_rect,
            tokens.small_radius,
            egui::Stroke::new(1.0, tokens.line),
            egui::epaint::StrokeKind::Inside,
        );
        let icon_galley =
            ui.painter()
                .layout_no_wrap(icon.into(), egui::TextStyle::Button.resolve(ui.style()), tokens.text);
        ui.painter()
            .galley(icon_rect.center() - icon_galley.size() / 2.0, icon_galley, tokens.text);

        let label_galley = egui::WidgetText::from(label.into()).into_galley(
            ui,
            Some(egui::TextWrapMode::Wrap),
            rect.width() - 58.0,
            egui::TextStyle::Button,
        );
        ui.painter().galley(
            egui::pos2(rect.left() + 58.0, rect.center().y - label_galley.size().y / 2.0),
            label_galley,
            tokens.text,
        );

        if let Some(badge) = badge {
            let badge_rect = egui::Rect::from_min_size(rect.min + egui::vec2(6.0, -5.0), egui::vec2(34.0, 14.0));
            ui.painter().rect_filled(badge_rect, 4.0, tokens.crit);
            ui.painter().text(
                badge_rect.center(),
                egui::Align2::CENTER_CENTER,
                badge,
                egui::TextStyle::Small.resolve(ui.style()),
                egui::Color32::WHITE,
            );
        }
    }

    response
}
