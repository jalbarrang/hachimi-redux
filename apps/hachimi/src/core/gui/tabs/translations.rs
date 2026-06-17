//! L1 Translations tab — translation actions, dictionary stats, and the
//! translation-related config settings (sharing the config editor's working copy).

use std::borrow::Cow;
use std::thread;

use rust_i18n::t;

use crate::core::gui::widgets;
use crate::core::gui::Gui;
use crate::core::hachimi;
use crate::core::Hachimi;
use crate::il2cpp::{hook::umamusume::Localize, symbols::Thread};

use egui_taffy::Tui;

use super::super::scale::get_scale;
use super::super::window::SimpleOkDialog;
use super::layout::{auto_cell, fill_cell, flex_wrap, label_cell};

/// Translation-related options grid cells.
pub(crate) fn options(config: &mut hachimi::Config, tui: &mut Tui) {
    label_cell(tui, t!("config_editor.meta_index_url"));
    fill_cell(tui, |ui| {
        let res = ui.add(egui::TextEdit::singleline(&mut config.meta_index_url).lock_focus(true));
        #[cfg(target_os = "windows")]
        if res.has_focus() {
            ui.memory_mut(|mem| {
                mem.set_focus_lock_filter(
                    res.id,
                    egui::EventFilter {
                        tab: true,
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        escape: true,
                    },
                )
            });
        }
        #[cfg(not(target_os = "windows"))]
        let _ = res;
    });

    label_cell(tui, t!("config_editor.disable_translations"));
    auto_cell(tui, |ui| {
        ui.checkbox(&mut config.disable_translations, "");
    });

    label_cell(tui, t!("config_editor.lazy_translation_updates"));
    auto_cell(tui, |ui| {
        ui.checkbox(&mut config.lazy_translation_updates, "");
    });

    label_cell(tui, t!("config_editor.translator_mode"));
    auto_cell(tui, |ui| {
        ui.checkbox(&mut config.translator_mode, "");
    });

    label_cell(tui, t!("config_editor.disable_skill_name_translation"));
    auto_cell(tui, |ui| {
        ui.checkbox(&mut config.disable_skill_name_translation, "");
    });

    label_cell(tui, t!("config_editor.auto_translate_stories"));
    auto_cell(tui, |ui| {
        if ui.checkbox(&mut config.auto_translate_stories, "").clicked() && config.auto_translate_stories {
            thread::spawn(|| {
                Gui::instance()
                    .expect("unexpected failure")
                    .lock()
                    .expect("unexpected failure")
                    .show_window(Box::new(SimpleOkDialog::new(
                        &t!("warning"),
                        &t!("config_editor.auto_tl_warning"),
                        || {},
                    )));
            });
        }
    });

    label_cell(tui, t!("config_editor.auto_translate_ui"));
    auto_cell(tui, |ui| {
        if ui.checkbox(&mut config.auto_translate_localize, "").clicked() && config.auto_translate_localize {
            thread::spawn(|| {
                Gui::instance()
                    .expect("unexpected failure")
                    .lock()
                    .expect("unexpected failure")
                    .show_window(Box::new(SimpleOkDialog::new(
                        &t!("warning"),
                        &t!("config_editor.auto_tl_warning"),
                        || {},
                    )));
            });
        }
    });
}

impl Gui {
    pub(crate) fn run_translations_tab(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        show_notification: &mut Option<Cow<'_, str>>,
    ) {
        let hachimi = Hachimi::instance();
        let localized_data = hachimi.localized_data.load();
        let localize_dict_count = localized_data.localize_dict.len().to_string();
        let hashed_dict_count = localized_data.hashed_dict.len().to_string();
        let scale = get_scale(ctx);

        flex_wrap(ui, ui.id().with("tl_actions"), scale, 8.0, |tui| {
            auto_cell(tui, |ui| {
                if widgets::primary_button(ui, t!("menu.reload_localized_data").into_owned()).clicked() {
                    hachimi.load_localized_data();
                    *show_notification = Some(t!("notification.localized_data_reloaded"));
                }
            });
            auto_cell(tui, |ui| {
                if widgets::secondary_button(ui, t!("menu.tl_check_for_updates").into_owned()).clicked() {
                    hachimi.tl_updater.clone().check_for_updates(false);
                }
            });
            auto_cell(tui, |ui| {
                if widgets::secondary_button(ui, t!("menu.tl_check_for_updates_pedantic").into_owned()).clicked() {
                    hachimi.tl_updater.clone().check_for_updates(true);
                }
            });
            if hachimi.config.load().translator_mode {
                auto_cell(tui, |ui| {
                    if widgets::secondary_button(ui, t!("menu.dump_localize_dict").into_owned()).clicked() {
                        Thread::main_thread().schedule(|| {
                            let data = Localize::dump_strings();
                            let dict_path = Hachimi::instance().get_data_path("localize_dump.json");
                            let mut gui = Gui::instance()
                                .expect("unexpected failure")
                                .lock()
                                .expect("lock poisoned");
                            if let Err(e) = crate::core::utils::write_json_file(&data, dict_path) {
                                gui.show_notification(&e.to_string())
                            } else {
                                gui.show_notification(&t!("notification.saved_localize_dump"))
                            }
                        })
                    }
                });
            }
        });

        widgets::section_header(ui, t!("menu.stats_heading"));
        ui.label(t!("menu.localize_dict_entries", count = localize_dict_count));
        ui.label(t!("menu.hashed_dict_entries", count = hashed_dict_count));

        self.config_editor.ui_translations(ui, ctx);
    }
}
