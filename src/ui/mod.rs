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
