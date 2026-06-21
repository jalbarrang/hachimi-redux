use std::sync::atomic;

use serde::{Deserialize, Serialize};

use crate::{
    core::Hachimi,
    il2cpp::{
        hook::UnityEngine_CoreModule::{
            FullScreenMode_ExclusiveFullScreen, FullScreenMode_FullScreenWindow, FullScreenMode_Windowed,
            QualitySettings, Screen,
        },
        symbols::Thread,
        types::Resolution,
    },
};

use super::{utils, wnd_hook};

pub fn is_il2cpp_lib(filename: &str) -> bool {
    filename == "GameAssembly.dll"
}

pub fn is_criware_lib(filename: &str) -> bool {
    filename == "cri_ware_unity.dll"
}

pub fn on_hooking_finished(hachimi: &Hachimi) {
    wnd_hook::init();

    // Kill unity crash handler (just to be safe)
    // SAFETY: FFI / raw pointer operation required by IL2CPP interop
    unsafe {
        if let Err(e) = utils::kill_process_by_name(c"UnityCrashHandler64.exe") {
            warn!("Error occurred while trying to kill crash handler: {}", e);
        }
    };

    // Apply vsync
    if hachimi.vsync_count.load(atomic::Ordering::Relaxed) != -1 {
        QualitySettings::set_vSyncCount(1);
    }

    // Apply auto full screen, or an explicit windowed resolution when not in
    // auto-full-screen mode.
    let windows_config = &hachimi.config.load().windows;
    if windows_config.auto_full_screen {
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_secs(2));
            Thread::main_thread().schedule(|| {
                Screen::apply_auto_full_screen(Screen::get_width(), Screen::get_height());
            });
        });
    } else {
        let windowed_res = &windows_config.windowed_res;
        if windowed_res.width > 0 && windowed_res.height > 0 {
            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_secs(2));
                Thread::main_thread().schedule(apply_resolution_main_thread);
            });
        }
    }

    // Clean up the update installer
    _ = std::fs::remove_file(utils::get_tmp_installer_path());
}

/// Main-thread worker: re-reads config and applies the configured resolution.
/// Picks the full-screen target when currently in (or configured for) full
/// screen, otherwise the windowed target. No-op when the relevant resolution is
/// left at default (0x0). Must be a non-capturing `fn` for [`Thread::schedule`].
fn apply_resolution_main_thread() {
    let windows_config = &Hachimi::instance().config.load().windows;
    let fullscreen = Screen::get_fullScreen() || windows_config.auto_full_screen;
    let (res, mode, refresh) = if fullscreen {
        (
            &windows_config.full_screen_res,
            windows_config.full_screen_mode as i32,
            windows_config.full_screen_res.refresh_rate,
        )
    } else {
        (&windows_config.windowed_res, FullScreenMode_Windowed, 0)
    };

    if res.width > 0 && res.height > 0 {
        Screen::set_resolution(res.width, res.height, mode, refresh);
    }
}

/// Apply the resolution configured in `windows.{full_screen_res,windowed_res}`
/// to the running game, scheduled on the main thread.
pub fn apply_current_resolution() {
    Thread::main_thread().schedule(apply_resolution_main_thread);
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default = "Config::default_vsync_count")]
    pub vsync_count: i32,
    #[serde(default)]
    pub load_libraries: Vec<String>,
    /// Opt-in allowlist of manifest-less, legacy-ABI plugins (e.g. upstream
    /// Hachimi data-dumpers) that may load through the compatibility path. Entries
    /// here load on their own — they need not also appear in `load_libraries`
    /// (though listing them in both is harmless; they load once). These plugins only
    /// see the stable vtable prefix; the host cannot track or unload their IL2CPP hooks.
    #[serde(default)]
    pub legacy_libraries: Vec<String>,
    #[serde(default = "Config::default_menu_open_key")]
    pub menu_open_key: u16,
    #[serde(default = "Config::default_hide_ingame_ui_hotkey_bind")]
    pub hide_ingame_ui_hotkey_bind: u16,
    #[serde(default)]
    pub auto_full_screen: bool,
    #[serde(default)]
    pub full_screen_mode: FullScreenMode,
    #[serde(default)]
    pub full_screen_res: Resolution,
    #[serde(default)]
    pub windowed_res: Resolution,
    #[serde(default)]
    pub resolution_scaling: ResolutionScaling,
    #[serde(default)]
    pub block_minimize_in_full_screen: bool,
    #[serde(default)]
    pub window_always_on_top: bool,
    #[serde(default = "Config::default_true")]
    pub discord_rpc: bool,
    #[serde(default = "Config::default_gui_landscape_ratio")]
    pub gui_landscape_ratio: f32,
}

impl Config {
    fn default_vsync_count() -> i32 {
        -1
    }
    fn default_menu_open_key() -> u16 {
        windows::Win32::UI::Input::KeyboardAndMouse::VK_RIGHT.0
    }
    fn default_hide_ingame_ui_hotkey_bind() -> u16 {
        windows::Win32::UI::Input::KeyboardAndMouse::VK_INSERT.0
    }
    fn default_true() -> bool {
        true
    }
    fn default_gui_landscape_ratio() -> f32 {
        1.0
    }
}

#[derive(Deserialize, Serialize, Copy, Clone, Default, Eq, PartialEq)]
#[repr(i32)]
pub enum FullScreenMode {
    #[default]
    ExclusiveFullScreen = FullScreenMode_ExclusiveFullScreen,
    FullScreenWindow = FullScreenMode_FullScreenWindow,
}

#[derive(Deserialize, Serialize, Copy, Clone, Default, Eq, PartialEq)]
pub enum ResolutionScaling {
    #[default]
    Default,
    ScaleToScreenSize,
    ScaleToWindowSize,
}

impl ResolutionScaling {
    pub fn is_not_default(&self) -> bool {
        *self != Self::Default
    }
}
