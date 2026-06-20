//! Dual-tier callback storage shared by the plugin registries.
//!
//! A registration's callback can come from either tier:
//! - **C tier** — a `extern "C"` function pointer plus an opaque `userdata`,
//!   used by cdylib plugins reached over the [`super::Vtable`].
//! - **Rust tier** — a boxed Rust closure, used by in-core [`super::CoreModule`]s
//!   that register directly without crossing the FFI boundary.
//!
//! Both tiers are stored side by side and stay owner-scoped, so `remove_by_owner`
//! / `teardown_owner` reclaim them identically regardless of tier.

use std::ffi::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use super::types::{GuiMenuCallback, GuiMenuSectionCallback};
use hachimi_plugin_abi::PluginEventFn;

/// A UI-drawing callback handed an `egui::Ui` (menu sections and overlays).
#[derive(Clone)]
pub(crate) enum UiCallback {
    /// cdylib `extern "C"` callback + opaque userdata.
    C {
        func: GuiMenuSectionCallback,
        userdata: usize,
    },
    /// In-core Rust closure. (Constructed by in-core modules; first user lands with
    /// the training-tracker port.)
    #[allow(dead_code)]
    Rust(Arc<dyn Fn(&mut egui::Ui) + Send + Sync>),
}

impl UiCallback {
    /// Invoke the callback with `ui`. The C tier callback pointer is a plain
    /// `extern "C" fn`, safe to call; the cast to the FFI `*mut c_void` matches the
    /// pointer the cdylib expects.
    pub(crate) fn invoke(&self, ui: &mut egui::Ui) {
        match self {
            UiCallback::C { func, userdata } => {
                func(ui as *mut egui::Ui as *mut c_void, *userdata as *mut c_void);
            }
            UiCallback::Rust(f) => f(ui),
        }
    }
}

/// An action callback taking no UI (menu items and hotkeys).
#[derive(Clone)]
pub(crate) enum ActionCallback {
    /// cdylib `extern "C"` callback + opaque userdata.
    C { func: GuiMenuCallback, userdata: usize },
    /// In-core Rust closure. (First user lands with the training-tracker port.)
    #[allow(dead_code)]
    Rust(Arc<dyn Fn() + Send + Sync>),
}

impl ActionCallback {
    pub(crate) fn invoke(&self) {
        match self {
            ActionCallback::C { func, userdata } => func(*userdata as *mut c_void),
            ActionCallback::Rust(f) => f(),
        }
    }
}

/// A host→plugin event callback (`event_id`, event-specific `data`).
#[derive(Clone)]
pub(crate) enum EventCallback {
    /// cdylib `extern "C"` callback + opaque userdata.
    C { func: PluginEventFn, userdata: usize },
    /// In-core Rust closure. (First user lands with the training-tracker port.)
    #[allow(dead_code)]
    Rust(Arc<dyn Fn(u32, *const c_void) + Send + Sync>),
}

impl EventCallback {
    /// Invoke the callback for `event_id`, wrapped in `catch_unwind` so a panic in
    /// plugin/module code cannot unwind across the host's event-dispatch loop.
    pub(crate) fn invoke_catch(&self, event_id: u32, data: *const c_void) {
        let result = catch_unwind(AssertUnwindSafe(|| match self {
            EventCallback::C { func, userdata } => func(event_id, data, *userdata as *mut c_void),
            EventCallback::Rust(f) => f(event_id, data),
        }));
        if result.is_err() {
            error!("plugin event callback panicked (event {})", event_id);
        }
    }
}
