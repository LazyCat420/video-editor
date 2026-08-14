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
            painter.rect_stroke(rect, Rounding::same(8.0), egui::Stroke::new(1.5, AppTheme::BG_HOVER));

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
            } else if total_dur.as_secs_f64() > 0.0 && timeline.playhead >= total_dur {
                // Helpful end-of-video message
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🏁 End of video reached.\nClick '⏮ Rewind to Start' below to watch again.",
                    egui::FontId::proportional(16.0),
                    AppTheme::TEXT_SECONDARY,
                );
            } else if total_dur.as_secs_f64() > 0.0 {
                // Loading or empty timeline slot message
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🎞 Loading preview frame...",
                    egui::FontId::proportional(16.0),
                    AppTheme::ACCENT_CYAN,
                );
            } else {
                // Initial prompt when no video is loaded
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🎬 Welcome! Click '1. 📂 Open Video / Music' above to start.",
                    egui::FontId::proportional(16.0),
                    AppTheme::TEXT_MUTED,
                );
            }

            ui.add_space(8.0);

            // 2. Transport & Playhead Controls (Large & Clear for Seniors)
            ui.horizontal(|ui| {
                ui.add_space(4.0);

                // Rewind to start
                let rewind_btn = Button::new(RichText::new("⏮ Rewind").size(15.0).strong())
                    .min_size(egui::vec2(100.0, 40.0))
                    .fill(AppTheme::BG_CARD);
                if ui.add(rewind_btn).on_hover_text("Jump back to the beginning").clicked() {
                    action = PlayerAction::Seek(TimeCode::ZERO);
                }

                // Step Back 1 second
                let back_btn = Button::new(RichText::new("⏪ -1s").size(14.0))
                    .min_size(egui::vec2(65.0, 40.0))
                    .fill(AppTheme::BG_CARD);
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
                .fill(if is_playing { AppTheme::ACCENT_YELLOW } else { AppTheme::ACCENT_BLUE });

                if ui.add(play_btn).on_hover_text("Play or Pause video (Spacebar)").clicked() {
                    action = PlayerAction::PlayPauseToggle;
                }

                // Step Forward 1 second
                let fwd_btn = Button::new(RichText::new("⏩ +1s").size(14.0))
                    .min_size(egui::vec2(65.0, 40.0))
                    .fill(AppTheme::BG_CARD);
                if ui.add(fwd_btn).on_hover_text("Go forward 1 second").clicked() {
                    action = PlayerAction::StepSeconds(1.0);
                }

                // Stop Button
                let stop_btn = Button::new(RichText::new("⏹ Stop").size(14.0))
                    .min_size(egui::vec2(75.0, 40.0))
                    .fill(AppTheme::BG_CARD);
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
                        .color(AppTheme::ACCENT_CYAN),
                );

                // Big Scrub Slider across remaining width
                let mut slider_val = cur_secs;
                let max_secs = tot_secs.max(1.0);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add_sized(
                            [ui.available_width().max(80.0), 30.0],
                            egui::Slider::new(&mut slider_val, 0.0..=max_secs)
                                .show_value(false)
                                .text(""),
                        )
                        .changed()
                    {
                        action = PlayerAction::Seek(TimeCode::from_secs_f64(slider_val));
                    }
                });
            });
        });

        action
    }
}
