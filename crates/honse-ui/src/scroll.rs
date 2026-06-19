//! Vertical scroll region.

use dioxus_egui::dioxus::prelude::*;

#[component]
pub fn ScrollArea(children: Element) -> Element {
    rsx! {
        div {
            "scroll": "y",
            "grow": "1",
            "min-height": "0",
            "align": "stretch",
            {children}
        }
    }
}
