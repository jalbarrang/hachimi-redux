//! Separator — a 1px divider line. Fills the width of a stretch container
//! (e.g. inside a [`crate::Card`]).

use dioxus_egui::dioxus::prelude::*;

use crate::theme;

#[component]
pub fn Separator() -> Element {
    let bg = theme::LINE;
    rsx! {
        div { "height": "1", "bg": bg }
    }
}
