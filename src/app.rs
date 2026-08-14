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
use crate::ui::media_bin::{MediaBinAction, MediaBinView};
use crate::ui::menu_bar::{MenuAction, MenuBarView};
use crate::ui::preview_player::{PlayerAction, PreviewPlayerView};
use crate::ui::theme::AppTheme;
use crate::ui::timeline_view::{TimelineAction, TimelineView};
use egui::{Button, ColorImage, Context, Key, RichText, TextureHandle};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

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
    pub show_help_dialog: bool,
}

impl Default for VideoEditorApp {
    fn default() -> Self {
        Self {
            project: Project::default(),
            player: AudioPlayer::new(),
            stream_player: DualDeckPlayer::new(),
            frame_cache: FrameCache::new(40), // 40 frames max @ 360p (~27MB)
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
            show_help_dialog: false,
        }
    }
}

impl VideoEditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        AppTheme::apply(&cc.egui_ctx);
        Self::default()
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

    /// Import a media file into the project's media bin and automatically place it on timeline if empty.
    pub fn import_file<P: AsRef<Path>>(&mut self, path: P) {
        let p = path.as_ref();
        // Avoid re-importing the same file (e.g. when a whole folder is scanned twice).
        if self.project.media_assets.iter().any(|a| a.path == p) {
            return;
        }
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
                    self.frame_version += 1;
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

    /// Pick the best track to drop `asset` onto, honoring the desired track kind
    /// while preferring the track the user aimed at.
    fn resolve_drop_track(&self, asset: &MediaAsset, preferred_track_id: u64) -> u64 {
        let want_kind = if asset.has_video {
            TrackKind::Video
        } else {
            TrackKind::Audio
        };

        if self
            .project
            .timeline
            .get_track(preferred_track_id)
            .map(|t| t.kind)
            == Some(want_kind)
        {
            preferred_track_id
        } else if let Some(track) = self
            .project
            .timeline
            .tracks
            .iter()
            .find(|t| t.kind == want_kind)
        {
            track.id
        } else {
            preferred_track_id
        }
    }

    /// Place a media asset onto a specific track at a specific timeline start time.
    pub fn place_asset_on_timeline(
        &mut self,
        asset: MediaAsset,
        preferred_track_id: u64,
        start: TimeCode,
    ) {
        let target_track_id = self.resolve_drop_track(&asset, preferred_track_id);
        let clip_id = self.project.timeline.next_id();
        let source_dur = TimeCode::from_secs_f64(asset.duration_secs);

        let mut clip = Clip::new(
            clip_id,
            target_track_id,
            asset.name.clone(),
            asset.path.clone(),
            source_dur,
            asset.has_video,
            asset.has_audio,
        );

        if let Some(track) = self.project.timeline.get_track_mut(target_track_id) {
            clip.timeline_start = start;
            track.add_clip(clip);
        }

        self.project.timeline.playhead = TimeCode::ZERO;
    }

    pub fn start_playback(&mut self, ctx: &Context) {
        self.player.play();
        self.project.timeline.is_playing = true;

        let playhead = self.project.timeline.playhead;
        if let Some((clip_id, path, sec, rem_dur)) = self.get_active_video_clip_info(playhead) {
            self.current_playing_clip_id = Some(clip_id);
            self.stream_player.switch_to_clip(clip_id, path, sec, Some(rem_dur), Some(ctx));
        } else {
            self.current_playing_clip_id = None;
            self.stream_player.stop();
        }
    }

    pub fn pause_playback(&mut self) {
        self.player.pause();
        self.project.timeline.is_playing = false;
        self.current_playing_clip_id = None;
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
            if let Some((clip_id, path, sec, rem_dur)) = self.get_active_video_clip_info(target_time) {
                self.current_playing_clip_id = Some(clip_id);
                self.stream_player.switch_to_clip(clip_id, path, sec, Some(rem_dur), Some(ctx));
            } else {
                self.current_playing_clip_id = None;
                self.stream_player.stop();
            }
        } else {
            self.current_playing_clip_id = None;
            self.stream_player.stop();
            self.refresh_preview_frame(Some(ctx));
        }
    }

    pub fn get_active_video_clip_info(&self, time: TimeCode) -> Option<(u64, PathBuf, f64, f64)> {
        for track in &self.project.timeline.tracks {
            if track.kind == TrackKind::Video && !track.is_muted {
                if let Some(clip) = track.get_clip_at(time) {
                    if clip.has_video {
                        if let Some(source_time) = clip.timeline_to_source_time(time) {
                            let rem_dur = (clip.timeline_end() - time).as_secs_f64().max(0.1);
                            return Some((clip.id, clip.source_path.clone(), source_time.as_secs_f64(), rem_dur));
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

        if let Some((_id, path, sec, _dur)) = self.get_active_video_clip_info(playhead) {
            if let Some(frame) = self.frame_cache.fetch_frame(path, sec, ctx) {
                self.current_frame = Some(frame);
                self.frame_version += 1;
            }
        } else if self.current_frame.is_some() {
            self.current_frame = None;
            self.frame_version += 1;
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
        if ctx.input(|i| i.key_pressed(Key::S) && !i.modifiers.ctrl && !i.modifiers.command) {
            self.snapshot_timeline();
            self.project.timeline.split_at_playhead();
            self.refresh_preview_frame(Some(ctx));
        }
        if ctx.input(|i| (i.modifiers.command || i.modifiers.ctrl) && !i.modifiers.shift && i.key_pressed(Key::Z)) {
            self.undo(Some(ctx));
        }
        if ctx.input(|i| ((i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(Key::Y)) || ((i.modifiers.command || i.modifiers.ctrl) && i.modifiers.shift && i.key_pressed(Key::Z))) {
            self.redo(Some(ctx));
        }
        if ctx.input(|i| (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(Key::C)) {
            if let Some(clip) = self.project.timeline.get_selected_clip() {
                self.clipboard_clip = Some(clip.clone());
            }
        }
        if ctx.input(|i| (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(Key::V)) {
            if let Some(clip) = self.clipboard_clip.clone() {
                self.snapshot_timeline();
                let track_id = self.project.timeline.tracks.first().map(|t| t.id).unwrap_or(0);
                let playhead = self.project.timeline.playhead;
                self.project.timeline.paste_clip(clip, track_id, playhead);
                self.refresh_preview_frame(Some(ctx));
            }
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

            // A. Lookahead pre-warming: pre-warm upcoming clip 0.5s in advance
            let lookahead_time = self.project.timeline.playhead + TimeCode::from_secs_f64(0.5);
            if let Some((up_id, up_path, up_sec, up_dur)) = self.get_active_video_clip_info(lookahead_time) {
                if Some(up_id) != self.current_playing_clip_id {
                    self.stream_player.prewarm(up_id, up_path, up_sec, Some(up_dur), Some(ctx));
                }
            }

            // B. Cross-clip transition detection: switch stream when crossing clips or entering gaps
            let active_clip = self.get_active_video_clip_info(self.project.timeline.playhead);
            let new_clip_id = active_clip.as_ref().map(|(id, _, _, _)| *id);
            if new_clip_id != self.current_playing_clip_id {
                self.current_playing_clip_id = new_clip_id;
                if let Some((clip_id, path, sec, rem_dur)) = &active_clip {
                    self.stream_player.switch_to_clip(*clip_id, path, *sec, Some(*rem_dur), Some(ctx));
                } else {
                    self.stream_player.stop();
                    self.current_frame = None;
                    self.frame_version += 1;
                }
            }

            // C. Consume frames from the continuous streaming decoder synchronized to PTS.
            // Only update the preview when a genuinely NEW frame was decoded, so we do not
            // re-clone a ~691 KB ColorImage (and re-upload it) on every UI tick at 2x the
            // 30 FPS video rate.
            if let Some((_, _, source_sec, _)) = active_clip {
                let (had_new_frame, stream_frame) =
                    self.stream_player.get_frame_for_time(source_sec);
                if had_new_frame {
                    if let Some(f) = stream_frame {
                        self.current_frame = Some(f);
                        self.frame_version += 1;
                    }
                }
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
                match MediaBinView::render(
                    ui,
                    &mut self.project,
                    &mut self.media_bin_collapsed,
                    &self.frame_cache,
                    &mut self.thumb_textures,
                ) {
                    MediaBinAction::ImportFiles(paths) => {
                        for path in paths {
                            self.import_file(path);
                        }
                        self.refresh_preview_frame(Some(ctx));
                    }
                    MediaBinAction::ImportFolder(dir) => {
                        let files = crate::media::probe::scan_folder_for_media(&dir);
                        for path in files {
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
        let can_undo = self.history.can_undo();
        let can_redo = self.history.can_redo();
        let has_clipboard = self.clipboard_clip.is_some();

        egui::TopBottomPanel::bottom("bottom_timeline_panel")
            .resizable(true)
            .default_height(280.0)
            .min_height(200.0)
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
                    }
                    TimelineAction::ClipMoved {
                        clip_id,
                        target_track_id,
                        new_start,
                    } => {
                        self.snapshot_timeline();
                        self.project.timeline.move_clip(clip_id, target_track_id, new_start);
                        self.refresh_preview_frame(Some(ctx));
                    }
                    TimelineAction::ClipTrimmed { .. } => {
                        self.snapshot_timeline();
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
                    TimelineAction::DivideClipInHalf(clip_id) => {
                        self.snapshot_timeline();
                        self.project.timeline.divide_clip_in_half(clip_id);
                        self.refresh_preview_frame(Some(ctx));
                    }
                    TimelineAction::TrimStartToPlayhead(clip_id) => {
                        self.snapshot_timeline();
                        self.project.timeline.trim_clip_start_to_playhead(clip_id);
                        self.refresh_preview_frame(Some(ctx));
                    }
                    TimelineAction::TrimEndToPlayhead(clip_id) => {
                        self.snapshot_timeline();
                        self.project.timeline.trim_clip_end_to_playhead(clip_id);
                        self.refresh_preview_frame(Some(ctx));
                    }
                    TimelineAction::ApplyFadeIn(clip_id) => {
                        self.snapshot_timeline();
                        self.project.timeline.apply_fade_in(clip_id, 1.0);
                    }
                    TimelineAction::ApplyFadeOut(clip_id) => {
                        self.snapshot_timeline();
                        self.project.timeline.apply_fade_out(clip_id, 1.0);
                    }
                    TimelineAction::CopyClip(clip_id) => {
                        if let Some(clip) = self.project.timeline.get_clip(clip_id) {
                            self.clipboard_clip = Some(clip.clone());
                        }
                    }
                    TimelineAction::PasteClip { track_id, target_time } => {
                        if let Some(clip) = self.clipboard_clip.clone() {
                            self.snapshot_timeline();
                            self.project.timeline.paste_clip(clip, track_id, target_time);
                            self.refresh_preview_frame(Some(ctx));
                        }
                    }
                    TimelineAction::DeleteClip(clip_id) => {
                        self.snapshot_timeline();
                        self.project.timeline.delete_clip(clip_id);
                        self.refresh_preview_frame(Some(ctx));
                    }
                    TimelineAction::DeleteSelected => {
                        self.snapshot_timeline();
                        self.project.timeline.delete_selected_clips();
                        self.refresh_preview_frame(Some(ctx));
                    }
                    TimelineAction::DeleteTrack(track_id) => {
                        // Always keep at least one track so there's somewhere to put clips.
                        if self.project.timeline.tracks.len() > 1 {
                            self.snapshot_timeline();
                            self.project.timeline.remove_track(track_id);
                            self.refresh_preview_frame(Some(ctx));
                        }
                    }
                    TimelineAction::Undo => {
                        self.undo(Some(ctx));
                    }
                    TimelineAction::Redo => {
                        self.redo(Some(ctx));
                    }
                    TimelineAction::CloseGaps(track_id_opt) => {
                        self.snapshot_timeline();
                        self.project.timeline.close_gaps(track_id_opt);
                        self.refresh_preview_frame(Some(ctx));
                    }
                    TimelineAction::AddVideoTrack => {
                        self.snapshot_timeline();
                        self.project
                            .timeline
                            .add_track("🎬 Video Track".to_string(), TrackKind::Video);
                    }
                    TimelineAction::AddAudioTrack => {
                        self.snapshot_timeline();
                        self.project
                            .timeline
                            .add_track("🎵 Music & Sound".to_string(), TrackKind::Audio);
                    }
                    TimelineAction::ReorderTrack { from_id, to_index } => {
                        self.snapshot_timeline();
                        self.project.timeline.reorder_track(from_id, to_index);
                    }
                    TimelineAction::AddMediaToTimeline {
                        asset_id,
                        track_id,
                        start,
                    } => {
                        if let Some(asset) = self
                            .project
                            .media_assets
                            .iter()
                            .find(|a| a.id == asset_id)
                            .cloned()
                        {
                            self.snapshot_timeline();
                            self.place_asset_on_timeline(asset, track_id, start);
                            self.refresh_preview_frame(Some(ctx));
                        }
                    }
                    TimelineAction::None => {}
                }
            });

        // ==========================================
        // 8. Render Central Viewport: Preview Player
        // ==========================================
        let is_dirty = self.frame_version != self.last_uploaded_version;
        if is_dirty {
            self.last_uploaded_version = self.frame_version;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            match PreviewPlayerView::render(
                ui,
                &mut self.project.timeline,
                self.current_frame.as_ref(),
                &mut self.preview_texture,
                is_dirty,
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
