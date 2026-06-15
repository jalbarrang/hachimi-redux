//! Desktop preview of the Control Center menu.
//!
//! Renders the real menu shell + config tab bodies in a native eframe window
//! with a default config, so the UI can be iterated on without launching the
//! Honse game.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p hachimi --example menu_preview --features dev-harness
//! ```

fn main() -> eframe::Result {
    hachimi::core::gui::dev_harness::run()
}
