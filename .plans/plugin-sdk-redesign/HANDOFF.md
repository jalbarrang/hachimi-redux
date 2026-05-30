## Goal
Redesign the Hachimi native plugin architecture to adopt pi-extension *patterns* over a stable C ABI. Breaking changes are acceptable (sole author of host + the one plugin). Bump to a clean `API_VERSION`.

## Decisions (locked via questionnaire)
- **GUI**: share the real `egui::Ui` across the FFI boundary. Remove the ~15 `gui_ui_*` widget slots. Plugins write normal egui code. Safe because the whole repo is one cargo workspace → egui 0.33.3 is already lockstep. `abi` stays egui-free (opaque `*mut c_void`); the typed cast happens host-side and in the SDK.
- **Events**: dynamic `host_subscribe(event_id, callback, userdata) -> handle` + `host_unsubscribe(handle)`. Extensible without new slots. Initial events: `Frame=1`, `ConfigReload=2`, `Shutdown=3`.
- **Scope**: build the full foundation now (macro + manifest + events + capability flags + registration handles + egui-share), converting `training-tracker`.

## Key facts
- Workspace members: `.`, `crates/hachimi-plugin-abi`, `crates/hachimi-plugin-sdk`, `plugins/training-tracker`. Shared Cargo.lock.
- Host depends on `abi` only; plugins depend on `abi` + `sdk`.
- Host already passes real `&mut egui::Ui` as `*mut c_void` at `src/core/gui/overlays.rs:49` and `src/core/gui/menu.rs:233`.
- Event dispatch points: `run_overlays` (`src/core/gui/overlays.rs:10`) = Frame; `Hachimi::reload_config`/`save_and_reload_config` (`src/core/hachimi/mod.rs:194/215`) = ConfigReload; `DLL_PROCESS_DETACH` (`src/windows/main.rs:74`) = Shutdown.
- Current Vtable: il2cpp 24, gui 22, interceptor 4, hachimi 2, log 1 = 53 slots. Keep il2cpp/interceptor/hachimi/log + 7 gui registration slots; remove 15 widget slots; add subscribe/unsubscribe/capabilities/gui_unregister.
- Loader: `src/windows/main.rs::load_libraries` resolves `hachimi_init`. Will also resolve `hachimi_plugin_manifest` for pre-init introspection/validation.

## New ABI surface (abi crate)
- `PluginManifest { abi_version: i32, name, version: *const c_char, min_host_api: i32, requested_caps: u64 }` (all 'static).
- New plugin export: `hachimi_plugin_manifest() -> *const PluginManifest`.
- `PluginEventFn = extern "C" fn(event_id: u32, data: *const c_void, userdata: *mut c_void)`.
- Vtable additions: `host_subscribe`, `host_unsubscribe`, `host_capabilities -> u64`, `gui_unregister(handle: u64)`. GUI registration slots now return `u64` handle (0 = fail).
- Capability bitflags (u64): GUI, OVERLAY, EVENTS, IL2CPP.

## SDK additions
- `pub use egui;` (add egui dep, workspace 0.33.3) + `gui::ui_from_ptr(ptr) -> &mut egui::Ui`.
- `Sdk::on(event, cb) -> Subscription`, `Sdk::capabilities()/has_capability()`, registration methods return handles, `pub use hachimi_plugin_macros::hachimi_plugin;`.

## New crate
- `crates/hachimi-plugin-macros` (proc-macro): `#[hachimi_plugin(name=, version=, min_api=)]` on `fn(&Sdk) -> Result<(), InitError>` generates `hachimi_plugin_manifest` + `hachimi_init` shims.

## Verification gate
After each phase: `cargo build` (workspace) + `cargo build -p hachimi-training-tracker`. Final: clippy + fmt + `cargo test`. Update `docs/architecture.md` plugin section. Track via beads.
