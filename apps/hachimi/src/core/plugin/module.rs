//! In-core module interface — the first-party tier of the two-tier plugin model.
//!
//! A [`CoreModule`] is a feature compiled directly into `hachimi.dll` that uses the
//! same owner-scoped registries as cdylib plugins ([`super::menu`], [`super::overlay`],
//! [`super::events`], [`super::hotkeys`]) but registers Rust closures directly instead
//! of crossing the C ABI. Both tiers implement this trait, so the host drives init and
//! teardown through one interface:
//! - cdylib plugins: [`super::Plugin`] implements [`CoreModule`] (the C-ABI adapter).
//! - first-party features: their own structs implement [`CoreModule`] and are registered
//!   here, then driven by [`bootstrap`].
//!
//! In-core modules are attributed to high owner ids ([`MODULE_OWNER_BASE`]) that cannot
//! collide with the small, load-order ids cdylib plugins receive, so
//! [`super::teardown_owner`] reclaims a module's registrations exactly as for a plugin.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use once_cell::sync::Lazy;

use super::OwnerScope;

/// A first-party feature compiled into the host. Its lifecycle mirrors a cdylib
/// plugin's: `init` runs once (under the module's [`OwnerScope`]) to register UI,
/// overlays, events and hooks; `shutdown` undoes anything that must be torn down
/// (e.g. IL2CPP hooks) before the host detaches.
pub trait CoreModule: Send {
    /// Stable, human-readable module name (used in logs).
    fn name(&self) -> &str;
    /// Register UI / overlays / events / hooks. Runs inside the module's owner scope,
    /// so registrations are attributed to it automatically.
    fn init(&mut self);
    /// Release anything that must be explicitly undone (IL2CPP hooks especially).
    /// Owner-scoped GUI/event registrations are reclaimed separately by
    /// [`super::teardown_owner`] and need not be removed here.
    fn shutdown(&mut self) {}
}

struct RegisteredModule {
    owner: u32,
    module: Box<dyn CoreModule>,
}

/// Owner-id base for in-core modules. Far above the load-order ids (1, 2, …) handed
/// to cdylib plugins, so the two id spaces never overlap.
pub(crate) const MODULE_OWNER_BASE: u32 = 0xF000_0000;

static NEXT_MODULE_OWNER: AtomicU32 = AtomicU32::new(MODULE_OWNER_BASE);

static MODULES: Lazy<Mutex<Vec<RegisteredModule>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Register an in-core module, assigning it a fresh owner id. Call before
/// [`bootstrap`]; modules registered after bootstrap are not initialized until the
/// next bootstrap.
#[allow(dead_code)] // used by `register_builtins` (feature-gated) and tests
pub(crate) fn register(module: Box<dyn CoreModule>) -> u32 {
    let owner = NEXT_MODULE_OWNER.fetch_add(1, Ordering::Relaxed);
    MODULES
        .lock()
        .expect("lock poisoned")
        .push(RegisteredModule { owner, module });
    owner
}

/// Construct and register the built-in first-party modules. Each in-core feature
/// adds a (feature-gated) registration here.
fn register_builtins() {
    #[cfg(feature = "training-tracker")]
    register(Box::new(crate::core::modules::training_tracker::TrainingTracker::new()));
}

/// Initialize every registered in-core module. Called once during host startup,
/// right after cdylib plugins are initialized, so both tiers come up in the same
/// phase. A module panic is contained so it cannot abort host init.
pub fn bootstrap() {
    register_builtins();
    let mut modules = MODULES.lock().expect("lock poisoned");
    for rm in modules.iter_mut() {
        info!("Initializing in-core module: {}", rm.module.name());
        let _scope = OwnerScope::enter(rm.owner);
        let name = rm.module.name().to_owned();
        let result = catch_unwind(AssertUnwindSafe(|| rm.module.init()));
        if result.is_err() {
            error!("in-core module '{}' panicked during init", name);
        }
    }
}

/// Shut down every in-core module and reclaim its registrations. Called on host
/// detach. Safe to call when no modules are registered.
pub fn shutdown_all() {
    let mut modules = MODULES.lock().expect("lock poisoned");
    for rm in modules.iter_mut() {
        let _scope = OwnerScope::enter(rm.owner);
        let name = rm.module.name().to_owned();
        let _ = catch_unwind(AssertUnwindSafe(|| rm.module.shutdown()))
            .inspect_err(|_| error!("in-core module '{}' panicked during shutdown", name));
        super::teardown_owner(rm.owner);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// A dummy in-core module that registers a menu section on init, so the test can
    /// prove the Rust-tier registration path is owner-scoped and torn down like the
    /// C-tier path.
    struct DummyModule;

    impl CoreModule for DummyModule {
        fn name(&self) -> &str {
            "dummy"
        }
        fn init(&mut self) {
            super::super::menu::register_menu_section_rust(
                Some("Dummy".to_owned()),
                None,
                Arc::new(|_ui: &mut egui::Ui| {}),
            );
        }
    }

    #[test]
    fn rust_tier_registration_is_owner_scoped_and_torn_down() {
        let _guard = super::super::TEST_LOCK.lock().expect("lock poisoned");
        super::super::menu::PLUGIN_MENU_SECTIONS
            .lock()
            .expect("lock poisoned")
            .clear();

        // Register + init a dummy module under a fresh owner id.
        let owner = NEXT_MODULE_OWNER.fetch_add(1, Ordering::Relaxed);
        {
            let _scope = OwnerScope::enter(owner);
            let mut m = DummyModule;
            m.init();
        }

        // Its section is registered and attributed to the module's owner.
        let sections = super::super::menu::get_plugin_menu_sections();
        assert!(sections.iter().any(|s| s.owner == owner));

        // Teardown removes exactly that module's registrations.
        super::super::teardown_owner(owner);
        let sections = super::super::menu::get_plugin_menu_sections();
        assert!(!sections.iter().any(|s| s.owner == owner));

        super::super::menu::PLUGIN_MENU_SECTIONS
            .lock()
            .expect("lock poisoned")
            .clear();
    }
}
