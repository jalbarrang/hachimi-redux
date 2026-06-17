//! Control Center modal shell: header, tab bar, scrolling body, pinned footer.
//!
//! Shared by the live `Gui` modal and the desktop `dev_harness` preview via
//! [`render_control_center`] and the [`ControlCenterHost`] trait.

use rust_i18n::t;

use egui_taffy::taffy::prelude::{auto, length};
use egui_taffy::{taffy, tui, TuiBuilderLogic};

use super::components::{self as widgets, PillButtonKind};
use super::debug::dbg_outline;
use super::theme::ThemeTokens;
use super::window::ConfigEditor;

/// Base (unscaled) width of the Control Center modal shell. Multiplied by the
/// GUI scale to get the deterministic pixel width. Shared with `config_editor`
/// so the body grids reserve a pinned width derived from this same value
/// (NOT `reserve_available_width`, which feeds the modal's own width back into
/// layout and stretches the panel on tab change).
pub(crate) const SHELL_WIDTH: f32 = 600.0;

fn shell_content_style() -> taffy::Style {
    taffy::Style {
        flex_grow: 1.0,
        flex_basis: length(0.0),
        min_size: taffy::Size {
            width: auto(),
            height: length(0.0),
        },
        ..Default::default()
    }
}

/// Fixed top-level tabs of the Control Center. The former Config sub-tabs
/// (General/Graphics/Gameplay/Hotkeys) are now top-level; Overlay was removed.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ControlTab {
    #[default]
    General,
    Graphics,
    Gameplay,
    Hotkeys,
    Translations,
    Plugins,
    About,
}

impl ControlTab {
    /// Tabs whose body edits the config working-copy (the Save/Cancel footer is
    /// active there; disabled on the others).
    pub(crate) fn edits_config(self) -> bool {
        matches!(
            self,
            ControlTab::General
                | ControlTab::Graphics
                | ControlTab::Gameplay
                | ControlTab::Hotkeys
                | ControlTab::Translations
        )
    }
}

/// Host abstraction for the Control Center shell so the live `Gui` modal and the
/// desktop `dev_harness` preview share the exact same shell / tab-bar / footer
/// layout (`render_control_center`). The host owns the active tab, the config
/// working copy (for the footer), the title icon, and per-tab body drawing.
pub(crate) trait ControlCenterHost {
    fn active_tab(&self) -> ControlTab;
    fn set_active_tab(&mut self, tab: ControlTab);
    fn config_editor(&mut self) -> &mut ConfigEditor;
    fn draw_icon(&mut self, ui: &mut egui::Ui, ctx: &egui::Context);
    fn draw_body(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, tab: ControlTab);
}

/// Render the Control Center shell (header / tab bar / scrolling content / pinned
/// footer) into `ui`, dispatching tab bodies through `host`. Shared by the live
/// `Gui` modal and the desktop preview harness. Returns whether the menu should
/// stay open (the header close button clears it).
pub(crate) fn render_control_center(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    scale: f32,
    host: &mut dyn ControlCenterHost,
) -> bool {
    // Deterministic shell size (NOT available_width/height, which feed back into
    // the auto-sizing container and make the panel jitter/stretch). Width fixed;
    // height caps at 85% of the viewport.
    let shell_w = SHELL_WIDTH * scale;
    let shell_h = ctx.input(|i| i.viewport_rect().height()) * 0.85;
    ui.set_width(shell_w);
    ui.set_max_height(shell_h);
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

    let mut keep_open = true;

    let tokens = ThemeTokens::from_ui(ui);
    egui::Frame::new().fill(tokens.window).show(ui, |ui| {
        // Menu shell
        tui(ui, ui.id().with("menu_shell"))
            .reserve_width(shell_w)
            .style(taffy::Style {
                display: taffy::Display::Flex,
                flex_direction: taffy::FlexDirection::Column,
                align_items: Some(taffy::AlignItems::Stretch),
                gap: taffy::Size {
                    width: length(0.0),
                    height: length(8.0 * scale),
                },
                size: taffy::Size {
                    width: length(shell_w),
                    height: length(shell_h),
                },
                max_size: taffy::Size {
                    width: length(shell_w),
                    height: length(shell_h),
                },
                ..Default::default()
            })
            .show(|tui| {
                // Header row: icon + title + version flex-packed to the left, a
                // grow spacer, then the close button pinned to the right edge.
                widgets::card_node(&mut *tui, |tui| {
                    tui.style(taffy::Style {
                        display: taffy::Display::Flex,
                        flex_direction: taffy::FlexDirection::Row,
                        align_items: Some(taffy::AlignItems::Center),
                        gap: taffy::Size {
                            width: length(8.0 * scale),
                            height: length(0.0),
                        },
                        ..Default::default()
                    })
                    .add(|tui| {
                        dbg_outline(tui.egui_ui(), egui::Color32::from_rgb(0, 255, 0), "row");
                        tui.ui(|ui| {
                            dbg_outline(ui, egui::Color32::from_rgb(0, 200, 255), "icon");
                            host.draw_icon(ui, ctx);
                        });
                        tui.ui(|ui| {
                            dbg_outline(ui, egui::Color32::from_rgb(255, 200, 0), "title");
                            ui.heading(t!("hachimi"));
                        });
                        tui.ui(|ui| {
                            dbg_outline(ui, egui::Color32::from_rgb(255, 120, 0), "tag");
                            widgets::category_tag(ui, env!("HACHIMI_DISPLAY_VERSION"));
                        });

                        // Grow spacer eats the remaining width, flexing the close
                        // button to the right edge of the card.
                        tui.style(taffy::Style {
                            flex_grow: 1.0,
                            ..Default::default()
                        })
                        .add(|tui| {
                            dbg_outline(tui.egui_ui(), egui::Color32::from_rgb(120, 120, 120), "spacer");
                        });

                        tui.ui(|ui| {
                            dbg_outline(ui, egui::Color32::from_rgb(255, 0, 255), "close");
                            if widgets::ghost_button(ui, "\u{f00d}")
                                .on_hover_text(t!("menu.close_menu"))
                                .clicked()
                            {
                                keep_open = false;
                            }
                        });
                    });
                });

                // Top tab bar: a single horizontally-scrollable row (content height).
                // Pin the node to the shell width with a zero automatic minimum so the
                // wide 7-tab row's intrinsic min-content width can't inflate the flex
                // column's (stretched) cross-axis and overflow the shell — without this
                // the row sizes to its content (~820px), stretching the body node and
                // pushing `section_banner` past the modal frame. The fixed width also
                // lets the horizontal `ScrollArea` actually scroll instead of overflow.
                tui.style(taffy::Style {
                    size: taffy::Size {
                        width: length(shell_w),
                        height: auto(),
                    },
                    max_size: taffy::Size {
                        width: length(shell_w),
                        height: auto(),
                    },
                    min_size: taffy::Size {
                        width: length(0.0),
                        height: auto(),
                    },
                    ..Default::default()
                })
                .ui(|ui| {
                    dbg_outline(ui, egui::Color32::from_rgb(0, 255, 255), "tabs");
                    egui::ScrollArea::horizontal().id_salt("l1_tabs_scroll").show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 6.0 * scale;

                            shell_tab_button(ui, host, ControlTab::General, &t!("config_editor.general_tab"));
                            shell_tab_button(ui, host, ControlTab::Graphics, &t!("config_editor.graphics_tab"));
                            shell_tab_button(ui, host, ControlTab::Gameplay, &t!("config_editor.gameplay_tab"));
                            shell_tab_button(ui, host, ControlTab::Hotkeys, &t!("config_editor.hotkeys_tab"));
                            shell_tab_button(ui, host, ControlTab::Translations, "\u{f1ab} Translations");
                            shell_tab_button(ui, host, ControlTab::Plugins, "\u{f12e} Plugins");
                            shell_tab_button(ui, host, ControlTab::About, "\u{f129} About");
                        });
                    });
                });

                // Scrolling content fills the remaining height.
                tui.style(shell_content_style()).ui(|ui| {
                    dbg_outline(ui, egui::Color32::from_rgb(80, 160, 255), "body");
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                        let tab = host.active_tab();
                        host.draw_body(ui, ctx, tab);
                    });
                });

                // Pinned footer: Save/Cancel (greyed where the tab doesn't edit config).
                tui.ui(|ui| {
                    dbg_outline(ui, egui::Color32::from_rgb(255, 255, 0), "footer");
                    let tab = host.active_tab();
                    host.config_editor().ui_footer(ui, tab.edits_config());
                });
            });
    });

    keep_open
}

fn shell_tab_button(ui: &mut egui::Ui, host: &mut dyn ControlCenterHost, tab: ControlTab, label: &str) {
    let kind = if host.active_tab() == tab {
        PillButtonKind::Primary
    } else {
        PillButtonKind::Secondary
    };
    if widgets::pill_button(ui, label, kind).clicked() {
        host.set_active_tab(tab);
    }
}
