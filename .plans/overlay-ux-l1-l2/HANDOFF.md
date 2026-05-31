## Goal (beads: Hachimi-Edge-zuv)
Replace the cramped left `SidePanel` menu with a two-layer UI **framework**. Ship the chrome + SDK only — **no** feature/data tabs (stats/skills/race/shop), **no** hidden-data reads (energy, friendship%), no race HUD. Those come later as plugin pages/panels once the framework exists.

## Converged design (from a long design conversation + user mockups)
- **L1 — Control Center**: an `egui::Modal` (confirmed present in egui 0.33.3 `containers/modal.rs`), **hotkey-toggled** via the existing `menu_open_key` (currently `V`). No edge bookmark. Fixed top tabs (horizontal `selectable_label`/`toggle_value` row): **Settings · Plugins · Overlay · About**.
  - *Settings*: migrate the existing Hachimi config UI here.
  - *Plugins*: a sub-list/sub-nav of plugin-registered L1 pages (scales to N plugins without N top-level tabs).
  - *Overlay*: manage L2 panels — per-panel show/hide, the global **lock** toggle, opacity slider.
  - *About*: existing about content.
- **L2 — floating HUD**: evolve the existing `gui_register_overlay` → `egui::Window` system into a draggable **badge ↔ panel** (collapsed badge = `title_bar(false)`/headerless; expanded = panel), `Window.movable(true)`, **global lock** (when locked, panels are click-through), positions **persisted**.
- **SDK**: add clear `Sdk::register_page` (L1) and `Sdk::register_panel` (L2) wrappers over the **existing vtable slots** (`gui_register_menu_section_with_icon` / `gui_register_overlay`). **No ABI break** — `API_VERSION` stays 9, `VTABLE_SLOT_COUNT` stays 42, abi-layout test unchanged.
- Host **dogfoods** the registries: Settings/Overlay/About are host-registered L1 content; any built-in HUD is a host-registered L2 panel.

## egui 0.33.3 feasibility (verified against installed source)
- `egui::Modal` ✅ (dimmed backdrop + centered + input capture)
- `Window`: `.movable()`, `.anchor(Align2,..)`, `.resizable()`, `.collapsible()`, `.title_bar()` ✅
- `Area` ✅, `Slider` ✅, `selectable_label`/`toggle_value` ✅
- **No built-in switch** — implement the canonical ~25-line `toggle_ui` widget (only `Checkbox`/`RadioButton` ship). This is the one custom widget.

## Input/z-order contract (the thing that makes it not-in-the-way)
- **L1 open** → `egui::Modal` captures all input + dims; game gets nothing.
- **L1 closed, L2 present** → input hook gates on `ctx.wants_pointer_input()` / pointer-over-area; block the game's mouse **only** when the cursor is over an L2 panel, else clicks fall through to the game. Locked L2 = display-only/click-through.
- Investigate the current capture path first: how `Gui.show_menu` is toggled by `menu_open_key`, and where the render/wnd hook decides to swallow game input (`src/windows/`, `src/core/gui/{frame,instance}.rs`, `wnd_hook`).

## Key files
- L1/menu: `src/core/gui/menu.rs` (currently `SidePanel::left` + `ScrollArea` — replace with `Modal`), `src/core/gui/mod.rs`, `src/core/gui/window/{about,config_editor,...}.rs` (existing settings/about content to migrate/reuse).
- L2: `src/core/plugin/overlay.rs` (registry — already has owner-scoped handles), `src/core/gui/overlays.rs` (render — currently `egui::Window` per overlay).
- Menu/plugin sections registry: `src/core/plugin/menu.rs` (sections → L1 pages).
- SDK: `crates/hachimi-plugin-sdk/src/sdk.rs` (+ `gui.rs`); ABI unchanged (`crates/hachimi-plugin-abi`).
- Host vtable impls: `src/core/plugin/api.rs` (already wires `gui_register_menu_section_with_icon` / `gui_register_overlay`).
- Test plugin: `plugins/training-tracker/src/ui.rs` (registers a menu section + an overlay today → becomes L1 page + L2 panel).
- Docs: `docs/architecture.md` (Plugin model section), `docs/reverse-engineering/il2cpp-signatures.md` (unrelated, for reference).

## Constraints / conventions
- After each change: `cargo build`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt`, `cargo test`. Ignore fmt nightly-import warnings.
- Deploy ONLY via `scripts/deploy-windows.ps1` (game must be closed; never launch or kill the game). Current install: proxy at game-root `cri_mana_vpx.dll`, forward target `hachimi/cri_mana_vpx.dll`, config `hachimi/config.json` (`menu_open_key:86`).
- Keep files small; extract pure logic; preserve owner-scoped handle/teardown behavior (plugin unload/reload still works).
- Reuse existing config fields where possible; persist L2 positions + lock + opacity (config or a ui-state file).

## Out of scope (explicitly defer)
Data/feature tabs, hidden-data IL2CPP reads, race HUD, any new ABI slots. If a new slot is ever needed, that's a separate `API_VERSION` bump issue.

## Definition of done
Modal opens/closes on hotkey with 4 working tabs; existing settings reachable in Settings; training-tracker shows up as a page under Plugins AND as a draggable, lockable L2 badge/panel listed in Overlay; locked panels are click-through; positions persist; SDK exposes `register_page`/`register_panel` with docs; full gate green; deployed.