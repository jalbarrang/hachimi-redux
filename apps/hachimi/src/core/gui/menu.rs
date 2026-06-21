//! L1 "Control Center": hotkey-toggled modal glue for the live `Gui` instance.
//!
//! The shell layout lives in [`super::shell`]; tab bodies live in `gui/tabs/`.
//! This module wires the shell to `Gui` state.

use super::scale::get_scale;
use super::Gui;

impl Gui {
    pub(crate) fn draw_plugins_tab(&mut self, ui: &mut egui::Ui) {
        let mut note = None;
        let ctx = self.context.clone();
        self.run_plugins_tab(ui, &ctx, &mut note);
        if let Some(n) = note {
            self.show_notification(n.as_ref());
        }
    }

    #[cfg(feature = "training-tracker")]
    pub(crate) fn draw_training_tracker_tab(&mut self, ui: &mut egui::Ui) {
        crate::core::plugin::tab::draw(ui);
    }

    pub(crate) fn draw_about_tab(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let mut note = None;
        let mut window = None;
        self.run_about_tab(ui, ctx, &mut note, &mut window);
        if let Some(n) = note {
            self.show_notification(n.as_ref());
        }
        if let Some(w) = window {
            self.show_window(w);
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

    /// Draw the modal shell + tab bar and dispatch to the active tab.
    fn run_control_center(&mut self) {
        let ctx = self.context.clone();
        let scale = get_scale(&ctx);

        let mut keep_open = true;
        // The egui-native shell paints its own rounded panel — use Frame::NONE
        // so the modal only provides the dimmed backdrop.
        let response = egui::Modal::new(egui::Id::new("hachimi_control_center"))
            .frame(egui::Frame::NONE)
            .show(&ctx, |ui| {
                keep_open = super::shell::render_control_center_gui(self, ui, &ctx, scale);
            });

        // Close on backdrop click / Escape, or via the header button.
        if response.should_close() || !keep_open {
            self.show_menu = false;
            self.menu_anim_time = None;
        }
    }
}
