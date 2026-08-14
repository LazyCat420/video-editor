use crate::core::text_overlay::TitleCardBackground;
use egui::{Color32, ColorImage};

/// Generate a solid color ColorImage frame for a Title Card
pub fn generate_solid_color_frame(color: Color32, width: usize, height: usize) -> ColorImage {
    ColorImage {
        size: [width, height],
        pixels: vec![color; width * height],
    }
}

/// Generate or fetch frame for a Title Card background
pub fn generate_title_card_frame(
    bg: &TitleCardBackground,
    width: usize,
    height: usize,
) -> ColorImage {
    match bg {
        TitleCardBackground::SolidColor(color) => generate_solid_color_frame(*color, width, height),
        TitleCardBackground::Picture(_) => {
            // Default dark background placeholder if photo frame is loaded via media frame cache
            generate_solid_color_frame(Color32::from_rgb(18, 18, 24), width, height)
        }
    }
}
