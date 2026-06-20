//! Dioxus embed shells — native egui bodies bridged via `native="egui"`.

// The Dioxus `rsx!` macro expands to internal `Option::unwrap()` calls banned by
// the workspace `disallowed_methods` lint.
#![allow(clippy::disallowed_methods)]

use std::cell::RefCell;

use crate::core::modules::training_tracker::compat::{dioxus::prelude::*, dioxus_egui::set_native_draw, UiMount};

thread_local! {
    static OVERLAY_MOUNT: RefCell<Option<UiMount>> = const { RefCell::new(None) };
}

pub fn render_overlay(ui: &mut crate::core::modules::training_tracker::compat::egui::Ui) {
    set_native_draw(super::draw_overlay_inner);
    // Shrink-wrap: the host draws the overlay in an `auto_sized()` window, so the
    // filling render path would inflate it to the whole viewport.
    OVERLAY_MOUNT.with(|slot| render_slot(slot, overlay_shell, ui, true));
}

fn render_slot(
    slot: &RefCell<Option<UiMount>>,
    app: fn() -> Element,
    ui: &mut crate::core::modules::training_tracker::compat::egui::Ui,
    shrink: bool,
) {
    let mut mount = slot.borrow_mut();
    if mount.is_none() {
        *mount = Some(UiMount::new(app));
    }
    let mount = mount.as_mut().expect("mount");
    if shrink {
        mount.render_shrink(ui);
    } else {
        mount.render(ui);
    }
}

fn overlay_shell() -> Element {
    rsx! {
        div {
            "native": "egui"
        }
    }
}
