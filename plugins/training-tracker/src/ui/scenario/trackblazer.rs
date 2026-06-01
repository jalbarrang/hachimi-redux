//! Trackblazer RaceCoin shop rendering.

use hachimi_plugin_sdk::egui;

use crate::memory_reader;

use crate::ui::overlay;
use crate::ui::util::worth_color;

pub(super) fn draw(ui: &mut egui::Ui, shop: &memory_reader::TrackblazerShop) {
    draw_header(ui, shop);
    ui.separator();
    if shop.items.is_empty() {
        ui.small("Shop lineup unavailable (open the shop in-game first).");
        return;
    }
    overlay::scroll_list(ui, |ui| {
        draw_lineup(ui, &shop.items);
        draw_owned(ui, &shop.owned);
    });
}

fn draw_header(ui: &mut egui::Ui, shop: &memory_reader::TrackblazerShop) {
    ui.horizontal(|ui| {
        ui.strong(format!("\u{1f3c5} RaceCoins: {}", shop.coins));
        if shop.sale_value > 0 {
            ui.colored_label(
                egui::Color32::from_rgb(220, 120, 60),
                format!("Sale {}%", shop.sale_value),
            );
        }
        if shop.win_points > 0 {
            ui.small(format!("WinPt: {}", shop.win_points));
        }
    });
}

fn draw_lineup(ui: &mut egui::Ui, items: &[memory_reader::TrackblazerShopItem]) {
    egui::Grid::new("trackblazer_shop_grid")
        .num_columns(5)
        .striped(true)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            ui.strong("Item");
            ui.strong("Effect");
            ui.strong("Price");
            ui.strong("Avail");
            ui.strong("Worth");
            ui.end_row();
            for item in items {
                draw_item_row(ui, item);
            }
        });
}

fn draw_owned(ui: &mut egui::Ui, owned: &[memory_reader::TrackblazerOwnedItem]) {
    if owned.is_empty() {
        return;
    }
    ui.add_space(8.0);
    ui.separator();
    ui.strong("Owned items");
    egui::Grid::new("trackblazer_owned_grid")
        .num_columns(3)
        .striped(true)
        .spacing([12.0, 4.0])
        .show(ui, |ui| {
            for o in owned {
                if o.name.is_empty() {
                    ui.small(format!("#{}", o.item_id));
                } else {
                    ui.label(&o.name);
                }
                if o.effect.is_empty() {
                    ui.small("—");
                } else {
                    ui.small(&o.effect);
                }
                ui.label(format!("\u{00d7}{}", o.count));
                ui.end_row();
            }
        });
}

/// One shop lineup row in the Item | Effect | Price | Worth grid.
fn draw_item_row(ui: &mut egui::Ui, item: &memory_reader::TrackblazerShopItem) {
    let dim = egui::Color32::from_rgb(140, 140, 140);

    if item.name.is_empty() {
        ui.small(format!("#{}", item.item_id));
    } else {
        ui.label(&item.name);
    }

    if item.effect.is_empty() {
        ui.small("—");
    } else {
        ui.label(&item.effect);
    }

    ui.horizontal(|ui| {
        let price_color = if item.sold_out() {
            dim
        } else if item.discounted() {
            egui::Color32::from_rgb(220, 120, 60)
        } else {
            egui::Color32::from_rgb(230, 200, 90)
        };
        ui.colored_label(price_color, format!("{} \u{1fa99}", item.coin_num));
        if item.discounted() {
            ui.colored_label(
                dim,
                egui::RichText::new(format!("{}", item.original_coin_num)).strikethrough(),
            );
        }
    });

    if item.turns_left > 0 {
        ui.colored_label(
            egui::Color32::from_rgb(220, 120, 60),
            format!("{} turn(s)", item.turns_left),
        );
    } else {
        ui.small("—");
    }

    match item.worth {
        Some(w) => ui.colored_label(worth_color(w), w.label()),
        None => ui.small("—"),
    };
    ui.end_row();
}
