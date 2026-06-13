//! L2 overlay shell: tracking toggle, tab bar, scroll helper, content scaling.

use hachimi_plugin_sdk::{egui, Sdk};

use crate::memory_reader;
use crate::overlay_prefs;

use super::constants::{MIN_LIST_HEIGHT, OVERLAY_BASE_WIDTH, OVERLAY_FONT_SIZE};

/// Panel-frame inner margin (must match [`panel_frame`]); content sits inside it.
const PANEL_INNER_MARGIN: f32 = 10.0;
/// Width reserved for the vertical scrollbar gutter so full-width content doesn't
/// sit under the scrollbar (or trigger a horizontal scroll) when it appears.
const SCROLLBAR_GUTTER: f32 = 14.0;
use crate::tabs::{self, selected_tab, set_selected_tab, Tab};

/// Apply the user's content zoom to `ui` (font size + spacing) so the whole
/// panel scales uniformly. The zoom is an explicit setting (slider), not derived
/// from the window size — deriving it from width fed back into the panel's
/// auto-sizing and grew without bound. Returns the applied scale.
pub(super) fn apply_scale(ui: &mut egui::Ui) -> f32 {
    let scale = overlay_prefs::zoom();
    ui.style_mut().override_font_id = Some(egui::FontId::proportional(OVERLAY_FONT_SIZE * scale));
    let sp = ui.spacing_mut();
    sp.item_spacing *= scale;
    sp.button_padding *= scale;
    sp.interact_size *= scale;
    sp.indent *= scale;
    scale
}

/// The current overlay content scale.
pub(super) fn scale() -> f32 {
    overlay_prefs::zoom()
}

/// Deterministic content column width (inside the panel-frame margins), driven by
/// the fixed base width × zoom. Use this instead of `ui.available_width()` for
/// full-width elements: under the host's `auto_sized` window `available_width` is
/// measured with a huge value and would inflate the panel (and the window).
pub(super) fn content_width() -> f32 {
    (OVERLAY_BASE_WIDTH * scale() - 2.0 * PANEL_INNER_MARGIN - SCROLLBAR_GUTTER).max(80.0)
}

/// Max height for a tab body's scroll area: keep the whole overlay within the
/// viewport (leaving room for the shell above + window chrome/margins). Driven by
/// the screen height since `available_height` is unbounded under the auto_sized
/// window.
pub(super) fn body_max_height(ui: &egui::Ui) -> f32 {
    (ui.ctx().content_rect().height() - 200.0).max(220.0)
}

/// The overlay's own background panel (the whole visual, since the host renders
/// the panel chromeless). Rounded dark face with a faint border, matching the
/// Career card so the "inner frame" reads as the overlay itself.
pub(super) fn panel_frame() -> egui::Frame {
    egui::Frame::new()
        .inner_margin(egui::Margin::same(PANEL_INNER_MARGIN as i8))
        .corner_radius(egui::CornerRadius::same(12))
        .fill(egui::Color32::from_rgb(0x12, 0x16, 0x1f))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0x2c, 0x36, 0x48)))
}

/// Scaled base font size for callers that set an explicit text size.
pub(super) fn font_size() -> f32 {
    OVERLAY_FONT_SIZE * scale()
}

/// Add vertical space that scales with the panel.
pub(super) fn space(ui: &mut egui::Ui, base: f32) {
    ui.add_space(base * scale());
}

/// Compact zoom slider so the user can scale the whole panel up or down.
///
/// The slider edits a *pending* value; the live zoom (and thus the panel + the
/// slider's own size) only changes when the drag ends. This stops the slider from
/// rescaling under the cursor mid-drag, which made it jitter/overshoot.
fn draw_zoom_control(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("\u{1f50d} Zoom");
        let mut z = overlay_prefs::pending_zoom();
        let slider = egui::Slider::new(&mut z, overlay_prefs::MIN_ZOOM..=overlay_prefs::MAX_ZOOM)
            .fixed_decimals(2)
            .show_value(true);
        let resp = ui.add(slider);
        if resp.changed() {
            overlay_prefs::set_pending_zoom(z);
        }
        // Commit on release (drag) or on a discrete change (click / keyboard).
        if resp.drag_stopped() || (resp.changed() && !resp.dragged()) {
            overlay_prefs::commit_zoom();
            crate::config::persist();
        }
    });
}

/// Apply overlay chrome and draw tracking toggle + tab bar when tracking is on.
pub(super) fn draw_shell(ui: &mut egui::Ui, tracking: bool) -> bool {
    draw_tracking_toggle(ui, tracking);

    if !tracking {
        draw_start_hint(ui);
        return false;
    }

    ui.separator();
    draw_zoom_control(ui);
    // Hide the tab row when only one tab is enabled — the overlay becomes a single
    // clean panel showing just that tab's body.
    if tabs::enabled_count() > 1 {
        draw_tab_bar(ui);
        ui.separator();
    }
    true
}

/// Hint shown when memory tracking is off.
fn draw_start_hint(ui: &mut egui::Ui) {
    ui.small("\u{1f3cb} Training Tracker");
    ui.small("Memory tracking is off — press Start Tracking above.");
}

/// Horizontal tab bar (text labels) — only the user-enabled tabs are shown.
fn draw_tab_bar(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        for (tab, label) in Tab::ALL {
            if !tabs::is_enabled(tab) {
                continue;
            }
            if ui.selectable_label(selected_tab() == tab, label).clicked() {
                set_selected_tab(tab);
            }
        }
    });
}

/// Compact Start/Stop memory-tracking button for the overlay (above the tabs).
fn draw_tracking_toggle(ui: &mut egui::Ui, tracking: bool) {
    let sdk = Sdk::get();
    if tracking {
        if ui.button("\u{23f9} Stop Tracking").clicked() {
            memory_reader::stop_tracking();
            sdk.show_notification("Memory tracking stopped");
        }
    } else if ui.button("\u{25b6} Start Tracking").clicked() {
        match memory_reader::start_tracking() {
            Ok(()) => {
                sdk.show_notification("Memory tracking started!");
            }
            Err(e) => {
                sdk.show_notification(&format!("Failed: {}", e));
                hlog_error!("start_tracking failed: {}", e);
            }
        }
    }
}

pub(super) fn scroll_list(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
    // Fill the remaining height of the (resizable) panel so vertical resizing is
    // meaningful; fall back to a small minimum when the panel is tiny.
    let max_height = ui.available_height().max(MIN_LIST_HEIGHT);
    egui::ScrollArea::vertical()
        .max_height(max_height)
        .auto_shrink([false, false])
        .show(ui, body);
}
