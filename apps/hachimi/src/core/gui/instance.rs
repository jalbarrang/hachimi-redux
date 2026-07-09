use std::sync::Mutex;
use std::time::Instant;

use rust_i18n::t;

use crate::core::Hachimi;

use super::theme;
use super::tween::{Easing, TweenInOutWithDelay};
use super::window::{ConfigEditor, FirstTimeSetupWindow};
use super::{Gui, INSTANCE};

impl Gui {
    pub fn apply_theme(ctx: &egui::Context, style: &mut egui::Style, _config: &crate::core::hachimi::Config) {
        style.visuals = egui::Visuals::dark();
        theme::apply_style(style, crate::core::plugin::overlay::opacity());
        ctx.set_visuals(style.visuals.clone());
    }

    pub fn instance_or_init(#[cfg_attr(target_os = "windows", allow(unused))] open_key_id: &str) -> &Mutex<Gui> {
        if let Some(instance) = INSTANCE.get() {
            return instance;
        }

        let hachimi = Hachimi::instance();
        let config = (**Hachimi::instance().config.load()).clone();

        let context = egui::Context::default();
        egui_extras::install_image_loaders(&context);
        super::splash::register_icon_bytes(&context);

        // egui_taffy lays out via `request_discard` (it re-runs the UI within the
        // same frame until the taffy layout settles). That only works when the
        // render loop performs egui multi-pass (see `frame::run`); `max_passes`
        // caps how many passes a single frame may take. >1 is required or every
        // taffy surface (menu + plugin overlays) flickers as it settles one step
        // per frame instead of per frame-internal pass.
        context.options_mut(|o| o.max_passes = std::num::NonZeroUsize::new(3).expect("nonzero"));

        context.set_fonts(Self::get_font_definitions());

        let mut style = egui::Style::default();
        Self::apply_theme(&context, &mut style, &config);

        // egui paints red "Unaligned" markers wherever a widget lands off the pixel
        // grid. The overlay's content zoom scales fonts/spacing by fractional factors,
        // which pushes layout off-grid and floods the panel with the marker. Disable it.
        // The `debug` field (and the marker) only exist in debug builds; release builds
        // default it off already.
        #[cfg(debug_assertions)]
        {
            style.debug.show_unaligned = false;
        }

        context.set_style(style.clone());

        let default_style = style.clone();

        let mut windows: Vec<super::window::BoxedWindow> = Vec::new();
        if !config.skip_first_time_setup {
            windows.push(Box::new(FirstTimeSetupWindow::new()));
        }

        let now = Instant::now();
        let initial_landscape_ratio = config.windows.gui_landscape_ratio;
        let instance = Gui {
            context,
            input: egui::RawInput::default(),
            gui_scale: 1.0,
            finalized_scale: 1.0,
            finalized_landscape_ratio: initial_landscape_ratio,
            default_style,
            start_time: now,
            prev_main_axis_size: 1,
            last_fps_update: now,
            tmp_frame_count: 0,
            fps_text: "FPS: 0".to_string(),
            last_focused: None,

            show_menu: false,
            menu_tab: super::shell::ControlTab::default(),
            config_editor: ConfigEditor::new(),
            plugins_selected: None,

            splash_visible: true,
            splash_tween: TweenInOutWithDelay::new(0.8, 3.0, Easing::OutQuad),
            splash_sub_str: {
                #[cfg(target_os = "windows")]
                {
                    let key_label =
                        crate::windows::utils::vk_to_display_label(hachimi.config.load().windows.menu_open_key);
                    t!("splash_sub", open_key_str = key_label).into_owned()
                }
                #[cfg(not(target_os = "windows"))]
                {
                    t!("splash_sub", open_key_str = t!(open_key_id)).into_owned()
                }
            },

            menu_visible: false,
            menu_anim_time: None,

            update_progress_visible: false,

            notifications: Vec::new(),
            next_notification_id: 0,
            windows,
        };

        // SAFETY: FFI / raw pointer operation required by IL2CPP interop
        unsafe {
            INSTANCE.set(Mutex::new(instance)).unwrap_unchecked();

            // Surface any conflicting-injector warning recorded by the startup scan
            // now that the GUI exists to show a notification.
            if let Some(summary) = crate::core::conflicts::startup_summary() {
                INSTANCE
                    .get()
                    .unwrap_unchecked()
                    .lock()
                    .expect("lock poisoned")
                    .show_notification(&summary);
            }

            hachimi.run_auto_update_check();

            INSTANCE.get().unwrap_unchecked()
        }
    }

    pub fn instance() -> Option<&'static Mutex<Gui>> {
        INSTANCE.get()
    }
}
