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
use crate::media::stream_player::StreamVideoPlayer;
use crate::ui::export_dialog::{ExportDialog, ExportDialogAction};
use crate::ui::media_bin::{MediaBinAction, MediaBinView};
use crate::ui::menu_bar::{MenuAction, MenuBarView};
use crate::ui::preview_player::{PlayerAction, PreviewPlayerView};
use crate::ui::theme::AppTheme;
use crate::ui::timeline_view::{TimelineAction, TimelineView};
use egui::{Button, ColorImage, Context, Key, RichText, TextureHandle};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct VideoEditorApp {
    pub project: Project,
    pub player: AudioPlayer,
    pub stream_player: StreamVideoPlayer,
    pub frame_cache: FrameCache,
    pub peak_cache: HashMap<String, WaveformPeaks>,
    pub export_dialog: ExportDialog,
    pub preview_texture: Option<TextureHandle>,
    pub current_frame: Option<ColorImage>,
    pub last_frame_time: Option<TimeCode>,
    pub proxy_tasks: HashMap<u64, tokio::sync::watch::Receiver<ProxyStatus>>,
    pub show_help_dialog: bool,
}

impl Default for VideoEditorApp {
    fn default() -> Self {
        Self {
            project: Project::default(),
            player: AudioPlayer::new(),
            stream_player: StreamVideoPlayer::new(),
            frame_cache: FrameCache::new(120),
            peak_cache: HashMap::new(),
            export_dialog: ExportDialog::default(),
            preview_texture: None,
            current_frame: None,
            last_frame_time: None,
            proxy_tasks: HashMap::new(),
            show_help_dialog: false,
        }
    }
}

impl VideoEditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        AppTheme::apply(&cc.egui_ctx);
        Self::default()
    }

    /// Import a media file into the project's media bin and automatically place it on timeline if empty.
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

            // Extract waveform peaks in background for instant audio rendering
            if meta.has_audio {
                if let Ok(peaks) = extract_peaks(p, meta.duration_secs) {
                    self.peak_cache.insert(stem.clone(), peaks);
                }
            }

            // Extract first frame immediately so preview is ready with zero delay
            if meta.has_video {
                if let Some(initial_frame) = self.frame_cache.extract_initial_frame(p) {
                    self.current_frame = Some(initial_frame);
                }

                // Spawn background proxy generator for smooth low-spec playback
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

            self.project.add_asset(asset.clone());

            // Automatically place on timeline
            self.add_asset_to_timeline(asset);

            // Rewind playhead to start
            self.project.timeline.playhead = TimeCode::ZERO;
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
                        .add_track("🎬 Video Track".to_string(), TrackKind::Video)
                });

            if let Some(track) = self.project.timeline.get_track_mut(target_track_id) {
                clip.timeline_start = track.duration();
                track.add_clip(clip);
            }
        } else {
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
                        .add_track("🎵 Music & Sound".to_string(), TrackKind::Audio)
                });

            if let Some(track) = self.project.timeline.get_track_mut(target_track_id) {
                clip.timeline_start = track.duration();
                track.add_clip(clip);
            }
        }

        self.project.timeline.playhead = TimeCode::ZERO;
    }

    pub fn start_playback(&mut self, ctx: &Context) {
        self.player.play();
        self.project.timeline.is_playing = true;

        let playhead = self.project.timeline.playhead;
        if let Some((path, sec)) = self.get_active_video_clip_info(playhead) {
            self.stream_player.start(path, sec, Some(ctx));
        }
    }

    pub fn pause_playback(&mut self) {
        self.player.pause();
        self.project.timeline.is_playing = false;
        self.stream_player.stop();
    }

    pub fn toggle_playback(&mut self, ctx: &Context) {
        if self.project.timeline.is_playing {
            self.pause_playback();
        } else {
            self.start_playback(ctx);
        }
    }

    pub fn stop_playback(&mut self, ctx: &Context) {
        self.pause_playback();
        self.project.timeline.playhead = TimeCode::ZERO;
        self.refresh_preview_frame(Some(ctx));
    }

    pub fn seek_to(&mut self, target_time: TimeCode, ctx: &Context) {
        self.project.timeline.playhead = target_time;
        if self.project.timeline.is_playing {
            if let Some((path, sec)) = self.get_active_video_clip_info(target_time) {
                self.stream_player.start(path, sec, Some(ctx));
            } else {
                self.stream_player.stop();
            }
        } else {
            self.stream_player.stop();
            self.refresh_preview_frame(Some(ctx));
        }
    }

    pub fn get_active_video_clip_info(&self, time: TimeCode) -> Option<(PathBuf, f64)> {
        for track in &self.project.timeline.tracks {
            if track.kind == TrackKind::Video && !track.is_muted {
                if let Some(clip) = track.get_clip_at(time) {
                    if clip.has_video {
                        if let Some(source_time) = clip.timeline_to_source_time(time) {
                            return Some((clip.source_path.clone(), source_time.as_secs_f64()));
                        }
                    }
                }
            }
        }
        None
    }

    /// Update preview frame based on current playhead position with UI repaint callback.
    fn refresh_preview_frame(&mut self, ctx: Option<&Context>) {
        let playhead = self.project.timeline.playhead;

        if let Some((path, sec)) = self.get_active_video_clip_info(playhead) {
            if let Some(frame) = self.frame_cache.fetch_frame(path, sec, ctx) {
                self.current_frame = Some(frame);
            }
        } else {
            self.current_frame = None;
        }

        self.last_frame_time = Some(playhead);
    }
}

impl eframe::App for VideoEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Process Global Keyboard Shortcuts
        if ctx.input(|i| i.key_pressed(Key::Space)) {
            self.toggle_playback(ctx);
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
                    self.refresh_preview_frame(Some(ctx));
                }
            }
        }

        // 2. Playback Clock Step & Auto-stop at timeline duration
        if self.project.timeline.is_playing {
            let max_dur = self.project.timeline.duration();
            if max_dur.as_secs_f64() > 0.0 {
                let new_playhead = self.player.update_playhead(self.project.timeline.playhead, max_dur);
                if new_playhead >= max_dur {
                    self.pause_playback();
                    self.project.timeline.playhead = max_dur;
                } else {
                    self.project.timeline.playhead = new_playhead;
                }
            } else {
                self.pause_playback();
            }

            // Consume frames from the continuous streaming decoder
            if let Some(stream_frame) = self.stream_player.get_next_frame() {
                self.current_frame = Some(stream_frame);
            }

            ctx.request_repaint();
        }

        // ==========================================
        // 5. Render Top Menu Bar (Senior 3-Step Header)
        // ==========================================
        egui::TopBottomPanel::top("top_menu_panel")
            .min_height(50.0)
            .show(ctx, |ui| {
                match MenuBarView::render(ui, &mut self.project) {
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
                            self.refresh_preview_frame(Some(ctx));
                        }
                    }
                    MenuAction::SplitAtPlayhead => {
                        self.project.timeline.split_at_playhead();
                    }
                    MenuAction::DeleteSelected => {
                        self.project.timeline.delete_selected_clips();
                        self.refresh_preview_frame(Some(ctx));
                    }
                    MenuAction::OpenExportDialog => {
                        self.export_dialog.is_open = true;
                    }
                    MenuAction::ToggleHelp => {
                        self.show_help_dialog = !self.show_help_dialog;
                    }
                    MenuAction::None => {}
                }
            });

        // ==========================================
        // 6. Render Left Side Panel: Media Bin
        // ==========================================
        egui::SidePanel::left("left_media_bin_panel")
            .resizable(true)
            .default_width(280.0)
            .min_width(220.0)
            .max_width(450.0)
            .show(ctx, |ui| {
                match MediaBinView::render(ui, &mut self.project) {
                    MediaBinAction::ImportFiles(paths) => {
                        for path in paths {
                            self.import_file(path);
                        }
                        self.refresh_preview_frame(Some(ctx));
                    }
                    MediaBinAction::AddAssetToTimeline(asset) => {
                        self.add_asset_to_timeline(asset);
                        self.refresh_preview_frame(Some(ctx));
                    }
                    MediaBinAction::None => {}
                }
            });

        // ==========================================
        // 7. Render Bottom Panel: Timeline Canvas
        // ==========================================
        egui::TopBottomPanel::bottom("bottom_timeline_panel")
            .resizable(true)
            .default_height(280.0)
            .min_height(200.0)
            .show(ctx, |ui| {
                match TimelineView::render(ui, &mut self.project.timeline, &self.peak_cache) {
                    TimelineAction::Seek(time) => {
                        self.seek_to(time, ctx);
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
                        self.refresh_preview_frame(Some(ctx));
                    }
                    TimelineAction::ClipTrimmed { .. } => {
                        self.refresh_preview_frame(Some(ctx));
                    }
                    TimelineAction::SplitAtPlayhead => {
                        self.project.timeline.split_at_playhead();
                    }
                    TimelineAction::DeleteSelected => {
                        self.project.timeline.delete_selected_clips();
                        self.refresh_preview_frame(Some(ctx));
                    }
                    TimelineAction::AddVideoTrack => {
                        self.project
                            .timeline
                            .add_track("🎬 Video Track".to_string(), TrackKind::Video);
                    }
                    TimelineAction::AddAudioTrack => {
                        self.project
                            .timeline
                            .add_track("🎵 Music & Sound".to_string(), TrackKind::Audio);
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
                    self.toggle_playback(ctx);
                }
                PlayerAction::StepFrames(delta) => {
                    let fps = self.project.timeline.fps;
                    let current_frame = self.project.timeline.playhead.as_frames(fps);
                    let new_frame = (current_frame + delta).max(0);
                    let target = TimeCode::from_frames(new_frame, fps);
                    self.seek_to(target, ctx);
                }
                PlayerAction::StepSeconds(delta_secs) => {
                    let cur = self.project.timeline.playhead.as_secs_f64();
                    let max = self.project.timeline.duration().as_secs_f64();
                    let target_secs = (cur + delta_secs).clamp(0.0, max.max(0.0));
                    let target = TimeCode::from_secs_f64(target_secs);
                    self.seek_to(target, ctx);
                }
                PlayerAction::Seek(time) => {
                    self.seek_to(time, ctx);
                }
                PlayerAction::Stop => {
                    self.stop_playback(ctx);
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

        // ==========================================
        // 10. Senior Help / How-To Dialog Modal
        // ==========================================
        if self.show_help_dialog {
            egui::Window::new("❓ Easy Step-by-Step Guide")
                .collapsible(false)
                .resizable(false)
                .default_width(420.0)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.heading(RichText::new("How to Edit Your Video").color(AppTheme::ACCENT_BLUE).size(18.0));
                        ui.add_space(8.0);

                        ui.label(RichText::new("1. Add Videos & Music:").strong().size(15.0));
                        ui.label(RichText::new("Click the big blue '1. 📂 Open Video / Music' button at the top to choose files from your computer. Your video will appear automatically on the screen.").size(14.0));
                        ui.add_space(8.0);

                        ui.label(RichText::new("2. Watch & Cut Video:").strong().size(15.0));
                        ui.label(RichText::new("Click '▶ PLAY' or tap Spacebar to watch. Click '2. ✂ Cut Video' to slice your video where the red line is. Select unwanted parts and click '🗑 Delete Clip'.").size(14.0));
                        ui.add_space(8.0);

                        ui.label(RichText::new("3. Lower or Fade Sound:").strong().size(15.0));
                        ui.label(RichText::new("On any music clip, click the yellow volume line to create a dot, then drag it down to make the music softer at that point.").size(14.0));
                        ui.add_space(8.0);

                        ui.label(RichText::new("4. Save Your Finished Video:").strong().size(15.0));
                        ui.label(RichText::new("Click the green '3. 🚀 Export Finished Video' button at top right to save your video file.").size(14.0));
                        ui.add_space(12.0);

                        ui.vertical_centered(|ui| {
                            if ui.add(Button::new(RichText::new("Got it, Close").size(15.0).strong()).min_size(egui::vec2(140.0, 36.0))).clicked() {
                                self.show_help_dialog = false;
                            }
                        });
                    });
                });
        }
    }
}
