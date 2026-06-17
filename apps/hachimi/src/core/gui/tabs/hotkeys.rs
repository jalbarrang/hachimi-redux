//! L1 Hotkeys tab — view and safely rebind every registered hotkey.
//!
//! Reads the central hotkey registry (`core::plugin::hotkeys`) and the persisted
//! binds in `Config::hotkeys`, grouping rows by owner (host vs each plugin).

use std::collections::HashMap;

use rust_i18n::t;

use crate::core::hachimi::{Config, HotkeyBind};
use crate::core::plugin::hotkeys::{self, Chord, HotkeyInfo};
use crate::core::Hachimi;
use crate::windows::utils::chord_to_display_label;

use super::super::scale::get_scale;
use super::super::widgets;
use super::super::Gui;
use super::layout::{auto_cell, flex_row, flex_spacer};

/// Virtual keys we warn about when bound without a modifier (game/system critical).
const RESERVED_VKS: &[u16] = &[
    0x01, // VK_LBUTTON
    0x02, // VK_RBUTTON
    0x04, // VK_MBUTTON
    0x08, // VK_BACK
    0x09, // VK_TAB
    0x0D, // VK_RETURN
    0x1B, // VK_ESCAPE
    0x20, // VK_SPACE
];

/// Render the Hotkeys tab against the config-editor working copy.
pub(crate) fn ui_hotkeys(ui: &mut egui::Ui, ctx: &egui::Context, config: &mut Config) {
    let scale = get_scale(ctx);

    if let Some((id, chord)) = hotkeys::take_capture_result() {
        config.hotkeys.insert(id, chord.into());
    }

    let infos = hotkeys::snapshot();
    if infos.is_empty() {
        ui.add_space(8.0);
        widgets::empty_state(ui, t!("hotkeys.empty").into_owned());
        return;
    }

    let effective: HashMap<String, Chord> = infos
        .iter()
        .map(|info| {
            let chord = config.hotkeys.get(&info.id).map_or(info.default, |b| Chord::from(*b));
            (info.id.clone(), chord)
        })
        .collect();

    let mut chord_counts: HashMap<(u8, u16), u32> = HashMap::new();
    for chord in effective.values() {
        if chord.is_bound() {
            *chord_counts.entry((chord.mods, chord.vk)).or_insert(0) += 1;
        }
    }

    ui.add_space(4.0);
    ui.label(t!("hotkeys.description"));
    ui.add_space(4.0);

    let owner_names = plugin_owner_names();

    let mut owners: Vec<u32> = infos.iter().map(|i| i.owner).collect();
    owners.sort_unstable();
    owners.dedup();

    for owner in owners {
        let heading = if owner == 0 {
            t!("hachimi").into_owned()
        } else {
            owner_names
                .get(&owner)
                .cloned()
                .unwrap_or_else(|| t!("hotkeys.unknown_plugin").into_owned())
        };
        widgets::section_header(ui, heading);

        for info in infos.iter().filter(|i| i.owner == owner) {
            let chord = effective.get(&info.id).copied().unwrap_or_default();
            let conflict = chord.is_bound() && chord_counts.get(&(chord.mods, chord.vk)).copied().unwrap_or(0) > 1;
            let reserved = chord.is_bound() && chord.mods == 0 && RESERVED_VKS.contains(&chord.vk);
            match hotkey_row(ui, scale, info, chord, conflict, reserved) {
                Some(RowAction::Set) => {
                    hotkeys::start_capture(info.id.clone());
                    notify_capture_start();
                }
                Some(RowAction::Clear) => {
                    config.hotkeys.insert(info.id.clone(), HotkeyBind::default());
                }
                Some(RowAction::Reset) => {
                    config.hotkeys.insert(info.id.clone(), info.default.into());
                }
                None => {}
            }
        }
    }
}

enum RowAction {
    Set,
    Clear,
    Reset,
}

fn hotkey_row(
    ui: &mut egui::Ui,
    scale: f32,
    info: &HotkeyInfo,
    chord: Chord,
    conflict: bool,
    reserved: bool,
) -> Option<RowAction> {
    let mut action = None;
    let label = t!(info.label.as_str()).into_owned();
    let chord_label = chord_to_display_label(chord.mods, chord.vk);

    flex_row(ui, ui.id().with(("hotkey", &info.id)), scale, 6.0, |tui| {
        auto_cell(tui, |ui| {
            ui.add_sized([180.0 * scale, 20.0 * scale], egui::Label::new(&label).truncate());
        });

        if conflict {
            auto_cell(tui, |ui| {
                ui.colored_label(egui::Color32::from_rgb(240, 180, 80), "\u{f071}")
                    .on_hover_text(t!("hotkeys.conflict_warning"));
            });
        } else if reserved {
            auto_cell(tui, |ui| {
                ui.colored_label(egui::Color32::from_rgb(240, 180, 80), "\u{f071}")
                    .on_hover_text(t!("hotkeys.reserved_warning"));
            });
        }

        flex_spacer(tui);

        auto_cell(tui, |ui| {
            ui.label(&chord_label);
        });
        auto_cell(tui, |ui| {
            if widgets::secondary_button(ui, t!("hotkeys.set").into_owned()).clicked() {
                action = Some(RowAction::Set);
            }
        });
        auto_cell(tui, |ui| {
            if widgets::secondary_button(ui, t!("hotkeys.clear").into_owned()).clicked() {
                action = Some(RowAction::Clear);
            }
        });
        auto_cell(tui, |ui| {
            if widgets::secondary_button(ui, t!("hotkeys.reset").into_owned()).clicked() {
                action = Some(RowAction::Reset);
            }
        });
    });

    action
}

fn notify_capture_start() {
    std::thread::spawn(|| {
        if let Some(gui) = Gui::instance() {
            gui.lock()
                .expect("lock poisoned")
                .show_notification(&t!("notification.press_to_set_hotkey"));
        }
    });
}

fn plugin_owner_names() -> HashMap<u32, String> {
    Hachimi::instance()
        .plugins
        .lock()
        .expect("lock poisoned")
        .iter()
        .map(|p| (p.id, p.name.clone()))
        .collect()
}
