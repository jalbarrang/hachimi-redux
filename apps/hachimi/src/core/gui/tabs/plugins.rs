//! L1 Plugins tab — a sub-nav of plugin-registered pages (menu sections become
//! L1 pages). Scales to N plugins without N top-level tabs: pages are listed as
//! selectable chips, and the selected page's callback is rendered below.

use std::borrow::Cow;
use std::panic::{self, AssertUnwindSafe};

use crate::core::gui::components as widgets;
use crate::core::gui::components::PillButtonKind;
use crate::core::gui::scale::get_scale;
use crate::core::gui::Gui;
use crate::core::plugin::menu::{get_plugin_menu_icon, get_plugin_menu_items, get_plugin_menu_sections};
use crate::core::plugin::OwnerScope;

use super::layout::{auto_cell, flex_row, flex_wrap};

impl Gui {
    pub(crate) fn run_plugins_tab(
        &mut self,
        ui: &mut egui::Ui,
        _ctx: &egui::Context,
        _show_notification: &mut Option<Cow<'_, str>>,
    ) {
        let scale = get_scale(ui.ctx());
        let items = get_plugin_menu_items();
        let sections = get_plugin_menu_sections();

        if items.is_empty() && sections.is_empty() {
            ui.add_space(8.0);
            widgets::empty_state(ui, "No plugins have registered any pages.");
            return;
        }

        if !items.is_empty() {
            flex_wrap(ui, ui.id().with("plugin_items"), scale, 8.0, |tui| {
                for (i, item) in items.iter().enumerate() {
                    auto_cell(tui, |ui| {
                        let clicked = match get_plugin_menu_icon(&item.label) {
                            Some(icon) => {
                                let size = 18.0 * scale;
                                ui.add(
                                    egui::Image::new((icon.uri, icon.bytes)).fit_to_exact_size(egui::Vec2::splat(size)),
                                );
                                widgets::secondary_button(ui, item.label.clone()).clicked()
                            }
                            None => widgets::secondary_button(ui, item.label.clone()).clicked(),
                        };
                        if clicked {
                            if let Some(callback) = &item.callback {
                                let _ = panic::catch_unwind(AssertUnwindSafe(|| {
                                    callback.invoke();
                                }))
                                .inspect_err(|_| error!("plugin menu item callback panicked: {}", item.label));
                            }
                        }
                        let _ = i;
                    });
                }
            });
            if !sections.is_empty() {
                ui.separator();
            }
        }

        if sections.is_empty() {
            return;
        }

        let selected_valid = self
            .plugins_selected
            .is_some_and(|h| sections.iter().any(|s| s.handle == h));
        if !selected_valid {
            self.plugins_selected = Some(sections[0].handle);
        }

        flex_wrap(ui, ui.id().with("plugin_pages"), scale, 6.0, |tui| {
            for (i, section) in sections.iter().enumerate() {
                let label = page_label(section.title.as_deref(), i);
                let selected = self.plugins_selected == Some(section.handle);
                let kind = if selected {
                    PillButtonKind::Primary
                } else {
                    PillButtonKind::Secondary
                };
                auto_cell(tui, |ui| {
                    if widgets::pill_button(ui, label, kind).clicked() {
                        self.plugins_selected = Some(section.handle);
                    }
                });
            }
        });
        ui.separator();

        let Some(section) = sections.iter().find(|s| Some(s.handle) == self.plugins_selected) else {
            return;
        };

        if let Some(title) = &section.title {
            flex_row(ui, ui.id().with("plugin_page_title"), scale, 8.0, |tui| {
                if let Some(icon) = &section.icon {
                    auto_cell(tui, |ui| {
                        let size = 18.0 * scale;
                        ui.add(
                            egui::Image::new((icon.uri.clone(), icon.bytes.clone()))
                                .fit_to_exact_size(egui::Vec2::splat(size)),
                        );
                    });
                }
                auto_cell(tui, |ui| {
                    widgets::section_banner(ui, title.clone());
                });
            });
        }

        let _scope = OwnerScope::enter(section.owner);
        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            section.callback.invoke(ui);
        }))
        .inspect_err(|_| error!("plugin menu section callback panicked"));
    }
}

fn page_label(title: Option<&str>, index: usize) -> String {
    match title {
        Some(t) if !t.is_empty() => t.to_owned(),
        _ => format!("Plugin {}", index + 1),
    }
}
