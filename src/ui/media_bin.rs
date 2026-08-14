use crate::core::project::{MediaAsset, Project};
use crate::media::frame_cache::FrameCache;
use crate::ui::theme::AppTheme;
use crate::ui::MediaAssetDrag;
use egui::{Button, Color32, ColorImage, Frame, Rect, RichText, Rounding, ScrollArea, TextureHandle, TextureOptions, Ui};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub struct MediaBinView;

pub enum MediaBinAction {
    None,
    ImportFiles(Vec<PathBuf>),
    ImportFolder(PathBuf),
    AddAssetToTimeline(MediaAsset),
    RemoveAsset(u64),
    ReorderAsset { from_id: u64, to_index: usize },
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
                        .color(AppTheme::text_primary())
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
            .fill(AppTheme::accent_green());

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
            .fill(AppTheme::accent_blue());

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
                    .fill(AppTheme::bg_card())
                    .rounding(Rounding::same(8.0))
                    .inner_margin(16.0)
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("📂 No videos loaded yet")
                                    .size(15.0)
                                    .color(AppTheme::text_secondary()),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new("Click '+ Add Entire Folder' to bring in a whole folder of videos at once, or '+ Add Video / Music' for a single file.")
                                    .size(13.0)
                                    .color(AppTheme::text_muted()),
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
                    let mut render_asset = |ui: &mut Ui, asset: &MediaAsset, index: usize| {
                        ui.vertical(|ui| {
                            // The picture + name strip is the drag handle: grab anywhere on it
                            // to drag this file onto a track. Buttons stay separate so clicking
                            // them never accidentally starts a drag.
                            let tex = if asset.has_video {
                                // Prefer the small cached thumbnail; if a project was loaded (no
                                // import) or the frame cache was evicted, lazily re-extract one.
                                if !thumbnail_frames.contains_key(&asset.id) {
                                    if let Some(img) = frame_cache.get_cached(&asset.path, 0.0) {
                                        thumbnail_frames.insert(
                                            asset.id,
                                            crate::media::thumbnail::downscale(&img, 192, 108),
                                        );
                                    } else {
                                        frame_cache.fetch_frame(&asset.path, 0.0, Some(ui.ctx()));
                                    }
                                }

                                thumbnail_frames.get(&asset.id).and_then(|img| {
                                    if let Some(t) = thumbs.get(&asset.id) {
                                        return Some(t.clone());
                                    }
                                    let t = ui.ctx().load_texture(
                                        format!("asset_thumb_{}", asset.id),
                                        img.clone(),
                                        TextureOptions::LINEAR,
                                    );
                                    thumbs.insert(asset.id, t.clone());
                                    Some(t)
                                })
                            } else {
                                None
                            };

                            // Whole card is the drag source - easy to grab anywhere - while the
                            // + / X buttons are placed ON TOP via ui.put (registered after the
                            // drag), so they always win clicks.
                            let card_size = egui::vec2(ui.available_width(), 72.0);
                            let (card_rect, card_resp) =
                                ui.allocate_exact_size(card_size, egui::Sense::click_and_drag());
                            card_resp.dnd_set_drag_payload(MediaAssetDrag(asset.id));
                            if card_resp.dragged() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                            } else if card_resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                            }

                            // Reorder-within-list: if this card is the drop target of another
                            // asset still being dragged, move that asset to this card's position.
                            if let Some(released) =
                                card_resp.dnd_release_payload::<MediaAssetDrag>()
                            {
                                if released.0 != asset.id {
                                    action = MediaBinAction::ReorderAsset {
                                        from_id: released.0,
                                        to_index: index,
                                    };
                                }
                            }

                            let cp = ui.painter_at(card_rect);
                            cp.rect_filled(card_rect, Rounding::same(8.0), AppTheme::bg_card());
                            cp.rect_stroke(
                                card_rect,
                                Rounding::same(8.0),
                                egui::Stroke::new(1.5, AppTheme::bg_hover()),
                            );

                            // Thumbnail / icon on the left.
                            let pad = 8.0;
                            let thumb = Rect::from_min_size(
                                egui::pos2(card_rect.min.x + pad, card_rect.center().y - 20.0),
                                egui::vec2(74.0, 40.0),
                            );
                            if let Some(t) = &tex {
                                cp.image(
                                    t.id(),
                                    thumb,
                                    Rect::from_min_max(
                                        egui::pos2(0.0, 0.0),
                                        egui::pos2(1.0, 1.0),
                                    ),
                                    Color32::WHITE,
                                );
                            } else {
                                let icon = if asset.has_video { "🎬" } else { "🎵" };
                                cp.text(
                                    thumb.center(),
                                    egui::Align2::CENTER_CENTER,
                                    icon,
                                    egui::FontId::proportional(22.0),
                                    AppTheme::text_secondary(),
                                );
                            }
                            cp.rect_stroke(
                                thumb,
                                Rounding::same(4.0),
                                egui::Stroke::new(1.0, AppTheme::bg_hover()),
                            );

                            // Name + duration next to the thumbnail, clipped so long names can
                            // never run under the + / X buttons (which caused the overlap).
                            let text_x = thumb.max.x + 8.0;
                            let btn_col_left = card_rect.max.x - pad - 20.0 - 6.0;
                            let text_clip = Rect::from_min_max(
                                egui::pos2(text_x - 2.0, card_rect.min.y),
                                egui::pos2(btn_col_left.max(text_x), card_rect.max.y),
                            );
                            let tc = cp.with_clip_rect(text_clip);
                            tc.text(
                                egui::pos2(text_x, card_rect.min.y + 16.0),
                                egui::Align2::LEFT_TOP,
                                &asset.name,
                                egui::FontId::proportional(14.0),
                                AppTheme::text_primary(),
                            );
                            let dur_m = (asset.duration_secs / 60.0).floor() as u64;
                            let dur_s = (asset.duration_secs % 60.0).floor() as u64;
                            let dur_text = if dur_m > 0 {
                                format!("Duration: {}m {}s", dur_m, dur_s)
                            } else {
                                format!("Duration: {} seconds", dur_s)
                            };
                            tc.text(
                                egui::pos2(text_x, card_rect.min.y + 38.0),
                                egui::Align2::LEFT_TOP,
                                dur_text,
                                egui::FontId::proportional(12.0),
                                AppTheme::text_muted(),
                            );

                            // Buttons placed on top (top-right): X to remove, + to put on
                            // timeline. ui.put registers them after the drag source, so they
                            // receive the click instead of the drag.
                            let btn = egui::vec2(20.0, 18.0);
                            let btns_right = card_rect.max.x - pad;
                            let x_rect = Rect::from_min_size(
                                egui::pos2(btns_right - btn.x, card_rect.min.y + 6.0),
                                btn,
                            );
                            let plus_rect = Rect::from_min_size(
                                egui::pos2(btns_right - btn.x, card_rect.min.y + 6.0 + btn.y + 2.0),
                                btn,
                            );

                            if ui
                                .put(x_rect, Button::new(RichText::new("X").size(11.0)))
                                .on_hover_text("Remove from list")
                                .clicked()
                            {
                                action = MediaBinAction::RemoveAsset(asset.id);
                            }

                            let put_btn = Button::new(
                                RichText::new("+").size(13.0).strong().color(Color32::WHITE),
                            )
                            .fill(AppTheme::accent_blue());

                            if ui
                                .put(plus_rect, put_btn)
                                .on_hover_text("Put on Timeline")
                                .clicked()
                            {
                                action = MediaBinAction::AddAssetToTimeline(asset.clone());
                            }

                            ui.add_space(6.0);
                        });

                        ui.add_space(4.0);
                    };

                    // Render folder groups
                    for (folder_name, assets) in &folders {
                        let is_collapsed = collapsed.contains(folder_name);
                        let chevron = if is_collapsed { "▶" } else { "▼" };
                        let header_btn = Button::new(
                            RichText::new(format!("{} 📁 {}", chevron, folder_name))
                                .size(13.0)
                                .strong()
                                .color(AppTheme::text_primary()),
                        )
                        .fill(AppTheme::bg_hover())
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
                                    let idx = project
                                        .media_assets
                                        .iter()
                                        .position(|a| a.id == asset.id)
                                        .unwrap_or(0);
                                    render_asset(ui, asset, idx);
                                }
                            });
                        }
                        ui.add_space(6.0);
                    }

                    // Loose files with no folder
                    for asset in &loose {
                        let idx = project
                            .media_assets
                            .iter()
                            .position(|a| a.id == asset.id)
                            .unwrap_or(0);
                        render_asset(ui, asset, idx);
                    }

                    // End drop-zone so a dragged file can be moved to the very last position.
                    if !project.media_assets.is_empty() {
                        let last = project.media_assets.len() - 1;
                        let zone = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), 12.0),
                            egui::Sense::hover(),
                        );
                        if let Some(released) = zone.1.dnd_release_payload::<MediaAssetDrag>() {
                            action = MediaBinAction::ReorderAsset {
                                from_id: released.0,
                                to_index: last,
                            };
                        }
                    }
                });
            }
        });

        action
    }
}
