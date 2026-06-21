//! Ergonomic Hachimi plugin SDK — safe wrappers around [`hachimi_plugin_abi`].
//!
//! Plugins typically use the [`hachimi_plugin`] attribute to generate their C entry
//! points, then draw GUI with the shared [`egui`] re-export.

pub use hachimi_plugin_abi::*;
pub use hachimi_plugin_macros::hachimi_plugin;

/// Re-exported egui — plugins MUST use this so the version matches the host exactly.
pub use egui;

mod gui;
mod hook;
mod il2cpp;
mod sdk;
mod version;
pub mod widgets;

pub use gui::ui_from_ptr;
pub use sdk::{init_result_to_i32, InitError, Sdk};
pub use version::ApiVersion;
