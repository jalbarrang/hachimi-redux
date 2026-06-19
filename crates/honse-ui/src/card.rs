//! Card — a raised surface with a border, plus the text-styling helpers
//! `CardTitle`/`CardDescription`. Compose them like shadcn:
//!
//! ```ignore
//! Card {
//!     CardTitle { "Trained Speed" }
//!     CardDescription { "since last race" }
//!     // ...content
//! }
//! ```

use dioxus_egui::dioxus::prelude::*;

use crate::theme;

/// A bordered, padded surface that stacks its children in a column.
#[component]
pub fn Card(children: Element) -> Element {
    let bg = theme::SURFACE_1;
    let border = theme::LINE;
    let radius = theme::RADIUS;
    rsx! {
        div {
            "dir": "col",
            "gap": "10",
            "align": "stretch",
            "padding": "14",
            "bg": bg,
            "border": border,
            "radius": radius,
            {children}
        }
    }
}

/// Prominent card heading.
#[component]
pub fn CardTitle(children: Element) -> Element {
    let color = theme::FG;
    rsx! {
        div { "color": color, "font-size": "18", "weight": "bold", {children} }
    }
}

/// Muted secondary text under a [`CardTitle`].
#[component]
pub fn CardDescription(children: Element) -> Element {
    let color = theme::FG_MUTED;
    rsx! {
        div { "color": color, "font-size": "13", {children} }
    }
}
