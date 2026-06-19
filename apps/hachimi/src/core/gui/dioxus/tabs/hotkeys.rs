//! Hotkeys tab — native egui slot (capture UI).

use dioxus_egui::dioxus::prelude::*;

#[component]
pub fn HotkeysTab() -> Element {
    rsx! {
        div { "native": "egui" }
    }
}
