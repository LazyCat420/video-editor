use crate::core::project::{MediaAsset, Project};
use crate::core::time::TimeCode;
use crate::ui::theme::AppTheme;
use egui::{Button, Frame, RichText, Rounding, ScrollArea, Ui};
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
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Media Bin").color(AppTheme::TEXT_PRIMARY).size(16.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(Button::new(RichText::new("+ Import").size(12.0)))
                        .clicked()
                    {
                        if let Some(files) = rfd::FileDialog::new()
                            .add_filter("Media Files", &["mp4", "mkv", "mov", "avi", "webm", "mp3", "wav", "flac", "aac"])
                            .pick_files()
                        {
                            action = MediaBinAction::ImportFiles(files);
                        }
                    }
                });
            });

            ui.separator();

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
                    .inner_margin(16.0)
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("📂 No media imported yet").color(AppTheme::TEXT_SECONDARY));
                            ui.label(
                                RichText::new("Drag & drop video/audio files here or click '+ Import'")
                                    .size(11.0)
                                    .color(AppTheme::TEXT_MUTED),
                            );
                        });
                    });
            } else {
                ScrollArea::vertical().show(ui, |ui| {
                    for asset in &project.media_assets {
                        Frame::none()
                            .fill(AppTheme::BG_CARD)
                            .stroke(egui::Stroke::new(1.0, AppTheme::BG_HOVER))
                            .rounding(Rounding::same(6.0))
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Media Icon badge
                                    let icon = if asset.has_video { "🎬" } else { "🎵" };
                                    ui.label(RichText::new(icon).size(20.0));

                                    ui.vertical(|ui| {
                                        ui.label(
                                            RichText::new(&asset.name)
                                                .strong()
                                                .color(AppTheme::TEXT_PRIMARY)
                                                .size(13.0),
                                        );

                                        let dur_tc = TimeCode::from_secs_f64(asset.duration_secs);
                                        let meta_info = if asset.has_video {
                                            format!("{}x{} | {} | {:.1} fps", asset.width, asset.height, dur_tc, asset.fps)
                                        } else {
                                            format!("Audio | {}", dur_tc)
                                        };
                                        ui.label(RichText::new(meta_info).size(11.0).color(AppTheme::TEXT_MUTED));
                                    });

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui
                                            .add(Button::new(RichText::new("+ Timeline").size(11.0)).fill(AppTheme::ACCENT_BLUE))
                                            .clicked()
                                        {
                                            action = MediaBinAction::AddAssetToTimeline(asset.clone());
                                        }
                                    });
                                });
                            });
                        ui.add_space(4.0);
                    }
                });
            }
        });

        action
    }
}
