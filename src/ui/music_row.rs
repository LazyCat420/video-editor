use crate::core::timeline::Timeline;
use crate::core::track::TrackKind;
use crate::ui::theme::AppTheme;
use egui::{Button, RichText, Ui};

/// What the user did in the music row this frame.
pub enum MusicRowAction {
    None,
    AddMusicClicked,
    RemoveClip(u64),
    /// Live value from the whole-track volume slider, 0.0..=1.0.
    SetTrackVolume(f32),
}

fn mmss(secs: f64) -> String {
    let s = secs.max(0.0) as u64;
    format!("{}:{:02}", s / 60, s % 60)
}

/// The song list under the slide filmstrip: add / remove songs, one volume
/// slider. Songs always play one after another — no positioning to manage.
pub struct MusicRowView;

impl MusicRowView {
    pub fn render(ui: &mut Ui, timeline: &Timeline) -> MusicRowAction {
        let mut action = MusicRowAction::None;
        let track = timeline.tracks.iter().find(|t| t.kind == TrackKind::Audio);

        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(
                RichText::new("🎵 Music:")
                    .strong()
                    .color(AppTheme::text_secondary()),
            );

            if ui
                .add(
                    Button::new(RichText::new("🎵 Add Music").strong())
                        .min_size(egui::vec2(110.0, 30.0))
                        .fill(AppTheme::bg_card()),
                )
                .on_hover_text("Pick a song — it plays after the songs already added")
                .clicked()
            {
                action = MusicRowAction::AddMusicClicked;
            }

            let Some(track) = track else {
                return;
            };

            // Right side first (right_to_left): volume + length summary, so the
            // chip list can scroll in whatever width remains.
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                ui.label(
                    RichText::new(format!("{:.0}%", track.volume * 100.0))
                        .color(AppTheme::accent_cyan()),
                );
                let mut vol = track.volume;
                let resp = ui.add(
                    egui::Slider::new(&mut vol, 0.0..=1.0)
                        .show_value(false),
                );
                if resp.changed() {
                    action = MusicRowAction::SetTrackVolume(vol);
                }
                ui.label(RichText::new("Volume:").color(AppTheme::text_secondary()));

                let music_len = track.duration().as_secs_f64();
                if music_len > 0.0 {
                    let video_len = timeline
                        .tracks
                        .iter()
                        .filter(|t| t.kind == TrackKind::Video)
                        .map(|t| t.duration().as_secs_f64())
                        .fold(0.0f64, f64::max);
                    let mut txt = format!("Music {} · Slides {}", mmss(music_len), mmss(video_len));
                    if music_len > video_len + 0.5 {
                        txt.push_str("  (music is longer)");
                    }
                    ui.add_space(10.0);
                    ui.label(RichText::new(txt).color(AppTheme::text_secondary()));
                }

                // Chips fill the leftover middle span, scrolling when crowded.
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    egui::ScrollArea::horizontal()
                        .id_salt("music_row_chips")
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if track.clips.is_empty() {
                                    ui.label(
                                        RichText::new("No music yet — press “🎵 Add Music”")
                                            .italics()
                                            .color(AppTheme::text_secondary()),
                                    );
                                }
                                for clip in &track.clips {
                                    egui::Frame::group(ui.style())
                                        .fill(AppTheme::bg_panel())
                                        .rounding(6.0)
                                        .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                                        .show(ui, |ui| {
                                            let name: String = if clip.name.chars().count() > 24 {
                                                format!(
                                                    "{}…",
                                                    clip.name.chars().take(23).collect::<String>()
                                                )
                                            } else {
                                                clip.name.clone()
                                            };
                                            ui.label(
                                                RichText::new(format!("🎵 {}", name)).strong(),
                                            );
                                            ui.label(
                                                RichText::new(mmss(clip.duration().as_secs_f64()))
                                                    .color(AppTheme::text_secondary()),
                                            );
                                            if ui
                                                .add(
                                                    Button::new("🗑")
                                                        .min_size(egui::vec2(26.0, 26.0)),
                                                )
                                                .on_hover_text("Remove this song")
                                                .clicked()
                                            {
                                                action = MusicRowAction::RemoveClip(clip.id);
                                            }
                                        });
                                }
                            });
                        });
                });
            });
        });
        action
    }
}
