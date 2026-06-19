//! Host-side Control Center Dioxus mount + state sync.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use dioxus::prelude::{ReadableExt, WritableExt};
use dioxus_egui::set_native_draw;

use super::dioxus::context::{ControlCenterCtx, HostAction};
use super::dioxus::control_center_app;
use super::dioxus_bridge::DioxusMount;
use super::shell::ControlTab;
use super::window::{save_and_reload_config, ConfigEditor};
use super::Gui;
use crate::core::hachimi;

thread_local! {
    static HOST_GUI: Cell<*mut Gui> = const { Cell::new(std::ptr::null_mut()) };
}

/// Long-lived Dioxus mount for the Control Center shell.
pub(crate) struct ControlCenterMount {
    mount: DioxusMount,
    ctx: ControlCenterCtx,
}

impl ControlCenterMount {
    pub fn new() -> Self {
        let (mount, ctx) = DioxusMount::with_root_context_factory(control_center_app, ControlCenterCtx::new_in_runtime);
        Self { mount, ctx }
    }

    /// Live game path — `Gui` is accessed through `HOST_GUI` only while the VDOM renders.
    pub fn render_live(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        scale: f32,
        shell_h: f32,
        gui: &mut Gui,
    ) -> bool {
        HOST_GUI.set(gui as *mut Gui);
        let keep = self.render_with_host_gui(ui, ctx, scale, shell_h, false);
        HOST_GUI.set(std::ptr::null_mut());
        keep
    }

    /// Desktop preview path.
    #[cfg(feature = "dev-harness")]
    pub fn render_preview(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        scale: f32,
        shell_h: f32,
        editor: &mut ConfigEditor,
        tab: &mut ControlTab,
    ) -> bool {
        self.render_with_editor(ui, ctx, scale, shell_h, editor, tab, true)
    }

    fn render_with_host_gui(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        scale: f32,
        shell_h: f32,
        preview_stubs: bool,
    ) -> bool {
        let tab = {
            // SAFETY: `HOST_GUI` is set for the duration of `render_live`.
            let gui = unsafe { &mut *HOST_GUI.get() };
            let tab = gui.menu_tab;
            self.prepare_frame(&mut gui.config_editor);
            tab
        };

        self.sync_runtime_inputs(scale, shell_h, preview_stubs, &tab);

        let active_tab = *self.mount.in_runtime(|| self.ctx.active_tab.peek());
        register_native_tab(active_tab, ctx, &self.ctx.config, preview_stubs);

        self.mount.render(ui);

        let keep = self.ctx.keep_open.get();
        let bump_revision = {
            // SAFETY: `HOST_GUI` is set for the duration of `render_live`.
            let gui = unsafe { &mut *HOST_GUI.get() };
            gui.menu_tab = *self.mount.in_runtime(|| self.ctx.active_tab.peek());
            *gui.config_editor.config_mut() = self.ctx.config.borrow().clone();
            drain_actions(&self.ctx, gui)
        };
        if bump_revision {
            self.mount.in_runtime(|| self.ctx.bump_revision());
        }
        keep
    }

    #[cfg(feature = "dev-harness")]
    fn render_with_editor(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        scale: f32,
        shell_h: f32,
        editor: &mut ConfigEditor,
        tab: &mut ControlTab,
        preview_stubs: bool,
    ) -> bool {
        self.prepare_frame(editor);
        self.sync_runtime_inputs(scale, shell_h, preview_stubs, tab);

        let active_tab = *self.mount.in_runtime(|| self.ctx.active_tab.peek());
        register_native_tab(active_tab, ctx, &self.ctx.config, preview_stubs);

        self.mount.render(ui);

        *tab = *self.mount.in_runtime(|| self.ctx.active_tab.peek());
        *editor.config_mut() = self.ctx.config.borrow().clone();

        let keep = self.ctx.keep_open.get();
        let bump_revision = drain_preview_actions(&self.ctx, editor);
        if bump_revision {
            self.mount.in_runtime(|| self.ctx.bump_revision());
        }
        keep
    }

    fn prepare_frame(&mut self, editor: &mut ConfigEditor) {
        editor.sync();
        *self.ctx.config.borrow_mut() = editor.config().clone();
        self.ctx.detached = editor.is_detached();
        self.ctx.actions.borrow_mut().clear();
        self.ctx.keep_open.set(true);
    }

    fn sync_runtime_inputs(&mut self, scale: f32, shell_h: f32, preview_stubs: bool, tab: &ControlTab) {
        self.mount.in_runtime(|| {
            if *self.ctx.scale.peek() != scale {
                self.ctx.scale.set(scale);
            }
            if *self.ctx.height.peek() != shell_h {
                self.ctx.height.set(shell_h);
            }
            if *self.ctx.preview_stubs.peek() != preview_stubs {
                self.ctx.preview_stubs.set(preview_stubs);
            }
            if *self.ctx.active_tab.peek() != *tab {
                self.ctx.active_tab.set(*tab);
            }
        });
    }
}

fn register_native_tab(
    tab: ControlTab,
    egui_ctx: &egui::Context,
    config: &Rc<RefCell<hachimi::Config>>,
    preview_stubs: bool,
) {
    match tab {
        ControlTab::Hotkeys => {
            let cfg = Rc::clone(config);
            let ctx = egui_ctx.clone();
            set_native_draw(move |ui| {
                super::tabs::hotkeys::ui_hotkeys(ui, &ctx, &mut cfg.borrow_mut());
            });
        }
        ControlTab::Translations => {
            if preview_stubs {
                set_native_draw(|ui| {
                    super::stub::stub_tab(ui, "Translations", "Translation actions need the live game.");
                });
            } else {
                let cfg = Rc::clone(config);
                set_native_draw(move |ui| {
                    // SAFETY: HOST_GUI points at the live `Gui` while the native tab draws.
                    let gui = unsafe { &mut *HOST_GUI.get() };
                    gui.draw_translations_actions(ui, &cfg);
                });
            }
        }
        ControlTab::Plugins => {
            if preview_stubs {
                set_native_draw(|ui| {
                    super::stub::stub_tab(ui, "Plugins", "Plugin pages need the loaded plugin registry.");
                });
            } else {
                set_native_draw(|ui| {
                    // SAFETY: HOST_GUI points at the live `Gui` while the native tab draws.
                    let gui = unsafe { &mut *HOST_GUI.get() };
                    gui.draw_plugins_tab(ui);
                });
            }
        }
        ControlTab::About => {
            if preview_stubs {
                set_native_draw(|ui| {
                    super::stub::stub_tab(ui, "About", "About actions need the live game.");
                });
            } else {
                let ctx = egui_ctx.clone();
                set_native_draw(move |ui| {
                    // SAFETY: HOST_GUI points at the live `Gui` while the native tab draws.
                    let gui = unsafe { &mut *HOST_GUI.get() };
                    gui.draw_about_tab(ui, &ctx);
                });
            }
        }
        ControlTab::Graphics => {
            #[cfg(target_os = "windows")]
            {
                let cfg = Rc::clone(config);
                set_native_draw(move |ui| {
                    super::Gui::run_vsync_combo(ui, &mut cfg.borrow_mut().windows.vsync_count);
                });
            }
        }
        _ => {}
    }
}

fn drain_actions(ctx: &ControlCenterCtx, host: &mut Gui) -> bool {
    let actions = std::mem::take(&mut *ctx.actions.borrow_mut());
    let mut bump_revision = false;
    for action in actions {
        match action {
            HostAction::CloseMenu => ctx.keep_open.set(false),
            HostAction::SaveConfig if !host.config_editor.is_detached() => {
                save_and_reload_config(host.config_editor.config().clone());
            }
            HostAction::SaveConfig => {}
            HostAction::RevertConfig => {
                host.config_editor.revert_edits();
                *ctx.config.borrow_mut() = host.config_editor.config().clone();
                bump_revision = true;
            }
            HostAction::OpenThemeEditor => host.show_window(Box::new(super::window::ThemeEditorWindow::new())),
            HostAction::OpenDisableOverlayWarning => {
                host.show_window(Box::new(super::window::SimpleOkDialog::new(
                    &rust_i18n::t!("warning"),
                    &rust_i18n::t!("config_editor.disable_overlay_warning"),
                    || {},
                )));
            }
            HostAction::OpenLiveVocalsSwap => {
                host.show_window(Box::new(super::window::LiveVocalsSwapWindow::new()));
            }
            HostAction::AutoTranslateWarning => {
                host.show_window(Box::new(super::window::SimpleOkDialog::new(
                    &rust_i18n::t!("warning"),
                    &rust_i18n::t!("config_editor.auto_tl_warning"),
                    || {},
                )));
            }
        }
    }
    bump_revision
}

#[cfg(feature = "dev-harness")]
fn drain_preview_actions(ctx: &ControlCenterCtx, editor: &mut ConfigEditor) -> bool {
    let actions = std::mem::take(&mut *ctx.actions.borrow_mut());
    let mut bump_revision = false;
    for action in actions {
        match action {
            HostAction::CloseMenu => ctx.keep_open.set(false),
            HostAction::RevertConfig => {
                editor.revert_edits();
                *ctx.config.borrow_mut() = editor.config().clone();
                bump_revision = true;
            }
            HostAction::SaveConfig => {}
            HostAction::OpenThemeEditor
            | HostAction::OpenDisableOverlayWarning
            | HostAction::OpenLiveVocalsSwap
            | HostAction::AutoTranslateWarning => {}
        }
    }
    bump_revision
}
