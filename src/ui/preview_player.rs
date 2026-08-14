use crate::core::text_overlay::{TextAlignment, TextBoxStyle, TextOverlay};
use crate::core::time::TimeCode;
use crate::core::timeline::Timeline;
use crate::ui::theme::AppTheme;
use egui::{
    Align2, Button, Color32, ColorImage, FontFamily, FontId, Id, Pos2, Rect, RichText, Rounding,
    Sense, TextureHandle, TextureOptions, Ui, Vec2,
};
use std::sync::Arc;

/// One resolved element handed to the preview for drawing + hit-testing. `texture` carries
/// the frame for Picture/Video elements; `overlay` carries a Text element's styling. `idx`
/// is the element's index in the clip's `elements` list, used to mutate it via actions.
pub struct SlideVisual {
    pub idx: usize,
    /// Normalized (x, y, w, h). Text reports its anchor with w=h=0.
    pub bounds: (f32, f32, f32, f32),
    pub texture: Option<TextureHandle>,
    pub overlay: Option<TextOverlay>,
    pub kind: SlideVisualKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SlideVisualKind {
    Text,
    Picture,
    Video,
}

pub struct PreviewPlayerView;

pub enum PlayerAction {
    None,
    PlayPauseToggle,
    StepFrames(i64),
    StepSeconds(f64),
    Seek(TimeCode),
    Stop,
    /// Place a pending element at a normalized point on the frame (0..1).
    PlaceAt { x: f32, y: f32 },
    /// Drag an element: boxes report the new top-left, text the new center anchor.
    MoveElement { idx: usize, x: f32, y: f32 },
    /// Resize a box element (x, y, w, h all normalized, x/y top-left).
    ResizeElement { idx: usize, x: f32, y: f32, w: f32, h: f32 },
    /// Fill the whole frame with element `idx`.
    FullSlide { idx: usize },
}

#[derive(Clone, Copy)]
enum DragMode {
    Move,
    Resize,
}

fn drag_id() -> Id {
    Id::new("preview_slide_drag")
}

fn visual_screen_rect(frame: Rect, b: (f32, f32, f32, f32)) -> Rect {
    let (x, y, w, h) = b;
    let tl = frame.min + Vec2::new(x * frame.width(), y * frame.height());
    if w <= 0.0 && h <= 0.0 {
        Rect::from_center_size(tl, Vec2::new(140.0, 48.0))
    } else {
        Rect::from_min_size(tl, Vec2::new(w * frame.width(), h * frame.height()))
    }
}

impl PreviewPlayerView {
    pub fn render(
        ui: &mut Ui,
        timeline: &Timeline,
        current_frame: Option<&ColorImage>,
        visuals: &[SlideVisual],
        texture_cache: &mut Option<TextureHandle>,
        frame_is_dirty: bool,
        place_mode: bool,
    ) -> PlayerAction {
        let mut action = PlayerAction::None;

        ui.vertical(|ui| {
            let available_size = ui.available_size();
            let canvas_height = (available_size.y - 85.0).max(180.0);
            let canvas_width = available_size.x;

            // 16:9 aspect ratio bounding box
            let target_aspect = 16.0 / 9.0;
            let (view_w, view_h) = if canvas_width / canvas_height > target_aspect {
                (canvas_height * target_aspect, canvas_height)
            } else {
                (canvas_width, canvas_width / target_aspect)
            };

            let (rect, response) =
                ui.allocate_exact_size(Vec2::new(view_w, view_h), Sense::click_and_drag());

            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, Rounding::same(8.0), Color32::BLACK);
            painter.rect_stroke(
                rect,
                Rounding::same(8.0),
                egui::Stroke::new(1.5, AppTheme::bg_hover()),
            );

            let total_dur = timeline.duration();

            if let Some(frame) = current_frame {
                let texture = texture_cache.get_or_insert_with(|| {
                    ui.ctx()
                        .load_texture("video_preview", frame.clone(), TextureOptions::LINEAR)
                });
                if frame_is_dirty {
                    texture.set(frame.clone(), TextureOptions::LINEAR);
                }
                painter.image(
                    texture.id(),
                    rect,
                    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );

                // Draw each visual element (pictures/videos as quads, text via painter).
                let screen_rects: Vec<Rect> = visuals
                    .iter()
                    .map(|v| visual_screen_rect(rect, v.bounds))
                    .collect();
                for (v, srect) in visuals.iter().zip(screen_rects.iter()) {
                    if let Some(tex) = &v.texture {
                        painter.image(
                            tex.id(),
                            *srect,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                        painter.rect_stroke(
                            *srect,
                            Rounding::same(2.0),
                            egui::Stroke::new(1.0, Color32::from_white_alpha(120)),
                        );
                    }
                    if let Some(overlay) = &v.overlay {
                        Self::draw_text_overlay(&painter, rect, overlay);
                    }
                }

                // Interactions: place / move / resize / full-slide.
                Self::handle_interactions(
                    ui,
                    response,
                    rect,
                    visuals,
                    &screen_rects,
                    place_mode,
                    &mut action,
                );
            } else if total_dur.as_secs_f64() > 0.0 && timeline.playhead >= total_dur {
                painter.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "🏁 End of video reached.\nClick '⏮ Rewind to Start' below to watch again.",
                    egui::FontId::proportional(16.0),
                    AppTheme::text_secondary(),
                );
            } else if total_dur.as_secs_f64() > 0.0 {
                painter.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "🎞 Loading preview frame...",
                    egui::FontId::proportional(16.0),
                    AppTheme::accent_cyan(),
                );
            } else {
                painter.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "🎬 Welcome! Click '1. 📂 Open Video / Music' above to start.",
                    egui::FontId::proportional(16.0),
                    AppTheme::text_muted(),
                );
            }

            ui.add_space(8.0);

            // Transport & playhead controls.
            ui.horizontal(|ui| {
                ui.add_space(4.0);

                let rewind_btn = Button::new(RichText::new("⏮ Rewind").size(15.0).strong())
                    .min_size(Vec2::new(100.0, 40.0))
                    .fill(AppTheme::bg_card());
                if ui
                    .add(rewind_btn)
                    .on_hover_text("Jump back to the beginning")
                    .clicked()
                {
                    action = PlayerAction::Seek(TimeCode::ZERO);
                }

                let back_btn = Button::new(RichText::new("⏪ -1s").size(14.0))
                    .min_size(Vec2::new(65.0, 40.0))
                    .fill(AppTheme::bg_card());
                if ui.add(back_btn).on_hover_text("Go back 1 second").clicked() {
                    action = PlayerAction::StepSeconds(-1.0);
                }

                let is_playing = timeline.is_playing;
                let play_text = if is_playing { "⏸ PAUSE" } else { "▶ PLAY" };
                let play_btn = Button::new(
                    RichText::new(play_text).size(17.0).strong().color(Color32::WHITE),
                )
                .min_size(Vec2::new(130.0, 40.0))
                .fill(if is_playing {
                    AppTheme::accent_yellow()
                } else {
                    AppTheme::accent_blue()
                });
                if ui.add(play_btn).on_hover_text("Play or Pause video (Spacebar)").clicked() {
                    action = PlayerAction::PlayPauseToggle;
                }

                let fwd_btn = Button::new(RichText::new("⏩ +1s").size(14.0))
                    .min_size(Vec2::new(65.0, 40.0))
                    .fill(AppTheme::bg_card());
                if ui.add(fwd_btn).on_hover_text("Go forward 1 second").clicked() {
                    action = PlayerAction::StepSeconds(1.0);
                }

                let stop_btn = Button::new(RichText::new("⏹ Stop").size(14.0))
                    .min_size(Vec2::new(75.0, 40.0))
                    .fill(AppTheme::bg_card());
                if ui.add(stop_btn).on_hover_text("Stop and return to start").clicked() {
                    action = PlayerAction::Stop;
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                let cur_secs = timeline.playhead.as_secs_f64();
                let tot_secs = total_dur.as_secs_f64();
                let m = |s: f64| (s / 60.0).floor() as u64;
                let s = |s: f64| (s % 60.0).floor() as u64;
                let time_label = format!("{:02}:{:02} / {:02}:{:02}", m(cur_secs), s(cur_secs), m(tot_secs), s(tot_secs));
                ui.label(
                    RichText::new(time_label)
                        .size(16.0)
                        .strong()
                        .color(AppTheme::accent_cyan()),
                );
            });
        });

        action
    }

    fn handle_interactions(
        ui: &Ui,
        response: egui::Response,
        frame: Rect,
        visuals: &[SlideVisual],
        screen_rects: &[Rect],
        place_mode: bool,
        action: &mut PlayerAction,
    ) {
        let to_norm =
            |p: Pos2| ((p.x - frame.min.x) / frame.width(), (p.y - frame.min.y) / frame.height());

        // Placement mode: a plain click drops a pending element where clicked.
        if place_mode && response.clicked() {
            if let Some(p) = response.interact_pointer_pos() {
                let (nx, ny) = to_norm(p);
                *action = PlayerAction::PlaceAt {
                    x: nx.clamp(0.0, 1.0),
                    y: ny.clamp(0.0, 1.0),
                };
                return;
            }
        }

        // Right-click an element -> Full Slide.
        if response.secondary_clicked() {
            if let Some(p) = response.interact_pointer_pos() {
                for (v, srect) in visuals.iter().zip(screen_rects.iter()).rev() {
                    if srect.contains(p) {
                        *action = PlayerAction::FullSlide { idx: v.idx };
                        return;
                    }
                }
            }
        }

        // Drag / resize state machine (stored across frames in egui memory).
        let state = ui.data(|d| d.get_temp::<DragState>(drag_id()));
        if response.dragged() {
            if let Some(st) = state {
                if let Some(p) = response.interact_pointer_pos() {
                    let (px, py) = to_norm(p);
                    match st.mode {
                        DragMode::Move => {
                            if let Some(v) = visuals.iter().find(|v| v.idx == st.idx) {
                                if matches!(v.overlay, Some(_)) {
                                    // Text moves by its center anchor.
                                    *action = PlayerAction::MoveElement {
                                        idx: st.idx,
                                        x: px.clamp(0.0, 1.0),
                                        y: py.clamp(0.0, 1.0),
                                    };
                                } else {
                                    let (sx, sy, _sw, _sh) = st.start_bounds;
                                    *action = PlayerAction::MoveElement {
                                        idx: st.idx,
                                        x: (px - st.grab.x).clamp(0.0, 1.0),
                                        y: (py - st.grab.y).clamp(0.0, 1.0),
                                    };
                                    let _ = (sx, sy);
                                }
                            }
                        }
                        DragMode::Resize => {
                            let (sx, sy, _sw, _sh) = st.start_bounds;
                            let w = (px - sx).clamp(0.02, 1.0);
                            let h = (py - sy).clamp(0.02, 1.0);
                            *action = PlayerAction::ResizeElement {
                                idx: st.idx,
                                x: sx,
                                y: sy,
                                w,
                                h,
                            };
                        }
                    }
                    ui.data_mut(|d| d.insert_temp(drag_id(), st));
                }
                return;
            }
        }

        if response.drag_started() {
            if let Some(p) = response.interact_pointer_pos() {
                // Hit-test topmost element first.
                for (v, srect) in visuals.iter().zip(screen_rects.iter()).rev() {
                    if srect.contains(p) {
                        // Resize only via the bottom-right corner of a box.
                        let corner = srect.max - Vec2::new(12.0, 12.0);
                        let mode = if v.overlay.is_none() && p.x > corner.x && p.y > corner.y {
                            DragMode::Resize
                        } else {
                            DragMode::Move
                        };
                        let (x, y, w, h) = v.bounds;
                        let to_n = |q: Pos2| {
                            ((q.x - frame.min.x) / frame.width(), (q.y - frame.min.y) / frame.height())
                        };
                        let (px, py) = to_n(p);
                        let grab = if v.overlay.is_some() {
                            Vec2::ZERO
                        } else {
                            Vec2::new(px - x, py - y)
                        };
                        ui.data_mut(|d| {
                            d.insert_temp(
                                drag_id(),
                                DragState {
                                    mode,
                                    idx: v.idx,
                                    grab,
                                    start_bounds: (x, y, w, h),
                                },
                            )
                        });
                        return;
                    }
                }
            }
        }

        if response.drag_stopped() {
            ui.data_mut(|d| d.remove::<DragState>(drag_id()));
        }
    }

    fn draw_text_overlay(painter: &egui::Painter, rect: Rect, overlay: &TextOverlay) {
        let raw_text = overlay.formatted_text();
        let lines: Vec<&str> = raw_text.lines().collect();
        if lines.is_empty() {
            return;
        }

        let scale = (rect.height() / 400.0).clamp(0.6, 2.5);
        let font_size = (overlay.font_size * scale * 0.55).max(12.0);
        let family = FontFamily::Name(Arc::from(overlay.font_family.preview_family()));
        let font_id = FontId::new(font_size, family);
        let text_color = overlay.text_color;

        let line_galleys: Vec<_> = lines
            .iter()
            .map(|l| painter.layout_no_wrap(l.to_string(), font_id.clone(), text_color))
            .collect();
        let max_line_w = line_galleys
            .iter()
            .map(|g| g.size().x)
            .fold(0.0f32, |a, b| a.max(b));
        let total_text_h = line_galleys
            .iter()
            .map(|g| g.size().y)
            .sum::<f32>()
            + ((line_galleys.len().saturating_sub(1)) as f32 * 4.0 * scale);

        let pad_x = 20.0 * scale;
        let pad_y = 10.0 * scale;
        let anchor = rect.min
            + Vec2::new(overlay.x * rect.width(), overlay.y * rect.height());

        // Background box.
        let box_rect = match overlay.box_style {
            TextBoxStyle::None => Rect::NOTHING,
            TextBoxStyle::TranslucentBox => Rect::from_center_size(
                anchor,
                Vec2::new(max_line_w + pad_x * 2.0, total_text_h + pad_y * 2.0),
            ),
            TextBoxStyle::SolidBanner => Rect::from_min_max(
                Pos2::new(rect.min.x, anchor.y - total_text_h / 2.0 - pad_y),
                Pos2::new(rect.max.x, anchor.y + total_text_h / 2.0 + pad_y),
            ),
        };
        if overlay.box_style != TextBoxStyle::None {
            let alpha = ((overlay.box_opacity * 255.0).clamp(10.0, 255.0)) as u8;
            painter.rect_filled(box_rect, Rounding::same(6.0), Color32::from_black_alpha(alpha));
            painter.rect_stroke(
                box_rect,
                Rounding::same(6.0),
                egui::Stroke::new(1.0, Color32::from_white_alpha((alpha / 4).max(10))),
            );
        }

        // Draw each line around the anchor.
        let top_y = match overlay.box_style {
            TextBoxStyle::SolidBanner => anchor.y - total_text_h / 2.0,
            _ => anchor.y - total_text_h / 2.0,
        };
        let mut cur_y = top_y;
        let shadow_offset = Vec2::new(1.5 * scale, 1.5 * scale);

        for (i, line) in lines.iter().enumerate() {
            let line_w = line_galleys[i].size().x;
            let line_h = line_galleys[i].size().y;
            let line_x = match overlay.alignment {
                TextAlignment::Left => anchor.x - max_line_w / 2.0 + line_w / 2.0,
                TextAlignment::Center => anchor.x,
                TextAlignment::Right => anchor.x + max_line_w / 2.0 - line_w / 2.0,
            };
            let line_pos = Pos2::new(line_x, cur_y + line_h / 2.0);

            if overlay.show_shadow {
                painter.text(
                    line_pos + shadow_offset,
                    Align2::CENTER_CENTER,
                    *line,
                    font_id.clone(),
                    Color32::from_black_alpha(220),
                );
            }
            painter.text(
                line_pos,
                Align2::CENTER_CENTER,
                *line,
                font_id.clone(),
                text_color,
            );
            cur_y += line_h + 4.0 * scale;
        }
    }
}

#[derive(Clone, Copy)]
struct DragState {
    mode: DragMode,
    idx: usize,
    grab: Vec2,
    start_bounds: (f32, f32, f32, f32),
}
