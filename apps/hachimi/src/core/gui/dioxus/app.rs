//! Control Center Dioxus root component.

use dioxus_egui::dioxus::prelude::*;
use honse_ui::{theme, Button, ButtonVariant, Image, TabBar, TabItem};
use rust_i18n::t;

use super::context::{bind_action, ControlCenterCtx, HostAction};
use super::tabs::{
    about::AboutTab, gameplay::GameplayTab, general::GeneralTab, graphics::GraphicsTab, hotkeys::HotkeysTab,
    plugins::PluginsTab, translations::TranslationsTab,
};
use crate::core::gui::shell::{ControlTab, SHELL_WIDTH};

pub fn control_center_app() -> Element {
    rsx! { ControlCenterShell {} }
}

#[component]
fn ControlCenterShell() -> Element {
    let ctx = use_context::<ControlCenterCtx>();
    let actions = ctx.actions_rc();
    let mut active_tab = ctx.active_tab;
    let tab = (ctx.active_tab)();
    let scale = (ctx.scale)();
    let shell_w = SHELL_WIDTH * scale;
    let shell_h = (ctx.height)();
    let footer_on = tab.edits_config();

    let tabs = vec![
        TabItem {
            id: "general".into(),
            label: t!("config_editor.general_tab").to_string(),
        },
        TabItem {
            id: "graphics".into(),
            label: t!("config_editor.graphics_tab").to_string(),
        },
        TabItem {
            id: "gameplay".into(),
            label: t!("config_editor.gameplay_tab").to_string(),
        },
        TabItem {
            id: "hotkeys".into(),
            label: t!("config_editor.hotkeys_tab").to_string(),
        },
        TabItem {
            id: "translations".into(),
            label: "\u{f1ab} Translations".to_string(),
        },
        TabItem {
            id: "plugins".into(),
            label: "\u{f12e} Plugins".to_string(),
        },
        TabItem {
            id: "about".into(),
            label: "\u{f129} About".to_string(),
        },
    ];

    let active_id = tab_id(tab);

    let hachimi_title = t!("hachimi").to_string();
    let cancel_label = t!("config_editor.cancel").to_string();
    let save_label = t!("save").to_string();

    rsx! {
        div {
            "dir": "col",
            "align": "stretch",
            "width": "{shell_w}",
            "height": "{shell_h}",
            "bg": theme::SURFACE_1,
            "border": theme::LINE,
            "radius": theme::RADIUS,

            div {
                "dir": "col",
                "gap": "8",
                "align": "stretch",
                "padding": "14",
                div {
                    "dir": "row",
                    "gap": "8",
                    "align": "center",
                    Image {
                        src: crate::core::gui::splash::ICON_URI.to_string(),
                        width: 24.0,
                        height: 24.0,
                    }
                    div {
                        "color": theme::FG,
                        "font-size": "18",
                        "weight": "bold",
                        {hachimi_title}
                    }
                    div {
                        "color": theme::FG_MUTED,
                        "font-size": "12",
                        {env!("HACHIMI_DISPLAY_VERSION")}
                    }
                    div { "grow": "1" }
                    Button {
                        variant: ButtonVariant::Ghost,
                        onclick: bind_action(&actions, HostAction::CloseMenu),
                        "\u{f00d}"
                    }
                }
                TabBar {
                    active: active_id,
                    tabs,
                    onselect: move |id: String| {
                        active_tab.set(parse_tab(&id));
                    },
                }
            }

            honse_ui::ScrollArea {
                div {
                    "dir": "col",
                    "gap": "8",
                    "align": "stretch",
                    "padding": "14",
                    match tab {
                        ControlTab::General => rsx! { GeneralTab {} },
                        ControlTab::Graphics => rsx! { GraphicsTab {} },
                        ControlTab::Gameplay => rsx! { GameplayTab {} },
                        ControlTab::Hotkeys => rsx! { HotkeysTab {} },
                        ControlTab::Translations => rsx! { TranslationsTab {} },
                        ControlTab::Plugins => rsx! { PluginsTab {} },
                        ControlTab::About => rsx! { AboutTab {} },
                    }
                }
            }

            if footer_on {
                div {
                    "dir": "row",
                    "gap": "8",
                    "justify": "end",
                    "padding": "14",
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: bind_action(&actions, HostAction::RevertConfig),
                        {cancel_label}
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: bind_action(&actions, HostAction::SaveConfig),
                        {save_label}
                    }
                }
            }
        }
    }
}

fn tab_id(tab: ControlTab) -> String {
    match tab {
        ControlTab::General => "general",
        ControlTab::Graphics => "graphics",
        ControlTab::Gameplay => "gameplay",
        ControlTab::Hotkeys => "hotkeys",
        ControlTab::Translations => "translations",
        ControlTab::Plugins => "plugins",
        ControlTab::About => "about",
    }
    .into()
}

fn parse_tab(id: &str) -> ControlTab {
    match id {
        "graphics" => ControlTab::Graphics,
        "gameplay" => ControlTab::Gameplay,
        "hotkeys" => ControlTab::Hotkeys,
        "translations" => ControlTab::Translations,
        "plugins" => ControlTab::Plugins,
        "about" => ControlTab::About,
        _ => ControlTab::General,
    }
}
