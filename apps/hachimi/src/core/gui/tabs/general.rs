//! General tab — core config + overlay controls (egui-native).

use rust_i18n::t;

use crate::core::gui::components::{
    ghost_button, secondary_button, settings_grid, settings_label, settings_section, slider_f32, toggle,
};
use crate::core::gui::BoxedWindow;
use crate::core::hachimi::{self, Language};
use crate::core::plugin::overlay;

/// Draw the General tab body. Edits `config` in-place; pushes deferred windows
/// into `windows`.
pub(crate) fn draw(ui: &mut egui::Ui, config: &mut hachimi::Config, windows: &mut Vec<BoxedWindow>) {
    settings_grid(ui, "general_settings", |ui| {
        // Language
        settings_label(ui, &t!("config_editor.language"));
        {
            let choices: Vec<(Language, &str)> = Language::CHOICES.iter().map(|(lang, name)| (*lang, *name)).collect();
            let selected = choices
                .iter()
                .find(|(l, _)| *l == config.language)
                .map_or("Unknown", |(_, n)| n);
            let mut changed = false;
            egui::ComboBox::new(ui.id().with("lang_combo"), "")
                .selected_text(selected)
                .show_ui(ui, |ui| {
                    for (lang, name) in &choices {
                        if ui.selectable_value(&mut config.language, *lang, *name).changed() {
                            changed = true;
                        }
                    }
                });
            if changed {
                config.language.set_locale();
            }
        }
        ui.end_row();

        // Disable overlay
        settings_label(ui, &t!("config_editor.disable_overlay"));
        {
            let prev = config.disable_gui;
            toggle(ui, "", &mut config.disable_gui);
            if config.disable_gui && !prev {
                windows.push(Box::new(super::super::window::SimpleOkDialog::new(
                    &t!("warning"),
                    &t!("config_editor.disable_overlay_warning"),
                    || {},
                )));
            }
        }
        ui.end_row();

        // IPv4 only
        settings_label(ui, &t!("config_editor.ipv4_only"));
        toggle(ui, "", &mut config.ipv4_only);
        ui.end_row();

        // GUI scale
        settings_label(ui, &t!("config_editor.gui_scale"));
        slider_f32(ui, &mut config.gui_scale, 0.25..=2.0, 0.05);
        ui.end_row();

        // Windows-specific fields
        #[cfg(target_os = "windows")]
        {
            settings_label(ui, &t!("config_editor.gui_landscape_ratio"));
            slider_f32(ui, &mut config.windows.gui_landscape_ratio, 0.25..=1.0, 0.05);
            ui.end_row();

            settings_label(ui, &t!("config_editor.discord_rpc"));
            toggle(ui, "", &mut config.windows.discord_rpc);
            ui.end_row();
        }

        // Theme editor
        settings_label(ui, &t!("theme_editor.title"));
        if secondary_button(ui, t!("open").to_string()).clicked() {
            windows.push(Box::new(super::super::window::ThemeEditorWindow::new()));
        }
        ui.end_row();

        // Debug mode
        settings_label(ui, &t!("config_editor.debug_mode"));
        toggle(ui, "", &mut config.debug_mode);
        ui.end_row();

        // File logging
        settings_label(ui, &t!("config_editor.enable_file_logging"));
        toggle(ui, "", &mut config.enable_file_logging);
        ui.end_row();

        // Atlas workaround
        settings_label(ui, &t!("config_editor.apply_atlas_workaround"));
        toggle(ui, "", &mut config.apply_atlas_workaround);
        ui.end_row();

        // Skip first-time setup
        settings_label(ui, &t!("config_editor.skip_first_time_setup"));
        toggle(ui, "", &mut config.skip_first_time_setup);
        ui.end_row();

        // Disable auto-update check
        settings_label(ui, &t!("config_editor.disable_auto_update_check"));
        toggle(ui, "", &mut config.disable_auto_update_check);
        ui.end_row();

        // Enable IPC
        settings_label(ui, &t!("config_editor.enable_ipc"));
        toggle(ui, "", &mut config.enable_ipc);
        ui.end_row();

        // IPC listen all
        settings_label(ui, &t!("config_editor.ipc_listen_all"));
        toggle(ui, "", &mut config.ipc_listen_all);
        ui.end_row();
    });

    // ── Overlays panel ──────────────────────────────────────────────
    settings_section(ui, &t!("config_editor.overlays_heading"));
    draw_overlays_panel(ui);
}

fn draw_overlays_panel(ui: &mut egui::Ui) {
    let mut opacity = overlay::opacity();
    if slider_f32(ui, &mut opacity, 0.1..=1.0, 0.05) {
        overlay::set_opacity(opacity);
    }

    let overlays = overlay::get_plugin_overlays();
    if overlays.is_empty() {
        let tokens = crate::core::gui::theme::ThemeTokens::from_ui(ui);
        ui.label(egui::RichText::new(t!("config_editor.overlays_none")).color(tokens.text_faint));
    }
    for ov in overlays {
        ui.horizontal(|ui| {
            let title = overlay::display_title(&ov.id);
            let mut visible = overlay::is_overlay_visible(&ov.id);
            if toggle(ui, &title, &mut visible) {
                overlay::set_overlay_visible(&ov.id, visible);
            }
            if ghost_button(ui, t!("config_editor.overlay_reset").to_string()).clicked() {
                overlay::reset_panel(&ov.id);
            }
        });
    }
}
