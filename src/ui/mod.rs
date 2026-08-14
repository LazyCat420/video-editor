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
pub mod theme;
pub mod timeline_view;

pub use export_dialog::ExportDialog;
pub use media_bin::MediaBinView;
pub use menu_bar::MenuBarView;
pub use node_graph_view::render_audio_envelope_graph;
pub use preview_player::PreviewPlayerView;
pub use theme::AppTheme;
pub use timeline_view::TimelineView;

/// Run `f` with a smaller `interact_size.y` so horizontal sliders render with a shorter
/// track and a much smaller drag knob (egui sizes the knob from `max(text, interact_size.y)`,
/// not from `add_sized`).
pub fn small_slider<R>(ui: &mut egui::Ui, height: f32, f: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let old = (*ui.ctx().style()).clone();
    let mut s = old.clone();
    s.spacing.interact_size.y = height;
    ui.ctx().set_style(s);
    let r = f(ui);
    ui.ctx().set_style(old);
    r
}
