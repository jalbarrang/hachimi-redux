//! Floating overlay for view-transition diagnostics (Dioxus).

// The Dioxus `rsx!` macro expands to internal `Option::unwrap()` calls banned by
// the workspace `disallowed_methods` lint.
#![allow(clippy::disallowed_methods)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::panic::{self, AssertUnwindSafe};

use hachimi_plugin_sdk::{
    dioxus::prelude::*,
    egui,
    honse_ui::{theme, Separator, WindowChrome},
    ui_from_ptr, Sdk, UiMount,
};

const OVERLAY_ID: &str = "debug_viewer";
const OVERLAY_MIN_WIDTH: f32 = 260.0;

thread_local! {
    static MOUNT: RefCell<Option<UiMount>> = const { RefCell::new(None) };
}

/// Register the debug overlay panel with Hachimi's GUI.
pub fn register_ui() {
    let sdk = Sdk::get();
    let handle = sdk.register_panel(OVERLAY_ID, draw_overlay, std::ptr::null_mut());

    if handle == 0 {
        hlog_warn!(target: "debug-viewer", "Overlay panel registration declined by host");
    } else {
        hlog_info!(target: "debug-viewer", "Overlay panel registered ({})", handle);
    }

    sdk.register_hotkey(
        "debug-viewer.toggle",
        "Toggle Debug Window",
        0,
        0,
        toggle_overlay_hotkey,
        std::ptr::null_mut(),
    );
}

extern "C" fn toggle_overlay_hotkey(_userdata: *mut c_void) {
    if panic::catch_unwind(|| Sdk::get().toggle_overlay(OVERLAY_ID)).is_err() {
        hlog_error!(target: "debug-viewer", "toggle_overlay_hotkey panicked");
    }
}

extern "C" fn draw_overlay(ui: *mut c_void, _userdata: *mut c_void) {
    // SAFETY: the host passes a valid `&mut egui::Ui` pointer for this callback.
    let ui = unsafe { ui_from_ptr(ui) };
    if panic::catch_unwind(AssertUnwindSafe(|| draw_overlay_inner(ui))).is_err() {
        hlog_error!(target: "debug-viewer", "draw_overlay panicked");
    }
}

fn draw_overlay_inner(ui: &mut egui::Ui) {
    ui.set_min_width(OVERLAY_MIN_WIDTH);
    MOUNT.with(|slot| {
        let mut mount = slot.borrow_mut();
        if mount.is_none() {
            *mount = Some(UiMount::new(overlay_app));
        }
        mount.as_mut().expect("mount").render(ui);
    });
}

fn overlay_app() -> Element {
    rsx! { OverlayPanel {} }
}

#[component]
fn OverlayPanel() -> Element {
    let snapshot = crate::state::snapshot();
    let current = format_view(snapshot.current_view_id);
    let previous = format_view(snapshot.previous_view_id);
    let sequence = snapshot.sequence.to_string();
    let history_rows: Vec<String> = snapshot
        .history
        .iter()
        .rev()
        .map(|entry| {
            format!(
                "#{}  {:.1}s  {}",
                entry.sequence,
                entry.seconds_since_start,
                format_view(Some(entry.view_id))
            )
        })
        .collect();

    rsx! {
        WindowChrome {
            title: "Debug Viewer".to_string(),
            div {
                "dir": "col",
                "gap": "4",
                "font-family": "monospace",
                div { "Current view:  {current}" }
                div { "Previous view: {previous}" }
                div { "Transitions:   {sequence}" }
            }
            Separator {}
            div { "Recent view changes" }
            if history_rows.is_empty() {
                div {
                    "color": theme::FG_DIM,
                    "font-size": "12",
                    "No VIEW_CHANGE events observed yet."
                }
            } else {
                div {
                    "dir": "col",
                    "gap": "2",
                    for row in history_rows {
                        div {
                            "font-family": "monospace",
                            "font-size": "12",
                            {row}
                        }
                    }
                }
            }
        }
    }
}

fn format_view(view_id: Option<i32>) -> String {
    view_id.map_or_else(
        || "—".to_owned(),
        |id| match Sdk::get().view_name(id) {
            Some(name) => format!("{id} ({name})"),
            None => id.to_string(),
        },
    )
}
