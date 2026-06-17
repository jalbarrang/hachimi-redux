//! L1 "Control Center": hotkey-toggled modal glue for the live `Gui` instance.
//!
//! The shell layout lives in [`super::shell`]; tab bodies live in `gui/tabs/`
//! and `window/config_editor.rs`. This module wires the shell to `Gui` state.

use std::borrow::Cow;

use super::scale::get_scale;
use super::shell::{render_control_center, ControlCenterHost, ControlTab};
use super::window::{BoxedWindow, ConfigEditor, ConfigEditorTab};
use super::Gui;

impl ControlCenterHost for Gui {
    fn active_tab(&self) -> ControlTab {
        self.menu_tab
    }

    fn set_active_tab(&mut self, tab: ControlTab) {
        self.menu_tab = tab;
    }

    fn config_editor(&mut self) -> &mut ConfigEditor {
        &mut self.config_editor
    }

    fn draw_icon(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.add(Self::icon(ctx));
    }

    fn draw_body(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, tab: ControlTab) {
        let mut show_notification: Option<Cow<'_, str>> = None;
        let mut show_window: Option<BoxedWindow> = None;

        match tab {
            ControlTab::General => self.config_editor.ui_body(ui, ctx, ConfigEditorTab::General),
            ControlTab::Graphics => self.config_editor.ui_body(ui, ctx, ConfigEditorTab::Graphics),
            ControlTab::Gameplay => self.config_editor.ui_body(ui, ctx, ConfigEditorTab::Gameplay),
            ControlTab::Hotkeys => self.config_editor.ui_body(ui, ctx, ConfigEditorTab::Hotkeys),
            ControlTab::Translations => self.run_translations_tab(ui, ctx, &mut show_notification),
            ControlTab::Plugins => self.run_plugins_tab(ui, ctx, &mut show_notification),
            ControlTab::About => self.run_about_tab(ui, ctx, &mut show_notification, &mut show_window),
        }
        if let Some(content) = show_notification {
            self.show_notification(content.as_ref());
        }
        if let Some(window) = show_window {
            self.show_window(window);
        }
    }
}

impl Gui {
    pub(crate) fn run_menu(&mut self) {
        if self.show_menu {
            self.run_control_center();
        }

        // The modal has no slide-out animation, so release input as soon as it closes.
        if !self.show_menu {
            self.menu_visible = false;
        }
    }

    /// Draw the modal shell + tab bar and dispatch to the active tab. The shell
    /// layout lives in the shared `render_control_center` so the desktop preview
    /// harness renders the exact same chrome.
    fn run_control_center(&mut self) {
        let ctx = self.context.clone();
        let scale = get_scale(&ctx);

        let mut keep_open = true;
        let response = egui::Modal::new(egui::Id::new("hachimi_control_center")).show(&ctx, |ui| {
            keep_open = render_control_center(ui, &ctx, scale, self);
        });

        // Close on backdrop click / Escape, or via the header button.
        if response.should_close() || !keep_open {
            self.show_menu = false;
            self.menu_anim_time = None;
        }
    }
}
