//! Training tracking via host lifecycle events (no plugin-side IL2CPP hooks).
//!
//! Previously this module re-implemented IL2CPP hooks on
//! `SingleModeMainViewController.SendCommandAsync` to observe training commands.
//! The host now owns that hook and emits [`event::TRAINING_COMMAND`], plus
//! [`event::CAREER_START`] / [`event::CAREER_END`] from its Single Mode watcher,
//! so we simply subscribe instead of deriving our own hooks.

use std::ffi::c_void;

use hachimi_plugin_abi::{event, TrainingCommandEvent};
use hachimi_plugin_sdk::Sdk;

use crate::tracker::{Facility, TRACKER};

extern "C" fn on_training_command(_event_id: u32, data: *const c_void, _userdata: *mut c_void) {
    if data.is_null() {
        return;
    }
    // SAFETY: for TRAINING_COMMAND the host points `data` at a TrainingCommandEvent
    // that is valid for the callback's duration.
    let cmd = unsafe { &*(data as *const TrainingCommandEvent) };

    if let Some(facility) = Facility::from_command_id(cmd.command_id) {
        if let Ok(mut tracker) = TRACKER.lock() {
            tracker.active = true;
            tracker.record_training(facility);
            hlog_info!(
                "Training recorded: {} (command_id={}, total={})",
                facility.name(),
                cmd.command_id,
                tracker.counts[facility as usize]
            );
        }
    }
}

extern "C" fn on_career_start(_event_id: u32, _data: *const c_void, _userdata: *mut c_void) {
    if let Ok(mut tracker) = TRACKER.lock() {
        tracker.reset();
        tracker.active = true;
    }
    hlog_info!("Career started — tracker reset");
}

extern "C" fn on_career_end(_event_id: u32, _data: *const c_void, _userdata: *mut c_void) {
    if let Ok(mut tracker) = TRACKER.lock() {
        tracker.active = false;
    }
    hlog_info!("Career ended");
}

/// Fired before the host unloads this plugin (or on process detach). Remove every
/// IL2CPP hook we installed so the host can safely free the DLL (UNLOADABLE).
extern "C" fn on_shutdown(_event_id: u32, _data: *const c_void, _userdata: *mut c_void) {
    crate::shop_hooks::uninstall_shop_hooks();
    hlog_info!("Shutdown: hooks removed");
}

/// Subscribe to the host events that drive tracking. Returns `true` if the host
/// advertises the events capability and the training-command subscription took.
pub fn subscribe_events() -> bool {
    let sdk = Sdk::get();
    if !sdk.has_capability(hachimi_plugin_abi::capability::EVENTS) {
        hlog_warn!("Host does not advertise the EVENTS capability; tracking disabled");
        return false;
    }

    let null = std::ptr::null_mut();
    let training = sdk.on(event::TRAINING_COMMAND, on_training_command, null);
    sdk.on(event::CAREER_START, on_career_start, null);
    sdk.on(event::CAREER_END, on_career_end, null);
    sdk.on(event::SHUTDOWN, on_shutdown, null);

    training != 0
}
