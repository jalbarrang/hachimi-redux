//! Labeled slider row.

use dioxus_egui::dioxus::prelude::*;

use crate::theme;

#[component]
pub fn SliderRow(label: String, value: f64, min: f64, max: f64, step: f64, oninput: EventHandler<f64>) -> Element {
    rsx! {
        div {
            "dir": "row",
            "gap": "8",
            "align": "center",
            div {
                "color": theme::FG_MUTED,
                "font-size": "14",
                "width": "140",
                {label}
            }
            input {
                r#type: "range",
                value: "{value}",
                min: "{min}",
                max: "{max}",
                step: "{step}",
                oninput: move |e| {
                    if let Ok(v) = e.value().parse::<f64>() {
                        oninput.call(v);
                    }
                },
            }
            div {
                "color": theme::FG,
                "font-size": "13",
                "{value:.2}"
            }
        }
    }
}
