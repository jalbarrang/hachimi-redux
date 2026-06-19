# Control Center tab bodies

The Control Center shell is authored in Dioxus under [`../dioxus/`](../dioxus/). Tab modules live in [`../dioxus/tabs/`](../dioxus/tabs/).

| Tab | Dioxus module | Notes |
|-----|---------------|-------|
| General | `dioxus/tabs/general.rs` | Full rsx grid |
| Graphics | `dioxus/tabs/graphics.rs` | VSync row uses `native="egui"` |
| Gameplay | `dioxus/tabs/gameplay.rs` | Full rsx grid |
| Hotkeys | `dioxus/tabs/hotkeys.rs` | `native="egui"` → `tabs/hotkeys.rs` |
| Translations | `dioxus/tabs/translations.rs` | Grid + native action strip |
| Plugins | `dioxus/tabs/plugins.rs` | `native="egui"` → `menu.rs` |
| About | `dioxus/tabs/about.rs` | `native="egui"` → `menu.rs` |

**Dispatch:** `menu.rs` → `shell::render_control_center_gui` → `control_center_mount` embeds the Dioxus VDOM each frame. Preview: `dev_harness.rs` → `render_control_center_preview`.

**Legacy:** imperative helpers in this folder (`general.rs`, `graphics.rs`, …) remain for reference during migration; the live path uses `dioxus/tabs/*`.

**Layout kit:** `dioxus/tabs/layout.rs` (`SettingsGrid`, `LabelCell`, …). Shared widgets: [`honse-ui`](../../../../../../crates/honse-ui/) + [`components/`](../components/) (legacy).
