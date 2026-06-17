use super::super::theme::ThemeTokens;

pub(crate) fn card_frame(ui: &egui::Ui) -> egui::Frame {
    let tokens = ThemeTokens::from_ui(ui);

    egui::Frame::new()
        .fill(tokens.surface)
        .stroke(egui::Stroke::new(1.0, tokens.line))
        .inner_margin(egui::Margin::symmetric(12, 10))
}

/// Taffy-native equivalent of [`card_frame`]: a flex-column node that paints the
/// card surface + 1px outline as its own background, so its children participate
/// directly in the taffy layout (flex grow/shrink, gaps) instead of being a
/// single opaque egui blob. The 12/10 padding mirrors `card_frame`'s inner margin.
pub(crate) fn card_node<'r, R>(
    tui: impl egui_taffy::TuiBuilderLogic<'r>,
    content: impl FnOnce(&mut egui_taffy::Tui) -> R,
) -> R {
    use egui_taffy::taffy::prelude::{auto, length, percent};
    use egui_taffy::{taffy, TaffyContainerUi, TuiBuilderLogic};

    fn paint_card(ui: &mut egui::Ui, container: &TaffyContainerUi) {
        let tokens = ThemeTokens::from_ui(ui);
        let rect = container.full_container();
        ui.painter().rect(
            rect,
            tokens.card_radius,
            tokens.surface,
            egui::Stroke::new(1.0, tokens.line),
            egui::StrokeKind::Inside,
        );
        if crate::core::gui::debug::gui_debug_enabled() {
            ui.painter().rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::RED),
                egui::StrokeKind::Inside,
            );
            ui.painter().text(
                rect.left_top() + egui::vec2(2.0, 2.0),
                egui::Align2::LEFT_TOP,
                format!("card w={:.0}", rect.width()),
                egui::FontId::monospace(11.0),
                egui::Color32::RED,
            );
        }
    }

    tui.style(taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Column,
        size: taffy::Size {
            width: percent(1.0),
            height: auto(),
        },
        padding: taffy::Rect {
            left: length(12.0),
            right: length(12.0),
            top: length(10.0),
            bottom: length(10.0),
        },
        ..Default::default()
    })
    .add_with_background_ui(paint_card, |tui, _| content(tui))
    .main
}
