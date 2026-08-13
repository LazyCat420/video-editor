use crate::core::project::{MediaAsset, Project};
use crate::ui::theme::AppTheme;
use egui::{Button, Color32, Frame, RichText, Rounding, ScrollArea, Ui};
use std::path::PathBuf;

pub struct MediaBinView;

pub enum MediaBinAction {
    None,
    ImportFiles(Vec<PathBuf>),
    AddAssetToTimeline(MediaAsset),
}

impl MediaBinView {
    pub fn render(ui: &mut Ui, project: &mut Project) -> MediaBinAction {
        let mut action = MediaBinAction::None;

        ui.vertical(|ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("📁 Your Files")
                        .color(AppTheme::TEXT_PRIMARY)
                        .strong()
                        .size(17.0),
                );
            });

            ui.add_space(4.0);

            // Large Add File Button
            let add_btn = Button::new(
                RichText::new("+ Add Video / Music")
                    .size(14.0)
                    .strong()
                    .color(Color32::WHITE),
            )
            .min_size(egui::vec2(ui.available_width(), 36.0))
            .fill(AppTheme::ACCENT_BLUE);

            if ui.add(add_btn).clicked() {
                if let Some(files) = rfd::FileDialog::new()
                    .add_filter(
                        "Video & Music Files",
                        &["mp4", "mkv", "mov", "avi", "webm", "mp3", "wav", "flac", "aac"],
                    )
                    .pick_files()
                {
                    action = MediaBinAction::ImportFiles(files);
                }
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // Drag-and-drop file detection
            let dropped_files = ui.input(|i| i.raw.dropped_files.clone());
            if !dropped_files.is_empty() {
                let paths: Vec<PathBuf> = dropped_files
                    .into_iter()
                    .filter_map(|f| f.path)
                    .collect();
                if !paths.is_empty() {
                    action = MediaBinAction::ImportFiles(paths);
                }
            }

            if project.media_assets.is_empty() {
                Frame::none()
                    .fill(AppTheme::BG_CARD)
                    .rounding(Rounding::same(8.0))
                    .inner_margin(16.0)
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("📂 No videos loaded yet")
                                    .size(15.0)
                                    .color(AppTheme::TEXT_SECONDARY),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new("Click '+ Add Video / Music' above to choose a video from your computer.")
                                    .size(13.0)
                                    .color(AppTheme::TEXT_MUTED),
                            );
                        });
                    });
            } else {
                ScrollArea::vertical().show(ui, |ui| {
                    for asset in &project.media_assets {
                        Frame::none()
                            .fill(AppTheme::BG_CARD)
                            .stroke(egui::Stroke::new(1.5, AppTheme::BG_HOVER))
                            .rounding(Rounding::same(8.0))
                            .inner_margin(10.0)
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        let icon = if asset.has_video { "🎬" } else { "🎵" };
                                        ui.label(RichText::new(icon).size(22.0));

                                        ui.vertical(|ui| {
                                            ui.label(
                                                RichText::new(&asset.name)
                                                    .strong()
                                                    .color(AppTheme::TEXT_PRIMARY)
                                                    .size(14.0),
                                            );

                                            let dur_m = (asset.duration_secs / 60.0).floor() as u64;
                                            let dur_s = (asset.duration_secs % 60.0).floor() as u64;
                                            let dur_text = if dur_m > 0 {
                                                format!("Duration: {}m {}s", dur_m, dur_s)
                                            } else {
                                                format!("Duration: {} seconds", dur_s)
                                            };
                                            ui.label(
                                                RichText::new(dur_text)
                                                    .size(12.0)
                                                    .color(AppTheme::TEXT_MUTED),
                                            );
                                        });
                                    });

                                    ui.add_space(6.0);

                                    let add_to_timeline_btn = Button::new(
                                        RichText::new("▶ Put on Timeline")
                                            .size(13.0)
                                            .strong()
                                            .color(Color32::WHITE),
                                    )
                                    .min_size(egui::vec2(ui.available_width(), 30.0))
                                    .fill(AppTheme::ACCENT_BLUE);

                                    if ui.add(add_to_timeline_btn).clicked() {
                                        action = MediaBinAction::AddAssetToTimeline(asset.clone());
                                    }
                                });
                            });
                        ui.add_space(6.0);
                    }
                });
            }
        });

        action
    }
}
