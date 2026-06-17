use rust_i18n::t;

use crate::core::gui::scale::get_scale;
use crate::core::gui::Gui;

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
        let mut selected = "Unknown";
        for choice in choices.iter() {
            if *value == choice.0 {
                selected = choice.1;
            }
        }

        let mut changed = false;
        egui::ComboBox::new(ui.id().with(id_child), "")
            .wrap_mode(egui::TextWrapMode::Wrap)
            .selected_text(selected)
            .show_ui(ui, |ui| {
                for choice in choices.iter() {
                    changed |= ui.selectable_value(value, choice.0, choice.1).changed();
                }
            });

        changed
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

    // egui's code originally (https://github.com/emilk/egui/blob/main/crates/egui/src/containers/combo_box.rs)
    pub(crate) fn down_triangle_icon(painter: &egui::Painter, rect: egui::Rect, visuals: &egui::style::WidgetVisuals) {
        let rect = egui::Rect::from_center_size(rect.center(), egui::vec2(rect.width() * 0.7, rect.height() * 0.45));

        painter.add(egui::Shape::convex_polygon(
            vec![rect.left_top(), rect.right_top(), rect.center_bottom()],
            visuals.fg_stroke.color,
            visuals.fg_stroke,
        ));
    }
}
