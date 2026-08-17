use crate::core::text_overlay::{TextAlignment, TextBoxStyle, TextOverlay};
use egui::{Align2, Color32, FontFamily, FontId, Pos2, Rect, Rounding, Vec2};
use std::sync::Arc;

pub struct TextRenderer;

impl TextRenderer {
    pub fn draw_text_overlay(painter: &egui::Painter, rect: Rect, overlay: &TextOverlay) {
        let raw_text = overlay.formatted_text();
        let is_empty = raw_text.trim().is_empty();
        let display_text = if is_empty { "Type text here..." } else { raw_text.as_str() };
        let lines: Vec<&str> = display_text.lines().collect();
        if lines.is_empty() {
            return;
        }

        let scale = (rect.height() / 400.0).clamp(0.6, 2.5);
        let font_size = (overlay.font_size * scale * 0.55).max(12.0);
        let family = FontFamily::Name(Arc::from(overlay.font_family.preview_family()));
        let font_id = FontId::new(font_size, family);
        let text_color = if is_empty { Color32::from_gray(140) } else { overlay.text_color };

        let line_galleys: Vec<_> = lines
            .iter()
            .map(|l| painter.layout_no_wrap(l.to_string(), font_id.clone(), text_color))
            .collect();
        let max_line_w = line_galleys
            .iter()
            .map(|g| g.size().x)
            .fold(0.0f32, |a, b| a.max(b));
        let total_text_h = line_galleys
            .iter()
            .map(|g| g.size().y)
            .sum::<f32>()
            + ((line_galleys.len().saturating_sub(1)) as f32 * 4.0 * scale);

        let pad_x = 20.0 * scale;
        let pad_y = 10.0 * scale;
        let anchor = rect.min
            + Vec2::new(overlay.x * rect.width(), overlay.y * rect.height());

        // Background box
        let box_rect = match overlay.box_style {
            TextBoxStyle::None => Rect::NOTHING,
            TextBoxStyle::TranslucentBox => Rect::from_center_size(
                anchor,
                Vec2::new(max_line_w + pad_x * 2.0, total_text_h + pad_y * 2.0),
            ),
            TextBoxStyle::SolidBanner => Rect::from_min_max(
                Pos2::new(rect.min.x, anchor.y - total_text_h / 2.0 - pad_y),
                Pos2::new(rect.max.x, anchor.y + total_text_h / 2.0 + pad_y),
            ),
        };
        if overlay.box_style != TextBoxStyle::None {
            let alpha = ((overlay.box_opacity * 255.0).clamp(10.0, 255.0)) as u8;
            painter.rect_filled(box_rect, Rounding::same(6.0), Color32::from_black_alpha(alpha));
            painter.rect_stroke(
                box_rect,
                Rounding::same(6.0),
                egui::Stroke::new(1.0, Color32::from_white_alpha(30)),
            );
        }

        let mut cur_y = anchor.y - total_text_h / 2.0;
        let shadow_offset = Vec2::new(1.5 * scale, 1.5 * scale);

        for (i, line) in lines.iter().enumerate() {
            let line_w = line_galleys[i].size().x;
            let line_h = line_galleys[i].size().y;
            let line_x = match overlay.alignment {
                TextAlignment::Left => anchor.x - max_line_w / 2.0 + line_w / 2.0,
                TextAlignment::Center => anchor.x,
                TextAlignment::Right => anchor.x + max_line_w / 2.0 - line_w / 2.0,
            };
            let line_pos = Pos2::new(line_x, cur_y + line_h / 2.0);

            if overlay.show_shadow {
                painter.text(
                    line_pos + shadow_offset,
                    Align2::CENTER_CENTER,
                    *line,
                    font_id.clone(),
                    Color32::from_black_alpha(220),
                );
            }
            painter.text(
                line_pos,
                Align2::CENTER_CENTER,
                *line,
                font_id.clone(),
                text_color,
            );
            cur_y += line_h + 4.0 * scale;
        }
    }
}
