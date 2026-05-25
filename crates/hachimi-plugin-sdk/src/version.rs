//! Host API version from `hachimi_init` — use for load-time compatibility only.

use hachimi_plugin_abi::API_VERSION;

/// Host plugin API version supplied to `hachimi_init`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ApiVersion(i32);

impl ApiVersion {
    #[must_use]
    pub const fn new(version: i32) -> Self {
        Self(version)
    }

    #[must_use]
    pub const fn raw(self) -> i32 {
        self.0
    }

    /// Whether the host reported at least `min` (check once at init, not per call).
    #[must_use]
    pub const fn at_least(self, min: i32) -> bool {
        self.0 >= min
    }

    /// API version of the abi crate this plugin was built against.
    #[must_use]
    pub const fn abi_version() -> Self {
        Self(API_VERSION)
    }
}
