/// Accent section banner. Used in place of the old heading + separator.
pub(crate) fn section_banner(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    let green = egui::Color32::from_rgb(64, 160, 76);
    let ink = egui::Color32::WHITE;
    let avail = ui.available_width();
    let desired = egui::vec2(avail, 28.0);
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::hover());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, green);

        let galley = ui
            .painter()
            .layout_no_wrap(text.into(), egui::TextStyle::Button.resolve(ui.style()), ink);
        let text_pos = egui::pos2(rect.left() + 8.0, rect.center().y - galley.size().y / 2.0);
        painter.galley(text_pos, galley, ink);
    }

    if crate::core::gui::debug::gui_debug_enabled() {
        // Magenta outline of the *allocated* banner rect + a width readout, so an
        // in-game/preview screenshot shows exactly how wide the banner became and
        // which source width it came from (avail / max_rect / clip / cursor). This is
        // how the overflow root cause was found: a vertical `ScrollArea` grows its
        // content ui to fit its widest child, so `available_width()` here can exceed
        // the shell unless every child pins its own width.
        let max_rect_w = ui.max_rect().width();
        let clip_w = ui.clip_rect().width();
        let cursor_x = ui.cursor().min.x;
        ui.painter().rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 0, 255)),
            egui::StrokeKind::Outside,
        );
        ui.painter().text(
            rect.left_bottom() + egui::vec2(2.0, 1.0),
            egui::Align2::LEFT_TOP,
            format!(
                "banner avail={avail:.0} max={max_rect_w:.0} clip={clip_w:.0} curx={cursor_x:.0} rectw={:.0}",
                rect.width()
            ),
            egui::FontId::monospace(10.0),
            egui::Color32::from_rgb(255, 0, 255),
        );
        eprintln!(
            "[gui-debug] section_banner avail={avail:.1} max_rect_w={max_rect_w:.1} clip_w={clip_w:.1} cursor_x={cursor_x:.1} rect=[{:.1}..{:.1}] w={:.1}",
            rect.left(),
            rect.right(),
            rect.width(),
        );
    }

    response
}

/// Consistent section header used across the Control Center tabs.
pub(crate) fn section_header(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.add_space(6.0);
    section_banner(ui, text);
    ui.add_space(6.0);
}
