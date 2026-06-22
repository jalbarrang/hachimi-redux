//! Host-facing re-exports for shared Honse UI components.

#[allow(unused_imports)]
pub(crate) use honse_ui::components::{
    badge, card_frame, chip, combo, danger_button, empty_state, pill, pill_button, primary_button, row_frame,
    secondary_button, separator, stat_chip, stat_chip_chrome, window_chrome, PillButtonKind,
};

use rust_i18n::t;

use crate::core::gui::Gui;

use crate::core::gui::scale::get_scale;
use honse_ui::theme::Tokens;

/// Toggle switch (checkbox-style). Returns `true` when the value changed.
pub(crate) fn toggle(ui: &mut egui::Ui, label: &str, value: &mut bool) -> bool {
    if let Some(new_value) = honse_ui::components::toggle(ui, label, *value) {
        *value = new_value;
        true
    } else {
        false
    }
}

/// Labelled slider. Returns `true` when the value changed.
pub(crate) fn slider_f32(ui: &mut egui::Ui, value: &mut f32, range: std::ops::RangeInclusive<f32>, step: f64) -> bool {
    honse_ui::components::slider_f32(ui, value, range, step)
}

/// Ghost button (borderless, icon/text only).
pub(crate) fn ghost_button(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    ui.add(egui::Button::new(egui::RichText::new(text).color(Tokens::DEFAULT.fg_dim)).frame(false))
}

impl Gui {
    #[cfg(target_os = "windows")]
    pub(crate) fn run_vsync_combo(ui: &mut egui::Ui, value: &mut i32) {
        Self::run_combo(
            ui,
            "vsync_combo",
            value,
            &[
                (-1, &t!("default")),
                (0, &t!("off")),
                (1, &t!("on")),
                (2, "1/2"),
                (3, "1/3"),
                (4, "1/4"),
            ],
        );
    }

    pub(crate) fn run_combo<T: PartialEq + Copy>(
        ui: &mut egui::Ui,
        id_child: impl std::hash::Hash,
        value: &mut T,
        choices: &[(T, &str)],
    ) -> bool {
        honse_ui::components::combo(ui, id_child, value, choices)
    }

    pub(crate) fn run_combo_menu<T: PartialEq + Copy>(
        ui: &mut egui::Ui,
        id_salt: impl std::hash::Hash,
        value: &mut T,
        choices: &[(T, &str)],
        search_term: &mut String,
    ) -> bool {
        let mut changed = false;
        let scale = get_scale(ui.ctx());
        let fixed_width = 145.0 * scale;
        let row_height = 24.0 * scale;
        let padding = ui.spacing().button_padding;

        let button_id = ui.make_persistent_id(id_salt);
        let popup_id = button_id.with("popup");

        let selected_text = choices.iter().find(|(v, _)| v == value).map_or("Unknown", |(_, s)| *s);

        let (rect, _) = ui.allocate_exact_size(egui::vec2(fixed_width, row_height), egui::Sense::hover());
        let button_res = ui.interact(rect, button_id, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let is_open = egui::Popup::is_id_open(ui.ctx(), popup_id);
            let visuals = if is_open {
                &ui.visuals().widgets.open
            } else {
                ui.style().interact(&button_res)
            };

            ui.painter().rect(
                rect.expand(visuals.expansion),
                visuals.corner_radius,
                visuals.weak_bg_fill,
                visuals.bg_stroke,
                egui::epaint::StrokeKind::Inside,
            );

            let icon_size = 12.0 * scale;
            let icon_rect = egui::Rect::from_center_size(
                egui::pos2(rect.right() - padding.x - icon_size / 2.0, rect.center().y),
                egui::vec2(icon_size, icon_size),
            );
            Self::down_triangle_icon(ui.painter(), icon_rect, visuals);

            let galley = ui.painter().layout_no_wrap(
                selected_text.to_owned(),
                egui::TextStyle::Button.resolve(ui.style()),
                visuals.text_color(),
            );

            let text_pos = egui::pos2(rect.left() + padding.x, rect.center().y - galley.size().y / 2.0);
            ui.painter().galley(text_pos, galley, visuals.text_color());
        }

        egui::Popup::menu(&button_res)
            .id(popup_id)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                ui.set_width(fixed_width);
                ui.set_max_width(fixed_width);

                ui.horizontal(|ui| {
                    ui.add_sized(
                        [ui.available_width() - 30.0 * scale, row_height],
                        egui::TextEdit::singleline(search_term).hint_text(t!("search_filter")),
                    );

                    if ui.button("X").clicked() {
                        search_term.clear();
                    }
                });

                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(250.0 * scale)
                    .hscroll(false)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                            for (choice_val, label) in choices {
                                if !search_term.is_empty()
                                    && !label.to_lowercase().contains(&search_term.to_lowercase())
                                {
                                    continue;
                                }

                                let is_selected = value == choice_val;
                                if ui.add(egui::Button::selectable(is_selected, *label)).clicked() {
                                    *value = *choice_val;
                                    changed = true;
                                    egui::Popup::close_id(ui.ctx(), popup_id);
                                    search_term.clear();
                                }
                            }
                        });
                    });
            });

        changed
    }

    pub(crate) fn down_triangle_icon(painter: &egui::Painter, rect: egui::Rect, visuals: &egui::style::WidgetVisuals) {
        let rect = egui::Rect::from_center_size(rect.center(), egui::vec2(rect.width() * 0.7, rect.height() * 0.45));

        painter.add(egui::Shape::convex_polygon(
            vec![rect.left_top(), rect.right_top(), rect.center_bottom()],
            visuals.fg_stroke.color,
            visuals.fg_stroke,
        ));
    }
}

/// Section title used across the native Control Center tabs.
pub(crate) fn section_banner(ui: &mut egui::Ui, text: impl Into<String>) -> egui::Response {
    let scale = get_scale(ui.ctx());
    ui.add(egui::Label::new(egui::RichText::new(text).strong().size(15.0 * scale)))
}

/// Consistent section header used across the Control Center tabs.
pub(crate) fn section_header(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.add_space(8.0);
    section_banner(ui, text);
    ui.add_space(4.0);
}

/// Begin a two-column settings grid (label + control). Call `end_row()` after each pair.
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
    ui.label(egui::RichText::new(text).color(Tokens::DEFAULT.fg_dim));
}

/// Accent-colored section heading (bold, with spacing).
pub(crate) fn settings_section(ui: &mut egui::Ui, text: &str) {
    let scale = get_scale(ui.ctx());
    ui.add_space(8.0 * scale);
    ui.label(
        egui::RichText::new(text)
            .color(Tokens::DEFAULT.accent)
            .strong()
            .size(15.0 * scale),
    );
    ui.add_space(4.0 * scale);
}
