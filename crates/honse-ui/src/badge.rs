//! Badge — a small status tag mirroring honse-tracker uma-kit pills: a soft tinted
//! fill with a strong matching text color, no border, `radius: 4px`, `font-size: 10px`.

use dioxus_egui::dioxus::prelude::*;

use crate::theme::{self, Grade, Mood, Stat};

/// Badge color treatment. `Default` impl is [`BadgeVariant::Neutral`].
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Neutral,
    /// Blue accent — links, leaders.
    Accent,
    /// Green success (uma-500).
    Good,
    /// Amber warning.
    Warn,
    /// Red danger (grade-a).
    Destructive,
    /// Training grade letter color.
    Grade(Grade),
    /// Stat column color.
    Stat(Stat),
    /// Mood indicator color.
    Mood(Mood),
}

/// `(bg, fg)` for a variant — soft tinted fill + strong text, no border.
fn palette(v: BadgeVariant) -> (&'static str, &'static str) {
    match v {
        BadgeVariant::Neutral => (theme::NEUTRAL_SOFT, theme::FG_MUTED),
        BadgeVariant::Accent => (theme::ACCENT_SOFT, theme::ACCENT),
        BadgeVariant::Good => (theme::GOOD_SOFT, theme::GOOD),
        BadgeVariant::Warn => (theme::WARN_SOFT, theme::WARN),
        BadgeVariant::Destructive => (theme::DESTRUCTIVE_SOFT, theme::BAD),
        BadgeVariant::Grade(g) => {
            let c = g.color();
            (theme::soft_fill(c), c)
        }
        BadgeVariant::Stat(s) => {
            let c = s.color();
            (theme::soft_fill(c), c)
        }
        BadgeVariant::Mood(m) => {
            let c = m.color();
            (theme::soft_fill(c), c)
        }
    }
}

#[component]
pub fn Badge(#[props(default)] variant: BadgeVariant, children: Element) -> Element {
    let (bg, fg) = palette(variant);
    rsx! {
        div {
            "dir": "row",
            "align": "center",
            "gap": "4",
            "padding": "2",
            "bg": bg,
            "color": fg,
            "radius": theme::RADIUS_BADGE,
            "font-size": "10",
            "weight": "bold",
            {children}
        }
    }
}
