use egui::{Color32, ColorImage};
use image::{ImageBuffer, Rgba};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum HolidayCategory {
    American,
    Chinese,
    Custom,
}

impl HolidayCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::American => "[US] American Holidays",
            Self::Chinese => "[CN] Chinese Festivals",
            Self::Custom => "[Event] Custom / Family Events",
        }
    }
}

/// Visual style of the calendar on slides and printable sheets
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum CalendarStyle {
    #[default]
    BoxedGrid, // Classic wall calendar with individual day boxes and bottom-right holidays
    SimpleGrid, // Simple monospace table
}

impl CalendarStyle {
    pub fn label(&self) -> &'static str {
        match self {
            Self::BoxedGrid => "🧱 Boxed Wall Calendar",
            Self::SimpleGrid => "📋 Simple Grid",
        }
    }
}

/// A configurable holiday or cultural event with custom color
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HolidayItem {
    pub id: String,
    pub name: String,
    pub month: u32,
    pub day: u32,
    pub category: HolidayCategory,
    pub enabled: bool,
    #[serde(default = "default_holiday_color")]
    pub color: [u8; 4], // RGBA
}

fn default_holiday_color() -> [u8; 4] {
    [255, 215, 0, 255] // Gold
}

impl HolidayItem {
    pub fn new(id: &str, name: &str, month: u32, day: u32, category: HolidayCategory, color: Color32) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            month,
            day,
            category,
            enabled: true,
            color: [color.r(), color.g(), color.b(), color.a()],
        }
    }

    pub fn color32(&self) -> Color32 {
        Color32::from_rgba_premultiplied(self.color[0], self.color[1], self.color[2], self.color[3])
    }

    pub fn set_color32(&mut self, col: Color32) {
        self.color = [col.r(), col.g(), col.b(), col.a()];
    }

    /// Short label displayed inside the day box (bottom-right)
    pub fn short_badge(&self) -> &'static str {
        match self.id.as_str() {
            "us_new_year" => "NewYr",
            "us_mlk" => "MLK",
            "us_valentines" => "V-Day",
            "us_presidents" => "Pres",
            "us_st_patrick" => "StPat",
            "us_easter" => "Easter",
            "us_earth_day" => "Earth",
            "us_mothers_day" => "Moms",
            "us_memorial" => "Mem",
            "us_juneteenth" => "June19",
            "us_fathers_day" => "Dads",
            "us_independence" => "July4",
            "us_labor" => "Labor",
            "us_columbus" => "Columb",
            "us_halloween" => "Spooky",
            "us_veterans" => "Vets",
            "us_thanksgiving" => "Thanks",
            "us_christmas_eve" => "XmEve",
            "us_christmas" => "Xmas",
            "us_new_years_eve" => "NYE",
            "cn_cny" => "CNY",
            "cn_lantern" => "Lantern",
            "cn_qingming" => "QingM",
            "cn_dragon_boat" => "Dragon",
            "cn_qixi" => "Qixi",
            "cn_mid_autumn" => "Moon",
            "cn_double_ninth" => "9-9th",
            "cn_dongzhi" => "DongZ",
            _ => "Event",
        }
    }
}

/// Custom event / family holiday (e.g. Grandma's Birthday)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomCalendarEvent {
    pub month: u32,
    pub day: u32,
    pub label: String,
    pub color: [u8; 4],
}

impl Default for CustomCalendarEvent {
    fn default() -> Self {
        Self {
            month: 1,
            day: 1,
            label: "Family Birthday".to_string(),
            color: [255, 105, 180, 255], // Hot pink
        }
    }
}

impl CustomCalendarEvent {
    pub fn short_badge(&self) -> String {
        let clean = self.label.trim();
        if clean.chars().count() <= 6 {
            format!("★{}", clean)
        } else {
            let truncated: String = clean.chars().take(5).collect();
            format!("★{}", truncated)
        }
    }
}

fn pad_visual_right(s: &str, target_width: usize) -> String {
    let char_count = s.chars().count();
    if char_count >= target_width {
        s.chars().take(target_width).collect()
    } else {
        let pad = " ".repeat(target_width - char_count);
        format!("{}{}", pad, s)
    }
}

fn pad_visual_left(s: &str, target_width: usize) -> String {
    let char_count = s.chars().count();
    if char_count >= target_width {
        s.chars().take(target_width).collect()
    } else {
        let pad = " ".repeat(target_width - char_count);
        format!("{}{}", s, pad)
    }
}

fn pad_visual_center(s: &str, target_width: usize) -> String {
    let char_count = s.chars().count();
    if char_count >= target_width {
        s.chars().take(target_width).collect()
    } else {
        let total_pad = target_width - char_count;
        let left_pad = total_pad / 2;
        let right_pad = total_pad - left_pad;
        format!("{}{}{}", " ".repeat(left_pad), s, " ".repeat(right_pad))
    }
}

/// Accurate Gregorian calendar month helper with holiday calculations and multi-month layouts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarMonth {
    pub year: i32,
    pub month: u32, // 1..=12
}

impl CalendarMonth {
    pub fn new(year: i32, month: u32) -> Self {
        Self {
            year,
            month: month.clamp(1, 12),
        }
    }

    pub fn month_name(&self) -> &'static str {
        Self::name_for_month(self.month)
    }

    pub fn name_for_month(m: u32) -> &'static str {
        match m {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "January",
        }
    }

    pub fn short_name_for_month(m: u32) -> &'static str {
        match m {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => "Jan",
        }
    }

    pub fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    pub fn days_in_month(&self) -> u32 {
        Self::days_in_year_month(self.year, self.month)
    }

    pub fn days_in_year_month(year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }

    pub fn first_weekday(&self) -> usize {
        Self::first_weekday_for(self.year, self.month)
    }

    pub fn first_weekday_for(year: i32, month: u32) -> usize {
        let (y, m) = if month < 3 {
            (year - 1, month + 12)
        } else {
            (year, month)
        };
        let k = y % 100;
        let j = y / 100;
        let h = (1 + (13 * (m as i32 + 1)) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
        ((h + 6) % 7) as usize
    }

    pub fn nth_weekday_of_month(year: i32, month: u32, weekday_target: usize, n: usize) -> u32 {
        let first_w = Self::first_weekday_for(year, month);
        let mut day = 1;
        let mut count = 0;
        for d in 1..=Self::days_in_year_month(year, month) {
            let cur_w = (first_w + (d as usize - 1)) % 7;
            if cur_w == weekday_target {
                count += 1;
                if count == n {
                    day = d;
                    break;
                }
            }
        }
        day
    }

    pub fn last_weekday_of_month(year: i32, month: u32, weekday_target: usize) -> u32 {
        let first_w = Self::first_weekday_for(year, month);
        let num_days = Self::days_in_year_month(year, month);
        let mut last_d = 1;
        for d in 1..=num_days {
            let cur_w = (first_w + (d as usize - 1)) % 7;
            if cur_w == weekday_target {
                last_d = d;
            }
        }
        last_d
    }

    pub fn calculate_easter(year: i32) -> (u32, u32) {
        let a = year % 19;
        let b = year / 100;
        let c = year % 100;
        let d = b / 4;
        let e = b % 4;
        let f = (b + 8) / 25;
        let g = (b - f + 1) / 3;
        let h = (19 * a + b - d - g + 15) % 30;
        let i = c / 4;
        let k = c % 4;
        let l = (32 + 2 * e + 2 * i - h - k) % 7;
        let m = (a + 11 * h + 22 * l) / 451;
        let month = (h + l - 7 * m + 114) / 31;
        let day = ((h + l - 7 * m + 114) % 31) + 1;
        (month as u32, day as u32)
    }

    pub fn get_chinese_festival_date(year: i32, festival_id: &str) -> Option<(u32, u32)> {
        match (year, festival_id) {
            (2024, "cny") => Some((2, 10)),
            (2024, "lantern") => Some((2, 24)),
            (2024, "qingming") => Some((4, 4)),
            (2024, "dragon_boat") => Some((6, 10)),
            (2024, "qixi") => Some((8, 10)),
            (2024, "mid_autumn") => Some((9, 17)),
            (2024, "double_ninth") => Some((10, 11)),
            (2024, "dongzhi") => Some((12, 21)),

            (2025, "cny") => Some((1, 29)),
            (2025, "lantern") => Some((2, 12)),
            (2025, "qingming") => Some((4, 4)),
            (2025, "dragon_boat") => Some((5, 31)),
            (2025, "qixi") => Some((8, 29)),
            (2025, "mid_autumn") => Some((10, 6)),
            (2025, "double_ninth") => Some((10, 29)),
            (2025, "dongzhi") => Some((12, 21)),

            (2026, "cny") => Some((2, 17)),
            (2026, "lantern") => Some((3, 3)),
            (2026, "qingming") => Some((4, 5)),
            (2026, "dragon_boat") => Some((6, 19)),
            (2026, "qixi") => Some((8, 19)),
            (2026, "mid_autumn") => Some((9, 25)),
            (2026, "double_ninth") => Some((10, 18)),
            (2026, "dongzhi") => Some((12, 21)),

            (2027, "cny") => Some((2, 6)),
            (2027, "lantern") => Some((2, 20)),
            (2027, "qingming") => Some((4, 5)),
            (2027, "dragon_boat") => Some((6, 9)),
            (2027, "qixi") => Some((8, 8)),
            (2027, "mid_autumn") => Some((9, 15)),
            (2027, "double_ninth") => Some((10, 8)),
            (2027, "dongzhi") => Some((12, 22)),

            (2028, "cny") => Some((1, 26)),
            (2028, "lantern") => Some((2, 9)),
            (2028, "qingming") => Some((4, 4)),
            (2028, "dragon_boat") => Some((5, 28)),
            (2028, "qixi") => Some((8, 26)),
            (2028, "mid_autumn") => Some((10, 3)),
            (2028, "double_ninth") => Some((10, 26)),
            (2028, "dongzhi") => Some((12, 21)),

            (2029, "cny") => Some((2, 13)),
            (2029, "lantern") => Some((2, 27)),
            (2029, "qingming") => Some((4, 4)),
            (2029, "dragon_boat") => Some((6, 16)),
            (2029, "qixi") => Some((8, 16)),
            (2029, "mid_autumn") => Some((9, 22)),
            (2029, "double_ninth") => Some((10, 16)),
            (2029, "dongzhi") => Some((12, 21)),

            (2030, "cny") => Some((2, 3)),
            (2030, "lantern") => Some((2, 17)),
            (2030, "qingming") => Some((4, 5)),
            (2030, "dragon_boat") => Some((6, 5)),
            (2030, "qixi") => Some((8, 5)),
            (2030, "mid_autumn") => Some((9, 12)),
            (2030, "double_ninth") => Some((10, 5)),
            (2030, "dongzhi") => Some((12, 22)),

            _ => {
                let approx_day = ((year * 11) % 25 + 1) as u32;
                match festival_id {
                    "cny" => Some((2, approx_day.clamp(1, 20))),
                    "lantern" => Some((2, (approx_day + 14).clamp(1, 28))),
                    "qingming" => Some((4, 5)),
                    "dragon_boat" => Some((6, (approx_day + 5).clamp(1, 28))),
                    "qixi" => Some((8, (approx_day + 5).clamp(1, 28))),
                    "mid_autumn" => Some((9, (approx_day + 10).clamp(1, 28))),
                    "double_ninth" => Some((10, (approx_day + 5).clamp(1, 28))),
                    "dongzhi" => Some((12, 21)),
                    _ => None,
                }
            }
        }
    }

    pub fn default_holidays_for_year(year: i32) -> Vec<HolidayItem> {
        let mut list = Vec::new();

        // -----------------
        // American Holidays
        // -----------------
        list.push(HolidayItem::new("us_new_year", "New Year's Day", 1, 1, HolidayCategory::American, Color32::from_rgb(52, 152, 219)));
        let mlk_day = Self::nth_weekday_of_month(year, 1, 1, 3);
        list.push(HolidayItem::new("us_mlk", "Martin Luther King Jr. Day", 1, mlk_day, HolidayCategory::American, Color32::from_rgb(155, 89, 182)));
        list.push(HolidayItem::new("us_valentines", "Valentine's Day", 2, 14, HolidayCategory::American, Color32::from_rgb(231, 76, 60)));
        let pres_day = Self::nth_weekday_of_month(year, 2, 1, 3);
        list.push(HolidayItem::new("us_presidents", "Presidents' Day", 2, pres_day, HolidayCategory::American, Color32::from_rgb(41, 128, 185)));
        list.push(HolidayItem::new("us_st_patrick", "St. Patrick's Day", 3, 17, HolidayCategory::American, Color32::from_rgb(46, 204, 113)));
        let (easter_m, easter_d) = Self::calculate_easter(year);
        list.push(HolidayItem::new("us_easter", "Easter Sunday", easter_m, easter_d, HolidayCategory::American, Color32::from_rgb(241, 196, 15)));
        list.push(HolidayItem::new("us_earth_day", "Earth Day", 4, 22, HolidayCategory::American, Color32::from_rgb(39, 174, 96)));
        let mothers_day = Self::nth_weekday_of_month(year, 5, 0, 2);
        list.push(HolidayItem::new("us_mothers_day", "Mother's Day", 5, mothers_day, HolidayCategory::American, Color32::from_rgb(255, 105, 180)));
        let mem_day = Self::last_weekday_of_month(year, 5, 1);
        list.push(HolidayItem::new("us_memorial", "Memorial Day", 5, mem_day, HolidayCategory::American, Color32::from_rgb(192, 57, 43)));
        list.push(HolidayItem::new("us_juneteenth", "Juneteenth", 6, 19, HolidayCategory::American, Color32::from_rgb(230, 126, 34)));
        let fathers_day = Self::nth_weekday_of_month(year, 6, 0, 3);
        list.push(HolidayItem::new("us_fathers_day", "Father's Day", 6, fathers_day, HolidayCategory::American, Color32::from_rgb(52, 152, 219)));
        list.push(HolidayItem::new("us_independence", "Independence Day (4th of July)", 7, 4, HolidayCategory::American, Color32::from_rgb(231, 76, 60)));
        let labor_day = Self::nth_weekday_of_month(year, 9, 1, 1);
        list.push(HolidayItem::new("us_labor", "Labor Day", 9, labor_day, HolidayCategory::American, Color32::from_rgb(41, 128, 185)));
        let columbus_day = Self::nth_weekday_of_month(year, 10, 1, 2);
        list.push(HolidayItem::new("us_columbus", "Columbus / Indigenous Peoples Day", 10, columbus_day, HolidayCategory::American, Color32::from_rgb(142, 68, 173)));
        list.push(HolidayItem::new("us_halloween", "Halloween", 10, 31, HolidayCategory::American, Color32::from_rgb(230, 126, 34)));
        list.push(HolidayItem::new("us_veterans", "Veterans Day", 11, 11, HolidayCategory::American, Color32::from_rgb(41, 128, 185)));
        let thanks_day = Self::nth_weekday_of_month(year, 11, 4, 4);
        list.push(HolidayItem::new("us_thanksgiving", "Thanksgiving Day", 11, thanks_day, HolidayCategory::American, Color32::from_rgb(211, 84, 0)));
        list.push(HolidayItem::new("us_christmas_eve", "Christmas Eve", 12, 24, HolidayCategory::American, Color32::from_rgb(39, 174, 96)));
        list.push(HolidayItem::new("us_christmas", "Christmas Day", 12, 25, HolidayCategory::American, Color32::from_rgb(192, 57, 43)));
        list.push(HolidayItem::new("us_new_years_eve", "New Year's Eve", 12, 31, HolidayCategory::American, Color32::from_rgb(241, 196, 15)));

        // ----------------
        // Chinese Holidays
        // ----------------
        if let Some((m, d)) = Self::get_chinese_festival_date(year, "cny") {
            list.push(HolidayItem::new("cn_cny", "Chinese New Year (Spring Festival)", m, d, HolidayCategory::Chinese, Color32::from_rgb(230, 0, 18)));
        }
        if let Some((m, d)) = Self::get_chinese_festival_date(year, "lantern") {
            list.push(HolidayItem::new("cn_lantern", "Lantern Festival (Yuanxiao)", m, d, HolidayCategory::Chinese, Color32::from_rgb(255, 80, 0)));
        }
        if let Some((m, d)) = Self::get_chinese_festival_date(year, "qingming") {
            list.push(HolidayItem::new("cn_qingming", "Qingming (Tomb Sweeping)", m, d, HolidayCategory::Chinese, Color32::from_rgb(46, 204, 113)));
        }
        if let Some((m, d)) = Self::get_chinese_festival_date(year, "dragon_boat") {
            list.push(HolidayItem::new("cn_dragon_boat", "Dragon Boat Festival (Duanwu)", m, d, HolidayCategory::Chinese, Color32::from_rgb(26, 188, 156)));
        }
        if let Some((m, d)) = Self::get_chinese_festival_date(year, "qixi") {
            list.push(HolidayItem::new("cn_qixi", "Qixi (Chinese Valentine's)", m, d, HolidayCategory::Chinese, Color32::from_rgb(255, 105, 180)));
        }
        if let Some((m, d)) = Self::get_chinese_festival_date(year, "mid_autumn") {
            list.push(HolidayItem::new("cn_mid_autumn", "Mid-Autumn Moon Festival", m, d, HolidayCategory::Chinese, Color32::from_rgb(241, 196, 15)));
        }
        if let Some((m, d)) = Self::get_chinese_festival_date(year, "double_ninth") {
            list.push(HolidayItem::new("cn_double_ninth", "Double Ninth (Chongyang)", m, d, HolidayCategory::Chinese, Color32::from_rgb(175, 122, 197)));
        }
        if let Some((m, d)) = Self::get_chinese_festival_date(year, "dongzhi") {
            list.push(HolidayItem::new("cn_dongzhi", "Winter Solstice (Dongzhi)", m, d, HolidayCategory::Chinese, Color32::from_rgb(127, 140, 141)));
        }

        list.sort_by_key(|h| (h.month, h.day));
        list
    }

    /// Single month simple grid lines (fixed 8 lines: header, days header, and up to 6 week rows)
    pub fn get_simple_month_grid_lines(year: i32, month: u32, compact: bool) -> Vec<String> {
        let mut lines = Vec::new();
        let m_name = Self::name_for_month(month);
        let first_w = Self::first_weekday_for(year, month);
        let num_days = Self::days_in_year_month(year, month);

        if compact {
            lines.push(format!("{:^27}", format!("{} {}", m_name, year)));
            lines.push("Su Mo Tu We Th Fr Sa".to_string());
            let mut row = String::new();
            for _ in 0..first_w {
                row.push_str("   ");
            }
            let mut cur_col = first_w;
            for d in 1..=num_days {
                row.push_str(&format!("{:>2} ", d));
                cur_col += 1;
                if cur_col % 7 == 0 {
                    lines.push(row.trim_end().to_string());
                    row.clear();
                }
            }
            if !row.is_empty() {
                lines.push(row.trim_end().to_string());
            }
        } else {
            lines.push(format!("{:^35}", format!("{} {}", m_name, year)));
            lines.push("Sun  Mon  Tue  Wed  Thu  Fri  Sat".to_string());
            let mut row = String::new();
            for _ in 0..first_w {
                row.push_str("     ");
            }
            let mut cur_col = first_w;
            for d in 1..=num_days {
                row.push_str(&format!("{:>3}  ", d));
                cur_col += 1;
                if cur_col % 7 == 0 {
                    lines.push(row.trim_end().to_string());
                    row.clear();
                }
            }
            if !row.is_empty() {
                lines.push(row.trim_end().to_string());
            }
        }

        while lines.len() < 8 {
            lines.push(String::new());
        }
        lines
    }

    /// Boxed wall calendar grid lines: individual cell boxes, day number in top-left, holiday in bottom-right
    pub fn get_boxed_month_lines(
        year: i32,
        month: u32,
        month_count: u32,
        show_holidays: bool,
        holidays: &[HolidayItem],
        custom_events: &[CustomCalendarEvent],
    ) -> Vec<String> {
        let mut lines = Vec::new();
        let m_name = Self::name_for_month(month);
        let first_w = Self::first_weekday_for(year, month);
        let num_days = Self::days_in_year_month(year, month);

        let cell_inner = match month_count {
            1 => 8,
            2 => 5,
            _ => 4,
        };

        let horiz: String = "─".repeat(cell_inner);
        let top_border = format!("┌{}┐", vec![horiz.clone(); 7].join("┬"));
        let mid_border = format!("├{}┤", vec![horiz.clone(); 7].join("┼"));
        let bot_border = format!("└{}┘", vec![horiz.clone(); 7].join("┴"));

        let days = match month_count {
            1 => ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
            2 => ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"],
            _ => ["S", "M", "T", "W", "T", "F", "S"],
        };

        let day_headers: Vec<String> = days.iter().map(|d| pad_visual_center(d, cell_inner)).collect();
        let day_header_row = format!("│{}│", day_headers.join("│"));

        let title = format!("{} {}", m_name, year);
        lines.push(pad_visual_center(&title, top_border.chars().count()));
        lines.push(top_border);
        lines.push(day_header_row);
        lines.push(mid_border.clone());

        // Build map of day -> holiday badge
        let mut day_badges: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
        if show_holidays {
            for h in holidays {
                if h.enabled && h.month == month {
                    let b = h.short_badge().to_string();
                    day_badges.entry(h.day).and_modify(|existing| {
                        if existing.chars().count() + b.chars().count() + 1 <= cell_inner {
                            *existing = format!("{}/{}", existing, b);
                        }
                    }).or_insert(b);
                }
            }
            for ev in custom_events {
                if ev.month == month {
                    let b = ev.short_badge();
                    day_badges.insert(ev.day, b);
                }
            }
        }

        let mut cur_d = 1;
        for w in 0..6 {
            if cur_d > num_days {
                break;
            }
            let mut num_cells = Vec::new();
            let mut hol_cells = Vec::new();

            for col in 0..7 {
                if (w == 0 && col < first_w) || cur_d > num_days {
                    num_cells.push(" ".repeat(cell_inner));
                    hol_cells.push(" ".repeat(cell_inner));
                } else {
                    // Top line: Day number top-left
                    num_cells.push(pad_visual_left(&format!("{}", cur_d), cell_inner));
                    // Bottom line: Holiday in bottom-right
                    let hol = day_badges.get(&cur_d).cloned().unwrap_or_default();
                    hol_cells.push(pad_visual_right(&hol, cell_inner));
                    cur_d += 1;
                }
            }

            lines.push(format!("│{}│", num_cells.join("│")));
            lines.push(format!("│{}│", hol_cells.join("│")));

            if cur_d <= num_days {
                lines.push(mid_border.clone());
            } else {
                lines.push(bot_border.clone());
            }
        }

        while lines.len() < 17 {
            lines.push(String::new());
        }

        lines
    }

    /// Format multi-month calendar (1, 2, or 3 months) with optional holiday notes and style support
    pub fn format_multi_month_string(
        year: i32,
        start_month: u32,
        month_count: u32,
        show_holidays: bool,
        style: CalendarStyle,
        holidays: &[HolidayItem],
        custom_events: &[CustomCalendarEvent],
    ) -> String {
        let count = month_count.clamp(1, 3);
        let mut holiday_notes = Vec::new();

        for offset in 0..count {
            let m = start_month + offset;
            if m > 12 {
                break;
            }
            if show_holidays {
                for h in holidays {
                    if h.enabled && h.month == m {
                        let prefix = match h.category {
                            HolidayCategory::American => "[US]",
                            HolidayCategory::Chinese => "[CN]",
                            HolidayCategory::Custom => "[Event]",
                        };
                        holiday_notes.push(format!("{} {} {}: {}", prefix, Self::short_name_for_month(m), h.day, h.name));
                    }
                }
                for ev in custom_events {
                    if ev.month == m {
                        holiday_notes.push(format!("[Event] {} {}: {}", Self::short_name_for_month(m), ev.day, ev.label));
                    }
                }
            }
        }

        let mut output = String::new();

        match style {
            CalendarStyle::BoxedGrid => {
                let mut all_month_lines = Vec::new();
                for offset in 0..count {
                    let m = start_month + offset;
                    if m > 12 {
                        break;
                    }
                    all_month_lines.push(Self::get_boxed_month_lines(
                        year,
                        m,
                        count,
                        show_holidays,
                        holidays,
                        custom_events,
                    ));
                }

                let max_rows = all_month_lines.iter().map(|l| l.len()).max().unwrap_or(0);
                let col_sep = "   ";

                for r in 0..max_rows {
                    let mut line = String::new();
                    for (i, m_lines) in all_month_lines.iter().enumerate() {
                        let part = m_lines.get(r).map(|s| s.as_str()).unwrap_or("");
                        if i > 0 {
                            line.push_str(col_sep);
                        }
                        line.push_str(part);
                    }
                    output.push_str(line.trim_end());
                    output.push('\n');
                }
            }
            CalendarStyle::SimpleGrid => {
                let compact = count >= 3;
                let mut all_month_lines = Vec::new();
                for offset in 0..count {
                    let m = start_month + offset;
                    if m > 12 {
                        break;
                    }
                    all_month_lines.push(Self::get_simple_month_grid_lines(year, m, compact));
                }

                let num_rows = 8;
                let col_sep = if compact { "   " } else { "      " };
                let col_width = if compact { 27 } else { 35 };

                for r in 0..num_rows {
                    let mut line = String::new();
                    for (i, m_lines) in all_month_lines.iter().enumerate() {
                        let part = m_lines.get(r).map(|s| s.as_str()).unwrap_or("");
                        if i > 0 {
                            line.push_str(col_sep);
                        }
                        let padded = format!("{:width$}", part, width = col_width);
                        line.push_str(&padded);
                    }
                    output.push_str(line.trim_end());
                    output.push('\n');
                }
            }
        }

        // Add holiday footer footnotes if enabled
        if show_holidays && !holiday_notes.is_empty() {
            output.push('\n');
            let chunk_size = if count == 1 { 2 } else if count == 2 { 3 } else { 4 };
            for chunk in holiday_notes.chunks(chunk_size) {
                output.push_str(&chunk.join("   |   "));
                output.push('\n');
            }
        }

        output.trim_end().to_string()
    }

    /// Format calendar as formatted string table (backward compatible single-month)
    pub fn format_grid_string(&self) -> String {
        Self::format_multi_month_string(self.year, self.month, 1, false, CalendarStyle::SimpleGrid, &[], &[])
    }

    /// Render landscape calendar grid image with boxed day cells and bottom-right colored badges
    pub fn render_multi_grid_image(
        year: i32,
        start_month: u32,
        month_count: u32,
        width: u32,
        height: u32,
        show_holidays: bool,
        holidays: &[HolidayItem],
    ) -> ColorImage {
        let count = month_count.clamp(1, 3);
        let mut img = ImageBuffer::from_pixel(width, height, Rgba([248, 249, 252, 255]));

        let header_h = height / 8;
        for y in 0..header_h {
            for x in 0..width {
                img.put_pixel(x, y, Rgba([30, 41, 59, 255]));
            }
        }

        let footer_h = if show_holidays { height / 10 } else { 0 };
        let grid_top = header_h;
        let grid_bottom = height.saturating_sub(footer_h);
        let available_h = grid_bottom.saturating_sub(grid_top);

        let rows = 6;
        let cols_per_month = 7;
        let month_w = width / count;

        for m_idx in 0..count {
            let m_num = start_month + m_idx;
            if m_num > 12 {
                break;
            }
            let m_x_start = m_idx * month_w;
            let first_w = Self::first_weekday_for(year, m_num);
            let num_days = Self::days_in_year_month(year, m_num);

            let cell_w = (month_w / cols_per_month).max(1);
            let cell_h = (available_h / rows).max(1);

            // Month boundary line
            if m_idx > 0 && m_x_start < width {
                for y in 0..height {
                    img.put_pixel(m_x_start, y, Rgba([148, 163, 184, 255]));
                }
            }

            let mut cur_d = 1;
            for r in 0..rows {
                for c in 0..cols_per_month {
                    let is_active = (r > 0 || c >= first_w as u32) && cur_d <= num_days;
                    let cell_x = m_x_start + c * cell_w;
                    let cell_y = grid_top + r * cell_h;

                    // Box outline borders
                    for px in cell_x..(cell_x + cell_w).min(width) {
                        img.put_pixel(px, cell_y, Rgba([203, 213, 225, 255]));
                        if cell_y + cell_h - 1 < height {
                            img.put_pixel(px, cell_y + cell_h - 1, Rgba([203, 213, 225, 255]));
                        }
                    }
                    for py in cell_y..(cell_y + cell_h).min(height) {
                        img.put_pixel(cell_x, py, Rgba([203, 213, 225, 255]));
                        if cell_x + cell_w - 1 < width {
                            img.put_pixel(cell_x + cell_w - 1, py, Rgba([203, 213, 225, 255]));
                        }
                    }

                    if is_active {
                        // Check if this day is a holiday
                        if show_holidays {
                            if let Some(h) = holidays.iter().find(|h| h.enabled && h.month == m_num && h.day == cur_d) {
                                // Highlight bottom-right badge in cell with holiday color
                                let badge_w = cell_w / 3;
                                let badge_h = cell_h / 4;
                                let badge_x = cell_x + cell_w - badge_w - 4;
                                let badge_y = cell_y + cell_h - badge_h - 4;
                                for by in badge_y..(badge_y + badge_h).min(height) {
                                    for bx in badge_x..(badge_x + badge_w).min(width) {
                                        img.put_pixel(bx, by, Rgba([h.color[0], h.color[1], h.color[2], 230]));
                                    }
                                }
                            }
                        }
                        cur_d += 1;
                    }
                }
            }
        }

        let raw: Vec<Color32> = img.pixels().map(|p| Color32::from_rgba_premultiplied(p[0], p[1], p[2], p[3])).collect();
        ColorImage {
            size: [width as usize, height as usize],
            pixels: raw,
        }
    }
}

/// Preset positioning for calendar elements on slides
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CalendarPositionPreset {
    LeftColumn,    // Left 50% column (Default): x=0.03, y=0.04, w=0.46, h=0.92
    RightColumn,   // Right 50% column: x=0.51, y=0.04, w=0.46, h=0.92
    BottomHalf,    // Bottom 50% row: x=0.05, y=0.52, w=0.90, h=0.44
    TopHalf,       // Top 50% row: x=0.05, y=0.04, w=0.90, h=0.44
    TopLeft,       // Top-Left quadrant: x=0.03, y=0.04, w=0.46, h=0.44
    TopRight,      // Top-Right quadrant: x=0.51, y=0.04, w=0.46, h=0.44
    BottomLeft,    // Bottom-Left quadrant: x=0.03, y=0.52, w=0.46, h=0.44
    BottomRight,   // Bottom-Right quadrant: x=0.51, y=0.52, w=0.46, h=0.44
    FullSlide,     // Full slide: x=0.03, y=0.04, w=0.94, h=0.92
}

impl Default for CalendarPositionPreset {
    fn default() -> Self {
        Self::LeftColumn
    }
}

impl CalendarPositionPreset {
    pub const ALL: &'static [CalendarPositionPreset] = &[
        CalendarPositionPreset::LeftColumn,
        CalendarPositionPreset::RightColumn,
        CalendarPositionPreset::BottomHalf,
        CalendarPositionPreset::TopHalf,
        CalendarPositionPreset::TopLeft,
        CalendarPositionPreset::TopRight,
        CalendarPositionPreset::BottomLeft,
        CalendarPositionPreset::BottomRight,
        CalendarPositionPreset::FullSlide,
    ];

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::LeftColumn => (0.03, 0.04, 0.46, 0.92),
            Self::RightColumn => (0.51, 0.04, 0.46, 0.92),
            Self::BottomHalf => (0.05, 0.52, 0.90, 0.44),
            Self::TopHalf => (0.05, 0.04, 0.90, 0.44),
            Self::TopLeft => (0.03, 0.04, 0.46, 0.44),
            Self::TopRight => (0.51, 0.04, 0.46, 0.44),
            Self::BottomLeft => (0.03, 0.52, 0.46, 0.44),
            Self::BottomRight => (0.51, 0.52, 0.46, 0.44),
            Self::FullSlide => (0.03, 0.04, 0.94, 0.92),
        }
    }

    /// Complementary media/photo placeholder bounds for this calendar preset
    pub fn complementary_photo_bounds(&self) -> Option<(f32, f32, f32, f32)> {
        match self {
            Self::LeftColumn => Some((0.51, 0.04, 0.46, 0.92)),
            Self::RightColumn => Some((0.03, 0.04, 0.46, 0.92)),
            Self::BottomHalf => Some((0.05, 0.04, 0.90, 0.44)),
            Self::TopHalf => Some((0.05, 0.52, 0.90, 0.44)),
            Self::TopLeft => Some((0.51, 0.04, 0.46, 0.92)),
            Self::TopRight => Some((0.03, 0.04, 0.46, 0.92)),
            Self::BottomLeft => Some((0.51, 0.04, 0.46, 0.92)),
            Self::BottomRight => Some((0.03, 0.04, 0.46, 0.92)),
            Self::FullSlide => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::LeftColumn => "Left 50% (Default)",
            Self::RightColumn => "Right 50%",
            Self::BottomHalf => "Bottom 50%",
            Self::TopHalf => "Top 50%",
            Self::TopLeft => "Top Left",
            Self::TopRight => "Top Right",
            Self::BottomLeft => "Bottom Left",
            Self::BottomRight => "Bottom Right",
            Self::FullSlide => "Full Slide",
        }
    }

    pub fn button_title(&self) -> &'static str {
        match self {
            Self::LeftColumn => "◧ Left 50% (Default)",
            Self::RightColumn => "◨ Right 50%",
            Self::BottomHalf => "⬓ Bottom 50%",
            Self::TopHalf => "⬒ Top 50%",
            Self::TopLeft => "◰ Top Left",
            Self::TopRight => "◳ Top Right",
            Self::BottomLeft => "◱ Bottom Left",
            Self::BottomRight => "◲ Bottom Right",
            Self::FullSlide => "⏹ Full Slide",
        }
    }
}

/// Interactive vector graphical calendar element configuration
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CalendarOverlay {
    pub year: i32,
    pub start_month: u32,
    #[serde(default = "default_month_count")]
    pub month_count: u32,
    #[serde(default = "default_true")]
    pub show_holidays: bool,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    #[serde(default)]
    pub holidays: Vec<HolidayItem>,
    #[serde(default)]
    pub custom_events: Vec<CustomCalendarEvent>,
}

fn default_month_count() -> u32 {
    1
}

fn default_true() -> bool {
    true
}

impl Default for CalendarOverlay {
    fn default() -> Self {
        Self {
            year: 2026,
            start_month: 1,
            month_count: 1,
            show_holidays: true,
            x: 0.03,
            y: 0.04,
            w: 0.46,
            h: 0.92,
            holidays: Vec::new(),
            custom_events: Vec::new(),
        }
    }
}
