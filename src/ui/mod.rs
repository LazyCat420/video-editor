pub mod components;
pub mod calendar_renderer;
pub mod text_renderer;
pub mod export_dialog;

/// Drag-and-drop payload: a media asset id being dragged from the files panel
/// onto a timeline track to be placed there.
#[derive(Clone, Debug)]
pub struct MediaAssetDrag(pub u64);

/// Drag-and-drop payload: a track id being dragged (by its header) to reorder rows.
#[derive(Clone, Debug)]
pub struct TrackReorderDrag(pub u64);

/// Drag-and-drop payload: slide element index being dragged to reorder in items list.
#[derive(Clone, Debug)]
pub struct SlideElementDrag(pub usize);

pub mod media_bin;
pub mod menu_bar;
pub mod node_graph_view;
pub mod preview_player;
pub mod slide_bin;
pub mod slide_deck;
pub mod theme;
pub mod timeline_view;
pub mod transition_bin;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MainViewMode {
    #[default]
    Slideshow,
    Timeline,
}

impl MainViewMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Slideshow => "🖼 Slideshow Studio",
            Self::Timeline => "⏱ Timeline Editor",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SidebarTab {
    Slides,
    Formatting,
    Transitions,
}

impl Default for SidebarTab {
    fn default() -> Self {
        Self::Slides
    }
}

pub use export_dialog::ExportDialog;
pub use media_bin::MediaBinView;
pub use menu_bar::MenuBarView;
pub use node_graph_view::render_audio_envelope_graph;
pub use preview_player::PreviewPlayerView;
pub use slide_bin::SlideBinView;
pub use slide_deck::{SlideDeckAction, SlideDeckView};
pub use theme::AppTheme;
pub use timeline_view::TimelineView;
pub use transition_bin::{TransitionBinAction, TransitionBinView, TransitionSlot};

use crate::core::text_overlay::{SlideElement, TextOverlay};
use std::path::PathBuf;

/// A slide element waiting to be dropped on the preview frame at a clicked point.
pub enum PendingElement {
    Text(TextOverlay),
    Picture(PathBuf),
    Video(PathBuf),
}

/// Actions emitted by the slide builder panel.
pub enum SlideBinAction {
    None,
    AddBlankSlide { duration: f64 },
    SetActiveBackground(crate::core::text_overlay::SlideBackground),
    AddAudioElement(PathBuf),
    AddTextElement(TextOverlay),
    /// Arm a pending element so the next preview click places it.
    ArmPlace(PendingElement),
    UpdateElement { idx: usize, element: SlideElement },
    UpdateAudioVolume { idx: usize, volume: f32 },
    SelectElement(Option<usize>),
    RemoveElement(usize),
    ReorderElement { idx: usize, dir: i32 },
    ReorderElementTo { from_idx: usize, to_idx: usize },
    FullSlide(usize),
    SetElementAsBackground(usize),
    ApplyTemplateTitle2Media,
    ApplyTemplateTitle4Media,
    ApplyTemplateShowcase,
    ApplyTemplateTitle2MediaToActive,
    ApplyTemplateTitle4MediaToActive,
    ApplyTemplateShowcaseToActive,
    ApplyTemplateCalendarSlide { year: i32, start_month: u32, month_count: u32, show_holidays: bool },
    ApplyTemplateCalendarSlideToActive { year: i32, start_month: u32, month_count: u32, show_holidays: bool },
    Generate12MonthCalendar { year: i32, month_count: u32, show_holidays: bool },
    UpdateActiveCalendarSlide,
    OpenCalendarExportDialog,
}

/// Run `f` with a smaller `interact_size.y` and sleek `slider_rail_height` so horizontal sliders
/// render with a compact, proportional drag knob (egui sizes the knob circle from `spacing.interact_size.y`).
pub fn small_slider<R>(ui: &mut egui::Ui, height: f32, f: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let old_interact = ui.spacing().interact_size;
    let old_rail = ui.spacing().slider_rail_height;
    ui.spacing_mut().interact_size.y = height;
    ui.spacing_mut().slider_rail_height = 4.0;
    let r = f(ui);
    ui.spacing_mut().interact_size = old_interact;
    ui.spacing_mut().slider_rail_height = old_rail;
    r
}
