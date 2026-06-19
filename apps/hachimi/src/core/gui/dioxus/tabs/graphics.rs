//! Graphics tab — rendering and display options.

use dioxus_egui::dioxus::prelude::*;
use honse_ui::{Combo, ComboOption, SliderRow};
use rust_i18n::t;

use crate::il2cpp::hook::{
    umamusume::{
        CameraData::ShadowResolution,
        GraphicSettings::{GraphicsQuality, MsaaQuality},
    },
    UnityEngine_CoreModule::Texture::AnisoLevel,
};

use super::super::context::ControlCenterCtx;
use super::layout::{LabelCell, SettingsGrid};

#[component]
pub fn GraphicsTab() -> Element {
    let ctx = use_context::<ControlCenterCtx>();
    let _ = ctx.revision.read();
    let cfg = ctx.config.borrow().clone();

    let label_target_fps = t!("config_editor.target_fps").to_string();
    let label_virtual_resolution_multiplier = t!("config_editor.virtual_resolution_multiplier").to_string();
    let label_ui_scale = t!("config_editor.ui_scale").to_string();
    let label_ui_animation_scale = t!("config_editor.ui_animation_scale").to_string();
    let label_loading_fade_scale = t!("config_editor.loading_fade_scale").to_string();
    let label_flash_animation_scale = t!("config_editor.flash_animation_scale").to_string();
    let label_render_scale = t!("config_editor.render_scale").to_string();
    let label_msaa = t!("config_editor.msaa").to_string();
    let label_aniso_level = t!("config_editor.aniso_level").to_string();
    let label_shadow_resolution = t!("config_editor.shadow_resolution").to_string();
    let label_graphics_quality = t!("config_editor.graphics_quality").to_string();

    rsx! {
        SettingsGrid {
            LabelCell { text: label_target_fps }
            OptionalSlider {
                label: String::new(),
                enabled: cfg.target_fps.is_some(),
                value: cfg.target_fps.unwrap_or(30) as f64,
                min: 30.0,
                max: 1000.0,
                step: 1.0,
                on_toggle: ctx.bind(|c, on| {
                    c.target_fps = if on { Some(30) } else { None };
                }),
                oninput: ctx.bind(|c, v| c.target_fps = Some(v as i32)),
            }

            LabelCell { text: label_virtual_resolution_multiplier }
            SliderRow {
                label: String::new(),
                value: cfg.virtual_res_mult as f64,
                min: 1.0,
                max: 4.0,
                step: 0.1,
                oninput: ctx.bind(|c, v| c.virtual_res_mult = v as f32),
            }

            LabelCell { text: label_ui_scale }
            SliderRow {
                label: String::new(),
                value: cfg.ui_scale as f64,
                min: 0.1,
                max: 10.0,
                step: 0.05,
                oninput: ctx.bind(|c, v| c.ui_scale = v as f32),
            }

            LabelCell { text: label_ui_animation_scale }
            SliderRow {
                label: String::new(),
                value: cfg.ui_animation_scale as f64,
                min: 0.1,
                max: 10.0,
                step: 0.1,
                oninput: ctx.bind(|c, v| c.ui_animation_scale = v as f32),
            }

            LabelCell { text: label_loading_fade_scale }
            SliderRow {
                label: String::new(),
                value: cfg.loading_fade_scale as f64,
                min: 0.1,
                max: 10.0,
                step: 0.1,
                oninput: ctx.bind(|c, v| c.loading_fade_scale = v as f32),
            }

            LabelCell { text: label_flash_animation_scale }
            SliderRow {
                label: String::new(),
                value: cfg.flash_animation_scale as f64,
                min: 0.1,
                max: 10.0,
                step: 0.1,
                oninput: ctx.bind(|c, v| c.flash_animation_scale = v as f32),
            }

            LabelCell { text: label_render_scale }
            SliderRow {
                label: String::new(),
                value: cfg.render_scale as f64,
                min: 0.1,
                max: 10.0,
                step: 0.1,
                oninput: ctx.bind(|c, v| c.render_scale = v as f32),
            }

            LabelCell { text: label_msaa }
            EnumCombo {
                value: msaa_key(cfg.msaa),
                options: msaa_options(),
                onchange: ctx.bind(|c, k: String| c.msaa = parse_msaa(&k)),
            }

            LabelCell { text: label_aniso_level }
            EnumCombo {
                value: aniso_key(cfg.aniso_level),
                options: aniso_options(),
                onchange: ctx.bind(|c, k: String| c.aniso_level = parse_aniso(&k)),
            }

            LabelCell { text: label_shadow_resolution }
            EnumCombo {
                value: shadow_key(cfg.shadow_resolution),
                options: shadow_options(),
                onchange: ctx.bind(|c, k: String| c.shadow_resolution = parse_shadow(&k)),
            }

            LabelCell { text: label_graphics_quality }
            EnumCombo {
                value: quality_key(cfg.graphics_quality),
                options: quality_options(),
                onchange: ctx.bind(|c, k: String| c.graphics_quality = parse_quality(&k)),
            }

            WindowsGraphicsOptions {}
        }
    }
}

#[component]
fn OptionalSlider(
    label: String,
    enabled: bool,
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    on_toggle: EventHandler<bool>,
    oninput: EventHandler<f64>,
) -> Element {
    let enable_label = t!("enable").to_string();

    rsx! {
        div {
            "dir": "col",
            "gap": "4",
            honse_ui::Toggle {
                label: enable_label,
                checked: enabled,
                onchange: move |v| on_toggle.call(v),
            }
            if enabled {
                SliderRow {
                    label,
                    value,
                    min,
                    max,
                    step,
                    oninput: move |v| oninput.call(v),
                }
            }
        }
    }
}

#[component]
fn EnumCombo(value: String, options: Vec<ComboOption>, onchange: EventHandler<String>) -> Element {
    rsx! {
        Combo { value, options, onchange: move |v| onchange.call(v) }
    }
}

#[cfg(target_os = "windows")]
#[component]
fn WindowsGraphicsOptions() -> Element {
    use crate::windows::hachimi_impl::{FullScreenMode, ResolutionScaling};

    let ctx = use_context::<ControlCenterCtx>();
    let _ = ctx.revision.read();
    let cfg = ctx.config.borrow().clone();

    let label_vsync = t!("config_editor.vsync").to_string();
    let label_auto_full_screen = t!("config_editor.auto_full_screen").to_string();
    let label_full_screen_mode = t!("config_editor.full_screen_mode").to_string();
    let fullscreen_options = vec![
        ComboOption {
            value: "exclusive".into(),
            label: t!("config_editor.full_screen_mode_exclusive").to_string(),
        },
        ComboOption {
            value: "borderless".into(),
            label: t!("config_editor.full_screen_mode_borderless").to_string(),
        },
    ];
    let label_block_minimize = t!("config_editor.block_minimize_in_full_screen").to_string();
    let label_resolution_scaling = t!("config_editor.resolution_scaling").to_string();
    let scaling_options = vec![
        ComboOption {
            value: "default".into(),
            label: t!("config_editor.resolution_scaling_default").to_string(),
        },
        ComboOption {
            value: "screen".into(),
            label: t!("config_editor.resolution_scaling_ssize").to_string(),
        },
        ComboOption {
            value: "window".into(),
            label: t!("config_editor.resolution_scaling_wsize").to_string(),
        },
    ];
    let label_window_always_on_top = t!("config_editor.window_always_on_top").to_string();

    rsx! {
        LabelCell { text: label_vsync }
        div { "native": "egui" }

        LabelCell { text: label_auto_full_screen }
        honse_ui::Toggle {
            label: String::new(),
            checked: cfg.windows.auto_full_screen,
            onchange: ctx.bind(|c, v| c.windows.auto_full_screen = v),
        }

        LabelCell { text: label_full_screen_mode }
        EnumCombo {
            value: fullscreen_key(cfg.windows.full_screen_mode),
            options: fullscreen_options,
            onchange: ctx.bind(|c, k: String| {
                c.windows.full_screen_mode = match k.as_str() {
                    "borderless" => FullScreenMode::FullScreenWindow,
                    _ => FullScreenMode::ExclusiveFullScreen,
                };
            }),
        }

        LabelCell { text: label_block_minimize }
        honse_ui::Toggle {
            label: String::new(),
            checked: cfg.windows.block_minimize_in_full_screen,
            onchange: ctx.bind(|c, v| c.windows.block_minimize_in_full_screen = v),
        }

        LabelCell { text: label_resolution_scaling }
        EnumCombo {
            value: scaling_key(cfg.windows.resolution_scaling),
            options: scaling_options,
            onchange: ctx.bind(|c, k: String| {
                c.windows.resolution_scaling = match k.as_str() {
                    "screen" => ResolutionScaling::ScaleToScreenSize,
                    "window" => ResolutionScaling::ScaleToWindowSize,
                    _ => ResolutionScaling::Default,
                };
            }),
        }

        LabelCell { text: label_window_always_on_top }
        honse_ui::Toggle {
            label: String::new(),
            checked: cfg.windows.window_always_on_top,
            onchange: ctx.bind(|c, v| c.windows.window_always_on_top = v),
        }
    }
}

#[cfg(not(target_os = "windows"))]
#[component]
fn WindowsGraphicsOptions() -> Element {
    rsx! {}
}

fn msaa_options() -> Vec<ComboOption> {
    vec![
        ComboOption {
            value: "default".into(),
            label: t!("default").to_string(),
        },
        ComboOption {
            value: "2x".into(),
            label: "2x".into(),
        },
        ComboOption {
            value: "4x".into(),
            label: "4x".into(),
        },
        ComboOption {
            value: "8x".into(),
            label: "8x".into(),
        },
    ]
}

fn msaa_key(v: MsaaQuality) -> String {
    match v {
        MsaaQuality::Disabled => "default",
        MsaaQuality::_2x => "2x",
        MsaaQuality::_4x => "4x",
        MsaaQuality::_8x => "8x",
    }
    .into()
}

fn parse_msaa(k: &str) -> MsaaQuality {
    match k {
        "2x" => MsaaQuality::_2x,
        "4x" => MsaaQuality::_4x,
        "8x" => MsaaQuality::_8x,
        _ => MsaaQuality::Disabled,
    }
}

fn aniso_options() -> Vec<ComboOption> {
    vec![
        ComboOption {
            value: "default".into(),
            label: t!("default").to_string(),
        },
        ComboOption {
            value: "2x".into(),
            label: "2x".into(),
        },
        ComboOption {
            value: "4x".into(),
            label: "4x".into(),
        },
        ComboOption {
            value: "8x".into(),
            label: "8x".into(),
        },
        ComboOption {
            value: "16x".into(),
            label: "16x".into(),
        },
    ]
}

fn aniso_key(v: AnisoLevel) -> String {
    match v {
        AnisoLevel::Default => "default",
        AnisoLevel::_2x => "2x",
        AnisoLevel::_4x => "4x",
        AnisoLevel::_8x => "8x",
        AnisoLevel::_16x => "16x",
    }
    .into()
}

fn parse_aniso(k: &str) -> AnisoLevel {
    match k {
        "2x" => AnisoLevel::_2x,
        "4x" => AnisoLevel::_4x,
        "8x" => AnisoLevel::_8x,
        "16x" => AnisoLevel::_16x,
        _ => AnisoLevel::Default,
    }
}

fn shadow_options() -> Vec<ComboOption> {
    vec![
        ComboOption {
            value: "default".into(),
            label: t!("default").to_string(),
        },
        ComboOption {
            value: "256".into(),
            label: "256x".into(),
        },
        ComboOption {
            value: "512".into(),
            label: "512x".into(),
        },
        ComboOption {
            value: "1024".into(),
            label: "1K".into(),
        },
        ComboOption {
            value: "2048".into(),
            label: "2K".into(),
        },
        ComboOption {
            value: "4096".into(),
            label: "4K".into(),
        },
    ]
}

fn shadow_key(v: ShadowResolution) -> String {
    match v {
        ShadowResolution::Default => "default",
        ShadowResolution::_256 => "256",
        ShadowResolution::_512 => "512",
        ShadowResolution::_1024 => "1024",
        ShadowResolution::_2048 => "2048",
        ShadowResolution::_4096 => "4096",
    }
    .into()
}

fn parse_shadow(k: &str) -> ShadowResolution {
    match k {
        "256" => ShadowResolution::_256,
        "512" => ShadowResolution::_512,
        "1024" => ShadowResolution::_1024,
        "2048" => ShadowResolution::_2048,
        "4096" => ShadowResolution::_4096,
        _ => ShadowResolution::Default,
    }
}

fn quality_options() -> Vec<ComboOption> {
    vec![
        ComboOption {
            value: "default".into(),
            label: t!("default").to_string(),
        },
        ComboOption {
            value: "toon1280".into(),
            label: "Toon1280".into(),
        },
        ComboOption {
            value: "toon1280x2".into(),
            label: "Toon1280x2".into(),
        },
        ComboOption {
            value: "toon1280x4".into(),
            label: "Toon1280x4".into(),
        },
        ComboOption {
            value: "toonfull".into(),
            label: "ToonFull".into(),
        },
        ComboOption {
            value: "max".into(),
            label: "Max".into(),
        },
    ]
}

fn quality_key(v: GraphicsQuality) -> String {
    match v {
        GraphicsQuality::Default => "default",
        GraphicsQuality::Toon1280 => "toon1280",
        GraphicsQuality::Toon1280x2 => "toon1280x2",
        GraphicsQuality::Toon1280x4 => "toon1280x4",
        GraphicsQuality::ToonFull => "toonfull",
        GraphicsQuality::Max => "max",
    }
    .into()
}

fn parse_quality(k: &str) -> GraphicsQuality {
    match k {
        "toon1280" => GraphicsQuality::Toon1280,
        "toon1280x2" => GraphicsQuality::Toon1280x2,
        "toon1280x4" => GraphicsQuality::Toon1280x4,
        "toonfull" => GraphicsQuality::ToonFull,
        "max" => GraphicsQuality::Max,
        _ => GraphicsQuality::Default,
    }
}

#[cfg(target_os = "windows")]
fn fullscreen_key(v: crate::windows::hachimi_impl::FullScreenMode) -> String {
    match v {
        crate::windows::hachimi_impl::FullScreenMode::FullScreenWindow => "borderless",
        _ => "exclusive",
    }
    .into()
}

#[cfg(target_os = "windows")]
fn scaling_key(v: crate::windows::hachimi_impl::ResolutionScaling) -> String {
    match v {
        crate::windows::hachimi_impl::ResolutionScaling::ScaleToScreenSize => "screen",
        crate::windows::hachimi_impl::ResolutionScaling::ScaleToWindowSize => "window",
        _ => "default",
    }
    .into()
}
