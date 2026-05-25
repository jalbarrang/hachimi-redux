# hachimi-plugin-sdk

Recommended plugin SDK: re-exports `hachimi-plugin-abi`, plus `Sdk`, `ApiVersion`, and safe wrappers (`gui`, `il2cpp`, `hook`).

```toml
[dependencies]
hachimi-plugin-abi = { path = "../../crates/hachimi-plugin-abi" }
hachimi-plugin-sdk = { path = "../../crates/hachimi-plugin-sdk" }
```

```rust
#[macro_use]
extern crate hachimi_plugin_abi;

use hachimi_plugin_abi::{InitResult, API_VERSION, Vtable};
use hachimi_plugin_sdk::{init_result_to_i32, InitError, Sdk};

#[no_mangle]
pub extern "C" fn hachimi_init(vtable_ptr: *const std::ffi::c_void, version: i32) -> i32 {
    // SAFETY: Host passes a valid vtable at load.
    match unsafe { Sdk::init_min(vtable_ptr as *const Vtable, version, API_VERSION) } {
        Ok(()) => {
            Sdk::get().show_notification("My plugin loaded");
            init_result_to_i32(InitResult::Ok)
        }
        Err(InitError::HostApiTooOld { required, actual }) => {
            hlog_error!("need host API v{required}, got v{actual}");
            init_result_to_i32(InitResult::Error)
        }
        Err(_) => init_result_to_i32(InitResult::Error),
    }
}
```

## Versioning

- **`API_VERSION`** in `hachimi-plugin-abi` is the layout your plugin was built against.
- Check compatibility **once at init** with `Sdk::init_min(ptr, version, MIN_HOST_API)` or `ApiVersion::new(version).at_least(MIN)`.
- Do **not** gate individual SDK calls with per-feature `supports_*()` helpers — use host return values (`false` from `register_overlay`, etc.) and logging instead.
