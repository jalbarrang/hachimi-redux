//! Floating overlay panel: live per-runner feed + race facts.

use std::ffi::c_void;
use std::panic::{self, AssertUnwindSafe};

use hachimi_plugin_sdk::{egui, ui_from_ptr, Sdk};

use crate::state::UiState;

const OVERLAY_ID: &str = "race_hud";
const OVERLAY_MIN_WIDTH: f32 = 280.0;

/// Register the race-hud overlay panel with Hachimi's GUI.
pub fn register_ui() {
    let sdk = Sdk::get();
    let handle = sdk.register_panel(OVERLAY_ID, draw_overlay, std::ptr::null_mut());
    if handle == 0 {
        hlog_warn!(target: "race-hud", "Overlay panel registration declined by host");
    } else {
        hlog_info!(target: "race-hud", "Overlay panel registered ({})", handle);
    }
}

extern "C" fn draw_overlay(ui: *mut c_void, _userdata: *mut c_void) {
    // SAFETY: host passes its live `&mut egui::Ui` for this callback.
    let ui = unsafe { ui_from_ptr(ui) };
    if panic::catch_unwind(AssertUnwindSafe(|| draw_overlay_inner(ui))).is_err() {
        hlog_error!(target: "race-hud", "draw_overlay panicked");
    }
}

fn draw_overlay_inner(ui: &mut egui::Ui) {
    let st = crate::state::ui_state();

    ui.set_min_width(OVERLAY_MIN_WIDTH);
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.strong("Race HUD");
            if ui.small_button("Reset").clicked() {
                crate::state::clear_all();
            }
        });
        ui.separator();

        if st.live.is_some() {
            draw_live(ui, &st);
        } else {
            draw_facts(ui, &st);
        }
    });
}

fn draw_live(ui: &mut egui::Ui, st: &UiState) {
    let Some(live) = &st.live else { return };

    ui.horizontal(|ui| {
        ui.monospace(format!("t={:.1}s", live.elapsed));
        ui.monospace(format!("frame {}/{}", live.frame_index + 1, live.frame_count));
    });

    draw_watch_picker(ui, st);

    egui::Grid::new("race_hud_live")
        .striped(true)
        .num_columns(6)
        .show(ui, |ui| {
            ui.small("#");
            ui.small("post");
            ui.small("name");
            ui.small("dist");
            ui.small("m/s");
            ui.small("stam");
            ui.end_row();

            for r in &live.rows {
                let kakari = r.temptation > 0;
                ui.monospace(r.rank.to_string());
                ui.monospace(r.post.to_string());
                ui.label(runner_label(&r.name, r.post));
                ui.monospace(format!("{:.0}", r.distance));
                // Raw speed is stored x100 (cm/s) — show m/s.
                ui.monospace(format!("{:.1}", f32::from(r.speed) / 100.0));
                if kakari {
                    // Temptation (かかり) active — flag the runner.
                    ui.colored_label(egui::Color32::ORANGE, format!("{}!", r.hp));
                } else {
                    ui.monospace(r.hp.to_string());
                }
                ui.end_row();
            }
        });
}

/// Fallback label for runners without a resolved name.
fn runner_label(name: &str, post: u8) -> String {
    if name.is_empty() {
        format!("#{post}")
    } else {
        name.to_owned()
    }
}

/// Collapsible per-runner watch filter. Unchecked runners are hidden from the grid.
fn draw_watch_picker(ui: &mut egui::Ui, st: &UiState) {
    if st.watch.is_empty() {
        return;
    }
    egui::CollapsingHeader::new("Watch").default_open(false).show(ui, |ui| {
        ui.horizontal(|ui| {
            if ui.small_button("All").clicked() {
                crate::state::set_all_watched(true);
            }
            if ui.small_button("None").clicked() {
                crate::state::set_all_watched(false);
            }
        });
        for entry in &st.watch {
            let mut checked = entry.watched;
            let label = format!("{}. {}", entry.post, runner_label(&entry.name, entry.post));
            if ui.checkbox(&mut checked, label).changed() {
                crate::state::toggle_watch(entry.post);
            }
        }
    });
}

fn draw_facts(ui: &mut egui::Ui, st: &UiState) {
    match st.summary {
        Some(s) => {
            ui.colored_label(egui::Color32::LIGHT_GREEN, "Race decoded");
            ui.monospace(format!("Runners:   {}", s.horse_num));
            ui.monospace(format!("Frames:    {}", s.frame_count));
            ui.monospace(format!("Sim dist:  ~{:.0} m (incl. over-run)", s.race_length_m));
            ui.monospace(format!("Sim ver:   {}", s.version));
            ui.monospace(format!("Captures:  {}", st.capture_count));
            ui.add_space(4.0);
            ui.label(egui::RichText::new("Waiting for race start…").small());
        }
        None if st.captured => {
            ui.colored_label(egui::Color32::ORANGE, "Captured, decode failed");
        }
        None => {
            ui.colored_label(egui::Color32::YELLOW, "Waiting for a race…");
            ui.label(egui::RichText::new("Enter a race to capture SimData.").small());
        }
    }
}
