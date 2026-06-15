//! Desktop preview harness for the Control Center menu.
//!
//! Renders the *same* menu shell + config tab bodies as the in-game overlay
//! (via the shared [`render_control_center`]), but inside a plain `eframe` window
//! driven by a default config — no game process, no IL2CPP, no D3D11. This lets
//! layout/styling be iterated on with a ~1s rebuild instead of launching the
//! Honse game.
//!
//! Run it with:
//!
//! ```sh
//! cargo run -p hachimi --example menu_preview --features dev-harness
//! ```
//!
//! Caveats vs. in-game: fonts/DPI differ slightly from the host, the homescreen
//! season combo shows static English placeholders, and the Save / plugin / about
//! actions are inert. Everything layout-related is faithful because it is the
//! exact same draw code on the same egui version.

use crate::core::hachimi::Config;

use super::menu::{render_control_center, ControlCenterHost, ControlTab, SHELL_WIDTH};
use super::scale::get_scale;
use super::window::{ConfigEditor, ConfigEditorTab};
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
    ctx.set_style(style);
    egui_extras::install_image_loaders(ctx);
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

/// Draw one frame of the preview. Returns whether the window should stay open.
/// This is the hot-reloadable unit: the `menu-hot` dylib forwards to it, and the
/// example calls it through that dylib so edits to the shell/widgets swap live.
pub fn draw_frame(state: &mut PreviewState, ctx: &egui::Context) -> bool {
    // Mirror the working copy's GUI scale into the context, like the in-game
    // render loop does, so the preview reflects the General → GUI scale slider.
    let scale = state.config_editor.working_config().gui_scale;
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("gui_scale"), scale));

    // Pull the preview window to the foreground on first paint.
    if !state.brought_to_front {
        state.brought_to_front = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    let mut keep_open = true;
    let desktop = egui::Frame::default().fill(egui::Color32::from_rgb(0, 180, 0));
    egui::CentralPanel::default().frame(desktop).show(ctx, |ui| {
        // Center the modal shell within the desktop by handing it an explicit
        // child rect (NOT a centered layout or a horizontal pad wrapper — both
        // disturb the shell's internal taffy + scroll-area sizing). Mirrors the
        // in-game `egui::Modal` which centers on screen.
        let gui_scale = get_scale(ctx);
        let shell_w = SHELL_WIDTH * gui_scale;
        let shell_h = ctx.input(|i| i.viewport_rect().height()) * 0.85;
        let full = ui.available_rect_before_wrap();
        let left = full.left() + ((full.width() - shell_w) * 0.5).max(0.0);
        let top = full.top() + ((full.height() - shell_h) * 0.5).max(0.0);
        let rect = egui::Rect::from_min_size(egui::pos2(left, top), egui::vec2(shell_w, shell_h));

        keep_open = ui
            .scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                render_control_center(ui, ctx, gui_scale, state)
            })
            .inner;
    });
    keep_open
}

/// Plain (non-hot) eframe app used by `run()`.
#[derive(Default)]
struct PreviewApp {
    state: PreviewState,
}

impl eframe::App for PreviewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !draw_frame(&mut self.state, ctx) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl ControlCenterHost for PreviewState {
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
        ui.add(Gui::icon(ctx));
    }

    fn draw_body(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, tab: ControlTab) {
        match tab {
            ControlTab::General => self.config_editor.ui_body(ui, ctx, ConfigEditorTab::General),
            ControlTab::Graphics => self.config_editor.ui_body(ui, ctx, ConfigEditorTab::Graphics),
            ControlTab::Gameplay => self.config_editor.ui_body(ui, ctx, ConfigEditorTab::Gameplay),
            ControlTab::Hotkeys => self.config_editor.ui_body(ui, ctx, ConfigEditorTab::Hotkeys),
            ControlTab::Translations => self.config_editor.ui_translations(ui, ctx),
            ControlTab::Plugins => stub(
                ui,
                "Plugins",
                "Plugin tab bodies need the loaded plugin registry (in-game only).",
            ),
            ControlTab::About => stub(
                ui,
                "About",
                "About actions (update check, links, soft restart) need the live game (in-game only).",
            ),
        }
    }
}

/// Placeholder body for tabs that can't render off-game.
fn stub(ui: &mut egui::Ui, title: &str, note: &str) {
    ui.add_space(12.0);
    ui.heading(title);
    ui.add_space(6.0);
    ui.weak(note);
}
