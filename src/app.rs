use crate::audio::player::AudioPlayer;
use crate::core::clip::Clip;
use crate::core::project::{MediaAsset, Project};
use crate::core::time::TimeCode;
use crate::core::track::TrackKind;
use crate::export::renderer::render_project_async;
use crate::media::frame_cache::FrameCache;
use crate::media::peak_extractor::{extract_peaks, WaveformPeaks};
use crate::media::probe::probe_media_file;
use crate::media::proxy_generator::{generate_proxy_async, ProxyStatus};
use crate::ui::export_dialog::{ExportDialog, ExportDialogAction};
use crate::ui::media_bin::{MediaBinAction, MediaBinView};
use crate::ui::menu_bar::{MenuAction, MenuBarView};
use crate::ui::preview_player::{PlayerAction, PreviewPlayerView};
use crate::ui::theme::AppTheme;
use crate::ui::timeline_view::{TimelineAction, TimelineView};
use egui::{ColorImage, Key, TextureHandle};
use std::collections::HashMap;
use std::path::Path;

pub struct VideoEditorApp {
    pub project: Project,
    pub player: AudioPlayer,
    pub frame_cache: FrameCache,
    pub peak_cache: HashMap<String, WaveformPeaks>,
    pub export_dialog: ExportDialog,
    pub preview_texture: Option<TextureHandle>,
    pub current_frame: Option<ColorImage>,
    pub last_frame_time: Option<TimeCode>,
    pub proxy_tasks: HashMap<u64, tokio::sync::watch::Receiver<ProxyStatus>>,
}

impl Default for VideoEditorApp {
    fn default() -> Self {
        Self {
            project: Project::default(),
            player: AudioPlayer::new(),
            frame_cache: FrameCache::new(60),
            peak_cache: HashMap::new(),
            export_dialog: ExportDialog::default(),
            preview_texture: None,
            current_frame: None,
            last_frame_time: None,
            proxy_tasks: HashMap::new(),
        }
    }
}

impl VideoEditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        AppTheme::apply(&cc.egui_ctx);
        Self::default()
    }

    /// Import a media file into the project's media bin.
    pub fn import_file<P: AsRef<Path>>(&mut self, path: P) {
        let p = path.as_ref();
        if let Ok(meta) = probe_media_file(p) {
            let id = self.project.next_asset_id();
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Media")
                .to_string();

            let stem = p
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("media")
                .to_string();

            // Extract waveform peaks in background/sync for instant audio rendering
            if meta.has_audio {
                if let Ok(peaks) = extract_peaks(p, meta.duration_secs) {
                    self.peak_cache.insert(stem.clone(), peaks);
                }
            }

            // Spawn background proxy generator for smooth low-spec playback
            if meta.has_video {
                let rx = generate_proxy_async(p, meta.duration_secs);
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
        }
    }

    /// Add an asset from the media bin directly to the timeline.
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
            // Find first available video track or add one
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

            if let Some(track) = self.project.timeline.get_track_mut(target_track_id) {
                clip.timeline_start = track.duration();
                track.add_clip(clip);
            }
        } else {
            // Find first available audio track or add one
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

            if let Some(track) = self.project.timeline.get_track_mut(target_track_id) {
                clip.timeline_start = track.duration();
                track.add_clip(clip);
            }
        }
    }

    /// Update preview frame based on current playhead position.
    fn refresh_preview_frame(&mut self) {
        let playhead = self.project.timeline.playhead;

        // Check if playhead has changed or frame is missing
        if self.last_frame_time == Some(playhead) && self.current_frame.is_some() {
            return;
        }
        self.last_frame_time = Some(playhead);

        // Find active video clip under playhead
        let mut active_clip_info = None;
        for track in &self.project.timeline.tracks {
            if track.kind == TrackKind::Video && !track.is_muted {
                if let Some(clip) = track.get_clip_at(playhead) {
                    if clip.has_video {
                        if let Some(source_time) = clip.timeline_to_source_time(playhead) {
                            active_clip_info = Some((clip.active_preview_path().clone(), source_time.as_secs_f64()));
                            break;
                        }
                    }
                }
            }
        }

        if let Some((path, sec)) = active_clip_info {
            self.current_frame = self.frame_cache.fetch_frame(path, sec);
        } else {
            self.current_frame = None;
        }
    }
}

impl eframe::App for VideoEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Process Global Keyboard Shortcuts
        if ctx.input(|i| i.key_pressed(Key::Space)) {
            let is_playing = self.player.toggle();
            self.project.timeline.is_playing = is_playing;
        }
        if ctx.input(|i| i.key_pressed(Key::S)) {
            self.project.timeline.split_at_playhead();
        }
        if ctx.input(|i| (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(Key::E)) {
            self.export_dialog.is_open = true;
        }
        if ctx.input(|i| (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(Key::S)) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Video Project", &["vproj", "json"])
                .set_file_name("project.vproj")
                .save_file()
            {
                let _ = self.project.save_to_file(path);
            }
        }
        if ctx.input(|i| (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(Key::O)) {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Video Project", &["vproj", "json"])
                .pick_file()
            {
                if let Ok(loaded) = Project::load_from_file(path) {
                    self.project = loaded;
                }
            }
        }

        // 2. Playback Clock Step
        if self.project.timeline.is_playing {
            let max_dur = self.project.timeline.duration();
            let new_playhead = self.player.update_playhead(self.project.timeline.playhead, max_dur);
            self.project.timeline.playhead = new_playhead;
            ctx.request_repaint();
        }

        // 3. Update Preview Frame Cache
        self.refresh_preview_frame();

        // 4. Update Background Proxy Generation Statuses
        for (asset_id, rx) in &self.proxy_tasks {
            if let ProxyStatus::Ready { ref proxy_path } = *rx.borrow() {
                if let Some(asset) = self.project.media_assets.iter_mut().find(|a| a.id == *asset_id) {
                    asset.proxy_path = Some(proxy_path.clone());
                }
                for track in &mut self.project.timeline.tracks {
                    for clip in &mut track.clips {
                        if let Some(asset) = self.project.media_assets.iter().find(|a| a.id == *asset_id) {
                            if clip.source_path == asset.path {
                                clip.proxy_path = Some(proxy_path.clone());
                            }
                        }
                    }
                }
            }
        }

        // ==========================================
        // 5. Render Top Menu Bar
        // ==========================================
        egui::TopBottomPanel::top("top_menu_panel").show(ctx, |ui| {
            match MenuBarView::render(ui, &mut self.project) {
                MenuAction::NewProject => {
                    self.project = Project::default();
                    self.player.pause();
                }
                MenuAction::OpenProject => {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Video Project", &["vproj", "json"])
                        .pick_file()
                    {
                        if let Ok(loaded) = Project::load_from_file(path) {
                            self.project = loaded;
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
                    if let Some(files) = rfd::FileDialog::new()
                        .add_filter("Media Files", &["mp4", "mkv", "mov", "avi", "webm", "mp3", "wav", "flac", "aac"])
                        .pick_files()
                    {
                        for file in files {
                            self.import_file(file);
                        }
                    }
                }
                MenuAction::SplitAtPlayhead => {
                    self.project.timeline.split_at_playhead();
                }
                MenuAction::DeleteSelected => {
                    self.project.timeline.delete_selected_clips();
                }
                MenuAction::OpenExportDialog => {
                    self.export_dialog.is_open = true;
                }
                MenuAction::None => {}
            }
        });

        // ==========================================
        // 6. Render Left Side Panel: Media Bin
        // ==========================================
        egui::SidePanel::left("left_media_bin_panel")
            .resizable(true)
            .default_width(260.0)
            .min_width(200.0)
            .max_width(450.0)
            .show(ctx, |ui| {
                match MediaBinView::render(ui, &mut self.project) {
                    MediaBinAction::ImportFiles(paths) => {
                        for path in paths {
                            self.import_file(path);
                        }
                    }
                    MediaBinAction::AddAssetToTimeline(asset) => {
                        self.add_asset_to_timeline(asset);
                    }
                    MediaBinAction::None => {}
                }
            });

        // ==========================================
        // 7. Render Bottom Panel: Timeline Canvas
        // ==========================================
        egui::TopBottomPanel::bottom("bottom_timeline_panel")
            .resizable(true)
            .default_height(320.0)
            .min_height(200.0)
            .show(ctx, |ui| {
                match TimelineView::render(ui, &mut self.project.timeline, &self.peak_cache) {
                    TimelineAction::Seek(time) => {
                        self.project.timeline.playhead = time;
                        self.refresh_preview_frame();
                    }
                    TimelineAction::ClipSelected(id) => {
                        self.project.timeline.select_clip(id);
                    }
                    TimelineAction::ClipMoved {
                        clip_id,
                        target_track_id,
                        new_start,
                    } => {
                        self.project.timeline.move_clip(clip_id, target_track_id, new_start);
                    }
                    TimelineAction::ClipTrimmed { .. } => {}
                    TimelineAction::SplitAtPlayhead => {
                        self.project.timeline.split_at_playhead();
                    }
                    TimelineAction::DeleteSelected => {
                        self.project.timeline.delete_selected_clips();
                    }
                    TimelineAction::None => {}
                }
            });

        // ==========================================
        // 8. Render Central Viewport: Preview Player
        // ==========================================
        egui::CentralPanel::default().show(ctx, |ui| {
            match PreviewPlayerView::render(
                ui,
                &mut self.project.timeline,
                self.current_frame.as_ref(),
                &mut self.preview_texture,
            ) {
                PlayerAction::PlayPauseToggle => {
                    let is_playing = self.player.toggle();
                    self.project.timeline.is_playing = is_playing;
                }
                PlayerAction::StepFrames(delta) => {
                    let fps = self.project.timeline.fps;
                    let current_frame = self.project.timeline.playhead.as_frames(fps);
                    let new_frame = (current_frame + delta).max(0);
                    self.project.timeline.playhead = TimeCode::from_frames(new_frame, fps);
                    self.refresh_preview_frame();
                }
                PlayerAction::Seek(time) => {
                    self.project.timeline.playhead = time;
                    self.refresh_preview_frame();
                }
                PlayerAction::Stop => {
                    self.player.pause();
                    self.project.timeline.is_playing = false;
                    self.project.timeline.playhead = TimeCode::ZERO;
                    self.refresh_preview_frame();
                }
                PlayerAction::None => {}
            }
        });

        // ==========================================
        // 9. Render Export Dialog Modal
        // ==========================================
        if self.export_dialog.is_open {
            match self.export_dialog.render(ctx) {
                ExportDialogAction::StartExport(config) => {
                    let rx = render_project_async(self.project.timeline.clone(), config);
                    self.export_dialog.progress_rx = Some(rx);
                }
                ExportDialogAction::Close => {
                    self.export_dialog.is_open = false;
                }
                ExportDialogAction::None => {}
            }
        }
    }
}
