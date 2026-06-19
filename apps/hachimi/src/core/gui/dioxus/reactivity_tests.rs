//! Headless reactivity tests for the Control Center Dioxus tree.

#[cfg(test)]
#[allow(clippy::disallowed_methods)]
mod tests {
    use dioxus::prelude::ReadableExt;
    use dioxus_egui::{init_event_converter, DomEvent};

    use crate::core::gui::dioxus::context::ControlCenterCtx;
    use crate::core::gui::dioxus::control_center_app;
    use crate::core::gui::dioxus_bridge::DioxusMount;
    use crate::core::gui::shell::ControlTab;

    fn mount() -> (DioxusMount, ControlCenterCtx) {
        init_event_converter();
        DioxusMount::with_root_context_factory(control_center_app, ControlCenterCtx::new_in_runtime)
    }

    fn click_button(mount: &mut DioxusMount, needle: &str) {
        let eid = mount
            .renderer()
            .buttons()
            .into_iter()
            .find(|(_, label)| label.contains(needle))
            .unwrap_or_else(|| {
                panic!(
                    "no button matching {needle:?}; labels: {:?}",
                    mount.renderer().buttons()
                )
            })
            .0;
        mount.deliver(&DomEvent::Click(eid));
    }

    #[test]
    fn tab_click_switches_active_tab() {
        let (mut mount, ctx) = mount();
        mount.render_immediate();
        assert_eq!(*mount.vdom().in_runtime(|| ctx.active_tab.peek()), ControlTab::General);

        click_button(&mut mount, "Graphics");
        assert_eq!(*mount.vdom().in_runtime(|| ctx.active_tab.peek()), ControlTab::Graphics);

        let dump = mount.renderer().dump();
        assert!(
            dump.contains("[select") || dump.contains("MSAA") || dump.contains("msaa"),
            "graphics tab body not rendered; dump:\n{dump}"
        );
    }

    #[test]
    fn toggle_bumps_revision_and_updates_dump() {
        let (mut mount, ctx) = mount();
        mount.render_immediate();
        let rev_before = *mount.vdom().in_runtime(|| ctx.revision.peek());
        let before = mount.renderer().dump();

        let (eid, kind) = mount
            .renderer()
            .inputs()
            .into_iter()
            .find(|(_, ty)| ty == "checkbox")
            .expect("expected at least one checkbox on the general tab");
        assert_eq!(kind, "checkbox");

        mount.deliver(&DomEvent::Form {
            id: eid,
            name: "change",
            value: "true".into(),
        });

        let rev_after = *mount.vdom().in_runtime(|| ctx.revision.peek());
        assert!(rev_after > rev_before, "revision should bump after toggle");

        let after = mount.renderer().dump();
        assert_ne!(before, after, "checkbox checked state should change in tree");
        assert!(
            after.contains("value=true"),
            "expected checked checkbox in dump:\n{after}"
        );
    }
}
