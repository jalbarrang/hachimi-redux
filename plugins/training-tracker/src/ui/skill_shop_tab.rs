//! Skill Shop tab: SP + filters + purchasable list (scrollable).

use egui_taffy::taffy::prelude::{auto, length};
use egui_taffy::{taffy, tui, TuiBuilderLogic, TuiContainerResponse};
use hachimi_plugin_sdk::egui::{self, Color32, RichText, Vec2, Vec2b};

use crate::overlay_cache;
use crate::skill_shop;
use crate::skill_shop_prefs::{cycle_sort_mode, prefs, set_prefs, sort_mode_label, DistanceFilter, StyleFilter};

use super::dimens;
use super::overlay;

pub(super) fn draw(ui: &mut egui::Ui) {
    overlay_cache::maybe_request_refresh();
    if overlay_cache::snapshot().is_none() {
        ui.small("Loading shop data…");
        return;
    }

    draw_header(ui);
    draw_controls(ui);
    ui.separator();
    overlay::scroll_list(ui, draw_list);
}

fn draw_header(ui: &mut egui::Ui) {
    if let Some(sp) = overlay_cache::skill_points() {
        ui.strong(format!("SP: {}", sp));
    }
}

fn draw_controls(ui: &mut egui::Ui) {
    let p = prefs();

    if ui
        .small_button(format!("Sort: {}", sort_mode_label(p.sort_mode)))
        .clicked()
    {
        cycle_sort_mode();
    }

    ui.horizontal_wrapped(|ui| {
        ui.small("Style:");
        for &(label, filter) in StyleFilter::LABELS {
            let selected = p.style_filter == filter;
            if ui
                .small_button(format!("{}{}", if selected { "*" } else { "" }, label))
                .clicked()
            {
                set_prefs(|prefs| prefs.style_filter = filter);
            }
        }
    });

    ui.horizontal_wrapped(|ui| {
        ui.small("Dist:");
        for &(label, filter) in DistanceFilter::LABELS {
            let selected = p.distance_filter == filter;
            if ui
                .small_button(format!("{}{}", if selected { "*" } else { "" }, label))
                .clicked()
            {
                set_prefs(|prefs| prefs.distance_filter = filter);
            }
        }
    });

    let mut show_hintless = p.show_hintless;
    if ui.checkbox(&mut show_hintless, "Show full-price (no hint)").changed() {
        set_prefs(|prefs| prefs.show_hintless = show_hintless);
    }
    if show_hintless {
        ui.small("Open the in-game skill shop once to capture purchasable rows.");
    }
}

fn draw_list(ui: &mut egui::Ui) {
    let entries = skill_shop::prepare_entries_for_display(overlay_cache::skill_shop(), &prefs());
    if entries.is_empty() {
        ui.small("No shop skills match filters");
        return;
    }

    let w = overlay::content_width();
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
    for (idx, entry) in entries.iter().enumerate() {
        let icon = skill_shop::rarity_label(entry.rarity);
        let discount = skill_shop::discount_pct(entry.hint_level, false);
        let color = if entry.rarity >= 2 {
            Color32::from_rgb(255, 200, 50)
        } else {
            Color32::from_rgb(220, 220, 220)
        };

        let name = if entry.name.is_empty() {
            format!("#{}", entry.group_id)
        } else {
            entry.name.clone()
        };
        let prefix = if !entry.has_hint { "[full] " } else { "" };
        let left = format!("{prefix}{icon} {name}");

        let cost = (entry.base_cost > 0).then(|| skill_shop::discounted_cost(entry.base_cost, entry.hint_level, false));
        let right = match (discount > 0, cost) {
            (true, Some(c)) => format!("-{discount}%  {c}pt"),
            (true, None) => format!("-{discount}%"),
            (false, Some(c)) => format!("{c}pt"),
            (false, None) => String::new(),
        };
        let right_color = if discount > 0 {
            Color32::from_rgb(120, 200, 120)
        } else {
            Color32::from_rgb(160, 160, 160)
        };

        shop_row(ui, idx, w, &left, color, &right, right_color);
    }
}

/// `[icon+name (fills, truncates) | discount/cost (right)]` as a taffy flex row.
#[allow(clippy::too_many_arguments)]
fn shop_row(ui: &mut egui::Ui, idx: usize, w: f32, left: &str, left_color: Color32, right: &str, right_color: Color32) {
    tui(ui, ui.id().with("shop_row").with(idx))
        .reserve_width(w)
        .style(taffy::Style {
            display: taffy::Display::Flex,
            flex_direction: taffy::FlexDirection::Row,
            align_items: Some(taffy::AlignItems::Center),
            gap: taffy::Size {
                width: length(dimens::z(dimens::GAP_MD)),
                height: length(0.0),
            },
            size: taffy::Size {
                width: length(w),
                height: auto(),
            },
            ..Default::default()
        })
        .show(|tui| {
            // Name fills and truncates; constant reported size avoids the
            // relayout feedback loop (see career/bonds.rs).
            tui.style(taffy::Style {
                display: taffy::Display::Flex,
                flex_grow: 1.0,
                align_items: Some(taffy::AlignItems::Center),
                justify_content: Some(taffy::JustifyContent::Start),
                min_size: taffy::Size {
                    width: length(0.0),
                    height: auto(),
                },
                ..Default::default()
            })
            .add(|tui| {
                tui.ui_manual(|ui, _| {
                    ui.add(egui::Label::new(RichText::new(left).small().color(left_color)).truncate());
                    let h = ui.min_size().y;
                    TuiContainerResponse {
                        inner: (),
                        min_size: Vec2::new(0.0, h),
                        intrinsic_size: None,
                        max_size: Vec2::new(0.0, h),
                        infinite: Vec2b::new(true, false),
                    }
                });
            });
            if !right.is_empty() {
                tui.style(taffy::Style {
                    display: taffy::Display::Flex,
                    align_items: Some(taffy::AlignItems::Center),
                    ..Default::default()
                })
                .add(|tui| {
                    tui.ui(|ui| {
                        ui.label(RichText::new(right).small().strong().color(right_color));
                    });
                });
            }
        });
}
