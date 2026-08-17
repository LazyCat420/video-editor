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
    pub calendar: Option<crate::core::text_overlay::CalendarOverlay>,
    pub kind: SlideVisualKind,
    pub label: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SlideVisualKind {
    Text,
    Picture,
    Video,
    Placeholder,
    Calendar,
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
    /// Resize / scale text font size.
    ScaleTextSize { idx: usize, font_size: f32 },
    /// Fill the whole frame with element `idx`.
    FullSlide { idx: usize },
    /// Promote picture element `idx` to slide background.
    SetAsBackground { idx: usize },
    /// Select or deselect element.
    SelectElement(Option<usize>),
    /// Delete element `idx`.
    DeleteElement(usize),
    /// Live inline text edit on canvas.
    UpdateTextContent { idx: usize, text: String },
    /// Drop a media asset from MediaBin onto canvas at normalized (x, y).
    DropMediaAsset { asset_id: u64, x: f32, y: f32 },
    /// Drop external files from OS onto canvas at normalized (x, y).
    DropFiles { paths: Vec<std::path::PathBuf>, x: f32, y: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeHandle {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug)]
enum DragMode {
    Move,
    Resize(ResizeHandle),
}

#[derive(Clone, Copy, Debug)]
struct DragState {
    mode: DragMode,
    idx: usize,
    grab: Vec2,
    start_bounds: (f32, f32, f32, f32),
}

fn drag_id() -> Id {
    Id::new("preview_slide_drag")
}

pub fn detect_resize_handle(srect: Rect, p: Pos2) -> Option<ResizeHandle> {
    let handle_radius = 18.0; // 36x36px generous hit zone around corners/edges

    // 1. Check 4 corners first (highest priority)
    if p.distance(srect.left_top()) <= handle_radius {
        return Some(ResizeHandle::TopLeft);
    }
    if p.distance(srect.right_top()) <= handle_radius {
        return Some(ResizeHandle::TopRight);
    }
    if p.distance(srect.left_bottom()) <= handle_radius {
        return Some(ResizeHandle::BottomLeft);
    }
    if p.distance(srect.right_bottom()) <= handle_radius {
        return Some(ResizeHandle::BottomRight);
    }

    // 2. Check 4 edge centers
    if p.distance(srect.center_top()) <= handle_radius {
        return Some(ResizeHandle::Top);
    }
    if p.distance(srect.center_bottom()) <= handle_radius {
        return Some(ResizeHandle::Bottom);
    }
    if p.distance(srect.left_center()) <= handle_radius {
        return Some(ResizeHandle::Left);
    }
    if p.distance(srect.right_center()) <= handle_radius {
        return Some(ResizeHandle::Right);
    }

    // 3. Check border edge zones (8px band along outer edges)
    let edge_tol = 8.0;
    let in_horizontal = p.x >= srect.left() - edge_tol && p.x <= srect.right() + edge_tol;
    let in_vertical = p.y >= srect.top() - edge_tol && p.y <= srect.bottom() + edge_tol;

    if in_horizontal && (p.y - srect.top()).abs() <= edge_tol {
        return Some(ResizeHandle::Top);
    }
    if in_horizontal && (p.y - srect.bottom()).abs() <= edge_tol {
        return Some(ResizeHandle::Bottom);
    }
    if in_vertical && (p.x - srect.left()).abs() <= edge_tol {
        return Some(ResizeHandle::Left);
    }
    if in_vertical && (p.x - srect.right()).abs() <= edge_tol {
        return Some(ResizeHandle::Right);
    }

    None
}

pub fn resize_cursor_icon(handle: ResizeHandle) -> egui::CursorIcon {
    match handle {
        ResizeHandle::TopLeft => egui::CursorIcon::ResizeNorthWest,
        ResizeHandle::TopRight => egui::CursorIcon::ResizeNorthEast,
        ResizeHandle::BottomLeft => egui::CursorIcon::ResizeSouthWest,
        ResizeHandle::BottomRight => egui::CursorIcon::ResizeSouthEast,
        ResizeHandle::Top => egui::CursorIcon::ResizeNorth,
        ResizeHandle::Bottom => egui::CursorIcon::ResizeSouth,
        ResizeHandle::Left => egui::CursorIcon::ResizeWest,
        ResizeHandle::Right => egui::CursorIcon::ResizeEast,
    }
}

pub fn calculate_resized_bounds(
    handle: ResizeHandle,
    start_bounds: (f32, f32, f32, f32),
    px: f32,
    py: f32,
) -> (f32, f32, f32, f32) {
    let (sx, sy, sw, sh) = start_bounds;
    let min_size: f32 = 0.04;
    let right_edge = sx + sw;
    let bottom_edge = sy + sh;

    match handle {
        ResizeHandle::BottomRight => {
            let w = (px - sx).clamp(min_size, 1.0 - sx);
            let h = (py - sy).clamp(min_size, 1.0 - sy);
            (sx, sy, w, h)
        }
        ResizeHandle::BottomLeft => {
            let left = px.clamp(0.0, right_edge - min_size);
            let w = right_edge - left;
            let h = (py - sy).clamp(min_size, 1.0 - sy);
            (left, sy, w, h)
        }
        ResizeHandle::TopRight => {
            let w = (px - sx).clamp(min_size, 1.0 - sx);
            let top = py.clamp(0.0, bottom_edge - min_size);
            let h = bottom_edge - top;
            (sx, top, w, h)
        }
        ResizeHandle::TopLeft => {
            let left = px.clamp(0.0, right_edge - min_size);
            let top = py.clamp(0.0, bottom_edge - min_size);
            let w = right_edge - left;
            let h = bottom_edge - top;
            (left, top, w, h)
        }
        ResizeHandle::Top => {
            let top = py.clamp(0.0, bottom_edge - min_size);
            let h = bottom_edge - top;
            (sx, top, sw, h)
        }
        ResizeHandle::Bottom => {
            let h = (py - sy).clamp(min_size, 1.0 - sy);
            (sx, sy, sw, h)
        }
        ResizeHandle::Left => {
            let left = px.clamp(0.0, right_edge - min_size);
            let w = right_edge - left;
            (left, sy, w, sh)
        }
        ResizeHandle::Right => {
            let w = (px - sx).clamp(min_size, 1.0 - sx);
            (sx, sy, w, sh)
        }
    }
}

fn visual_screen_rect(frame: Rect, b: (f32, f32, f32, f32), overlay: Option<&TextOverlay>) -> Rect {
    let (x, y, w, h) = b;
    let tl = frame.min + Vec2::new(x * frame.width(), y * frame.height());
    if w <= 0.0 && h <= 0.0 {
        let scale = (frame.height() / 400.0).clamp(0.6, 2.5);
        let font_size = overlay.map(|o| (o.font_size * scale * 0.55).max(11.0)).unwrap_or(18.0);
        let line_count = overlay.map(|o| o.text.lines().count().max(1)).unwrap_or(1);
        let max_chars = overlay
            .map(|o| o.text.lines().map(|l| l.chars().count()).max().unwrap_or(12).max(12))
            .unwrap_or(12);

        let char_width = font_size * 0.58;
        let content_w = (max_chars as f32) * char_width;
        let content_h = (line_count as f32) * (font_size * 1.18);

        let pad_x = 16.0 * scale;
        let pad_y = 10.0 * scale;
        let estimated_w = (content_w + pad_x * 2.0).clamp(80.0, frame.width() * 0.98);
        let estimated_h = (content_h + pad_y * 2.0).clamp(32.0, frame.height() * 0.98);
        Rect::from_center_size(tl, Vec2::new(estimated_w, estimated_h))
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
        selected_element: Option<usize>,
        view_mode: &mut crate::ui::MainViewMode,
    ) -> PlayerAction {
        let mut action = PlayerAction::None;

        ui.vertical_centered(|ui| {
            let available_size = ui.available_size();
            let canvas_height = (available_size.y - 72.0).max(200.0);
            let canvas_width = (available_size.x - 12.0).max(300.0);

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

            let screen_rects: Vec<Rect> = visuals
                .iter()
                .map(|v| visual_screen_rect(rect, v.bounds, v.overlay.as_ref()))
                .collect();

            // 1. Draw Background (Video frame, Solid Color, or Blank Canvas)
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
            } else if visuals.is_empty() && total_dur.as_secs_f64() > 0.0 && timeline.playhead >= total_dur {
                painter.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "🏁 End of video reached.\nClick '⏮ Rewind to Start' below to watch again.",
                    egui::FontId::proportional(16.0),
                    AppTheme::text_secondary(),
                );
            } else if visuals.is_empty() && total_dur.as_secs_f64() > 0.0 {
                painter.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "🎞 Loading preview frame...",
                    egui::FontId::proportional(16.0),
                    AppTheme::accent_cyan(),
                );
            } else if visuals.is_empty() {
                painter.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "🎬 Drag & drop videos/photos here to start, or click '+ Add Video / Music'",
                    egui::FontId::proportional(14.0),
                    AppTheme::text_muted(),
                );
            }

            // 2. ALWAYS draw slide visual elements (pictures, videos, text, calendar, placeholders)
            for (v, srect) in visuals.iter().zip(screen_rects.iter()) {
                if v.kind == SlideVisualKind::Placeholder {
                    let is_hovered = ui.input(|i| i.pointer.hover_pos()).map(|p| srect.contains(p)).unwrap_or(false);
                    let bg_col = if is_hovered {
                        Color32::from_rgba_premultiplied(40, 60, 90, 180)
                    } else {
                        Color32::from_rgba_premultiplied(20, 24, 35, 160)
                    };
                    painter.rect_filled(*srect, Rounding::same(6.0), bg_col);
                    painter.rect_stroke(
                        *srect,
                        Rounding::same(6.0),
                        egui::Stroke::new(1.8, if is_hovered { AppTheme::accent_yellow() } else { AppTheme::accent_blue() }),
                    );
                    let slot_text = v.label.as_deref().unwrap_or("➕ Drop Media Here");
                    painter.text(
                        srect.center(),
                        Align2::CENTER_CENTER,
                        slot_text,
                        egui::FontId::proportional(13.0),
                        if is_hovered { Color32::WHITE } else { AppTheme::accent_cyan() },
                    );
                } else if v.kind == SlideVisualKind::Calendar {
                    if let Some(cal) = &v.calendar {
                        crate::ui::calendar_renderer::CalendarRenderer::draw_calendar_element(&painter, *srect, cal);
                    }
                } else if let Some(tex) = &v.texture {
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
                    if selected_element == Some(v.idx) {
                        let anchor = rect.min + Vec2::new(overlay.x * rect.width(), overlay.y * rect.height());
                        let scale = (rect.height() / 400.0).clamp(0.6, 2.5);
                        let font_size = (overlay.font_size * scale * 0.55).max(12.0);
                        let family = FontFamily::Name(Arc::from(overlay.font_family.preview_family()));
                        let font_id = FontId::new(font_size, family);

                        let mut text_buf = overlay.text.clone();
                        let edit_id = Id::new("canvas_inline_text_edit").with(v.idx);

                        let align = match overlay.alignment {
                            TextAlignment::Left => egui::Align::Min,
                            TextAlignment::Center => egui::Align::Center,
                            TextAlignment::Right => egui::Align::Max,
                        };

                        let edit_w = srect.width().max(160.0);
                        let edit_h = srect.height().max(44.0);
                        let edit_rect = Rect::from_center_size(anchor, Vec2::new(edit_w, edit_h));

                        // Background styling
                        if overlay.box_style != TextBoxStyle::None {
                            let alpha = ((overlay.box_opacity * 255.0).clamp(10.0, 255.0)) as u8;
                            painter.rect_filled(edit_rect, Rounding::same(6.0), Color32::from_black_alpha(alpha));
                            painter.rect_stroke(
                                edit_rect,
                                Rounding::same(6.0),
                                egui::Stroke::new(1.0, Color32::from_white_alpha((alpha / 4).max(10))),
                            );
                        }

                        let text_edit = egui::TextEdit::multiline(&mut text_buf)
                            .id(edit_id)
                            .font(font_id)
                            .text_color(overlay.text_color)
                            .horizontal_align(align)
                            .hint_text("Type text here...")
                            .frame(false)
                            .desired_width(edit_w - 12.0)
                            .margin(egui::Margin::symmetric(6.0, 3.0));

                        let edit_resp = ui.put(edit_rect, text_edit);

                        // Auto focus when element was newly selected
                        let focus_id = Id::new("canvas_text_focused_elem");
                        let last_focused: Option<usize> = ui.data(|d| d.get_temp(focus_id));
                        if last_focused != Some(v.idx) {
                            edit_resp.request_focus();
                            ui.data_mut(|d| d.insert_temp(focus_id, Some(v.idx)));
                        }

                        if edit_resp.changed() {
                            action = PlayerAction::UpdateTextContent {
                                idx: v.idx,
                                text: text_buf,
                            };
                        }
                    } else {
                        crate::ui::text_renderer::TextRenderer::draw_text_overlay(&painter, rect, overlay);
                    }
                }
            }

            // 3. ALWAYS draw selection highlight and 8 resize handles for active element
            for (v, srect) in visuals.iter().zip(screen_rects.iter()) {
                if selected_element == Some(v.idx) {
                    painter.rect_stroke(
                        *srect,
                        Rounding::same(4.0),
                        egui::Stroke::new(2.0, AppTheme::accent_blue()),
                    );

                    // 8 Resize Handles (4 corners + 4 edge centers) with crisp high-contrast white fill & blue border
                    let handle_size = Vec2::splat(11.0);
                    let handle_points = [
                        srect.left_top(),
                        srect.center_top(),
                        srect.right_top(),
                        srect.right_center(),
                        srect.right_bottom(),
                        srect.center_bottom(),
                        srect.left_bottom(),
                        srect.left_center(),
                    ];
                    for pt in handle_points {
                        let hrect = Rect::from_center_size(pt, handle_size);
                        painter.rect_filled(hrect, Rounding::same(2.5), Color32::WHITE);
                        painter.rect_stroke(
                            hrect,
                            Rounding::same(2.5),
                            egui::Stroke::new(1.5, AppTheme::accent_blue()),
                        );
                    }
                }
            }
            // Visual feedback when dragging media over the canvas
            let is_dnd_hovering = response.dnd_hover_payload::<crate::ui::MediaAssetDrag>().is_some();
            if is_dnd_hovering {
                painter.rect_filled(
                    rect,
                    Rounding::same(8.0),
                    Color32::from_rgba_premultiplied(0, 140, 255, 45),
                );
                painter.rect_stroke(
                    rect,
                    Rounding::same(8.0),
                    egui::Stroke::new(2.5, AppTheme::accent_blue()),
                );
                painter.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    "⬇ Drop to add media to slide",
                    egui::FontId::proportional(18.0),
                    Color32::WHITE,
                );
            }

            // Interactions: drag-and-drop, place, select, move, multi-handle resize, context menu
            Self::handle_interactions(
                ui,
                response,
                rect,
                visuals,
                &screen_rects,
                place_mode,
                selected_element,
                &mut action,
            );

            ui.add_space(8.0);

            // Transport & playhead controls + Mode Switcher
            ui.horizontal_centered(|ui| {
                ui.add_space(4.0);

                // Mode Switcher: Slideshow Studio vs Timeline Editor (placed right next to Rewind button!)
                let is_slideshow = *view_mode == crate::ui::MainViewMode::Slideshow;
                let slide_mode_btn = Button::new(
                    RichText::new("🖼 Slideshow")
                        .size(13.5)
                        .strong()
                        .color(if is_slideshow { Color32::WHITE } else { AppTheme::text_secondary() }),
                )
                .fill(if is_slideshow { AppTheme::accent_blue() } else { AppTheme::bg_card() })
                .min_size(Vec2::new(105.0, 40.0));

                if ui.add(slide_mode_btn).on_hover_text("Switch to PowerPoint-style Slideshow Studio").clicked() {
                    *view_mode = crate::ui::MainViewMode::Slideshow;
                }

                let time_mode_btn = Button::new(
                    RichText::new("⏱ Timeline")
                        .size(13.5)
                        .strong()
                        .color(if !is_slideshow { Color32::WHITE } else { AppTheme::text_secondary() }),
                )
                .fill(if !is_slideshow { AppTheme::accent_blue() } else { AppTheme::bg_card() })
                .min_size(Vec2::new(105.0, 40.0));

                if ui.add(time_mode_btn).on_hover_text("Switch to Multi-Track Video Timeline Editor").clicked() {
                    *view_mode = crate::ui::MainViewMode::Timeline;
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                let rewind_btn = Button::new(RichText::new("⏮ Rewind").size(14.0))
                    .min_size(Vec2::new(80.0, 40.0))
                    .fill(AppTheme::bg_card());
                if ui.add(rewind_btn).on_hover_text("Jump to the beginning").clicked() {
                    action = PlayerAction::Seek(TimeCode::ZERO);
                }

                let step_back_btn = Button::new(RichText::new("⏪ -1s").size(14.0))
                    .min_size(Vec2::new(65.0, 40.0))
                    .fill(AppTheme::bg_card());
                if ui.add(step_back_btn).on_hover_text("Step backward 1 second").clicked() {
                    action = PlayerAction::StepSeconds(-1.0);
                }

                let play_icon = if timeline.is_playing {
                    "⏸ PAUSE"
                } else {
                    "▶ PLAY"
                };
                let play_btn = Button::new(RichText::new(play_icon).size(15.0).strong().color(Color32::WHITE))
                    .min_size(Vec2::new(95.0, 40.0))
                    .fill(if timeline.is_playing {
                        AppTheme::accent_yellow()
                    } else {
                        AppTheme::accent_blue()
                    });

                if ui.add(play_btn).on_hover_text("Play or Pause video (Spacebar)").clicked() {
                    action = PlayerAction::PlayPauseToggle;
                }

                let step_fwd_btn = Button::new(RichText::new("⏩ +1s").size(14.0))
                    .min_size(Vec2::new(65.0, 40.0))
                    .fill(AppTheme::bg_card());
                if ui.add(step_fwd_btn).on_hover_text("Step forward 1 second").clicked() {
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
        selected_element: Option<usize>,
        action: &mut PlayerAction,
    ) {
        let to_norm =
            |p: Pos2| ((p.x - frame.min.x) / frame.width(), (p.y - frame.min.y) / frame.height());

        // 1. Drag & drop hovering from Files panel (MediaAssetDrag)
        if let Some(_payload) = response.dnd_hover_payload::<crate::ui::MediaAssetDrag>() {
            let hover_painter = ui.painter_at(frame);
            hover_painter.rect_stroke(
                frame,
                Rounding::same(8.0),
                egui::Stroke::new(3.0, AppTheme::accent_cyan()),
            );
            let badge_rect = Rect::from_center_size(frame.center(), Vec2::new(220.0, 44.0));
            hover_painter.rect_filled(badge_rect, Rounding::same(8.0), Color32::from_black_alpha(210));
            hover_painter.rect_stroke(badge_rect, Rounding::same(8.0), egui::Stroke::new(1.5, AppTheme::accent_cyan()));
            hover_painter.text(
                frame.center(),
                Align2::CENTER_CENTER,
                "➕ Drop onto slide",
                FontId::proportional(16.0),
                AppTheme::accent_cyan(),
            );
        }

        // 2. Drag & drop release from Files panel (MediaAssetDrag)
        if let Some(drop) = response.dnd_release_payload::<crate::ui::MediaAssetDrag>() {
            if let Some(p) = ui.input(|i| i.pointer.hover_pos()) {
                let (nx, ny) = to_norm(p);
                *action = PlayerAction::DropMediaAsset {
                    asset_id: drop.0,
                    x: nx.clamp(0.0, 1.0),
                    y: ny.clamp(0.0, 1.0),
                };
                return;
            }
        }

        // 4. Placement mode: a plain click drops a pending element where clicked.
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

        // 5. Keyboard shortcuts for selected element
        if let Some(sel_idx) = selected_element {
            if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
                *action = PlayerAction::DeleteElement(sel_idx);
                return;
            }
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                *action = PlayerAction::SelectElement(None);
                return;
            }
        }

        // 6. Right-click context menu on element
        response.context_menu(|ui| {
            if let Some(p) = ui.input(|i| i.pointer.hover_pos()) {
                let mut hit_idx = None;
                let mut hit_kind = None;
                for (v, srect) in visuals.iter().zip(screen_rects.iter()).rev() {
                    if srect.contains(p) {
                        hit_idx = Some(v.idx);
                        hit_kind = Some(v.kind);
                        break;
                    }
                }
                if let Some(idx) = hit_idx {
                    ui.label(RichText::new("Slide Item Actions").strong().color(AppTheme::accent_yellow()));
                    ui.separator();
                    if ui.button("⛶ Fill Entire Slide").clicked() {
                        *action = PlayerAction::FullSlide { idx };
                        ui.close_menu();
                    }
                    if hit_kind == Some(SlideVisualKind::Picture) {
                        if ui.button("🖼 Set as Slide Background").clicked() {
                            *action = PlayerAction::SetAsBackground { idx };
                            ui.close_menu();
                        }
                    }
                    if ui.button("🗑 Delete Element").clicked() {
                        *action = PlayerAction::DeleteElement(idx);
                        ui.close_menu();
                    }
                } else {
                    ui.label("Click on an element for actions");
                }
            }
        });

        // 7. Dynamic cursor hover styling across all 8 handles + inside element
        if let Some(p) = response.hover_pos() {
            let mut cursor_set = false;
            // Check selected element first for handle priority
            if let Some(sel_idx) = selected_element {
                if let Some((_v, srect)) = visuals.iter().zip(screen_rects.iter()).find(|(v, _)| v.idx == sel_idx) {
                    if let Some(handle) = detect_resize_handle(*srect, p) {
                        ui.ctx().set_cursor_icon(resize_cursor_icon(handle));
                        cursor_set = true;
                    }
                }
            }
            if !cursor_set {
                for (_v, srect) in visuals.iter().zip(screen_rects.iter()).rev() {
                    if let Some(handle) = detect_resize_handle(*srect, p) {
                        ui.ctx().set_cursor_icon(resize_cursor_icon(handle));
                        break;
                    } else if srect.expand(6.0).contains(p) {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Move);
                        break;
                    }
                }
            }
        }

        // 8. Selection via click
        if response.clicked() && !place_mode {
            if let Some(p) = response.interact_pointer_pos() {
                let mut hit = None;
                for (v, srect) in visuals.iter().zip(screen_rects.iter()).rev() {
                    if srect.expand(8.0).contains(p) || detect_resize_handle(*srect, p).is_some() {
                        hit = Some(v.idx);
                        break;
                    }
                }
                *action = PlayerAction::SelectElement(hit);
            }
        }

        // 9. Drag / resize state machine supporting all handles & sides
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
                                    let (_sx, _sy, sw, sh) = st.start_bounds;
                                    let max_x = (1.0 - sw).max(0.0);
                                    let max_y = (1.0 - sh).max(0.0);
                                    *action = PlayerAction::MoveElement {
                                        idx: st.idx,
                                        x: (px - st.grab.x).clamp(0.0, max_x),
                                        y: (py - st.grab.y).clamp(0.0, max_y),
                                    };
                                }
                            }
                        }
                        DragMode::Resize(handle) => {
                            if let Some(v) = visuals.iter().find(|v| v.idx == st.idx) {
                                if let Some(_o) = &v.overlay {
                                    let (sx, sy, orig_font_size, _orig_w): (f32, f32, f32, f32) = st.start_bounds;
                                    let dist: f32 = ((px - sx).powi(2) + (py - sy).powi(2)).sqrt().max(0.05);
                                    let scale: f32 = (dist / 0.20).clamp(0.4, 3.5);
                                    let new_size: f32 = (orig_font_size * scale).clamp(10.0, 72.0);
                                    *action = PlayerAction::ScaleTextSize {
                                        idx: st.idx,
                                        font_size: new_size,
                                    };
                                } else {
                                    let (x, y, w, h) = calculate_resized_bounds(handle, st.start_bounds, px, py);
                                    *action = PlayerAction::ResizeElement {
                                        idx: st.idx,
                                        x,
                                        y,
                                        w,
                                        h,
                                    };
                                }
                            }
                        }
                    }
                    ui.data_mut(|d| d.insert_temp(drag_id(), st));
                }
                return;
            }
        }

        if response.drag_started() {
            if let Some(p) = response.interact_pointer_pos() {
                // First check if selected element's handles were grabbed
                let mut target_v_srect = None;
                if let Some(sel_idx) = selected_element {
                    if let Some((v, srect)) = visuals.iter().zip(screen_rects.iter()).find(|(v, _)| v.idx == sel_idx) {
                        if let Some(handle) = detect_resize_handle(*srect, p) {
                            target_v_srect = Some((v, *srect, Some(handle), true));
                        }
                    }
                }

                // If not dragging selected handle, hit-test all visuals from top to bottom
                if target_v_srect.is_none() {
                    for (v, srect) in visuals.iter().zip(screen_rects.iter()).rev() {
                        let handle_opt = detect_resize_handle(*srect, p);
                        let is_inside = srect.expand(8.0).contains(p);
                        if handle_opt.is_some() || is_inside {
                            target_v_srect = Some((v, *srect, handle_opt, is_inside));
                            break;
                        }
                    }
                }

                if let Some((v, srect, handle_opt, _is_inside)) = target_v_srect {
                    *action = PlayerAction::SelectElement(Some(v.idx));
                    let mode = if let Some(handle) = handle_opt {
                        DragMode::Resize(handle)
                    } else {
                        DragMode::Move
                    };
                    let (x, y, w, h) = if let Some(o) = &v.overlay {
                        (o.x, o.y, o.font_size, srect.width())
                    } else {
                        v.bounds
                    };
                    let to_n = |q: Pos2| {
                        ((q.x - frame.min.x) / frame.width(), (q.y - frame.min.y) / frame.height())
                    };
                    let (px, py) = to_n(p);
                    let grab = if handle_opt.is_some() || v.overlay.is_some() {
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

        if response.drag_stopped() {
            ui.data_mut(|d| d.remove::<DragState>(drag_id()));
        }
    }
}