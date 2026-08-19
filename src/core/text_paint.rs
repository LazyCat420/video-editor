use egui::Color32;

/// Canonical paint specification for text rendering across preview, FFmpeg, and PDF.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextPaint {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl TextPaint {
    pub fn from_color32(c: Color32) -> Self {
        let [r, g, b, a] = c.to_srgba_unmultiplied();
        Self { r, g, b, a }
    }

    /// Normalized RGB for PDF content stream (0.0 to 1.0)
    pub fn to_pdf_rgb(&self) -> (f64, f64, f64) {
        (
            self.r as f64 / 255.0,
            self.g as f64 / 255.0,
            self.b as f64 / 255.0,
        )
    }

    /// FFmpeg drawtext fontcolor specification: `0xRRGGBB` or `0xRRGGBB@alpha`
    pub fn to_ffmpeg_fontcolor(&self) -> String {
        let alpha = self.a as f32 / 255.0;
        if self.a >= 254 {
            format!("0x{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("0x{:02X}{:02X}{:02X}@{:.2}", self.r, self.g, self.b, alpha)
        }
    }

    /// Convert back to egui Color32
    pub fn to_color32(&self) -> Color32 {
        Color32::from_rgba_unmultiplied(self.r, self.g, self.b, self.a)
    }
}
