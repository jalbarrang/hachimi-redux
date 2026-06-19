//! Toggle switch — checkbox styled as a labeled row.

use dioxus_egui::dioxus::prelude::*;

use crate::theme;

#[component]
pub fn Toggle(label: String, checked: bool, onchange: EventHandler<bool>) -> Element {
    rsx! {
        div {
            "dir": "row",
            "gap": "8",
            "align": "center",
            input {
                r#type: "checkbox",
                checked: checked,
                onchange: move |e| onchange.call(e.checked()),
            }
            div {
                "color": theme::FG,
                "font-size": "14",
                {label}
            }
        }
    }
}
