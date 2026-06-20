---
name: "honse-ui Image slot + Control Center header logo"
overview: "Add a generic `Image` component to honse-ui that emits the renderer's already-supported `<img>` element, register the bundled app icon bytes under a stable `bytes://` URI at GUI init, and render the icon in the Dioxus Control Center header (restoring the logo lost in the egui→honse-ui migration)."
base_commit: "0541582ef3667b02f6ccf43f21aaeb3dcaf38c14"
todo:
  - id: "image-1"
    task: "Add `Image` component to honse-ui (src/image.rs) and export it from lib.rs"
    status: done
  - id: "image-2"
    task: "Register the bundled icon bytes under a `bytes://` URI at both GUI init sites (instance.rs live, dev_harness.rs preview)"
    status: done
  - id: "image-3"
    task: "Render the icon in the Control Center header in dioxus/app.rs"
    status: done
  - id: "image-4"
    task: "Add `[img src=…]` to the renderer dump + a headless honse-ui test for Image, and add Image to the gallery"
    status: done
---

# honse-ui Image slot + Control Center header logo

## Goal
Restore the Hachimi logo in the Control Center header by giving honse-ui a generic
`Image` component, and feed it the bundled `icon.png` through a registered
`bytes://` URI.

## Context
- Prior turn established: the header (`apps/hachimi/src/core/gui/dioxus/app.rs`)
  renders only title text + version + close button. The logo exists in the
  startup splash (`Gui::icon`) and the About tab (`Gui::icon_2x`) but not the header.
  honse-ui has no image primitive, which is why the logo was dropped in the migration.
- Module roots: `crates/honse-ui/`, `crates/dioxus-egui/`, `apps/hachimi/src/core/gui/`.
- No dependency on other in-flight slices.

### What exists (verified on disk at base commit)

**The renderer already supports `<img>`.** No renderer change is required to *draw* an image:

```638:650:crates/dioxus-egui/src/renderer.rs
    /// Render an `<img src="…" width="…" height="…">` via egui's image widget.
    fn walk_img(&self, attrs: &[(String, String)], tui: &mut Tui) {
        let src = Self::attr(attrs, "src").unwrap_or("");
        let w = Self::attr(attrs, "width").and_then(|v| v.parse::<f32>().ok());
        let h = Self::attr(attrs, "height").and_then(|v| v.parse::<f32>().ok());
        tui.ui(|ui| {
            let mut img = egui::Image::new(src);
            if let (Some(w), Some(h)) = (w, h) {
                img = img.fit_to_exact_size(egui::vec2(w, h));
            }
            ui.add(img);
        });
    }
```

Dispatched by tag at `crates/dioxus-egui/src/renderer.rs:409`:
`"img" => self.walk_img(attrs, tui),`.

`egui::Image::new(src)` takes a `&str` URI and resolves it through egui's image
loaders. Loaders are installed at BOTH init sites:
- Live: `apps/hachimi/src/core/gui/instance.rs:29` — `egui_extras::install_image_loaders(&context);`
- Preview harness: `apps/hachimi/src/core/gui/dev_harness.rs:45` — `egui_extras::install_image_loaders(ctx);`

**The gap:** a string `src` only resolves if the bytes for that URI are already
registered. Plugin icons sidestep this with the tuple form
`egui::Image::new((uri, bytes))` (registers inline each frame —
`apps/hachimi/src/core/gui/tabs/plugins.rs:43,105`), but `walk_img` only has the
string URI. So for the bundled icon we must pre-register the bytes once under a
known `bytes://` URI via `egui::Context::include_bytes`.

**The icon asset:** `apps/hachimi/assets/icon.png`, embedded today via the
`include_image!` macro:

```5:5:apps/hachimi/src/core/gui/splash.rs
    const ICON_IMAGE: egui::ImageSource<'static> = egui::include_image!("../../../assets/icon.png");
```

`splash.rs` and `instance.rs` live in the same dir (`core/gui/`), so the relative
asset path `../../../assets/icon.png` is identical from both.

**honse-ui component pattern** (simplest existing example):

```8:14:crates/honse-ui/src/separator.rs
#[component]
pub fn Separator() -> Element {
    let bg = theme::LINE;
    rsx! {
        div { "height": "1", "bg": bg }
    }
}
```

Components are declared `mod x;` + `pub use x::X;` in `crates/honse-ui/src/lib.rs:30-53`.

**The header to change:**

```74:96:apps/hachimi/src/core/gui/dioxus/app.rs
            Card {
                div {
                    "dir": "row",
                    "gap": "8",
                    "align": "center",
                    div {
                        "color": theme::FG,
                        "font-size": "18",
                        "weight": "bold",
                        {hachimi_title}
                    }
                    div {
                        "color": theme::FG_MUTED,
                        "font-size": "12",
                        {env!("HACHIMI_DISPLAY_VERSION")}
                    }
                    div { "grow": "1" }
                    Button {
                        variant: ButtonVariant::Ghost,
                        onclick: bind_action(&actions, HostAction::CloseMenu),
                        "\u{f00d}"
                    }
                }
            }
```

## API inventory

```rust
// egui 0.34 (git rev 7288e4b) — Context image-bytes registration.
// Registers raw bytes under a URI so `egui::Image::new("bytes://…")` resolves.
impl egui::Context {
    pub fn include_bytes(&self, uri: impl Into<std::borrow::Cow<'static, str>>, bytes: impl Into<egui::load::Bytes>);
}

// egui::load::Bytes implements From<&'static [u8]>, so `include_bytes!(...)` works directly.

// dioxus-egui renderer: already routes the `img` tag through walk_img.
// Reads native attributes: src (string URI), width (f32), height (f32).

// honse-ui component contract (dioxus 0.7 rsx):
// #[component] pub fn Name(props…) -> Element
```

## Tasks

1. **Add the `Image` component** — create `crates/honse-ui/src/image.rs`.
   Keep it asset-agnostic: it only forwards a URI string + pixel size to the
   renderer's `<img>`. Width/height are `f32` props formatted into the native
   `width`/`height` attributes that `walk_img` parses.

```rust
//! Image — a fixed-size image slot. Forwards a URI (`src`) to the dioxus-egui
//! renderer's `<img>` element, which resolves it through egui's image loaders.
//! For embedded assets, register the bytes once with
//! `egui::Context::include_bytes("bytes://…", …)` and pass that URI as `src`.

use dioxus_egui::dioxus::prelude::*;

/// A fixed-size image. `src` is any egui-resolvable URI
/// (`bytes://…`, `file://…`, `https://…`).
#[component]
pub fn Image(src: String, width: f32, height: f32) -> Element {
    rsx! {
        img { src: "{src}", width: "{width}", height: "{height}" }
    }
}
```

   Then register + export it in `crates/honse-ui/src/lib.rs`: add `mod image;`
   (alphabetical, after `mod field;`) and `pub use image::Image;` (after
   `pub use field::Field;`).
   - **Verify:** `cargo build -p honse-ui` → builds clean.

2. **Register the icon bytes under a `bytes://` URI** — expose a shared URI const
   + registration helper, then call it at both init sites.
   In `apps/hachimi/src/core/gui/splash.rs`, add a module-level const and a helper
   (alongside the existing `ICON_IMAGE`):

```rust
/// Stable URI for the bundled app icon, registered into egui's byte loader at
/// GUI init so `<img src="bytes://hachimi-icon.png">` resolves in the Dioxus shell.
pub(crate) const ICON_URI: &str = "bytes://hachimi-icon.png";

pub(crate) fn register_icon_bytes(ctx: &egui::Context) {
    ctx.include_bytes(ICON_URI, include_bytes!("../../../assets/icon.png") as &[u8]);
}
```

   (If `register_icon_bytes` cannot be a free fn in `splash.rs` due to the `impl Gui`
   layout, make it `impl Gui { pub(crate) fn register_icon_bytes(ctx: &egui::Context) {…} }`
   and call `Gui::register_icon_bytes(...)` — match whatever keeps the existing
   `splash.rs` module style.)

   Call it right after each `install_image_loaders`:
   - `apps/hachimi/src/core/gui/instance.rs:29` — add `super::splash::register_icon_bytes(&context);` (or `Self::register_icon_bytes(&context);`).
   - `apps/hachimi/src/core/gui/dev_harness.rs:45` — add `crate::core::gui::splash::register_icon_bytes(ctx);` (or the `Gui::` form). Confirm the `splash` mod path is reachable from `dev_harness.rs`.
   - **Verify:** `cargo build --release -p hachimi` → builds clean. `cargo run -p hachimi --example menu_preview --features dev-harness` → launches without an image-load panic.

3. **Render the logo in the header** — in `apps/hachimi/src/core/gui/dioxus/app.rs`,
   add the `Image` import and place it as the first child of the header row (before
   the title `div`), inside the existing `div { "dir": "row", "align": "center" … }`:

```rust
honse_ui::Image {
    src: crate::core::gui::splash::ICON_URI.to_string(),
    width: 24.0,
    height: 24.0,
}
```

   Add `Image` to the `use honse_ui::{…}` import line at the top of `app.rs`
   (currently imports `Button, ButtonVariant, Card, TabBar, TabItem`). Keep the
   24px size consistent with the splash `Gui::icon` (24×24).
   - **Verify:** `cargo run -p hachimi --example menu_preview --features dev-harness` → the icon renders to the left of the "Hachimi" title in the header card.

4. **Testability: dump + test + gallery.**
   a. Extend the renderer's introspection dump so `<img>` is visible to headless
      tests. In `crates/dioxus-egui/src/renderer.rs`, in `dump_node` (the
      `match … NodeKind::Element { tag … }` arms around lines 668-688), add an arm:

```rust
NodeKind::Element { tag: "img", attrs, .. } => {
    let src = Self::attr(attrs, "src").unwrap_or("");
    out.push_str(&format!("[img src={src}]\n"));
}
```

   b. Add a headless test in `crates/honse-ui/tests/components.rs` (mirror the
      existing `components_render_and_buttons_are_clickable` pattern): render an
      `Image { src: "bytes://test.png", width: 24.0, height: 24.0 }` and assert
      `r.dump()` contains `[img src=bytes://test.png]`.
   c. Add an `Image` showcase Card to `crates/honse-ui/src/bin/gallery.rs` (it
      will show a broken/empty image in the gallery since no bytes are registered
      there — that's fine; the goal is API coverage). Import `Image`.
   - **Verify:** `cargo test -p honse-ui --features dev` (the `introspect` feature is enabled via the dev-dependency at `crates/honse-ui/Cargo.toml:13`) → the new img test passes. Confirm the exact test invocation against `crates/honse-ui/Cargo.toml` before running.

## Files to create
- `crates/honse-ui/src/image.rs` — the `Image` component.
- `docs/plans/honse-ui-image-slot.plan.md` — this plan (already created).

## Files to modify
- `crates/honse-ui/src/lib.rs` — `mod image;` + `pub use image::Image;`.
- `apps/hachimi/src/core/gui/splash.rs` — `ICON_URI` const + `register_icon_bytes` helper.
- `apps/hachimi/src/core/gui/instance.rs` — call `register_icon_bytes` after `install_image_loaders` (line ~29).
- `apps/hachimi/src/core/gui/dev_harness.rs` — call `register_icon_bytes` after `install_image_loaders` (line ~45).
- `apps/hachimi/src/core/gui/dioxus/app.rs` — import `Image`, add the header logo child.
- `crates/dioxus-egui/src/renderer.rs` — add `img` arm to `dump_node`.
- `crates/honse-ui/tests/components.rs` — Image render test.
- `crates/honse-ui/src/bin/gallery.rs` — Image showcase.

## Testing notes
- The renderer's headless `dump()` (gated behind `test`/`introspect`) is the only
  way to assert UI in CI (no game process, no GPU). It currently has no `img` arm,
  so task 4a is required before 4b can assert anything.
- Visual confirmation is via the menu preview example (AGENTS.md): it uses the
  same `render_control_center_*` shell as the live game, so the header will look
  identical in-game.
- `bytes://` only resolves after `include_bytes`; if the header shows a broken
  image in the preview, the registration in task 2 didn't run before the first
  paint at that init site.

## Patterns to follow
- `crates/honse-ui/src/separator.rs:8-14` — minimal `#[component]` shape.
- `crates/honse-ui/src/card.rs:17-34` — component with attributes + theme tokens.
- `crates/honse-ui/src/lib.rs:30-53` — `mod` + `pub use` registration ordering.
- `crates/dioxus-egui/src/renderer.rs:638-650` — `walk_img` (the attrs it reads).
- `crates/dioxus-egui/src/renderer.rs:668-688` — `dump_node` element arms to mirror.
- `apps/hachimi/src/core/gui/splash.rs:5-15` — existing icon embed + `Gui::icon` 24px size.
- `crates/honse-ui/tests/components.rs:25-44` — headless dump-assertion test pattern.

## CI gates (AGENTS.md) before handoff
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --lib`
