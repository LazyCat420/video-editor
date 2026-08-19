pub mod calendar_gen;
pub use calendar_gen::{CalendarMonth, CalendarStyle, CustomCalendarEvent, HolidayCategory, HolidayItem};
pub mod clip;
pub mod effects;
pub use effects::{EffectParticleSimulator, SlideEffect, SlideEffectKind};
pub mod envelope;
pub mod history;
pub mod stickers;
pub use stickers::{StickerCatalog, StickerCategory, StickerHolidayCategory, StickerItem};
pub mod transition;
pub use transition::{Transition, TransitionKind};
pub mod project;
pub mod time;
pub mod timeline;
pub mod track;

pub mod text_overlay;
pub mod text_paint;
pub use text_overlay::{
    CalendarOverlay, FontFamilyPreset, SlideBackground, SlideElement, TextAlignment, TextBoxStyle, TextOverlay,
    TitleCardBackground,
};
pub use text_paint::TextPaint;

pub use clip::Clip;
pub use envelope::{CurveType, VolumeEnvelope, VolumeNode};
pub use history::TimelineHistory;
pub use project::Project;
pub use time::TimeCode;
pub use timeline::Timeline;
pub use track::{Track, TrackKind};
