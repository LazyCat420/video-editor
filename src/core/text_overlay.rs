use egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TextOverlay {
    pub text: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub position: TextPosition,
    #[serde(default)]
    pub style: TextStylePreset,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default = "default_show_box")]
    pub show_box: bool,
}

fn default_font_size() -> f32 {
    30.0
}

fn default_show_box() -> bool {
    true
}

impl Default for TextOverlay {
    fn default() -> Self {
        Self {
            text: String::new(),
            subtitle: None,
            position: TextPosition::BottomBanner,
            style: TextStylePreset::DarkBoxWhiteText,
            font_size: 30.0,
            show_box: true,
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
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TextPosition {
    BottomBanner,
    CenterTitle,
    TopHeader,
    LowerThird,
}

impl Default for TextPosition {
    fn default() -> Self {
        Self::BottomBanner
    }
}

impl TextPosition {
    pub fn label(&self) -> &'static str {
        match self {
            Self::BottomBanner => "Bottom Banner (Recommended)",
            Self::CenterTitle => "Centered Big Title",
            Self::TopHeader => "Top Header",
            Self::LowerThird => "Lower Third Left",
        }
    }

    pub fn all() -> &'static [TextPosition] {
        &[
            Self::BottomBanner,
            Self::CenterTitle,
            Self::TopHeader,
            Self::LowerThird,
        ]
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TextStylePreset {
    DarkBoxWhiteText,
    GoldElegance,
    SunsetGlow,
    CinemaClean,
}

impl Default for TextStylePreset {
    fn default() -> Self {
        Self::DarkBoxWhiteText
    }
}

impl TextStylePreset {
    pub fn label(&self) -> &'static str {
        match self {
            Self::DarkBoxWhiteText => "Classic White + Dark Box",
            Self::GoldElegance => "Warm Vacation Gold",
            Self::SunsetGlow => "Sunset Coral & Orange",
            Self::CinemaClean => "Cinema Pure White",
        }
    }

    pub fn text_color(&self) -> Color32 {
        match self {
            Self::DarkBoxWhiteText => Color32::WHITE,
            Self::GoldElegance => Color32::from_rgb(255, 220, 110),
            Self::SunsetGlow => Color32::from_rgb(255, 185, 120),
            Self::CinemaClean => Color32::from_rgb(245, 245, 255),
        }
    }

    pub fn all() -> &'static [TextStylePreset] {
        &[
            Self::DarkBoxWhiteText,
            Self::GoldElegance,
            Self::SunsetGlow,
            Self::CinemaClean,
        ]
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TitleCardTheme {
    SunsetGlow,
    OceanBlue,
    TropicalCoral,
    WarmSand,
    CinemaBlack,
    EmeraldForest,
}

impl Default for TitleCardTheme {
    fn default() -> Self {
        Self::SunsetGlow
    }
}

impl TitleCardTheme {
    pub fn label(&self) -> &'static str {
        match self {
            Self::SunsetGlow => "🌅 Sunset Glow",
            Self::OceanBlue => "🌊 Ocean Blue",
            Self::TropicalCoral => "🌴 Tropical Coral",
            Self::WarmSand => "🏖 Warm Sand",
            Self::CinemaBlack => "🎬 Cinema Black",
            Self::EmeraldForest => "🌲 Emerald Forest",
        }
    }

    pub fn colors(&self) -> (Color32, Color32) {
        match self {
            Self::SunsetGlow => (Color32::from_rgb(60, 20, 35), Color32::from_rgb(25, 15, 30)),
            Self::OceanBlue => (Color32::from_rgb(15, 35, 65), Color32::from_rgb(10, 20, 40)),
            Self::TropicalCoral => (Color32::from_rgb(55, 25, 30), Color32::from_rgb(20, 20, 35)),
            Self::WarmSand => (Color32::from_rgb(50, 40, 25), Color32::from_rgb(30, 25, 20)),
            Self::CinemaBlack => (Color32::from_rgb(18, 18, 22), Color32::from_rgb(10, 10, 12)),
            Self::EmeraldForest => (Color32::from_rgb(15, 45, 30), Color32::from_rgb(10, 25, 20)),
        }
    }

    pub fn all() -> &'static [TitleCardTheme] {
        &[
            Self::SunsetGlow,
            Self::OceanBlue,
            Self::TropicalCoral,
            Self::WarmSand,
            Self::CinemaBlack,
            Self::EmeraldForest,
        ]
    }
}
