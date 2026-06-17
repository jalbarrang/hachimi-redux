use super::Gui;

macro_rules! add_font {
    ($fonts:expr, $family_fonts:expr, $filename:literal) => {
        $fonts.font_data.insert(
            $filename.to_owned(),
            egui::FontData::from_static(include_bytes!(concat!("../../../assets/fonts/", $filename))).into(),
        );
        $family_fonts.push($filename.to_owned());
    };
}

impl Gui {
    pub(crate) fn get_font_definitions() -> egui::FontDefinitions {
        let mut fonts = egui::FontDefinitions::default();
        let proportional_fonts = fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .expect("unexpected failure");

        add_font!(fonts, proportional_fonts, "Inter_24pt-Regular.ttf");
        add_font!(fonts, proportional_fonts, "AlibabaPuHuiTi-3-45-Light.otf");
        add_font!(fonts, proportional_fonts, "FontAwesome.otf");

        fonts
    }
}
