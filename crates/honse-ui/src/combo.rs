//! Combo box — `<select>` wrapper with typed options.

use dioxus_egui::dioxus::prelude::*;

use crate::theme;

#[derive(Clone, PartialEq)]
pub struct ComboOption {
    pub value: String,
    pub label: String,
}

#[component]
pub fn Combo(value: String, options: Vec<ComboOption>, onchange: EventHandler<String>) -> Element {
    rsx! {
        select {
            "value": value,
            "bg": theme::SURFACE_2,
            "border": theme::LINE,
            "radius": theme::RADIUS_SM,
            "padding": "6",
            onchange: move |e| onchange.call(e.value()),
            for opt in options {
                option {
                    "value": opt.value.clone(),
                    {opt.label.clone()}
                }
            }
        }
    }
}
