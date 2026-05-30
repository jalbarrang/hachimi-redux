//! Global `Sdk` instance holding the vtable and negotiated API version.

use std::ffi::{c_void, CString};
use std::sync::OnceLock;

use hachimi_plugin_abi::{log_level, set_vtable, vt, GuiMenuSectionCallback, InitResult, PluginEventFn, Vtable};

use crate::ApiVersion;

static SDK: OnceLock<Sdk> = OnceLock::new();

/// Errors returned when plugin initialization fails.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum InitError {
    NullVtable,
    AlreadyInitialized,
    /// Host `version` is below the plugin's required minimum.
    HostApiTooOld {
        required: i32,
        actual: i32,
    },
}

/// Plugin runtime handle: vtable access and version-aware helpers.
pub struct Sdk {
    version: ApiVersion,
}

impl Sdk {
    /// Install the host vtable and API version. Call once from `hachimi_init`.
    ///
    /// # Safety
    /// `vtable_ptr` must point to a valid `Vtable` for the process lifetime.
    pub unsafe fn init(vtable_ptr: *const Vtable, version: i32) -> Result<(), InitError> {
        // SAFETY: Caller guarantees a valid vtable pointer for process lifetime.
        unsafe { Self::init_min(vtable_ptr, version, i32::MIN) }
    }

    /// Install the vtable after verifying the host API is new enough.
    ///
    /// # Safety
    /// `vtable_ptr` must point to a valid `Vtable` for the process lifetime.
    pub unsafe fn init_min(vtable_ptr: *const Vtable, version: i32, min_host_api: i32) -> Result<(), InitError> {
        if vtable_ptr.is_null() {
            return Err(InitError::NullVtable);
        }
        if version < min_host_api {
            return Err(InitError::HostApiTooOld {
                required: min_host_api,
                actual: version,
            });
        }
        // SAFETY: Caller guarantees a valid vtable pointer for process lifetime.
        unsafe {
            set_vtable(vtable_ptr);
        }
        let sdk = Self {
            version: ApiVersion::new(version),
        };
        SDK.set(sdk).map_err(|_| InitError::AlreadyInitialized)
    }

    /// Global SDK instance after successful [`Self::init`].
    ///
    /// # Panics
    /// If `init` was not called.
    #[must_use]
    pub fn get() -> &'static Self {
        SDK.get().expect("Sdk::init not called")
    }

    /// Global SDK if initialization completed.
    #[must_use]
    pub fn try_get() -> Option<&'static Self> {
        SDK.get()
    }

    #[must_use]
    pub fn version(&self) -> ApiVersion {
        self.version
    }

    #[must_use]
    pub fn vtable(&self) -> &'static Vtable {
        vt()
    }

    // ── Logging ──

    fn log(&self, level: i32, target: &str, message: &str) {
        let (Ok(msg_c), Ok(target_c)) = (CString::new(message), CString::new(target)) else {
            return;
        };
        // SAFETY: host vtable `log` slot is valid after init.
        unsafe { (vt().log)(level, target_c.as_ptr(), msg_c.as_ptr()) }
    }

    pub fn log_info(&self, target: &str, message: &str) {
        self.log(log_level::INFO, target, message);
    }

    pub fn log_warn(&self, target: &str, message: &str) {
        self.log(log_level::WARN, target, message);
    }

    pub fn log_error(&self, target: &str, message: &str) {
        self.log(log_level::ERROR, target, message);
    }

    // ── Capabilities ──

    /// Bitset of host capabilities (see [`hachimi_plugin_abi::capability`]).
    #[must_use]
    pub fn capabilities(&self) -> u64 {
        // SAFETY: host vtable slot is valid after init.
        unsafe { (vt().host_capabilities)() }
    }

    /// Whether the host advertises a given capability bit.
    #[must_use]
    pub fn has_capability(&self, cap: u64) -> bool {
        self.capabilities() & cap != 0
    }

    // ── Events ──

    /// Subscribe to a host event (see [`hachimi_plugin_abi::event`]).
    /// Returns a non-zero handle, or 0 on failure. Keep it to later [`Self::off`].
    pub fn on(&self, event_id: u32, callback: PluginEventFn, userdata: *mut c_void) -> u64 {
        // SAFETY: callback lifetime is managed by the plugin until unsubscribe.
        unsafe { (vt().host_subscribe)(event_id, callback, userdata) }
    }

    /// Remove an event subscription previously returned by [`Self::on`].
    pub fn off(&self, handle: u64) {
        // SAFETY: handle was issued by `host_subscribe`.
        unsafe { (vt().host_unsubscribe)(handle) }
    }

    // ── GUI registration ──

    /// Register a menu section. The callback receives a host `Ui` pointer; cast it
    /// with [`crate::ui_from_ptr`] and draw egui directly. Returns a handle (0 = fail).
    pub fn register_menu_section(&self, callback: GuiMenuSectionCallback, userdata: *mut c_void) -> u64 {
        // SAFETY: callback lifetime managed by plugin; host stores until unregister/unload.
        unsafe { (vt().gui_register_menu_section)(Some(callback), userdata) }
    }

    /// Register an overlay window. Returns a handle (0 = fail).
    pub fn register_overlay(&self, id: &str, callback: GuiMenuSectionCallback, userdata: *mut c_void) -> u64 {
        let Ok(id_c) = CString::new(id) else {
            return 0;
        };
        // SAFETY: callback lifetime managed by plugin.
        unsafe { (vt().gui_register_overlay)(id_c.as_ptr(), Some(callback), userdata) }
    }

    /// Remove a menu item/section/overlay registration by its handle.
    pub fn unregister(&self, handle: u64) -> bool {
        // SAFETY: handle was issued by a `gui_register_*` slot.
        unsafe { (vt().gui_unregister)(handle) }
    }
}

/// Convert init result to raw `i32` for the C entry point.
#[must_use]
pub const fn init_result_to_i32(result: InitResult) -> i32 {
    result as i32
}
