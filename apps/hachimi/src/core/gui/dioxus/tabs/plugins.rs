//! Plugins tab — native egui slot (plugin page registry).

use dioxus_egui::dioxus::prelude::*;

#[component]
pub fn PluginsTab() -> Element {
    rsx! {
        div { "native": "egui" }
    }
}
