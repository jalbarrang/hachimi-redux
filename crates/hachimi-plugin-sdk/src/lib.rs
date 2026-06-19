//! Ergonomic Hachimi plugin SDK — safe wrappers around [`hachimi_plugin_abi`].
//!
//! Plugins typically use the [`hachimi_plugin`] attribute to generate their C entry
//! points, then draw GUI with the shared [`egui`] re-export.

pub use hachimi_plugin_abi::*;
pub use hachimi_plugin_macros::hachimi_plugin;

/// Re-exported egui — plugins MUST use this so the version matches the host exactly.
pub use egui;

pub use dioxus;
pub use dioxus_egui;
pub use honse_ui;

mod gui;
mod hook;
mod il2cpp;
mod mount;
mod sdk;
mod version;

pub use gui::ui_from_ptr;
pub use mount::{mount, UiMount};
pub use sdk::{init_result_to_i32, InitError, Sdk};
pub use version::ApiVersion;
