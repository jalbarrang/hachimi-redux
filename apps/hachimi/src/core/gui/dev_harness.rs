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

use super::menu::{render_control_center, ControlCenterHost, ControlTab};
use super::scale::get_scale;
use super::window::{ConfigEditor, ConfigEditorTab};
use super::Gui;

/// Entry point invoked by the `menu_preview` example.
pub fn run() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("HachimiRedux — Control Center preview")
            .with_inner_size([720.0, 1000.0]),
        ..Default::default()
    };

    eframe::run_native(
        "hachimi-menu-preview",
        options,
        Box::new(|cc| {
            let ctx = &cc.egui_ctx;

            // egui_taffy settles its layout via in-frame multi-pass; without this
            // every taffy surface flickers (matches the in-game Context config).
            ctx.options_mut(|o| o.max_passes = std::num::NonZeroUsize::new(3).expect("nonzero"));

            // Faithful chrome: the real fonts, theme and image loaders (for the
            // title icon, which is an embedded PNG).
            let config = Config::default();
            ctx.set_fonts(Gui::get_font_definitions());
            let mut style = egui::Style::default();
            Gui::apply_theme(ctx, &mut style, &config);
            ctx.set_style(style);
            egui_extras::install_image_loaders(ctx);

            Ok(Box::new(Harness {
                menu_tab: ControlTab::default(),
                config_editor: ConfigEditor::new_detached(config),
            }))
        }),
    )
}

struct Harness {
    menu_tab: ControlTab,
    config_editor: ConfigEditor,
}

impl eframe::App for Harness {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Mirror the working copy's GUI scale into the context, like the in-game
        // render loop does, so the preview reflects the General → GUI scale slider.
        let scale = self.config_editor.working_config().gui_scale;
        ctx.data_mut(|d| d.insert_temp(egui::Id::new("gui_scale"), scale));

        egui::CentralPanel::default().show(ctx, |ui| {
            let keep_open = render_control_center(ui, ctx, get_scale(ctx), self);
            if !keep_open {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }
}

impl ControlCenterHost for Harness {
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
