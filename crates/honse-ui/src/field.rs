//! Field — a labeled form row: a muted caption above its control. Wrap any of
//! the renderer's value widgets (`input` checkbox/range/text):
//!
//! ```ignore
//! Field { label: "Volume".to_string(),
//!     input { r#type: "range", value: "{vol}", min: "0", max: "100",
//!         oninput: move |e| { /* ... */ } }
//! }
//! ```

use dioxus_egui::dioxus::prelude::*;

use crate::theme;

#[component]
pub fn Field(label: String, children: Element) -> Element {
    let color = theme::FG_MUTED;
    rsx! {
        div { "dir": "col", "gap": "4", "align": "stretch",
            div { "color": color, "font-size": "13", "weight": "bold", "{label}" }
            {children}
        }
    }
}
