use crate::core::timeline::Timeline;
use crate::core::time::TimeCode;
use crate::core::track::TrackKind;
use crate::core::{Transition, TransitionKind};
use crate::media::peak_extractor::WaveformPeaks;
use crate::ui::node_graph_view::render_audio_envelope_graph;
use crate::ui::theme::AppTheme;
use crate::ui::{MediaAssetDrag, small_slider, TrackReorderDrag};
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
    SplitClipAtTime {
        clip_id: u64,
        split_time: TimeCode,
    },
    DivideClipInHalf(u64),
    TrimStartToPlayhead(u64),
    TrimEndToPlayhead(u64),
    ApplyFadeIn(u64),
    ApplyFadeOut(u64),
    CopyClip(u64),
    PasteClip {
        track_id: u64,
        target_time: TimeCode,
    },
    DeleteClip(u64),
    DeleteSelected,
    DeleteTrack(u64),
    SetTransition {
        clip_id: u64,
        slot: crate::ui::transition_bin::TransitionSlot,
        transition: Option<Transition>,
    },
    ReorderTrack { from_id: u64, to_index: usize },
    AddMediaToTimeline { asset_id: u64, track_id: u64, start: TimeCode },
    Undo,
    Redo,
    CloseGaps(Option<u64>),
    AddVideoTrack,
    AddAudioTrack,
}

impl TimelineView {
    pub const HEADER_WIDTH: f32 = 210.0;
    pub const RULER_HEIGHT: f32 = 30.0;
    pub const TRACK_HEIGHT: f32 = 78.0;

    pub fn render(
        ui: &mut Ui,
        timeline: &mut Timeline,
        peak_cache: &HashMap<String, WaveformPeaks>,
        can_undo: bool,
        can_redo: bool,
        has_clipboard: bool,
    ) -> TimelineAction {
        let mut action = TimelineAction::None;

        let available_size = ui.available_size();
        let pps = timeline.zoom_pps;

        // Process global hotkeys
        if ui.input(|i| i.key_pressed(Key::S) && !i.modifiers.ctrl && !i.modifiers.command) {
            action = TimelineAction::SplitAtPlayhead;
        }
        if ui.input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace)) {
            action = TimelineAction::DeleteSelected;
        }

        // Collect snapping candidate timestamps (paired with optional clip ID)
        let mut snap_candidates = Vec::new();
        if timeline.snapping_enabled {
            snap_candidates.push((None, TimeCode::ZERO));
            snap_candidates.push((None, timeline.playhead));
            for track in &timeline.tracks {
                for clip in &track.clips {
                    snap_candidates.push((Some(clip.id), clip.timeline_start));
                    snap_candidates.push((Some(clip.id), clip.timeline_end()));
                }
            }
        }
        let snapping_enabled = timeline.snapping_enabled;
        let snap_threshold_pixels = timeline.snap_threshold_pixels;

        let snap_fn = |target: TimeCode, exclude_id: Option<u64>| -> TimeCode {
            if !snapping_enabled || pps <= 0.0 {
                return target;
            }
            let threshold_secs = (snap_threshold_pixels / pps) as f64;
            let target_secs = target.as_secs_f64();
            let mut closest = target;
            let mut min_diff = threshold_secs;

            for (clip_id_opt, cand) in &snap_candidates {
                if let Some(cand_clip_id) = clip_id_opt {
                    if Some(*cand_clip_id) == exclude_id {
                        continue;
                    }
                }
                let diff = (cand.as_secs_f64() - target_secs).abs();
                if diff < min_diff {
                    min_diff = diff;
                    closest = *cand;
                }
            }
            closest
        };

        ui.vertical(|ui| {
            // ====================================================
            // 0. Top Toolbar: Big Simple Buttons (Undo, Cut, Delete)
            // ====================================================
            ui.horizontal(|ui| {
                ui.add_space(4.0);

                if ui
                    .add_enabled(
                        can_undo,
                        Button::new(
                            RichText::new("↩ Undo")
                                .size(14.0)
                                .color(if can_undo { Color32::WHITE } else { AppTheme::text_muted() }),
                        ),
                    )
                    .clicked()
                {
                    action = TimelineAction::Undo;
                }

                if ui
                    .add_enabled(
                        can_redo,
                        Button::new(
                            RichText::new("↪ Redo")
                                .size(14.0)
                                .color(if can_redo { Color32::WHITE } else { AppTheme::text_muted() }),
                        ),
                    )
                    .clicked()
                {
                    action = TimelineAction::Redo;
                }

                ui.separator();

                // Zoom slider - moved to the left side of the toolbar (right after Undo/Redo)
                // so it is never cut off at the right edge, and compact so it fits anywhere.
                ui.label(
                    RichText::new("Zoom:").size(13.0).color(AppTheme::text_secondary()),
                );
                small_slider(ui, 12.0, |ui| {
                    ui.add_sized(
                        [90.0, 12.0],
                        egui::Slider::new(&mut timeline.zoom_pps, 5.0..=200.0)
                            .logarithmic(true)
                            .show_value(false),
                    );
                });

                // Playhead scrub: the slider that moves the orange timeline marker. Kept right
                // next to the Zoom slider (beside Redo) so it's easy to grab.
                ui.separator();
                ui.label(
                    RichText::new("Marker:").size(13.0).color(AppTheme::text_secondary()),
                )
                .on_hover_text("Move the orange marker through your video");
                let max_secs = timeline.duration().as_secs_f64().max(1.0);
                let mut marker_val = timeline.playhead.as_secs_f64();
                let marker_resp = small_slider(ui, 12.0, |ui| {
                    ui.add_sized(
                        [150.0, 12.0],
                        egui::Slider::new(&mut marker_val, 0.0..=max_secs).show_value(false),
                    )
                });
                if marker_resp.changed() || marker_resp.dragged() {
                    action = TimelineAction::Seek(TimeCode::from_secs_f64(marker_val));
                }

                ui.separator();

                if ui
                    .button(
                        RichText::new("✂ Cut Video Here (S)")
                            .size(14.0)
                            .color(AppTheme::accent_blue()),
                    )
                    .clicked()
                {
                    action = TimelineAction::SplitAtPlayhead;
                }

                if ui
                    .button(
                        RichText::new("🗑 Delete Clip (Del)")
                            .size(14.0)
                            .color(AppTheme::text_secondary()),
                    )
                    .clicked()
                {
                    action = TimelineAction::DeleteSelected;
                }
            });

            ui.add_space(3.0);

            ui.horizontal(|ui| {
                // Shared reorder geometry: one reference top for the row grid, so the header
                // column and the timeline body agree on where a dragged row would land (and
                // commits work when releasing from either side).
                let row_h = Self::TRACK_HEIGHT;
                let row_gap = 2.0;
                let track_count = timeline.tracks.len();
                let reorder_anchor_top = ui.cursor().top() + Self::RULER_HEIGHT + 8.0;
                let slot_from_y = |y: f32| -> usize {
                    (((y - reorder_anchor_top) / (row_h + row_gap)).floor() as isize)
                        .clamp(0, track_count as isize) as usize
                };

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
                    painter.rect_filled(corner_rect, Rounding::ZERO, AppTheme::bg_card());
                    painter.text(
                        corner_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Tracks",
                        egui::FontId::proportional(14.0),
                        AppTheme::text_primary(),
                    );

                    // Track Controls Column
                    for (track_index, track) in timeline.tracks.iter_mut().enumerate() {
                        let track_id = track.id;
                        let header_resp = Frame::none()
                            .fill(if track.kind == TrackKind::Video {
                                AppTheme::track_video_bg()
                            } else {
                                AppTheme::track_audio_bg()
                            })
                            .stroke(Stroke::new(
                                1.5,
                                if track.kind == TrackKind::Video {
                                    AppTheme::track_video_border()
                                } else {
                                    AppTheme::track_audio_border()
                                },
                            ))
                            .rounding(Rounding::same(4.0))
                            .inner_margin(6.0)
                            .show(ui, |ui| {
                                ui.set_height(Self::TRACK_HEIGHT - 12.0);
                                ui.set_width(Self::HEADER_WIDTH - 12.0);

                                // Row 1: Drag handle (grip + name) - grab to move this row up/down.
                                let handle = ui.allocate_response(
                                    egui::vec2(ui.available_width(), 20.0),
                                    egui::Sense::drag(),
                                );
                                handle.dnd_set_drag_payload(TrackReorderDrag(track_id));
                                if handle.dragged() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                                } else if handle.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                                }
                                ui.painter().text(
                                    handle.rect.left_center() + egui::vec2(2.0, 0.0),
                                    egui::Align2::LEFT_CENTER,
                                    format!("≡  {}", track.name),
                                    egui::FontId::proportional(12.5),
                                    AppTheme::text_primary(),
                                );

                                ui.add_space(2.0);

                                ui.horizontal(|ui| {
                                    let mute_text = if track.is_muted {
                                        RichText::new("Muted").size(12.0).color(Color32::from_rgb(255, 100, 100))
                                    } else {
                                        RichText::new("Mute").size(12.0).color(AppTheme::text_secondary())
                                    };
                                    if ui.button(mute_text).clicked() {
                                        track.is_muted = !track.is_muted;
                                    }

                                    let solo_text = if track.is_solo {
                                        RichText::new("Solo").size(12.0).color(AppTheme::accent_yellow())
                                    } else {
                                        RichText::new("Solo").size(12.0).color(AppTheme::text_muted())
                                    };
                                    if ui.button(solo_text).clicked() {
                                        track.is_solo = !track.is_solo;
                                    }

                                    // Delete this whole row
                                    if ui
                                        .add(
                                            Button::new(RichText::new("🗑").size(13.0))
                                                .min_size(egui::vec2(24.0, 20.0)),
                                        )
                                        .on_hover_text("Remove this whole row")
                                        .clicked()
                                    {
                                        action = TimelineAction::DeleteTrack(track_id);
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Vol").size(11.0).color(AppTheme::text_muted()));
                                    small_slider(ui, 12.0, |ui| {
                                        ui.add_sized(
                                            [ui.available_width(), 12.0],
                                            egui::Slider::new(&mut track.volume, 0.0..=2.0)
                                                .show_value(false),
                                        );
                                    });
                                });
                            })
                            .response;

                        // Reorder drop target: atop this header, show a highlight border.
                        if let Some(payload) = header_resp.dnd_hover_payload::<TrackReorderDrag>() {
                            if payload.0 != track_id {
                                let hp = ui.painter();
                                hp.rect_stroke(
                                    header_resp.rect.expand(2.0),
                                    Rounding::same(6.0),
                                    Stroke::new(2.5, AppTheme::accent_blue()),
                                );
                            }
                        }
                        if let Some(payload) = header_resp.dnd_release_payload::<TrackReorderDrag>() {
                            if payload.0 != track_id {
                                let slot = ui
                                    .input(|i| i.pointer.hover_pos())
                                    .map(|p| slot_from_y(p.y))
                                    .unwrap_or(track_index);
                                action = TimelineAction::ReorderTrack {
                                    from_id: payload.0,
                                    to_index: slot,
                                };
                            }
                        }

                        ui.add_space(2.0);
                    }

                    // Add Track Buttons
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                RichText::new("+ Video Track")
                                    .size(12.0)
                                    .color(AppTheme::accent_blue()),
                            )
                            .clicked()
                        {
                            action = TimelineAction::AddVideoTrack;
                        }
                        if ui
                            .button(
                                RichText::new("+ Music Track")
                                    .size(12.0)
                                    .color(AppTheme::accent_green()),
                            )
                            .clicked()
                        {
                            action = TimelineAction::AddAudioTrack;
                        }
                    });
                });

                // ==========================================
                // 2. Right Horizontally Scrollable Timeline Area
                // ==========================================
                let track_area_width = available_size.x - Self::HEADER_WIDTH - 20.0;
                let total_timeline_pixels = (timeline.duration().to_pixels(pps) + 400.0)
                    .max(track_area_width);

                ScrollArea::horizontal()
                    .id_salt("timeline_scroll")
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
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
                                AppTheme::bg_app(),
                            );

                            // Click or drag on ruler to seek
                            if ruler_response.clicked() || ruler_response.dragged() {
                                if let Some(pos) = ruler_response.interact_pointer_pos() {
                                    let offset_px = (pos.x - ruler_rect.min.x).max(0.0);
                                    let clicked_time = TimeCode::from_pixels(offset_px, pps);
                                    let snapped = snap_fn(clicked_time, None);
                                    action = TimelineAction::Seek(snapped);
                                }
                            }

                            // Draw Ruler Hash Marks and Friendly Time Labels
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
                                        Stroke::new(1.0, AppTheme::text_muted()),
                                    );

                                    let cur_m = (cur_sec / 60.0).floor() as u64;
                                    let cur_s = (cur_sec % 60.0).floor() as u64;
                                    let label_str = format!("{:02}:{:02}", cur_m, cur_s);

                                    ruler_painter.text(
                                        Pos2::new(x + 3.0, ruler_rect.min.y + 6.0),
                                        egui::Align2::LEFT_TOP,
                                        label_str,
                                        egui::FontId::proportional(12.0),
                                        AppTheme::text_secondary(),
                                    );
                                }
                                cur_sec += sec_interval;
                            }

                            // ----------------------------------------------------
                            // 2B. Multi-Track Canvas & Clips (floating reorder)
                            // ----------------------------------------------------

                            let row_gap_used = row_gap;
                            let row_h_used = row_h;
                            let track_count_used = track_count;
                            let total_tracks_h = (track_count_used as f32) * (row_h_used + row_gap_used);

                            // Reserve the full vertical space once, so the playhead & scroll
                            // geometry stay correct while individual rows animate.
                            let (_bg_block, _bg_resp) = ui.allocate_exact_size(
                                Vec2::new(total_timeline_pixels, total_tracks_h),
                                Sense::hover(),
                            );
                            let area_left = _bg_block.min.x;
                            // Rows are laid out from the shared reference top so the header and
                            // body agree on the row grid (needed for some reorder math).
                            let area_top = reorder_anchor_top;
                            let content_bottom = _bg_block.max.y;

                            // While a track header is being dragged, float that row as a ghost
                            // and slide the other rows apart so the user sees where it will land.
                            let is_reordering = egui::DragAndDrop::has_payload_of_type::<TrackReorderDrag>(ui.ctx());
                            let drag_id: Option<u64> =
                                egui::DragAndDrop::payload::<TrackReorderDrag>(ui.ctx())
                                    .map(|p| p.0);

                            // Drop slot from the pointer (shared mapping, allows the row to go
                            // at the very end of the list too).
                            let slot_index = if is_reordering {
                                // Keep animating while dragging so the rows visibly nudge apart.
                                ui.ctx().request_repaint();
                                ui.input(|i| i.pointer.hover_pos())
                                    .map(|p| slot_from_y(p.y))
                                    .unwrap_or(0)
                            } else {
                                0
                            };

                            // Rows that are NOT the dragged row reflow to make room around the slot.
                            let others: Vec<u64> = if let Some(d) = drag_id {
                                timeline
                                    .tracks
                                    .iter()
                                    .map(|t| t.id)
                                    .filter(|&id| id != d)
                                    .collect()
                            } else {
                                Vec::new()
                            };

                            for (track_index, track) in timeline.tracks.iter_mut().enumerate() {
                                let is_drag = drag_id == Some(track.id);
                                if is_drag && is_reordering {
                                    continue; // this row is drawn as the floating ghost below
                                }

                                // Reserve one slot for the ghost when reordering.
                                let slot = if is_reordering {
                                    let n_others = others
                                        .iter()
                                        .position(|&id| id == track.id)
                                        .unwrap_or(track_index);
                                    if n_others >= slot_index {
                                        n_others + 1
                                    } else {
                                        n_others
                                    }
                                } else {
                                    track_index
                                };
                                let target_top = area_top + slot as f32 * (row_h + row_gap);
                                let anim_top = ui
                                    .ctx()
                                    .animate_value_with_time(
                                        Id::new(("track_row_anim", track.id)),
                                        target_top,
                                        0.14,
                                    );

                                let track_rect = Rect::from_min_size(
                                    Pos2::new(area_left, anim_top),
                                    Vec2::new(total_timeline_pixels, row_h),
                                );
                                let track_response = ui.interact(
                                    track_rect,
                                    Id::new(("track_body", track.id)),
                                    Sense::click_and_drag(),
                                );
                                let track_painter = ui.painter_at(track_rect);
                                track_painter.rect_filled(
                                    track_rect,
                                    Rounding::ZERO,
                                    AppTheme::bg_panel(),
                                );
                                track_painter.line_segment(
                                    [
                                        Pos2::new(track_rect.min.x, track_rect.max.y),
                                        Pos2::new(track_rect.max.x, track_rect.max.y),
                                    ],
                                    Stroke::new(1.0, AppTheme::bg_hover()),
                                );

                                // Right-click context menu on Track Background
                                track_response.context_menu(|ui| {
                                    ui.set_min_width(180.0);
                                    ui.label(
                                        RichText::new(&track.name)
                                            .strong()
                                            .size(13.0)
                                            .color(AppTheme::text_secondary()),
                                    );
                                    ui.separator();

                                    if ui
                                        .add_enabled(
                                            has_clipboard,
                                            Button::new(
                                                RichText::new("📋 Paste Clip")
                                                    .size(14.0)
                                                    .color(if has_clipboard { Color32::WHITE } else { AppTheme::text_muted() }),
                                            ),
                                        )
                                        .clicked()
                                    {
                                        let mouse_pos = ui.input(|i| {
                                            i.pointer.hover_pos().unwrap_or(track_rect.left_top())
                                        });
                                        let offset_px = (mouse_pos.x - track_rect.min.x).max(0.0);
                                        let target_time = TimeCode::from_pixels(offset_px, pps);
                                        action = TimelineAction::PasteClip {
                                            track_id: track.id,
                                            target_time,
                                        };
                                        ui.close_menu();
                                    }
                                });

                                // Drag a file from the files panel and drop it onto this track.
                                let content_origin_x = ui.min_rect().min.x;
                                if let Some(_payload) = track_response.dnd_hover_payload::<MediaAssetDrag>() {
                                    let hover_painter = ui.painter();
                                    hover_painter.rect_stroke(
                                        track_rect,
                                        Rounding::same(6.0),
                                        Stroke::new(2.5, AppTheme::accent_blue()),
                                    );
                                    hover_painter.text(
                                        track_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "➕ Put file here",
                                        egui::FontId::proportional(15.0),
                                        AppTheme::accent_blue(),
                                    );
                                }
                                if let Some(drop) = track_response.dnd_release_payload::<MediaAssetDrag>() {
                                    let asset_id = drop.0;
                                    let mut start = TimeCode::ZERO;
                                    if let Some(screen_pos) = ui.input(|i| i.pointer.hover_pos()) {
                                        let offset_px = (screen_pos.x - content_origin_x - track_rect.min.x).max(0.0);
                                        start = TimeCode::from_pixels(offset_px, pps);
                                    }
                                    action = TimelineAction::AddMediaToTimeline {
                                        asset_id,
                                        track_id: track.id,
                                        start,
                                    };
                                }

                                // Commit reorder if the user lets go over this row.
                                if let Some(released) =
                                    track_response.dnd_release_payload::<TrackReorderDrag>()
                                {
                                    if released.0 != track.id {
                                        action = TimelineAction::ReorderTrack {
                                            from_id: released.0,
                                            to_index: slot_index,
                                        };
                                    }
                                }

                                // Render clips on this track
                                for clip in &mut track.clips {
                                    let clip_dur = clip.duration();
                                    let clip_start_x =
                                        track_rect.min.x + clip.timeline_start.to_pixels(pps);
                                    let clip_width = clip_dur.to_pixels(pps).max(16.0);
                                    let clip_rect = Rect::from_min_size(
                                        Pos2::new(clip_start_x, track_rect.min.y + 4.0),
                                        Vec2::new(clip_width, Self::TRACK_HEIGHT - 8.0),
                                    );

                                    let clip_id_egui = Id::new(format!("clip_{}", clip.id));
                                    let clip_resp = ui.interact(
                                        clip_rect,
                                        clip_id_egui,
                                        Sense::click_and_drag(),
                                    );

                                    if clip_resp.clicked() {
                                        action = TimelineAction::ClipSelected(clip.id);
                                    }

                                    // Hand cursor feedback when hovering or dragging clips
                                    if clip_resp.dragged() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                                    } else if clip_resp.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                                    }

                                    // Handle Clip Dragging with signed float delta (excludes self from snapping)
                                    if clip_resp.dragged() {
                                        let delta_x = clip_resp.drag_delta().x;
                                        let cur_secs = clip.timeline_start.as_secs_f64();
                                        let delta_secs = (delta_x / pps) as f64;
                                        let new_secs = (cur_secs + delta_secs).max(0.0);
                                        let new_start = TimeCode::from_secs_f64(new_secs);
                                        let snapped_start = snap_fn(new_start, Some(clip.id));
                                        action = TimelineAction::ClipMoved {
                                            clip_id: clip.id,
                                            target_track_id: track.id,
                                            new_start: snapped_start,
                                        };
                                    }

                                    // Dead-Simple 5-Item Senior Context Menu on Clip
                                    clip_resp.context_menu(|ui| {
                                        ui.set_min_width(220.0);
                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new(&clip.name)
                                                .strong()
                                                .size(14.0)
                                                .color(AppTheme::accent_cyan()),
                                        );
                                        ui.separator();

                                        if ui
                                            .button(
                                                RichText::new("✂ Cut Video Here")
                                                    .size(15.0)
                                                    .color(AppTheme::accent_blue()),
                                            )
                                            .clicked()
                                        {
                                            action = TimelineAction::SplitClipAtTime {
                                                clip_id: clip.id,
                                                split_time: timeline.playhead,
                                            };
                                            ui.close_menu();
                                        }

                                        if ui
                                            .button(
                                                RichText::new("➗ Divide in Half")
                                                    .size(15.0)
                                                    .color(Color32::WHITE),
                                            )
                                            .clicked()
                                        {
                                            action = TimelineAction::DivideClipInHalf(clip.id);
                                            ui.close_menu();
                                        }

                                        ui.separator();

                                        if ui
                                            .button(
                                                RichText::new("📋 Copy Clip")
                                                    .size(15.0)
                                                    .color(Color32::WHITE),
                                            )
                                            .clicked()
                                        {
                                            action = TimelineAction::CopyClip(clip.id);
                                            ui.close_menu();
                                        }

                                        if ui
                                            .add_enabled(
                                                has_clipboard,
                                                Button::new(
                                                    RichText::new("📋 Paste Clip")
                                                        .size(15.0)
                                                        .color(if has_clipboard {
                                                            Color32::WHITE
                                                        } else {
                                                            AppTheme::text_muted()
                                                        }),
                                                ),
                                            )
                                            .clicked()
                                        {
                                            action = TimelineAction::PasteClip {
                                                track_id: track.id,
                                                target_time: clip.timeline_end(),
                                            };
                                            ui.close_menu();
                                        }

                                        ui.menu_button(
                                            RichText::new("✨ Beginning Transition (In)...").size(14.0),
                                            |ui| {
                                                ui.set_min_width(200.0);
                                                for kind in TransitionKind::all() {
                                                    if ui
                                                        .button(
                                                            RichText::new(kind.label()).size(14.0),
                                                        )
                                                        .clicked()
                                                    {
                                                        let dur = clip
                                                            .start_transition()
                                                            .map(|t| t.duration_secs)
                                                            .unwrap_or(0.5);
                                                        action = TimelineAction::SetTransition {
                                                            clip_id: clip.id,
                                                            slot: crate::ui::transition_bin::TransitionSlot::In,
                                                            transition: Some(Transition {
                                                                kind: *kind,
                                                                duration_secs: dur,
                                                            }),
                                                        };
                                                        ui.close_menu();
                                                    }
                                                }
                                                if clip.start_transition().is_some() {
                                                    ui.separator();
                                                    if ui
                                                        .button(
                                                            RichText::new(
                                                                "❌ Remove Beginning Transition",
                                                            )
                                                            .size(14.0)
                                                            .color(Color32::from_rgb(255, 130, 130)),
                                                        )
                                                        .clicked()
                                                    {
                                                        action = TimelineAction::SetTransition {
                                                            clip_id: clip.id,
                                                            slot: crate::ui::transition_bin::TransitionSlot::In,
                                                            transition: None,
                                                        };
                                                        ui.close_menu();
                                                    }
                                                }
                                            },
                                        );

                                        ui.menu_button(
                                            RichText::new("✨ End Transition (Out)...").size(14.0),
                                            |ui| {
                                                ui.set_min_width(200.0);
                                                for kind in TransitionKind::all() {
                                                    if ui
                                                        .button(
                                                            RichText::new(kind.label()).size(14.0),
                                                        )
                                                        .clicked()
                                                    {
                                                        let dur = clip
                                                            .end_transition()
                                                            .map(|t| t.duration_secs)
                                                            .unwrap_or(0.5);
                                                        action = TimelineAction::SetTransition {
                                                            clip_id: clip.id,
                                                            slot: crate::ui::transition_bin::TransitionSlot::Out,
                                                            transition: Some(Transition {
                                                                kind: *kind,
                                                                duration_secs: dur,
                                                            }),
                                                        };
                                                        ui.close_menu();
                                                    }
                                                }
                                                if clip.end_transition().is_some() {
                                                    ui.separator();
                                                    if ui
                                                        .button(
                                                            RichText::new(
                                                                "❌ Remove End Transition",
                                                            )
                                                            .size(14.0)
                                                            .color(Color32::from_rgb(255, 130, 130)),
                                                        )
                                                        .clicked()
                                                    {
                                                        action = TimelineAction::SetTransition {
                                                            clip_id: clip.id,
                                                            slot: crate::ui::transition_bin::TransitionSlot::Out,
                                                            transition: None,
                                                        };
                                                        ui.close_menu();
                                                    }
                                                }
                                            },
                                        );

                                        ui.separator();

                                        if ui
                                            .button(
                                                RichText::new("🗑 Delete Clip")
                                                    .size(15.0)
                                                    .color(Color32::from_rgb(255, 110, 110)),
                                            )
                                            .clicked()
                                        {
                                            action = TimelineAction::DeleteClip(clip.id);
                                            ui.close_menu();
                                        }
                                    });

                                    // Draw Clip Background
                                    let clip_painter = ui.painter_at(clip_rect);
                                    let bg_color = if clip.is_selected {
                                        if clip.is_title_card {
                                            Color32::from_rgb(85, 45, 95)
                                        } else if track.kind == TrackKind::Video {
                                            AppTheme::clip_video_selected()
                                        } else {
                                            AppTheme::clip_audio_selected()
                                        }
                                    } else {
                                        if clip.is_title_card {
                                            Color32::from_rgb(50, 25, 60)
                                        } else if track.kind == TrackKind::Video {
                                            AppTheme::clip_video_bg()
                                        } else {
                                            AppTheme::clip_audio_bg()
                                        }
                                    };

                                    clip_painter.rect_filled(
                                        clip_rect,
                                        Rounding::same(6.0),
                                        bg_color,
                                    );
                                    clip_painter.rect_stroke(
                                        clip_rect,
                                        Rounding::same(6.0),
                                        Stroke::new(
                                            if clip.is_selected { 2.5 } else { 1.0 },
                                            if clip.is_selected {
                                                Color32::WHITE
                                            } else {
                                                AppTheme::bg_hover()
                                            },
                                        ),
                                    );

                                    // Clip Title Header (offset to avoid overlapping with in-transition badge)
                                    let title_offset_x = if clip.start_transition().is_some() && clip_width > 120.0 {
                                        80.0
                                    } else {
                                        8.0
                                    };
                                    let display_title = if clip.is_title_card {
                                        format!("🎬 {}", clip.name)
                                    } else {
                                        clip.name.clone()
                                    };
                                    clip_painter.text(
                                        Pos2::new(clip_rect.min.x + title_offset_x, clip_rect.min.y + 6.0),
                                        egui::Align2::LEFT_TOP,
                                        display_title,
                                        egui::FontId::proportional(13.0),
                                        Color32::WHITE,
                                    );

                                    // Caption Indicator Badge
                                    if clip.text_overlay.is_some() && !clip.is_title_card && clip_width > 130.0 {
                                        clip_painter.text(
                                            Pos2::new(clip_rect.center().x, clip_rect.min.y + 6.0),
                                            egui::Align2::CENTER_TOP,
                                            "💬 Caption",
                                            egui::FontId::proportional(11.0),
                                            Color32::from_rgb(255, 215, 100),
                                        );
                                    }

                                    // 1. Beginning Transition Badge (Anchored to the LEFT edge)
                                    if let Some(tr) = clip.start_transition() {
                                        if clip_width > 60.0 {
                                            let badge_text = format!("⇤ {} {:.1}s", tr.kind.label(), tr.duration_secs);
                                            let font_id = egui::FontId::proportional(11.0);
                                            let text_len = (badge_text.len() as f32 * 6.2).min((clip_width / 2.0) - 8.0);
                                            let pill_rect = Rect::from_min_max(
                                                Pos2::new(clip_rect.min.x + 4.0, clip_rect.min.y + 4.0),
                                                Pos2::new(clip_rect.min.x + 12.0 + text_len, clip_rect.min.y + 20.0),
                                            );
                                            clip_painter.rect_filled(
                                                pill_rect,
                                                Rounding::same(4.0),
                                                Color32::from_rgb(18, 38, 55),
                                            );
                                            clip_painter.rect_stroke(
                                                pill_rect,
                                                Rounding::same(4.0),
                                                Stroke::new(1.0, Color32::from_rgb(0, 210, 235)),
                                            );
                                            clip_painter.text(
                                                Pos2::new(clip_rect.min.x + 8.0, clip_rect.min.y + 5.5),
                                                egui::Align2::LEFT_TOP,
                                                badge_text,
                                                font_id,
                                                Color32::from_rgb(0, 225, 255),
                                            );
                                        }
                                    }

                                    // 2. End Transition Badge (Anchored to the RIGHT edge)
                                    if let Some(tr) = clip.end_transition() {
                                        if clip_width > 60.0 {
                                            let badge_text = format!("{} {:.1}s ⇥", tr.kind.label(), tr.duration_secs);
                                            let font_id = egui::FontId::proportional(11.0);
                                            let text_len = (badge_text.len() as f32 * 6.2).min((clip_width / 2.0) - 8.0);
                                            let pill_rect = Rect::from_min_max(
                                                Pos2::new(clip_rect.max.x - 12.0 - text_len, clip_rect.min.y + 4.0),
                                                Pos2::new(clip_rect.max.x - 4.0, clip_rect.min.y + 20.0),
                                            );
                                            clip_painter.rect_filled(
                                                pill_rect,
                                                Rounding::same(4.0),
                                                Color32::from_rgb(45, 38, 20),
                                            );
                                            clip_painter.rect_stroke(
                                                pill_rect,
                                                Rounding::same(4.0),
                                                Stroke::new(1.0, AppTheme::accent_yellow()),
                                            );
                                            clip_painter.text(
                                                Pos2::new(clip_rect.max.x - 8.0, clip_rect.min.y + 5.5),
                                                egui::Align2::RIGHT_TOP,
                                                badge_text,
                                                font_id,
                                                AppTheme::accent_yellow(),
                                            );
                                        }
                                    }

                                    // Audio Track Envelope Graph (only for dedicated audio tracks or selected clips for 60 FPS speed)
                                    if track.kind == TrackKind::Audio || (clip.is_selected && clip.has_audio) {
                                        let stem = clip
                                            .source_path
                                            .file_stem()
                                            .and_then(|s| s.to_str())
                                            .unwrap_or_default();
                                        let peaks = peak_cache.get(stem);

                                        let envelope_rect = Rect::from_min_max(
                                            Pos2::new(clip_rect.min.x, clip_rect.min.y + 22.0),
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
                            // 2B2. Floating ghost of the dragged track + drop marker
                            // ----------------------------------------------------
                            if is_reordering {
                                if let Some(d) = drag_id {
                                    if let Some(track) = timeline.tracks.iter().find(|t| t.id == d) {
                                        let ghost_top =
                                            area_top + slot_index as f32 * (row_h + row_gap);
                                        let ghost_rect = Rect::from_min_size(
                                            Pos2::new(area_left, ghost_top),
                                            Vec2::new(total_timeline_pixels, row_h),
                                        );
                                        let gp = ui.painter();

                                        // Floating translucent card with a bright outline.
                                        let fill = if track.kind == TrackKind::Video {
                                            AppTheme::clip_video_bg()
                                        } else {
                                            AppTheme::clip_audio_bg()
                                        }
                                        .gamma_multiply(0.85);
                                        gp.rect_filled(
                                            ghost_rect,
                                            Rounding::same(8.0),
                                            fill,
                                        );
                                        gp.rect_stroke(
                                            ghost_rect,
                                            Rounding::same(8.0),
                                            Stroke::new(2.5, AppTheme::accent_blue()),
                                        );
                                        gp.text(
                                            ghost_rect.left_top() + egui::vec2(8.0, 6.0),
                                            egui::Align2::LEFT_TOP,
                                            &track.name,
                                            egui::FontId::proportional(13.0),
                                            Color32::WHITE,
                                        );

                                        // Ghost clips so the row reads as a moving video strip.
                                        for clip in &track.clips {
                                            let cw = clip.duration().to_pixels(pps).max(16.0);
                                            let crect = Rect::from_min_size(
                                                Pos2::new(
                                                    ghost_rect.min.x
                                                        + clip.timeline_start.to_pixels(pps),
                                                    ghost_rect.min.y + 4.0,
                                                ),
                                                Vec2::new(cw, row_h - 8.0),
                                            );
                                            gp.rect_filled(
                                                crect,
                                                Rounding::same(6.0),
                                                Color32::WHITE.gamma_multiply(0.20),
                                            );
                                        }
                                    }
                                }

                                // Blue drop line showing exactly where the row will land.
                                let ly = area_top + slot_index as f32 * (row_h + row_gap);
                                ui.painter().line_segment(
                                    [
                                        Pos2::new(area_left, ly),
                                        Pos2::new(area_left + total_timeline_pixels, ly),
                                    ],
                                    Stroke::new(2.0, AppTheme::accent_blue()),
                                );
                            }

                            // ----------------------------------------------------
                            // 2C. Playhead Vertical Line & Scrubber Marker
                            // ----------------------------------------------------
                            let playhead_x =
                                ruler_rect.min.x + timeline.playhead.to_pixels(pps);

                            let playhead_top = Pos2::new(playhead_x, ruler_rect.min.y);
                            let playhead_bottom = Pos2::new(playhead_x, content_bottom);

                            let playhead_painter = ui.painter();

                            // Glowing vertical line
                            playhead_painter.line_segment(
                                [playhead_top, playhead_bottom],
                                Stroke::new(2.5, AppTheme::playhead_color()),
                            );

                            // Triangular Playhead Cap on Ruler
                            let cap_size = 9.0;
                            let cap_tri = vec![
                                Pos2::new(playhead_x - cap_size, ruler_rect.min.y),
                                Pos2::new(playhead_x + cap_size, ruler_rect.min.y),
                                Pos2::new(playhead_x, ruler_rect.min.y + 16.0),
                            ];
                            playhead_painter.add(egui::Shape::convex_polygon(
                                cap_tri,
                                AppTheme::playhead_color(),
                                Stroke::new(1.2, Color32::WHITE),
                            ));
                        });
                    });
            });

            // 3. Simple Tips at Bottom
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Tips:")
                        .strong()
                        .size(13.0)
                        .color(AppTheme::accent_yellow()),
                );
                ui.label(
                    RichText::new("Right-click any clip to Cut, Copy, or Delete.")
                        .size(13.0)
                        .color(AppTheme::text_secondary()),
                );
            });
        });

        action
    }
}
