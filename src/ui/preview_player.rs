use crate::core::timeline::Timeline;
use crate::core::time::TimeCode;
use crate::ui::theme::AppTheme;
use egui::{Button, Color32, ColorImage, Pos2, Rect, RichText, Rounding, TextureHandle, TextureOptions, Ui, Vec2};

pub struct PreviewPlayerView;

pub enum PlayerAction {
    None,
    PlayPauseToggle,
    StepFrames(i64),
    Seek(TimeCode),
    Stop,
}

impl PreviewPlayerView {
    pub fn render(
        ui: &mut Ui,
        timeline: &mut Timeline,
        current_frame: Option<&ColorImage>,
        texture_cache: &mut Option<TextureHandle>,
    ) -> PlayerAction {
        let mut action = PlayerAction::None;

        ui.vertical(|ui| {
            // 1. Video Canvas Display
            let available_size = ui.available_size();
            let canvas_height = (available_size.y - 70.0).max(180.0);
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
            painter.rect_filled(rect, Rounding::same(6.0), Color32::BLACK);

            if let Some(frame) = current_frame {
                let texture = texture_cache.get_or_insert_with(|| {
                    ui.ctx().load_texture("video_preview", frame.clone(), TextureOptions::LINEAR)
                });
                texture.set(frame.clone(), TextureOptions::LINEAR);

                painter.image(
                    texture.id(),
                    rect,
                    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                // Placeholder watermark
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🎞 Video Preview",
                    egui::FontId::proportional(16.0),
                    AppTheme::TEXT_MUTED,
                );
            }

            ui.add_space(6.0);

            // 2. Transport & Playhead Controls
            ui.horizontal(|ui| {
                // Jump to start
                if ui.button("⏮").on_hover_text("Jump to Start (Home)").clicked() {
                    action = PlayerAction::Seek(TimeCode::ZERO);
                }

                // Step Back 1 frame
                if ui.button("⏪").on_hover_text("Step Back 1 Frame (Left Arrow)").clicked() {
                    action = PlayerAction::StepFrames(-1);
                }

                // Play / Pause Toggle
                let play_icon = if timeline.is_playing { "⏸" } else { "▶" };
                let play_btn = Button::new(RichText::new(play_icon).size(15.0).strong())
                    .fill(if timeline.is_playing { AppTheme::ACCENT_YELLOW } else { AppTheme::ACCENT_BLUE });
                if ui.add(play_btn).on_hover_text("Play/Pause (Space)").clicked() {
                    action = PlayerAction::PlayPauseToggle;
                }

                // Step Forward 1 frame
                if ui.button("⏩").on_hover_text("Step Forward 1 Frame (Right Arrow)").clicked() {
                    action = PlayerAction::StepFrames(1);
                }

                // Stop
                if ui.button("⏹").on_hover_text("Stop & Return to Start").clicked() {
                    action = PlayerAction::Stop;
                }

                ui.separator();

                // SMPTE Timecode Counter
                let current_smpte = timeline.playhead.to_smpte_str(timeline.fps);
                let total_smpte = timeline.duration().to_smpte_str(timeline.fps);

                ui.label(
                    RichText::new(format!("{} / {}", current_smpte, total_smpte))
                        .monospace()
                        .color(AppTheme::ACCENT_CYAN)
                        .size(13.0),
                );

                // Scrub Slider
                let mut playhead_secs = timeline.playhead.as_secs_f64();
                let max_secs = timeline.duration().as_secs_f64().max(1.0);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Slider::new(&mut playhead_secs, 0.0..=max_secs)
                                .show_value(false)
                                .text(""),
                        )
                        .changed()
                    {
                        action = PlayerAction::Seek(TimeCode::from_secs_f64(playhead_secs));
                    }
                });
            });
        });

        action
    }
}
