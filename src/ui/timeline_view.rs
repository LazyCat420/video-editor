use crate::core::timeline::Timeline;
use crate::core::time::TimeCode;
use crate::core::track::TrackKind;
use crate::media::peak_extractor::WaveformPeaks;
use crate::ui::node_graph_view::render_audio_envelope_graph;
use crate::ui::theme::AppTheme;
use egui::{
    Button, Color32, Frame, Id, Key, Pos2, Rect, RichText, Rounding, ScrollArea, Sense, Stroke, Ui,
    Vec2,
};
use std::collections::HashMap;

pub struct TimelineView;

pub enum TimelineAction {
    None,
    Seek(TimeCode),
    ClipSelected(u64),
    ClipMoved {
        clip_id: u64,
        target_track_id: u64,
        new_start: TimeCode,
    },
    ClipTrimmed {
        clip_id: u64,
        new_in: TimeCode,
        new_out: TimeCode,
        new_start: TimeCode,
    },
    SplitAtPlayhead,
    DeleteSelected,
}

impl TimelineView {
    pub const HEADER_WIDTH: f32 = 180.0;
    pub const RULER_HEIGHT: f32 = 28.0;
    pub const TRACK_HEIGHT: f32 = 70.0;
    pub const TRIM_HANDLE_WIDTH: f32 = 8.0;

    pub fn render(
        ui: &mut Ui,
        timeline: &mut Timeline,
        peak_cache: &HashMap<String, WaveformPeaks>,
    ) -> TimelineAction {
        let mut action = TimelineAction::None;

        let available_size = ui.available_size();
        let pps = timeline.zoom_pps;

        // Process global hotkeys
        if ui.input(|i| i.key_pressed(Key::S)) {
            action = TimelineAction::SplitAtPlayhead;
        }
        if ui.input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace)) {
            action = TimelineAction::DeleteSelected;
        }

        // Collect snapping candidate timestamps ahead of time to avoid borrow conflicts
        let mut snap_candidates = Vec::new();
        if timeline.snapping_enabled {
            snap_candidates.push(TimeCode::ZERO);
            snap_candidates.push(timeline.playhead);
            for track in &timeline.tracks {
                for clip in &track.clips {
                    snap_candidates.push(clip.timeline_start);
                    snap_candidates.push(clip.timeline_end());
                }
            }
        }
        let snapping_enabled = timeline.snapping_enabled;
        let snap_threshold_pixels = timeline.snap_threshold_pixels;

        let snap_fn = |target: TimeCode| -> TimeCode {
            if !snapping_enabled || pps <= 0.0 {
                return target;
            }
            let threshold_secs = (snap_threshold_pixels / pps) as f64;
            let target_secs = target.as_secs_f64();
            let mut closest = target;
            let mut min_diff = threshold_secs;

            for cand in &snap_candidates {
                let diff = (cand.as_secs_f64() - target_secs).abs();
                if diff < min_diff {
                    min_diff = diff;
                    closest = *cand;
                }
            }
            closest
        };

        ui.horizontal(|ui| {
            // ==========================================
            // 1. Left Fixed Column: Track Headers
            // ==========================================
            ui.vertical(|ui| {
                ui.set_width(Self::HEADER_WIDTH);

                // Top Ruler Corner
                let (corner_rect, _) = ui.allocate_exact_size(
                    Vec2::new(Self::HEADER_WIDTH, Self::RULER_HEIGHT),
                    Sense::hover(),
                );
                let painter = ui.painter_at(corner_rect);
                painter.rect_filled(corner_rect, Rounding::ZERO, AppTheme::BG_CARD);
                painter.text(
                    corner_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Tracks",
                    egui::FontId::proportional(12.0),
                    AppTheme::TEXT_MUTED,
                );

                // Track Controls Column
                for track in &mut timeline.tracks {
                    Frame::none()
                        .fill(if track.kind == TrackKind::Video {
                            AppTheme::TRACK_VIDEO_BG
                        } else {
                            AppTheme::TRACK_AUDIO_BG
                        })
                        .stroke(Stroke::new(
                            1.0,
                            if track.kind == TrackKind::Video {
                                AppTheme::TRACK_VIDEO_BORDER
                            } else {
                                AppTheme::TRACK_AUDIO_BORDER
                            },
                        ))
                        .inner_margin(6.0)
                        .show(ui, |ui| {
                            ui.set_height(Self::TRACK_HEIGHT - 12.0);
                            ui.set_width(Self::HEADER_WIDTH - 12.0);

                            ui.horizontal(|ui| {
                                let badge = if track.kind == TrackKind::Video { "🎬" } else { "🎵" };
                                ui.label(RichText::new(badge).size(13.0));
                                ui.label(
                                    RichText::new(&track.name)
                                        .strong()
                                        .size(12.0)
                                        .color(AppTheme::TEXT_PRIMARY),
                                );
                            });

                            ui.horizontal(|ui| {
                                // Mute Button
                                let mute_btn = Button::new(RichText::new("M").size(10.0))
                                    .fill(if track.is_muted { AppTheme::ACCENT_RED } else { AppTheme::BG_CARD });
                                if ui.add(mute_btn).clicked() {
                                    track.is_muted = !track.is_muted;
                                }

                                // Solo Button
                                let solo_btn = Button::new(RichText::new("S").size(10.0))
                                    .fill(if track.is_solo { AppTheme::ACCENT_YELLOW } else { AppTheme::BG_CARD });
                                if ui.add(solo_btn).clicked() {
                                    track.is_solo = !track.is_solo;
                                }

                                // Track Volume slider
                                ui.add(
                                    egui::Slider::new(&mut track.volume, 0.0..=2.0)
                                        .show_value(false)
                                        .text(""),
                                );
                            });
                        });
                    ui.add_space(2.0);
                }
            });

            // ==========================================
            // 2. Right Column: Scrollable Timeline Canvas
            // ==========================================
            let canvas_width = (available_size.x - Self::HEADER_WIDTH).max(400.0);
            let total_dur_secs = timeline.duration().as_secs_f64().max(30.0) + 10.0;
            let total_timeline_pixels = (total_dur_secs as f32 * pps).max(canvas_width);

            ScrollArea::horizontal()
                .id_salt("timeline_scroll_area")
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_width(total_timeline_pixels);

                        // ----------------------------------------------------
                        // 2A. Top Time Ruler Canvas
                        // ----------------------------------------------------
                        let (ruler_rect, ruler_response) = ui.allocate_exact_size(
                            Vec2::new(total_timeline_pixels, Self::RULER_HEIGHT),
                            Sense::click_and_drag(),
                        );
                        let ruler_painter = ui.painter_at(ruler_rect);
                        ruler_painter.rect_filled(
                            ruler_rect,
                            Rounding::ZERO,
                            AppTheme::BG_APP,
                        );

                        // Click or drag on ruler to seek
                        if ruler_response.clicked() || ruler_response.dragged() {
                            if let Some(pos) = ruler_response.interact_pointer_pos() {
                                let offset_px = (pos.x - ruler_rect.min.x).max(0.0);
                                let clicked_time = TimeCode::from_pixels(offset_px, pps);
                                let snapped = snap_fn(clicked_time);
                                action = TimelineAction::Seek(snapped);
                            }
                        }

                        // Draw Ruler Hash Marks and Timecode Labels
                        let sec_interval = if pps > 100.0 {
                            1.0
                        } else if pps > 40.0 {
                            2.0
                        } else if pps > 15.0 {
                            5.0
                        } else {
                            10.0
                        };

                        let max_time_sec = (total_timeline_pixels / pps) as f64;
                        let mut cur_sec = 0.0f64;
                        while cur_sec <= max_time_sec {
                            let x = ruler_rect.min.x + (cur_sec as f32 * pps);
                            if x <= ruler_rect.max.x {
                                ruler_painter.line_segment(
                                    [Pos2::new(x, ruler_rect.min.y + 16.0), Pos2::new(x, ruler_rect.max.y)],
                                    Stroke::new(1.0, AppTheme::TEXT_MUTED),
                                );

                                let tc_str = TimeCode::from_secs_f64(cur_sec).to_timecode_str();
                                ruler_painter.text(
                                    Pos2::new(x + 3.0, ruler_rect.min.y + 6.0),
                                    egui::Align2::LEFT_TOP,
                                    tc_str,
                                    egui::FontId::monospace(10.0),
                                    AppTheme::TEXT_SECONDARY,
                                );
                            }
                            cur_sec += sec_interval;
                        }

                        // ----------------------------------------------------
                        // 2B. Multi-Track Canvas & Clips
                        // ----------------------------------------------------
                        for track in &mut timeline.tracks {
                            let (track_rect, _track_response) = ui.allocate_exact_size(
                                Vec2::new(total_timeline_pixels, Self::TRACK_HEIGHT),
                                Sense::click_and_drag(),
                            );
                            let track_painter = ui.painter_at(track_rect);
                            track_painter.rect_filled(
                                track_rect,
                                Rounding::ZERO,
                                AppTheme::BG_PANEL,
                            );
                            track_painter.line_segment(
                                [
                                    Pos2::new(track_rect.min.x, track_rect.max.y),
                                    Pos2::new(track_rect.max.x, track_rect.max.y),
                                ],
                                Stroke::new(1.0, AppTheme::BG_HOVER),
                            );

                            // Render clips on this track
                            for clip in &mut track.clips {
                                let clip_dur = clip.duration();
                                let clip_start_x =
                                    track_rect.min.x + clip.timeline_start.to_pixels(pps);
                                let clip_width = clip_dur.to_pixels(pps).max(12.0);
                                let clip_rect = Rect::from_min_size(
                                    Pos2::new(clip_start_x, track_rect.min.y + 4.0),
                                    Vec2::new(clip_width, Self::TRACK_HEIGHT - 8.0),
                                );

                                let clip_id_egui = Id::new(format!("clip_{}", clip.id));
                                let clip_resp = ui.interact(clip_rect, clip_id_egui, Sense::click_and_drag());

                                if clip_resp.clicked() {
                                    action = TimelineAction::ClipSelected(clip.id);
                                }

                                // Handle Clip Dragging
                                if clip_resp.dragged() {
                                    let delta_x = clip_resp.drag_delta().x;
                                    let delta_time = TimeCode::from_pixels(delta_x, pps);
                                    let new_start = if delta_x > 0.0 {
                                        clip.timeline_start + delta_time
                                    } else {
                                        clip.timeline_start - delta_time
                                    };
                                    let snapped_start = snap_fn(new_start);
                                    action = TimelineAction::ClipMoved {
                                        clip_id: clip.id,
                                        target_track_id: track.id,
                                        new_start: snapped_start,
                                    };
                                }

                                // Draw Clip Background
                                let clip_painter = ui.painter_at(clip_rect);
                                let bg_color = if clip.is_selected {
                                    if track.kind == TrackKind::Video {
                                        AppTheme::CLIP_VIDEO_SELECTED
                                    } else {
                                        AppTheme::CLIP_AUDIO_SELECTED
                                    }
                                } else {
                                    if track.kind == TrackKind::Video {
                                        AppTheme::CLIP_VIDEO_BG
                                    } else {
                                        AppTheme::CLIP_AUDIO_BG
                                    }
                                };

                                clip_painter.rect_filled(
                                    clip_rect,
                                    Rounding::same(5.0),
                                    bg_color,
                                );
                                clip_painter.rect_stroke(
                                    clip_rect,
                                    Rounding::same(5.0),
                                    Stroke::new(
                                        if clip.is_selected { 2.0 } else { 1.0 },
                                        if clip.is_selected { Color32::WHITE } else { AppTheme::BG_HOVER },
                                    ),
                                );

                                // Clip Title Header
                                clip_painter.text(
                                    Pos2::new(clip_rect.min.x + 6.0, clip_rect.min.y + 4.0),
                                    egui::Align2::LEFT_TOP,
                                    &clip.name,
                                    egui::FontId::proportional(11.0),
                                    Color32::WHITE,
                                );

                                // ----------------------------------------------------
                                // Audio Track: Interactive Envelope Node Line Graph
                                // ----------------------------------------------------
                                if track.kind == TrackKind::Audio || clip.has_audio {
                                    let stem = clip
                                        .source_path
                                        .file_stem()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or_default();
                                    let peaks = peak_cache.get(stem);

                                    let envelope_rect = Rect::from_min_max(
                                        Pos2::new(clip_rect.min.x, clip_rect.min.y + 18.0),
                                        Pos2::new(clip_rect.max.x, clip_rect.max.y - 2.0),
                                    );

                                    render_audio_envelope_graph(
                                        ui,
                                        envelope_rect,
                                        &mut clip.volume_envelope,
                                        peaks,
                                        clip_dur,
                                        pps,
                                        clip.id,
                                    );
                                }
                            }
                            ui.add_space(2.0);
                        }

                        // ----------------------------------------------------
                        // 2C. Playhead Vertical Line & Scrubber Marker
                        // ----------------------------------------------------
                        let playhead_x = ruler_rect.min.x + timeline.playhead.to_pixels(pps);
                        let total_canvas_height = Self::RULER_HEIGHT + (timeline.tracks.len() as f32 * Self::TRACK_HEIGHT);

                        let playhead_top = Pos2::new(playhead_x, ruler_rect.min.y);
                        let playhead_bottom = Pos2::new(playhead_x, ruler_rect.min.y + total_canvas_height);

                        let playhead_painter = ui.painter();

                        // Glowing vertical line
                        playhead_painter.line_segment(
                            [playhead_top, playhead_bottom],
                            Stroke::new(2.0, AppTheme::PLAYHEAD_COLOR),
                        );

                        // Triangular Playhead Cap on Ruler
                        let cap_size = 7.0;
                        let cap_tri = vec![
                            Pos2::new(playhead_x - cap_size, ruler_rect.min.y),
                            Pos2::new(playhead_x + cap_size, ruler_rect.min.y),
                            Pos2::new(playhead_x, ruler_rect.min.y + 14.0),
                        ];
                        playhead_painter.add(egui::Shape::convex_polygon(
                            cap_tri,
                            AppTheme::PLAYHEAD_COLOR,
                            Stroke::new(1.0, Color32::WHITE),
                        ));
                    });
                });
        });

        action
    }
}
