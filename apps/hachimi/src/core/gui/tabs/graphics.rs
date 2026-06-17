//! L1 Graphics tab — rendering and display config options.

use std::ops::RangeInclusive;

use rust_i18n::t;

use crate::core::hachimi;
use crate::il2cpp::hook::{
    umamusume::{
        CameraData::ShadowResolution,
        GraphicSettings::{GraphicsQuality, MsaaQuality},
    },
    UnityEngine_CoreModule::Texture::AnisoLevel,
};

use egui_taffy::Tui;

use super::super::Gui;
use super::layout::{auto_cell, fill_cell, label_cell};

pub(crate) fn option_slider<Num: egui::emath::Numeric>(
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

pub(crate) fn options(config: &mut hachimi::Config, tui: &mut Tui) {
    option_slider(tui, &t!("config_editor.target_fps"), &mut config.target_fps, 30..=1000);

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
