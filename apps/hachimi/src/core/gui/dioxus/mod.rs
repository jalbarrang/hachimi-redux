// The Dioxus `rsx!` macro expands to internal `Option::unwrap()` calls, which the
// workspace `disallowed_methods` lint bans. Allow it across the whole Dioxus UI
// subtree (this attribute is inherited by the child modules below).
#![allow(clippy::disallowed_methods)]

pub(crate) mod app;
pub(crate) mod context;
pub(crate) mod tabs;

#[cfg(test)]
mod reactivity_tests;

pub(crate) use app::control_center_app;
