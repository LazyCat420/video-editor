use crate::app::VideoEditorApp;
use crate::core::text_overlay::{SlideBackground, SlideElement};
use egui::Context;
use std::path::PathBuf;

impl VideoEditorApp {
    /// Drop the armed pending element on the slide at a normalized point (0..1).
    pub(crate) fn place_pending_element(&mut self, x: f32, y: f32, ctx: Option<&Context>) {
        let Some(pending) = self.pending_place.take() else {
            return;
        };
        self.snapshot_timeline();
        let slide_id = self.resolve_target_slide_id();

        let element = match pending {
            crate::ui::PendingElement::Text(mut overlay) => {
                overlay.x = x.clamp(0.0, 1.0);
                overlay.y = y.clamp(0.0, 1.0);
                SlideElement::Text(overlay)
            }
            crate::ui::PendingElement::Picture(path) => {
                let _ = self.add_media_to_bin(&path);
                SlideElement::Picture {
                    path,
                    x: (x - 0.20).clamp(0.0, 0.60),
                    y: (y - 0.15).clamp(0.0, 0.70),
                    w: 0.40,
                    h: 0.30,
                }
            }
            crate::ui::PendingElement::Sticker { path, name, category } => {
                SlideElement::Sticker {
                    path,
                    name,
                    category,
                    x: (x - 0.12).clamp(0.0, 0.76),
                    y: (y - 0.12).clamp(0.0, 0.76),
                    w: 0.24,
                    h: 0.24,
                }
            }
            crate::ui::PendingElement::Video(path) => {
                let _ = self.add_media_to_bin(&path);
                SlideElement::Video {
                    path,
                    x: (x - 0.25).clamp(0.0, 0.50),
                    y: (y - 0.15).clamp(0.0, 0.70),
                    w: 0.50,
                    h: 0.30,
                }
            }
        };

        if let Some(clip) = self.project.timeline.get_clip_mut(slide_id) {
            clip.elements.push(element);
            self.selected_slide_element = Some(clip.elements.len() - 1);
        }
        self.auto_adjust_slide_duration_to_media(slide_id);
        self.refresh_preview_frame(ctx);
    }

    pub fn drop_media_asset_on_canvas(&mut self, asset_id: u64, x: f32, y: f32, ctx: Option<&Context>) {
        let asset = self.project.media_assets.iter().find(|a| a.id == asset_id).cloned();
        let Some(asset) = asset else {
            return;
        };
        self.snapshot_timeline();
        let slide_id = self.resolve_target_slide_id();
        if let Some(clip) = self.project.timeline.get_clip_mut(slide_id) {
            let mut replaced_slot = false;

            // 1. Check if dropped over a placeholder slot
            for (idx, el) in clip.elements.iter_mut().enumerate() {
                if let SlideElement::Placeholder { x: px, y: py, w: pw, h: ph, .. } = el {
                    if x >= *px && x <= *px + *pw && y >= *py && y <= *py + *ph {
                        *el = if asset.has_video {
                            SlideElement::Video {
                                path: asset.path.clone(),
                                x: *px,
                                y: *py,
                                w: *pw,
                                h: *ph,
                            }
                        } else {
                            SlideElement::Picture {
                                path: asset.path.clone(),
                                x: *px,
                                y: *py,
                                w: *pw,
                                h: *ph,
                            }
                        };
                        self.selected_slide_element = Some(idx);
                        replaced_slot = true;
                        break;
                    }
                }
            }

            // 2. Otherwise fill first unfilled placeholder slot
            if !replaced_slot {
                for (idx, el) in clip.elements.iter_mut().enumerate() {
                    if let SlideElement::Placeholder { x: px, y: py, w: pw, h: ph, .. } = el {
                        *el = if asset.has_video {
                            SlideElement::Video {
                                path: asset.path.clone(),
                                x: *px,
                                y: *py,
                                w: *pw,
                                h: *ph,
                            }
                        } else {
                            SlideElement::Picture {
                                path: asset.path.clone(),
                                x: *px,
                                y: *py,
                                w: *pw,
                                h: *ph,
                            }
                        };
                        self.selected_slide_element = Some(idx);
                        replaced_slot = true;
                        break;
                    }
                }
            }

            // 3. Otherwise add as interactive resizable card
            if !replaced_slot {
                let is_empty_slide = clip.elements.is_empty();
                let element = if is_empty_slide {
                    if asset.has_video {
                        SlideElement::Video {
                            path: asset.path,
                            x: 0.10,
                            y: 0.10,
                            w: 0.80,
                            h: 0.80,
                        }
                    } else {
                        SlideElement::Picture {
                            path: asset.path,
                            x: 0.10,
                            y: 0.10,
                            w: 0.80,
                            h: 0.80,
                        }
                    }
                } else if asset.has_video {
                    SlideElement::Video {
                        path: asset.path,
                        x: (x - 0.22).clamp(0.0, 0.55),
                        y: (y - 0.22).clamp(0.0, 0.55),
                        w: 0.45,
                        h: 0.45,
                    }
                } else {
                    SlideElement::Picture {
                        path: asset.path,
                        x: (x - 0.22).clamp(0.0, 0.55),
                        y: (y - 0.22).clamp(0.0, 0.55),
                        w: 0.45,
                        h: 0.45,
                    }
                };
                clip.elements.push(element);
                self.selected_slide_element = Some(clip.elements.len() - 1);
            }
        }
        self.auto_adjust_slide_duration_to_media(slide_id);
        self.refresh_preview_frame(ctx);
    }

    pub fn drop_files_on_canvas(&mut self, paths: Vec<PathBuf>, x: f32, y: f32, ctx: Option<&Context>) {
        if paths.is_empty() {
            return;
        }
        self.snapshot_timeline();
        let total_files_count = paths.len();
        let slide_id = self.resolve_target_slide_id();
        let mut first_new_idx = None;
        for (i, p) in paths.into_iter().enumerate() {
            let is_audio = crate::media::probe::is_audio_path(&p);
            let asset_id = self.add_media_to_bin(&p);
            let has_video = if is_audio {
                false
            } else {
                asset_id
                    .and_then(|id| self.project.media_assets.iter().find(|a| a.id == id))
                    .map(|a| a.has_video)
                    .unwrap_or_else(|| {
                        crate::media::probe::probe_media_file(&p).map(|inf| inf.has_video).unwrap_or(false)
                    })
            };

            if let Some(clip) = self.project.timeline.get_clip_mut(slide_id) {
                clip.is_selected = true;

                if is_audio {
                    clip.elements.push(SlideElement::Audio { path: p, volume: 1.0 });
                    if first_new_idx.is_none() {
                        first_new_idx = Some(clip.elements.len() - 1);
                    }
                } else {
                    let mut replaced_slot = false;

                    // A. Check if dropped directly over a specific placeholder slot
                    for (idx, el) in clip.elements.iter_mut().enumerate() {
                        if let SlideElement::Placeholder { x: px, y: py, w: pw, h: ph, .. } = el {
                            if x >= *px && x <= *px + *pw && y >= *py && y <= *py + *ph {
                                *el = if has_video {
                                    SlideElement::Video {
                                        path: p.clone(),
                                        x: *px,
                                        y: *py,
                                        w: *pw,
                                        h: *ph,
                                    }
                                } else {
                                    SlideElement::Picture {
                                        path: p.clone(),
                                        x: *px,
                                        y: *py,
                                        w: *pw,
                                        h: *ph,
                                    }
                                };
                                if first_new_idx.is_none() {
                                    first_new_idx = Some(idx);
                                }
                                replaced_slot = true;
                                break;
                            }
                        }
                    }

                    // B. If not dropped over a specific slot, fill the first unfilled placeholder slot if one exists
                    if !replaced_slot {
                        for (idx, el) in clip.elements.iter_mut().enumerate() {
                            if let SlideElement::Placeholder { x: px, y: py, w: pw, h: ph, .. } = el {
                                *el = if has_video {
                                    SlideElement::Video {
                                        path: p.clone(),
                                        x: *px,
                                        y: *py,
                                        w: *pw,
                                        h: *ph,
                                    }
                                } else {
                                    SlideElement::Picture {
                                        path: p.clone(),
                                        x: *px,
                                        y: *py,
                                        w: *pw,
                                        h: *ph,
                                    }
                                };
                                if first_new_idx.is_none() {
                                    first_new_idx = Some(idx);
                                }
                                replaced_slot = true;
                                break;
                            }
                        }
                    }

                    // C. Otherwise place as a centered resizable media card
                    if !replaced_slot {
                        let was_initially_empty = clip.elements.is_empty();
                        let total_dropped = total_files_count;
                        let (el_x, el_y, el_w, el_h) = if was_initially_empty && total_dropped == 1 {
                            (0.10, 0.10, 0.80, 0.80)
                        } else if was_initially_empty && total_dropped == 2 {
                            if i == 0 {
                                (0.04, 0.15, 0.44, 0.70)
                            } else {
                                (0.52, 0.15, 0.44, 0.70)
                            }
                        } else if was_initially_empty && total_dropped == 3 {
                            if i == 0 {
                                (0.04, 0.15, 0.44, 0.70)
                            } else if i == 1 {
                                (0.52, 0.12, 0.44, 0.36)
                            } else {
                                (0.52, 0.52, 0.44, 0.36)
                            }
                        } else if was_initially_empty && total_dropped == 4 {
                            let col = (i % 2) as f32;
                            let row = (i / 2) as f32;
                            (0.04 + col * 0.48, 0.10 + row * 0.42, 0.44, 0.38)
                        } else {
                            let offset_x = ((i % 3) as f32) * 0.05;
                            let offset_y = ((i / 3) as f32) * 0.05;
                            (
                                (x - 0.22 + offset_x).clamp(0.0, 0.55),
                                (y - 0.22 + offset_y).clamp(0.0, 0.55),
                                0.45,
                                0.45,
                            )
                        };

                        let element = if has_video {
                            SlideElement::Video {
                                path: p,
                                x: el_x,
                                y: el_y,
                                w: el_w,
                                h: el_h,
                            }
                        } else {
                            SlideElement::Picture {
                                path: p,
                                x: el_x,
                                y: el_y,
                                w: el_w,
                                h: el_h,
                            }
                        };
                        clip.elements.push(element);
                        if first_new_idx.is_none() {
                            first_new_idx = Some(clip.elements.len() - 1);
                        }
                    }
                }
            }
        }
        if let Some(idx) = first_new_idx {
            self.selected_slide_element = Some(idx);
        }
        self.auto_adjust_slide_duration_to_media(slide_id);
        self.refresh_preview_frame(ctx);
    }

    pub(crate) fn move_slide_element(&mut self, idx: usize, x: f32, y: f32) {
        if let Some(id) = self.active_slide().map(|c| c.id) {
            if let Some(clip) = self.project.timeline.get_clip_mut(id) {
                if let Some(el) = clip.elements.get_mut(idx) {
                    let (_, _, w, h) = el.bounds();
                    let max_x = (1.0 - w).max(0.0);
                    let max_y = (1.0 - h).max(0.0);
                    el.set_bounds(x.clamp(0.0, max_x), y.clamp(0.0, max_y), w, h);
                }
            }
        }
    }

    pub(crate) fn resize_slide_element(&mut self, idx: usize, x: f32, y: f32, w: f32, h: f32) {
        if let Some(id) = self.active_slide().map(|c| c.id) {
            if let Some(clip) = self.project.timeline.get_clip_mut(id) {
                if let Some(el) = clip.elements.get_mut(idx) {
                    el.set_bounds(x, y, w, h);
                }
            }
        }
    }

    pub fn full_slide_element(&mut self, idx: usize, ctx: Option<&Context>) {
        if let Some(id) = self.active_slide().map(|c| c.id) {
            self.snapshot_timeline();
            if let Some(clip) = self.project.timeline.get_clip_mut(id) {
                if let Some(el) = clip.elements.get_mut(idx) {
                    match el {
                        SlideElement::Text(o) => {
                            o.x = 0.5;
                            o.y = 0.5;
                        }
                        SlideElement::Calendar(c) => {
                            c.x = 0.05;
                            c.y = 0.05;
                            c.w = 0.90;
                            c.h = 0.90;
                        }
                        SlideElement::Placeholder { x, y, w, h, .. } => {
                            *x = 0.0;
                            *y = 0.0;
                            *w = 1.0;
                            *h = 1.0;
                        }
                        SlideElement::Picture { x, y, w, h, .. } | SlideElement::Video { x, y, w, h, .. } => {
                            if *x == 0.0 && *y == 0.0 && *w == 1.0 && *h == 1.0 {
                                el.set_bounds(0.10, 0.10, 0.80, 0.80);
                            } else {
                                el.set_bounds(0.0, 0.0, 1.0, 1.0);
                            }
                        }
                        _ => {}
                    }
                }
            }
            self.refresh_preview_frame(ctx);
        }
    }

    pub(crate) fn set_element_as_background(&mut self, idx: usize, ctx: Option<&Context>) {
        if let Some(id) = self.active_slide().map(|c| c.id) {
            self.snapshot_timeline();
            if let Some(clip) = self.project.timeline.get_clip_mut(id) {
                if idx < clip.elements.len() {
                    if let SlideElement::Picture { path, .. } = clip.elements.remove(idx) {
                        clip.background = Some(SlideBackground::Picture(path));
                        self.selected_slide_element = None;
                    }
                }
            }
            self.refresh_preview_frame(ctx);
        }
    }

    pub(crate) fn delete_slide_element(&mut self, idx: usize, ctx: Option<&Context>) {
        if let Some(id) = self.active_slide().map(|c| c.id) {
            self.snapshot_timeline();
            if let Some(clip) = self.project.timeline.get_clip_mut(id) {
                if idx < clip.elements.len() {
                    clip.elements.remove(idx);
                    self.selected_slide_element = None;
                }
            }
            self.auto_adjust_slide_duration_to_media(id);
            self.refresh_preview_frame(ctx);
        }
    }

    pub fn scale_text_element(&mut self, idx: usize, font_size: f32) {
        if let Some(id) = self.active_slide().map(|c| c.id) {
            if let Some(clip) = self.project.timeline.get_clip_mut(id) {
                if let Some(SlideElement::Text(o)) = clip.elements.get_mut(idx) {
                    o.font_size = font_size.clamp(10.0, 120.0);
                }
            }
        }
    }
}
