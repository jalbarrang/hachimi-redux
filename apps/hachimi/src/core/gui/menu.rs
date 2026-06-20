//! L1 "Control Center": hotkey-toggled modal glue for the live `Gui` instance.
//!
//! The shell layout lives in [`super::shell`]; tab bodies live in `gui/tabs/`
//! and `window/config_editor.rs`. This module wires the shell to `Gui` state.

use super::scale::get_scale;
use super::Gui;

impl Gui {
    pub(crate) fn draw_translations_actions(
        &mut self,
        ui: &mut egui::Ui,
        config: &std::rc::Rc<std::cell::RefCell<crate::core::hachimi::Config>>,
    ) {
        let mut note: Option<std::borrow::Cow<'_, str>> = None;
        *self.config_editor.config_mut() = config.borrow().clone();
        let ctx = self.context.clone();
        self.run_translations_tab(ui, &ctx, &mut note);
        *config.borrow_mut() = self.config_editor.config().clone();
        if let Some(n) = note {
            self.show_notification(n.as_ref());
        }
    }

    pub(crate) fn draw_plugins_tab(&mut self, ui: &mut egui::Ui) {
        let mut note = None;
        let ctx = self.context.clone();
        self.run_plugins_tab(ui, &ctx, &mut note);
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

    /// Draw the modal shell + tab bar and dispatch to the active tab. The shell
    /// layout lives in the shared `render_control_center` so the desktop preview
    /// harness renders the exact same chrome.
    fn run_control_center(&mut self) {
        let ctx = self.context.clone();
        let scale = get_scale(&ctx);

        let mut keep_open = true;
        // No popup frame: the Dioxus shell paints its own rounded panel (bg +
        // border + radius). The default `Frame::popup` drew a second, offset
        // window frame + shadow behind it. Keep only the dimmed backdrop.
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
