//! Shared Rust-side plugin SDK types.
//! Defines plugin metadata, init result values, and callback signatures used by the ABI.
//! These types are referenced by `api` and by plugin loading code.

pub use hachimi_plugin_abi::{
    GuiMenuCallback, GuiMenuSectionCallback, GuiUiCallback, HachimiInitFn, InitResult, Vtable,
};

pub struct Plugin {
    pub name: String,
    /// Non-zero owner id used to attribute this plugin's registrations and event
    /// subscriptions (see [`super::OwnerScope`]).
    pub id: u32,
    pub init_fn: HachimiInitFn,
}

impl Plugin {
    pub fn init(&self) -> InitResult {
        // Attribute everything the plugin registers during init to its owner id.
        let _scope = super::OwnerScope::enter(self.id);
        super::api::init_plugin(self.init_fn)
    }
}

/// C-ABI adapter: a cdylib plugin is driven through the same [`CoreModule`] lifecycle
/// interface as in-core modules. `init` runs the plugin's `hachimi_init` over the
/// vtable (with owner attribution); `shutdown` reclaims its owner-scoped registrations.
/// The plugin's DLL itself is kept mapped — see `windows::main::unload_plugin`.
impl super::CoreModule for Plugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn init(&mut self) {
        if !Plugin::init(self).is_ok() {
            info!("Plugin init failed: {}", self.name);
        }
    }

    fn shutdown(&mut self) {
        super::teardown_owner(self.id);
    }
}
