use std::sync::Arc;

use rust_i18n::t;

use crate::core::hachimi;
use crate::core::Hachimi;

use egui_taffy::taffy::prelude::length;
use egui_taffy::{taffy, tui};

use super::super::components as widgets;
use super::super::scale::get_scale;
use super::super::tabs::{self, layout};
use super::{random_id, save_and_reload_config};

pub(crate) struct ConfigEditor {
    last_ptr_config: usize,
    config: hachimi::Config,
    id: egui::Id,
    current_tab: ConfigEditorTab,
    /// Desktop-preview mode: no `Hachimi::instance()` backing store. `sync`/
    /// `revert`/Save become inert and game-data combos use static placeholders,
    /// so the editor renders against a plain in-memory config off-game.
    detached: bool,
}

/// Which config body to render. Driven by the L1 Control Center tab (the former
/// Config sub-tabs are now top-level tabs).
#[derive(Eq, PartialEq, Clone, Copy)]
pub(crate) enum ConfigEditorTab {
    General,
    Graphics,
    Gameplay,
    Hotkeys,
}

impl ConfigEditor {
    pub fn new() -> ConfigEditor {
        let handle = Hachimi::instance().config.load();
        ConfigEditor {
            last_ptr_config: Arc::as_ptr(&handle) as usize,
            config: (**Hachimi::instance().config.load()).clone(),
            id: random_id(),
            current_tab: ConfigEditorTab::General,
            detached: false,
        }
    }

    /// Read the working-copy config (preview harness uses this to mirror the GUI
    /// scale into the egui context).
    #[cfg(feature = "dev-harness")]
    pub(crate) fn working_config(&self) -> &hachimi::Config {
        &self.config
    }

    /// Build a detached editor for the desktop preview harness: backed by the
    /// given in-memory config, with no `Hachimi::instance()` coupling.
    #[cfg(feature = "dev-harness")]
    pub(crate) fn new_detached(config: hachimi::Config) -> ConfigEditor {
        ConfigEditor {
            last_ptr_config: 0,
            config,
            id: random_id(),
            current_tab: ConfigEditorTab::General,
            detached: true,
        }
    }

    /// Discard unsaved edits: reset the working copy to the currently saved config
    /// and re-apply its language locale (the language combo applies locale live).
    fn revert(&mut self) {
        if self.detached {
            self.config = hachimi::Config::default();
            self.config.language.set_locale();
            return;
        }
        let handle = Hachimi::instance().config.load();
        self.last_ptr_config = Arc::as_ptr(&handle) as usize;
        self.config = (**handle).clone();
        self.config.language.set_locale();
    }

    /// Sync the working copy if the saved config changed underneath us.
    fn sync(&mut self) {
        if self.detached {
            return;
        }
        let global_handle = Hachimi::instance().config.load();
        let global_ptr = Arc::as_ptr(&global_handle) as usize;
        if global_ptr != self.last_ptr_config {
            self.config = (**global_handle).clone();
            self.last_ptr_config = global_ptr;
        }
        #[cfg(target_os = "windows")]
        {
            self.config.windows.menu_open_key = global_handle.windows.menu_open_key;
        }
    }

    /// Body for one config tab (General / Graphics / Gameplay / Hotkeys), driven
    /// by the L1 Control Center tab. No inner sub-tab strip — the former sub-tabs
    /// are now top-level tabs.
    pub(crate) fn ui_body(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, tab: ConfigEditorTab) {
        self.current_tab = tab;
        self.sync();

        if tab == ConfigEditorTab::Hotkeys {
            tabs::hotkeys::ui_hotkeys(ui, ctx, &mut self.config);
            return;
        }

        let scale = get_scale(ctx);
        let id = self.id;
        let grid_id = match tab {
            ConfigEditorTab::General => "grid_general",
            ConfigEditorTab::Graphics => "grid_graphics",
            ConfigEditorTab::Gameplay => "grid_gameplay",
            ConfigEditorTab::Hotkeys => "grid_hotkeys",
        };

        let harness = self.detached;
        layout::settings_grid(ui, scale, id.with(grid_id), |tui| match tab {
            ConfigEditorTab::General => tabs::general::options(&mut self.config, tui),
            ConfigEditorTab::Graphics => tabs::graphics::options(&mut self.config, tui),
            ConfigEditorTab::Gameplay => tabs::gameplay::options(&mut self.config, tui, harness),
            ConfigEditorTab::Hotkeys => {}
        });

        if tab == ConfigEditorTab::General {
            tabs::general::overlays(ui, ctx);
        }
    }

    /// Translations tab body: the translation-related options grid.
    pub(crate) fn ui_translations(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let scale = get_scale(ctx);
        self.sync();

        let id = self.id;
        layout::settings_grid(ui, scale, id.with("grid_translations"), |tui| {
            tabs::translations::options(&mut self.config, tui);
        });
    }

    /// Always-present footer: right-aligned `Cancel · Save`. `enabled` is false on
    /// tabs that don't edit the config working-copy (Plugins / About) — the
    /// buttons render greyed there. Cancel discards unsaved edits (keeps the menu
    /// open); Save persists the working copy and reloads.
    pub(crate) fn ui_footer(&mut self, ui: &mut egui::Ui, enabled: bool) {
        ui.separator();

        let mut cancel_clicked = false;
        let id = self.id;
        let detached = self.detached;
        let config = &self.config;
        let footer_w = (super::super::shell::SHELL_WIDTH * get_scale(ui.ctx()) - 16.0).max(120.0);
        ui.add_enabled_ui(enabled, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            tui(ui, id.with("footer"))
                .reserve_width(footer_w)
                .style(taffy::Style {
                    display: taffy::Display::Flex,
                    flex_direction: taffy::FlexDirection::Row,
                    align_items: Some(taffy::AlignItems::Center),
                    justify_content: Some(taffy::JustifyContent::End),
                    gap: taffy::Size {
                        width: length(8.0),
                        height: length(0.0),
                    },
                    ..Default::default()
                })
                .show(|tui| {
                    layout::auto_cell(tui, |ui| {
                        if widgets::secondary_button(ui, t!("config_editor.cancel").into_owned()).clicked() {
                            cancel_clicked = true;
                        }
                    });
                    layout::auto_cell(tui, |ui| {
                        if widgets::primary_button(ui, t!("save").into_owned()).clicked() && !detached {
                            save_and_reload_config(config.clone());
                        }
                    });
                });
        });

        if cancel_clicked {
            self.revert();
        }
    }
}
