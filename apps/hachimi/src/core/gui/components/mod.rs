//! Shared egui widget kit for the host UI.
//!
//! The kit is a dark reinterpretation of the Honse game's UI language: pill
//! controls, bright accent section banners, compact cards, and cockpit-friendly
//! mono numbers for telemetry-like values.

mod buttons;
mod cards;
mod combos;
mod feedback;
mod sections;
mod tags;

pub(crate) use buttons::*;
pub(crate) use feedback::*;
pub(crate) use sections::*;
