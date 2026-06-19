//! Tab bar — horizontal pill row.

use dioxus_egui::dioxus::prelude::*;

use crate::{Button, ButtonVariant};

#[derive(Clone, PartialEq)]
pub struct TabItem {
    pub id: String,
    pub label: String,
}

#[component]
pub fn TabBar(active: String, tabs: Vec<TabItem>, onselect: EventHandler<String>) -> Element {
    rsx! {
        div {
            "scroll": "x",
            "dir": "row",
            "gap": "6",
            for tab in tabs {
                Button {
                    variant: if active == tab.id { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                    onclick: {
                        let id = tab.id.clone();
                        move |_| onselect.call(id.clone())
                    },
                    {tab.label.clone()}
                }
            }
        }
    }
}
