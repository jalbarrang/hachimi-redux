//! Gameplay tab (egui-native).

use rust_i18n::t;

use crate::core::gui::components::{combo, secondary_button, settings_grid, settings_label, slider_f32, toggle};
use crate::core::gui::BoxedWindow;
use crate::core::hachimi;
use crate::il2cpp::hook::umamusume::{CySpringController::SpringUpdateMode, TimeUtil::BgSeason};

/// Draw the Gameplay tab (live path).
pub(crate) fn draw(ui: &mut egui::Ui, config: &mut hachimi::Config, windows: &mut Vec<BoxedWindow>) {
    draw_inner(ui, config, windows, false);
}

/// Draw the Gameplay tab (preview path — season labels use fallback strings).
#[cfg(feature = "dev-harness")]
pub(crate) fn draw_preview(ui: &mut egui::Ui, config: &mut hachimi::Config, windows: &mut Vec<BoxedWindow>) {
    draw_inner(ui, config, windows, true);
}

fn draw_inner(ui: &mut egui::Ui, config: &mut hachimi::Config, windows: &mut Vec<BoxedWindow>, harness: bool) {
    let physics_choices: &[(Option<SpringUpdateMode>, &str)] = &[
        (None, &t!("default")),
        (Some(SpringUpdateMode::ModeNormal), "ModeNormal"),
        (Some(SpringUpdateMode::Mode60FPS), "Mode60FPS"),
        (Some(SpringUpdateMode::SkipFrame), "SkipFrame"),
        (Some(SpringUpdateMode::SkipFramePostAlways), "SkipFramePostAlways"),
    ];

    let season_choices = season_choices(harness);
    let season_refs: Vec<(BgSeason, &str)> = season_choices.iter().map(|(s, l)| (*s, l.as_str())).collect();

    settings_grid(ui, "gameplay_settings", |ui| {
        // Physics update mode
        settings_label(ui, &t!("config_editor.physics_update_mode"));
        combo(ui, "physics", &mut config.physics_update_mode, physics_choices);
        ui.end_row();

        // Story choice auto-select delay
        settings_label(ui, &t!("config_editor.story_choice_auto_select_delay"));
        slider_f32(ui, &mut config.story_choice_auto_select_delay, 0.1..=10.0, 0.05);
        ui.end_row();

        // Story text speed multiplier
        settings_label(ui, &t!("config_editor.story_text_speed_multiplier"));
        slider_f32(ui, &mut config.story_tcps_multiplier, 0.1..=10.0, 0.1);
        ui.end_row();

        // Force allow dynamic camera
        settings_label(ui, &t!("config_editor.force_allow_dynamic_camera"));
        toggle(ui, "", &mut config.force_allow_dynamic_camera);
        ui.end_row();

        // Live theater allow same chara
        settings_label(ui, &t!("config_editor.live_theater_allow_same_chara"));
        toggle(ui, "", &mut config.live_theater_allow_same_chara);
        ui.end_row();

        // Live vocals swap
        settings_label(ui, &t!("config_editor.live_vocals_swap"));
        if secondary_button(ui, t!("open").to_string()).clicked() {
            windows.push(Box::new(super::super::window::LiveVocalsSwapWindow::new()));
        }
        ui.end_row();

        // Skill info dialog
        settings_label(ui, &t!("config_editor.skill_info_dialog"));
        toggle(ui, "", &mut config.skill_info_dialog);
        ui.end_row();

        // Homescreen background season
        settings_label(ui, &t!("config_editor.homescreen_bgseason"));
        combo(ui, "season", &mut config.homescreen_bgseason, &season_refs);
        ui.end_row();
    });
}

fn season_choices(harness: bool) -> Vec<(BgSeason, String)> {
    if harness {
        vec![
            (BgSeason::None, t!("default").to_string()),
            (BgSeason::Spring, "Spring".into()),
            (BgSeason::Summer, "Summer".into()),
            (BgSeason::Fall, "Fall".into()),
            (BgSeason::Winter, "Winter".into()),
            (BgSeason::CherryBlossom, "Cherry Blossom".into()),
        ]
    } else {
        use crate::core::utils::get_localized_string;
        vec![
            (BgSeason::None, t!("default").to_string()),
            (BgSeason::Spring, get_localized_string("Common0108")),
            (BgSeason::Summer, get_localized_string("Common0109")),
            (BgSeason::Fall, get_localized_string("Common0110")),
            (BgSeason::Winter, get_localized_string("Common0111")),
            (BgSeason::CherryBlossom, get_localized_string("Common0112")),
        ]
    }
}
