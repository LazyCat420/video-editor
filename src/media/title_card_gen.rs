use crate::core::text_overlay::TitleCardTheme;
use egui::{Color32, ColorImage};

/// Generate a high-quality gradient backdrop ColorImage for a Title Card
pub fn generate_title_card_frame(theme: TitleCardTheme, width: usize, height: usize) -> ColorImage {
    let (top_col, bot_col) = theme.colors();
    let mut pixels = Vec::with_capacity(width * height);

    for y in 0..height {
        let t = (y as f32) / (height as f32).max(1.0);
        let r = ((top_col.r() as f32) * (1.0 - t) + (bot_col.r() as f32) * t).round() as u8;
        let g = ((top_col.g() as f32) * (1.0 - t) + (bot_col.g() as f32) * t).round() as u8;
        let b = ((top_col.b() as f32) * (1.0 - t) + (bot_col.b() as f32) * t).round() as u8;
        let color = Color32::from_rgb(r, g, b);

        for _ in 0..width {
            pixels.push(color);
        }
    }

    ColorImage {
        size: [width, height],
        pixels,
    }
}
