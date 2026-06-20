//! GUI rendering via the Hachimi plugin menu system.
//!
//! With API v9 the host hands plugins the real `egui::Ui`, so we draw with egui
//! directly. Registers a menu section and an overlay that display:
//! - Live career stats read directly from game memory (memory-read mode)

use std::ffi::c_void;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::Ordering;

use crate::core::modules::training_tracker::compat::{egui, ui_from_ptr, Sdk};

use crate::core::modules::training_tracker::memory_reader;

mod career;
mod constants;
mod dimens;
mod dioxus_mount;
// Race-condition icon toggles (weather/season/time). Currently hidden from the
// UI per product decision; kept dormant so it can be re-enabled cheaply.
#[allow(dead_code)]
mod icons;
mod menu;
mod overlay;
mod scenario;
mod skill_shop_tab;
mod snapshot;
mod textures;
// Shared formatting/color helpers; several were consumed only by the removed
// Training/Skills tabs but are kept for reuse.
#[allow(dead_code)]
mod util;

// Public API re-export (unused within crate; kept for external callers).
#[allow(unused_imports)]
pub use util::bond_color;

/// Register the plugin's UI components with the Hachimi GUI.
pub fn register_ui() {
    let sdk = Sdk::get();

    // Top-level Control Center tab (was an L1 page under the Plugins tab). The host
    // already hands us a live `egui::Ui` inside its own native slot, and the page
    // body is pure egui — so draw it directly. (The old C menu-section path went
    // through `dioxus_mount::render_menu`, which spins up a *nested* Dioxus mount +
    // `set_native_draw`; nesting that native slot inside the host's native slot
    // leaves the body unpainted.)
    sdk.register_tab(|ui| {
        if panic::catch_unwind(AssertUnwindSafe(|| menu::draw(ui))).is_err() {
            hlog_error!("training-tracker tab draw PANICKED");
        }
    });

    // Chromeless: the host draws no window frame/header, so our own rounded panel
    // is the entire overlay. Still draggable when overlays are unlocked (a FIXED
    // variant exists via register_panel_chromeless_fixed for pinning it later).
    // Falls back to a framed panel on hosts older than v12.
    if sdk.register_panel_chromeless(constants::OVERLAY_ID, draw_overlay, std::ptr::null_mut()) != 0 {
        hlog_info!(target: "training-tracker", "UI registered (L1 page + chromeless L2 panel)");
    } else {
        hlog_warn!(
            target: "training-tracker",
            "L1 page registered; L2 panel registration declined by host"
        );
    }

    // Unbound by default; the user assigns a chord from the host's Hotkeys tab.
    sdk.register_hotkey(
        "training-tracker.toggle_overlay",
        "Toggle Training Tracker Overlay",
        0,
        0,
        toggle_overlay_hotkey,
        std::ptr::null_mut(),
    );
}

extern "C" fn toggle_overlay_hotkey(_userdata: *mut c_void) {
    if panic::catch_unwind(|| Sdk::get().toggle_overlay(constants::OVERLAY_ID)).is_err() {
        hlog_error!(target: "training-tracker", "toggle_overlay_hotkey PANICKED");
    }
}

extern "C" fn draw_overlay(ui: *mut c_void, _userdata: *mut c_void) {
    // SAFETY: host passes its live `&mut egui::Ui` for this callback.
    let ui = unsafe { ui_from_ptr(ui) };
    if panic::catch_unwind(AssertUnwindSafe(|| dioxus_mount::render_overlay(ui))).is_err() {
        hlog_error!("draw_overlay PANICKED");
    }
}

/// Render the overlay panel directly into a caller-provided `Ui`. Used by the
/// desktop dev-harness to draw the exact same panel in a plain eframe window.
#[cfg(feature = "dev-harness")]
pub fn draw_overlay_for_harness(ui: &mut egui::Ui) {
    draw_overlay_inner(ui);
}

/// Re-export so the dev-harness can point the texture loader at an on-disk icon
/// root (the `textures` submodule is otherwise private to `ui`).
#[cfg(feature = "dev-harness")]
pub(crate) use textures::set_harness_icon_root;

fn draw_overlay_inner(ui: &mut egui::Ui) {
    let tracking = memory_reader::TRACKING.load(Ordering::Relaxed);

    // Fixed content width (× zoom), auto height. Pinning both min and max width
    // bounds `available_width`, which the Career panel uses to size its full-width
    // section strips / columns — without this the host's auto-sizing window and
    // those width-following elements feed back into each other and the overlay
    // grows without bound. Zoom scales the whole panel (font + spacing + width).
    let scale = overlay::apply_scale(ui);
    let width = constants::OVERLAY_BASE_WIDTH * scale;
    // Cap the overlay height so the host's auto-sizing window stops growing with
    // content (which made it scroll the whole panel). Tab bodies scroll inside the
    // remaining height instead. Clamp to the viewport so it never exceeds screen.
    let max_height = (ui.ctx().content_rect().height() * 0.9).min(constants::OVERLAY_MAX_HEIGHT * scale);

    // Hard-allocate a fixed-width column. The host renders a chromeless panel in an
    // auto-sizing window whose `available_width` is large; any content that follows
    // it (section strips, taffy `reserve_available_width`, scroll areas) would grow
    // the panel — and the window with it — without bound. Allocating an exact-width
    // region pins `available_width` so the window auto-sizes down to us.
    ui.allocate_ui_with_layout(egui::vec2(width, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
        ui.set_width(width);
        // Bound available_height for the body so per-tab scroll areas actually
        // scroll instead of inflating the panel.
        ui.set_max_height(max_height);
        // Our own rounded background panel is the overlay's whole visual.
        overlay::panel_frame().show(ui, |ui| {
            if !overlay::draw_shell(ui, tracking) {
                return;
            }
            match crate::core::modules::training_tracker::tabs::selected_tab() {
                crate::core::modules::training_tracker::tabs::Tab::Career => career::draw_tab(ui),
                crate::core::modules::training_tracker::tabs::Tab::Shop => skill_shop_tab::draw(ui),
                crate::core::modules::training_tracker::tabs::Tab::Scenario => scenario::draw(ui),
            }
        });
    });
}
