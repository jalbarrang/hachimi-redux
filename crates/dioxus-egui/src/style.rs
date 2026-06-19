//! Small taffy-style helpers the renderer emits. egui_taffy does not export a
//! default-style constructor, so we own these.

use egui_taffy::taffy::prelude::length;
use egui_taffy::taffy::{AlignItems, Display, FlexDirection, FlexWrap, JustifyContent, Overflow, Point, Style};

/// Base style generated containers start from.
pub fn default_style() -> Style {
    Style {
        gap: length(4.0),
        padding: length(4.0),
        ..Default::default()
    }
}

/// Flex container style.
pub fn flex(direction: FlexDirection, gap: f32, align: Option<AlignItems>) -> Style {
    Style {
        display: Display::Flex,
        flex_direction: direction,
        flex_wrap: FlexWrap::NoWrap,
        align_items: align,
        gap: length(gap),
        ..default_style()
    }
}

fn attr<'a>(attrs: &'a [(String, String)], name: &str) -> Option<&'a str> {
    attrs.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str())
}

fn attr_f32(attrs: &[(String, String)], name: &str) -> Option<f32> {
    attr(attrs, name).and_then(|v| v.parse::<f32>().ok())
}

/// Map an `"align"` attribute value to a taffy `AlignItems` (cross-axis).
fn parse_align(align: Option<&str>) -> AlignItems {
    match align {
        Some("start") => AlignItems::Start,
        Some("end") => AlignItems::End,
        Some("center") => AlignItems::Center,
        Some("stretch") => AlignItems::Stretch,
        Some("baseline") => AlignItems::Baseline,
        // Default keeps the previous behaviour: center children on the cross axis.
        _ => AlignItems::Center,
    }
}

/// Map a `"scroll"` attribute value to taffy `overflow` (egui_taffy wraps children
/// in a bounded `ScrollArea` when an axis is `Scroll`).
fn parse_overflow(scroll: Option<&str>) -> Point<Overflow> {
    match scroll {
        Some("y" | "vertical") => Point {
            x: Overflow::Visible,
            y: Overflow::Scroll,
        },
        Some("x" | "horizontal") => Point {
            x: Overflow::Scroll,
            y: Overflow::Visible,
        },
        _ => Point {
            x: Overflow::Visible,
            y: Overflow::Visible,
        },
    }
}

/// Map a `"justify"` attribute value to a taffy `JustifyContent` (main-axis).
fn parse_justify(justify: Option<&str>) -> Option<JustifyContent> {
    match justify {
        Some("start") => Some(JustifyContent::Start),
        Some("end") => Some(JustifyContent::End),
        Some("center") => Some(JustifyContent::Center),
        Some("between") => Some(JustifyContent::SpaceBetween),
        Some("around") => Some(JustifyContent::SpaceAround),
        Some("evenly") => Some(JustifyContent::SpaceEvenly),
        _ => None,
    }
}

/// Map layout attributes including optional CSS grid (`display="grid"`).
pub fn container_style(attrs: &[(String, String)]) -> Style {
    use egui_taffy::taffy::prelude::{fr, max_content};
    use egui_taffy::taffy::Display;

    let direction = match attr(attrs, "dir") {
        Some("row") => FlexDirection::Row,
        _ => FlexDirection::Column,
    };
    let gap = attr_f32(attrs, "gap").unwrap_or(8.0);
    let mut style = flex(direction, gap, Some(parse_align(attr(attrs, "align"))));
    if attr(attrs, "display") == Some("grid") {
        style.display = Display::Grid;
        style.grid_template_columns = match attr(attrs, "grid-cols") {
            Some("label-control") => vec![max_content(), fr(1.0)],
            Some(cols) => {
                let n = cols.parse::<u16>().unwrap_or(2);
                vec![fr(1.0); n as usize]
            }
            None => vec![fr(1.0); 2],
        };
        style.gap = length(gap);
        let item_align = parse_align(attr(attrs, "align"));
        style.align_items = Some(item_align);
        style.justify_items = Some(item_align);
    }
    if let Some(w) = attr_f32(attrs, "width") {
        style.size.width = length(w);
    }
    if let Some(h) = attr_f32(attrs, "height") {
        style.size.height = length(h);
    }
    if let Some(g) = attr_f32(attrs, "grow") {
        style.flex_grow = g;
    }
    if let Some(p) = attr_f32(attrs, "padding") {
        style.padding = length(p);
    }
    if let Some(min_h) = attr_f32(attrs, "min-height") {
        style.min_size.height = length(min_h);
    }
    style.overflow = parse_overflow(attr(attrs, "scroll"));
    style.justify_content = parse_justify(attr(attrs, "justify"));
    style
}
