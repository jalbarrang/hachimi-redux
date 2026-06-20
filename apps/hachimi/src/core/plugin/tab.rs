//! Top-level Control Center tab registry (Rust tier only).
//!
//! Unlike menu sections — which are listed as L1 page chips under the shared
//! Plugins tab — a registered *tab* is surfaced as its own top-level tab in the
//! Control Center shell. The shell's tab enum carries a feature-gated slot for
//! each built-in that registers here; the slot's native draw pulls the registered
//! callback back out and invokes it.
//!
//! Only in-core [`super::CoreModule`]s register tabs (the C ABI exposes no
//! tab-registration entry point), so this registry is Rust-tier only and stores an
//! owner-scoped [`UiCallback::Rust`]. Teardown is owner-scoped like every other
//! registry.
//!
//! Registration/draw are only reachable from a built-in that opts in (currently
//! training-tracker); without that feature the registry is inert, so its
//! producer-side helpers are dead but the owner-scoped teardown hooks still run.
#![cfg_attr(not(feature = "training-tracker"), allow(dead_code))]

use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;

use super::callback::UiCallback;

#[derive(Clone)]
pub(crate) struct RegisteredTab {
    pub(crate) handle: u64,
    pub(crate) owner: u32,
    pub(crate) callback: UiCallback,
}

static PLUGIN_TABS: Lazy<Mutex<Vec<RegisteredTab>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Register a top-level Control Center tab body with a Rust closure. Attributed to
/// the current owner like every other registry, so `teardown_owner` reclaims it.
pub(crate) fn register_tab_rust(callback: Arc<dyn Fn(&mut egui::Ui) + Send + Sync>) -> u64 {
    let handle = super::next_handle();
    PLUGIN_TABS.lock().expect("lock poisoned").push(RegisteredTab {
        handle,
        owner: super::current_owner(),
        callback: UiCallback::Rust(callback),
    });
    handle
}

/// Invoke every registered tab body into `ui`, owner-scoped and panic-isolated.
/// The shell renders a single feature-gated tab slot, so in practice this draws the
/// one registered built-in (training-tracker); extra registrations stack in order.
pub(crate) fn draw(ui: &mut egui::Ui) {
    let tabs = PLUGIN_TABS.lock().expect("lock poisoned").clone();
    for tab in &tabs {
        let _scope = super::OwnerScope::enter(tab.owner);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            tab.callback.invoke(ui);
        }))
        .inspect_err(|_| error!("plugin tab callback panicked"));
    }
}

/// Whether any top-level tab is registered (the shell uses this to decide whether
/// to show an empty state).
#[allow(dead_code)]
pub(crate) fn has_tabs() -> bool {
    !PLUGIN_TABS.lock().expect("lock poisoned").is_empty()
}

/// Remove all tabs owned by `owner`.
pub(crate) fn remove_by_owner(owner: u32) {
    PLUGIN_TABS.lock().expect("lock poisoned").retain(|t| t.owner != owner);
}

/// Remove a tab by handle. Returns whether anything was removed.
pub(crate) fn remove_by_handle(handle: u64) -> bool {
    let mut tabs = PLUGIN_TABS.lock().expect("lock poisoned");
    let before = tabs.len();
    tabs.retain(|t| t.handle != handle);
    tabs.len() != before
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_teardown_is_owner_scoped() {
        let _guard = super::super::TEST_LOCK.lock().expect("lock poisoned");
        PLUGIN_TABS.lock().expect("lock poisoned").clear();

        let handle = {
            let _s = super::super::OwnerScope::enter(7);
            register_tab_rust(Arc::new(|_ui: &mut egui::Ui| {}))
        };
        assert!(has_tabs());

        remove_by_owner(7);
        assert!(!has_tabs());
        // Handle no longer present after owner teardown.
        assert!(!remove_by_handle(handle));

        PLUGIN_TABS.lock().expect("lock poisoned").clear();
    }
}
