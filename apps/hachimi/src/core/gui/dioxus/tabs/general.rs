//! General tab — core config + overlay controls.

use dioxus_egui::dioxus::prelude::*;
use honse_ui::{Button, ButtonVariant, Combo, ComboOption, SliderRow, Toggle};
use rust_i18n::t;

use crate::core::hachimi::Language;
use crate::core::plugin::overlay;

use super::super::context::{bind_action, ControlCenterCtx, HostAction};
use super::layout::{LabelCell, SectionHeader, SettingsGrid};

#[component]
pub fn GeneralTab() -> Element {
    let ctx = use_context::<ControlCenterCtx>();
    let _rev = ctx.revision.read();
    let actions = ctx.actions_rc();
    let cfg = ctx.config.borrow().clone();

    let lang_options: Vec<ComboOption> = Language::CHOICES
        .iter()
        .map(|(lang, name)| ComboOption {
            value: lang.locale_str().to_string(),
            label: (*name).to_string(),
        })
        .collect();
    let lang_value = cfg.language.locale_str().to_string();

    let label_language = t!("config_editor.language").to_string();
    let label_disable_overlay = t!("config_editor.disable_overlay").to_string();
    let label_ipv4_only = t!("config_editor.ipv4_only").to_string();
    let label_gui_scale = t!("config_editor.gui_scale").to_string();
    let label_theme_editor = t!("theme_editor.title").to_string();
    let open_label = t!("open").to_string();
    let label_debug_mode = t!("config_editor.debug_mode").to_string();
    let label_enable_file_logging = t!("config_editor.enable_file_logging").to_string();
    let label_apply_atlas_workaround = t!("config_editor.apply_atlas_workaround").to_string();
    let label_skip_first_time_setup = t!("config_editor.skip_first_time_setup").to_string();
    let label_disable_auto_update_check = t!("config_editor.disable_auto_update_check").to_string();
    let label_enable_ipc = t!("config_editor.enable_ipc").to_string();
    let label_ipc_listen_all = t!("config_editor.ipc_listen_all").to_string();
    let overlays_heading = t!("config_editor.overlays_heading").to_string();

    let on_disable_overlay = {
        let ctx = ctx.clone();
        move |on: bool| {
            ctx.config.borrow_mut().disable_gui = on;
            if on {
                ctx.actions.borrow_mut().push(HostAction::OpenDisableOverlayWarning);
            }
            ctx.bump_revision();
        }
    };

    rsx! {
        SettingsGrid {
            LabelCell { text: label_language }
            Combo {
                value: lang_value,
                options: lang_options,
                onchange: ctx.bind(|c, v: String| {
                    if let Some((lang, _)) = Language::CHOICES.iter().find(|(l, _)| l.locale_str() == v) {
                        c.language = *lang;
                        c.language.set_locale();
                    }
                }),
            }

            LabelCell { text: label_disable_overlay }
            Toggle {
                label: String::new(),
                checked: cfg.disable_gui,
                onchange: on_disable_overlay,
            }

            LabelCell { text: label_ipv4_only }
            Toggle {
                label: String::new(),
                checked: cfg.ipv4_only,
                onchange: ctx.bind(|c, v| c.ipv4_only = v),
            }

            LabelCell { text: label_gui_scale }
            SliderRow {
                label: String::new(),
                value: cfg.gui_scale as f64,
                min: 0.25,
                max: 2.0,
                step: 0.05,
                oninput: ctx.bind(|c, v| c.gui_scale = v as f32),
            }

            WindowsGeneralFields {}

            LabelCell { text: label_theme_editor }
            Button {
                variant: ButtonVariant::Secondary,
                onclick: bind_action(&actions, HostAction::OpenThemeEditor),
                {open_label}
            }

            LabelCell { text: label_debug_mode }
            Toggle {
                label: String::new(),
                checked: cfg.debug_mode,
                onchange: ctx.bind(|c, v| c.debug_mode = v),
            }

            LabelCell { text: label_enable_file_logging }
            Toggle {
                label: String::new(),
                checked: cfg.enable_file_logging,
                onchange: ctx.bind(|c, v| c.enable_file_logging = v),
            }

            LabelCell { text: label_apply_atlas_workaround }
            Toggle {
                label: String::new(),
                checked: cfg.apply_atlas_workaround,
                onchange: ctx.bind(|c, v| c.apply_atlas_workaround = v),
            }

            LabelCell { text: label_skip_first_time_setup }
            Toggle {
                label: String::new(),
                checked: cfg.skip_first_time_setup,
                onchange: ctx.bind(|c, v| c.skip_first_time_setup = v),
            }

            LabelCell { text: label_disable_auto_update_check }
            Toggle {
                label: String::new(),
                checked: cfg.disable_auto_update_check,
                onchange: ctx.bind(|c, v| c.disable_auto_update_check = v),
            }

            LabelCell { text: label_enable_ipc }
            Toggle {
                label: String::new(),
                checked: cfg.enable_ipc,
                onchange: ctx.bind(|c, v| c.enable_ipc = v),
            }

            LabelCell { text: label_ipc_listen_all }
            Toggle {
                label: String::new(),
                checked: cfg.ipc_listen_all,
                onchange: ctx.bind(|c, v| c.ipc_listen_all = v),
            }
        }

        SectionHeader { text: overlays_heading }
        OverlaysPanel {}
    }
}

#[cfg(target_os = "windows")]
#[component]
fn WindowsGeneralFields() -> Element {
    let ctx = use_context::<ControlCenterCtx>();
    let _rev = ctx.revision.read();
    let ratio = ctx.config.borrow().windows.gui_landscape_ratio;
    let discord = ctx.config.borrow().windows.discord_rpc;
    let label_ratio = t!("config_editor.gui_landscape_ratio").to_string();
    let label_discord = t!("config_editor.discord_rpc").to_string();

    rsx! {
        LabelCell { text: label_ratio }
        SliderRow {
            label: String::new(),
            value: ratio as f64,
            min: 0.25,
            max: 1.0,
            step: 0.05,
            oninput: ctx.bind(|c, v| c.windows.gui_landscape_ratio = v as f32),
        }
        LabelCell { text: label_discord }
        Toggle {
            label: String::new(),
            checked: discord,
            onchange: ctx.bind(|c, v| c.windows.discord_rpc = v),
        }
    }
}

#[cfg(not(target_os = "windows"))]
#[component]
fn WindowsGeneralFields() -> Element {
    rsx! {}
}

#[component]
fn OverlaysPanel() -> Element {
    let ctx = use_context::<ControlCenterCtx>();
    let _rev = ctx.revision.read();
    let opacity = overlay::opacity();
    let overlays = overlay::get_plugin_overlays();
    let overlay_opacity_label = t!("config_editor.overlay_opacity").to_string();
    let overlays_none = t!("config_editor.overlays_none").to_string();
    let overlay_reset = t!("config_editor.overlay_reset").to_string();

    let bump = ctx.clone();
    rsx! {
        div {
            "dir": "col",
            "gap": "8",
            "padding": "8",
            "align": "stretch",
            SliderRow {
                label: overlay_opacity_label,
                value: opacity as f64,
                min: 0.1,
                max: 1.0,
                step: 0.05,
                oninput: move |v| {
                    overlay::set_opacity(v as f32);
                    bump.bump_revision();
                },
            }
            if overlays.is_empty() {
                div {
                    "color": honse_ui::theme::FG_DIM,
                    {overlays_none}
                }
            }
            for ov in overlays {
                {
                    let ctx_toggle = ctx.clone();
                    let ctx_reset = ctx.clone();
                    let ov_id_toggle = ov.id.clone();
                    let ov_id_reset = ov.id;
                    let reset_label = overlay_reset.clone();
                    rsx! {
                        div {
                            "dir": "row",
                            "gap": "8",
                            "align": "center",
                            Toggle {
                                label: overlay::display_title(&ov_id_toggle),
                                checked: overlay::is_overlay_visible(&ov_id_toggle),
                                onchange: move |v| {
                                    overlay::set_overlay_visible(&ov_id_toggle, v);
                                    ctx_toggle.bump_revision();
                                },
                            }
                            Button {
                                variant: ButtonVariant::Ghost,
                                onclick: move |_| {
                                    overlay::reset_panel(&ov_id_reset);
                                    ctx_reset.bump_revision();
                                },
                                {reset_label}
                            }
                        }
                    }
                }
            }
        }
    }
}
