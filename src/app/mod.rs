pub mod calendar_ops;
pub mod canvas_ops;
pub mod playback;
pub mod slide_ops;

use crate::audio::player::AudioPlayer;
use crate::core::clip::Clip;
use crate::core::history::TimelineHistory;
use crate::core::project::{MediaAsset, Project};
use crate::core::time::TimeCode;
use crate::core::track::TrackKind;
use crate::export::renderer::render_project_async;
use crate::media::frame_cache::FrameCache;
use crate::media::peak_extractor::{extract_peaks, WaveformPeaks};
use crate::media::probe::probe_media_file;
use crate::media::proxy_generator::{generate_proxy_async, ProxyStatus};
use crate::media::stream_player::DualDeckPlayer;
use crate::ui::export_dialog::{ExportDialog, ExportDialogAction};
use crate::ui::menu_bar::{MenuAction, MenuBarView};
use crate::ui::preview_player::{PlayerAction, PreviewPlayerView};
use crate::ui::theme::{AppTheme, ThemeKind};
use crate::ui::timeline_view::{TimelineAction, TimelineView};
use egui::{ColorImage, Context, Key, TextureHandle};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// User-chosen program-wide settings (theme + text size).
#[derive(Clone, Copy, Debug)]
pub struct AppSettings {
    pub theme: ThemeKind,
    pub font_scale: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: ThemeKind::Dark,
            font_scale: 1.0,
        }
    }
}

pub struct VideoEditorApp {
    pub project: Project,
    pub player: AudioPlayer,
    pub stream_player: DualDeckPlayer,
    pub frame_cache: FrameCache,
    pub peak_cache: HashMap<String, WaveformPeaks>,
    pub export_dialog: ExportDialog,
    pub preview_texture: Option<TextureHandle>,
    pub current_frame: Option<ColorImage>,
    pub current_playing_clip_id: Option<u64>,
    pub history: TimelineHistory,
    pub clipboard_clip: Option<Clip>,
    pub frame_version: u64,
    pub last_uploaded_version: u64,
    pub last_frame_time: Option<TimeCode>,
    pub proxy_tasks: HashMap<u64, tokio::sync::watch::Receiver<ProxyStatus>>,
    pub media_bin_collapsed: HashSet<String>,
    pub thumb_textures: HashMap<u64, TextureHandle>,
    pub thumbnail_frames: HashMap<u64, ColorImage>,
    pub settings: AppSettings,
    pub sidebar_tab: crate::ui::SidebarTab,
    pub show_settings_dialog: bool,
    pub show_help_dialog: bool,
    /// A slide element armed by the slide panel, dropped on the next preview click.
    pub pending_place: Option<crate::ui::PendingElement>,
    /// Draft text styling shared with the slide panel for a new text element.
    pub text_draft: crate::core::text_overlay::TextOverlay,
    /// Cached egui textures for slide picture/video element frames.
    pub slide_textures: HashMap<PathBuf, TextureHandle>,
    /// Persistent streaming decoders for video elements on active slides.
    pub slide_video_players: HashMap<PathBuf, crate::media::stream_player::StreamVideoPlayer>,
    /// Set of picture file paths that failed to load (to prevent repeated IO stalls).
    pub failed_picture_loads: HashSet<PathBuf>,
    /// Index of the currently selected element on the active slide.
    pub selected_slide_element: Option<usize>,
    pub main_view_mode: crate::ui::MainViewMode,
    pub calendar_year: i32,
    pub calendar_start_month: u32,
    pub calendar_month_count: u32,
    pub calendar_show_holidays: bool,
    pub calendar_style: crate::core::calendar_gen::CalendarStyle,
    pub calendar_holidays: Vec<crate::core::calendar_gen::HolidayItem>,
    pub calendar_custom_events: Vec<crate::core::calendar_gen::CustomCalendarEvent>,
    pub new_custom_event_month: u32,
    pub new_custom_event_day: u32,
    pub new_custom_event_label: String,
    pub new_custom_event_color: [u8; 4],
}

impl Default for VideoEditorApp {
    fn default() -> Self {
        Self {
            project: Project::default(),
            player: AudioPlayer::new(),
            stream_player: DualDeckPlayer::new(),
            frame_cache: FrameCache::new(40),
            peak_cache: HashMap::new(),
            export_dialog: ExportDialog::default(),
            preview_texture: None,
            current_frame: None,
            current_playing_clip_id: None,
            history: TimelineHistory::new(50),
            clipboard_clip: None,
            frame_version: 0,
            last_uploaded_version: 999999,
            last_frame_time: None,
            proxy_tasks: HashMap::new(),
            media_bin_collapsed: HashSet::new(),
            thumb_textures: HashMap::new(),
            thumbnail_frames: HashMap::new(),
            settings: AppSettings::default(),
            show_settings_dialog: false,
            show_help_dialog: false,
            pending_place: None,
            text_draft: crate::core::text_overlay::TextOverlay::new(""),
            slide_textures: HashMap::new(),
            slide_video_players: HashMap::new(),
            failed_picture_loads: HashSet::new(),
            selected_slide_element: None,
            main_view_mode: crate::ui::MainViewMode::Slideshow,
            sidebar_tab: crate::ui::SidebarTab::Formatting,
            calendar_year: 2026,
            calendar_start_month: 1,
            calendar_month_count: 1,
            calendar_show_holidays: true,
            calendar_style: crate::core::calendar_gen::CalendarStyle::BoxedGrid,
            calendar_holidays: crate::core::calendar_gen::CalendarMonth::default_holidays_for_year(2026),
            calendar_custom_events: Vec::new(),
            new_custom_event_month: 1,
            new_custom_event_day: 1,
            new_custom_event_label: "Family Birthday".to_string(),
            new_custom_event_color: [255, 105, 180, 255],
        }
    }
}

impl VideoEditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let s = Self::default();
        AppTheme::configure(&cc.egui_ctx, s.settings.theme, s.settings.font_scale);
        cc.egui_ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
        s
    }

    pub fn snapshot_timeline(&mut self) {
        self.history.push_snapshot(&self.project.timeline);
    }

    pub fn undo(&mut self, ctx: Option<&Context>) {
        if let Some(prev) = self.history.undo(&self.project.timeline) {
            self.project.timeline = prev;
            self.refresh_preview_frame(ctx);
        }
    }

    pub fn redo(&mut self, ctx: Option<&Context>) {
        if let Some(next) = self.history.redo(&self.project.timeline) {
            self.project.timeline = next;
            self.refresh_preview_frame(ctx);
        }
    }

    pub fn add_media_to_bin<P: AsRef<Path>>(&mut self, path: P) -> Option<u64> {
        let p = path.as_ref();
        if let Some(existing) = self.project.media_assets.iter().find(|a| a.path == p) {
            return Some(existing.id);
        }

        let meta = probe_media_file(p).ok()?;
        let id = self.project.timeline.next_id();
        let name = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string();

        if meta.has_audio {
            let (tx, rx) = tokio::sync::watch::channel(ProxyStatus::Generating { progress_pct: 0.0 });
            self.proxy_tasks.insert(id, rx);
            let path_clone = p.to_path_buf();
            std::thread::spawn(move || {
                let _ = tx.send(ProxyStatus::Generating { progress_pct: 0.0 });
                match extract_peaks(&path_clone, 100.0) {
                    Ok(peaks) => {
                        let _ = tx.send(ProxyStatus::Ready { proxy_path: path_clone });
                        let _ = peaks;
                    }
                    Err(e) => {
                        let _ = tx.send(ProxyStatus::Failed { error: e });
                    }
                }
            });
        }

        if meta.has_video && (meta.width > 1280 || meta.height > 720) {
            let rx = generate_proxy_async(p.to_path_buf(), meta.duration_secs);
            self.proxy_tasks.insert(id, rx);
        }

        let asset = MediaAsset {
            id,
            name,
            path: p.to_path_buf(),
            duration_secs: meta.duration_secs,
            width: meta.width,
            height: meta.height,
            fps: meta.fps,
            has_video: meta.has_video,
            has_audio: meta.has_audio,
            proxy_path: None,
            peak_path: None,
        };

        self.project.add_asset(asset);
        Some(id)
    }

    pub fn import_file<P: AsRef<Path>>(&mut self, path: P) -> Option<u64> {
        self.add_media_to_bin(path)
    }

    pub fn add_asset_to_timeline(&mut self, asset: MediaAsset) {
        let source_dur = TimeCode::from_secs_f64(asset.duration_secs);
        let clip_id = self.project.timeline.next_id();

        let mut clip = Clip::new(
            clip_id,
            0,
            asset.name.clone(),
            asset.path.clone(),
            source_dur,
            asset.has_video,
            asset.has_audio,
        );

        if asset.has_video {
            let target_track_id = self
                .project
                .timeline
                .tracks
                .iter()
                .find(|t| t.kind == TrackKind::Video)
                .map(|t| t.id)
                .unwrap_or_else(|| {
                    self.project
                        .timeline
                        .add_track("Video Track".to_string(), TrackKind::Video)
                });
            clip.track_id = target_track_id;
            let start = self
                .project
                .timeline
                .get_track(target_track_id)
                .map(|t| t.duration())
                .unwrap_or(TimeCode::ZERO);
            clip.timeline_start = start;
            if let Some(track) = self.project.timeline.get_track_mut(target_track_id) {
                track.add_clip(clip);
            }
        } else if asset.has_audio {
            let target_track_id = self
                .project
                .timeline
                .tracks
                .iter()
                .find(|t| t.kind == TrackKind::Audio)
                .map(|t| t.id)
                .unwrap_or_else(|| {
                    self.project
                        .timeline
                        .add_track("Audio Track".to_string(), TrackKind::Audio)
                });
            clip.track_id = target_track_id;
            let start = self
                .project
                .timeline
                .get_track(target_track_id)
                .map(|t| t.duration())
                .unwrap_or(TimeCode::ZERO);
            clip.timeline_start = start;
            if let Some(track) = self.project.timeline.get_track_mut(target_track_id) {
                track.add_clip(clip);
            }
        }
    }

    pub(crate) fn resolve_drop_track(&self, asset: &MediaAsset, preferred_track_id: u64) -> u64 {
        let preferred_is_match = self
            .project
            .timeline
            .get_track(preferred_track_id)
            .map(|t| {
                if asset.has_video {
                    t.kind == TrackKind::Video
                } else {
                    t.kind == TrackKind::Audio
                }
            })
            .unwrap_or(false);

        if preferred_is_match {
            return preferred_track_id;
        }

        if asset.has_video {
            self.project
                .timeline
                .tracks
                .iter()
                .find(|t| t.kind == TrackKind::Video)
                .map(|t| t.id)
                .unwrap_or(preferred_track_id)
        } else {
            self.project
                .timeline
                .tracks
                .iter()
                .find(|t| t.kind == TrackKind::Audio)
                .map(|t| t.id)
                .unwrap_or(preferred_track_id)
        }
    }

    pub fn place_asset_on_timeline(
        &mut self,
        asset_id: u64,
        target_track_id: u64,
        start_time: TimeCode,
    ) {
        let asset = self
            .project
            .media_assets
            .iter()
            .find(|a| a.id == asset_id)
            .cloned();
        let Some(asset) = asset else {
            return;
        };

        let resolved_track_id = self.resolve_drop_track(&asset, target_track_id);
        let source_dur = TimeCode::from_secs_f64(asset.duration_secs);
        let clip_id = self.project.timeline.next_id();

        let mut clip = Clip::new(
            clip_id,
            resolved_track_id,
            asset.name.clone(),
            asset.path.clone(),
            source_dur,
            asset.has_video,
            asset.has_audio,
        );

        if let Some(track) = self.project.timeline.get_track_mut(resolved_track_id) {
            let start = start_time.min(track.duration());
            clip.timeline_start = start.max(track.duration());
            track.add_clip(clip);
        }

        self.project.timeline.playhead = TimeCode::ZERO;
    }
}

impl eframe::App for VideoEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle global OS dropped files
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                let paths: Vec<PathBuf> = i.raw.dropped_files.iter().filter_map(|f| f.path.clone()).collect();
                if !paths.is_empty() {
                    self.drop_files_on_canvas(paths, 0.5, 0.5, Some(ctx));
                }
            }
        });

        // ==========================================
        // 1. Audio and Stream Synchronization
        // ==========================================
        if self.project.timeline.is_playing {
            let total_duration = self.project.timeline.duration();
            let current_playhead = self.player.update_playhead(self.project.timeline.playhead, total_duration);
            self.project.timeline.playhead = current_playhead;

            if current_playhead >= total_duration && total_duration.as_secs_f64() > 0.0 {
                self.pause_playback();
                self.project.timeline.playhead = TimeCode::ZERO;
            } else {
                // The picture must follow the playhead, not the selection —
                // and the deck highlight follows the picture.
                self.sync_selection_to_playhead();
                if let Some(active_slide) = self.slide_for_playback().cloned() {
                    let base = self.base_frame_for(&active_slide, Some(ctx));
                    if let Some(base) = base {
                        let final_frame = self.composite_transition(
                            active_slide.track_id,
                            active_slide.id,
                            base,
                            current_playhead,
                            Some(ctx),
                        );
                        self.current_frame = Some(final_frame);
                        self.frame_version += 1;
                    }
                } else if let Some((clip_id, path, sec, rem_dur)) =
                    self.get_active_video_clip_info(current_playhead)
                {
                    if self.current_playing_clip_id != Some(clip_id) {
                        self.current_playing_clip_id = Some(clip_id);
                        self.stream_player.switch_to_clip(
                            clip_id,
                            path.clone(),
                            sec,
                            Some(rem_dur),
                            Some(ctx),
                        );
                    }

                    let (_is_continuous, frame_opt) = self.stream_player.get_frame_for_time(sec);

                    let track_id = self
                        .project
                        .timeline
                        .tracks
                        .iter()
                        .find(|t| t.kind == TrackKind::Video)
                        .map(|t| t.id)
                        .unwrap_or(0);

                    if let Some(f) = frame_opt {
                        let final_frame = self.composite_transition(track_id, clip_id, f, current_playhead, Some(ctx));
                        self.current_frame = Some(final_frame);
                        self.frame_version += 1;
                    } else if let Some(f) = self.frame_cache.fetch_frame(&path, sec, Some(ctx)) {
                        let final_frame = self.composite_transition(track_id, clip_id, f, current_playhead, Some(ctx));
                        self.current_frame = Some(final_frame);
                        self.frame_version += 1;
                    }
                } else {
                    self.current_playing_clip_id = None;
                    self.stream_player.stop();
                }
            }

            ctx.request_repaint();
        } else {
            let playhead = self.project.timeline.playhead;
            if self.last_frame_time != Some(playhead) {
                if let Some(active_slide) = self.slide_to_render().cloned() {
                    let base = self.base_frame_for(&active_slide, Some(ctx));
                    if let Some(base) = base {
                        let final_frame = self.composite_transition(
                            active_slide.track_id,
                            active_slide.id,
                            base,
                            playhead,
                            Some(ctx),
                        );
                        self.current_frame = Some(final_frame);
                        self.frame_version += 1;
                    }
                } else if let Some((clip_id, path, sec, _)) = self.get_active_video_clip_info(playhead) {
                    let track_id = self
                        .project
                        .timeline
                        .tracks
                        .iter()
                        .find(|t| t.kind == TrackKind::Video)
                        .map(|t| t.id)
                        .unwrap_or(0);
                    if let Some(f) = self.frame_cache.fetch_frame(&path, sec, Some(ctx)) {
                        let final_frame = self.composite_transition(track_id, clip_id, f, playhead, Some(ctx));
                        self.current_frame = Some(final_frame);
                        self.frame_version += 1;
                    }
                }
            }
            ctx.request_repaint();
        }

        // ==========================================
        // 2. Render Top Menu Bar
        // ==========================================
        egui::TopBottomPanel::top("top_menu_panel")
            .min_height(50.0)
            .show(ctx, |ui| {
                let is_timeline = self.main_view_mode == crate::ui::MainViewMode::Timeline;
                match MenuBarView::render(ui, &mut self.project, is_timeline) {
                    MenuAction::NewProject => {
                        self.project = Project::default();
                        self.pause_playback();
                        self.current_frame = None;
                    }
                    MenuAction::OpenProject => {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Video Project", &["vproj", "json"])
                            .pick_file()
                        {
                            if let Ok(loaded) = Project::load_from_file(path) {
                                self.project = loaded;
                                self.pause_playback();
                                self.refresh_preview_frame(Some(ctx));
                            }
                        }
                    }
                    MenuAction::SaveProject => {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Video Project", &["vproj", "json"])
                            .set_file_name("project.vproj")
                            .save_file()
                        {
                            let _ = self.project.save_to_file(path);
                        }
                    }
                    MenuAction::ImportMedia => {
                        if let Some(files) = crate::media::probe::create_media_file_dialog().pick_files() {
                            for file in files {
                                self.import_file(file);
                            }
                            self.sidebar_tab = crate::ui::SidebarTab::Slides;
                            self.refresh_preview_frame(Some(ctx));
                        }
                    }
                    MenuAction::SplitAtPlayhead => {
                        self.snapshot_timeline();
                        self.project.timeline.split_at_playhead();
                        self.refresh_preview_frame(Some(ctx));
                    }
                    MenuAction::DeleteSelected => {
                        self.snapshot_timeline();
                        self.project.timeline.delete_selected_clips();
                        self.refresh_preview_frame(Some(ctx));
                    }
                    MenuAction::OpenTransitions => {
                        self.sidebar_tab = crate::ui::SidebarTab::Transitions;
                    }
                    MenuAction::OpenExportDialog => {
                        self.export_dialog.is_open = true;
                    }
                    MenuAction::OpenSettings => {
                        self.show_settings_dialog = true;
                    }
                    MenuAction::None => {}
                }
            });

        // ==========================================
        // 3. Render Left Sidebar Panel (Independent adaptive width per tab)
        // ==========================================
        // Horizontal inner margin of the sidebar frame, subtracted from the panel width to
        // get the body's usable content width. Kept as a constant so the frame margin and
        // the content clamp below can never drift apart.
        const SIDEBAR_INNER_MARGIN_X: f32 = 8.0;

        let sidebar_width = match self.sidebar_tab {
            crate::ui::SidebarTab::Formatting => 280.0,
            crate::ui::SidebarTab::Transitions => 280.0,
            crate::ui::SidebarTab::Slides => 280.0,
        };

        egui::SidePanel::left("video_editor_sidebar_v6")
            .resizable(false)
            .exact_width(sidebar_width)
            .frame(
                egui::Frame::none()
                    .fill(AppTheme::bg_panel())
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 52, 68)))
                    .inner_margin(egui::Margin::symmetric(SIDEBAR_INNER_MARGIN_X, 6.0))
            )
            .show(ctx, |ui| {
                // Hard-cap the body at the panel's exact content width.
                //
                // `exact_width` does NOT cap a SidePanel: egui reports the panel's rect
                // as `content min_rect + margins` with no post-clamp (egui-0.29.1
                // panel.rs:286) and hands that grown rect to the CentralPanel
                // (panel.rs:293, :391) — so any over-wide child opens a dead black gap
                // between sidebar and preview. The previous fix here (`set_max_width` +
                // clip) only made the overflow INVISIBLE: set_max_width is advisory and
                // an oversized allocation is unioned straight back into min_rect
                // (egui layout.rs:49-52). show_width_capped makes it IMPOSSIBLE — the
                // parent advances by exactly content_w regardless of the children.
                let content_w = sidebar_width - 2.0 * SIDEBAR_INNER_MARGIN_X;
                crate::ui::components::show_width_capped(ui, content_w, |ui| {
                if self.main_view_mode == crate::ui::MainViewMode::Slideshow {
                    if self.sidebar_tab == crate::ui::SidebarTab::Slides {
                        self.sidebar_tab = crate::ui::SidebarTab::Formatting;
                    }
                    let (t_format, t_trans) = crate::ui::components::SidebarTabs::render_2_tabs(
                        ui,
                        "🎨 Formatting",
                        self.sidebar_tab == crate::ui::SidebarTab::Formatting,
                        "✨ Transitions",
                        self.sidebar_tab == crate::ui::SidebarTab::Transitions,
                    );
                    if t_format {
                        self.sidebar_tab = crate::ui::SidebarTab::Formatting;
                    }
                    if t_trans {
                        self.sidebar_tab = crate::ui::SidebarTab::Transitions;
                    }
                } else {
                    let (t_slides, t_format, t_trans) = crate::ui::components::SidebarTabs::render_3_tabs(
                        ui,
                        "🗂 Slides",
                        self.sidebar_tab == crate::ui::SidebarTab::Slides,
                        "🎨 Formatting",
                        self.sidebar_tab == crate::ui::SidebarTab::Formatting,
                        "✨ Transitions",
                        self.sidebar_tab == crate::ui::SidebarTab::Transitions,
                    );
                    if t_slides {
                        self.sidebar_tab = crate::ui::SidebarTab::Slides;
                    }
                    if t_format {
                        self.sidebar_tab = crate::ui::SidebarTab::Formatting;
                    }
                    if t_trans {
                        self.sidebar_tab = crate::ui::SidebarTab::Transitions;
                    }
                }

                ui.separator();

                match self.sidebar_tab {
                    crate::ui::SidebarTab::Slides => {
                        use crate::ui::slide_deck::{SlideDeckAction, SlideDeckView};
                        match SlideDeckView::render(ui, self) {
                            SlideDeckAction::None => {}
                            SlideDeckAction::SelectSlide(id) => {
                                self.project.timeline.select_clip(id);
                                if let Some(c) = self.project.timeline.get_clip(id) {
                                    let start = c.timeline_start;
                                    self.seek_to(start, ctx);
                                }
                            }
                            SlideDeckAction::AddBlankSlide { duration } => {
                                self.insert_blank_slide_at_playhead(duration, Some(ctx));
                            }
                            SlideDeckAction::DuplicateSlide(id) => {
                                self.duplicate_slide(id, Some(ctx));
                            }
                            SlideDeckAction::DeleteSlide(id) => {
                                self.delete_slide_by_id(id, Some(ctx));
                            }
                            SlideDeckAction::MoveSlideUp(idx) => {
                                if idx > 0 {
                                    self.reorder_slide(idx, idx - 1, Some(ctx));
                                }
                            }
                            SlideDeckAction::MoveSlideDown(idx) => {
                                self.reorder_slide(idx, idx + 1, Some(ctx));
                            }
                            SlideDeckAction::AdjustSlideDuration { clip_id, delta_secs } => {
                                self.adjust_slide_duration(clip_id, delta_secs, Some(ctx));
                            }
                            SlideDeckAction::DropFilesOnSlide { clip_id, paths } => {
                                self.project.timeline.select_clip(clip_id);
                                self.drop_files_on_canvas(paths, 0.5, 0.5, Some(ctx));
                            }
                        }
                    }
                    crate::ui::SidebarTab::Transitions => {
                        match crate::ui::TransitionBinView::render(ui, &mut self.project.timeline) {
                            crate::ui::TransitionBinAction::SetTransition {
                                clip_id,
                                slot,
                                transition,
                            } => {
                                if self.project.timeline.get_clip(clip_id).is_some() {
                                    self.snapshot_timeline();
                                    if let Some(c) = self.project.timeline.get_clip_mut(clip_id) {
                                        match slot {
                                            crate::ui::TransitionSlot::In => {
                                                c.transition_in = transition;
                                                c.transition = None;
                                            }
                                            crate::ui::TransitionSlot::Out => {
                                                c.transition_out = transition;
                                            }
                                        }
                                    }
                                    self.refresh_preview_frame(Some(ctx));
                                }
                            }
                            crate::ui::TransitionBinAction::None => {}
                        }
                    }
                    crate::ui::SidebarTab::Formatting => {
                        match crate::ui::SlideBinView::render(ui, self) {
                            crate::ui::SlideBinAction::None => {}
                            crate::ui::SlideBinAction::AddBlankSlide { duration } => {
                                self.insert_blank_slide_at_playhead(duration, Some(ctx));
                            }
                            crate::ui::SlideBinAction::SetActiveBackground(bg) => {
                                if let Some(id) = self.active_slide().map(|c| c.id) {
                                    self.snapshot_timeline();
                                    if let Some(clip) = self.project.timeline.get_clip_mut(id) {
                                        clip.background = Some(bg);
                                    }
                                    self.refresh_preview_frame(Some(ctx));
                                }
                            }
                            crate::ui::SlideBinAction::AddAudioElement(path) => {
                                if let Some(id) = self.active_slide().map(|c| c.id) {
                                    self.snapshot_timeline();
                                    if let Some(clip) = self.project.timeline.get_clip_mut(id) {
                                        clip.elements.push(crate::core::text_overlay::SlideElement::Audio { path, volume: 1.0 });
                                    }
                                    self.refresh_preview_frame(Some(ctx));
                                }
                            }
                            crate::ui::SlideBinAction::AddTextElement(overlay) => {
                                if let Some(id) = self.active_slide().map(|c| c.id) {
                                    self.snapshot_timeline();
                                    if let Some(clip) = self.project.timeline.get_clip_mut(id) {
                                        clip.elements.push(crate::core::text_overlay::SlideElement::Text(overlay));
                                        self.selected_slide_element = Some(clip.elements.len() - 1);
                                    }
                                    self.refresh_preview_frame(Some(ctx));
                                }
                            }
                            crate::ui::SlideBinAction::ArmPlace(pending) => {
                                self.pending_place = Some(pending);
                            }
                            crate::ui::SlideBinAction::UpdateElement { idx, element } => {
                                if let Some(id) = self.active_slide().map(|c| c.id) {
                                    if let Some(clip) = self.project.timeline.get_clip_mut(id) {
                                        if let Some(el) = clip.elements.get_mut(idx) {
                                            *el = element;
                                        }
                                    }
                                    self.refresh_preview_frame(Some(ctx));
                                }
                            }
                            crate::ui::SlideBinAction::UpdateAudioVolume { idx, volume } => {
                                if let Some(id) = self.active_slide().map(|c| c.id) {
                                    if let Some(clip) = self.project.timeline.get_clip_mut(id) {
                                        if let Some(crate::core::text_overlay::SlideElement::Audio { volume: v, .. }) = clip.elements.get_mut(idx) {
                                            *v = volume;
                                        }
                                    }
                                }
                            }
                            crate::ui::SlideBinAction::ApplyTemplateTitle2Media => {
                                self.insert_template_title_2_media(Some(ctx));
                            }
                            crate::ui::SlideBinAction::ApplyTemplateTitle4Media => {
                                self.insert_template_title_4_media(Some(ctx));
                            }
                            crate::ui::SlideBinAction::ApplyTemplateShowcase => {
                                self.insert_template_showcase(Some(ctx));
                            }
                            crate::ui::SlideBinAction::ApplyTemplateTitle2MediaToActive => {
                                self.apply_template_title_2_media_to_active(Some(ctx));
                            }
                            crate::ui::SlideBinAction::ApplyTemplateTitle4MediaToActive => {
                                self.apply_template_title_4_media_to_active(Some(ctx));
                            }
                            crate::ui::SlideBinAction::ApplyTemplateShowcaseToActive => {
                                self.apply_template_showcase_to_active(Some(ctx));
                            }
                            crate::ui::SlideBinAction::ApplyTemplateCalendarSlideToActive { year, start_month, month_count, show_holidays } => {
                                self.apply_template_calendar_to_active(year, start_month, month_count, show_holidays, Some(ctx));
                            }
                            crate::ui::SlideBinAction::ApplyTemplateCalendarSlide { year, start_month, month_count, show_holidays } => {
                                self.insert_template_calendar_slide(year, start_month, month_count, show_holidays, Some(ctx));
                            }
                            crate::ui::SlideBinAction::Generate12MonthCalendar { year, month_count, show_holidays } => {
                                self.generate_12_month_calendar(year, month_count, show_holidays, Some(ctx));
                            }
                            crate::ui::SlideBinAction::UpdateActiveCalendarSlide => {
                                self.update_active_calendar_slide(Some(ctx));
                            }
                            crate::ui::SlideBinAction::OpenCalendarExportDialog => {
                                let default_dir = std::env::temp_dir().join("Printable_Calendar");
                                let _ = self.export_printable_calendar_sheets(&default_dir, self.calendar_year, self.calendar_month_count, self.calendar_show_holidays);
                            }
                            crate::ui::SlideBinAction::SelectElement(idx) => {
                                self.selected_slide_element = idx;
                            }
                            crate::ui::SlideBinAction::RemoveElement(idx) => {
                                self.delete_slide_element(idx, Some(ctx));
                            }
                            crate::ui::SlideBinAction::FullSlide(idx) => {
                                self.full_slide_element(idx, Some(ctx));
                            }
                            crate::ui::SlideBinAction::SetElementAsBackground(idx) => {
                                self.set_element_as_background(idx, Some(ctx));
                            }
                            crate::ui::SlideBinAction::ReorderElement { idx, dir } => {
                                if let Some(id) = self.active_slide().map(|c| c.id) {
                                    self.snapshot_timeline();
                                    if let Some(clip) = self.project.timeline.get_clip_mut(id) {
                                        let target = if dir < 0 { idx.saturating_sub(1) } else { (idx + 1).min(clip.elements.len().saturating_sub(1)) };
                                        if target != idx && idx < clip.elements.len() {
                                            let el = clip.elements.remove(idx);
                                            clip.elements.insert(target, el);
                                            self.selected_slide_element = Some(target);
                                        }
                                    }
                                    self.refresh_preview_frame(Some(ctx));
                                }
                            }
                            crate::ui::SlideBinAction::ReorderElementTo { from_idx, to_idx } => {
                                if let Some(id) = self.active_slide().map(|c| c.id) {
                                    self.snapshot_timeline();
                                    if let Some(clip) = self.project.timeline.get_clip_mut(id) {
                                        if from_idx < clip.elements.len() && to_idx < clip.elements.len() {
                                            let el = clip.elements.remove(from_idx);
                                            clip.elements.insert(to_idx, el);
                                            self.selected_slide_element = Some(to_idx);
                                        }
                                    }
                                    self.refresh_preview_frame(Some(ctx));
                                }
                            }
                        }
                    }
                }
                }); // end show_width_capped body
            });

        // ==========================================
        // 4. Render Bottom Panel (Slideshow Studio Bar vs Multi-Track Timeline)
        // ==========================================
        if self.main_view_mode == crate::ui::MainViewMode::Slideshow {
            egui::TopBottomPanel::bottom("bottom_slideshow_panel")
                .resizable(true)
                .default_height(190.0)
                .min_height(140.0)
                .max_height(320.0)
                .show(ctx, |ui| {
                    self.render_bottom_slideshow_bar(ui, ctx);
                });
        } else {
            let can_undo = self.history.can_undo();
            let can_redo = self.history.can_redo();
            let has_clipboard = self.clipboard_clip.is_some();

            egui::TopBottomPanel::bottom("bottom_timeline_panel")
                .resizable(true)
                .default_height(280.0)
                .min_height(160.0)
                .max_height(500.0)
                .show(ctx, |ui| {
                    match TimelineView::render(
                        ui,
                        &mut self.project.timeline,
                        &self.peak_cache,
                        can_undo,
                        can_redo,
                        has_clipboard,
                    ) {
                        TimelineAction::Seek(time) => {
                            self.seek_to(time, ctx);
                        }
                        TimelineAction::ClipSelected(id) => {
                            self.project.timeline.select_clip(id);
                            self.refresh_preview_frame(Some(ctx));
                        }
                        TimelineAction::ClipMoved { clip_id, target_track_id, new_start } => {
                            self.snapshot_timeline();
                            if let Some(clip) = self.project.timeline.get_clip_mut(clip_id) {
                                clip.track_id = target_track_id;
                                clip.timeline_start = new_start;
                            }
                            self.refresh_preview_frame(Some(ctx));
                        }
                        TimelineAction::ClipTrimmed { clip_id, new_in, new_out, new_start } => {
                            self.snapshot_timeline();
                            if let Some(clip) = self.project.timeline.get_clip_mut(clip_id) {
                                clip.source_in = new_in;
                                clip.source_out = new_out;
                                clip.timeline_start = new_start;
                            }
                            self.refresh_preview_frame(Some(ctx));
                        }
                        TimelineAction::SplitAtPlayhead => {
                            self.snapshot_timeline();
                            self.project.timeline.split_at_playhead();
                            self.refresh_preview_frame(Some(ctx));
                        }
                        TimelineAction::SplitClipAtTime { clip_id, split_time } => {
                            self.snapshot_timeline();
                            self.project.timeline.split_clip_at_time(clip_id, split_time);
                            self.refresh_preview_frame(Some(ctx));
                        }
                        TimelineAction::DivideClipInHalf(id) => {
                            if let Some(clip) = self.project.timeline.get_clip(id) {
                                let half_secs = clip.duration().as_secs_f64() / 2.0;
                                let mid = clip.timeline_start + TimeCode::from_secs_f64(half_secs);
                                self.snapshot_timeline();
                                self.project.timeline.split_clip_at_time(id, mid);
                                self.refresh_preview_frame(Some(ctx));
                            }
                        }
                        TimelineAction::TrimStartToPlayhead(id) => {
                            self.snapshot_timeline();
                            let playhead = self.project.timeline.playhead;
                            if let Some(clip) = self.project.timeline.get_clip_mut(id) {
                                if playhead > clip.timeline_start && playhead < clip.timeline_end() {
                                    let delta = playhead - clip.timeline_start;
                                    clip.source_in = clip.source_in + delta;
                                    clip.timeline_start = playhead;
                                }
                            }
                            self.refresh_preview_frame(Some(ctx));
                        }
                        TimelineAction::TrimEndToPlayhead(id) => {
                            self.snapshot_timeline();
                            let playhead = self.project.timeline.playhead;
                            if let Some(clip) = self.project.timeline.get_clip_mut(id) {
                                if playhead > clip.timeline_start && playhead < clip.timeline_end() {
                                    let delta = playhead - clip.timeline_start;
                                    clip.source_out = clip.source_in + delta;
                                }
                            }
                            self.refresh_preview_frame(Some(ctx));
                        }
                        TimelineAction::ApplyFadeIn(id) => {
                            self.snapshot_timeline();
                            if let Some(clip) = self.project.timeline.get_clip_mut(id) {
                                clip.transition_in = Some(crate::core::Transition { kind: crate::core::TransitionKind::CrossFade, duration_secs: 1.0 });
                            }
                            self.refresh_preview_frame(Some(ctx));
                        }
                        TimelineAction::ApplyFadeOut(id) => {
                            self.snapshot_timeline();
                            if let Some(clip) = self.project.timeline.get_clip_mut(id) {
                                clip.transition_out = Some(crate::core::Transition { kind: crate::core::TransitionKind::CrossFade, duration_secs: 1.0 });
                            }
                            self.refresh_preview_frame(Some(ctx));
                        }
                        TimelineAction::CopyClip(id) => {
                            if let Some(c) = self.project.timeline.get_clip(id) {
                                self.clipboard_clip = Some(c.clone());
                            }
                        }
                        TimelineAction::PasteClip { track_id, target_time } => {
                            if let Some(c) = self.clipboard_clip.clone() {
                                self.snapshot_timeline();
                                let mut dup = c;
                                dup.id = self.project.timeline.next_id();
                                dup.track_id = track_id;
                                dup.timeline_start = target_time;
                                if let Some(t) = self.project.timeline.get_track_mut(track_id) {
                                    t.add_clip(dup);
                                }
                                self.refresh_preview_frame(Some(ctx));
                            }
                        }
                        TimelineAction::DeleteClip(id) => {
                            self.snapshot_timeline();
                            for t in &mut self.project.timeline.tracks {
                                t.clips.retain(|c| c.id != id);
                            }
                            self.refresh_preview_frame(Some(ctx));
                        }
                        TimelineAction::DeleteSelected => {
                            self.snapshot_timeline();
                            self.project.timeline.delete_selected_clips();
                            self.refresh_preview_frame(Some(ctx));
                        }
                        TimelineAction::DeleteTrack(id) => {
                            self.snapshot_timeline();
                            self.project.timeline.tracks.retain(|t| t.id != id);
                            self.refresh_preview_frame(Some(ctx));
                        }
                        TimelineAction::SetTransition { clip_id, slot, transition } => {
                            self.snapshot_timeline();
                            if let Some(c) = self.project.timeline.get_clip_mut(clip_id) {
                                match slot {
                                    crate::ui::transition_bin::TransitionSlot::In => c.transition_in = transition,
                                    crate::ui::transition_bin::TransitionSlot::Out => c.transition_out = transition,
                                }
                            }
                            self.refresh_preview_frame(Some(ctx));
                        }
                        TimelineAction::ReorderTrack { from_id, to_index } => {
                            self.snapshot_timeline();
                            if let Some(pos) = self.project.timeline.tracks.iter().position(|t| t.id == from_id) {
                                if pos < self.project.timeline.tracks.len() && to_index < self.project.timeline.tracks.len() {
                                    let t = self.project.timeline.tracks.remove(pos);
                                    self.project.timeline.tracks.insert(to_index, t);
                                }
                            }
                            self.refresh_preview_frame(Some(ctx));
                        }
                        TimelineAction::AddMediaToTimeline { asset_id, track_id, start } => {
                            self.snapshot_timeline();
                            self.place_asset_on_timeline(asset_id, track_id, start);
                            self.refresh_preview_frame(Some(ctx));
                        }
                        TimelineAction::Undo => {
                            self.undo(Some(ctx));
                        }
                        TimelineAction::Redo => {
                            self.redo(Some(ctx));
                        }
                        TimelineAction::CloseGaps(track_opt) => {
                            self.snapshot_timeline();
                            self.project.timeline.close_gaps(track_opt);
                            self.refresh_preview_frame(Some(ctx));
                        }
                        TimelineAction::AddVideoTrack => {
                            self.snapshot_timeline();
                            self.project.timeline.add_track("Video Track".to_string(), TrackKind::Video);
                        }
                        TimelineAction::AddAudioTrack => {
                            self.snapshot_timeline();
                            self.project.timeline.add_track("Audio Track".to_string(), TrackKind::Audio);
                        }
                        TimelineAction::AddBlankSlide { duration } => {
                            self.insert_blank_slide_at_playhead(duration, Some(ctx));
                        }
                        TimelineAction::None => {}
                    }
                });
        }

        // ==========================================
        // 5. Render Central Preview Player Panel
        // ==========================================
        let is_dirty = self.frame_version != self.last_uploaded_version;
        if is_dirty {
            self.last_uploaded_version = self.frame_version;
        }

        let place_mode = self.pending_place.is_some();

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(14, 16, 22))
                    .inner_margin(egui::Margin::symmetric(6.0, 6.0))
            )
            .show(ctx, |ui| {
            let visuals = self.slide_visuals(Some(ctx));
            match PreviewPlayerView::render(
                ui,
                &self.project.timeline,
                self.current_frame.as_ref(),
                &visuals,
                &mut self.preview_texture,
                is_dirty,
                place_mode,
                self.selected_slide_element,
                &mut self.main_view_mode,
            ) {
                PlayerAction::PlayPauseToggle => {
                    self.toggle_playback(ctx);
                }
                PlayerAction::Seek(time) => {
                    self.seek_to(time, ctx);
                }
                PlayerAction::StepSeconds(secs) => {
                    let cur = self.project.timeline.playhead.as_secs_f64();
                    let target = (cur + secs).max(0.0);
                    self.seek_to(TimeCode::from_secs_f64(target), ctx);
                }
                PlayerAction::Stop => {
                    self.stop_playback(ctx);
                }
                PlayerAction::PlaceAt { x, y } => {
                    self.place_pending_element(x, y, Some(ctx));
                }
                PlayerAction::MoveElement { idx, x, y } => {
                    self.move_slide_element(idx, x, y);
                }
                PlayerAction::ResizeElement { idx, x, y, w, h } => {
                    self.resize_slide_element(idx, x, y, w, h);
                }
                PlayerAction::ScaleTextSize { idx, font_size } => {
                    self.scale_text_element(idx, font_size);
                }
                PlayerAction::FullSlide { idx } => {
                    self.full_slide_element(idx, Some(ctx));
                }
                PlayerAction::SetAsBackground { idx } => {
                    self.set_element_as_background(idx, Some(ctx));
                }
                PlayerAction::SelectElement(idx) => {
                    self.selected_slide_element = idx;
                }
                PlayerAction::DeleteElement(idx) => {
                    self.delete_slide_element(idx, Some(ctx));
                }
                PlayerAction::DropMediaAsset { asset_id, x, y } => {
                    self.drop_media_asset_on_canvas(asset_id, x, y, Some(ctx));
                }
                PlayerAction::DropFiles { paths, x, y } => {
                    self.drop_files_on_canvas(paths, x, y, Some(ctx));
                }
                PlayerAction::StepFrames(frames) => {
                    let fps = self.project.timeline.fps.max(1.0);
                    let secs = (frames as f64) / fps;
                    let cur = self.project.timeline.playhead.as_secs_f64();
                    let target = (cur + secs).max(0.0);
                    self.seek_to(TimeCode::from_secs_f64(target), ctx);
                }
                PlayerAction::UpdateTextContent { idx, text } => {
                    if let Some(id) = self.active_slide().map(|c| c.id) {
                        if let Some(clip) = self.project.timeline.get_clip_mut(id) {
                            if let Some(crate::core::text_overlay::SlideElement::Text(o)) = clip.elements.get_mut(idx) {
                                o.text = text;
                            }
                        }
                    }
                }
                PlayerAction::None => {}
            }
        });

        // ==========================================
        // 6. Global Keyboard Shortcuts
        // ==========================================
        if ctx.input(|i| i.key_pressed(Key::Space)) && !ctx.wants_keyboard_input() {
            self.toggle_playback(ctx);
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::Z)) {
            if ctx.input(|i| i.modifiers.shift) {
                self.redo(Some(ctx));
            } else {
                self.undo(Some(ctx));
            }
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::Y)) {
            self.redo(Some(ctx));
        }

        // ==========================================
        // 7. Modals: Export, Settings, Help
        // ==========================================
                if self.export_dialog.is_open {
            match self.export_dialog.render(ctx) {
                ExportDialogAction::StartExportVideo(config) => {
                    let rx = render_project_async(self.project.timeline.clone(), config);
                    self.export_dialog.progress_rx = Some(rx);
                }
                ExportDialogAction::StartExportPptx(path) => {
                    match crate::export::export_to_pptx(&self.project.timeline, &path) {
                        Ok(()) => self.export_dialog.export_status = Some(Ok(path)),
                        Err(e) => self.export_dialog.export_status = Some(Err(e.to_string())),
                    }
                }
                ExportDialogAction::StartExportPdf(path) => {
                    match crate::export::export_to_pdf(&self.project.timeline, &path) {
                        Ok(()) => self.export_dialog.export_status = Some(Ok(path)),
                        Err(e) => self.export_dialog.export_status = Some(Err(e.to_string())),
                    }
                }
                ExportDialogAction::Close => {
                    self.export_dialog.is_open = false;
                }
                ExportDialogAction::None => {}
            }
        }

if self.show_settings_dialog {
            egui::Window::new("⚙ Settings")
                .collapsible(false)
                .resizable(false)
                .open(&mut self.show_settings_dialog)
                .show(ctx, |ui| {
                    ui.heading("Appearance");
                    ui.horizontal(|ui| {
                        ui.label("Theme:");
                        if ui.selectable_label(self.settings.theme == ThemeKind::Dark, "🌙 Dark").clicked() {
                            self.settings.theme = ThemeKind::Dark;
                            AppTheme::configure(ctx, ThemeKind::Dark, self.settings.font_scale);
                        }
                        if ui.selectable_label(self.settings.theme == ThemeKind::Light, "☀ Light").clicked() {
                            self.settings.theme = ThemeKind::Light;
                            AppTheme::configure(ctx, ThemeKind::Light, self.settings.font_scale);
                        }
                    });
                });
        }

        if self.show_help_dialog {
            egui::Window::new("❓ Help & Shortcuts")
                .collapsible(false)
                .open(&mut self.show_help_dialog)
                .show(ctx, |ui| {
                    ui.heading("Keyboard Shortcuts");
                    ui.label("• Space: Play / Pause");
                    ui.label("• Ctrl+Z / Ctrl+Y: Undo / Redo");
                    ui.label("• Delete: Remove Selected Slide Item");
                    ui.label("• Esc: Deselect Item");
                });
        }
    }
}
