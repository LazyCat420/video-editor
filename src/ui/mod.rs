pub mod export_dialog;

/// Drag-and-drop payload: a media asset id being dragged from the files panel
/// onto a timeline track to be placed there.
#[derive(Clone, Debug)]
pub struct MediaAssetDrag(pub u64);

/// Drag-and-drop payload: a track id being dragged (by its header) to reorder rows.
#[derive(Clone, Debug)]
pub struct TrackReorderDrag(pub u64);

pub mod media_bin;
pub mod menu_bar;
pub mod node_graph_view;
pub mod preview_player;
pub mod text_bin;
pub mod theme;
pub mod timeline_view;
pub mod transition_bin;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SidebarTab {
    Files,
    Transitions,
    Titles,
}

impl Default for SidebarTab {
    fn default() -> Self {
        Self::Files
    }
}

pub use export_dialog::ExportDialog;
pub use media_bin::MediaBinView;
pub use menu_bar::MenuBarView;
pub use node_graph_view::render_audio_envelope_graph;
pub use preview_player::PreviewPlayerView;
pub use text_bin::{TextBinAction, TextBinView};
pub use theme::AppTheme;
pub use timeline_view::TimelineView;
pub use transition_bin::{TransitionBinAction, TransitionBinView, TransitionSlot};

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
