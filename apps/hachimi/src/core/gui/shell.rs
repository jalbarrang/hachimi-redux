//! Control Center modal shell: egui-native tab bar + body dispatch.

use rust_i18n::t;

use super::components;
use super::theme::Tokens;
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
    #[cfg(feature = "training-tracker")]
    TrainingTracker,
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

    fn all_tabs() -> Vec<(ControlTab, String)> {
        #[allow(unused_mut)]
        let mut tabs = vec![
            (Self::General, t!("config_editor.general_tab").to_string()),
            (Self::Graphics, t!("config_editor.graphics_tab").to_string()),
            (Self::Gameplay, t!("config_editor.gameplay_tab").to_string()),
            (Self::Hotkeys, t!("config_editor.hotkeys_tab").to_string()),
            (Self::Translations, "\u{f1ab} Translations".into()),
            (Self::Plugins, "\u{f12e} Plugins".into()),
            (Self::About, "\u{f129} About".into()),
        ];
        #[cfg(feature = "training-tracker")]
        tabs.insert(5, (Self::TrainingTracker, "\u{f201} Training Tracker".into()));
        tabs
    }
}

/// Render the egui-native Control Center for the live `Gui`.
pub(crate) fn render_control_center_gui(gui: &mut Gui, ui: &mut egui::Ui, ctx: &egui::Context, scale: f32) -> bool {
    gui.config_editor.sync();
    let tokens = Tokens::DEFAULT;
    let shell_w = SHELL_WIDTH * scale;
    let shell_h = ctx.input(|i| i.viewport_rect().height()) * 0.85;
    ui.set_width(shell_w);
    ui.set_max_height(shell_h);
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

    let mut keep_open = true;
    let mut windows_to_show: Vec<super::BoxedWindow> = Vec::new();
    let mut save_requested = false;
    let mut revert_requested = false;
    let mut apply_resolution = false;

    let tab = gui.menu_tab;
    let footer_on = tab.edits_config();

    // ── Outer panel ──────────────────────────────────────────────────
    egui::Frame::NONE
        .fill(tokens.surface_1)
        .stroke(egui::Stroke::new(1.0, tokens.line))
        .corner_radius(tokens.radius_card)
        .show(ui, |ui| {
            ui.set_width(shell_w);

            // ── Header ──────────────────────────────────────────────
            ui.add_space(14.0 * scale);
            ui.horizontal(|ui| {
                ui.add_space(14.0 * scale);
                ui.add(Gui::icon(ctx));
                ui.add_space(8.0 * scale);
                ui.label(
                    egui::RichText::new(t!("hachimi"))
                        .size(18.0 * scale)
                        .strong()
                        .color(tokens.fg),
                );
                ui.add_space(4.0 * scale);
                ui.label(
                    egui::RichText::new(env!("HACHIMI_DISPLAY_VERSION"))
                        .size(12.0 * scale)
                        .color(tokens.fg_dim),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(14.0 * scale);
                    if components::ghost_button(ui, "\u{f00d}").clicked() {
                        keep_open = false;
                    }
                });
            });
            ui.add_space(8.0 * scale);

            // ── Tab bar ─────────────────────────────────────────────
            draw_tab_bar(ui, &mut gui.menu_tab, scale);
            ui.add_space(4.0 * scale);
            ui.separator();

            // ── Body scroll area ────────────────────────────────────
            let footer_h = if footer_on { 52.0 * scale } else { 0.0 };
            // Reserve space for header (~60px), tab bar (~32px), separator, footer.
            let body_h = (shell_h - 100.0 * scale - footer_h).max(100.0 * scale);
            egui::ScrollArea::vertical()
                .id_salt("cc_body")
                .max_height(body_h)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(8.0 * scale);
                    egui::Frame::NONE
                        .inner_margin(egui::Margin::symmetric((14.0 * scale) as i8, 0))
                        .show(ui, |ui| {
                            draw_tab_body_live(gui.menu_tab, ui, ctx, gui, &mut windows_to_show, &mut apply_resolution);
                        });
                    ui.add_space(8.0 * scale);
                });

            // ── Footer ──────────────────────────────────────────────
            if footer_on {
                ui.separator();
                ui.add_space(4.0 * scale);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(14.0 * scale);
                        if components::primary_button(ui, t!("save").to_string()).clicked() {
                            save_requested = true;
                        }
                        if components::secondary_button(ui, t!("config_editor.cancel").to_string()).clicked() {
                            revert_requested = true;
                        }
                    });
                });
                ui.add_space(8.0 * scale);
            }
        });

    // ── Apply deferred side-effects ─────────────────────────────────
    // Apply AFTER drawing so the X-button close propagates this frame
    // (mirrors the original drain_actions ordering).
    for w in windows_to_show {
        gui.show_window(w);
    }
    if save_requested && !gui.config_editor.is_detached() {
        super::window::save_and_reload_config(gui.config_editor.config().clone());
    }
    if revert_requested {
        gui.config_editor.revert_edits();
    }
    if apply_resolution && !gui.config_editor.is_detached() {
        super::window::save_and_reload_config(gui.config_editor.config().clone());
        #[cfg(target_os = "windows")]
        crate::windows::hachimi_impl::apply_current_resolution();
    }

    keep_open
}

/// Render the egui-native Control Center for the desktop preview harness.
#[cfg(feature = "dev-harness")]
pub(crate) fn render_control_center_preview(
    menu_tab: &mut ControlTab,
    editor: &mut ConfigEditor,
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    scale: f32,
) -> bool {
    editor.sync();
    let tokens = Tokens::DEFAULT;
    let shell_w = SHELL_WIDTH * scale;
    let shell_h = ctx.input(|i| i.viewport_rect().height()) * 0.85;
    ui.set_width(shell_w);
    ui.set_max_height(shell_h);
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

    let mut keep_open = true;
    let mut revert_requested = false;

    let tab = *menu_tab;
    let footer_on = tab.edits_config();

    egui::Frame::NONE
        .fill(tokens.surface_1)
        .stroke(egui::Stroke::new(1.0, tokens.line))
        .corner_radius(tokens.radius_card)
        .show(ui, |ui| {
            ui.set_width(shell_w);

            // Header
            ui.add_space(14.0 * scale);
            ui.horizontal(|ui| {
                ui.add_space(14.0 * scale);
                ui.add(Gui::icon(ctx));
                ui.add_space(8.0 * scale);
                ui.label(
                    egui::RichText::new(t!("hachimi"))
                        .size(18.0 * scale)
                        .strong()
                        .color(tokens.fg),
                );
                ui.add_space(4.0 * scale);
                ui.label(
                    egui::RichText::new(env!("HACHIMI_DISPLAY_VERSION"))
                        .size(12.0 * scale)
                        .color(tokens.fg_dim),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(14.0 * scale);
                    if components::ghost_button(ui, "\u{f00d}").clicked() {
                        keep_open = false;
                    }
                });
            });
            ui.add_space(8.0 * scale);

            draw_tab_bar(ui, menu_tab, scale);
            ui.add_space(4.0 * scale);
            ui.separator();

            let footer_h = if footer_on { 52.0 * scale } else { 0.0 };
            let body_h = (shell_h - 100.0 * scale - footer_h).max(100.0 * scale);
            egui::ScrollArea::vertical()
                .id_salt("cc_body_preview")
                .max_height(body_h)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(8.0 * scale);
                    egui::Frame::NONE
                        .inner_margin(egui::Margin::symmetric((14.0 * scale) as i8, 0))
                        .show(ui, |ui| {
                            draw_tab_body_preview(*menu_tab, ui, ctx, editor);
                        });
                    ui.add_space(8.0 * scale);
                });

            if footer_on {
                ui.separator();
                ui.add_space(4.0 * scale);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(14.0 * scale);
                        // Save is a no-op in preview; just flash a visual.
                        components::primary_button(ui, t!("save").to_string());
                        if components::secondary_button(ui, t!("config_editor.cancel").to_string()).clicked() {
                            revert_requested = true;
                        }
                    });
                });
                ui.add_space(8.0 * scale);
            }
        });

    if revert_requested {
        editor.revert_edits();
    }

    keep_open
}

// ─── Shared helpers ─────────────────────────────────────────────────────────

fn draw_tab_bar(ui: &mut egui::Ui, active: &mut ControlTab, scale: f32) {
    let tokens = Tokens::DEFAULT;
    // Horizontal scrollable tab strip for narrow viewports.
    egui::ScrollArea::horizontal()
        .id_salt("cc_tab_bar")
        .max_width(ui.available_width())
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(14.0 * scale);
                for (tab, label) in ControlTab::all_tabs() {
                    let selected = tab == *active;
                    let text = if selected {
                        egui::RichText::new(&label).strong().color(tokens.accent)
                    } else {
                        egui::RichText::new(&label).color(tokens.fg_dim)
                    };
                    if ui
                        .add(
                            egui::Button::new(text)
                                .frame(false)
                                .min_size(egui::vec2(0.0, 24.0 * scale)),
                        )
                        .clicked()
                        && !selected
                    {
                        *active = tab;
                    }
                }
            });
        });
}

/// Live tab body dispatch — has full `Gui` access.
fn draw_tab_body_live(
    tab: ControlTab,
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    gui: &mut Gui,
    windows: &mut Vec<super::BoxedWindow>,
    apply_resolution: &mut bool,
) {
    match tab {
        ControlTab::General => {
            super::tabs::general::draw(ui, gui.config_editor.config_mut(), windows);
        }
        ControlTab::Graphics => {
            super::tabs::graphics::draw(ui, gui.config_editor.config_mut(), apply_resolution);
        }
        ControlTab::Gameplay => {
            super::tabs::gameplay::draw(ui, gui.config_editor.config_mut(), windows);
        }
        ControlTab::Hotkeys => {
            super::tabs::hotkeys::ui_hotkeys(ui, ctx, gui.config_editor.config_mut());
        }
        ControlTab::Translations => {
            super::tabs::translations::draw_full(ui, ctx, gui);
        }
        #[cfg(feature = "training-tracker")]
        ControlTab::TrainingTracker => {
            gui.draw_training_tracker_tab(ui);
        }
        ControlTab::Plugins => {
            gui.draw_plugins_tab(ui);
        }
        ControlTab::About => {
            gui.draw_about_tab(ui, ctx);
        }
    }
}

/// Preview tab body dispatch — config-editing tabs work, live-only tabs show stubs.
#[cfg(feature = "dev-harness")]
fn draw_tab_body_preview(tab: ControlTab, ui: &mut egui::Ui, ctx: &egui::Context, editor: &mut ConfigEditor) {
    match tab {
        ControlTab::General => {
            let mut windows = Vec::new();
            super::tabs::general::draw(ui, editor.config_mut(), &mut windows);
        }
        ControlTab::Graphics => {
            let mut apply = false;
            super::tabs::graphics::draw(ui, editor.config_mut(), &mut apply);
        }
        ControlTab::Gameplay => {
            let mut windows = Vec::new();
            super::tabs::gameplay::draw_preview(ui, editor.config_mut(), &mut windows);
        }
        ControlTab::Hotkeys => {
            super::tabs::hotkeys::ui_hotkeys(ui, ctx, editor.config_mut());
        }
        ControlTab::Translations => {
            super::tabs::translations::draw_config_settings(ui, editor.config_mut(), &mut Vec::new());
        }
        #[cfg(feature = "training-tracker")]
        ControlTab::TrainingTracker => {
            super::stub::stub_tab(ui, "Training Tracker", "Training Tracker needs the live game.");
        }
        ControlTab::Plugins => {
            super::stub::stub_tab(ui, "Plugins", "Plugin pages need the loaded plugin registry.");
        }
        ControlTab::About => {
            super::stub::stub_tab(ui, "About", "About actions need the live game.");
        }
    }
}
