//! Button — variant- and size-driven, like shadcn's `<Button variant size>`.
//!
//! The renderer paints a themed button (its own variant-colored face with
//! hover/press states) whenever the `button` element carries `bg`/`border`/
//! `color` attributes, so every variant here is fully styled, not the default
//! egui button. Primary uses the uma-green ramp from index.css.

use dioxus_egui::dioxus::prelude::*;

use crate::theme;

/// Visual emphasis of a button. `Default` impl is [`ButtonVariant::Primary`].
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// Solid accent — the main call to action.
    #[default]
    Primary,
    /// Solid neutral surface.
    Secondary,
    /// Transparent with a visible border.
    Outline,
    /// Transparent until hovered.
    Ghost,
    /// Solid danger color.
    Destructive,
}

/// Button footprint. `Default` impl is [`ButtonSize::Md`].
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    Sm,
    #[default]
    Md,
    Lg,
}

/// `(bg, fg, border)` for a variant — soft tinted fills like dashboard pills.
fn palette(v: ButtonVariant) -> (&'static str, &'static str, &'static str) {
    match v {
        // Uma-green primary face (index.css `.uma-green-face` approximated as solid uma-500).
        ButtonVariant::Primary => (theme::GOOD_SOFT, theme::GOOD, theme::UMA_600),
        ButtonVariant::Secondary => (theme::SURFACE_2, theme::FG, theme::LINE),
        ButtonVariant::Outline => (theme::TRANSPARENT, theme::FG, theme::LINE),
        ButtonVariant::Ghost => (theme::TRANSPARENT, theme::FG, theme::TRANSPARENT),
        ButtonVariant::Destructive => (theme::DESTRUCTIVE_SOFT, theme::BAD, theme::BAD),
    }
}

/// `(padding_px, font_px)` for a size.
fn metrics(s: ButtonSize) -> (&'static str, &'static str) {
    match s {
        ButtonSize::Sm => ("6", "13"),
        ButtonSize::Md => ("9", "15"),
        ButtonSize::Lg => ("12", "17"),
    }
}

#[component]
pub fn Button(
    #[props(default)] variant: ButtonVariant,
    #[props(default)] size: ButtonSize,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    let (bg, fg, border) = palette(variant);
    let (padding, font) = metrics(size);
    let radius = theme::RADIUS_SM;
    rsx! {
        button {
            "dir": "row",
            "gap": "6",
            "align": "center",
            "justify": "center",
            "bg": bg,
            "border": border,
            "color": fg,
            "radius": radius,
            "padding": padding,
            "font-size": font,
            "weight": "bold",
            onclick: move |e| onclick.call(e),
            {children}
        }
    }
}
