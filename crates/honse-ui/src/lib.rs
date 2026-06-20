//! `honse-ui` — a small, **owned** component kit for [`dioxus-egui`], built the
//! way [shadcn/ui](https://ui.shadcn.com) builds components: you don't install a
//! black-box widget library, you keep the source and restyle it. Each component
//! is a plain Dioxus `rsx!` function that composes the renderer's primitives
//! (`div`/`button`/`input` + the visual style attributes `bg`/`border`/`radius`/
//! `color`/`padding`/…). Variants are data — an enum maps to a set of style
//! attributes (the cva idea) — so the look lives in [`theme`] and the per-variant
//! tables, not scattered through call sites.
//!
//! The palette is borrowed from the honse-tracker Uma kit (`apps/web/src/index.css`).
//!
//! ```no_run
//! use dioxus_egui::dioxus::prelude::*;
//! use honse_ui::{Button, ButtonVariant, Card, CardTitle};
//!
//! fn app() -> Element {
//!     rsx! {
//!         Card {
//!             CardTitle { "Career" }
//!             Button { variant: ButtonVariant::Primary, onclick: move |_| {}, "Save" }
//!         }
//!     }
//! }
//! ```

#![allow(clippy::disallowed_methods)] // dioxus rsx! macro uses unwrap internally

pub mod theme;

mod badge;
mod button;
mod card;
mod combo;
mod field;
mod image;
mod scroll;
mod separator;
mod slider;
mod tabs;
mod toggle;
mod window;

pub use badge::{Badge, BadgeVariant};
pub use button::{Button, ButtonSize, ButtonVariant};
pub use card::{Card, CardDescription, CardTitle};
pub use combo::{Combo, ComboOption};
pub use field::Field;
pub use image::Image;
pub use scroll::ScrollArea;
pub use separator::Separator;
pub use slider::SliderRow;
pub use tabs::{TabBar, TabItem};
pub use theme::{Grade, Mood, Stat, Tier};
pub use toggle::Toggle;
pub use window::WindowChrome;
