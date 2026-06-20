//! First-party in-core feature modules.
//!
//! Each submodule is a feature that was historically shipped as a cdylib plugin but
//! is now compiled directly into the host, implementing
//! [`crate::core::plugin::CoreModule`]. They are feature-gated so a lean build can
//! exclude them, and registered with the module bootstrap in
//! [`crate::core::plugin::module`].

#[cfg(feature = "training-tracker")]
pub mod training_tracker;

/// Basenames of DLLs that were historically shipped as standalone cdylib plugins
/// but are now compiled into the host. Each entry is gated on the feature that
/// brings its in-core module up (matching
/// [`crate::core::plugin::module::register_builtins`]), so the list reflects what
/// is actually built in. The library loader skips these names.
pub const BUILTIN_MODULE_LIBRARIES: &[&str] = &[
    #[cfg(feature = "training-tracker")]
    "hachimi_training_tracker.dll",
];

/// True if `name` is a former standalone-plugin DLL whose feature now ships inside
/// `hachimi.dll`. Used by the loader to skip stale `load_libraries` entries left by
/// older installs, which would otherwise load a second copy alongside the in-core
/// module and double-register its overlays/hooks. Match is case-insensitive.
pub fn is_builtin_module_library(name: &str) -> bool {
    BUILTIN_MODULE_LIBRARIES
        .iter()
        .any(|lib| name.eq_ignore_ascii_case(lib))
}
