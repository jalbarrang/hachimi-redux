//! Desktop preview harness for the Control Center menu.

use crate::core::hachimi::Config;

use super::scale::get_scale;
use super::shell::{render_control_center_preview, ControlTab, SHELL_WIDTH};
use super::window::ConfigEditor;
use super::Gui;

/// Entry point invoked by the `menu_preview` example (the non-hot path).
pub fn run() -> eframe::Result {
    eframe::run_native(
        "hachimi-menu-preview",
        native_options(),
        Box::new(|cc| {
            init_context(&cc.egui_ctx);
            Ok(Box::new(PreviewApp::default()))
        }),
    )
}

/// Native window options shared by the plain and hot-reloading preview runners.
pub fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("HachimiRedux — Control Center preview")
            .with_inner_size([1000.0, 1000.0])
            .with_active(true),
        ..Default::default()
    }
}

/// One-time egui context setup: multi-pass taffy settling, the real fonts/theme,
/// and image loaders (for the embedded title icon). Call once on app creation.
pub fn init_context(ctx: &egui::Context) {
    // egui_taffy settles its layout via in-frame multi-pass; without this every
    // taffy surface flickers (matches the in-game Context config).
    ctx.options_mut(|o| o.max_passes = std::num::NonZeroUsize::new(3).expect("nonzero"));

    let config = Config::default();
    ctx.set_fonts(Gui::get_font_definitions());
    let mut style = egui::Style::default();
    Gui::apply_theme(ctx, &mut style, &config);
    ctx.set_global_style(style);
    egui_extras::install_image_loaders(ctx);
    super::splash::register_icon_bytes(ctx);
}

/// Persistent preview state. Lives in the host binary (the example) so it survives
/// hot-reloads of the draw code, mirroring the `hot-egui` pattern.
pub struct PreviewState {
    menu_tab: ControlTab,
    config_editor: ConfigEditor,
    brought_to_front: bool,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self {
            menu_tab: ControlTab::default(),
            config_editor: ConfigEditor::new_detached(Config::default()),
            brought_to_front: false,
        }
    }
}

/// Draw one frame of the preview into the central-panel `Ui`. Returns whether
/// the window should stay open. This is the hot-reloadable unit: the `menu-hot`
/// dylib forwards to it, and the example calls it so edits swap live.
pub fn draw_frame(state: &mut PreviewState, ui: &mut egui::Ui) -> bool {
    let ctx = ui.ctx().clone();

    // Mirror the working copy's GUI scale into the context, like the in-game
    // render loop does, so the preview reflects the General → GUI scale slider.
    let scale = state.config_editor.working_config().gui_scale;
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("gui_scale"), scale));

    // Pull the preview window to the foreground on first paint.
    if !state.brought_to_front {
        state.brought_to_front = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    // Green "desktop" backdrop behind the centered modal shell.
    ui.painter()
        .rect_filled(ui.max_rect(), 0.0, egui::Color32::from_rgb(0, 180, 0));

    // Center the modal shell within the desktop.
    let gui_scale = get_scale(&ctx);
    let shell_w = SHELL_WIDTH * gui_scale;
    let shell_h = ctx.input(|i| i.viewport_rect().height()) * 0.85;
    let full = ui.available_rect_before_wrap();
    let left = full.left() + ((full.width() - shell_w) * 0.5).max(0.0);
    let top = full.top() + ((full.height() - shell_h) * 0.5).max(0.0);
    let rect = egui::Rect::from_min_size(egui::pos2(left, top), egui::vec2(shell_w, shell_h));

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        render_control_center_preview(&mut state.menu_tab, &mut state.config_editor, ui, &ctx, gui_scale)
    })
    .inner
}

/// Plain (non-hot) eframe app used by `run()`.
#[derive(Default)]
struct PreviewApp {
    state: PreviewState,
}

impl eframe::App for PreviewApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if !draw_frame(&mut self.state, ui) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
