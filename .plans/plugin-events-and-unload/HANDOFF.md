## Goal
Work `Hachimi-Edge-6yd`: (1) add game-lifecycle events so plugins stop re-implementing IL2CPP hooks, and (2) build a plugin unload/reload path. User chose: implement BOTH now; events = VIEW_CHANGE, CAREER_START/END, TRAINING_COMMAND, SPLASH_SHOWN; payloads as versioned `#[repr(C)]` structs; demonstrate by migrating training-tracker off its own hooks where covered.

## Key design facts (from reading the repo)
- Events are additive: new event-id constants need **no** new vtable slot and **no** `API_VERSION` bump (stays 9/42). This proves the v9 extensibility claim. `abi_layout.rs` stays unchanged.
- Existing host dispatch points found:
  - `src/il2cpp/hook/umamusume/SceneManager.rs::ChangeViewCommon(next_view_id)` — already hooks view changes and already detects splash (`next_view_id == 1`). Wire VIEW_CHANGE + SPLASH_SHOWN here. (Windows-only module — acceptable, project is Windows-centric.)
  - `SingleModeMainViewController.SendCommandAsync(6)` arg1 = command_id (runtime-verified 2026-05-23, per docs/reverse-engineering/single-mode-architecture.md). The plugin currently hooks this in `plugins/training-tracker/src/hooks.rs`. Move it host-side → emit TRAINING_COMMAND, then drop the plugin's hook (avoids double-hooking the same address).
  - CAREER_START/END: no verified single hook point exists. Use the confirmed `WorkDataManager` singleton → `get_SingleMode()` → `get_IsPlaying()` chain (confirmed 2026-05-24), polled from the existing FRAME dispatch (render thread, same context the plugin's memory_reader already uses). Lazy-resolve + cache method addrs; on resolve failure, silently disable career events. Throttle (~ every N frames). Low blast radius: if wrong, events just don't fire and the plugin keeps working.
- Payload structs live in `hachimi_plugin_abi`: `ViewChangeEvent { view_id: i32 }`, `TrainingCommandEvent { command_id: i32 }`. CAREER_*/SPLASH carry null data.
- Dispatch helpers go in `src/core/plugin/events.rs` next to existing `dispatch_frame/config_reload/shutdown`. Core may call `crate::il2cpp` (api.rs already does).

## Unload/reload reality (Phase C)
Native cdylib hot-reload is fundamentally unsafe if the plugin installed IL2CPP hooks: `FreeLibrary` unmaps the plugin's code while the game still holds trampolines pointing into it → crash on next game call. The host cannot know what a plugin hooked. Therefore:
- Implement **owner-scoped** GUI registrations + event subscriptions (tag each handle with an owning plugin id) so the host can reclaim a plugin's callbacks before unmap (this is the real safety fix for the GUI/event side).
- Store `HMODULE` per `Plugin`.
- Implement a guarded `unload_plugin`: dispatch SHUTDOWN to that plugin's subscriptions, reclaim its GUI+event handles, then `FreeLibrary`.
- Document the contract: a plugin that installs IL2CPP hooks MUST unhook them in its SHUTDOWN handler or it is unsafe to FreeLibrary. Add unit tests for owner-scoped reclaim.

## Verification gate (after each change)
`cargo build`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt`, `cargo test`. Ignore fmt nightly-import warnings. Never launch the game.

## Files
ABI: `crates/hachimi-plugin-abi/src/lib.rs`. Events: `src/core/plugin/events.rs`, `mod.rs`. Dispatch sites: `src/il2cpp/hook/umamusume/SceneManager.rs`, new `SingleModeMainViewController.rs` (+ register in `umamusume/mod.rs`), new career watcher. Host vtable: `src/core/plugin/api.rs`. Loader/unload: `src/windows/main.rs`, `src/core/hachimi/mod.rs`. Plugin: `plugins/training-tracker/src/{lib,hooks,tracker}.rs`. Docs: `docs/architecture.md`.
