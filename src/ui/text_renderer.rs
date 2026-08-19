use crate::core::text_overlay::{TextAlignment, TextBoxStyle, TextOverlay};
use egui::{Color32, FontFamily, FontId, Pos2, Rect, Rounding, Vec2};
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

        let scale = (rect.height() / 400.0).clamp(0.12, 2.5);
        let font_size = (overlay.font_size * scale * 0.55).max(6.0);
        let family = FontFamily::Name(Arc::from(overlay.font_family.preview_family()));
        let font_id = FontId::new(font_size, family);
        let text_color = if is_empty { Color32::from_gray(140) } else { overlay.text_color };

        let line_galleys: Vec<_> = lines
            .iter()
            .map(|l| {
                let mut job = egui::text::LayoutJob::default();
                job.append(
                    l,
                    0.0,
                    egui::TextFormat {
                        font_id: font_id.clone(),
                        color: text_color,
                        italics: overlay.is_italic,
                        ..Default::default()
                    },
                );
                painter.layout_job(job)
            })
            .collect();

        let shadow_galleys: Option<Vec<_>> = if overlay.show_shadow {
            Some(
                lines
                    .iter()
                    .map(|l| {
                        let mut job = egui::text::LayoutJob::default();
                        job.append(
                            l,
                            0.0,
                            egui::TextFormat {
                                font_id: font_id.clone(),
                                color: Color32::from_black_alpha(220),
                                italics: overlay.is_italic,
                                ..Default::default()
                            },
                        );
                        painter.layout_job(job)
                    })
                    .collect(),
            )
        } else {
            None
        };

        let max_line_w = line_galleys
            .iter()
            .map(|g| g.size().x)
            .fold(0.0f32, |a, b| a.max(b));
        let total_text_h = line_galleys
            .iter()
            .map(|g| g.size().y)
            .sum::<f32>()
            + ((line_galleys.len().saturating_sub(1)) as f32 * 4.0 * scale);

        let pad_x = (20.0 * scale).max(2.0);
        let pad_y = (10.0 * scale).max(1.0);
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
            painter.rect_filled(box_rect, Rounding::same((6.0 * scale).max(2.0)), Color32::from_black_alpha(alpha));
            painter.rect_stroke(
                box_rect,
                Rounding::same((6.0 * scale).max(2.0)),
                egui::Stroke::new(1.0, Color32::from_white_alpha((alpha / 4).max(20))),
            );
        }

        let mut cur_y = anchor.y - total_text_h / 2.0;
        let shadow_offset = Vec2::new((1.5 * scale).max(0.5), (1.5 * scale).max(0.5));
        let bold_dx = (font_size * 0.038).clamp(0.4, 2.2);
        let bold_dy = (font_size * 0.016).clamp(0.2, 1.0);

        for (i, galley) in line_galleys.iter().enumerate() {
            let line_w = galley.size().x;
            let line_h = galley.size().y;
            let line_left_x = match overlay.alignment {
                TextAlignment::Left => anchor.x - max_line_w / 2.0,
                TextAlignment::Center => anchor.x - line_w / 2.0,
                TextAlignment::Right => anchor.x + max_line_w / 2.0 - line_w,
            };
            let line_pos = Pos2::new(line_left_x, cur_y);

            // Draw shadow if enabled
            if let Some(shadows) = &shadow_galleys {
                let s_galley = &shadows[i];
                let s_pos = line_pos + shadow_offset;
                painter.galley(s_pos, s_galley.clone(), Color32::BLACK);
                if overlay.is_bold {
                    painter.galley(s_pos + Vec2::new(bold_dx, 0.0), s_galley.clone(), Color32::BLACK);
                    painter.galley(s_pos + Vec2::new(bold_dx * 0.5, bold_dy), s_galley.clone(), Color32::BLACK);
                    painter.galley(s_pos + Vec2::new(0.0, bold_dy * 0.5), s_galley.clone(), Color32::BLACK);
                }
            }

            // Draw primary text with synthetic bold overstrikes if is_bold is active
            painter.galley(line_pos, galley.clone(), text_color);
            if overlay.is_bold {
                painter.galley(line_pos + Vec2::new(bold_dx, 0.0), galley.clone(), text_color);
                painter.galley(line_pos + Vec2::new(bold_dx * 0.5, bold_dy), galley.clone(), text_color);
                painter.galley(line_pos + Vec2::new(0.0, bold_dy * 0.5), galley.clone(), text_color);
            }

            cur_y += line_h + 4.0 * scale;
        }
    }
}
