//! Control Center modal shell: Dioxus shell + tab dispatch.

use super::control_center_tls;
#[cfg(feature = "dev-harness")]
use super::window::ConfigEditor;
use super::Gui;

/// Base (unscaled) width of the Control Center modal shell.
pub(crate) const SHELL_WIDTH: f32 = 800.0;

/// Fixed top-level tabs of the Control Center.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub(crate) enum ControlTab {
    #[default]
    General,
    Graphics,
    Gameplay,
    Hotkeys,
    Translations,
    Plugins,
    About,
}

impl ControlTab {
    pub(crate) fn edits_config(self) -> bool {
        matches!(
            self,
            ControlTab::General
                | ControlTab::Graphics
                | ControlTab::Gameplay
                | ControlTab::Hotkeys
                | ControlTab::Translations
        )
    }
}

/// Render the Dioxus Control Center for the live `Gui`.
pub(crate) fn render_control_center_gui(gui: &mut Gui, ui: &mut egui::Ui, ctx: &egui::Context, scale: f32) -> bool {
    render_inner_live(ui, ctx, scale, gui)
}

/// Render the Dioxus Control Center for the desktop preview harness.
#[cfg(feature = "dev-harness")]
pub(crate) fn render_control_center_preview(
    menu_tab: &mut ControlTab,
    editor: &mut ConfigEditor,
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    scale: f32,
) -> bool {
    render_inner_preview(ui, ctx, scale, editor, menu_tab)
}

fn render_inner_live(ui: &mut egui::Ui, ctx: &egui::Context, scale: f32, gui: &mut Gui) -> bool {
    let shell_w = SHELL_WIDTH * scale;
    let shell_h = ctx.input(|i| i.viewport_rect().height()) * 0.85;
    ui.set_width(shell_w);
    ui.set_max_height(shell_h);
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

    // No wrapping fill frame: the Dioxus shell paints its own rounded panel, so a
    // rectangular fill behind it only pokes square corners past the rounding.
    control_center_tls::with_mount(|mount| mount.render_live(ui, ctx, scale, shell_h, gui))
}

#[cfg(feature = "dev-harness")]
fn render_inner_preview(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    scale: f32,
    editor: &mut ConfigEditor,
    tab: &mut ControlTab,
) -> bool {
    let shell_w = SHELL_WIDTH * scale;
    let shell_h = ctx.input(|i| i.viewport_rect().height()) * 0.85;
    ui.set_width(shell_w);
    ui.set_max_height(shell_h);
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

    control_center_tls::with_mount(|mount| mount.render_preview(ui, ctx, scale, shell_h, editor, tab))
}
