//! L1 General tab — core config options + live overlay controls.

use std::thread;

use rust_i18n::t;

use crate::core::hachimi::{self, Language};
use crate::core::plugin::overlay;

use egui_taffy::Tui;

use super::super::scale::get_scale;
use super::super::widgets;
use super::super::window::{SimpleOkDialog, ThemeEditorWindow};
use super::super::Gui;
use super::layout::{auto_cell, content_width, fill_cell, label_cell};

pub(crate) fn options(config: &mut hachimi::Config, tui: &mut Tui) {
    label_cell(tui, t!("config_editor.language"));
    let lang_changed = auto_cell(tui, |ui| {
        Gui::run_combo(ui, "language", &mut config.language, Language::CHOICES)
    });
    if lang_changed {
        config.language.set_locale();
    }

    label_cell(tui, t!("config_editor.disable_overlay"));
    auto_cell(tui, |ui| {
        if ui.checkbox(&mut config.disable_gui, "").clicked() && config.disable_gui {
            thread::spawn(|| {
                Gui::instance()
                    .expect("unexpected failure")
                    .lock()
                    .expect("unexpected failure")
                    .show_window(Box::new(SimpleOkDialog::new(
                        &t!("warning"),
                        &t!("config_editor.disable_overlay_warning"),
                        || {},
                    )));
            });
        }
    });

    label_cell(tui, t!("config_editor.ipv4_only"));
    auto_cell(tui, |ui| {
        ui.checkbox(&mut config.ipv4_only, "");
    });

    label_cell(tui, t!("config_editor.gui_scale"));
    fill_cell(tui, |ui| {
        ui.add(egui::Slider::new(&mut config.gui_scale, 0.25..=2.0).step_by(0.05));
    });

    #[cfg(target_os = "windows")]
    {
        label_cell(tui, t!("config_editor.gui_landscape_ratio"));
        fill_cell(tui, |ui| {
            ui.add(
                egui::Slider::new(&mut config.windows.gui_landscape_ratio, 0.25..=1.0)
                    .step_by(0.05)
                    .fixed_decimals(2),
            );
        });
    }

    label_cell(tui, t!("theme_editor.title"));
    auto_cell(tui, |ui| {
        if widgets::secondary_button(ui, t!("open").into_owned()).clicked() {
            thread::spawn(|| {
                Gui::instance()
                    .expect("unexpected failure")
                    .lock()
                    .expect("unexpected failure")
                    .show_window(Box::new(ThemeEditorWindow::new()));
            });
        }
    });

    #[cfg(target_os = "windows")]
    {
        label_cell(tui, t!("config_editor.discord_rpc"));
        auto_cell(tui, |ui| {
            ui.checkbox(&mut config.windows.discord_rpc, "");
        });
    }

    label_cell(tui, t!("config_editor.debug_mode"));
    auto_cell(tui, |ui| {
        ui.checkbox(&mut config.debug_mode, "");
    });

    label_cell(tui, t!("config_editor.enable_file_logging"));
    auto_cell(tui, |ui| {
        ui.checkbox(&mut config.enable_file_logging, "");
    });

    label_cell(tui, t!("config_editor.apply_atlas_workaround"));
    auto_cell(tui, |ui| {
        ui.checkbox(&mut config.apply_atlas_workaround, "");
    });

    label_cell(tui, t!("config_editor.skip_first_time_setup"));
    auto_cell(tui, |ui| {
        ui.checkbox(&mut config.skip_first_time_setup, "");
    });

    label_cell(tui, t!("config_editor.disable_auto_update_check"));
    auto_cell(tui, |ui| {
        ui.checkbox(&mut config.disable_auto_update_check, "");
    });

    label_cell(tui, t!("config_editor.enable_ipc"));
    auto_cell(tui, |ui| {
        ui.checkbox(&mut config.enable_ipc, "");
    });

    label_cell(tui, t!("config_editor.ipc_listen_all"));
    auto_cell(tui, |ui| {
        ui.checkbox(&mut config.ipc_listen_all, "");
    });
}

/// Live overlay controls: global opacity + per-panel show/hide and reset-position.
pub(crate) fn overlays(ui: &mut egui::Ui, _ctx: &egui::Context) {
    let scale = get_scale(ui.ctx());
    super::super::menu::dbg_outline(ui, egui::Color32::from_rgb(255, 140, 0), "ov-outer");

    egui::Frame::NONE.show(ui, |ui| {
        ui.set_max_width(content_width(ui, scale));
        super::super::menu::dbg_outline(ui, egui::Color32::from_rgb(200, 0, 0), "ov-inner");
        widgets::section_header(ui, t!("config_editor.overlays_heading").into_owned());
        let mut opacity = overlay::opacity();
        ui.horizontal(|ui| {
            ui.label(t!("config_editor.overlay_opacity"));
            if ui
                .add(egui::Slider::new(&mut opacity, 0.1..=1.0).fixed_decimals(2))
                .changed()
            {
                overlay::set_opacity(opacity);
            }
        });

        let overlays = overlay::get_plugin_overlays();
        if overlays.is_empty() {
            ui.weak(t!("config_editor.overlays_none"));
            return;
        }
        for ov in &overlays {
            let title = overlay::display_title(&ov.id);
            let mut visible = overlay::is_overlay_visible(&ov.id);
            ui.horizontal(|ui| {
                if widgets::toggle_ui(ui, &mut visible).changed() {
                    overlay::set_overlay_visible(&ov.id, visible);
                }
                ui.label(&title);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if widgets::ghost_button(ui, t!("config_editor.overlay_reset").into_owned())
                        .on_hover_text(t!("config_editor.overlay_reset_hint"))
                        .clicked()
                    {
                        overlay::reset_panel(&ov.id);
                    }
                });
            });
        }
    });
}
