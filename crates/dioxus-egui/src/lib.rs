//! Render a [Dioxus](https://dioxuslabs.com) `VirtualDom` to egui, laid out with
//! [`egui_taffy`].
//!
//! Dioxus is immediate-mode-agnostic: it diffs your `rsx!` into a stream of
//! `Mutation`s. We apply those to a retained node tree ([`DioxusEgui`]) and walk
//! it into egui every frame. egui owns input, so `onclick` is delivered back to
//! the VDOM via `runtime().handle_event`.
//!
//! ## Embedded (in-game overlay)
//!
//! ```ignore
//! use dioxus_egui::{DioxusEgui, init_event_converter, render_in_ui};
//! use dioxus::prelude::*;
//!
//! init_event_converter();
//! let mut vdom = VirtualDom::new(app);
//! let mut renderer = DioxusEgui::new();
//! vdom.rebuild(&mut renderer);
//! // each egui frame:
//! render_in_ui(ui, &mut vdom, &mut renderer);
//! ```
//!
//! ## Standalone window (dev harness)
//!
//! Enable the `standalone` feature and call [`run`].

pub use dioxus;
pub use egui_taffy;

#[cfg(feature = "standalone")]
pub use eframe;

mod renderer;
mod style;

pub use renderer::{set_native_draw, DioxusEgui, DomEvent};
pub use style::{container_style, default_style, flex};

use std::any::Any;
use std::rc::Rc;
use std::sync::Once;

use dioxus::dioxus_core::{Event, VirtualDom};
use dioxus_html::{
    set_event_converter, PlatformEventData, SerializedFormData, SerializedHtmlEventConverter, SerializedMouseData,
};
use egui_taffy::{taffy, tui, Tui};

static EVENT_CONVERTER: Once = Once::new();

/// Install the html event converter once. Required before any VDOM receives events.
pub fn init_event_converter() {
    EVENT_CONVERTER.call_once(|| {
        set_event_converter(Box::new(SerializedHtmlEventConverter));
    });
}

/// Build a click event an html `onclick` listener can consume.
fn click_event() -> Event<dyn Any> {
    let platform = PlatformEventData::new(Box::new(SerializedMouseData::default()));
    Event::new(Rc::new(platform) as Rc<dyn Any>, true)
}

/// Build a form event (`oninput`/`onchange`) carrying `value`.
fn form_event(value: String) -> Event<dyn Any> {
    let platform = PlatformEventData::new(Box::new(SerializedFormData::new(value, Vec::new())));
    Event::new(Rc::new(platform) as Rc<dyn Any>, true)
}

/// Deliver one observed [`DomEvent`] into the VirtualDom as a real Dioxus event.
pub fn deliver_dom_event(vdom: &VirtualDom, event: &DomEvent) {
    deliver(vdom, event);
}

/// Deliver one observed [`DomEvent`] into the VirtualDom as a real Dioxus event.
fn deliver(vdom: &VirtualDom, event: &DomEvent) {
    match event {
        DomEvent::Click(eid) => {
            vdom.runtime().handle_event("click", click_event(), *eid);
        }
        DomEvent::Form { id, name, value } => {
            vdom.runtime().handle_event(name, form_event(value.clone()), *id);
        }
    }
}

/// Number of egui layout passes when embedding in an existing `Ui` (matches
/// Hachimi's egui_taffy settling requirement).
pub const EMBED_MAX_PASSES: usize = 3;

/// Render a Dioxus tree into an existing `egui::Ui` region (in-game overlay path).
///
/// Diffs the VDOM once, then walks the retained tree up to [`EMBED_MAX_PASSES`]
/// times so egui_taffy layout can settle. Delivers any observed input events
/// back into the VDOM and re-renders when state changes.
pub fn render_in_ui(ui: &mut egui::Ui, vdom: &mut VirtualDom, renderer: &mut DioxusEgui) {
    init_event_converter();

    ui.ctx().options_mut(|o| {
        o.max_passes = std::num::NonZeroUsize::new(EMBED_MAX_PASSES).expect("non-zero passes");
    });
    ui.ctx().all_styles_mut(|s| {
        s.wrap_mode = Some(egui::TextWrapMode::Extend);
    });

    let mut events: Vec<DomEvent> = Vec::new();
    for pass in 0..EMBED_MAX_PASSES {
        let mut pass_events: Vec<DomEvent> = Vec::new();
        tui(ui, ui.id().with(("dioxus-embed", pass)))
            .reserve_available_space()
            .style(taffy::Style {
                flex_direction: taffy::FlexDirection::Column,
                align_items: Some(taffy::AlignItems::Stretch),
                size: taffy::Size {
                    width: taffy::prelude::percent(1.0),
                    height: taffy::prelude::auto(),
                },
                ..Default::default()
            })
            .show(|tui: &mut Tui| {
                renderer.render(tui, &mut pass_events);
            });

        if pass_events.is_empty() {
            break;
        }
        events.extend(pass_events);
    }

    if !events.is_empty() {
        for event in &events {
            deliver(vdom, event);
        }
        vdom.process_events();
        vdom.render_immediate(renderer);
        ui.ctx().request_repaint();
    }
}

#[cfg(feature = "standalone")]
struct DioxusApp {
    title: String,
    vdom: VirtualDom,
    renderer: DioxusEgui,
}

#[cfg(feature = "standalone")]
impl eframe::App for DioxusApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let title = self.title.clone();
        let mut events: Vec<DomEvent> = Vec::new();
        tui(ui, ui.id().with("dioxus-egui-root"))
            .reserve_available_space()
            .style(taffy::Style {
                flex_direction: taffy::FlexDirection::Column,
                align_items: Some(taffy::AlignItems::Stretch),
                gap: taffy::prelude::length(10.0),
                padding: taffy::prelude::length(12.0),
                ..Default::default()
            })
            .show(|tui: &mut Tui| {
                tui.label(egui::RichText::new(title).strong());
                self.renderer.render(tui, &mut events);
            });

        if !events.is_empty() {
            for event in &events {
                deliver(&self.vdom, event);
            }
            self.vdom.process_events();
            self.vdom.render_immediate(&mut self.renderer);
            ui.ctx().request_repaint();
        }
    }
}

/// Run a Dioxus component in an eframe window (dev harness / gallery only).
#[cfg(feature = "standalone")]
pub fn run(title: &str, app: fn() -> Element) -> eframe::Result<()> {
    init_event_converter();

    let mut vdom = VirtualDom::new(app);
    let mut renderer = DioxusEgui::new();
    vdom.rebuild(&mut renderer);

    let title = title.to_string();
    let window_title = title.clone();
    eframe::run_native(
        &window_title,
        eframe::NativeOptions::default(),
        Box::new(move |_cc| Ok(Box::new(DioxusApp { title, vdom, renderer }))),
    )
}

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use dioxus::prelude::*;

    fn counter() -> Element {
        let mut count = use_signal(|| 0);
        let mut items = use_signal(|| vec!["alpha".to_string(), "beta".to_string()]);
        rsx! {
            div { "dir": "col",
                button { onclick: move |_| count += 1, "+" }
                "count = {count}"
                div { "dir": "col",
                    for (i, item) in items().into_iter().enumerate() {
                        div { "dir": "row", key: "{i}",
                            "{item}"
                            button { onclick: move |_| { items.write().remove(i); }, "remove" }
                        }
                    }
                }
            }
        }
    }

    fn click(vdom: &mut VirtualDom, r: &mut DioxusEgui, label: &str) -> String {
        let eid = r
            .buttons()
            .into_iter()
            .find(|(_, l)| l == label)
            .unwrap_or_else(|| panic!("no button {label:?}"))
            .0;
        deliver(vdom, &DomEvent::Click(eid));
        vdom.process_events();
        vdom.render_immediate(r);
        r.dump()
    }

    fn change(vdom: &mut VirtualDom, r: &mut DioxusEgui, kind: &str, name: &'static str, value: &str) -> String {
        let id = r
            .inputs()
            .into_iter()
            .find(|(_, k)| k == kind)
            .unwrap_or_else(|| panic!("no input of type {kind:?}"))
            .0;
        deliver(
            vdom,
            &DomEvent::Form {
                id,
                name,
                value: value.to_string(),
            },
        );
        vdom.process_events();
        vdom.render_immediate(r);
        r.dump()
    }

    #[test]
    fn onclick_drives_state() {
        init_event_converter();
        let mut vdom = VirtualDom::new(counter);
        let mut r = DioxusEgui::new();
        vdom.rebuild(&mut r);

        assert!(r.dump().contains("count = 0"));
        assert!(click(&mut vdom, &mut r, "+").contains("count = 1"));

        let d = click(&mut vdom, &mut r, "remove");
        assert!(!d.contains("alpha") && d.contains("beta"), "after remove: {d}");
    }

    fn widgets() -> Element {
        let mut agree = use_signal(|| false);
        let mut volume = use_signal(|| 25.0_f64);
        let mut name = use_signal(String::new);
        rsx! {
            div { "dir": "col",
                input {
                    r#type: "checkbox",
                    checked: agree(),
                    onchange: move |e| agree.set(e.checked()),
                }
                "agree = {agree}"
                input {
                    r#type: "range",
                    value: "{volume}",
                    min: "0",
                    max: "100",
                    oninput: move |e| {
                        if let Ok(v) = e.value().parse::<f64>() {
                            volume.set(v);
                        }
                    },
                }
                "volume = {volume}"
                input {
                    value: "{name}",
                    oninput: move |e| name.set(e.value()),
                }
                "hello {name}"
            }
        }
    }

    #[test]
    fn value_widgets_drive_state() {
        init_event_converter();
        let mut vdom = VirtualDom::new(widgets);
        let mut r = DioxusEgui::new();
        vdom.rebuild(&mut r);

        let d = r.dump();
        assert!(d.contains("agree = false"), "initial: {d}");
        assert!(d.contains("volume = 25"), "initial: {d}");

        let d = change(&mut vdom, &mut r, "checkbox", "change", "true");
        assert!(d.contains("agree = true"), "after check: {d}");

        let d = change(&mut vdom, &mut r, "range", "input", "80");
        assert!(d.contains("volume = 80"), "after slider: {d}");

        let d = change(&mut vdom, &mut r, "text", "input", "world");
        assert!(d.contains("hello world"), "after typing: {d}");
    }
}
