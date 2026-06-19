//! Dioxus embed mount for plugin GUI callbacks.
//!
//! `VirtualDom` is `!Send` — keep a [`UiMount`] on the render thread (inside the
//! host's overlay/menu draw callback) and call [`UiMount::render`] each frame.

use dioxus::dioxus_core::{Element, VirtualDom};
use dioxus_egui::{init_event_converter, render_in_ui, DioxusEgui};

/// Owns a Dioxus `VirtualDom` + renderer for repeated embed renders into `egui::Ui`.
pub struct UiMount {
    vdom: VirtualDom,
    renderer: DioxusEgui,
    events_ready: bool,
}

impl UiMount {
    /// Create a mount for the given root component function.
    #[must_use]
    pub fn new(app: fn() -> Element) -> Self {
        let mut vdom = VirtualDom::new(app);
        let mut renderer = DioxusEgui::new();
        vdom.rebuild(&mut renderer);
        Self {
            vdom,
            renderer,
            events_ready: false,
        }
    }

    /// Diff VDOM state, then walk into `ui` (multi-pass settling).
    pub fn render(&mut self, ui: &mut egui::Ui) {
        if !self.events_ready {
            init_event_converter();
            self.events_ready = true;
        }
        self.vdom.render_immediate(&mut self.renderer);
        render_in_ui(ui, &mut self.vdom, &mut self.renderer);
    }
}

/// Convenience alias matching the initiative name.
pub fn mount(app: fn() -> Element) -> UiMount {
    UiMount::new(app)
}
