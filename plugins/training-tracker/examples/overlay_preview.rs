//! Desktop preview of the Training Tracker overlay.
//!
//! Renders the real overlay panel in a native eframe window with mocked career
//! data, so the UI can be iterated on without launching the Honse game.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p hachimi-training-tracker --example overlay_preview --features dev-harness
//! ```

fn main() -> eframe::Result {
    hachimi_training_tracker::dev_harness::run()
}
