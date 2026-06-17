//! L1 About tab — config actions, about info, update check, stats, danger zone.

use std::borrow::Cow;

use chrono::{Datelike, Utc};
use rust_i18n::t;

use crate::core::gui::widgets;
use crate::core::gui::window::{BoxedWindow, FirstTimeSetupWindow, LicenseWindow, SimpleYesNoDialog};
use crate::core::gui::Gui;
use crate::core::hachimi::{REPO_PATH, WEBSITE_URL};
use crate::core::Hachimi;
use crate::il2cpp::{
    ext::StringExt,
    hook::{umamusume::GameSystem, UnityEngine_CoreModule::Application},
    symbols::Thread,
};

use super::super::scale::get_scale;
use super::layout::{auto_cell, flex_row, flex_wrap};

impl Gui {
    pub(crate) fn run_about_tab(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        show_notification: &mut Option<Cow<'_, str>>,
        show_window: &mut Option<BoxedWindow>,
    ) {
        let hachimi = Hachimi::instance();
        let scale = get_scale(ctx);

        widgets::section_header(ui, t!("about.config_actions_heading").into_owned());
        flex_wrap(ui, ui.id().with("about_config_actions"), scale, 8.0, |tui| {
            auto_cell(tui, |ui| {
                if widgets::primary_button(ui, t!("menu.reload_config").into_owned()).clicked() {
                    hachimi.reload_config();
                    *show_notification = Some(t!("notification.config_reloaded"));
                }
            });
            auto_cell(tui, |ui| {
                if widgets::secondary_button(ui, t!("menu.open_first_time_setup").into_owned()).clicked() {
                    *show_window = Some(Box::new(FirstTimeSetupWindow::new()));
                }
            });
            #[cfg(target_os = "windows")]
            auto_cell(tui, |ui| {
                if widgets::secondary_button(ui, t!("menu.save_diagnostics").into_owned()).clicked() {
                    match crate::windows::diagnostics::write_report() {
                        Ok(path) => {
                            *show_notification =
                                Some(t!("notification.diagnostics_saved", path = path.display().to_string()));
                        }
                        Err(e) => {
                            *show_notification = Some(t!("notification.diagnostics_failed", error = e.to_string()));
                        }
                    }
                }
            });
        });
        ui.add_space(8.0);

        ui.add_space(4.0);
        flex_row(ui, ui.id().with("about_brand"), scale, 8.0, |tui| {
            auto_cell(tui, |ui| {
                ui.add(Self::icon_2x(ctx));
            });
            auto_cell(tui, |ui| {
                ui.vertical(|ui| {
                    ui.heading(t!("hachimi"));
                    ui.label(env!("HACHIMI_DISPLAY_VERSION"));
                });
            });
        });
        ui.label(t!("about.copyright", year = Utc::now().year()));
        ui.add_space(4.0);

        flex_wrap(ui, ui.id().with("about_links"), scale, 8.0, |tui| {
            auto_cell(tui, |ui| {
                if widgets::secondary_button(ui, t!("about.view_license").into_owned()).clicked() {
                    *show_window = Some(Box::new(LicenseWindow::new()));
                }
            });
            auto_cell(tui, |ui| {
                if widgets::secondary_button(ui, t!("about.open_website").into_owned()).clicked() {
                    Application::OpenURL(WEBSITE_URL.to_il2cpp_string());
                }
            });
            auto_cell(tui, |ui| {
                if widgets::secondary_button(ui, t!("about.view_source_code").into_owned()).clicked() {
                    Application::OpenURL(format!("https://github.com/{}", REPO_PATH).to_il2cpp_string());
                }
            });
        });

        flex_wrap(ui, ui.id().with("about_updates"), scale, 8.0, |tui| {
            auto_cell(tui, |ui| {
                if widgets::primary_button(ui, t!("menu.check_for_updates").into_owned()).clicked() {
                    Hachimi::instance().updater.clone().check_for_updates(|_| {});
                }
            });
            auto_cell(tui, |ui| {
                if widgets::secondary_button(ui, t!("menu.sync_gametora_data").into_owned()).clicked() {
                    let hachimi = Hachimi::instance();
                    hachimi.gametora_updater.clone().sync(true);
                    hachimi.tracker_updater.clone().sync(true);
                }
            });
        });

        widgets::section_header(ui, t!("menu.stats_heading"));
        ui.label(&self.fps_text);

        widgets::section_header(ui, t!("menu.danger_zone_heading"));
        ui.label(t!("menu.danger_zone_warning"));
        flex_wrap(ui, ui.id().with("about_danger"), scale, 8.0, |tui| {
            auto_cell(tui, |ui| {
                if widgets::danger_button(ui, t!("menu.soft_restart").into_owned()).clicked() {
                    *show_window = Some(Box::new(SimpleYesNoDialog::new(
                        &t!("confirm_dialog_title"),
                        &t!("soft_restart_confirm_content"),
                        |ok| {
                            if !ok {
                                return;
                            }
                            Thread::main_thread().schedule(|| {
                                GameSystem::SoftwareReset(GameSystem::instance());
                            });
                        },
                    )));
                }
            });
            auto_cell(tui, |ui| {
                if widgets::secondary_button(ui, t!("menu.toggle_game_ui").into_owned()).clicked() {
                    Thread::main_thread().schedule(Self::toggle_game_ui);
                }
            });
        });
    }
}
