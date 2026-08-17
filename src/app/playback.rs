use crate::app::VideoEditorApp;
use crate::core::clip::Clip;
use crate::core::time::TimeCode;
use crate::core::track::TrackKind;
use egui::{ColorImage, Context};
use std::path::PathBuf;

impl VideoEditorApp {
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
        for player in self.slide_video_players.values_mut() {
            player.stop();
        }
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
        for player in self.slide_video_players.values_mut() {
            player.stop();
        }
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
                    } else if clip.is_static_slide() {
                        let mut best_media: Option<(PathBuf, f64)> = None;
                        let mut max_dur: f64 = 0.0;

                        for el in &clip.elements {
                            match el {
                                crate::core::text_overlay::SlideElement::Video { path, .. }
                                | crate::core::text_overlay::SlideElement::Audio { path, .. } => {
                                    let dur = self.project.media_assets.iter()
                                        .find(|a| &a.path == path)
                                        .map(|a| a.duration_secs)
                                        .or_else(|| crate::media::probe::probe_media_file(path).ok().map(|m| m.duration_secs))
                                        .unwrap_or(0.0);
                                    if dur >= max_dur {
                                        max_dur = dur;
                                        best_media = Some((path.clone(), dur));
                                    }
                                }
                                _ => {}
                            }
                        }

                        if let Some((path, _)) = best_media {
                            let elapsed = (time - clip.timeline_start).as_secs_f64().max(0.0);
                            let rem_dur = (clip.timeline_end() - time).as_secs_f64().max(0.1);
                            return Some((clip.id, path, elapsed, rem_dur));
                        }
                    }
                }
            }
        }
        None
    }

    pub(crate) fn composite_transition(
        &mut self,
        track_id: u64,
        clip_id: u64,
        base_frame: egui::ColorImage,
        playhead: TimeCode,
        ctx: Option<&Context>,
    ) -> egui::ColorImage {
        let clip = match self.project.timeline.get_clip(clip_id) {
            Some(c) => c.clone(),
            None => return base_frame,
        };
        let track = match self.project.timeline.tracks.iter().find(|t| t.id == track_id) {
            Some(t) => t.clone(),
            None => return base_frame,
        };

        // 1. Beginning (In) Transition
        if let Some(tr_in) = clip.start_transition() {
            let elapsed_in = playhead.as_secs_f64() - clip.timeline_start.as_secs_f64();
            if elapsed_in >= 0.0 && elapsed_in < tr_in.duration_secs && tr_in.duration_secs > 0.0 {
                let progress = (elapsed_in / tr_in.duration_secs) as f32;
                let prev_clip = track
                    .clips
                    .iter()
                    .filter(|c| c.timeline_end() <= clip.timeline_start)
                    .max_by_key(|c| c.timeline_start);
                if let Some(prev) = prev_clip {
                    let prev_end_sec = prev.source_out.as_secs_f64();
                    if let Some(frame_a) = self.frame_cache.fetch_frame(&prev.source_path, prev_end_sec, ctx) {
                        return crate::media::blend_transition(&frame_a, &base_frame, tr_in.kind, progress);
                    }
                }
                return crate::media::blend_fade_in(&base_frame, tr_in.kind, progress);
            }
        }

        // 2. Ending (Out) Transition
        if let Some(tr_out) = clip.end_transition() {
            let remaining = clip.timeline_end().as_secs_f64() - playhead.as_secs_f64();
            if remaining >= 0.0 && remaining < tr_out.duration_secs && tr_out.duration_secs > 0.0 {
                let progress = (1.0 - (remaining / tr_out.duration_secs)).clamp(0.0, 1.0) as f32;
                let next_clip = track
                    .clips
                    .iter()
                    .filter(|c| c.timeline_start >= clip.timeline_end())
                    .min_by_key(|c| c.timeline_start);
                if let Some(next) = next_clip {
                    let next_start_sec = next.source_in.as_secs_f64();
                    if let Some(frame_b) = self.frame_cache.fetch_frame(&next.source_path, next_start_sec, ctx) {
                        return crate::media::blend_transition(&base_frame, &frame_b, tr_out.kind, progress);
                    }
                }
                return crate::media::blend_fade_in(&base_frame, tr_out.kind, 1.0 - progress);
            }
        }

        base_frame
    }

    pub(crate) fn base_frame_for(&mut self, clip: &Clip, ctx: Option<&Context>) -> Option<ColorImage> {
        let playhead = self.project.timeline.playhead;
        if clip.is_static_slide() {
            return match &clip.background {
                Some(crate::core::text_overlay::SlideBackground::Solid(col)) => {
                    Some(crate::media::generate_solid_color_frame(*col, 640, 360))
                }
                Some(crate::core::text_overlay::SlideBackground::Picture(p)) => {
                    self.frame_cache.fetch_frame(p, 0.0, ctx)
                }
                None => Some(crate::media::generate_solid_color_frame(egui::Color32::from_rgb(18, 18, 24), 640, 360)),
            };
        }
        if clip.has_video {
            let source_time = clip.timeline_to_source_time(playhead).unwrap_or(clip.source_in);
            return self.frame_cache.fetch_frame(&clip.source_path, source_time.as_secs_f64(), ctx);
        }
        match &clip.background {
            Some(crate::core::text_overlay::SlideBackground::Solid(col)) => {
                Some(crate::media::generate_solid_color_frame(*col, 640, 360))
            }
            Some(crate::core::text_overlay::SlideBackground::Picture(p)) => {
                self.frame_cache.fetch_frame(p, 0.0, ctx)
            }
            None => None,
        }
    }

    pub(crate) fn slide_visuals(&mut self, ctx: Option<&Context>) -> Vec<crate::ui::preview_player::SlideVisual> {
        use crate::core::text_overlay::SlideElement;
        use crate::ui::preview_player::{SlideVisual, SlideVisualKind};
        let mut visuals = Vec::new();
        let Some(active) = self.active_slide().cloned() else {
            return visuals;
        };
        let playhead = self.project.timeline.playhead;
        let slide_elapsed = (playhead - active.timeline_start).as_secs_f64().max(0.0);

        for (idx, el) in active.elements.into_iter().enumerate() {
            match el {
                SlideElement::Text(overlay) => {
                    visuals.push(SlideVisual {
                        idx,
                        kind: SlideVisualKind::Text,
                        bounds: (overlay.x, overlay.y, 0.0, 0.0),
                        label: None,
                        texture: None,
                        overlay: Some(overlay),
                        calendar: None,
                    });
                }
                SlideElement::Calendar(cal) => {
                    visuals.push(SlideVisual {
                        idx,
                        kind: SlideVisualKind::Calendar,
                        bounds: (cal.x, cal.y, cal.w, cal.h),
                        label: None,
                        texture: None,
                        overlay: None,
                        calendar: Some(cal),
                    });
                }
                SlideElement::Placeholder { slot_id: _, label, x, y, w, h } => {
                    visuals.push(SlideVisual {
                        idx,
                        kind: SlideVisualKind::Placeholder,
                        bounds: (x, y, w, h),
                        label: Some(label),
                        texture: None,
                        overlay: None,
                        calendar: None,
                    });
                }
                SlideElement::Picture { path, x, y, w, h } => {
                    let mut tex = self.slide_textures.get(&path).cloned();
                    if tex.is_none() && path.exists() {
                        if let Some(c) = ctx {
                            // Fast direct load for static image files
                            if let Ok(dyn_img) = image::open(&path) {
                                let rgba = dyn_img.to_rgba8();
                                let size = [rgba.width() as usize, rgba.height() as usize];
                                let pixels = rgba.into_raw();
                                let color_img = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                                let label = format!("slide_pic_{}", path.display());
                                let t = c.load_texture(label, color_img, egui::TextureOptions::LINEAR);
                                self.slide_textures.insert(path.clone(), t.clone());
                                tex = Some(t);
                            } else if let Some(img) = self.frame_cache.fetch_frame(&path, 0.0, Some(c)) {
                                let label = format!("slide_pic_{}", path.display());
                                let t = c.load_texture(label, img, egui::TextureOptions::LINEAR);
                                self.slide_textures.insert(path.clone(), t.clone());
                                tex = Some(t);
                            }
                        }
                    }
                    visuals.push(SlideVisual {
                        idx,
                        kind: SlideVisualKind::Picture,
                        bounds: (x, y, w, h),
                        label: None,
                        texture: tex,
                        overlay: None,
                        calendar: None,
                    });
                }
                SlideElement::Video { path, x, y, w, h } => {
                    let tex = if self.project.timeline.is_playing {
                        let is_first = !self.slide_video_players.contains_key(&path);
                        let player = self.slide_video_players.entry(path.clone()).or_insert_with(|| {
                            crate::media::stream_player::StreamVideoPlayer::new()
                        });

                        if is_first {
                            player.start(&path, slide_elapsed, None, ctx);
                        }

                        let (_has_new, frame_opt) = player.get_frame_for_time(slide_elapsed);
                        if let Some(img) = frame_opt {
                            ctx.map(|c| {
                                let t = c.load_texture(
                                    format!("stream_{}", path.display()),
                                    img,
                                    egui::TextureOptions::LINEAR,
                                );
                                self.slide_textures.insert(path.clone(), t.clone());
                                t
                            })
                        } else {
                            self.slide_textures.get(&path).cloned().or_else(|| {
                                self.frame_cache
                                    .fetch_frame(&path, slide_elapsed, ctx)
                                    .and_then(|img| {
                                        ctx.map(|c| {
                                            let t = c.load_texture(
                                                format!("fallback_{}", path.display()),
                                                img,
                                                egui::TextureOptions::LINEAR,
                                            );
                                            self.slide_textures.insert(path.clone(), t.clone());
                                            t
                                        })
                                    })
                            })
                        }
                    } else {
                        if let Some(player) = self.slide_video_players.get_mut(&path) {
                            player.stop();
                        }
                        self.slide_textures.get(&path).cloned().or_else(|| {
                            self.frame_cache
                                .fetch_frame(&path, slide_elapsed, ctx)
                                .and_then(|img| {
                                    ctx.map(|c| {
                                        let t = c.load_texture(
                                            format!("static_{}", path.display()),
                                            img,
                                            egui::TextureOptions::LINEAR,
                                        );
                                        self.slide_textures.insert(path.clone(), t.clone());
                                        t
                                    })
                                })
                        })
                    };

                    visuals.push(SlideVisual {
                        idx,
                        kind: SlideVisualKind::Video,
                        bounds: (x, y, w, h),
                        label: None,
                        texture: tex,
                        overlay: None,
                        calendar: None,
                    });
                }
                SlideElement::Audio { .. } => {}
            }
        }
        visuals
    }

    pub(crate) fn refresh_preview_frame(&mut self, ctx: Option<&Context>) {
        let playhead = self.project.timeline.playhead;

        if let Some(active) = self.active_slide().cloned() {
            let base = self.base_frame_for(&active, ctx);
            let frame = if let Some(base) = base {
                self.composite_transition(active.track_id, active.id, base, playhead, ctx)
            } else {
                crate::media::generate_solid_color_frame(egui::Color32::from_rgb(18, 18, 24), 640, 360)
            };
            self.current_frame = Some(frame);
            self.frame_version += 1;
        } else if let Some((clip_id, path, sec, _)) = self.get_active_video_clip_info(playhead) {
            let track_id = self
                .project
                .timeline
                .tracks
                .iter()
                .find(|t| t.kind == TrackKind::Video)
                .map(|t| t.id)
                .unwrap_or(0);
            if let Some(f) = self.frame_cache.fetch_frame(&path, sec, ctx) {
                let final_frame = self.composite_transition(track_id, clip_id, f, playhead, ctx);
                self.current_frame = Some(final_frame);
                self.frame_version += 1;
            }
        } else {
            self.current_frame = None;
            self.frame_version += 1;
        }

        self.last_frame_time = Some(playhead);
    }
}
