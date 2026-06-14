use std::{
    os::raw::c_uint,
    ptr,
    sync::atomic::{self, AtomicIsize},
};

use windows::{
    core::w,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::Threading::GetCurrentThreadId,
        UI::{
            Input::Ime::ISC_SHOWUICOMPOSITIONWINDOW,
            WindowsAndMessaging::{
                CallNextHookEx, DefWindowProcW, FindWindowW, GetWindowLongPtrW, SetWindowsHookExW, UnhookWindowsHookEx,
                GWLP_WNDPROC, HCBT_MINMAX, HHOOK, SW_RESTORE, WA_INACTIVE, WH_CBT, WM_ACTIVATE, WM_CLOSE,
                WM_IME_NOTIFY, WM_IME_SETCONTEXT, WM_KEYDOWN, WM_SYSKEYDOWN, WNDPROC,
            },
        },
    },
};

use crate::{
    core::{game::Region, plugin::hotkeys, Gui, Hachimi},
    il2cpp::{hook::UnityEngine_CoreModule, symbols::Thread},
    windows::utils,
};
use rust_i18n::t;

use super::{discord, gui_impl::input};

static TARGET_HWND: AtomicIsize = AtomicIsize::new(0);
pub fn get_target_hwnd() -> HWND {
    HWND(TARGET_HWND.load(atomic::Ordering::Relaxed) as *mut _)
}

/// Whether `vk` is a modifier key (Ctrl/Shift/Alt/Win), which must not act as the
/// primary key of a hotkey chord.
fn is_modifier_key(vk: u16) -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        VIRTUAL_KEY, VK_CONTROL, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU, VK_RCONTROL, VK_RMENU, VK_RSHIFT,
        VK_RWIN, VK_SHIFT,
    };
    matches!(
        VIRTUAL_KEY(vk),
        VK_CONTROL
            | VK_LCONTROL
            | VK_RCONTROL
            | VK_SHIFT
            | VK_LSHIFT
            | VK_RSHIFT
            | VK_MENU
            | VK_LMENU
            | VK_RMENU
            | VK_LWIN
            | VK_RWIN
    )
}

/// Current modifier bitmask from the live keyboard state.
fn current_mods() -> u8 {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CONTROL, VK_MENU, VK_SHIFT};
    let mut mods = 0u8;
    // SAFETY: FFI / raw pointer operation required by IL2CPP interop
    unsafe {
        if GetKeyState(VK_CONTROL.0 as i32) < 0 {
            mods |= hotkeys::MOD_CTRL;
        }
        if GetKeyState(VK_SHIFT.0 as i32) < 0 {
            mods |= hotkeys::MOD_SHIFT;
        }
        if GetKeyState(VK_MENU.0 as i32) < 0 {
            mods |= hotkeys::MOD_ALT;
        }
    }
    mods
}

/// Stash a captured chord for the action whose "Set" is in progress, then notify.
/// The rebind goes into the settings UI's working copy (applied on the next GUI
/// frame) and only persists when the user clicks Save — not written live here.
fn capture_key(chord: hotkeys::Chord) {
    if hotkeys::finish_capture(chord).is_none() {
        return;
    }

    let key_label = utils::chord_to_display_label(chord.mods, chord.vk);
    let msg = t!("notification.hotkey_set", key = key_label);
    std::thread::spawn(move || {
        if let Some(gui) = Gui::instance() {
            gui.lock().expect("lock poisoned").show_notification(&msg);
        }
    });
}

// Safety: only modified once on init
static mut WNDPROC_ORIG: isize = 0;
static mut WNDPROC_RECALL: usize = 0;
extern "system" fn wnd_proc(hwnd: HWND, umsg: c_uint, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // SAFETY: Transmute required for IL2CPP type conversion
    let Some(orig_fn) = (unsafe { std::mem::transmute::<isize, WNDPROC>(WNDPROC_ORIG) }) else {
        // SAFETY: FFI / raw pointer operation required by IL2CPP interop
        return unsafe { DefWindowProcW(hwnd, umsg, wparam, lparam) };
    };

    match umsg {
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            let current_key = wparam.0 as u16;

            // Modifier keys never act as the primary key of a chord; let them pass
            // through so the game (and chord detection on the next key) sees them.
            if !is_modifier_key(current_key) {
                let chord = hotkeys::Chord::new(current_mods(), current_key);

                if current_key == 0x4B {
                    // Virtual keycode for "K", see the get_key method on gui_impl/input.rs.
                    // Swallow K while the hide-UI hotkey is held so it doesn't reach the IME.
                    if let Some(bind) = Hachimi::instance()
                        .config
                        .load()
                        .hotkeys
                        .get(crate::core::plugin::HOTKEY_HIDE_INGAME_UI)
                        .copied()
                    {
                        // SAFETY: FFI / raw pointer operation required by IL2CPP interop
                        let bind_held = bind.vk != 0
                            && unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState(bind.vk as i32) < 0 };
                        if bind_held {
                            if let Some(mut gui) = Gui::instance().map(|m| m.lock().expect("lock poisoned")) {
                                gui.set_consuming_input(false);
                            }
                            return LRESULT(0);
                        }
                    }
                }

                if hotkeys::is_capturing() {
                    capture_key(chord);
                    return LRESULT(0);
                }

                if hotkeys::dispatch(chord) {
                    return LRESULT(0);
                }
            }
        }
        WM_ACTIVATE => {
            // SAFETY: FFI / raw pointer operation required by IL2CPP interop
            let res = unsafe { orig_fn(hwnd, umsg, wparam, lparam) };

            if (wparam.0 & 0xFFFF) != WA_INACTIVE as usize {
                std::thread::spawn(move || {
                    if let Some(gui) = Gui::instance().map(|m| m.lock().expect("lock poisoned")) {
                        if gui.context.wants_keyboard_input() {
                            Thread::main_thread().schedule(|| {
                                crate::il2cpp::hook::UnityEngine_InputLegacyModule::Input::set_imeCompositionMode(1);
                            });
                        }
                    }
                });
            }
            return res;
        }
        WM_CLOSE => {
            if let Some(hook) = Hachimi::instance().interceptor.unhook(wnd_proc as *const () as _) {
                // SAFETY: FFI / raw pointer operation required by IL2CPP interop
                unsafe {
                    WNDPROC_RECALL = hook.orig_addr;
                }
                // SAFETY: FFI / raw pointer operation required by IL2CPP interop
                Thread::main_thread().schedule(|| unsafe {
                    let orig_fn = std::mem::transmute::<usize, WNDPROC>(WNDPROC_RECALL).expect("unexpected failure");
                    orig_fn(get_target_hwnd(), WM_CLOSE, WPARAM(0), LPARAM(0));
                });
            }
            return LRESULT(0);
        }
        _ => (),
    }

    // L2 overlays (L1 modal closed): feed mouse input to egui so it can track hover
    // and drag panels, but only *swallow* the input when the cursor is over a panel.
    // Everywhere else the click falls through to the game. Locked panels never
    // capture (L2_WANTS_POINTER stays false), so they are click-through.
    if !Gui::is_consuming_input_atomic() {
        if input::is_mouse_msg(umsg) && crate::core::plugin::overlay::has_plugin_overlays() {
            let wp = wparam;
            let lp = lparam;
            std::thread::spawn(move || {
                let Some(mut gui) = Gui::instance().map(|m| m.lock().expect("lock poisoned")) else {
                    return;
                };
                let zoom_factor = gui.context.zoom_factor();
                input::process(&mut gui.input, zoom_factor, umsg, wp.0, lp.0);
            });
            if Gui::l2_wants_pointer_atomic() {
                return LRESULT(0);
            }
        }
        // SAFETY: FFI / raw pointer operation required by IL2CPP interop
        return unsafe { orig_fn(hwnd, umsg, wparam, lparam) };
    }

    if umsg == WM_IME_SETCONTEXT {
        let new_lparam = lparam.0 & !(ISC_SHOWUICOMPOSITIONWINDOW as isize);
        if Gui::is_consuming_input_atomic() {
            // SAFETY: FFI / raw pointer operation required by IL2CPP interop
            return unsafe { DefWindowProcW(hwnd, umsg, wparam, LPARAM(new_lparam)) };
        }
        // SAFETY: FFI / raw pointer operation required by IL2CPP interop
        return unsafe { orig_fn(hwnd, umsg, wparam, LPARAM(new_lparam)) };
    }

    if umsg == WM_IME_NOTIFY && Gui::is_consuming_input_atomic() {
        // SAFETY: FFI / raw pointer operation required by IL2CPP interop
        return unsafe { DefWindowProcW(hwnd, umsg, wparam, lparam) };
    }

    // Extract the IME data BEFORE spanning the thread
    let (is_ime, ime_commit, ime_preedit) = input::process_ime_sync(hwnd, umsg, lparam.0);

    // Check if the input processor handles this message (Skip check if it is an IME msg)
    if !input::is_handled_msg(umsg) && !is_ime {
        // SAFETY: FFI / raw pointer operation required by IL2CPP interop
        return unsafe { orig_fn(hwnd, umsg, wparam, lparam) };
    }

    // A deadlock would *sometimes* consistently occur if this was done on the current thread
    // (when moving the window, etc.)
    // I assume that SwapChain::Present and WndProc are running on the same thread
    std::thread::spawn(move || {
        let Some(mut gui) = Gui::instance().map(|m| m.lock().expect("lock poisoned")) else {
            return;
        };

        // Inject IME strings directly into egui
        if let Some(s) = ime_commit {
            gui.input.events.push(egui::Event::Ime(egui::ImeEvent::Commit(s)));
        }
        if let Some(s) = ime_preedit {
            gui.input.events.push(egui::Event::Ime(egui::ImeEvent::Preedit(s)));
        }

        // Process standard Key/Mouse inputs ONLY if it wasn't an IME message
        if !is_ime {
            let zoom_factor = gui.context.zoom_factor();
            input::process(&mut gui.input, zoom_factor, umsg, wparam.0, lparam.0);
        }
    });

    if is_ime {
        return LRESULT(0);
    }

    LRESULT(0)
}

static mut HCBTHOOK: HHOOK = HHOOK(ptr::null_mut());
extern "system" fn cbt_proc(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if ncode == HCBT_MINMAX as i32
        && lparam.0 as i32 != SW_RESTORE.0
        && Hachimi::instance().config.load().windows.block_minimize_in_full_screen
        && UnityEngine_CoreModule::Screen::get_fullScreen()
    {
        return LRESULT(1);
    }

    // SAFETY: FFI / raw pointer operation required by IL2CPP interop
    unsafe { CallNextHookEx(Some(HCBTHOOK), ncode, wparam, lparam) }
}

pub fn init() {
    // SAFETY: FFI / raw pointer operation required by IL2CPP interop
    unsafe {
        let hachimi = Hachimi::instance();
        let game = &hachimi.game;

        let window_name = if game.region == Region::Japan && game.is_steam_release {
            // lmao
            w!("UmamusumePrettyDerby_Jpn")
        } else {
            // global technically has "Umamusume" as its title but this api
            // is case insensitive so it works. why am i surprised
            w!("umamusume")
        };
        let hwnd = FindWindowW(w!("UnityWndClass"), window_name).unwrap_or_default();
        if hwnd.0.is_null() {
            error!("Failed to find game window");
            return;
        }
        TARGET_HWND.store(hwnd.0 as isize, atomic::Ordering::Relaxed);

        hotkeys::register_builtins();

        info!("Hooking WndProc");
        let wnd_proc_addr = GetWindowLongPtrW(hwnd, GWLP_WNDPROC);
        match hachimi.interceptor.hook(wnd_proc_addr as _, wnd_proc as *const () as _) {
            Ok(trampoline_addr) => WNDPROC_ORIG = trampoline_addr as _,
            Err(e) => error!("Failed to hook WndProc: {}", e),
        }

        info!("Adding CBT hook");
        if let Ok(hhook) = SetWindowsHookExW(WH_CBT, Some(cbt_proc), None, GetCurrentThreadId()) {
            HCBTHOOK = hhook;
        }

        // Apply always on top
        if hachimi.window_always_on_top.load(atomic::Ordering::Relaxed) {
            _ = utils::set_window_topmost(hwnd, true);
        }

        if hachimi.discord_rpc.load(atomic::Ordering::Relaxed) {
            if let Err(e) = discord::start_rpc() {
                error!("{}", e);
            }
        }
    }
}

pub fn uninit() {
    // SAFETY: FFI / raw pointer operation required by IL2CPP interop
    unsafe {
        if !HCBTHOOK.0.is_null() {
            info!("Removing CBT hook");
            if let Err(e) = UnhookWindowsHookEx(HCBTHOOK) {
                error!("Failed to remove CBT hook: {}", e);
            }
            HCBTHOOK = HHOOK(ptr::null_mut());
        }
        if let Err(e) = discord::stop_rpc() {
            error!("{}", e);
        }
    }
}
