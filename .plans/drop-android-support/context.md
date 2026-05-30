# Drop Android Support - Planning Context

## Intent
- Fork should become Windows-only.
- Goal now: map files and code logic to remove or simplify, not edit product code in plan mode.

## Constraints
- Plan mode active: read-only investigation, no product code edits.
- Hard repo rule: never launch game, never kill game processes, never modify backup DLL.
- Future implementation must run rustfmt + clippy; test without launching game.

## Current architecture
- `src/windows/`: Windows platform implementation, should stay.
- `src/android/`: Android platform implementation, primary deletion target.
- `src/core/`: shared logic with scattered `#[cfg(target_os = "android")]` branches.
- `src/il2cpp/`: shared IL2CPP bindings/hook logic with some Android-only hooks/classes.
- `crates/hachimi-plugin-abi`: plugin ABI includes Android DEX slots currently no-op on Windows.
- `tools/android/`: Android build/deploy/zygisk tooling.
- `.github/workflows/`: CI/release still build/lint Android.

## Candidate removals found
- Delete `src/android/` whole tree.
- Delete `src/core/gui/android_keyboard.rs` and Android references in `src/core/gui/*`.
- Remove Android target dependencies from `Cargo.toml`: `libc`, `android_logger`, `procfs`, `jni`, `egui_glow`, `glow`, `dobby-rs`.
- Remove Android linker args from `build.rs`.
- Remove Android target from `deny.toml` graph.
- Remove `tools/android/` whole tree.
- Remove Android CI/release jobs and artifacts.
- Update docs/readmes/plugin docs to Windows-only.
- Update locale strings that describe Android controls/update prompt.

## Code logic hotspots
- `src/lib.rs`: remove Android module/use block, keep Windows block.
- `src/core/hachimi.rs`: remove Android flattened config, simplify `OsOption<T>` and `AssetInfo<T>` to Windows-only, remove `.nomedia` creation.
- `src/core/utils.rs`: simplify `get_data_path()` to Windows path only.
- `src/core/updater.rs`: remove Android update prompt/install path, keep Windows installer flow.
- `src/core/plugin/api.rs`: remove Android keyboard hook in text input; decide whether Android DEX vtable slots stay as permanent Windows no-ops or ABI-breaking removal.
- `src/core/gui/mod.rs`, `instance.rs`, `frame.rs`, `menu.rs`, `window/config_editor.rs`, `window/first_time_setup.rs`: remove Android IME/notch/UI branches.
- `src/il2cpp/hook/mod.rs`: remove Android-only `Cute_Core_Assembly` module/init.
- `src/il2cpp/hook/UnityEngine_CoreModule/mod.rs`: remove Android-only `TouchScreenKeyboard*` modules/init.
- `src/il2cpp/hook/umamusume/GameSystem.rs`: remove Android audio capture policy call.
- `src/il2cpp/hook/umamusume/Screen.rs`: likely delete all Android-only orientation hooks, may leave empty init if Windows has no logic.
- `src/il2cpp/hook/umamusume/UIManager.rs`: remove Android boot/setup hook and Android resolution scaler branch.
- `src/il2cpp/api.rs`, `src/il2cpp/types.rs`, `src/windows/symbols_impl.rs`: contain Unity Android API names/types generated/shared; optional cleanup only if safe.

## Open decisions
- Plugin ABI: keep Android DEX vtable fields as no-op for backwards-compatible ABI, or remove and bump `API_VERSION`/SDK docs.
- Config compatibility: keep accepting `windows` key only; no need to accept `android`. Decide if `OsOption` should remain a Windows-key helper or become plain `Option<T>`.
- Docs scope: update public READMEs and plugin docs now, or code-only cleanup first.

## Discarded/risky
- Do not remove generic IL2CPP generated Android names from `types.rs`/`api.rs` unless proven unused; generated bindings may include platform-neutral Unity symbols with Android in name.
