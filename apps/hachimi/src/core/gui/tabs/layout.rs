//! Shared egui_taffy layout helpers for Control Center tab bodies.
//!
//! Provides a Tailwind-ish authoring surface: pinned-width flex rows/columns for
//! button clusters, and a two-column settings grid for label + control pairs.

use egui_taffy::taffy::prelude::{auto, length};
use egui_taffy::{taffy, tui, Tui, TuiBuilderLogic};

use super::super::shell;

/// Deterministic width for a tab body root: the menu shell width. Pinned via
/// `reserve_width` instead of `reserve_available_width` — the latter reads the
/// modal's current width and feeds it back into layout, so a tab whose content
/// is wider than the shell makes the modal grow on each pass.
pub(crate) fn body_grid_width(scale: f32) -> f32 {
    (shell::SHELL_WIDTH * scale).max(120.0)
}

/// Width the scroll area reserves for its vertical bar (0 when floating).
fn scrollbar_allowance(ui: &egui::Ui) -> f32 {
    let s = &ui.spacing().scroll;
    if s.floating {
        0.0
    } else {
        s.bar_width + s.bar_inner_margin + s.bar_outer_margin
    }
}

/// True usable width of a tab body: shell width minus the scrollbar reservation.
/// Deterministic (derived from `SHELL_WIDTH`, NOT `ui.available_width()`), so it
/// keeps the anti-jitter guarantee while no longer overrunning the visible region.
pub(crate) fn content_width(ui: &egui::Ui, scale: f32) -> f32 {
    (body_grid_width(scale) - scrollbar_allowance(ui)).max(120.0)
}

fn flex_root_style(width: f32, direction: taffy::FlexDirection, gap: f32, wrap: taffy::FlexWrap) -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: direction,
        flex_wrap: wrap,
        align_items: Some(taffy::AlignItems::Center),
        gap: taffy::Size {
            width: length(gap),
            height: length(gap),
        },
        size: taffy::Size {
            width: length(width),
            height: auto(),
        },
        max_size: taffy::Size {
            width: length(width),
            height: auto(),
        },
        ..Default::default()
    }
}

pub(crate) fn cell_style() -> taffy::Style {
    taffy::Style {
        display: taffy::Display::Flex,
        align_items: Some(taffy::AlignItems::Center),
        justify_content: Some(taffy::JustifyContent::Start),
        min_size: taffy::Size {
            width: length(0.0),
            height: auto(),
        },
        ..Default::default()
    }
}

/// A content-sized control cell (checkbox / combo / button): stable size.
pub(crate) fn auto_cell<R>(tui: &mut Tui, content: impl FnOnce(&mut egui::Ui) -> R) -> R {
    tui.style(cell_style()).add(|tui| tui.ui(content))
}

/// Flex grow spacer — pushes siblings apart (like `flex-grow: 1` in CSS).
pub(crate) fn flex_spacer(tui: &mut Tui) {
    tui.style(taffy::Style {
        flex_grow: 1.0,
        min_size: taffy::Size {
            width: length(0.0),
            height: auto(),
        },
        ..Default::default()
    })
    .add(|_| {});
}

/// Pinned-width horizontal flex row (button groups, hotkey rows).
pub(crate) fn flex_row<R>(
    ui: &mut egui::Ui,
    id: egui::Id,
    scale: f32,
    gap: f32,
    content: impl FnOnce(&mut Tui) -> R,
) -> R {
    let w = content_width(ui, scale);
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
    tui(ui, id)
        .reserve_width(w)
        .style(flex_root_style(
            w,
            taffy::FlexDirection::Row,
            gap,
            taffy::FlexWrap::NoWrap,
        ))
        .show(content)
}

/// Pinned-width horizontal flex row with wrap (action button clusters).
pub(crate) fn flex_wrap<R>(
    ui: &mut egui::Ui,
    id: egui::Id,
    scale: f32,
    gap: f32,
    content: impl FnOnce(&mut Tui) -> R,
) -> R {
    let w = content_width(ui, scale);
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
    tui(ui, id)
        .reserve_width(w)
        .style(flex_root_style(
            w,
            taffy::FlexDirection::Row,
            gap,
            taffy::FlexWrap::Wrap,
        ))
        .show(content)
}

/// Pinned-width vertical flex column (stacked sections).
#[allow(dead_code)]
pub(crate) fn flex_col<R>(
    ui: &mut egui::Ui,
    id: egui::Id,
    scale: f32,
    gap: f32,
    content: impl FnOnce(&mut Tui) -> R,
) -> R {
    let w = content_width(ui, scale);
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
    tui(ui, id)
        .reserve_width(w)
        .style(taffy::Style {
            display: taffy::Display::Flex,
            flex_direction: taffy::FlexDirection::Column,
            align_items: Some(taffy::AlignItems::Stretch),
            gap: taffy::Size {
                width: length(gap),
                height: length(gap),
            },
            size: taffy::Size {
                width: length(w),
                height: auto(),
            },
            max_size: taffy::Size {
                width: length(w),
                height: auto(),
            },
            ..Default::default()
        })
        .show(content)
}
