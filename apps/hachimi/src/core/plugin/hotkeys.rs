//! Central hotkey registry.
//!
//! Host and plugins register named *actions* (a stable id + a display label + a
//! default key chord + a callback) here; the current bind for each id is persisted
//! in `Config::hotkeys` so rebinds survive reloads. The Windows `WndProc` hook
//! builds a [`Chord`] from each key press and calls [`dispatch`], which runs every
//! enabled action whose effective chord matches.
//!
//! Modeled on `events.rs`/`menu.rs`: entries are owner-scoped (host = owner 0) and
//! removed via `remove_by_owner` / `remove_by_handle` so a plugin's hotkeys are
//! reclaimed when it unloads.

use std::ffi::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;

use once_cell::sync::Lazy;

use super::types::GuiMenuCallback;
use super::{current_owner, next_handle, OwnerScope};
use crate::core::hachimi::HotkeyBind;

/// Modifier bit: Ctrl held.
pub const MOD_CTRL: u8 = 1 << 0;
/// Modifier bit: Shift held.
pub const MOD_SHIFT: u8 = 1 << 1;
/// Modifier bit: Alt held.
pub const MOD_ALT: u8 = 1 << 2;

/// A key combination: a set of modifier bits plus a primary virtual-key code.
/// `vk == 0` means "unbound" and never matches a key press.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Chord {
    pub mods: u8,
    pub vk: u16,
}

impl Chord {
    #[must_use]
    pub fn new(mods: u8, vk: u16) -> Self {
        Self { mods, vk }
    }

    #[must_use]
    pub fn is_bound(self) -> bool {
        self.vk != 0
    }

    #[must_use]
    pub fn matches(self, other: Chord) -> bool {
        self.is_bound() && self.vk == other.vk && self.mods == other.mods
    }
}

impl From<HotkeyBind> for Chord {
    fn from(b: HotkeyBind) -> Self {
        Self { mods: b.mods, vk: b.vk }
    }
}

impl From<Chord> for HotkeyBind {
    fn from(c: Chord) -> Self {
        Self { mods: c.mods, vk: c.vk }
    }
}

/// What runs when a hotkey fires.
#[derive(Clone)]
enum Action {
    /// A built-in host action.
    Host(fn()),
    /// A plugin-provided C callback.
    Plugin { callback: GuiMenuCallback, userdata: usize },
    /// Toggle a plugin overlay's visibility (host-executed, owner-scoped to the
    /// plugin that owns the overlay).
    ToggleOverlay { overlay_id: String },
}

struct Hotkey {
    handle: u64,
    owner: u32,
    id: String,
    label: String,
    default: Chord,
    action: Action,
}

/// A read-only view of a registered hotkey for the settings UI.
#[derive(Clone)]
pub struct HotkeyInfo {
    pub owner: u32,
    pub id: String,
    pub label: String,
    pub default: Chord,
}

static HOTKEYS: Lazy<Mutex<Vec<Hotkey>>> = Lazy::new(|| Mutex::new(Vec::new()));
static CAPTURE: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
/// A completed capture `(action id, chord)`, produced on the WndProc thread by
/// [`finish_capture`] and consumed by the settings UI ([`take_capture_result`]),
/// which applies it to the config working copy (deferred Save/Cancel).
static CAPTURE_RESULT: Lazy<Mutex<Option<(String, Chord)>>> = Lazy::new(|| Mutex::new(None));

fn register(id: String, label: String, default: Chord, action: Action) -> u64 {
    if id.is_empty() {
        return 0;
    }
    let mut hotkeys = HOTKEYS.lock().expect("lock poisoned");
    // Re-registering the same id (e.g. a plugin reload) replaces the old entry.
    hotkeys.retain(|h| h.id != id);
    let handle = next_handle();
    hotkeys.push(Hotkey {
        handle,
        owner: current_owner(),
        id,
        label,
        default,
        action,
    });
    handle
}

/// Register a built-in host hotkey (owner 0).
pub fn register_host(id: &str, label: &str, default: Chord, action: fn()) -> u64 {
    register(id.to_owned(), label.to_owned(), default, Action::Host(action))
}

/// Register a toggle-visibility hotkey for a plugin overlay. Attributed to the
/// current owner (the registering plugin) so it groups under that plugin in the
/// Hotkeys tab and is reclaimed on unload. Unbound by default — the user binds a
/// key in the Hotkeys tab. Id is `overlay.toggle.<overlay_id>` (stable, so a
/// persisted bind survives restarts).
pub(crate) fn register_overlay_toggle(overlay_id: &str) -> u64 {
    let id = format!("overlay.toggle.{overlay_id}");
    let label = super::overlay::display_title(overlay_id);
    register(
        id,
        label,
        Chord::default(),
        Action::ToggleOverlay {
            overlay_id: overlay_id.to_owned(),
        },
    )
}

/// Register a plugin hotkey. Attributed to the current owner so it is removed on
/// unload. Returns a non-zero handle, or 0 on failure.
pub fn register_plugin(
    id: String,
    label: String,
    default: Chord,
    callback: GuiMenuCallback,
    userdata: *mut c_void,
) -> u64 {
    register(
        id,
        label,
        default,
        Action::Plugin {
            callback,
            userdata: userdata as usize,
        },
    )
}

/// Remove all hotkeys owned by `owner`.
pub(crate) fn remove_by_owner(owner: u32) {
    HOTKEYS.lock().expect("lock poisoned").retain(|h| h.owner != owner);
}

/// Remove a hotkey by its action id (used when a single overlay unregisters).
pub(crate) fn remove_by_id(id: &str) {
    HOTKEYS.lock().expect("lock poisoned").retain(|h| h.id != id);
}

/// Remove a hotkey by handle. Returns whether anything was removed.
pub(crate) fn remove_by_handle(handle: u64) -> bool {
    let mut hotkeys = HOTKEYS.lock().expect("lock poisoned");
    let before = hotkeys.len();
    hotkeys.retain(|h| h.handle != handle);
    hotkeys.len() != before
}

/// Snapshot of all registered hotkeys, in registration order, for the UI.
pub fn snapshot() -> Vec<HotkeyInfo> {
    HOTKEYS
        .lock()
        .expect("lock poisoned")
        .iter()
        .map(|h| HotkeyInfo {
            owner: h.owner,
            id: h.id.clone(),
            label: h.label.clone(),
            default: h.default,
        })
        .collect()
}

/// The default chord for a registered hotkey id, if any.
pub fn default_chord(id: &str) -> Option<Chord> {
    HOTKEYS
        .lock()
        .expect("lock poisoned")
        .iter()
        .find(|h| h.id == id)
        .map(|h| h.default)
}

/// Resolve an action id's effective chord: the persisted bind if present, else the
/// registered default.
fn effective_chord(id: &str, default: Chord) -> Chord {
    crate::core::Hachimi::instance()
        .config
        .load()
        .hotkeys
        .get(id)
        .map_or(default, |b| Chord::from(*b))
}

/// Run every enabled hotkey whose effective chord matches `pressed`. Returns
/// whether at least one action fired (so the caller can swallow the key).
pub fn dispatch(pressed: Chord) -> bool {
    if !pressed.is_bound() {
        return false;
    }

    // Snapshot matching actions under the lock, then run them with the lock
    // released so an action may safely call back into the host.
    let targets: Vec<(u32, Action)> = {
        let hotkeys = HOTKEYS.lock().expect("lock poisoned");
        if hotkeys.is_empty() {
            return false;
        }
        hotkeys
            .iter()
            .filter(|h| effective_chord(&h.id, h.default).matches(pressed))
            .map(|h| (h.owner, h.action.clone()))
            .collect()
    };

    if targets.is_empty() {
        return false;
    }

    for (owner, action) in targets {
        match action {
            Action::Host(f) => {
                let _ = catch_unwind(AssertUnwindSafe(f)).inspect_err(|_| error!("host hotkey action panicked"));
            }
            Action::Plugin { callback, userdata } => {
                let _scope = OwnerScope::enter(owner);
                let _ = catch_unwind(AssertUnwindSafe(|| callback(userdata as *mut c_void)))
                    .inspect_err(|_| error!("plugin hotkey callback panicked (owner {})", owner));
            }
            Action::ToggleOverlay { overlay_id } => {
                let _ = catch_unwind(AssertUnwindSafe(|| {
                    let visible = super::overlay::is_overlay_visible(&overlay_id);
                    super::overlay::set_overlay_visible(&overlay_id, !visible);
                }))
                .inspect_err(|_| error!("overlay toggle hotkey panicked"));
            }
        }
    }
    true
}

/// Begin capturing the next key press for `id` (UI "Set" button).
pub fn start_capture(id: String) {
    *CAPTURE.lock().expect("lock poisoned") = Some(id);
}

/// If a capture is in progress, consume it and return the target action id.
pub fn take_capture() -> Option<String> {
    CAPTURE.lock().expect("lock poisoned").take()
}

/// Whether a capture is currently in progress.
pub fn is_capturing() -> bool {
    CAPTURE.lock().expect("lock poisoned").is_some()
}

/// Complete an in-progress capture (called from the WndProc hook on key press):
/// stash `(id, chord)` for the settings UI to apply to its working copy. Returns
/// the action id (for the "hotkey set" notification), or `None` if no capture was
/// active. Does NOT write the live config — the rebind only persists on Save.
pub fn finish_capture(chord: Chord) -> Option<String> {
    let id = take_capture()?;
    *CAPTURE_RESULT.lock().expect("lock poisoned") = Some((id.clone(), chord));
    Some(id)
}

/// Consume a completed capture result, if any. The settings UI polls this each
/// frame and writes it into the config working copy.
pub fn take_capture_result() -> Option<(String, Chord)> {
    CAPTURE_RESULT.lock().expect("lock poisoned").take()
}

/// Register the built-in host hotkeys. Called once after config is loaded.
pub fn register_builtins() {
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_INSERT, VK_RIGHT};

    register_host(
        super::HOTKEY_OPEN_MENU,
        "hotkeys.open_menu",
        Chord::new(0, VK_RIGHT.0),
        host_open_menu,
    );
    register_host(
        super::HOTKEY_HIDE_INGAME_UI,
        "hotkeys.hide_ingame_ui",
        Chord::new(0, VK_INSERT.0),
        host_hide_ingame_ui,
    );
}

fn host_open_menu() {
    if let Some(mut gui) = crate::core::Gui::instance().map(|m| m.lock().expect("lock poisoned")) {
        gui.toggle_menu();
    }
}

fn host_hide_ingame_ui() {
    crate::il2cpp::symbols::Thread::main_thread().schedule(crate::core::Gui::toggle_game_ui);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static HITS: AtomicU32 = AtomicU32::new(0);

    extern "C" fn count(_u: *mut c_void) {
        HITS.fetch_add(1, Ordering::Relaxed);
    }

    fn clear() {
        HOTKEYS.lock().expect("lock poisoned").clear();
    }

    #[test]
    fn remove_by_owner_is_scoped() {
        let _guard = super::super::TEST_LOCK.lock().expect("lock poisoned");
        clear();
        {
            let _s = OwnerScope::enter(7);
            register_plugin(
                "a.x".into(),
                "x".into(),
                Chord::new(0, 0x70),
                count,
                std::ptr::null_mut(),
            );
        }
        {
            let _s = OwnerScope::enter(8);
            register_plugin(
                "b.y".into(),
                "y".into(),
                Chord::new(0, 0x71),
                count,
                std::ptr::null_mut(),
            );
        }
        assert_eq!(HOTKEYS.lock().expect("lock poisoned").len(), 2);
        remove_by_owner(7);
        let hk = HOTKEYS.lock().expect("lock poisoned");
        assert_eq!(hk.len(), 1);
        assert_eq!(hk[0].owner, 8);
        drop(hk);
        clear();
    }

    #[test]
    fn chord_matches_requires_same_mods_and_vk() {
        let c = Chord::new(MOD_CTRL, 0x70);
        assert!(c.matches(Chord::new(MOD_CTRL, 0x70)));
        assert!(!c.matches(Chord::new(0, 0x70)));
        assert!(!c.matches(Chord::new(MOD_CTRL, 0x71)));
        assert!(!Chord::new(0, 0).matches(Chord::new(0, 0)));
    }

    #[test]
    fn re_register_same_id_replaces() {
        let _guard = super::super::TEST_LOCK.lock().expect("lock poisoned");
        clear();
        register_host("dup", "l1", Chord::new(0, 0x70), || {});
        register_host("dup", "l2", Chord::new(0, 0x71), || {});
        let hk = HOTKEYS.lock().expect("lock poisoned");
        assert_eq!(hk.len(), 1);
        assert_eq!(hk[0].default.vk, 0x71);
        drop(hk);
        clear();
    }
}
