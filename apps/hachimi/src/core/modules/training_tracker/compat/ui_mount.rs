//! Dioxus embed mount for the in-core training-tracker UI.
//!
//! A verbatim port of `hachimi-plugin-sdk`'s `UiMount` — it owns a Dioxus
//! `VirtualDom` + `DioxusEgui` renderer and renders into an `egui::Ui` each frame.
//! `VirtualDom` is `!Send`, so a [`UiMount`] must live on the render thread (inside
//! the host overlay/menu draw callback).

use dioxus::dioxus_core::{Element, VirtualDom};
use dioxus_egui::{init_event_converter, render_in_ui, render_in_ui_shrink, DioxusEgui};

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

    /// Diff VDOM state, then walk into `ui` (multi-pass settling), filling the
    /// parent's available width.
    pub fn render(&mut self, ui: &mut egui::Ui) {
        self.prepare();
        render_in_ui(ui, &mut self.vdom, &mut self.renderer);
    }

    /// Like [`Self::render`] but shrink-wraps to content (for auto-sized overlays).
    pub fn render_shrink(&mut self, ui: &mut egui::Ui) {
        self.prepare();
        render_in_ui_shrink(ui, &mut self.vdom, &mut self.renderer);
    }

    fn prepare(&mut self) {
        if !self.events_ready {
            init_event_converter();
            self.events_ready = true;
        }
        self.vdom.render_immediate(&mut self.renderer);
    }
}

/// Convenience constructor matching the SDK API the tracker calls.
#[must_use]
pub fn mount(app: fn() -> Element) -> UiMount {
    UiMount::new(app)
}
