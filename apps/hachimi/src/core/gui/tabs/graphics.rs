//! Graphics tab — rendering and display options (egui-native).

use rust_i18n::t;

use crate::core::gui::components::{
    combo, primary_button, settings_grid, settings_label, settings_section, slider_f32, toggle,
};
use crate::core::hachimi;
use crate::il2cpp::hook::{
    umamusume::{
        CameraData::ShadowResolution,
        GraphicSettings::{GraphicsQuality, MsaaQuality},
    },
    UnityEngine_CoreModule::Texture::AnisoLevel,
};

/// Draw the Graphics tab body. Edits `config` in-place. Sets `apply_resolution`
/// to `true` if the user clicked the Apply button.
pub(crate) fn draw(ui: &mut egui::Ui, config: &mut hachimi::Config, apply_resolution: &mut bool) {
    settings_grid(ui, "gfx_settings", |ui| {
        // Target FPS (optional)
        settings_label(ui, &t!("config_editor.target_fps"));
        {
            let mut enabled = config.target_fps.is_some();
            let mut value = config.target_fps.unwrap_or(30) as f32;
            ui.vertical(|ui| {
                if toggle(ui, &t!("enable"), &mut enabled) {
                    config.target_fps = if enabled { Some(value as i32) } else { None };
                }
                if enabled && slider_f32(ui, &mut value, 30.0..=244.0, 1.0) {
                    config.target_fps = Some(value as i32);
                }
            });
        }
        ui.end_row();

        // UI scale
        settings_label(ui, &t!("config_editor.ui_scale"));
        slider_f32(ui, &mut config.ui_scale, 0.1..=10.0, 0.05);
        ui.end_row();

        // UI animation scale
        settings_label(ui, &t!("config_editor.ui_animation_scale"));
        slider_f32(ui, &mut config.ui_animation_scale, 0.1..=10.0, 0.1);
        ui.end_row();

        // Loading fade scale
        settings_label(ui, &t!("config_editor.loading_fade_scale"));
        slider_f32(ui, &mut config.loading_fade_scale, 0.1..=10.0, 0.1);
        ui.end_row();

        // Flash animation scale
        settings_label(ui, &t!("config_editor.flash_animation_scale"));
        slider_f32(ui, &mut config.flash_animation_scale, 0.1..=10.0, 0.1);
        ui.end_row();

        // MSAA
        settings_label(ui, &t!("config_editor.msaa"));
        combo(ui, "msaa", &mut config.msaa, &MSAA_CHOICES);
        ui.end_row();

        // Aniso level
        settings_label(ui, &t!("config_editor.aniso_level"));
        combo(ui, "aniso", &mut config.aniso_level, &ANISO_CHOICES);
        ui.end_row();

        // Shadow resolution
        settings_label(ui, &t!("config_editor.shadow_resolution"));
        combo(ui, "shadow", &mut config.shadow_resolution, &SHADOW_CHOICES);
        ui.end_row();

        // Graphics quality
        settings_label(ui, &t!("config_editor.graphics_quality"));
        combo(ui, "quality", &mut config.graphics_quality, &QUALITY_CHOICES);
        ui.end_row();

        // Windows-specific graphics options
        #[cfg(target_os = "windows")]
        draw_windows_graphics(ui, config);
    });

    // Resolution section
    settings_section(ui, &t!("config_editor.resolution_section"));
    settings_grid(ui, "res_settings", |ui| {
        // Virtual resolution multiplier
        settings_label(ui, &t!("config_editor.virtual_resolution_multiplier"));
        slider_f32(ui, &mut config.virtual_res_mult, 1.0..=4.0, 0.1);
        ui.end_row();

        // Render scale
        settings_label(ui, &t!("config_editor.render_scale"));
        slider_f32(ui, &mut config.render_scale, 0.1..=10.0, 0.1);
        ui.end_row();

        // Windows-specific resolution options
        #[cfg(target_os = "windows")]
        draw_windows_resolution(ui, config, apply_resolution);
    });
}

#[cfg(target_os = "windows")]
fn draw_windows_graphics(ui: &mut egui::Ui, config: &mut hachimi::Config) {
    use crate::windows::hachimi_impl::FullScreenMode;

    // VSync (native combo already exists)
    settings_label(ui, &t!("config_editor.vsync"));
    crate::core::Gui::run_vsync_combo(ui, &mut config.windows.vsync_count);
    ui.end_row();

    // Auto full screen
    settings_label(ui, &t!("config_editor.auto_full_screen"));
    toggle(ui, "", &mut config.windows.auto_full_screen);
    ui.end_row();

    // Full screen mode
    settings_label(ui, &t!("config_editor.full_screen_mode"));
    combo(
        ui,
        "fullscreen_mode",
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
    ui.end_row();

    // Block minimize
    settings_label(ui, &t!("config_editor.block_minimize_in_full_screen"));
    toggle(ui, "", &mut config.windows.block_minimize_in_full_screen);
    ui.end_row();

    // Window always on top
    settings_label(ui, &t!("config_editor.window_always_on_top"));
    toggle(ui, "", &mut config.windows.window_always_on_top);
    ui.end_row();
}

#[cfg(target_os = "windows")]
fn draw_windows_resolution(ui: &mut egui::Ui, config: &mut hachimi::Config, apply_resolution: &mut bool) {
    use crate::windows::hachimi_impl::ResolutionScaling;

    // Resolution scaling
    settings_label(ui, &t!("config_editor.resolution_scaling"));
    combo(
        ui,
        "res_scaling",
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
    ui.end_row();

    // Windowed resolution
    settings_label(ui, &t!("config_editor.windowed_resolution"));
    draw_resolution_combo(
        ui,
        "windowed_res",
        &mut config.windows.windowed_res.width,
        &mut config.windows.windowed_res.height,
    );
    ui.end_row();

    // Fullscreen resolution
    settings_label(ui, &t!("config_editor.full_screen_resolution"));
    draw_resolution_combo(
        ui,
        "fullscreen_res",
        &mut config.windows.full_screen_res.width,
        &mut config.windows.full_screen_res.height,
    );
    ui.end_row();

    // Refresh rate
    settings_label(ui, &t!("config_editor.refresh_rate"));
    combo(
        ui,
        "refresh",
        &mut config.windows.full_screen_res.refresh_rate,
        &REFRESH_CHOICES,
    );
    ui.end_row();

    // Apply button
    settings_label(ui, "");
    if primary_button(ui, t!("config_editor.apply_resolution").to_string()).clicked() {
        *apply_resolution = true;
    }
    ui.end_row();
}

#[cfg(target_os = "windows")]
fn draw_resolution_combo(ui: &mut egui::Ui, id: &str, width: &mut i32, height: &mut i32) {
    let current = if *width > 0 && *height > 0 {
        format!("{} \u{00d7} {}", width, height)
    } else {
        t!("config_editor.resolution_default").to_string()
    };
    egui::ComboBox::new(ui.id().with(id), "")
        .selected_text(&current)
        .show_ui(ui, |ui| {
            if ui
                .selectable_label(*width <= 0 && *height <= 0, t!("config_editor.resolution_default"))
                .clicked()
            {
                *width = 0;
                *height = 0;
            }
            for &(w, h) in &RES_PRESETS {
                let label = format!("{w} \u{00d7} {h}");
                if ui.selectable_label(*width == w && *height == h, label).clicked() {
                    *width = w;
                    *height = h;
                }
            }
        });
}

// ─── Static choice data ─────────────────────────────────────────────────────

const MSAA_CHOICES: [(MsaaQuality, &str); 4] = [
    (MsaaQuality::Disabled, "Default"),
    (MsaaQuality::_2x, "2x"),
    (MsaaQuality::_4x, "4x"),
    (MsaaQuality::_8x, "8x"),
];

const ANISO_CHOICES: [(AnisoLevel, &str); 5] = [
    (AnisoLevel::Default, "Default"),
    (AnisoLevel::_2x, "2x"),
    (AnisoLevel::_4x, "4x"),
    (AnisoLevel::_8x, "8x"),
    (AnisoLevel::_16x, "16x"),
];

const SHADOW_CHOICES: [(ShadowResolution, &str); 6] = [
    (ShadowResolution::Default, "Default"),
    (ShadowResolution::_256, "256x"),
    (ShadowResolution::_512, "512x"),
    (ShadowResolution::_1024, "1K"),
    (ShadowResolution::_2048, "2K"),
    (ShadowResolution::_4096, "4K"),
];

const QUALITY_CHOICES: [(GraphicsQuality, &str); 6] = [
    (GraphicsQuality::Default, "Default"),
    (GraphicsQuality::Toon1280, "Toon1280"),
    (GraphicsQuality::Toon1280x2, "Toon1280x2"),
    (GraphicsQuality::Toon1280x4, "Toon1280x4"),
    (GraphicsQuality::ToonFull, "ToonFull"),
    (GraphicsQuality::Max, "Max"),
];

#[cfg(target_os = "windows")]
const REFRESH_CHOICES: [(i32, &str); 6] = [
    (0, "Auto"),
    (60, "60 Hz"),
    (120, "120 Hz"),
    (144, "144 Hz"),
    (165, "165 Hz"),
    (240, "240 Hz"),
];

#[cfg(target_os = "windows")]
const RES_PRESETS: [(i32, i32); 11] = [
    (1280, 720),
    (1600, 900),
    (1920, 1080),
    (2560, 1440),
    (3840, 2160),
    (2560, 1080),
    (3440, 1440),
    (3840, 1600),
    (720, 1280),
    (1080, 1920),
    (1440, 2560),
];
