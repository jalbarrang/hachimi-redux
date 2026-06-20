//! Host lifecycle subscription.
//!
//! The plugin reads career state directly from game memory (see `memory_reader`),
//! so it no longer tracks training commands or career start/end. The only event we
//! still need is [`event::SHUTDOWN`], to remove the IL2CPP hooks we installed so the
//! host can safely unload this DLL (UNLOADABLE).

use std::ffi::c_void;

use crate::core::modules::training_tracker::compat::Sdk;
use hachimi_plugin_abi::event;

/// Fired once per rendered frame on the render thread (`data` is null). Drive the
/// overlay-cache refresh here so career snapshots are read/published even when the
/// tracker overlay (or any of its tabs) is not being drawn. The refresh itself is
/// throttled to [`crate::core::modules::training_tracker::overlay_cache::AUTO_REFRESH_INTERVAL_MS`] and is a no-op
/// when tracking is off, so calling it every frame is cheap.
extern "C" fn on_frame(_event_id: u32, _data: *const c_void, _userdata: *mut c_void) {
    crate::core::modules::training_tracker::overlay_cache::maybe_request_refresh();
}

/// Fired before the host unloads this plugin (or on process detach). Remove every
/// IL2CPP hook we installed so the host can safely free the DLL (UNLOADABLE).
extern "C" fn on_shutdown(_event_id: u32, _data: *const c_void, _userdata: *mut c_void) {
    crate::core::modules::training_tracker::memory_reader::stop_tracking();
    crate::core::modules::training_tracker::overlay_cache::shutdown();
    crate::core::modules::training_tracker::shop_hooks::uninstall_shop_hooks();
    hachimi_telemetry::shutdown();
    hlog_info!("Shutdown: tracking stopped, hooks removed");
}

/// Subscribe to the host events we need. Returns `true` if the host advertises the
/// events capability (required for the shutdown teardown).
pub fn subscribe_events() -> bool {
    let sdk = Sdk::get();
    if !sdk.has_capability(hachimi_plugin_abi::capability::EVENTS) {
        hlog_warn!("Host does not advertise the EVENTS capability");
        return false;
    }
    sdk.on(event::SHUTDOWN, on_shutdown, std::ptr::null_mut());
    sdk.on(event::FRAME, on_frame, std::ptr::null_mut());
    true
}
