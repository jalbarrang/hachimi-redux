//! L1 Gameplay tab — gameplay-related config options.

use std::thread;

use rust_i18n::t;

use crate::core::hachimi;
use crate::core::utils::get_localized_string;
use crate::il2cpp::hook::umamusume::{CySpringController::SpringUpdateMode, TimeUtil::BgSeason};

use egui_taffy::Tui;

use super::super::components as widgets;
use super::super::window::LiveVocalsSwapWindow;
use super::super::Gui;
use super::layout::{auto_cell, fill_cell, label_cell};

pub(crate) fn options(config: &mut hachimi::Config, tui: &mut Tui, harness: bool) {
    label_cell(tui, t!("config_editor.physics_update_mode"));
    auto_cell(tui, |ui| {
        Gui::run_combo(
            ui,
            "physics_update_mode",
            &mut config.physics_update_mode,
            &[
                (None, &t!("default")),
                (SpringUpdateMode::ModeNormal.into(), "ModeNormal"),
                (SpringUpdateMode::Mode60FPS.into(), "Mode60FPS"),
                (SpringUpdateMode::SkipFrame.into(), "SkipFrame"),
                (SpringUpdateMode::SkipFramePostAlways.into(), "SkipFramePostAlways"),
            ],
        );
    });

    label_cell(tui, t!("config_editor.story_choice_auto_select_delay"));
    fill_cell(tui, |ui| {
        ui.add(egui::Slider::new(&mut config.story_choice_auto_select_delay, 0.1..=10.0).step_by(0.05));
    });

    label_cell(tui, t!("config_editor.story_text_speed_multiplier"));
    fill_cell(tui, |ui| {
        ui.add(egui::Slider::new(&mut config.story_tcps_multiplier, 0.1..=10.0).step_by(0.1));
    });

    label_cell(tui, t!("config_editor.force_allow_dynamic_camera"));
    auto_cell(tui, |ui| {
        ui.checkbox(&mut config.force_allow_dynamic_camera, "");
    });

    label_cell(tui, t!("config_editor.live_theater_allow_same_chara"));
    auto_cell(tui, |ui| {
        ui.checkbox(&mut config.live_theater_allow_same_chara, "");
    });

    label_cell(tui, t!("config_editor.live_vocals_swap"));
    auto_cell(tui, |ui| {
        if widgets::secondary_button(ui, t!("open").into_owned()).clicked() {
            thread::spawn(|| {
                Gui::instance()
                    .expect("unexpected failure")
                    .lock()
                    .expect("unexpected failure")
                    .show_window(Box::new(LiveVocalsSwapWindow::new()));
            });
        }
    });

    label_cell(tui, t!("config_editor.skill_info_dialog"));
    auto_cell(tui, |ui| {
        ui.checkbox(&mut config.skill_info_dialog, "");
    });

    label_cell(tui, t!("config_editor.homescreen_bgseason"));
    let (spring, summer, fall, winter, cherry) = if harness {
        (
            "Spring".to_owned(),
            "Summer".to_owned(),
            "Fall".to_owned(),
            "Winter".to_owned(),
            "Cherry Blossom".to_owned(),
        )
    } else {
        (
            get_localized_string("Common0108"),
            get_localized_string("Common0109"),
            get_localized_string("Common0110"),
            get_localized_string("Common0111"),
            get_localized_string("Common0112"),
        )
    };
    auto_cell(tui, |ui| {
        Gui::run_combo(
            ui,
            "homescreen_bgseason",
            &mut config.homescreen_bgseason,
            &[
                (BgSeason::None, &t!("default")),
                (BgSeason::Spring, spring.as_str()),
                (BgSeason::Summer, summer.as_str()),
                (BgSeason::Fall, fall.as_str()),
                (BgSeason::Winter, winter.as_str()),
                (BgSeason::CherryBlossom, cherry.as_str()),
            ],
        );
    });
}
