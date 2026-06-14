//! Plugin API version and vtable slot count.

/// Current plugin API version passed to `hachimi_init` alongside the vtable pointer.
///
/// v9 redesign: removed per-widget GUI slots (plugins now draw with the shared
/// `egui::Ui`), added `host_subscribe`/`host_unsubscribe`/`host_capabilities`/
/// `gui_unregister`, registration slots return handles, and plugins export a
/// `hachimi_plugin_manifest`.
///
/// v10: added `host_data_path` (resolve paths under the game data dir) and the
/// `capability::DATA_PATHS` bit, enabling plugins to locate host-cached data
/// such as the GameTora snapshots.
///
/// v11: added `host_view_name` (resolve a `Gallop.SceneDefine.ViewId` to a
/// host-owned label), letting plugins display view names without their own catalog.
///
/// v12: added `gui_register_overlay_ex` (register an L2 panel with presentation
/// flags, e.g. [`crate::overlay_flags::CHROMELESS`] to drop the host window chrome
/// so the plugin's own visuals float bare).
///
/// v13: added `host_register_hotkey` (register a named hotkey action into the
/// host's central Hotkeys tab; the user rebinds it there and the host persists the
/// chord). Unregister via `gui_unregister`.
///
/// v14: added `gui_overlay_get_visible` (query an overlay's current visibility),
/// letting plugins implement a toggle alongside `gui_overlay_set_visible`.
pub const API_VERSION: i32 = 14;

/// Number of function pointers in `Vtable`.
pub const VTABLE_SLOT_COUNT: usize = 47;
