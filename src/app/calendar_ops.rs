use crate::app::VideoEditorApp;
use crate::core::calendar_gen::{CalendarMonth, CalendarOverlay};
use crate::core::clip::Clip;
use crate::core::text_overlay::SlideElement;
use crate::core::time::TimeCode;
use crate::core::track::TrackKind;
use egui::Context;

impl VideoEditorApp {
    pub fn insert_template_calendar_slide(
        &mut self,
        year: i32,
        start_month: u32,
        month_count: u32,
        show_holidays: bool,
        ctx: Option<&Context>,
    ) {
        self.snapshot_timeline();
        let track_id = self.project.timeline.tracks.iter().find(|t| t.kind == TrackKind::Video).map(|t| t.id)
            .unwrap_or_else(|| self.project.timeline.add_track("Video Track".to_string(), TrackKind::Video));
        let next_id = self.project.timeline.next_id();
        let count = month_count.clamp(1, 3);
        let end_month = (start_month + count - 1).min(12);

        let slide_title = if count == 1 {
            format!("{} {}", CalendarMonth::name_for_month(start_month), year)
        } else {
            format!("{} - {} {}", CalendarMonth::short_name_for_month(start_month), CalendarMonth::short_name_for_month(end_month), year)
        };

        let mut slide = Clip::new_blank_slide(next_id, track_id, slide_title.clone(), 5.0);
        slide.timeline_start = self.project.timeline.playhead;
        slide.is_selected = true;

        // Top landscape photo / artwork slot
        slide.elements.push(SlideElement::Placeholder {
            slot_id: 1,
            label: format!("{} Artwork / Photo", slide_title),
            x: 0.05,
            y: 0.05,
            w: 0.90,
            h: 0.44,
        });

        // Bottom vector calendar element
        let mut cal_overlay = CalendarOverlay::default();
        cal_overlay.year = year;
        cal_overlay.start_month = start_month;
        cal_overlay.month_count = count;
        cal_overlay.show_holidays = show_holidays;
        cal_overlay.holidays = if self.calendar_holidays.is_empty() {
            CalendarMonth::default_holidays_for_year(year)
        } else {
            self.calendar_holidays.clone()
        };
        cal_overlay.custom_events = self.calendar_custom_events.clone();
        cal_overlay.x = 0.05;
        cal_overlay.y = 0.52;
        cal_overlay.w = 0.90;
        cal_overlay.h = 0.44;
        slide.elements.push(SlideElement::Calendar(cal_overlay));

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

    pub fn generate_12_month_calendar(
        &mut self,
        year: i32,
        month_count: u32,
        show_holidays: bool,
        ctx: Option<&Context>,
    ) {
        self.snapshot_timeline();
        let track_id = self.project.timeline.tracks.iter().find(|t| t.kind == TrackKind::Video).map(|t| t.id)
            .unwrap_or_else(|| self.project.timeline.add_track("Video Track".to_string(), TrackKind::Video));

        if self.calendar_holidays.is_empty() {
            self.calendar_holidays = CalendarMonth::default_holidays_for_year(year);
        }

        let mut cur_time = self.project.timeline.playhead;
        let count = month_count.clamp(1, 3);
        let mut start_m = 1;

        while start_m <= 12 {
            let end_m = (start_m + count - 1).min(12);
            let slide_title = if count == 1 {
                format!("{} {}", CalendarMonth::name_for_month(start_m), year)
            } else {
                format!("{} - {} {}", CalendarMonth::short_name_for_month(start_m), CalendarMonth::short_name_for_month(end_m), year)
            };
            let next_id = self.project.timeline.next_id();
            let mut slide = Clip::new_blank_slide(next_id, track_id, slide_title.clone(), 5.0);
            slide.timeline_start = cur_time;
            slide.is_selected = start_m == 1;

            // Top landscape photo / artwork slot
            slide.elements.push(SlideElement::Placeholder {
                slot_id: 1,
                label: format!("{} Artwork / Photo", slide_title),
                x: 0.05,
                y: 0.05,
                w: 0.90,
                h: 0.44,
            });

            // Bottom vector calendar element
            let mut cal_overlay = CalendarOverlay::default();
            cal_overlay.year = year;
            cal_overlay.start_month = start_m;
            cal_overlay.month_count = count;
            cal_overlay.show_holidays = show_holidays;
            cal_overlay.holidays = self.calendar_holidays.clone();
            cal_overlay.custom_events = self.calendar_custom_events.clone();
            cal_overlay.x = 0.05;
            cal_overlay.y = 0.52;
            cal_overlay.w = 0.90;
            cal_overlay.h = 0.44;
            slide.elements.push(SlideElement::Calendar(cal_overlay));

            if let Some(track) = self.project.timeline.get_track_mut(track_id) {
                track.add_clip(slide);
            }
            cur_time += TimeCode::from_secs_f64(5.0);
            start_m += count;
        }
        self.refresh_preview_frame(ctx);
    }

    pub fn apply_template_calendar_to_active(
        &mut self,
        year: i32,
        start_month: u32,
        month_count: u32,
        show_holidays: bool,
        ctx: Option<&Context>,
    ) {
        self.snapshot_timeline();
        if let Some(id) = self.active_slide().map(|c| c.id) {
            if let Some(slide) = self.project.timeline.get_clip_mut(id) {
                let count = month_count.clamp(1, 3);
                let end_month = (start_month + count - 1).min(12);
                let slide_title = if count == 1 {
                    format!("{} {}", CalendarMonth::name_for_month(start_month), year)
                } else {
                    format!("{} - {} {}", CalendarMonth::short_name_for_month(start_month), CalendarMonth::short_name_for_month(end_month), year)
                };
                slide.name = slide_title.clone();

                let mut cal_overlay = CalendarOverlay::default();
                cal_overlay.year = year;
                cal_overlay.start_month = start_month;
                cal_overlay.month_count = count;
                cal_overlay.show_holidays = show_holidays;
                cal_overlay.holidays = if self.calendar_holidays.is_empty() {
                    CalendarMonth::default_holidays_for_year(year)
                } else {
                    self.calendar_holidays.clone()
                };
                cal_overlay.custom_events = self.calendar_custom_events.clone();
                cal_overlay.x = 0.05;
                cal_overlay.y = 0.52;
                cal_overlay.w = 0.90;
                cal_overlay.h = 0.44;

                let mut replaced = false;
                for el in &mut slide.elements {
                    if let SlideElement::Calendar(c) = el {
                        *c = cal_overlay.clone();
                        replaced = true;
                        break;
                    }
                }
                if !replaced {
                    slide.elements.push(SlideElement::Calendar(cal_overlay));
                }
            }
        }
        self.refresh_preview_frame(ctx);
    }

    pub fn update_active_calendar_slide(&mut self, ctx: Option<&Context>) {
        if let Some(id) = self.active_slide().map(|c| c.id) {
            if let Some(clip) = self.project.timeline.get_clip_mut(id) {
                for el in &mut clip.elements {
                    if let SlideElement::Calendar(c) = el {
                        c.year = self.calendar_year;
                        c.start_month = self.calendar_start_month;
                        c.month_count = self.calendar_month_count;
                        c.show_holidays = self.calendar_show_holidays;
                        c.holidays = self.calendar_holidays.clone();
                        c.custom_events = self.calendar_custom_events.clone();
                    }
                }
            }
        }
        self.refresh_preview_frame(ctx);
    }

    pub fn export_printable_calendar_sheets(
        &self,
        output_dir: &std::path::Path,
        year: i32,
        month_count: u32,
        show_holidays: bool,
    ) -> std::io::Result<Vec<std::path::PathBuf>> {
        std::fs::create_dir_all(output_dir)?;
        let mut generated = Vec::new();
        let count = month_count.clamp(1, 3);
        let mut start_m = 1;

        while start_m <= 12 {
            let end_m = (start_m + count - 1).min(12);
            let grid_img = CalendarMonth::render_multi_grid_image(
                year,
                start_m,
                count,
                1920,
                1080,
                show_holidays,
                &self.calendar_holidays,
            );
            let name_label = if count == 1 {
                format!("{:02}_{}", start_m, CalendarMonth::name_for_month(start_m))
            } else {
                format!("{:02}_{}_to_{:02}_{}", start_m, CalendarMonth::short_name_for_month(start_m), end_m, CalendarMonth::short_name_for_month(end_m))
            };
            let grid_path = output_dir.join(format!("{}_{}_Calendar_Grid.png", name_label, year));
            let raw_bytes: Vec<u8> = grid_img.pixels.iter().flat_map(|p| p.to_array()).collect();
            if let Some(img_buf) = image::RgbaImage::from_raw(1920, 1080, raw_bytes) {
                let _ = img_buf.save(&grid_path);
                generated.push(grid_path);
            }
            start_m += count;
        }
        Ok(generated)
    }
}
