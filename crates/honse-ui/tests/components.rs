//! Headless checks: honse-ui components render through the dioxus-egui renderer
//! and themed buttons still register clickable listeners (so `onclick` works).

#![allow(clippy::disallowed_methods)] // dioxus rsx! macro uses unwrap internally

use dioxus_egui::dioxus::dioxus_core::VirtualDom;
use dioxus_egui::dioxus::prelude::*;
use dioxus_egui::DioxusEgui;
use honse_ui::{Badge, BadgeVariant, Button, ButtonVariant, Card, CardTitle, Field, Image};

fn demo() -> Element {
    rsx! {
        Card {
            CardTitle { "Career" }
            Button { variant: ButtonVariant::Primary, onclick: move |_| {}, "Save" }
            Button { variant: ButtonVariant::Ghost, onclick: move |_| {}, "Cancel" }
            Badge { variant: BadgeVariant::Accent, "S Rank" }
            Field { label: "Name".to_string(),
                input { value: "Special Week", oninput: move |_| {} }
            }
        }
    }
}

#[test]
fn components_render_and_buttons_are_clickable() {
    let mut vdom = VirtualDom::new(demo);
    let mut r = DioxusEgui::new();
    vdom.rebuild(&mut r);

    let dump = r.dump();
    assert!(dump.contains("Career"), "title text present: {dump}");
    assert!(dump.contains("S Rank"), "badge text present: {dump}");
    assert!(
        dump.contains("[input type=text value=Special Week]"),
        "field input rendered: {dump}"
    );

    // Both themed buttons (solid Primary + transparent Ghost) registered an
    // onclick listener — so they have an ElementId and are deliverable.
    let labels: Vec<String> = r.buttons().into_iter().map(|(_, l)| l).collect();
    assert!(labels.contains(&"Save".to_string()), "buttons: {labels:?}");
    assert!(labels.contains(&"Cancel".to_string()), "buttons: {labels:?}");
}

#[test]
fn image_renders_with_src() {
    fn demo() -> Element {
        rsx! {
            Image { src: "bytes://test.png".to_string(), width: 24.0, height: 24.0 }
        }
    }

    let mut vdom = VirtualDom::new(demo);
    let mut r = DioxusEgui::new();
    vdom.rebuild(&mut r);

    let dump = r.dump();
    assert!(
        dump.contains("[img src=bytes://test.png]"),
        "image element present: {dump}"
    );
}
