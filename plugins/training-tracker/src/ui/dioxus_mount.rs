//! Dioxus embed shells — native egui bodies bridged via `native="egui"`.

// The Dioxus `rsx!` macro expands to internal `Option::unwrap()` calls banned by
// the workspace `disallowed_methods` lint.
#![allow(clippy::disallowed_methods)]

use std::cell::RefCell;

use hachimi_plugin_sdk::{dioxus::prelude::*, dioxus_egui::set_native_draw, UiMount};

thread_local! {
    static MENU_MOUNT: RefCell<Option<UiMount>> = const { RefCell::new(None) };
    static OVERLAY_MOUNT: RefCell<Option<UiMount>> = const { RefCell::new(None) };
}

pub fn render_menu(ui: &mut hachimi_plugin_sdk::egui::Ui) {
    set_native_draw(super::menu::draw);
    MENU_MOUNT.with(|slot| render_slot(slot, menu_shell, ui, false));
}

pub fn render_overlay(ui: &mut hachimi_plugin_sdk::egui::Ui) {
    set_native_draw(super::draw_overlay_inner);
    // Shrink-wrap: the host draws the overlay in an `auto_sized()` window, so the
    // filling render path would inflate it to the whole viewport.
    OVERLAY_MOUNT.with(|slot| render_slot(slot, overlay_shell, ui, true));
}

fn render_slot(
    slot: &RefCell<Option<UiMount>>,
    app: fn() -> Element,
    ui: &mut hachimi_plugin_sdk::egui::Ui,
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

fn menu_shell() -> Element {
    rsx! {
        div {
            "native": "egui"
        }
    }
}

fn overlay_shell() -> Element {
    rsx! {
        div {
            "native": "egui"
        }
    }
}
