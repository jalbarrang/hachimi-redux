//! L1 Translations tab — config settings + translation actions/stats.

use std::borrow::Cow;

use rust_i18n::t;

use crate::core::gui::components::{self as widgets, settings_grid, settings_label, toggle};
use crate::core::gui::BoxedWindow;
use crate::core::hachimi;
use crate::core::Gui;

use super::super::scale::get_scale;
use super::layout::{auto_cell, flex_wrap};

/// Config-editing settings (meta_index_url, toggles). Used by both live and
/// preview paths. Pushes auto-translate warning windows into `windows`.
pub(crate) fn draw_config_settings(ui: &mut egui::Ui, config: &mut hachimi::Config, windows: &mut Vec<BoxedWindow>) {
    settings_grid(ui, "tl_config_settings", |ui| {
        // Meta index URL
        settings_label(ui, &t!("config_editor.meta_index_url"));
        ui.text_edit_singleline(&mut config.meta_index_url);
        ui.end_row();

        // Disable translations
        settings_label(ui, &t!("config_editor.disable_translations"));
        toggle(ui, "", &mut config.disable_translations);
        ui.end_row();

        // Lazy translation updates
        settings_label(ui, &t!("config_editor.lazy_translation_updates"));
        toggle(ui, "", &mut config.lazy_translation_updates);
        ui.end_row();

        // Translator mode
        settings_label(ui, &t!("config_editor.translator_mode"));
        toggle(ui, "", &mut config.translator_mode);
        ui.end_row();

        // Disable skill name translation
        settings_label(ui, &t!("config_editor.disable_skill_name_translation"));
        toggle(ui, "", &mut config.disable_skill_name_translation);
        ui.end_row();

        // Auto-translate stories (with warning)
        settings_label(ui, &t!("config_editor.auto_translate_stories"));
        {
            let prev = config.auto_translate_stories;
            toggle(ui, "", &mut config.auto_translate_stories);
            if config.auto_translate_stories && !prev {
                windows.push(Box::new(super::super::window::SimpleOkDialog::new(
                    &t!("warning"),
                    &t!("config_editor.auto_tl_warning"),
                    || {},
                )));
            }
        }
        ui.end_row();

        // Auto-translate UI (with warning)
        settings_label(ui, &t!("config_editor.auto_translate_ui"));
        {
            let prev = config.auto_translate_localize;
            toggle(ui, "", &mut config.auto_translate_localize);
            if config.auto_translate_localize && !prev {
                windows.push(Box::new(super::super::window::SimpleOkDialog::new(
                    &t!("warning"),
                    &t!("config_editor.auto_tl_warning"),
                    || {},
                )));
            }
        }
        ui.end_row();
    });
}

/// Full translations tab (live path): config settings + action buttons + stats.
pub(crate) fn draw_full(ui: &mut egui::Ui, ctx: &egui::Context, gui: &mut Gui) {
    // Config settings
    {
        let config = gui.config_editor.config_mut();
        let mut windows: Vec<BoxedWindow> = Vec::new();
        draw_config_settings(ui, config, &mut windows);
        // Apply deferred windows after releasing config borrow.
        for w in windows {
            gui.show_window(w);
        }
    }

    ui.add_space(8.0);

    // Action buttons + stats (needs Hachimi::instance, live-only).
    let mut note: Option<Cow<'_, str>> = None;
    gui.run_translations_actions(ui, ctx, &mut note);
    if let Some(n) = note {
        gui.show_notification(n.as_ref());
    }
}

impl Gui {
    /// Translation action buttons + dictionary stats. Separated from
    /// `draw_config_settings` because it needs the live `Hachimi` instance.
    fn run_translations_actions(
        &mut self,
        ui: &mut egui::Ui,
        _ctx: &egui::Context,
        show_notification: &mut Option<Cow<'_, str>>,
    ) {
        use crate::core::Hachimi;
        use crate::il2cpp::{hook::umamusume::Localize, symbols::Thread};

        let hachimi = Hachimi::instance();
        let localized_data = hachimi.localized_data.load();
        let localize_dict_count = localized_data.localize_dict.len().to_string();
        let hashed_dict_count = localized_data.hashed_dict.len().to_string();
        let scale = get_scale(ui.ctx());

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
                            let mut gui_lock = Gui::instance()
                                .expect("unexpected failure")
                                .lock()
                                .expect("lock poisoned");
                            if let Err(e) = crate::core::utils::write_json_file(&data, dict_path) {
                                gui_lock.show_notification(&e.to_string())
                            } else {
                                gui_lock.show_notification(&t!("notification.saved_localize_dump"))
                            }
                        })
                    }
                });
            }
        });

        widgets::section_header(ui, t!("menu.stats_heading"));
        ui.label(t!("menu.localize_dict_entries", count = localize_dict_count));
        ui.label(t!("menu.hashed_dict_entries", count = hashed_dict_count));
    }
}
