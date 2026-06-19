//! Gameplay tab.

use dioxus_egui::dioxus::prelude::*;
use honse_ui::{Button, ButtonVariant, Combo, ComboOption, SliderRow, Toggle};
use rust_i18n::t;

use crate::core::utils::get_localized_string;
use crate::il2cpp::hook::umamusume::{CySpringController::SpringUpdateMode, TimeUtil::BgSeason};

use super::super::context::{bind_action, ControlCenterCtx, HostAction};
use super::layout::{LabelCell, SettingsGrid};

#[component]
pub fn GameplayTab() -> Element {
    let ctx = use_context::<ControlCenterCtx>();
    let _ = ctx.revision.read();
    let actions = ctx.actions_rc();
    let cfg = ctx.config.borrow().clone();
    let harness = (ctx.preview_stubs)();

    let season_options = season_options(harness);

    let label_physics_update_mode = t!("config_editor.physics_update_mode").to_string();
    let label_story_choice_auto_select_delay = t!("config_editor.story_choice_auto_select_delay").to_string();
    let label_story_text_speed_multiplier = t!("config_editor.story_text_speed_multiplier").to_string();
    let label_force_allow_dynamic_camera = t!("config_editor.force_allow_dynamic_camera").to_string();
    let label_live_theater_allow_same_chara = t!("config_editor.live_theater_allow_same_chara").to_string();
    let label_live_vocals_swap = t!("config_editor.live_vocals_swap").to_string();
    let open_label = t!("open").to_string();
    let label_skill_info_dialog = t!("config_editor.skill_info_dialog").to_string();
    let label_homescreen_bgseason = t!("config_editor.homescreen_bgseason").to_string();

    rsx! {
        SettingsGrid {
            LabelCell { text: label_physics_update_mode }
            Combo {
                value: physics_key(cfg.physics_update_mode),
                options: physics_options(),
                onchange: ctx.bind(|c, k: String| c.physics_update_mode = parse_physics(&k)),
            }

            LabelCell { text: label_story_choice_auto_select_delay }
            SliderRow {
                label: String::new(),
                value: cfg.story_choice_auto_select_delay as f64,
                min: 0.1,
                max: 10.0,
                step: 0.05,
                oninput: ctx.bind(|c, v| c.story_choice_auto_select_delay = v as f32),
            }

            LabelCell { text: label_story_text_speed_multiplier }
            SliderRow {
                label: String::new(),
                value: cfg.story_tcps_multiplier as f64,
                min: 0.1,
                max: 10.0,
                step: 0.1,
                oninput: ctx.bind(|c, v| c.story_tcps_multiplier = v as f32),
            }

            LabelCell { text: label_force_allow_dynamic_camera }
            Toggle {
                label: String::new(),
                checked: cfg.force_allow_dynamic_camera,
                onchange: ctx.bind(|c, v| c.force_allow_dynamic_camera = v),
            }

            LabelCell { text: label_live_theater_allow_same_chara }
            Toggle {
                label: String::new(),
                checked: cfg.live_theater_allow_same_chara,
                onchange: ctx.bind(|c, v| c.live_theater_allow_same_chara = v),
            }

            LabelCell { text: label_live_vocals_swap }
            Button {
                variant: ButtonVariant::Secondary,
                onclick: bind_action(&actions, HostAction::OpenLiveVocalsSwap),
                {open_label}
            }

            LabelCell { text: label_skill_info_dialog }
            Toggle {
                label: String::new(),
                checked: cfg.skill_info_dialog,
                onchange: ctx.bind(|c, v| c.skill_info_dialog = v),
            }

            LabelCell { text: label_homescreen_bgseason }
            Combo {
                value: season_key(cfg.homescreen_bgseason),
                options: season_options,
                onchange: ctx.bind(|c, k: String| c.homescreen_bgseason = parse_season(&k)),
            }
        }
    }
}

fn physics_options() -> Vec<ComboOption> {
    vec![
        ComboOption {
            value: "default".into(),
            label: t!("default").to_string(),
        },
        ComboOption {
            value: "normal".into(),
            label: "ModeNormal".into(),
        },
        ComboOption {
            value: "60fps".into(),
            label: "Mode60FPS".into(),
        },
        ComboOption {
            value: "skip".into(),
            label: "SkipFrame".into(),
        },
        ComboOption {
            value: "skip_post".into(),
            label: "SkipFramePostAlways".into(),
        },
    ]
}

fn physics_key(mode: Option<SpringUpdateMode>) -> String {
    match mode {
        None => "default",
        Some(SpringUpdateMode::ModeNormal) => "normal",
        Some(SpringUpdateMode::Mode60FPS) => "60fps",
        Some(SpringUpdateMode::SkipFrame) => "skip",
        Some(SpringUpdateMode::SkipFramePostAlways) => "skip_post",
    }
    .into()
}

fn parse_physics(k: &str) -> Option<SpringUpdateMode> {
    match k {
        "normal" => Some(SpringUpdateMode::ModeNormal),
        "60fps" => Some(SpringUpdateMode::Mode60FPS),
        "skip" => Some(SpringUpdateMode::SkipFrame),
        "skip_post" => Some(SpringUpdateMode::SkipFramePostAlways),
        _ => None,
    }
}

fn season_options(harness: bool) -> Vec<ComboOption> {
    let (spring, summer, fall, winter, cherry) = if harness {
        (
            "Spring".to_string(),
            "Summer".to_string(),
            "Fall".to_string(),
            "Winter".to_string(),
            "Cherry Blossom".to_string(),
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
    vec![
        ComboOption {
            value: "none".into(),
            label: t!("default").to_string(),
        },
        ComboOption {
            value: "spring".into(),
            label: spring,
        },
        ComboOption {
            value: "summer".into(),
            label: summer,
        },
        ComboOption {
            value: "fall".into(),
            label: fall,
        },
        ComboOption {
            value: "winter".into(),
            label: winter,
        },
        ComboOption {
            value: "cherry".into(),
            label: cherry,
        },
    ]
}

fn season_key(season: BgSeason) -> String {
    match season {
        BgSeason::None => "none",
        BgSeason::Spring => "spring",
        BgSeason::Summer => "summer",
        BgSeason::Fall => "fall",
        BgSeason::Winter => "winter",
        BgSeason::CherryBlossom => "cherry",
    }
    .into()
}

fn parse_season(k: &str) -> BgSeason {
    match k {
        "spring" => BgSeason::Spring,
        "summer" => BgSeason::Summer,
        "fall" => BgSeason::Fall,
        "winter" => BgSeason::Winter,
        "cherry" => BgSeason::CherryBlossom,
        _ => BgSeason::None,
    }
}
