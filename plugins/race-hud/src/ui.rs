//! Race overlays: a standalone timer plus one independent widget per player-owned
//! uma (HP + velocity), each its own draggable chromeless panel.

// The Dioxus `rsx!` macro expands to internal `Option::unwrap()` calls banned by
// the workspace `disallowed_methods` lint.
#![allow(clippy::disallowed_methods)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::panic::{self, AssertUnwindSafe};

use hachimi_plugin_sdk::{
    dioxus::prelude::*,
    egui,
    honse_ui::{theme, Toggle},
    ui_from_ptr, Sdk, UiMount,
};

use crate::settings::{self, Metric};
use crate::state::UmaRow;

const TIMER_OVERLAY_ID: &str = "race_hud_timer";
/// Pool of pre-registered per-uma widget slots. Hidden until a race assigns them.
const MAX_UMA_WIDGETS: usize = 3;
const TIMER_WIDTH: f32 = 120.0;
const UMA_WIDTH: f32 = 120.0;

thread_local! {
    static CONTROL_PAGE_MOUNT: RefCell<Option<UiMount>> = const { RefCell::new(None) };
}

/// Overlay id for the `slot`-th uma widget (0-based; 1-based in the id/title).
fn uma_overlay_id(slot: usize) -> String {
    format!("race_hud_uma_{}", slot + 1)
}

/// Register the race-hud overlays: the timer plus the per-uma widget pool, and
/// the L1 control page for toggling which metrics each widget shows.
pub fn register_ui() {
    let sdk = Sdk::get();
    sdk.register_page(draw_control_page, std::ptr::null_mut());
    register_panel(sdk, TIMER_OVERLAY_ID, draw_timer_overlay, std::ptr::null_mut());
    for slot in 0..MAX_UMA_WIDGETS {
        let id = uma_overlay_id(slot);
        // SAFETY-free: the slot index is carried as the userdata "pointer".
        register_panel(sdk, &id, draw_uma_overlay, slot as *mut c_void);
    }

    // Unbound by default; the user assigns a chord from the host's Hotkeys tab.
    sdk.register_hotkey(
        "race-hud.toggle",
        "Toggle Race HUD",
        0,
        0,
        toggle_hud_hotkey,
        std::ptr::null_mut(),
    );
    // NOTE: the plugin never writes overlay visibility — show/hide is entirely the
    // user's choice (persisted by the host's Overlay tab). Visible slots always
    // render a card (placeholder when idle), so there are no invisible ghosts and
    // race start never overrides a hidden widget.
}

fn register_panel(sdk: &Sdk, id: &str, callback: extern "C" fn(*mut c_void, *mut c_void), userdata: *mut c_void) {
    // Chromeless: the overlays draw their own card/chip visuals, so the host must
    // not wrap them in a titled window with a frame and close/collapse buttons.
    let handle = sdk.register_panel_chromeless(id, callback, userdata);
    if handle == 0 {
        hlog_warn!(target: "race-hud", "Overlay panel registration declined: {id}");
    } else {
        hlog_info!(target: "race-hud", "Overlay panel registered: {id} ({handle})");
    }
}

extern "C" fn toggle_hud_hotkey(_userdata: *mut c_void) {
    if panic::catch_unwind(toggle_all_panels).is_err() {
        hlog_error!(target: "race-hud", "toggle_hud_hotkey panicked");
    }
}

/// Flip every race-hud panel to the inverse of the timer's current visibility,
/// so the whole HUD shows/hides together.
fn toggle_all_panels() {
    let sdk = Sdk::get();
    let visible = !sdk.overlay_visible(TIMER_OVERLAY_ID);
    sdk.overlay_set_visible(TIMER_OVERLAY_ID, visible);
    for slot in 0..MAX_UMA_WIDGETS {
        sdk.overlay_set_visible(&uma_overlay_id(slot), visible);
    }
}

extern "C" fn draw_control_page(ui: *mut c_void, _userdata: *mut c_void) {
    // SAFETY: host passes its live `&mut egui::Ui` for this callback.
    let ui = unsafe { ui_from_ptr(ui) };
    if panic::catch_unwind(AssertUnwindSafe(|| draw_control_page_inner(ui))).is_err() {
        hlog_error!(target: "race-hud", "draw_control_page panicked");
    }
}

fn draw_control_page_inner(ui: &mut egui::Ui) {
    CONTROL_PAGE_MOUNT.with(|slot| {
        let mut mount = slot.borrow_mut();
        if mount.is_none() {
            *mount = Some(UiMount::new(control_page_app));
        }
        mount.as_mut().expect("mount").render(ui);
    });
}

fn control_page_app() -> Element {
    rsx! { ControlPage {} }
}

#[component]
fn ControlPage() -> Element {
    let hint = "Show/hide each uma widget from the Overlay tab (Race Hud Uma 1\u{2026}9). \
        The HUD never changes a widget's visibility on its own, and your choices are saved."
        .to_string();

    rsx! {
        div {
            "dir": "col",
            "gap": "8",
            div {
                "color": theme::FG,
                "weight": "bold",
                "font-size": "16",
                "Race HUD"
            }
            div {
                "color": theme::FG_MUTED,
                "There is one draggable widget per uma slot. Choose which metrics each widget shows:"
            }
            for (metric, label) in Metric::ALL {
                MetricToggle { metric, label: label.to_string() }
            }
            div {
                "color": theme::FG_DIM,
                "font-size": "12",
                {hint}
            }
        }
    }
}

#[component]
fn MetricToggle(metric: Metric, label: String) -> Element {
    let checked = settings::is_shown(metric);
    rsx! {
        Toggle {
            label,
            checked,
            onchange: move |on| {
                settings::set_shown(metric, on);
                settings::persist();
            },
        }
    }
}

extern "C" fn draw_timer_overlay(ui: *mut c_void, _userdata: *mut c_void) {
    // SAFETY: host passes its live `&mut egui::Ui` for this callback.
    let ui = unsafe { ui_from_ptr(ui) };
    if panic::catch_unwind(AssertUnwindSafe(|| draw_timer_inner(ui))).is_err() {
        hlog_error!(target: "race-hud", "draw_timer_overlay panicked");
    }
}

extern "C" fn draw_uma_overlay(ui: *mut c_void, userdata: *mut c_void) {
    let slot = userdata as usize;
    // SAFETY: host passes its live `&mut egui::Ui` for this callback.
    let ui = unsafe { ui_from_ptr(ui) };
    if panic::catch_unwind(AssertUnwindSafe(|| draw_uma_inner(ui, slot))).is_err() {
        hlog_error!(target: "race-hud", "draw_uma_overlay panicked (slot {slot})");
    }
}

fn draw_timer_inner(ui: &mut egui::Ui) {
    ui.visuals_mut().override_text_color = Some(text_primary());
    ui.set_min_width(TIMER_WIDTH);
    let st = crate::state::ui_state();
    draw_timer(ui, st.live.as_ref().map(|l| l.elapsed));
}

fn draw_uma_inner(ui: &mut egui::Ui, slot: usize) {
    // Force bright, dark-theme text regardless of the host's configured override.
    ui.visuals_mut().override_text_color = Some(text_primary());
    ui.set_min_width(UMA_WIDTH);
    // Always render a card for a visible slot (placeholder when no race data), so a
    // user-enabled widget is never an invisible ghost.
    let row = crate::state::uma_row(slot).unwrap_or_else(|| placeholder_row(slot));
    draw_uma_card(ui, &row);
}

/// Idle placeholder shown when a visible slot has no uma assigned yet.
fn placeholder_row(slot: usize) -> UmaRow {
    UmaRow {
        name: String::new(),
        post: (slot + 1) as u8,
        hp: 0,
        initial_hp: 0,
        speed: 0,
        accel: 0.0,
        kakari: false,
        blocked: false,
        live: false,
    }
}

fn draw_uma_card(ui: &mut egui::Ui, row: &UmaRow) {
    let name = if row.name.is_empty() {
        format!("Uma {}", row.post)
    } else {
        row.name.clone()
    };

    panel_frame(ui).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 3.0;
        ui.set_min_width(UMA_WIDTH - 16.0);
        // Name (explicit light color: `.strong()` ignores the text override).
        ui.label(egui::RichText::new(name).size(14.0).color(text_primary()));

        // Each metric row is independently toggled from the control page. When the
        // slot has no live data, values show dashes (idle placeholder).
        if settings::is_shown(Metric::Hp) {
            ui.add_space(3.0);
            let hp_ratio = if row.live && row.initial_hp > 0 {
                (f32::from(row.hp) / f32::from(row.initial_hp)).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let hp_value = if row.live {
                row.hp.to_string()
            } else {
                "\u{2014}".to_owned()
            };
            bar_row(ui, "HP", hp_ratio, hp_color(ui, hp_ratio), hp_value);
        }

        let show_vel = settings::is_shown(Metric::Velocity);
        let show_acc = settings::is_shown(Metric::Acceleration);
        if show_vel || show_acc {
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                if show_vel {
                    ui.label(egui::RichText::new("VEL").small().strong().color(faint_text(ui)));
                    let v = if row.live {
                        format!("{:.1}", velocity_mps(row.speed))
                    } else {
                        "\u{2014}".to_owned()
                    };
                    ui.monospace(v);
                }
                if show_vel && show_acc {
                    ui.add_space(14.0);
                }
                if show_acc {
                    ui.label(egui::RichText::new("ACC").small().strong().color(faint_text(ui)));
                    if row.live {
                        ui.monospace(egui::RichText::new(format!("{:+.1}", row.accel)).color(accel_color(row.accel)));
                    } else {
                        ui.monospace("\u{2014}");
                    }
                }
            });
        }

        if settings::is_shown(Metric::States) {
            ui.add_space(3.0);
            states_row(ui, row);
        }
    });
}

fn accel_color(accel: f32) -> egui::Color32 {
    if accel > 0.05 {
        velocity_color()
    } else if accel < -0.05 {
        crit_color()
    } else {
        text_primary()
    }
}

/// Active-state badges (kakari / blocked); a faint "OK" when nothing is active.
fn states_row(ui: &mut egui::Ui, row: &UmaRow) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("ST").small().strong().color(faint_text(ui)));
        let mut any = false;

        if row.kakari {
            badge(ui, "Rushed", kakari_color());
            any = true;
        }

        if row.blocked {
            badge(ui, "Blocked", crit_color());
            any = true;
        }

        if !any {
            let label = if row.live { "OK" } else { "\u{2014}" };
            ui.label(egui::RichText::new(label).small().color(faint_text(ui)));
        }
    });
}

fn badge(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    ui.add(
        egui::Label::new(
            egui::RichText::new(text)
                .small()
                .strong()
                .color(egui::Color32::from_gray(20))
                .background_color(color),
        )
        .selectable(false),
    );
    ui.add_space(2.0);
}

fn bar_row(ui: &mut egui::Ui, label: &str, ratio: f32, color: egui::Color32, value: String) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).small().strong().color(faint_text(ui)));
        draw_bar(
            ui,
            ratio,
            color,
            egui::vec2((ui.available_width() - 62.0).max(24.0), 6.0),
        );
        ui.monospace(value);
    });
}

fn draw_timer(ui: &mut egui::Ui, elapsed: Option<f32>) {
    let fill = surface_color(ui);
    let line = line_color(ui);
    let text = text_primary();
    let live = elapsed.is_some();
    let label = elapsed.map(format_elapsed).unwrap_or_else(|| "--:--.-".to_owned());

    let desired = egui::vec2(TIMER_WIDTH, 30.0);
    let (rect, _) = ui.allocate_exact_size(desired, egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, 15.0, fill);
        ui.painter().rect_stroke(
            rect,
            15.0,
            egui::Stroke::new(1.0, line),
            egui::epaint::StrokeKind::Inside,
        );
        if live {
            ui.painter()
                .circle_filled(egui::pos2(rect.left() + 13.0, rect.center().y), 3.5, crit_color());
        }
        ui.painter().text(
            rect.center() + egui::vec2(if live { 6.0 } else { 0.0 }, 0.0),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::monospace(16.0),
            text,
        );
    }
}

fn draw_bar(ui: &mut egui::Ui, ratio: f32, color: egui::Color32, size: egui::Vec2) {
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        ui.painter()
            .rect_filled(rect, rect.height() / 2.0, ui.visuals().extreme_bg_color);
        let fill = egui::Rect::from_min_max(rect.min, egui::pos2(rect.left() + rect.width() * ratio, rect.bottom()));
        ui.painter().rect_filled(fill, rect.height() / 2.0, color);
    }
}

fn panel_frame(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::new()
        .fill(surface_color(ui))
        .stroke(egui::Stroke::new(1.0, line_color(ui)))
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(8, 6))
}

fn hp_color(ui: &egui::Ui, ratio: f32) -> egui::Color32 {
    if ratio <= 0.2 {
        crit_color()
    } else if ratio <= 0.4 {
        egui::Color32::from_rgb(217, 167, 43)
    } else {
        ui.visuals().widgets.active.bg_fill
    }
}

fn velocity_mps(speed_raw: u16) -> f32 {
    f32::from(speed_raw) / 100.0
}

fn format_elapsed(seconds: f32) -> String {
    let seconds = seconds.max(0.0);
    let minutes = (seconds / 60.0).floor() as u32;
    let seconds = seconds - minutes as f32 * 60.0;
    format!("{minutes}:{seconds:04.1}")
}

/// Solid (fully opaque) chip background so widgets stay readable over busy race
/// backdrops. Derived from the theme surface but forced opaque.
fn surface_color(ui: &egui::Ui) -> egui::Color32 {
    let c = ui.visuals().widgets.inactive.weak_bg_fill;
    egui::Color32::from_rgb(c.r(), c.g(), c.b())
}

fn line_color(ui: &egui::Ui) -> egui::Color32 {
    let c = ui.visuals().window_stroke.color;
    egui::Color32::from_rgb(c.r(), c.g(), c.b())
}

/// Primary HUD text: a fixed near-white so the overlay always reads as a dark-theme
/// element regardless of the host's configured (possibly dim) text override.
fn text_primary() -> egui::Color32 {
    egui::Color32::from_gray(236)
}

fn faint_text(_ui: &egui::Ui) -> egui::Color32 {
    egui::Color32::from_gray(170)
}

fn velocity_color() -> egui::Color32 {
    egui::Color32::from_rgb(70, 194, 232)
}

fn crit_color() -> egui::Color32 {
    egui::Color32::from_rgb(214, 81, 81)
}

fn kakari_color() -> egui::Color32 {
    egui::Color32::from_rgb(255, 140, 46)
}
