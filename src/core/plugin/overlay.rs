//! Plugin overlay registration and shared overlay state.
//! Overlays are registered once, stored behind `Lazy<Mutex<_>>`, and may be queued from plugin init.
//! The render side clones a snapshot via `get_plugin_overlays()` before invoking callbacks.
//! This snap-and-render pattern keeps lock scope short on the render thread.
//!
//! Visibility is tracked in a separate map so the render thread can toggle it
//! (via egui::Window close button) without holding the registration lock.

use std::{collections::HashMap, ffi::c_void, sync::Mutex};

use once_cell::sync::Lazy;

use super::types::GuiMenuSectionCallback;

#[derive(Clone)]
pub(crate) struct PluginOverlay {
    pub(crate) handle: u64,
    pub(crate) owner: u32,
    pub(crate) id: String,
    pub(crate) callback: GuiMenuSectionCallback,
    pub(crate) userdata: usize,
}

pub(crate) static PLUGIN_OVERLAYS: Lazy<Mutex<Vec<PluginOverlay>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Per-overlay visibility state, keyed by overlay ID.
/// Defaults to `true` (visible) when an overlay is first registered.
static OVERLAY_VISIBILITY: Lazy<Mutex<HashMap<String, bool>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn register_plugin_overlay(id: String, callback: GuiMenuSectionCallback, userdata: *mut c_void) -> u64 {
    OVERLAY_VISIBILITY
        .lock()
        .expect("lock poisoned")
        .entry(id.clone())
        .or_insert(true);
    let handle = super::next_handle();
    PLUGIN_OVERLAYS.lock().expect("lock poisoned").push(PluginOverlay {
        handle,
        owner: super::current_owner(),
        id,
        callback,
        userdata: userdata as usize,
    });
    handle
}

/// Remove all overlays owned by `owner`.
pub(crate) fn remove_by_owner(owner: u32) {
    PLUGIN_OVERLAYS
        .lock()
        .expect("lock poisoned")
        .retain(|o| o.owner != owner);
}

/// Remove an overlay by handle. Returns whether anything was removed.
pub(crate) fn remove_by_handle(handle: u64) -> bool {
    let mut overlays = PLUGIN_OVERLAYS.lock().expect("lock poisoned");
    let before = overlays.len();
    overlays.retain(|o| o.handle != handle);
    overlays.len() != before
}

pub(crate) fn get_plugin_overlays() -> Vec<PluginOverlay> {
    PLUGIN_OVERLAYS.lock().expect("lock poisoned").clone()
}

pub(crate) fn has_plugin_overlays() -> bool {
    !PLUGIN_OVERLAYS.lock().map_or(true, |o| o.is_empty())
}

/// Get the visibility state for an overlay. Returns `true` if unknown.
pub(crate) fn is_overlay_visible(id: &str) -> bool {
    OVERLAY_VISIBILITY.lock().map_or(true, |m| *m.get(id).unwrap_or(&true))
}

/// Set visibility for an overlay by ID (used by host close-button and plugin vtable call).
pub fn set_overlay_visible(id: &str, visible: bool) {
    OVERLAY_VISIBILITY
        .lock()
        .expect("lock poisoned")
        .insert(id.to_owned(), visible);
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::TEST_LOCK;

    extern "C" fn overlay_callback(_: *mut c_void, _: *mut c_void) {}

    #[test]
    fn has_plugin_overlays_reflects_registration_state() {
        let _guard = TEST_LOCK.lock().expect("lock poisoned");

        {
            let mut overlays = PLUGIN_OVERLAYS.lock().expect("lock poisoned");
            overlays.clear();
        }

        assert!(!has_plugin_overlays());

        let _ = register_plugin_overlay("test".to_owned(), overlay_callback, std::ptr::null_mut());
        assert!(has_plugin_overlays());

        {
            let mut overlays = PLUGIN_OVERLAYS.lock().expect("lock poisoned");
            overlays.clear();
        }
    }

    #[test]
    fn remove_by_owner_only_drops_matching() {
        let _guard = TEST_LOCK.lock().expect("lock poisoned");
        PLUGIN_OVERLAYS.lock().expect("lock poisoned").clear();

        {
            let _s = super::super::OwnerScope::enter(7);
            let _ = register_plugin_overlay("a".to_owned(), overlay_callback, std::ptr::null_mut());
        }
        {
            let _s = super::super::OwnerScope::enter(8);
            let _ = register_plugin_overlay("b".to_owned(), overlay_callback, std::ptr::null_mut());
        }

        remove_by_owner(7);
        let overlays = get_plugin_overlays();
        assert_eq!(overlays.len(), 1);
        assert_eq!(overlays[0].owner, 8);
        PLUGIN_OVERLAYS.lock().expect("lock poisoned").clear();
    }
}
