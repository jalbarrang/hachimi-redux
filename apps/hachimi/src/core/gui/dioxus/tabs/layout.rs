//! Dioxus layout helpers mirroring the egui_taffy settings grid.

use dioxus_egui::dioxus::prelude::*;
use honse_ui::theme;

#[component]
pub fn SettingsGrid(children: Element) -> Element {
    rsx! {
        div {
            "display": "grid",
            "grid-cols": "label-control",
            "gap": "8",
            "align": "start",
            {children}
        }
    }
}

#[component]
pub fn LabelCell(text: String) -> Element {
    rsx! {
        div {
            "color": theme::FG_MUTED,
            "font-size": "14",
            {text}
        }
    }
}

#[component]
pub fn SectionHeader(text: String) -> Element {
    rsx! {
        div {
            "color": theme::ACCENT,
            "font-size": "15",
            "weight": "bold",
            "padding": "8",
            {text}
        }
    }
}
