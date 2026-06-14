//! L2 overlay shell: tracking toggle, tab bar, scroll helper, content scaling.

use egui_taffy::taffy::prelude::{auto, length};
use egui_taffy::{taffy, tui, TuiBuilderLogic};
use hachimi_plugin_sdk::{egui, Sdk};

use crate::memory_reader;
use crate::overlay_prefs;

use super::constants::{MIN_LIST_HEIGHT, OVERLAY_BASE_WIDTH, OVERLAY_FONT_SIZE};
use super::dimens;

/// Panel-frame inner margin (must match [`panel_frame`]); content sits inside it.
const PANEL_INNER_MARGIN: f32 = 10.0;
use crate::tabs::{self, selected_tab, set_selected_tab, Tab};

/// Apply the user's content zoom to `ui` (font size + spacing) so the whole
/// panel scales uniformly. The zoom is an explicit setting (slider), not derived
/// from the window size — deriving it from width fed back into the panel's
/// auto-sizing and grew without bound. Returns the applied scale.
pub(super) fn apply_scale(ui: &mut egui::Ui) -> f32 {
    let scale = overlay_prefs::zoom();
    let style = ui.style_mut();
    // Scale every text style so `.small()` / `.strong()` / default labels follow
    // the zoom. `override_font_id` alone is not enough: text set via a `TextStyle`
    // or an explicit size bypasses it (egui 0.33 `FontSelection::resolve`).
    for (_, font_id) in style.text_styles.iter_mut() {
        font_id.size *= scale;
    }
    style.override_font_id = Some(egui::FontId::proportional(OVERLAY_FONT_SIZE * scale));
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
    OVERLAY_BASE_WIDTH * scale() - 2.0 * PANEL_INNER_MARGIN
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
#[allow(dead_code)]
pub(super) fn font_size() -> f32 {
    OVERLAY_FONT_SIZE * scale()
}

/// Add vertical space that scales with the panel.
#[allow(dead_code)]
pub(super) fn space(ui: &mut egui::Ui, base: f32) {
    ui.add_space(base * scale());
}

/// Compact zoom slider so the user can scale the whole panel up or down.
///
/// The slider edits a *pending* value; the live zoom (and thus the panel + the
/// slider's own size) only changes when the drag ends. This stops the slider from
/// rescaling under the cursor mid-drag, which made it jitter/overshoot.
fn draw_zoom_control(ui: &mut egui::Ui) {
    // Plain egui: the egui `Slider` is an interactive widget whose measured size
    // depends on `slider_width`/`interact_size` (both zoom-scaled per frame), so it
    // can't be a stable egui_taffy leaf — it kept the `shell:zoom` taffy node
    // dirty every frame and flickered the overlay. This is a leaf control row, so
    // egui's own horizontal layout is the right tool.
    ui.horizontal(|ui| {
        ui.label("\u{1f50d} Zoom");
        let mut z = overlay_prefs::pending_zoom();
        let slider = egui::Slider::new(&mut z, overlay_prefs::MIN_ZOOM..=overlay_prefs::MAX_ZOOM)
            // Log scale so the multiplicative range is symmetric: 0.4 .. 2.5
            // places 1.0 (100%) at the visual center (sqrt(0.4 * 2.5) == 1.0).
            .logarithmic(true)
            .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
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

/// A pinned-width flex row, vertically centered, wrapping, with a small gap.
fn row_style(width: f32) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: taffy::FlexDirection::Row,
        flex_wrap: taffy::FlexWrap::Wrap,
        align_items: Some(taffy::AlignItems::Center),
        gap: taffy::Size {
            width: length(dimens::z(dimens::GAP_MD)),
            height: length(dimens::z(dimens::GAP_SM)),
        },
        size: taffy::Size {
            width: length(width),
            height: auto(),
        },
        ..Default::default()
    }
}

fn item_center() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        align_items: Some(taffy::AlignItems::Center),
        ..Default::default()
    }
}



/// Apply overlay chrome and draw tracking toggle + tab bar when tracking is on.
pub(super) fn draw_shell(ui: &mut egui::Ui, tracking: bool) -> bool {
    draw_tracking_toggle(ui, tracking);

    if !tracking {
        draw_start_hint(ui);
        return false;
    }

    ui.separator();
    // TEMPORARY flicker diagnostic.
    let ctx = ui.ctx().clone();
    super::flicker_diag::watch(&ctx, "shell:zoom", || draw_zoom_control(ui));
    // Hide the tab row when only one tab is enabled — the overlay becomes a single
    // clean panel showing just that tab's body.
    if tabs::enabled_count() > 1 {
        super::flicker_diag::watch(&ctx, "shell:tab_bar", || draw_tab_bar(ui));
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
    let w = content_width();
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
    tui(ui, ui.id().with("tab_bar"))
        .reserve_width(w)
        .style(row_style(w))
        .show(|tui| {
            for (tab, label) in Tab::ALL {
                if !tabs::is_enabled(tab) {
                    continue;
                }
                tui.style(item_center()).add(|tui| {
                    tui.ui(|ui| {
                        if ui.selectable_label(selected_tab() == tab, label).clicked() {
                            set_selected_tab(tab);
                        }
                    });
                });
            }
        });
}

/// Compact Start/Stop memory-tracking button for the overlay (above the tabs).
fn draw_tracking_toggle(ui: &mut egui::Ui, tracking: bool) {
    // `try_get` (not `get`) so the desktop dev-harness, which never initializes the
    // SDK, can still render this control without panicking. In the real host the
    // SDK is always present, so notifications behave exactly as before.
    let sdk = Sdk::try_get();
    if tracking {
        if ui.button("\u{23f9} Stop Tracking").clicked() {
            memory_reader::stop_tracking();
            if let Some(sdk) = sdk {
                sdk.show_notification("Memory tracking stopped");
            }
        }
    } else if ui.button("\u{25b6} Start Tracking").clicked() {
        match memory_reader::start_tracking() {
            Ok(()) => {
                if let Some(sdk) = sdk {
                    sdk.show_notification("Memory tracking started!");
                }
            }
            Err(e) => {
                if let Some(sdk) = sdk {
                    sdk.show_notification(&format!("Failed: {}", e));
                }
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
