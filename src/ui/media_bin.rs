use crate::core::project::{MediaAsset, Project};
use crate::media::frame_cache::FrameCache;
use crate::ui::theme::AppTheme;
use crate::ui::MediaAssetDrag;
use egui::{Button, Color32, ColorImage, Frame, Id, RichText, Rounding, ScrollArea, TextureHandle, TextureOptions, Ui};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub struct MediaBinView;

pub enum MediaBinAction {
    None,
    ImportFiles(Vec<PathBuf>),
    ImportFolder(PathBuf),
    AddAssetToTimeline(MediaAsset),
}

impl MediaBinView {
    pub fn render(
        ui: &mut Ui,
        project: &mut Project,
        collapsed: &mut HashSet<String>,
        frame_cache: &FrameCache,
        thumbnail_frames: &mut HashMap<u64, ColorImage>,
        thumbs: &mut HashMap<u64, TextureHandle>,
    ) -> MediaBinAction {
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

            // Large Add Button: whole folder
            let folder_btn = Button::new(
                RichText::new("📁 Add Entire Folder")
                    .size(14.0)
                    .strong()
                    .color(Color32::WHITE),
            )
            .min_size(egui::vec2(ui.available_width(), 36.0))
            .fill(AppTheme::ACCENT_GREEN);

            if ui
                .add(folder_btn)
                .on_hover_text(
                    "Bring in all the videos & music that are inside a folder on your computer",
                )
                .clicked()
            {
                if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                    action = MediaBinAction::ImportFolder(dir);
                }
            }

            ui.add_space(4.0);

            // Large Add Button: single file(s)
            let add_btn = Button::new(
                RichText::new("+ Add Video / Music")
                    .size(14.0)
                    .strong()
                    .color(Color32::WHITE),
            )
            .min_size(egui::vec2(ui.available_width(), 36.0))
            .fill(AppTheme::ACCENT_BLUE);

            if ui.add(add_btn).clicked() {
                if let Some(files) = crate::media::probe::create_media_file_dialog().pick_files() {
                    action = MediaBinAction::ImportFiles(files);
                }
            }

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // Drag-and-drop file detection (drag files onto this panel from your computer)
            let dropped_files = ui.input(|i| i.raw.dropped_files.clone());
            if !dropped_files.is_empty() {
                let paths: Vec<PathBuf> = dropped_files.into_iter().filter_map(|f| f.path).collect();
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
                                RichText::new("Click '+ Add Entire Folder' to bring in a whole folder of videos at once, or '+ Add Video / Music' for a single file.")
                                    .size(13.0)
                                    .color(AppTheme::TEXT_MUTED),
                            );
                        });
                    });
            } else {
                // Group media by the folder they came from.
                let mut folders: Vec<(String, Vec<&MediaAsset>)> = Vec::new();
                let mut loose: Vec<&MediaAsset> = Vec::new();
                for asset in &project.media_assets {
                    if let Some(fname) = asset
                        .path
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                    {
                        match folders.iter_mut().find(|(f, _)| f == fname) {
                            Some((_, list)) => list.push(asset),
                            None => folders.push((fname.to_string(), vec![asset])),
                        }
                    } else {
                        loose.push(asset);
                    }
                }

                ScrollArea::vertical().show(ui, |ui| {
                    let mut render_asset = |ui: &mut Ui, asset: &MediaAsset| {
                        ui.dnd_drag_source(
                            Id::new(("media_asset_drag", asset.id)),
                            MediaAssetDrag(asset.id),
                            |ui| {
                                Frame::none()
                                    .fill(AppTheme::BG_CARD)
                                    .stroke(egui::Stroke::new(1.5, AppTheme::BG_HOVER))
                                    .rounding(Rounding::same(8.0))
                                    .inner_margin(10.0)
                                    .show(ui, |ui| {
                                            ui.vertical(|ui| {
                                                // Build a real preview picture for videos.
                                                let tex = if asset.has_video {
                                                    // Prefer the small cached thumbnail; if a
                                                    // project was loaded (no import) or the frame
                                                    // cache was evicted, lazily re-extract one.
                                                    if !thumbnail_frames.contains_key(&asset.id) {
                                                        if let Some(img) =
                                                            frame_cache.get_cached(&asset.path, 0.0)
                                                        {
                                                            thumbnail_frames.insert(
                                                                asset.id,
                                                                crate::media::thumbnail::downscale(
                                                                    &img, 192, 108,
                                                                ),
                                                            );
                                                        } else {
                                                            frame_cache.fetch_frame(
                                                                &asset.path,
                                                                0.0,
                                                                Some(ui.ctx()),
                                                            );
                                                        }
                                                    }

                                                    thumbnail_frames.get(&asset.id).and_then(
                                                        |img| {
                                                            if let Some(t) =
                                                                thumbs.get(&asset.id)
                                                            {
                                                                return Some(t.clone());
                                                            }
                                                            let t = ui.ctx().load_texture(
                                                                format!(
                                                                    "asset_thumb_{}",
                                                                    asset.id
                                                                ),
                                                                img.clone(),
                                                                TextureOptions::LINEAR,
                                                            );
                                                            thumbs.insert(asset.id, t.clone());
                                                            Some(t)
                                                        },
                                                    )
                                                } else {
                                                    None
                                                };

                                                ui.horizontal(|ui| {
                                                    if let Some(t) = &tex {
                                                        ui.add(
                                                            egui::Image::from_texture(t)
                                                                .fit_to_exact_size(egui::vec2(72.0, 40.0)),
                                                        );
                                                    } else {
                                                        let icon = if asset.has_video {
                                                            "🎬"
                                                        } else {
                                                            "🎵"
                                                        };
                                                        ui.label(
                                                            RichText::new(icon).size(22.0),
                                                        );
                                                    }

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
                            },
                        );

                        ui.add_space(6.0);
                    };

                    // Render folder groups
                    for (folder_name, assets) in &folders {
                        let is_collapsed = collapsed.contains(folder_name);
                        let chevron = if is_collapsed { "▶" } else { "▼" };
                        let header_btn = Button::new(
                            RichText::new(format!("{} 📁 {}", chevron, folder_name))
                                .size(13.0)
                                .strong()
                                .color(AppTheme::TEXT_PRIMARY),
                        )
                        .fill(AppTheme::BG_HOVER)
                        .min_size(egui::vec2(ui.available_width(), 26.0));

                        if ui.add(header_btn).clicked() {
                            if is_collapsed {
                                collapsed.remove(folder_name);
                            } else {
                                collapsed.insert(folder_name.clone());
                            }
                        }

                        if !is_collapsed {
                            ui.add_space(4.0);
                            ui.indent(folder_name, |ui| {
                                for asset in assets {
                                    render_asset(ui, asset);
                                }
                            });
                        }
                        ui.add_space(6.0);
                    }

                    // Loose files with no folder
                    for asset in &loose {
                        render_asset(ui, asset);
                    }
                });
            }
        });

        action
    }
}
