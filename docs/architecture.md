# Architecture

- **Core** (`src/core/`): Platform-agnostic — GUI (egui), plugin API, IL2CPP interceptor, game logic hooks
- **Windows** (`src/windows/`): DX11 render hook, window hook, DLL proxy, Steam integration
- **Android** (`src/android/`): Parallel platform impl — changes to render hook logic often need mirroring here
- **Plugins** (`plugins/`): External cdylib crates loaded at runtime via `load_libraries` in config.json
- **Plugin API** (`src/core/plugin_api.rs`): Flat C ABI vtable struct. **Field order is ABI** — new functions must be appended at the end only. Version field gates access to newer entries.
