# Control Center tab bodies

Each top-level tab has its own module here, laid out with **egui_taffy** (flex/grid via [`layout.rs`](layout.rs)).

| Tab | File | Entry point |
|-----|------|-------------|
| General | `general.rs` | `options` + `overlays` |
| Graphics | `graphics.rs` | `options` |
| Gameplay | `gameplay.rs` | `options` |
| Hotkeys | `hotkeys.rs` | `ui_hotkeys` |
| Translations | `translations.rs` | `run_translations_tab` (Gui) + `options` (grid) |
| Plugins | `plugins.rs` | `run_plugins_tab` (Gui) |
| About | `about_tab.rs` | `run_about_tab` (Gui) |

**Dispatch:** `menu.rs` → `ConfigEditor::ui_body` / `ui_translations` for config tabs; `Gui::run_*_tab` for Plugins/About (and Translations actions). Shell chrome (header, tab bar, footer) stays in [`menu.rs`](../menu.rs).

**Layout kit:** `settings_grid` for label+control rows; `flex_row` / `flex_wrap` / `flex_col` for button clusters. Width is pinned via `content_width` (shell width minus the vertical scrollbar reservation) so scroll areas don't stretch the modal and inner elements don't bleed past the tab body.
