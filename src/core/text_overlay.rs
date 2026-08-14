use egui::Color32;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TextOverlay {
    pub text: String,
    #[serde(default)]
    pub font_family: FontFamilyPreset,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default)]
    pub is_bold: bool,
    #[serde(default)]
    pub is_italic: bool,
    #[serde(default)]
    pub is_all_caps: bool,
    #[serde(default)]
    pub alignment: TextAlignment,
    #[serde(default)]
    pub position: TextPosition,
    #[serde(default = "default_text_color")]
    pub text_color: Color32,
    #[serde(default)]
    pub box_style: TextBoxStyle,
    #[serde(default = "default_box_opacity")]
    pub box_opacity: f32,
    #[serde(default = "default_true")]
    pub show_shadow: bool,
}

fn default_font_size() -> f32 {
    38.0
}

fn default_text_color() -> Color32 {
    Color32::WHITE
}

fn default_box_opacity() -> f32 {
    0.65
}

fn default_true() -> bool {
    true
}

impl Default for TextOverlay {
    fn default() -> Self {
        Self {
            text: String::new(),
            font_family: FontFamilyPreset::SansSerif,
            font_size: 38.0,
            is_bold: true,
            is_italic: false,
            is_all_caps: false,
            alignment: TextAlignment::Center,
            position: TextPosition::Center,
            text_color: Color32::WHITE,
            box_style: TextBoxStyle::None,
            box_opacity: 0.65,
            show_shadow: true,
        }
    }
}

impl TextOverlay {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Default::default()
        }
    }

    /// Processed text taking into account ALL CAPS formatting
    pub fn formatted_text(&self) -> String {
        if self.is_all_caps {
            self.text.to_uppercase()
        } else {
            self.text.clone()
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FontFamilyPreset {
    SansSerif,
    Serif,
    Monospace,
    Impact,
    Handwritten,
}

impl Default for FontFamilyPreset {
    fn default() -> Self {
        Self::SansSerif
    }
}

impl FontFamilyPreset {
    pub fn label(&self) -> &'static str {
        match self {
            Self::SansSerif => "Modern Sans",
            Self::Serif => "Classic Serif",
            Self::Monospace => "Typewriter Mono",
            Self::Impact => "Cinematic Impact",
            Self::Handwritten => "Handwritten Script",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::SansSerif => "Clean, crisp & geometric",
            Self::Serif => "Elegant, literary & timeless",
            Self::Monospace => "Retro code & typewriter",
            Self::Impact => "Heavy, bold blockbuster headline",
            Self::Handwritten => "Casual & personal cursive style",
        }
    }

    pub fn preview_sample(&self) -> &'static str {
        match self {
            Self::SansSerif => "Aa Modern",
            Self::Serif => "Aa Elegant",
            Self::Monospace => "Aa [Type]",
            Self::Impact => "AA IMPACT",
            Self::Handwritten => "Aa Script",
        }
    }

    pub fn ffmpeg_font_name(&self) -> &'static str {
        match self {
            Self::SansSerif => "Arial",
            Self::Serif => "Times New Roman",
            Self::Monospace => "Courier New",
            Self::Impact => "Impact",
            Self::Handwritten => "Comic Sans MS",
        }
    }

    pub fn all() -> &'static [FontFamilyPreset] {
        &[
            Self::SansSerif,
            Self::Serif,
            Self::Monospace,
            Self::Impact,
            Self::Handwritten,
        ]
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

impl Default for TextAlignment {
    fn default() -> Self {
        Self::Center
    }
}

impl TextAlignment {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Left => "Left Align",
            Self::Center => "Center Align",
            Self::Right => "Right Align",
        }
    }

    pub fn all() -> &'static [TextAlignment] {
        &[Self::Left, Self::Center, Self::Right]
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TextPosition {
    TopHeader,
    Center,
    BottomBanner,
    LowerThird,
}

impl Default for TextPosition {
    fn default() -> Self {
        Self::Center
    }
}

impl TextPosition {
    pub fn label(&self) -> &'static str {
        match self {
            Self::TopHeader => "Top Header",
            Self::Center => "Dead Center",
            Self::BottomBanner => "Bottom Banner",
            Self::LowerThird => "Lower Third Left",
        }
    }

    pub fn all() -> &'static [TextPosition] {
        &[
            Self::Center,
            Self::BottomBanner,
            Self::TopHeader,
            Self::LowerThird,
        ]
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TextBoxStyle {
    None,
    TranslucentBox,
    SolidBanner,
}

impl Default for TextBoxStyle {
    fn default() -> Self {
        Self::None
    }
}

impl TextBoxStyle {
    pub fn label(&self) -> &'static str {
        match self {
            Self::None => "None (Shadow Only)",
            Self::TranslucentBox => "Translucent Box",
            Self::SolidBanner => "Full Solid Banner",
        }
    }

    pub fn all() -> &'static [TextBoxStyle] {
        &[Self::None, Self::TranslucentBox, Self::SolidBanner]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TitleCardBackground {
    SolidColor(Color32),
    Picture(PathBuf),
}

impl Default for TitleCardBackground {
    fn default() -> Self {
        Self::SolidColor(Color32::from_rgb(18, 18, 24))
    }
}
