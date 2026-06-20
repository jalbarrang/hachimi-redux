use super::scale::get_scale;
use super::Gui;

/// Stable URI for the bundled app icon, registered into egui's byte loader at
/// GUI init so `<img src="bytes://hachimi-icon.png">` resolves in the Dioxus shell.
pub(crate) const ICON_URI: &str = "bytes://hachimi-icon.png";

pub(crate) fn register_icon_bytes(ctx: &egui::Context) {
    ctx.include_bytes(ICON_URI, include_bytes!("../../../assets/icon.png") as &[u8]);
}

impl Gui {
    const ICON_IMAGE: egui::ImageSource<'static> = egui::include_image!("../../../assets/icon.png");

    pub(crate) fn icon<'a>(ctx: &egui::Context) -> egui::Image<'a> {
        let scale = get_scale(ctx);
        egui::Image::new(Self::ICON_IMAGE).fit_to_exact_size(egui::Vec2::new(24.0 * scale, 24.0 * scale))
    }

    pub(crate) fn icon_2x<'a>(ctx: &egui::Context) -> egui::Image<'a> {
        let scale = get_scale(ctx);
        egui::Image::new(Self::ICON_IMAGE).fit_to_exact_size(egui::Vec2::new(48.0 * scale, 48.0 * scale))
    }

    pub(crate) fn run_splash(&mut self) {
        let ctx = &self.context;
        let scale = get_scale(ctx);

        let id = egui::Id::from("splash");
        let Some(tween_val) = self.splash_tween.run(ctx, id.with("tween")) else {
            self.splash_visible = false;
            return;
        };

        egui::Area::new(id)
            .fixed_pos(egui::Pos2 {
                x: (-250.0 * scale) * (1.0 - tween_val),
                y: 16.0 * scale,
            })
            .show(ctx, |ui| {
                egui::Frame::NONE
                    .fill(self.config.ui_panel_fill)
                    .inner_margin(egui::Margin::same((10.0 * scale) as i8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(Self::icon(ctx));
                            ui.heading("Hachimi");
                            ui.label(env!("HACHIMI_DISPLAY_VERSION"));
                        });
                        ui.label(&self.splash_sub_str);
                    });
            });
    }
}
