# Drop Android Support and Keep Windows-Only Code Paths

## Goal
Convert this fork from cross-platform Windows/Android to Windows-only. Remove Android build/deploy/runtime code and simplify shared code that only exists to support Android. Keep Windows behavior and config shape stable.

## User decisions
- **Plugin ABI:** remove Android DEX vtable slots and bump `API_VERSION`. This is an intentional ABI break for a Windows-only fork.
- **Docs scope:** code/build cleanup first. Public README/docs/locales cleanup is deferred to a separate follow-up.
- **Config shape:** keep existing nested `windows` config keys. Do not flatten Windows settings into top-level config in this change.

## Hard constraints
- Do not launch the game.
- Do not kill game processes.
- Do not modify `cri_mana_vpx.dll.backup` in the game directory.
- After code changes: run rustfmt, clippy, and tests without launching the game.

## Primary removal targets

### Android runtime/platform tree
Delete the whole `src/android/` tree:
- `src/android/mod.rs`
- `src/android/main.rs`
- `src/android/hook.rs`
- `src/android/game_impl.rs`
- `src/android/hachimi_impl.rs`
- `src/android/interceptor_impl.rs`
- `src/android/log_impl.rs`
- `src/android/symbols_impl.rs`
- `src/android/utils.rs`
- `src/android/dex_bridge.rs`
- `src/android/plugin_loader.rs`
- `src/android/gui_impl/*`
- `src/android/zygisk/*`

### Android tooling
Delete `tools/android/`:
- build/dev scripts
- Zygisk template
- Android README

### Build/dependency configuration
- `Cargo.toml`: remove `[target.'cfg(target_os = "android")'.dependencies]` section and Android-only crates: `libc`, `android_logger`, `procfs`, `jni`, `egui_glow`, `glow`, `dobby-rs`.
- `build.rs`: remove Android linker-arg branch (`-z max-page-size`, `-z common-page-size`).
- `deny.toml`: remove `aarch64-linux-android` from `[graph].targets`.
- `Cargo.lock`: update through Cargo after dependency removal.

### CI/release configuration
- `.github/workflows/ci.yml`: remove `clippy-android` job and Android target setup. Ensure downstream `needs` do not reference removed job.
- `.github/workflows/create_release.yml`: remove `build-android` job, remove Android artifacts, and change `create-release.needs` from `[build-android, build-windows]` to `[build-windows]`.

## Shared Rust code hotspots

### `src/lib.rs`
Remove Android module/use block:
- `#[cfg(target_os = "android")] mod android;`
- `#[cfg(target_os = "android")] use android::{...};`
Keep Windows block unchanged.

### `src/core/hachimi.rs`
Simplify platform config/data structures:
- Remove `Config.android` flattened field.
- Keep `Config.windows` nested field.
- Simplify `OsOption<T>` to Windows-only. Options:
  - minimal: keep struct with only `windows: Option<T>` and `as_ref()` returning `self.windows.as_ref()`;
  - deeper: replace uses with `Option<T>` later, but avoid broad config schema churn in this change.
- Remove `.nomedia` creation in localized data setup.
- Simplify `AssetInfo<T>` to hold only `windows: AssetMetadata` plus `data`.
- Simplify `metadata()`/`metadata_ref()` to return Windows metadata.

### `src/core/utils.rs`
Simplify `get_data_path()` to the Windows branch only. Remove Android `/data/data/{package}/files` branch.

### `src/core/updater.rs`
Remove Android update flow:
- Android prompt using `update_prompt_dialog.android_content`.
- Android `run()` branch using `android::utils::open_app_or_fallback` and `UMAPATCHER_*` constants.
Keep Windows installer asset/hash/download/run flow.

### `src/core/gui/*`
Remove Android IME/touch/notch handling and delete `src/core/gui/android_keyboard.rs`.
Files with Android references:
- `src/core/gui/mod.rs`: remove module/export and `ime_cooldown` field.
- `src/core/gui/instance.rs`: remove `ime_cooldown` initialization.
- `src/core/gui/frame.rs`: remove Android `orientation_scale = 1.0` branch and Android keyboard/back-button management; keep Windows IME composition block.
- `src/core/gui/menu.rs`: remove Android imports, Android notch header branch, `WebViewManager` branch, Android IME padding, and `handle_android_keyboard` call.
- `src/core/gui/window/config_editor.rs`: remove Android keyboard imports, `handle_android_keyboard`, and Android IME padding.
- `src/core/gui/window/first_time_setup.rs`: same Android keyboard/padding cleanup.

### Plugin ABI and host API
Intentional ABI break:
- `crates/hachimi-plugin-abi/src/lib.rs`: remove vtable fields:
  - `android_dex_load`
  - `android_dex_unload`
  - `android_dex_call_static_noargs`
  - `android_dex_call_static_string`
- `crates/hachimi-plugin-abi/src/version.rs`: bump `API_VERSION` from `7` to `8`; reduce `VTABLE_SLOT_COUNT` from `57` to `53`.
- `crates/hachimi-plugin-abi/tests/abi_layout.rs`: update expected constants.
- `crates/hachimi-plugin-abi/README.md`: code-adjacent package doc should be updated to slot count/version even though broader docs are deferred.
- `src/core/plugin/api.rs`: remove Android DEX host functions and remove those four entries from `build_host_vtable()`.
- Also remove Android-specific keyboard call in `gui_ui_text_edit_singleline`.

### IL2CPP hooks/modules
Remove Android-only hook modules and cfg branches:
- `src/il2cpp/hook/mod.rs`: remove `Cute_Core_Assembly` module declaration and `Cute_Core_Assembly::init()` call.
- Delete `src/il2cpp/hook/Cute_Core_Assembly/` if it is only Android-gated.
- `src/il2cpp/hook/UnityEngine_CoreModule/mod.rs`: remove Android-only `TouchScreenKeyboard` and `TouchScreenKeyboardType` module declarations and init calls.
- Delete `src/il2cpp/hook/UnityEngine_CoreModule/TouchScreenKeyboard.rs` and `TouchScreenKeyboardType.rs` if unused after removal.
- `src/il2cpp/hook/umamusume/GameSystem.rs`: remove Android `set_audio_capture_policy_all()` call.
- `src/il2cpp/hook/umamusume/Screen.rs`: Android orientation hook code can be removed. If file has no Windows logic after cleanup, leave a minimal `pub fn init() {}` only if `umamusume::mod.rs` still calls it.
- `src/il2cpp/hook/umamusume/UIManager.rs`: remove Android resolution scaler branch and Android `WaitBootSetup` hook.

### Generated/shared IL2CPP names: be conservative
Do **not** eagerly delete generic generated symbols/types just because their names include Android:
- `src/il2cpp/api.rs`: `il2cpp_unity_set_android_network_up_state_func` may be generated/shared.
- `src/il2cpp/types.rs`: Unity Android-specific structs may be generated/shared.
- `src/windows/symbols_impl.rs`: symbol string `il2cpp_unity_set_android_network_up_state_func` may exist in Unity exports.
Only remove these if a post-cleanup search proves they are unused and Windows build/tests pass.

## Deferred follow-up (not this code/build cleanup)
Public docs/locales still mention Android and should be cleaned in a separate change:
- `README.md`, `README-zh_cn.md`, `README-zh_tw.md`
- `docs/architecture.md`, `docs/patterns.md`, `docs/plugin-sdk.md`, `docs/reverse-engineering/hachimi-plugin-surface.md`, possibly old `docs/plans/*`
- `plugins/training-tracker/README.md`
- `assets/locales/*.yml` Android control/update strings and `android:` sections

## Validation commands
Run after implementation, without launching the game:
- `cargo fmt --check`
- `cargo clippy --target x86_64-pc-windows-msvc --all-targets -- -D warnings`
- `cargo test --lib`
- `cargo test -p hachimi-plugin-abi`
- `cargo test -p hachimi-training-tracker --lib`
- Optional: `cargo deny check` and `cargo machete` after dependency cleanup.

## Gotchas
- Removing Android DEX slots shifts later vtable fields. This is okay only because user chose an ABI break. Bump API/version constants and tests together.
- Keep `windows.load_libraries` config shape stable. Do not flatten config in this change.
- `docs_scope=code_first`: do not spend implementation time on broad public docs/locales cleanup, except package/version docs needed to keep ABI metadata honest.
- `cargo machete` may reveal newly-unused dependencies after Android removal.
- Some Windows code intentionally references strings with `android` in Unity/IL2CPP generated names; treat as generated/runtime export data, not Android platform support by default.
