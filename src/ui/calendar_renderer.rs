use crate::core::calendar_gen::{CalendarMonth, CustomCalendarEvent, HolidayItem};
use crate::core::text_overlay::CalendarOverlay;
use egui::{Align2, Color32, FontId, Pos2, Rect, Rounding, Vec2};

pub struct CalendarRenderer;

impl CalendarRenderer {
    pub fn draw_calendar_element(
        painter: &egui::Painter,
        card_rect: Rect,
        cal: &CalendarOverlay,
    ) {
        if card_rect.width() <= 30.0 || card_rect.height() <= 30.0 {
            return;
        }
        let scale = (card_rect.height() / 280.0).clamp(0.4, 3.0);

        // 1. Sleek Glassmorphic Card Container
        let card_bg = Color32::from_rgba_premultiplied(16, 22, 34, 240);
        let card_border = Color32::from_rgba_premultiplied(70, 92, 130, 220);
        painter.rect_filled(card_rect, Rounding::same(10.0 * scale), card_bg);
        painter.rect_stroke(
            card_rect,
            Rounding::same(10.0 * scale),
            egui::Stroke::new(1.5 * scale, card_border),
        );

        let month_count = cal.month_count.clamp(1, 3);
        let pad_x = 10.0 * scale;
        let pad_y = 8.0 * scale;
        let inner_rect = card_rect.shrink2(Vec2::new(pad_x, pad_y));
        if inner_rect.width() <= 20.0 || inner_rect.height() <= 20.0 {
            return;
        }

        let gutter = 10.0 * scale;
        let total_gutters = gutter * (month_count.saturating_sub(1) as f32);
        let col_w = (inner_rect.width() - total_gutters) / (month_count as f32);

        for m_offset in 0..month_count {
            let month = cal.start_month + m_offset;
            if month > 12 {
                break;
            }

            let col_min_x = inner_rect.min.x + (m_offset as f32) * (col_w + gutter);
            let col_rect = Rect::from_min_size(
                Pos2::new(col_min_x, inner_rect.min.y),
                Vec2::new(col_w, inner_rect.height()),
            );

            Self::draw_single_month_column(
                painter,
                col_rect,
                cal.year,
                month,
                cal.show_holidays,
                &cal.holidays,
                &cal.custom_events,
                scale,
            );
        }
    }

    fn draw_single_month_column(
        painter: &egui::Painter,
        col_rect: Rect,
        year: i32,
        month: u32,
        show_holidays: bool,
        holidays: &[HolidayItem],
        custom_events: &[CustomCalendarEvent],
        scale: f32,
    ) {
        let total_h = col_rect.height();
        let header_h = (26.0 * scale).clamp(18.0, 44.0);
        let weekday_h = (18.0 * scale).clamp(14.0, 30.0);
        let footnotes_h = if show_holidays {
            (20.0 * scale).clamp(14.0, 32.0)
        } else {
            0.0
        };
        let grid_h = (total_h - header_h - weekday_h - footnotes_h - 8.0 * scale).max(20.0);

        // 1. Month & Year Header Banner
        let header_rect = Rect::from_min_size(col_rect.min, Vec2::new(col_rect.width(), header_h));
        let header_bg = Color32::from_rgba_premultiplied(32, 44, 68, 230);
        painter.rect_filled(header_rect, Rounding::same(6.0 * scale), header_bg);
        painter.rect_stroke(
            header_rect,
            Rounding::same(6.0 * scale),
            egui::Stroke::new(1.0 * scale, Color32::from_rgba_premultiplied(70, 95, 140, 150)),
        );

        let title_text = format!("{} {}", CalendarMonth::name_for_month(month), year);
        let font_title = FontId::proportional((14.0 * scale).clamp(11.0, 24.0));
        painter.text(
            header_rect.center(),
            Align2::CENTER_CENTER,
            title_text,
            font_title,
            Color32::WHITE,
        );

        // 2. Weekday Header Row
        let weekday_rect = Rect::from_min_size(
            Pos2::new(col_rect.min.x, header_rect.max.y + 3.0 * scale),
            Vec2::new(col_rect.width(), weekday_h),
        );
        let weekday_names = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
        let day_w = weekday_rect.width() / 7.0;
        let font_weekday = FontId::proportional((10.0 * scale).clamp(8.0, 16.0));

        for (i, name) in weekday_names.iter().enumerate() {
            let wx = weekday_rect.min.x + (i as f32) * day_w + day_w / 2.0;
            let wy = weekday_rect.center().y;
            let col = if i == 0 || i == 6 {
                Color32::from_rgb(255, 175, 120) // Soft coral for weekends
            } else {
                Color32::from_rgb(180, 205, 235) // Clean cool white/blue for weekdays
            };
            painter.text(
                Pos2::new(wx, wy),
                Align2::CENTER_CENTER,
                *name,
                font_weekday.clone(),
                col,
            );
        }

        // 3. Graphical Day Grid (7 columns x 5 or 6 rows)
        let grid_rect = Rect::from_min_size(
            Pos2::new(col_rect.min.x, weekday_rect.max.y + 2.0 * scale),
            Vec2::new(col_rect.width(), grid_h),
        );

        let first_wd = CalendarMonth::first_weekday_for(year, month);
        let total_days = CalendarMonth::days_in_year_month(year, month);
        let total_slots = first_wd + total_days as usize;
        let num_rows = ((total_slots + 6) / 7).max(5);
        let cell_w = grid_rect.width() / 7.0;
        let cell_h = grid_rect.height() / (num_rows as f32);

        let font_day_num = FontId::proportional((12.0 * scale).clamp(9.0, 20.0));
        let font_badge = FontId::proportional((8.5 * scale).clamp(6.5, 13.0));

        for d in 1..=total_days {
            let slot = first_wd + (d as usize) - 1;
            let col_idx = slot % 7;
            let row_idx = slot / 7;

            let cell_box = Rect::from_min_size(
                Pos2::new(grid_rect.min.x + (col_idx as f32) * cell_w, grid_rect.min.y + (row_idx as f32) * cell_h),
                Vec2::new(cell_w, cell_h),
            ).shrink(1.5 * scale);

            let is_weekend = col_idx == 0 || col_idx == 6;

            // Find any holidays for this day
            let day_holiday = if show_holidays {
                holidays.iter().find(|h| h.enabled && h.month == month && h.day == d)
            } else {
                None
            };
            let day_custom = if show_holidays && day_holiday.is_none() {
                custom_events.iter().find(|e| e.month == month && e.day == d)
            } else {
                None
            };

            let (cell_bg, border_stroke) = if let Some(h) = day_holiday {
                let col = h.color32();
                let bg = Color32::from_rgba_premultiplied(
                    (col.r() / 5).max(28),
                    (col.g() / 5).max(34),
                    (col.b() / 5).max(50),
                    235,
                );
                let border = egui::Stroke::new(1.2 * scale, col);
                (bg, border)
            } else if let Some(ev) = day_custom {
                let col = Color32::from_rgba_premultiplied(ev.color[0], ev.color[1], ev.color[2], ev.color[3]);
                let bg = Color32::from_rgba_premultiplied(
                    (col.r() / 5).max(28),
                    (col.g() / 5).max(34),
                    (col.b() / 5).max(50),
                    235,
                );
                let border = egui::Stroke::new(1.2 * scale, col);
                (bg, border)
            } else if is_weekend {
                (
                    Color32::from_rgba_premultiplied(28, 35, 52, 220),
                    egui::Stroke::new(0.8 * scale, Color32::from_rgba_premultiplied(65, 80, 110, 140)),
                )
            } else {
                (
                    Color32::from_rgba_premultiplied(22, 28, 42, 220),
                    egui::Stroke::new(0.8 * scale, Color32::from_rgba_premultiplied(50, 65, 90, 120)),
                )
            };

            // Draw Day Card
            painter.rect_filled(cell_box, Rounding::same(4.0 * scale), cell_bg);
            painter.rect_stroke(cell_box, Rounding::same(4.0 * scale), border_stroke);

            // Draw Day Number (Top-Left)
            let num_pos = Pos2::new(cell_box.min.x + 4.0 * scale, cell_box.min.y + 2.5 * scale);
            painter.text(
                num_pos,
                Align2::LEFT_TOP,
                format!("{}", d),
                font_day_num.clone(),
                if is_weekend { Color32::from_rgb(255, 205, 170) } else { Color32::WHITE },
            );

            // Draw Holiday Mini-Pill Badge (Bottom)
            if let Some(h) = day_holiday {
                let badge_text = h.short_badge();
                let pill_h = (11.0 * scale).clamp(8.0, 18.0);
                let pill_rect = Rect::from_min_max(
                    Pos2::new(cell_box.min.x + 2.0 * scale, cell_box.max.y - pill_h - 2.0 * scale),
                    Pos2::new(cell_box.max.x - 2.0 * scale, cell_box.max.y - 2.0 * scale),
                );
                let hcol = h.color32();
                painter.rect_filled(
                    pill_rect,
                    Rounding::same(2.5 * scale),
                    Color32::from_rgba_premultiplied(hcol.r() / 4, hcol.g() / 4, hcol.b() / 4, 220),
                );
                painter.text(
                    pill_rect.center(),
                    Align2::CENTER_CENTER,
                    badge_text,
                    font_badge.clone(),
                    hcol,
                );
            } else if let Some(ev) = day_custom {
                let pill_h = (11.0 * scale).clamp(8.0, 18.0);
                let pill_rect = Rect::from_min_max(
                    Pos2::new(cell_box.min.x + 2.0 * scale, cell_box.max.y - pill_h - 2.0 * scale),
                    Pos2::new(cell_box.max.x - 2.0 * scale, cell_box.max.y - 2.0 * scale),
                );
                let ecol = Color32::from_rgba_premultiplied(ev.color[0], ev.color[1], ev.color[2], ev.color[3]);
                painter.rect_filled(
                    pill_rect,
                    Rounding::same(2.5 * scale),
                    Color32::from_rgba_premultiplied(ecol.r() / 4, ecol.g() / 4, ecol.b() / 4, 220),
                );
                let label_short = if ev.label.len() > 7 { &ev.label[..7] } else { &ev.label };
                painter.text(
                    pill_rect.center(),
                    Align2::CENTER_CENTER,
                    label_short,
                    font_badge.clone(),
                    ecol,
                );
            }
        }

        // 4. Bottom Holiday Footnote / Legend Bar
        if show_holidays && footnotes_h > 0.0 {
            let foot_rect = Rect::from_min_size(
                Pos2::new(col_rect.min.x, grid_rect.max.y + 4.0 * scale),
                Vec2::new(col_rect.width(), footnotes_h),
            );

            let mut month_holidays: Vec<(&str, u32, Color32)> = Vec::new();
            for h in holidays {
                if h.enabled && h.month == month {
                    month_holidays.push((&h.name, h.day, h.color32()));
                }
            }
            for ev in custom_events {
                if ev.month == month {
                    let col = Color32::from_rgba_premultiplied(ev.color[0], ev.color[1], ev.color[2], ev.color[3]);
                    month_holidays.push((&ev.label, ev.day, col));
                }
            }

            if !month_holidays.is_empty() {
                let font_foot = FontId::proportional((9.0 * scale).clamp(7.0, 14.0));
                let mut cur_x = foot_rect.min.x + 4.0 * scale;
                let cy = foot_rect.center().y;

                for (name, day, col) in month_holidays {
                    if cur_x + 60.0 * scale > foot_rect.max.x {
                        break;
                    }
                    // Draw colored dot
                    painter.circle_filled(Pos2::new(cur_x + 3.0 * scale, cy), 3.0 * scale, col);
                    cur_x += 8.0 * scale;

                    let label = format!("{}: {}", day, name);
                    let galley = painter.layout_no_wrap(label.clone(), font_foot.clone(), Color32::from_rgb(220, 230, 245));
                    let gw = galley.size().x;
                    painter.galley(Pos2::new(cur_x, cy - galley.size().y / 2.0), galley, Color32::from_rgb(220, 230, 245));
                    cur_x += gw + 10.0 * scale;
                }
            }
        }
    }
}
