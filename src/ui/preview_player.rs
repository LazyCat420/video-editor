use crate::core::timeline::Timeline;
use crate::core::time::TimeCode;
use crate::ui::theme::AppTheme;
use egui::{Button, Color32, ColorImage, Pos2, Rect, RichText, Rounding, TextureHandle, TextureOptions, Ui, Vec2};

pub struct PreviewPlayerView;

pub enum PlayerAction {
    None,
    PlayPauseToggle,
    StepFrames(i64),
    StepSeconds(f64),
    Seek(TimeCode),
    Stop,
}

impl PreviewPlayerView {
    pub fn render(
        ui: &mut Ui,
        timeline: &mut Timeline,
        current_frame: Option<&ColorImage>,
        active_text_overlay: Option<&crate::core::text_overlay::TextOverlay>,
        texture_cache: &mut Option<TextureHandle>,
        frame_is_dirty: bool,
    ) -> PlayerAction {
        let mut action = PlayerAction::None;

        ui.vertical(|ui| {
            // 1. Video Canvas Display
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

            let (rect, _response) =
                ui.allocate_exact_size(Vec2::new(view_w, view_h), egui::Sense::click());

            let painter = ui.painter_at(rect);
            painter.rect_filled(rect, Rounding::same(8.0), Color32::BLACK);
            painter.rect_stroke(rect, Rounding::same(8.0), egui::Stroke::new(1.5, AppTheme::bg_hover()));

            let total_dur = timeline.duration();

            if let Some(frame) = current_frame {
                let texture = texture_cache.get_or_insert_with(|| {
                    ui.ctx().load_texture("video_preview", frame.clone(), TextureOptions::LINEAR)
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

                // Render On-Screen Text Overlay (if any)
                if let Some(overlay) = active_text_overlay {
                    if !overlay.text.trim().is_empty() {
                        Self::draw_text_overlay(&painter, rect, overlay);
                    }
                }
            } else if total_dur.as_secs_f64() > 0.0 && timeline.playhead >= total_dur {
                // Helpful end-of-video message
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🏁 End of video reached.\nClick '⏮ Rewind to Start' below to watch again.",
                    egui::FontId::proportional(16.0),
                    AppTheme::text_secondary(),
                );
            } else if total_dur.as_secs_f64() > 0.0 {
                // Loading or empty timeline slot message
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🎞 Loading preview frame...",
                    egui::FontId::proportional(16.0),
                    AppTheme::accent_cyan(),
                );
            } else {
                // Initial prompt when no video is loaded
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🎬 Welcome! Click '1. 📂 Open Video / Music' above to start.",
                    egui::FontId::proportional(16.0),
                    AppTheme::text_muted(),
                );
            }

            ui.add_space(8.0);

            // 2. Transport & Playhead Controls (Large & Clear for Seniors)
            ui.horizontal(|ui| {
                ui.add_space(4.0);

                // Rewind to start
                let rewind_btn = Button::new(RichText::new("⏮ Rewind").size(15.0).strong())
                    .min_size(egui::vec2(100.0, 40.0))
                    .fill(AppTheme::bg_card());
                if ui.add(rewind_btn).on_hover_text("Jump back to the beginning").clicked() {
                    action = PlayerAction::Seek(TimeCode::ZERO);
                }

                // Step Back 1 second
                let back_btn = Button::new(RichText::new("⏪ -1s").size(14.0))
                    .min_size(egui::vec2(65.0, 40.0))
                    .fill(AppTheme::bg_card());
                if ui.add(back_btn).on_hover_text("Go back 1 second").clicked() {
                    action = PlayerAction::StepSeconds(-1.0);
                }

                // Big PLAY / PAUSE Toggle Button
                let is_playing = timeline.is_playing;
                let play_text = if is_playing { "⏸ PAUSE" } else { "▶ PLAY" };
                let play_btn = Button::new(
                    RichText::new(play_text)
                        .size(17.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .min_size(egui::vec2(130.0, 40.0))
                .fill(if is_playing { AppTheme::accent_yellow() } else { AppTheme::accent_blue() });

                if ui.add(play_btn).on_hover_text("Play or Pause video (Spacebar)").clicked() {
                    action = PlayerAction::PlayPauseToggle;
                }

                // Step Forward 1 second
                let fwd_btn = Button::new(RichText::new("⏩ +1s").size(14.0))
                    .min_size(egui::vec2(65.0, 40.0))
                    .fill(AppTheme::bg_card());
                if ui.add(fwd_btn).on_hover_text("Go forward 1 second").clicked() {
                    action = PlayerAction::StepSeconds(1.0);
                }

                // Stop Button
                let stop_btn = Button::new(RichText::new("⏹ Stop").size(14.0))
                    .min_size(egui::vec2(75.0, 40.0))
                    .fill(AppTheme::bg_card());
                if ui.add(stop_btn).on_hover_text("Stop and return to start").clicked() {
                    action = PlayerAction::Stop;
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Friendly Human-Readable Time Readout
                let cur_secs = timeline.playhead.as_secs_f64();
                let tot_secs = total_dur.as_secs_f64();

                let cur_m = (cur_secs / 60.0).floor() as u64;
                let cur_s = (cur_secs % 60.0).floor() as u64;
                let tot_m = (tot_secs / 60.0).floor() as u64;
                let tot_s = (tot_secs % 60.0).floor() as u64;

                let time_label = format!("{:02}:{:02} / {:02}:{:02}", cur_m, cur_s, tot_m, tot_s);

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

    fn draw_text_overlay(
        painter: &egui::Painter,
        rect: Rect,
        overlay: &crate::core::text_overlay::TextOverlay,
    ) {
        use crate::core::text_overlay::{FontFamilyPreset, TextAlignment, TextBoxStyle, TextPosition};

        let raw_text = overlay.formatted_text();
        let lines: Vec<&str> = raw_text.lines().collect();
        if lines.is_empty() {
            return;
        }

        let scale = (rect.height() / 400.0).clamp(0.6, 2.5);
        let font_size = (overlay.font_size * scale * 0.55).max(12.0);

        let font_family = match overlay.font_family {
            FontFamilyPreset::Monospace => egui::FontFamily::Monospace,
            _ => egui::FontFamily::Proportional,
        };

        let font_id = egui::FontId::new(font_size, font_family);
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

        let (anchor_pos, v_align) = match overlay.position {
            TextPosition::Center => (rect.center(), egui::Align2::CENTER_CENTER),
            TextPosition::BottomBanner => (
                Pos2::new(rect.center().x, rect.max.y - 35.0 * scale),
                egui::Align2::CENTER_BOTTOM,
            ),
            TextPosition::TopHeader => (
                Pos2::new(rect.center().x, rect.min.y + 25.0 * scale),
                egui::Align2::CENTER_TOP,
            ),
            TextPosition::LowerThird => (
                Pos2::new(rect.min.x + 35.0 * scale, rect.max.y - 35.0 * scale),
                egui::Align2::LEFT_BOTTOM,
            ),
        };

        // Calculate Box Bounds
        let box_rect = match overlay.box_style {
            TextBoxStyle::None => Rect::ZERO,
            TextBoxStyle::TranslucentBox => match v_align {
                egui::Align2::CENTER_CENTER => Rect::from_center_size(
                    anchor_pos,
                    egui::vec2(max_line_w + pad_x * 2.0, total_text_h + pad_y * 2.0),
                ),
                egui::Align2::CENTER_BOTTOM => Rect::from_min_max(
                    Pos2::new(anchor_pos.x - max_line_w / 2.0 - pad_x, anchor_pos.y - total_text_h - pad_y),
                    Pos2::new(anchor_pos.x + max_line_w / 2.0 + pad_x, anchor_pos.y + pad_y),
                ),
                egui::Align2::CENTER_TOP => Rect::from_min_max(
                    Pos2::new(anchor_pos.x - max_line_w / 2.0 - pad_x, anchor_pos.y - pad_y),
                    Pos2::new(anchor_pos.x + max_line_w / 2.0 + pad_x, anchor_pos.y + total_text_h + pad_y),
                ),
                egui::Align2::LEFT_BOTTOM => Rect::from_min_max(
                    Pos2::new(anchor_pos.x - pad_x, anchor_pos.y - total_text_h - pad_y),
                    Pos2::new(anchor_pos.x + max_line_w + pad_x, anchor_pos.y + pad_y),
                ),
                _ => Rect::from_center_size(anchor_pos, egui::vec2(max_line_w + pad_x * 2.0, total_text_h + pad_y * 2.0)),
            },
            TextBoxStyle::SolidBanner => {
                let box_top = match v_align {
                    egui::Align2::CENTER_BOTTOM => anchor_pos.y - total_text_h - pad_y * 1.5,
                    egui::Align2::CENTER_TOP => anchor_pos.y - pad_y * 0.5,
                    _ => anchor_pos.y - total_text_h / 2.0 - pad_y,
                };
                Rect::from_min_max(
                    Pos2::new(rect.min.x, box_top),
                    Pos2::new(rect.max.x, box_top + total_text_h + pad_y * 2.0),
                )
            }
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

        // Draw Multi-line Text
        let top_start_y = match v_align {
            egui::Align2::CENTER_CENTER => anchor_pos.y - total_text_h / 2.0,
            egui::Align2::CENTER_BOTTOM => anchor_pos.y - total_text_h,
            egui::Align2::CENTER_TOP => anchor_pos.y,
            egui::Align2::LEFT_BOTTOM => anchor_pos.y - total_text_h,
            _ => anchor_pos.y - total_text_h / 2.0,
        };

        let mut cur_y = top_start_y;
        let shadow_offset = egui::vec2(1.5 * scale, 1.5 * scale);

        for (i, line) in lines.iter().enumerate() {
            let line_w = line_galleys[i].size().x;
            let line_h = line_galleys[i].size().y;

            let line_center_x = match overlay.alignment {
                TextAlignment::Left => {
                    if overlay.position == TextPosition::LowerThird {
                        anchor_pos.x + line_w / 2.0
                    } else {
                        anchor_pos.x - max_line_w / 2.0 + line_w / 2.0
                    }
                }
                TextAlignment::Center => anchor_pos.x,
                TextAlignment::Right => {
                    if overlay.position == TextPosition::LowerThird {
                        anchor_pos.x + max_line_w - line_w / 2.0
                    } else {
                        anchor_pos.x + max_line_w / 2.0 - line_w / 2.0
                    }
                }
            };

            let line_pos = Pos2::new(line_center_x, cur_y + line_h / 2.0);

            if overlay.show_shadow {
                painter.text(
                    line_pos + shadow_offset,
                    egui::Align2::CENTER_CENTER,
                    *line,
                    font_id.clone(),
                    Color32::from_black_alpha(220),
                );
            }

            painter.text(
                line_pos,
                egui::Align2::CENTER_CENTER,
                *line,
                font_id.clone(),
                text_color,
            );

            cur_y += line_h + 4.0 * scale;
        }
    }
}
