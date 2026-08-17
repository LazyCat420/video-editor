pub use crate::core::calendar_gen::{CalendarOverlay, CalendarPositionPreset};
pub 

use egui::Color32;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// On-slide text styling. Position is a free, normalized anchor (0..1) so text can be
/// placed by clicking the frame and dragged, both in preview and export.
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
    /// Anchor (center of the text box) as a fraction of the frame, 0..1.
    #[serde(default = "default_center")]
    pub x: f32,
    #[serde(default = "default_center")]
    pub y: f32,
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

fn default_center() -> f32 {
    0.5
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
            x: 0.5,
            y: 0.5,
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

    /// Processed text taking into account ALL CAPS formatting.
    pub fn formatted_text(&self) -> String {
        if self.is_all_caps {
            self.text.to_uppercase()
        } else {
            self.text.clone()
        }
    }
}

/// Ten font styles. Each maps to a bundled preview font (registered in egui under the
/// `preview_family()` name) and an ffmpeg `drawtext font=` name for export. Export names
/// are standard Windows fonts so rendered videos look right on the target machine.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FontFamilyPreset {
    SansSerif,
    Serif,
    Monospace,
    Impact,
    Handwritten,
    Condensed,
    Display,
    VintageSerif,
    Script,
    Futuristic,
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
            Self::Condensed => "Condensed",
            Self::Display => "Poster Display",
            Self::VintageSerif => "Vintage Serif",
            Self::Script => "Elegant Script",
            Self::Futuristic => "Futuristic",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::SansSerif => "Clean, crisp & geometric",
            Self::Serif => "Elegant, literary & timeless",
            Self::Monospace => "Retro code & typewriter",
            Self::Impact => "Heavy, bold blockbuster headline",
            Self::Handwritten => "Casual & personal cursive style",
            Self::Condensed => "Tall, narrow, sports-style",
            Self::Display => "Bold rounded poster lettering",
            Self::VintageSerif => "Classic old-world serif",
            Self::Script => "Flowing calligraphic cursive",
            Self::Futuristic => "Geometric sci-fi lettering",
        }
    }

    pub fn preview_sample(&self) -> &'static str {
        match self {
            Self::SansSerif => "Aa Modern",
            Self::Serif => "Aa Elegant",
            Self::Monospace => "Aa [Type]",
            Self::Impact => "AA IMPACT",
            Self::Handwritten => "Aa Script",
            Self::Condensed => "Aa Narrow",
            Self::Display => "Aa Display",
            Self::VintageSerif => "Aa Vintage",
            Self::Script => "Aa Script",
            Self::Futuristic => "AA Future",
        }
    }

    /// egui FontFamily::Name registered for this preset's bundled TTF.
    pub fn preview_family(&self) -> &'static str {
        match self {
            Self::SansSerif => "ve_sans",
            Self::Serif => "ve_serif",
            Self::Monospace => "ve_mono",
            Self::Impact => "ve_impact",
            Self::Handwritten => "ve_hand",
            Self::Condensed => "ve_condensed",
            Self::Display => "ve_display",
            Self::VintageSerif => "ve_vintage",
            Self::Script => "ve_script",
            Self::Futuristic => "ve_futuristic",
        }
    }

    pub fn ffmpeg_font_name(&self) -> &'static str {
        match self {
            Self::SansSerif => "Arial",
            Self::Serif => "Times New Roman",
            Self::Monospace => "Courier New",
            Self::Impact => "Impact",
            Self::Handwritten => "Comic Sans MS",
            Self::Condensed => "Arial Narrow",
            Self::Display => "Cooper Black",
            Self::VintageSerif => "Georgia",
            Self::Script => "Brush Script MT",
            Self::Futuristic => "Century Gothic",
        }
    }

    pub fn all() -> &'static [FontFamilyPreset] {
        &[
            Self::SansSerif,
            Self::Serif,
            Self::Monospace,
            Self::Impact,
            Self::Handwritten,
            Self::Condensed,
            Self::Display,
            Self::VintageSerif,
            Self::Script,
            Self::Futuristic,
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
            Self::Left => "Left",
            Self::Center => "Center",
            Self::Right => "Right",
        }
    }

    pub fn all() -> &'static [TextAlignment] {
        &[Self::Left, Self::Center, Self::Right]
    }
}

/// Optional solid background behind the text: nothing, a tight rounded box around the
/// letters, or a full-width banner.
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
            Self::None => "No Background",
            Self::TranslucentBox => "Tight Box",
            Self::SolidBanner => "Full Banner",
        }
    }

    pub fn all() -> &'static [TextBoxStyle] {
        &[Self::None, Self::TranslucentBox, Self::SolidBanner]
    }
}

/// LEGACY pre-slide title-card background, kept only so old project files deserialize.
/// Migrated to `SlideBackground` on load. New code always uses `SlideBackground`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TitleCardBackground {
    SolidColor(Color32),
    Picture(PathBuf),
}

/// Background of a blank slide (a slide with no incoming video/image stream).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SlideBackground {
    Solid(Color32),
    Picture(PathBuf),
}

impl Default for SlideBackground {
    fn default() -> Self {
        Self::Solid(Color32::from_rgb(18, 18, 24))
    }
}



/// One element placed on a slide. Text/Picture/Video/Calendar sit in a free box; Audio is mixed.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SlideElement {
    Text(TextOverlay),
    Calendar(CalendarOverlay),
    Picture {
        path: PathBuf,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    Sticker {
        path: PathBuf,
        name: String,
        category: crate::core::stickers::StickerCategory,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    Video {
        path: PathBuf,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    Audio {
        path: PathBuf,
        #[serde(default = "default_volume")]
        volume: f32,
    },
    /// Interactive placeholder slot in a template (e.g. "+ Drop Photo / Video")
    Placeholder {
        slot_id: u32,
        label: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
}

fn default_volume() -> f32 {
    1.0
}

impl SlideElement {
    pub fn is_visual(&self) -> bool {
        !matches!(self, SlideElement::Audio { .. })
    }

    /// Normalized bounding box (x, y, w, h), 0..1 each. Text reports its anchor with 0 size.
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        match self {
            SlideElement::Text(o) => (o.x, o.y, 0.0, 0.0),
            SlideElement::Calendar(c) => (c.x, c.y, c.w, c.h),
            SlideElement::Picture { x, y, w, h, .. }
            | SlideElement::Sticker { x, y, w, h, .. }
            | SlideElement::Video { x, y, w, h, .. }
            | SlideElement::Placeholder { x, y, w, h, .. } => {
                (*x, *y, *w, *h)
            }
            SlideElement::Audio { .. } => (0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn set_bounds(&mut self, x: f32, y: f32, w: f32, h: f32) {
        match self {
            SlideElement::Text(o) => {
                o.x = x.clamp(0.0, 1.0);
                o.y = y.clamp(0.0, 1.0);
            }
            SlideElement::Calendar(c) => {
                c.x = x.clamp(0.0, 1.0);
                c.y = y.clamp(0.0, 1.0);
                c.w = w.clamp(0.05, 1.0);
                c.h = h.clamp(0.05, 1.0);
            }
            SlideElement::Picture { .. }
            | SlideElement::Sticker { .. }
            | SlideElement::Video { .. }
            | SlideElement::Placeholder { .. } => {
                *self = self.with_bounds(x, y, w, h);
            }
            SlideElement::Audio { .. } => {}
        }
    }

    fn with_bounds(&self, x: f32, y: f32, w: f32, h: f32) -> Self {
        let x = x.clamp(0.0, 1.0);
        let y = y.clamp(0.0, 1.0);
        let w = w.clamp(0.01, 1.0);
        let h = h.clamp(0.01, 1.0);
        match self {
            SlideElement::Text(o) => SlideElement::Text(o.clone()),
            SlideElement::Calendar(c) => {
                let mut updated = c.clone();
                updated.x = x;
                updated.y = y;
                updated.w = w;
                updated.h = h;
                SlideElement::Calendar(updated)
            }
            SlideElement::Picture { path, .. } => SlideElement::Picture {
                path: path.clone(),
                x,
                y,
                w,
                h,
            },
            SlideElement::Sticker { path, name, category, .. } => SlideElement::Sticker {
                path: path.clone(),
                name: name.clone(),
                category: *category,
                x,
                y,
                w,
                h,
            },
            SlideElement::Video { path, .. } => SlideElement::Video {
                path: path.clone(),
                x,
                y,
                w,
                h,
            },
            SlideElement::Placeholder { slot_id, label, .. } => SlideElement::Placeholder {
                slot_id: *slot_id,
                label: label.clone(),
                x,
                y,
                w,
                h,
            },
            SlideElement::Audio { path, volume } => SlideElement::Audio {
                path: path.clone(),
                volume: *volume,
            },
        }
    }
}
