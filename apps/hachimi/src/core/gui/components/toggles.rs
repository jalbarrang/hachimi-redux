use super::super::theme::ThemeTokens;

/// Canonical egui toggle-switch widget (adapted from the egui demo).
/// Flips `*on` when clicked and animates the knob between states.
pub(crate) fn toggle_ui(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let tokens = ThemeTokens::from_ui(ui);
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, ""));

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        let fill = if *on {
            tokens.accent
        } else if response.hovered() {
            tokens.surface_hi
        } else {
            tokens.surface
        };
        ui.painter().rect(
            rect,
            radius,
            fill,
            egui::Stroke::new(1.0, if *on { tokens.accent_2 } else { tokens.line }),
            egui::epaint::StrokeKind::Inside,
        );
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        ui.painter().circle_filled(center, 0.72 * radius, tokens.text);
    }

    response
}

/// A labelled toggle row: `[label .................... (switch)]`.
/// Returns the switch [`egui::Response`] so callers can react to `.changed()`.
#[allow(dead_code)]
pub(crate) fn toggle_row(ui: &mut egui::Ui, label: impl Into<egui::WidgetText>, on: &mut bool) -> egui::Response {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| toggle_ui(ui, on))
            .inner
    })
    .inner
}
