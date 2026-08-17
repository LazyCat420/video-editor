use serde::{Deserialize, Serialize};

/// A visual transition between two clips, applied at the leading edge of the clip that
/// carries it (it blends with the clip that came right before it on the same track).
///
/// The list is inspired by the transitions found in Premiere Pro, Clipchamp and PPT
/// (dissolves, wipes, slides, irises, radial, zoom, pixelate...). Each maps to an ffmpeg
/// `xfade` transition so it renders at export time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionKind {
    /// Gentle dissolve between the two pictures.
    CrossFade,
    /// Fade out to black, then fade the next picture in.
    DipToBlack,
    /// Fade out to white, then fade the next picture in.
    DipToWhite,
    WipeLeft,
    WipeRight,
    WipeUp,
    WipeDown,
    SlideLeft,
    SlideRight,
    SlideUp,
    SlideDown,
    /// Round iris growing to fill the screen (a "box/iris" style).
    CircleOpen,
    CircleClose,
    Radial,
    ZoomIn,
    SqueezeHorizontal,
    SmoothLeft,
    Pixelate,
}

impl TransitionKind {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::CrossFade => "🔀",
            Self::DipToBlack => "⬛",
            Self::DipToWhite => "⬜",
            Self::WipeLeft => "⬅️",
            Self::WipeRight => "➡️",
            Self::WipeUp => "⬆️",
            Self::WipeDown => "⬇️",
            Self::SlideLeft => "◀️",
            Self::SlideRight => "▶️",
            Self::SlideUp => "🔼",
            Self::SlideDown => "🔽",
            Self::CircleOpen => "⭕",
            Self::CircleClose => "🔘",
            Self::Radial => "🕐",
            Self::ZoomIn => "🔍",
            Self::SqueezeHorizontal => "↔️",
            Self::SmoothLeft => "💫",
            Self::Pixelate => "👾",
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Self::CrossFade => "Cross Fade",
            Self::DipToBlack => "Dip to Black",
            Self::DipToWhite => "Dip to White",
            Self::WipeLeft => "Wipe Left",
            Self::WipeRight => "Wipe Right",
            Self::WipeUp => "Wipe Up",
            Self::WipeDown => "Wipe Down",
            Self::SlideLeft => "Slide Left",
            Self::SlideRight => "Slide Right",
            Self::SlideUp => "Slide Up",
            Self::SlideDown => "Slide Down",
            Self::CircleOpen => "Circle / Box Open",
            Self::CircleClose => "Circle Close",
            Self::Radial => "Radial",
            Self::ZoomIn => "Zoom In",
            Self::SqueezeHorizontal => "Squeeze Horizontal",
            Self::SmoothLeft => "Smooth Slide",
            Self::Pixelate => "Pixelate",
        }
    }

    /// The ffmpeg `xfade` transition name for this kind.
    pub fn to_xfade(&self) -> &'static str {
        match self {
            Self::CrossFade => "dissolve",
            Self::DipToBlack => "fadeblack",
            Self::DipToWhite => "fadewhite",
            Self::WipeLeft => "wipeleft",
            Self::WipeRight => "wiperight",
            Self::WipeUp => "wipeup",
            Self::WipeDown => "wipedown",
            Self::SlideLeft => "slideleft",
            Self::SlideRight => "slideright",
            Self::SlideUp => "slideup",
            Self::SlideDown => "slidedown",
            Self::CircleOpen => "circleopen",
            Self::CircleClose => "circleclose",
            Self::Radial => "radial",
            Self::ZoomIn => "zoomin",
            Self::SqueezeHorizontal => "squeezeh",
            Self::SmoothLeft => "smoothleft",
            Self::Pixelate => "pixelize",
        }
    }

    pub fn all() -> &'static [TransitionKind] {
        &[
            Self::CrossFade,
            Self::DipToBlack,
            Self::DipToWhite,
            Self::WipeLeft,
            Self::WipeRight,
            Self::WipeUp,
            Self::WipeDown,
            Self::SlideLeft,
            Self::SlideRight,
            Self::SlideUp,
            Self::SlideDown,
            Self::CircleOpen,
            Self::CircleClose,
            Self::Radial,
            Self::ZoomIn,
            Self::SqueezeHorizontal,
            Self::SmoothLeft,
            Self::Pixelate,
        ]
    }
}

/// A transition attached to a clip, with its duration in seconds.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    pub kind: TransitionKind,
    /// Length of the blend in seconds.
    pub duration_secs: f64,
}

impl Transition {
    pub fn new(kind: TransitionKind) -> Self {
        Self {
            kind,
            duration_secs: 0.5,
        }
    }
}
