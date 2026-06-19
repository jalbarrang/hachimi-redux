//! Modal / floating-panel chrome (title bar + body slot).

use crate::theme;
use dioxus_egui::dioxus::prelude::*;

/// Titled panel chrome for plugin overlays and dialogs.
#[component]
pub fn WindowChrome(title: String, #[props(extends = div)] attrs: Vec<Attribute>, children: Element) -> Element {
    rsx! {
        div {
            "dir": "col",
            "gap": "4",
            "bg": theme::SURFACE_1,
            "border": theme::LINE,
            "radius": "8",
            "padding": "8",
            ..attrs,
            div {
                "dir": "row",
                "align": "center",
                "padding": "4",
                div {
                    "color": theme::FG,
                    "weight": "bold",
                    {title}
                }
            }
            div {
                "dir": "col",
                "gap": "4",
                {children}
            }
        }
    }
}
