//! Image — a fixed-size image slot. Forwards a URI (`src`) to the dioxus-egui
//! renderer's `<img>` element, which resolves it through egui's image loaders.
//! For embedded assets, register the bytes once with
//! `egui::Context::include_bytes("bytes://…", …)` and pass that URI as `src`.

use dioxus_egui::dioxus::prelude::*;

/// A fixed-size image. `src` is any egui-resolvable URI
/// (`bytes://…`, `file://…`, `https://…`).
#[component]
pub fn Image(src: String, width: f32, height: f32) -> Element {
    rsx! {
        img { src: "{src}", width: "{width}", height: "{height}" }
    }
}
