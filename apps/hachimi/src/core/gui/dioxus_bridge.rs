//! Dioxus mount lifecycle for embedding in an existing `egui::Ui`.

use dioxus::dioxus_core::{Element, VirtualDom};
#[cfg(test)]
use dioxus_egui::DomEvent;
use dioxus_egui::{render_in_ui, DioxusEgui};

/// Owns a Dioxus `VirtualDom` + renderer pair for repeated embed renders.
pub struct DioxusMount {
    vdom: VirtualDom,
    renderer: DioxusEgui,
}

impl DioxusMount {
    /// Build a mount: create `context` inside the runtime (so signals can be
    /// allocated), seed the root scope, then run the initial rebuild.
    pub fn with_root_context_factory<C: Clone + 'static>(
        app: fn() -> Element,
        make_ctx: impl FnOnce() -> C,
    ) -> (Self, C) {
        dioxus_egui::init_event_converter();
        let mut vdom = VirtualDom::new(app);
        let ctx = vdom.in_runtime(make_ctx);
        vdom.provide_root_context(ctx.clone());
        let mut renderer = DioxusEgui::new();
        vdom.rebuild(&mut renderer);
        (Self { vdom, renderer }, ctx)
    }

    /// Run a closure with the Dioxus runtime active (required for signal peek/set).
    pub fn in_runtime<O>(&self, f: impl FnOnce() -> O) -> O {
        self.vdom.in_runtime(f)
    }

    /// Diff VDOM state, then walk into `ui` (multi-pass settling).
    pub fn render(&mut self, ui: &mut egui::Ui) {
        self.vdom.render_immediate(&mut self.renderer);
        render_in_ui(ui, &mut self.vdom, &mut self.renderer);
    }

    #[cfg(test)]
    /// Re-diff the VDOM without walking egui (headless tests).
    pub fn render_immediate(&mut self) {
        self.vdom.render_immediate(&mut self.renderer);
    }

    #[cfg(test)]
    pub fn vdom(&self) -> &VirtualDom {
        &self.vdom
    }

    #[cfg(test)]
    pub fn renderer(&self) -> &DioxusEgui {
        &self.renderer
    }

    #[cfg(test)]
    pub fn deliver(&mut self, event: &DomEvent) {
        dioxus_egui::deliver_dom_event(&self.vdom, event);
        self.vdom.process_events();
        self.vdom.render_immediate(&mut self.renderer);
    }
}
