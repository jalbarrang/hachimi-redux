# AGENTS.md

## What this is

Fork of Hachimi: a Windows/Steam translation & enhancement mod for the Honse game (no Android). The core ships as `hachimi.dll`, deployed into the game dir as a `cri_mana_vpx.dll` proxy that hooks IL2CPP. Optional plugins are separate cdylibs loaded via `config.json`. The authoritative crate/tool list is the root `Cargo.toml` `members` — don't restate it elsewhere. Dependency direction: `apps/*` and `plugins/*` depend on `crates/*`, never the reverse.

## Stack

| Area | Tech |
|---|---|
| Language | Rust, edition 2021, stable toolchain |
| Build | Cargo workspace (root `Cargo.toml`) |
| Platform | Windows + Steam only |
| UI authoring | Dioxus 0.7 (`macro`/`html`/`signals`/`hooks`) + `dioxus-egui` + `honse-ui` |
| UI render | egui 0.34 + `egui_taffy` (=0.12.0) → `egui-directx11` (paint) |
| Hooking | `minhook`, `pelite`, `windows` crate; IL2CPP interop |
| egui source | git-pinned in `[patch.crates-io]` (egui / egui_extras / egui-directx11) |

## Commands

| Task | Command |
|---|---|
| Build core | `cargo build --release -p hachimi` → `hachimi.dll` |
| Build a plugin | `cargo build --release -p hachimi-training-tracker` (or `-debug-viewer` / `-race-hud`) |
| Build installer | `cargo build --release -p hachimi_installer` (only after artifacts staged) |
| Menu preview (no game) | `cargo run -p hachimi --example menu_preview --features dev-harness` |
| Deploy to game | `.\scripts\deploy-windows.ps1 -Build` (`$env:HACHIMI_GAME_DIR` overrides path) |
| Hot-swap plugin | `.\scripts\deploy-windows.ps1 -PluginOnly -HotSwap` (needs `enable_ipc` in config) |
| Local CI gates | `.\scripts\quality-gates.ps1` (fmt, cargo-deny, machete, clippy, check) |
| Format check | `cargo fmt --check` |
| Lint | `cargo clippy --all-targets -- -D warnings` (zero-warning) |
| Test | `cargo test --lib` (no game process required) |

## Rules

- **Never launch the game** (`steam://rungameid`, game executables, etc.). Copying DLLs is allowed.
- **Never kill game processes** (`taskkill`, etc.).
- **Never modify** `cri_mana_vpx.dll.backup` — the one-time backup of the stock `cri_mana_vpx.dll`.
- **Naming:** Never write the game's real, spelled-out name in prose, docs, comments, or commit messages. Always call it **"the Honse game"** (to avoid search-engine parsing). Exception: load-bearing identifiers that must match the game/OS — the `umamusume.dll` assembly name, `UmamusumePrettyDerby*` folder/exe/window-class names, CDN/API URLs, and the Rust `umamusume` module / IL2CPP class names — leave those exactly as-is.
- **Default members exclude `apps/installer`:** bare `cargo check/clippy/test/build` skip it. Build it explicitly with `-p hachimi_installer` after its embedded artifacts are staged.
- **Plugin ABI is versioned:** host API version is owned by `hachimi_plugin_abi::API_VERSION` (`crates/hachimi-plugin-abi/src/version.rs`). `capability::DIOXUS_UI` gates the Dioxus embed. Upstream Hachimi plugins do not load — only DLLs built from this repo.
- **License:** workspace is GPL-3.0-or-later; `apps/installer` overrides to MIT (`workspace.package` in root `Cargo.toml`).
- **Settings live in the game data directory:** `config.json` is in the game *data* dir, not the install folder. Plugins are enabled via its `windows.load_libraries`.
- **Lint floor is enforced in `Cargo.toml`:** `unsafe_op_in_unsafe_fn`, `undocumented_unsafe_blocks` warn; clippy `correctness` denied. Annotate unsafe blocks with safety comments.
- **`[patch.crates-io]` stays last in `Cargo.toml`:** the release workflow appends egui path patches to the file end; they must land under that table.
- **Per-crate fmt/clippy when editing a plugin:** run fmt/clippy inside `plugins/<plugin>/` for that crate too.
- **Confirm before third-party API code:** the egui/dioxus/taffy stack is git-pinned and version-sensitive — treat trained-in API knowledge as stale; check docs for the pinned versions.

## Key paths

```
apps/hachimi/         → core mod; builds hachimi.dll (deployed as cri_mana_vpx.dll)
  src/il2cpp/         → IL2CPP class/method hooks and interop
  src/windows/        → Win32 proxy, D3D11, input/IME, window hooks
  src/core/gui/       → Control Center: egui shell + Dioxus tabs/windows
  src/core/gui/dioxus → Dioxus app, tabs, context bridge to egui
  src/core/tl_repo/   → translation repo formats + updater
apps/installer/       → standalone installer (MIT); built separately
crates/dioxus-egui/   → Dioxus → egui_taffy renderer (the DOM-to-egui bridge)
crates/honse-ui/      → themed component kit authored against dioxus-egui
crates/hachimi-plugin-abi/    → versioned host/plugin ABI (API_VERSION)
crates/hachimi-plugin-sdk/    → plugin authoring SDK
crates/hachimi-plugin-macros/ → plugin proc-macros
crates/hachimi-telemetry/     → telemetry
plugins/              → cdylibs: training-tracker, debug-viewer, race-hud
tools/                → dev data tools (see tools/README.md + docs/updating-game-data.md)
scripts/              → deploy-windows.ps1, quality-gates.ps1, bump-version.ps1, ...
```

## Gotchas

- **Dioxus attrs map to taffy styles, not CSS.** In `crates/dioxus-egui/src/style.rs`, `align` sets cross-axis `align_items`; horizontal grid-item fill needs `justify-items`. A single-line `<input>` only fills its column when the grid item is stretched *and* the widget uses `desired_width(f32::INFINITY)` — both are required.
- **Game data refresh is a documented sequence,** not ad-hoc tool runs: follow `docs/updating-game-data.md` (`fetch-master-db`, `skill-grades`, `course-data`, `tracker-data-manifest`, `gametora-sync`).
- **`menu_preview` is dev-only,** behind the `dev-harness` feature; never built by CI/default. Plugins/About tabs are stubs there.
