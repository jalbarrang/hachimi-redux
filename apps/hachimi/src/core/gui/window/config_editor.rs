use std::{ops::RangeInclusive, sync::Arc, thread};

use rust_i18n::t;

use crate::core::{
    hachimi::{self, Language},
    utils::get_localized_string,
    Hachimi,
};
use crate::il2cpp::hook::{
    umamusume::{
        CameraData::ShadowResolution,
        CySpringController::SpringUpdateMode,
        GraphicSettings::{GraphicsQuality, MsaaQuality},
        TimeUtil::BgSeason,
    },
    UnityEngine_CoreModule::Texture::AnisoLevel,
};

use egui_taffy::taffy::prelude::{auto, fr, length, min_content};
use egui_taffy::{taffy, tui, Tui, TuiBuilderLogic, TuiContainerResponse};

use crate::core::plugin::overlay;

use super::super::scale::get_scale;
use super::super::widgets;
use super::super::Gui;
use super::{random_id, save_and_reload_config, LiveVocalsSwapWindow, SimpleOkDialog, ThemeEditorWindow};

// ── egui_taffy layout helpers for the two-column settings grids ────────────

/// Two-column settings grid: a `min_content` label column (sized to the widest
/// label, single line) + an `fr` control column that fills the rest. Idiomatic
/// egui_taffy (its README grid example); relies on egui multi-pass (see
/// `frame::run`) to settle, and on `wrap_mode = Extend` so labels render on one
/// line instead of a glyph-per-line column.
fn cfg_grid_style(scale: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Grid,
        grid_template_columns: vec![min_content(), fr(1.0)],
        grid_auto_rows: vec![min_content()],
        gap: taffy::Size {
            width: length(24.0 * scale),
            height: length(6.0 * scale),
        },
        align_items: Some(taffy::AlignItems::Center),
        ..Default::default()
    }
}

fn cell_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        align_items: Some(taffy::AlignItems::Center),
        justify_content: Some(taffy::JustifyContent::Start),
        min_size: taffy::Size {
            width: length(0.0),
            height: auto(),
        },
        ..Default::default()
    }
}

/// A label cell (wraps within the fixed label column).
fn label_cell(tui: &mut Tui, text: impl Into<egui::WidgetText>) {
    tui.style(cell_style()).add(|tui| {
        tui.ui(|ui| {
            ui.label(text);
        });
    });
}

/// A content-sized control cell (checkbox / combo / button): stable size.
fn auto_cell<R>(tui: &mut Tui, content: impl FnOnce(&mut egui::Ui) -> R) -> R {
    tui.style(cell_style()).add(|tui| tui.ui(content))
}

/// A width-filling control cell (slider / text edit). Reports a constant,
/// width-independent size via `ui_manual` so the filled width is not fed back
/// into fr-track sizing (which would keep the node dirty and flicker).
fn fill_cell<R>(tui: &mut Tui, content: impl FnOnce(&mut egui::Ui) -> R) -> R {
    tui.style(cell_style()).add(|tui| {
        tui.ui_manual(|ui, _| {
            let inner = content(ui);
            let h = ui.min_size().y;
            TuiContainerResponse {
                inner,
                min_size: egui::Vec2::new(0.0, h),
                intrinsic_size: None,
                max_size: egui::Vec2::new(0.0, h),
                infinite: egui::Vec2b::new(true, false),
            }
        })
    })
}

pub(crate) struct ConfigEditor {
    last_ptr_config: usize,
    config: hachimi::Config,
    id: egui::Id,
    current_tab: ConfigEditorTab,
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
        }
    }

    /// Discard unsaved edits: reset the working copy to the currently saved config
    /// and re-apply its language locale (the language combo applies locale live).
    fn revert(&mut self) {
        let handle = Hachimi::instance().config.load();
        self.last_ptr_config = Arc::as_ptr(&handle) as usize;
        self.config = (**handle).clone();
        self.config.language.set_locale();
    }

    fn option_slider<Num: egui::emath::Numeric>(
        tui: &mut Tui,
        label: &str,
        value: &mut Option<Num>,
        range: RangeInclusive<Num>,
    ) {
        let mut checked = value.is_some();
        label_cell(tui, label);
        auto_cell(tui, |ui| {
            ui.checkbox(&mut checked, t!("enable"));
        });

        if checked && value.is_none() {
            *value = Some(*range.start())
        } else if !checked && value.is_some() {
            *value = None;
        }

        if let Some(num) = value.as_mut() {
            label_cell(tui, "");
            fill_cell(tui, |ui| {
                ui.add(egui::Slider::new(num, range));
            });
        }
    }

    fn run_options_grid(config: &mut hachimi::Config, tui: &mut Tui, tab: ConfigEditorTab) {
        match tab {
            ConfigEditorTab::General => {
                label_cell(tui, t!("config_editor.language"));
                let lang_changed = auto_cell(tui, |ui| {
                    Gui::run_combo(ui, "language", &mut config.language, Language::CHOICES)
                });
                if lang_changed {
                    config.language.set_locale();
                }

                label_cell(tui, t!("config_editor.disable_overlay"));
                auto_cell(tui, |ui| {
                    if ui.checkbox(&mut config.disable_gui, "").clicked() && config.disable_gui {
                        thread::spawn(|| {
                            Gui::instance()
                                .expect("unexpected failure")
                                .lock()
                                .expect("unexpected failure")
                                .show_window(Box::new(SimpleOkDialog::new(
                                    &t!("warning"),
                                    &t!("config_editor.disable_overlay_warning"),
                                    || {},
                                )));
                        });
                    }
                });

                label_cell(tui, t!("config_editor.ipv4_only"));
                auto_cell(tui, |ui| {
                    ui.checkbox(&mut config.ipv4_only, "");
                });

                label_cell(tui, t!("config_editor.gui_scale"));
                fill_cell(tui, |ui| {
                    ui.add(egui::Slider::new(&mut config.gui_scale, 0.25..=2.0).step_by(0.05));
                });

                #[cfg(target_os = "windows")]
                {
                    label_cell(tui, t!("config_editor.gui_landscape_ratio"));
                    fill_cell(tui, |ui| {
                        ui.add(
                            egui::Slider::new(&mut config.windows.gui_landscape_ratio, 0.25..=1.0)
                                .step_by(0.05)
                                .fixed_decimals(2),
                        );
                    });
                }

                label_cell(tui, t!("theme_editor.title"));
                auto_cell(tui, |ui| {
                    if widgets::secondary_button(ui, t!("open").into_owned()).clicked() {
                        thread::spawn(|| {
                            Gui::instance()
                                .expect("unexpected failure")
                                .lock()
                                .expect("unexpected failure")
                                .show_window(Box::new(ThemeEditorWindow::new()));
                        });
                    }
                });

                #[cfg(target_os = "windows")]
                {
                    label_cell(tui, t!("config_editor.discord_rpc"));
                    auto_cell(tui, |ui| {
                        ui.checkbox(&mut config.windows.discord_rpc, "");
                    });
                }

                label_cell(tui, t!("config_editor.debug_mode"));
                auto_cell(tui, |ui| {
                    ui.checkbox(&mut config.debug_mode, "");
                });

                label_cell(tui, t!("config_editor.enable_file_logging"));
                auto_cell(tui, |ui| {
                    ui.checkbox(&mut config.enable_file_logging, "");
                });

                label_cell(tui, t!("config_editor.apply_atlas_workaround"));
                auto_cell(tui, |ui| {
                    ui.checkbox(&mut config.apply_atlas_workaround, "");
                });

                label_cell(tui, t!("config_editor.skip_first_time_setup"));
                auto_cell(tui, |ui| {
                    ui.checkbox(&mut config.skip_first_time_setup, "");
                });

                label_cell(tui, t!("config_editor.disable_auto_update_check"));
                auto_cell(tui, |ui| {
                    ui.checkbox(&mut config.disable_auto_update_check, "");
                });

                label_cell(tui, t!("config_editor.enable_ipc"));
                auto_cell(tui, |ui| {
                    ui.checkbox(&mut config.enable_ipc, "");
                });

                label_cell(tui, t!("config_editor.ipc_listen_all"));
                auto_cell(tui, |ui| {
                    ui.checkbox(&mut config.ipc_listen_all, "");
                });
            }

            ConfigEditorTab::Graphics => {
                Self::option_slider(tui, &t!("config_editor.target_fps"), &mut config.target_fps, 30..=1000);

                label_cell(tui, t!("config_editor.virtual_resolution_multiplier"));
                fill_cell(tui, |ui| {
                    ui.add(egui::Slider::new(&mut config.virtual_res_mult, 1.0..=4.0).step_by(0.1));
                });

                label_cell(tui, t!("config_editor.ui_scale"));
                fill_cell(tui, |ui| {
                    ui.add(egui::Slider::new(&mut config.ui_scale, 0.1..=10.0).step_by(0.05));
                });

                label_cell(tui, t!("config_editor.ui_animation_scale"));
                fill_cell(tui, |ui| {
                    ui.add(egui::Slider::new(&mut config.ui_animation_scale, 0.1..=10.0).step_by(0.1));
                });

                label_cell(tui, t!("config_editor.loading_fade_scale"));
                fill_cell(tui, |ui| {
                    ui.add(egui::Slider::new(&mut config.loading_fade_scale, 0.1..=10.0).step_by(0.1));
                });

                label_cell(tui, t!("config_editor.flash_animation_scale"));
                fill_cell(tui, |ui| {
                    ui.add(egui::Slider::new(&mut config.flash_animation_scale, 0.1..=10.0).step_by(0.1));
                });

                label_cell(tui, t!("config_editor.render_scale"));
                fill_cell(tui, |ui| {
                    ui.add(egui::Slider::new(&mut config.render_scale, 0.1..=10.0).step_by(0.1));
                });

                label_cell(tui, t!("config_editor.msaa"));
                auto_cell(tui, |ui| {
                    Gui::run_combo(
                        ui,
                        "msaa",
                        &mut config.msaa,
                        &[
                            (MsaaQuality::Disabled, &t!("default")),
                            (MsaaQuality::_2x, "2x"),
                            (MsaaQuality::_4x, "4x"),
                            (MsaaQuality::_8x, "8x"),
                        ],
                    );
                });

                label_cell(tui, t!("config_editor.aniso_level"));
                auto_cell(tui, |ui| {
                    Gui::run_combo(
                        ui,
                        "aniso_level",
                        &mut config.aniso_level,
                        &[
                            (AnisoLevel::Default, &t!("default")),
                            (AnisoLevel::_2x, "2x"),
                            (AnisoLevel::_4x, "4x"),
                            (AnisoLevel::_8x, "8x"),
                            (AnisoLevel::_16x, "16x"),
                        ],
                    );
                });

                label_cell(tui, t!("config_editor.shadow_resolution"));
                auto_cell(tui, |ui| {
                    Gui::run_combo(
                        ui,
                        "shadow_resolution",
                        &mut config.shadow_resolution,
                        &[
                            (ShadowResolution::Default, &t!("default")),
                            (ShadowResolution::_256, "256x"),
                            (ShadowResolution::_512, "512x"),
                            (ShadowResolution::_1024, "1K"),
                            (ShadowResolution::_2048, "2K"),
                            (ShadowResolution::_4096, "4K"),
                        ],
                    );
                });

                label_cell(tui, t!("config_editor.graphics_quality"));
                auto_cell(tui, |ui| {
                    Gui::run_combo(
                        ui,
                        "graphics_quality",
                        &mut config.graphics_quality,
                        &[
                            (GraphicsQuality::Default, &t!("default")),
                            (GraphicsQuality::Toon1280, "Toon1280"),
                            (GraphicsQuality::Toon1280x2, "Toon1280x2"),
                            (GraphicsQuality::Toon1280x4, "Toon1280x4"),
                            (GraphicsQuality::ToonFull, "ToonFull"),
                            (GraphicsQuality::Max, "Max"),
                        ],
                    );
                });

                #[cfg(target_os = "windows")]
                {
                    use crate::windows::hachimi_impl::{FullScreenMode, ResolutionScaling};

                    label_cell(tui, t!("config_editor.vsync"));
                    auto_cell(tui, |ui| {
                        Gui::run_vsync_combo(ui, &mut config.windows.vsync_count);
                    });

                    label_cell(tui, t!("config_editor.auto_full_screen"));
                    auto_cell(tui, |ui| {
                        ui.checkbox(&mut config.windows.auto_full_screen, "");
                    });

                    label_cell(tui, t!("config_editor.full_screen_mode"));
                    auto_cell(tui, |ui| {
                        Gui::run_combo(
                            ui,
                            "full_screen_mode",
                            &mut config.windows.full_screen_mode,
                            &[
                                (
                                    FullScreenMode::ExclusiveFullScreen,
                                    &t!("config_editor.full_screen_mode_exclusive"),
                                ),
                                (
                                    FullScreenMode::FullScreenWindow,
                                    &t!("config_editor.full_screen_mode_borderless"),
                                ),
                            ],
                        );
                    });

                    label_cell(tui, t!("config_editor.block_minimize_in_full_screen"));
                    auto_cell(tui, |ui| {
                        ui.checkbox(&mut config.windows.block_minimize_in_full_screen, "");
                    });

                    label_cell(tui, t!("config_editor.resolution_scaling"));
                    auto_cell(tui, |ui| {
                        Gui::run_combo(
                            ui,
                            "resolution_scaling",
                            &mut config.windows.resolution_scaling,
                            &[
                                (
                                    ResolutionScaling::Default,
                                    &t!("config_editor.resolution_scaling_default"),
                                ),
                                (
                                    ResolutionScaling::ScaleToScreenSize,
                                    &t!("config_editor.resolution_scaling_ssize"),
                                ),
                                (
                                    ResolutionScaling::ScaleToWindowSize,
                                    &t!("config_editor.resolution_scaling_wsize"),
                                ),
                            ],
                        );
                    });

                    label_cell(tui, t!("config_editor.window_always_on_top"));
                    auto_cell(tui, |ui| {
                        ui.checkbox(&mut config.windows.window_always_on_top, "");
                    });
                }
            }

            ConfigEditorTab::Gameplay => {
                label_cell(tui, t!("config_editor.physics_update_mode"));
                auto_cell(tui, |ui| {
                    Gui::run_combo(
                        ui,
                        "physics_update_mode",
                        &mut config.physics_update_mode,
                        &[
                            (None, &t!("default")),
                            (SpringUpdateMode::ModeNormal.into(), "ModeNormal"),
                            (SpringUpdateMode::Mode60FPS.into(), "Mode60FPS"),
                            (SpringUpdateMode::SkipFrame.into(), "SkipFrame"),
                            (SpringUpdateMode::SkipFramePostAlways.into(), "SkipFramePostAlways"),
                        ],
                    );
                });

                label_cell(tui, t!("config_editor.story_choice_auto_select_delay"));
                fill_cell(tui, |ui| {
                    ui.add(egui::Slider::new(&mut config.story_choice_auto_select_delay, 0.1..=10.0).step_by(0.05));
                });

                label_cell(tui, t!("config_editor.story_text_speed_multiplier"));
                fill_cell(tui, |ui| {
                    ui.add(egui::Slider::new(&mut config.story_tcps_multiplier, 0.1..=10.0).step_by(0.1));
                });

                label_cell(tui, t!("config_editor.force_allow_dynamic_camera"));
                auto_cell(tui, |ui| {
                    ui.checkbox(&mut config.force_allow_dynamic_camera, "");
                });

                label_cell(tui, t!("config_editor.live_theater_allow_same_chara"));
                auto_cell(tui, |ui| {
                    ui.checkbox(&mut config.live_theater_allow_same_chara, "");
                });

                label_cell(tui, t!("config_editor.live_vocals_swap"));
                auto_cell(tui, |ui| {
                    if widgets::secondary_button(ui, t!("open").into_owned()).clicked() {
                        thread::spawn(|| {
                            Gui::instance()
                                .expect("unexpected failure")
                                .lock()
                                .expect("unexpected failure")
                                .show_window(Box::new(LiveVocalsSwapWindow::new()));
                        });
                    }
                });

                label_cell(tui, t!("config_editor.skill_info_dialog"));
                auto_cell(tui, |ui| {
                    ui.checkbox(&mut config.skill_info_dialog, "");
                });

                label_cell(tui, t!("config_editor.homescreen_bgseason"));
                auto_cell(tui, |ui| {
                    Gui::run_combo(
                        ui,
                        "homescreen_bgseason",
                        &mut config.homescreen_bgseason,
                        &[
                            (BgSeason::None, &t!("default")),
                            // Season text from TextId enum
                            (BgSeason::Spring, get_localized_string("Common0108").as_str()),
                            (BgSeason::Summer, get_localized_string("Common0109").as_str()),
                            (BgSeason::Fall, get_localized_string("Common0110").as_str()),
                            (BgSeason::Winter, get_localized_string("Common0111").as_str()),
                            (BgSeason::CherryBlossom, get_localized_string("Common0112").as_str()),
                        ],
                    );
                });
            }
            // Rendered separately by `ui_hotkeys`; never reaches the options grid.
            ConfigEditorTab::Hotkeys => {}
        }
    }

    /// Translation-related options (moved out of the General/Gameplay grids into the
    /// dedicated Translations tab). Edits the same shared working copy.
    fn run_translations_grid(config: &mut hachimi::Config, tui: &mut Tui) {
        label_cell(tui, t!("config_editor.meta_index_url"));
        fill_cell(tui, |ui| {
            let res = ui.add(egui::TextEdit::singleline(&mut config.meta_index_url).lock_focus(true));
            #[cfg(target_os = "windows")]
            if res.has_focus() {
                ui.memory_mut(|mem| {
                    mem.set_focus_lock_filter(
                        res.id,
                        egui::EventFilter {
                            tab: true,
                            horizontal_arrows: true,
                            vertical_arrows: true,
                            escape: true,
                        },
                    )
                });
            }
            #[cfg(not(target_os = "windows"))]
            let _ = res;
        });

        label_cell(tui, t!("config_editor.disable_translations"));
        auto_cell(tui, |ui| {
            ui.checkbox(&mut config.disable_translations, "");
        });

        label_cell(tui, t!("config_editor.lazy_translation_updates"));
        auto_cell(tui, |ui| {
            ui.checkbox(&mut config.lazy_translation_updates, "");
        });

        label_cell(tui, t!("config_editor.translator_mode"));
        auto_cell(tui, |ui| {
            ui.checkbox(&mut config.translator_mode, "");
        });

        label_cell(tui, t!("config_editor.disable_skill_name_translation"));
        auto_cell(tui, |ui| {
            ui.checkbox(&mut config.disable_skill_name_translation, "");
        });

        label_cell(tui, t!("config_editor.auto_translate_stories"));
        auto_cell(tui, |ui| {
            if ui.checkbox(&mut config.auto_translate_stories, "").clicked() && config.auto_translate_stories {
                thread::spawn(|| {
                    Gui::instance()
                        .expect("unexpected failure")
                        .lock()
                        .expect("unexpected failure")
                        .show_window(Box::new(SimpleOkDialog::new(
                            &t!("warning"),
                            &t!("config_editor.auto_tl_warning"),
                            || {},
                        )));
                });
            }
        });

        label_cell(tui, t!("config_editor.auto_translate_ui"));
        auto_cell(tui, |ui| {
            if ui.checkbox(&mut config.auto_translate_localize, "").clicked() && config.auto_translate_localize {
                thread::spawn(|| {
                    Gui::instance()
                        .expect("unexpected failure")
                        .lock()
                        .expect("unexpected failure")
                        .show_window(Box::new(SimpleOkDialog::new(
                            &t!("warning"),
                            &t!("config_editor.auto_tl_warning"),
                            || {},
                        )));
                });
            }
        });
    }
}

impl ConfigEditor {
    /// Sync the working copy if the saved config changed underneath us.
    fn sync(&mut self) {
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

        // Hotkeys renders its own body (no options grid).
        if tab == ConfigEditorTab::Hotkeys {
            super::hotkeys_editor::ui_hotkeys(ui, ctx);
            return;
        }

        let scale = get_scale(ctx);
        let id = self.id;
        // Distinct grid id per tab so egui_taffy doesn't reuse layout state across
        // tabs whose cell sets differ.
        let grid_id = match tab {
            ConfigEditorTab::General => "grid_general",
            ConfigEditorTab::Graphics => "grid_graphics",
            ConfigEditorTab::Gameplay => "grid_gameplay",
            ConfigEditorTab::Hotkeys => "grid_hotkeys",
        };
        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(8, 0))
            .show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                tui(ui, id.with(grid_id))
                    .reserve_available_width()
                    .style(cfg_grid_style(scale))
                    .show(|tui| {
                        Self::run_options_grid(&mut self.config, tui, tab);
                    });
            });

        // General hosts the live overlay controls (the old Overlay tab is gone).
        if tab == ConfigEditorTab::General {
            Self::ui_overlays_section(ui);
        }
    }

    /// Live overlay controls relocated from the removed Overlay tab: global
    /// opacity + per-panel show/hide and reset-position. These are runtime overlay
    /// prefs (apply immediately, not part of the Save/Cancel working copy). The
    /// global lock toggle was dropped per design; per-panel show/hide stays until
    /// the overlay-toggle-hotkeys plan replaces it.
    fn ui_overlays_section(ui: &mut egui::Ui) {
        ui.add_space(8.0);
        widgets::section_header(ui, t!("config_editor.overlays_heading").into_owned());
        let mut opacity = overlay::opacity();
        ui.horizontal(|ui| {
            ui.label(t!("config_editor.overlay_opacity"));
            if ui
                .add(egui::Slider::new(&mut opacity, 0.1..=1.0).fixed_decimals(2))
                .changed()
            {
                overlay::set_opacity(opacity);
            }
        });

        let overlays = overlay::get_plugin_overlays();
        if overlays.is_empty() {
            ui.weak(t!("config_editor.overlays_none"));
            return;
        }
        for ov in &overlays {
            let title = overlay::display_title(&ov.id);
            let mut visible = overlay::is_overlay_visible(&ov.id);
            ui.horizontal(|ui| {
                if widgets::toggle_ui(ui, &mut visible).changed() {
                    overlay::set_overlay_visible(&ov.id, visible);
                }
                ui.label(&title);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if widgets::ghost_button(ui, t!("config_editor.overlay_reset").into_owned())
                        .on_hover_text(t!("config_editor.overlay_reset_hint"))
                        .clicked()
                    {
                        overlay::reset_panel(&ov.id);
                    }
                });
            });
        }
    }

    /// Translations tab body: the translation-related options grid.
    pub(crate) fn ui_translations(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let scale = get_scale(ctx);
        self.sync();

        let id = self.id;
        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(8, 0))
            .show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                tui(ui, id.with("grid_translations"))
                    .reserve_available_width()
                    .style(cfg_grid_style(scale))
                    .show(|tui| {
                        Self::run_translations_grid(&mut self.config, tui);
                    });
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
        let config = &self.config;
        ui.add_enabled_ui(enabled, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            tui(ui, id.with("footer"))
                .reserve_available_width()
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
                    auto_cell(tui, |ui| {
                        if widgets::secondary_button(ui, t!("config_editor.cancel").into_owned()).clicked() {
                            cancel_clicked = true;
                        }
                    });
                    auto_cell(tui, |ui| {
                        if widgets::primary_button(ui, t!("save").into_owned()).clicked() {
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
