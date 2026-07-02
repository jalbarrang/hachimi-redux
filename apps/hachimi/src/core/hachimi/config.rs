//! User configuration schema (`config.json`) and its defaults.

use std::collections::BTreeMap;

use fnv::FnvHashSet;
use serde::{Deserialize, Serialize};

use crate::{hachimi_impl, il2cpp::hook::umamusume::CySpringController::SpringUpdateMode};

use super::{default_serde_instance, Language};

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub debug_mode: bool,
    #[serde(default)]
    pub enable_file_logging: bool,
    #[serde(default)]
    pub apply_atlas_workaround: bool,
    #[serde(default)]
    pub translator_mode: bool,
    #[serde(default)]
    pub disable_gui: bool,
    #[serde(default)]
    pub disable_gui_once: bool,
    pub localized_data_dir: Option<String>,
    pub target_fps: Option<i32>,
    #[serde(default = "Config::default_open_browser_url")]
    pub open_browser_url: String,
    #[serde(default = "Config::default_virtual_res_mult")]
    pub virtual_res_mult: f32,
    pub translation_repo_index: Option<String>,
    #[serde(default)]
    pub skip_first_time_setup: bool,
    #[serde(default)]
    pub lazy_translation_updates: bool,
    #[serde(default)]
    pub disable_auto_update_check: bool,
    /// Disable the GameTora data catalog sync (skills/support-cards/umas/events)
    /// downloaded alongside the auto update check.
    #[serde(default)]
    pub disable_gametora_data: bool,
    /// Override the base URL the GameTora data catalog is downloaded from. `None`
    /// uses the repo's hosted copy. Dev/testing escape hatch.
    pub gametora_data_url: Option<String>,
    /// Disable the training-tracker resource sync (`skill_grades.json` /
    /// `course_params.json`) downloaded alongside the auto update check.
    #[serde(default)]
    pub disable_tracker_data: bool,
    /// Override the base URL the training-tracker resources are downloaded from.
    /// `None` uses the repo's hosted copy. Dev/testing escape hatch.
    pub tracker_data_url: Option<String>,
    /// Disable the Career-panel icon sprite sync (~16 MB of PNGs under `icons/`)
    /// downloaded alongside the auto update check.
    #[serde(default)]
    pub disable_icons_data: bool,
    /// Override the base URL the Career icon sprites are downloaded from. `None`
    /// uses the repo's hosted copy. Dev/testing escape hatch.
    pub icons_data_url: Option<String>,
    #[serde(default)]
    pub disable_translations: bool,
    #[serde(default = "Config::default_gui_scale")]
    pub gui_scale: f32,
    #[serde(default = "Config::default_ui_scale")]
    pub ui_scale: f32,
    #[serde(default = "Config::default_render_scale")]
    pub render_scale: f32,
    #[serde(default)]
    pub msaa: crate::il2cpp::hook::umamusume::GraphicSettings::MsaaQuality,
    #[serde(default)]
    pub aniso_level: crate::il2cpp::hook::UnityEngine_CoreModule::Texture::AnisoLevel,
    #[serde(default)]
    pub shadow_resolution: crate::il2cpp::hook::umamusume::CameraData::ShadowResolution,
    #[serde(default)]
    pub graphics_quality: crate::il2cpp::hook::umamusume::GraphicSettings::GraphicsQuality,
    #[serde(default = "Config::default_story_choice_auto_select_delay")]
    pub story_choice_auto_select_delay: f32,
    #[serde(default = "Config::default_story_tcps_multiplier")]
    pub story_tcps_multiplier: f32,
    #[serde(default)]
    pub enable_ipc: bool,
    #[serde(default)]
    pub ipc_listen_all: bool,
    #[serde(default)]
    pub force_allow_dynamic_camera: bool,
    #[serde(default)]
    pub live_theater_allow_same_chara: bool,
    #[serde(default = "Config::default_live_vocals_swap")]
    pub live_vocals_swap: [i32; 6],
    #[serde(default)]
    pub skill_info_dialog: bool,
    #[serde(default)]
    pub homescreen_bgseason: crate::il2cpp::hook::umamusume::TimeUtil::BgSeason,
    pub sugoi_url: Option<String>,
    #[serde(default)]
    pub auto_translate_stories: bool,
    #[serde(default)]
    pub auto_translate_localize: bool,
    #[serde(default)]
    pub disable_skill_name_translation: bool,
    #[serde(default)]
    pub hide_ingame_ui_hotkey: bool,
    /// Hotkey binds keyed by stable action id (e.g. `hachimi.open_menu`). The
    /// central hotkey registry resolves each registered action's effective chord
    /// from here, falling back to its registered default when absent. Cleared
    /// (`vk == 0`) means the action is unbound.
    #[serde(default)]
    pub hotkeys: BTreeMap<String, HotkeyBind>,
    #[serde(default)]
    pub language: Language,
    #[serde(default = "Config::default_meta_index_url")]
    pub meta_index_url: String,
    #[serde(default)]
    pub ipv4_only: bool,
    pub physics_update_mode: Option<SpringUpdateMode>,
    #[serde(default = "Config::default_ui_animation_scale")]
    pub ui_animation_scale: f32,
    #[serde(default = "Config::default_loading_fade_scale")]
    pub loading_fade_scale: f32,
    #[serde(default = "Config::default_flash_animation_scale")]
    pub flash_animation_scale: f32,
    #[serde(default)]
    pub disabled_hooks: FnvHashSet<String>,

    #[cfg(target_os = "windows")]
    #[serde(flatten)]
    pub windows: hachimi_impl::Config,
}

impl Config {
    fn default_open_browser_url() -> String {
        "https://www.google.com/".to_owned()
    }
    fn default_virtual_res_mult() -> f32 {
        1.0
    }
    fn default_ui_scale() -> f32 {
        1.0
    }
    fn default_render_scale() -> f32 {
        1.0
    }
    fn default_gui_scale() -> f32 {
        1.0
    }
    fn default_story_choice_auto_select_delay() -> f32 {
        1.2
    }
    fn default_story_tcps_multiplier() -> f32 {
        3.0
    }
    fn default_meta_index_url() -> String {
        "https://gitlab.com/umatl/hachimi-meta/-/raw/main/meta.json".to_owned()
    }
    fn default_ui_animation_scale() -> f32 {
        1.0
    }
    fn default_loading_fade_scale() -> f32 {
        1.0
    }
    fn default_flash_animation_scale() -> f32 {
        1.0
    }
    fn default_live_vocals_swap() -> [i32; 6] {
        [0; 6]
    }
    /// Seed the `hotkeys` map for the built-in host actions from the legacy
    /// single-key config fields, when those entries are not already present. This
    /// preserves binds for configs written before the central Hotkeys tab existed.
    pub fn migrate_legacy_hotkeys(&mut self) {
        #[cfg(target_os = "windows")]
        {
            self.hotkeys
                .entry("hachimi.open_menu".to_owned())
                .or_insert(HotkeyBind {
                    mods: 0,
                    vk: self.windows.menu_open_key,
                });

            // The legacy hide-UI hotkey had a separate enable flag; a disabled
            // hotkey maps to the unified "unbound" state (vk == 0).
            self.hotkeys
                .entry("hachimi.hide_ingame_ui".to_owned())
                .or_insert(HotkeyBind {
                    mods: 0,
                    vk: if self.hide_ingame_ui_hotkey {
                        self.windows.hide_ingame_ui_hotkey_bind
                    } else {
                        0
                    },
                });
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        default_serde_instance().expect("default instance")
    }
}

/// A persisted hotkey bind: modifier bitmask (Ctrl=1, Shift=2, Alt=4) plus a
/// primary Win32 virtual-key code. `vk == 0` means unbound.
#[derive(Deserialize, Serialize, Clone, Copy, Default, PartialEq, Eq)]
pub struct HotkeyBind {
    #[serde(default)]
    pub mods: u8,
    #[serde(default)]
    pub vk: u16,
}

#[derive(Deserialize, Default, Clone)]
pub struct OsOption<T> {
    #[cfg(target_os = "windows")]
    windows: Option<T>,
}

impl<T> OsOption<T> {
    pub fn as_ref(&self) -> Option<&T> {
        #[cfg(target_os = "windows")]
        return self.windows.as_ref();
    }
}
