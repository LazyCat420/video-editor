use crate::app::VideoEditorApp;
use crate::core::clip::Clip;
use crate::core::envelope::VolumeEnvelope;
use crate::core::text_overlay::{FontFamilyPreset, SlideElement, TextAlignment, TextBoxStyle, TextOverlay};
use crate::core::time::TimeCode;
use crate::core::track::TrackKind;
use crate::ui::theme::AppTheme;
use egui::{Button, Color32, Context, RichText, Ui, Vec2};

impl VideoEditorApp {
    /// The active slide (prioritizes selected clip on a video track, then clip under playhead).
    pub fn active_slide(&self) -> Option<&Clip> {
        for track in &self.project.timeline.tracks {
            if track.kind == TrackKind::Video {
                if let Some(c) = track.clips.iter().find(|c| c.is_selected) {
                    return Some(c);
                }
            }
        }
        let playhead = self.project.timeline.playhead;
        for track in &self.project.timeline.tracks {
            if track.kind == TrackKind::Video {
                if let Some(c) = track.get_clip_at(playhead) {
                    return Some(c);
                }
            }
        }
        for track in &self.project.timeline.tracks {
            if track.kind == TrackKind::Video {
                if let Some(c) = track.clips.first() {
                    return Some(c);
                }
            }
        }
        None
    }

    /// Resolves the target slide ID to receive drops/elements. If none exists, creates a fresh Blank Slide.
    pub fn resolve_target_slide_id(&mut self) -> u64 {
        if let Some(active) = self.active_slide() {
            return active.id;
        }
        let track_id = self
            .project
            .timeline
            .tracks
            .iter()
            .find(|t| t.kind == TrackKind::Video)
            .map(|t| t.id)
            .unwrap_or_else(|| {
                self.project
                    .timeline
                    .add_track("Video Track".to_string(), TrackKind::Video)
            });
        let next_id = self.project.timeline.next_id();
        let mut clip = Clip::new_blank_slide(next_id, track_id, "Slide 1".to_string(), 5.0);
        clip.timeline_start = self.project.timeline.playhead;
        clip.is_selected = true;
        if let Some(track) = self.project.timeline.get_track_mut(track_id) {
            track.add_clip(clip);
        }
        next_id
    }

    pub fn insert_blank_slide_at_playhead(&mut self, duration: f64, ctx: Option<&Context>) {
        self.snapshot_timeline();
        let track_id = self
            .project
            .timeline
            .tracks
            .iter()
            .find(|t| t.kind == TrackKind::Video)
            .map(|t| t.id)
            .unwrap_or_else(|| {
                self.project
                    .timeline
                    .add_track("Video Track".to_string(), TrackKind::Video)
            });
        let next_id = self.project.timeline.next_id();
        let mut clip = Clip::new_blank_slide(next_id, track_id, "Blank Slide".to_string(), duration);
        clip.timeline_start = self.project.timeline.playhead;
        clip.is_selected = true;
        // Deselect other clips so the newly created blank slide is the active slide
        for t in &mut self.project.timeline.tracks {
            for c in &mut t.clips {
                if c.id != next_id {
                    c.is_selected = false;
                }
            }
        }
        if let Some(track) = self.project.timeline.get_track_mut(track_id) {
            track.add_clip(clip);
        }
        self.sidebar_tab = crate::ui::SidebarTab::Formatting;
        self.selected_slide_element = None;
        self.refresh_preview_frame(ctx);
    }

    pub(crate) fn auto_adjust_slide_duration_to_media(&mut self, slide_id: u64) {
        let mut max_media_dur: f64 = 0.0;
        let mut has_media = false;

        let (track_id, old_dur, slide_start) = if let Some(clip) = self.project.timeline.get_clip(slide_id) {
            for el in &clip.elements {
                match el {
                    SlideElement::Video { path, .. } | SlideElement::Audio { path, .. } => {
                        has_media = true;
                        let dur = self.project.media_assets.iter()
                            .find(|a| &a.path == path)
                            .map(|a| a.duration_secs)
                            .or_else(|| crate::media::probe::probe_media_file(path).ok().map(|m| m.duration_secs))
                            .unwrap_or(0.0);
                        if dur > max_media_dur {
                            max_media_dur = dur;
                        }
                    }
                    _ => {}
                }
            }
            (clip.track_id, clip.duration(), clip.timeline_start)
        } else {
            return;
        };

        if has_media && max_media_dur > 0.0 {
            let target_dur = TimeCode::from_secs_f64(max_media_dur.max(0.5));
            if target_dur != old_dur {
                if let Some(clip) = self.project.timeline.get_clip_mut(slide_id) {
                    clip.source_duration = target_dur;
                    clip.source_out = target_dur;
                    clip.volume_envelope = VolumeEnvelope::default_for_duration(target_dur);
                }

                if target_dur > old_dur {
                    let delta = target_dur - old_dur;
                    let old_end = slide_start + old_dur;
                    if let Some(track) = self.project.timeline.get_track_mut(track_id) {
                        for c in &mut track.clips {
                            if c.id != slide_id && c.timeline_start >= old_end {
                                c.timeline_start = c.timeline_start + delta;
                            }
                        }
                        track.sort_clips();
                    }
                }
            }
        }
    }

    pub fn insert_template_title_2_media(&mut self, ctx: Option<&Context>) {
        self.snapshot_timeline();
        let track_id = self.project.timeline.tracks.iter().find(|t| t.kind == TrackKind::Video).map(|t| t.id)
            .unwrap_or_else(|| self.project.timeline.add_track("Video Track".to_string(), TrackKind::Video));
        let next_id = self.project.timeline.next_id();
        let mut slide = Clip::new_blank_slide(next_id, track_id, "Title + 2 Media".to_string(), 5.0);
        slide.timeline_start = self.project.timeline.playhead;
        slide.is_selected = true;

        let mut title = TextOverlay::default();
        title.text = "Add Slide Title Here".to_string();
        title.font_size = 32.0;
        title.x = 0.5;
        title.y = 0.12;
        title.alignment = TextAlignment::Center;
        title.font_family = FontFamilyPreset::Impact;
        title.box_style = TextBoxStyle::None;
        slide.elements.push(SlideElement::Text(title));

        slide.elements.push(SlideElement::Placeholder {
            slot_id: 1,
            label: "Left Photo/Video".to_string(),
            x: 0.07,
            y: 0.25,
            w: 0.41,
            h: 0.65,
        });

        slide.elements.push(SlideElement::Placeholder {
            slot_id: 2,
            label: "Right Photo/Video".to_string(),
            x: 0.52,
            y: 0.25,
            w: 0.41,
            h: 0.65,
        });

        for t in &mut self.project.timeline.tracks {
            for c in &mut t.clips {
                c.is_selected = false;
            }
        }
        if let Some(track) = self.project.timeline.get_track_mut(track_id) {
            track.add_clip(slide);
        }
        self.refresh_preview_frame(ctx);
    }

    pub fn insert_template_title_4_media(&mut self, ctx: Option<&Context>) {
        self.snapshot_timeline();
        let track_id = self.project.timeline.tracks.iter().find(|t| t.kind == TrackKind::Video).map(|t| t.id)
            .unwrap_or_else(|| self.project.timeline.add_track("Video Track".to_string(), TrackKind::Video));
        let next_id = self.project.timeline.next_id();
        let mut slide = Clip::new_blank_slide(next_id, track_id, "Title + 4 Grid".to_string(), 5.0);
        slide.timeline_start = self.project.timeline.playhead;
        slide.is_selected = true;

        let mut title = TextOverlay::default();
        title.text = "Grid Photo Gallery".to_string();
        title.font_size = 28.0;
        title.x = 0.5;
        title.y = 0.08;
        title.alignment = TextAlignment::Center;
        title.font_family = FontFamilyPreset::Impact;
        slide.elements.push(SlideElement::Text(title));

        let coords = [
            (0.04, 0.18, 0.44, 0.36),
            (0.52, 0.18, 0.44, 0.36),
            (0.04, 0.58, 0.44, 0.36),
            (0.52, 0.58, 0.44, 0.36),
        ];
        for (i, (x, y, w, h)) in coords.iter().enumerate() {
            slide.elements.push(SlideElement::Placeholder {
                slot_id: (i + 1) as u32,
                label: format!("Slot {}", i + 1),
                x: *x,
                y: *y,
                w: *w,
                h: *h,
            });
        }

        for t in &mut self.project.timeline.tracks {
            for c in &mut t.clips {
                c.is_selected = false;
            }
        }
        if let Some(track) = self.project.timeline.get_track_mut(track_id) {
            track.add_clip(slide);
        }
        self.refresh_preview_frame(ctx);
    }

    pub fn insert_template_showcase(&mut self, ctx: Option<&Context>) {
        self.snapshot_timeline();
        let track_id = self.project.timeline.tracks.iter().find(|t| t.kind == TrackKind::Video).map(|t| t.id)
            .unwrap_or_else(|| self.project.timeline.add_track("Video Track".to_string(), TrackKind::Video));
        let next_id = self.project.timeline.next_id();
        let mut slide = Clip::new_blank_slide(next_id, track_id, "Feature Showcase".to_string(), 5.0);
        slide.timeline_start = self.project.timeline.playhead;
        slide.is_selected = true;

        slide.elements.push(SlideElement::Placeholder {
            slot_id: 1,
            label: "Main Showcase Media".to_string(),
            x: 0.05,
            y: 0.08,
            w: 0.55,
            h: 0.84,
        });

        let mut title = TextOverlay::default();
        title.text = "Feature Headline".to_string();
        title.font_size = 32.0;
        title.x = 0.80;
        title.y = 0.30;
        title.alignment = TextAlignment::Left;
        title.font_family = FontFamilyPreset::Impact;
        slide.elements.push(SlideElement::Text(title));

        let mut body = TextOverlay::default();
        body.text = "Write your story or description here.
Great for highlights and memories.".to_string();
        body.font_size = 18.0;
        body.x = 0.80;
        body.y = 0.55;
        body.alignment = TextAlignment::Left;
        body.font_family = FontFamilyPreset::SansSerif;
        slide.elements.push(SlideElement::Text(body));

        for t in &mut self.project.timeline.tracks {
            for c in &mut t.clips {
                c.is_selected = false;
            }
        }
        if let Some(track) = self.project.timeline.get_track_mut(track_id) {
            track.add_clip(slide);
        }
        self.refresh_preview_frame(ctx);
    }

    pub fn reflow_slide_timeline_positions(&mut self) {
        let mut cur_time = TimeCode::ZERO;
        for track in &mut self.project.timeline.tracks {
            if track.kind == TrackKind::Video {
                for clip in &mut track.clips {
                    clip.timeline_start = cur_time;
                    cur_time = cur_time + clip.duration();
                }
            }
        }
    }

    pub fn reorder_slide(&mut self, from_idx: usize, to_idx: usize, ctx: Option<&Context>) {
        self.snapshot_timeline();
        for track in &mut self.project.timeline.tracks {
            if track.kind == TrackKind::Video {
                if from_idx < track.clips.len() && to_idx < track.clips.len() {
                    let c = track.clips.remove(from_idx);
                    track.clips.insert(to_idx, c);
                }
            }
        }
        self.reflow_slide_timeline_positions();
        self.refresh_preview_frame(ctx);
    }

    pub fn duplicate_slide(&mut self, clip_id: u64, ctx: Option<&Context>) {
        self.snapshot_timeline();
        let next_id = self.project.timeline.next_id();
        for track in &mut self.project.timeline.tracks {
            if track.kind == TrackKind::Video {
                if let Some(pos) = track.clips.iter().position(|c| c.id == clip_id) {
                    let mut dup = track.clips[pos].clone();
                    dup.id = next_id;
                    dup.name = format!("{} (Copy)", dup.name);
                    dup.is_selected = true;
                    for c in &mut track.clips {
                        c.is_selected = false;
                    }
                    track.clips.insert(pos + 1, dup);
                    break;
                }
            }
        }
        self.reflow_slide_timeline_positions();
        self.refresh_preview_frame(ctx);
    }

    pub fn delete_slide_by_id(&mut self, clip_id: u64, ctx: Option<&Context>) {
        self.snapshot_timeline();
        for track in &mut self.project.timeline.tracks {
            if track.kind == TrackKind::Video {
                track.clips.retain(|c| c.id != clip_id);
            }
        }
        self.reflow_slide_timeline_positions();
        self.refresh_preview_frame(ctx);
    }

    pub fn adjust_slide_duration(&mut self, clip_id: u64, delta_secs: f64, ctx: Option<&Context>) {
        self.snapshot_timeline();
        if let Some(clip) = self.project.timeline.get_clip_mut(clip_id) {
            let current = clip.duration().as_secs_f64();
            let new_dur = (current + delta_secs).max(0.5);
            let time_dur = TimeCode::from_secs_f64(new_dur);
            clip.source_duration = time_dur;
            clip.source_out = time_dur;
        }
        self.reflow_slide_timeline_positions();
        self.refresh_preview_frame(ctx);
    }

    pub fn apply_template_title_2_media_to_active(&mut self, ctx: Option<&Context>) {
        self.snapshot_timeline();
        if let Some(id) = self.active_slide().map(|c| c.id) {
            if let Some(slide) = self.project.timeline.get_clip_mut(id) {
                slide.name = "Title + 2 Media".to_string();
                slide.elements.clear();

                let mut title = TextOverlay::default();
                title.text = "Add Slide Title Here".to_string();
                title.font_size = 32.0;
                title.x = 0.5;
                title.y = 0.12;
                title.alignment = TextAlignment::Center;
                title.font_family = FontFamilyPreset::Impact;
                slide.elements.push(SlideElement::Text(title));

                slide.elements.push(SlideElement::Placeholder {
                    slot_id: 1,
                    label: "Left Photo/Video".to_string(),
                    x: 0.05,
                    y: 0.25,
                    w: 0.43,
                    h: 0.65,
                });

                slide.elements.push(SlideElement::Placeholder {
                    slot_id: 2,
                    label: "Right Photo/Video".to_string(),
                    x: 0.52,
                    y: 0.25,
                    w: 0.43,
                    h: 0.65,
                });
            }
        }
        self.refresh_preview_frame(ctx);
    }

    pub fn apply_template_title_4_media_to_active(&mut self, ctx: Option<&Context>) {
        self.snapshot_timeline();
        if let Some(id) = self.active_slide().map(|c| c.id) {
            if let Some(slide) = self.project.timeline.get_clip_mut(id) {
                slide.name = "Title + 4 Grid".to_string();
                slide.elements.clear();

                let mut title = TextOverlay::default();
                title.text = "Grid Photo Gallery".to_string();
                title.font_size = 28.0;
                title.x = 0.5;
                title.y = 0.08;
                title.alignment = TextAlignment::Center;
                title.font_family = FontFamilyPreset::Impact;
                slide.elements.push(SlideElement::Text(title));

                let coords = [
                    (0.04, 0.18, 0.44, 0.36),
                    (0.52, 0.18, 0.44, 0.36),
                    (0.04, 0.58, 0.44, 0.36),
                    (0.52, 0.58, 0.44, 0.36),
                ];
                for (i, (x, y, w, h)) in coords.iter().enumerate() {
                    slide.elements.push(SlideElement::Placeholder {
                        slot_id: (i + 1) as u32,
                        label: format!("Slot {}", i + 1),
                        x: *x,
                        y: *y,
                        w: *w,
                        h: *h,
                    });
                }
            }
        }
        self.refresh_preview_frame(ctx);
    }

    pub fn apply_template_showcase_to_active(&mut self, ctx: Option<&Context>) {
        self.snapshot_timeline();
        if let Some(id) = self.active_slide().map(|c| c.id) {
            if let Some(slide) = self.project.timeline.get_clip_mut(id) {
                slide.name = "Feature Showcase".to_string();
                slide.elements.clear();

                slide.elements.push(SlideElement::Placeholder {
                    slot_id: 1,
                    label: "Main Showcase Media".to_string(),
                    x: 0.05,
                    y: 0.08,
                    w: 0.55,
                    h: 0.84,
                });

                let mut title = TextOverlay::default();
                title.text = "Feature Headline".to_string();
                title.font_size = 32.0;
                title.x = 0.80;
                title.y = 0.30;
                title.alignment = TextAlignment::Left;
                title.font_family = FontFamilyPreset::Impact;
                slide.elements.push(SlideElement::Text(title));

                let mut body = TextOverlay::default();
                body.text = "Write your story or description here.
Great for highlights and memories.".to_string();
                body.font_size = 18.0;
                body.x = 0.80;
                body.y = 0.55;
                body.alignment = TextAlignment::Left;
                body.font_family = FontFamilyPreset::SansSerif;
                slide.elements.push(SlideElement::Text(body));
            }
        }
        self.refresh_preview_frame(ctx);
    }

    pub fn render_bottom_slideshow_bar(&mut self, ui: &mut Ui, ctx: &Context) {
        let mut to_duplicate = None;
        let mut to_delete = None;
        let mut to_add_blank = false;
        let mut duration_delta = None;
        let mut nav_prev = false;
        let mut nav_next = false;

        ui.horizontal_centered(|ui| {
            ui.add_space(8.0);

            let total_slides = self.project.timeline.tracks.iter()
                .filter(|t| t.kind == TrackKind::Video)
                .map(|t| t.clips.len())
                .sum::<usize>();

            let active_idx = self.project.timeline.tracks.iter()
                .filter(|t| t.kind == TrackKind::Video)
                .flat_map(|t| t.clips.iter())
                .position(|c| c.is_selected);

            let cur_dur = self.active_slide().map(|c| c.duration().as_secs_f64()).unwrap_or(5.0);
            let cur_id = self.active_slide().map(|c| c.id);

            // Previous Slide
            if ui.add_enabled(active_idx.unwrap_or(0) > 0, Button::new("◀ Previous Slide").min_size(Vec2::new(100.0, 32.0))).clicked() {
                nav_prev = true;
            }

            ui.add_space(4.0);
            let pos_label = if let Some(idx) = active_idx {
                format!("Slide {} of {}", idx + 1, total_slides)
            } else {
                format!("{} Slides", total_slides)
            };
            ui.label(RichText::new(pos_label).strong().color(AppTheme::accent_yellow()));

            ui.add_space(4.0);
            // Next Slide
            let has_next = active_idx.map(|i| i + 1 < total_slides).unwrap_or(false);
            if ui.add_enabled(has_next, Button::new("Next Slide ▶").min_size(Vec2::new(90.0, 32.0))).clicked() {
                nav_next = true;
            }

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(16.0);

            // Duration quick adjuster
            ui.label(RichText::new("⏱ Slide Duration:").color(AppTheme::text_secondary()));
            if ui.button("- 0.5s").clicked() {
                if let Some(id) = cur_id {
                    duration_delta = Some((id, -0.5));
                }
            }
            ui.label(RichText::new(format!("{:.1}s", cur_dur)).strong().color(AppTheme::accent_cyan()));
            if ui.button("+ 0.5s").clicked() {
                if let Some(id) = cur_id {
                    duration_delta = Some((id, 0.5));
                }
            }

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(16.0);

            // Quick Actions: Delete, Duplicate, Add Slide
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                if ui.button(RichText::new("➕ Add New Slide").strong().color(Color32::WHITE))
                    .on_hover_text("Add a new blank slide to your presentation")
                    .clicked()
                {
                    to_add_blank = true;
                }

                if let Some(id) = cur_id {
                    if ui.button("⎘ Duplicate").clicked() {
                        to_duplicate = Some(id);
                    }
                    if ui.button("🗑 Delete").clicked() {
                        to_delete = Some(id);
                    }
                }
            });
        });

        if let Some((id, delta)) = duration_delta {
            self.adjust_slide_duration(id, delta, Some(ctx));
        }
        if let Some(id) = to_duplicate {
            self.duplicate_slide(id, Some(ctx));
        }
        if let Some(id) = to_delete {
            self.delete_slide_by_id(id, Some(ctx));
        }
        if to_add_blank {
            self.insert_blank_slide_at_playhead(5.0, Some(ctx));
        }
        if nav_prev {
            let active_idx = self.project.timeline.tracks.iter()
                .filter(|t| t.kind == TrackKind::Video)
                .flat_map(|t| t.clips.iter())
                .position(|c| c.is_selected);
            if let Some(idx) = active_idx {
                if idx > 0 {
                    self.reorder_slide(0, 0, Some(ctx)); // triggers select
                    if let Some(track) = self.project.timeline.tracks.iter_mut().find(|t| t.kind == TrackKind::Video) {
                        for (i, c) in track.clips.iter_mut().enumerate() {
                            c.is_selected = i == idx - 1;
                        }
                    }
                    self.refresh_preview_frame(Some(ctx));
                }
            }
        }
        if nav_next {
            let active_idx = self.project.timeline.tracks.iter()
                .filter(|t| t.kind == TrackKind::Video)
                .flat_map(|t| t.clips.iter())
                .position(|c| c.is_selected);
            if let Some(idx) = active_idx {
                if let Some(track) = self.project.timeline.tracks.iter_mut().find(|t| t.kind == TrackKind::Video) {
                    if idx + 1 < track.clips.len() {
                        for (i, c) in track.clips.iter_mut().enumerate() {
                            c.is_selected = i == idx + 1;
                        }
                    }
                }
                self.refresh_preview_frame(Some(ctx));
            }
        }
    }
}
