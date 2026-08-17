use crate::VideoEditorApp;
use crate::core::clip::Clip;
use crate::core::text_overlay::{SlideBackground, SlideElement};
use crate::core::track::TrackKind;
use crate::ui::theme::AppTheme;
use egui::{Button, Color32, Frame, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use std::path::PathBuf;

/// Actions emitted by the Slide Deck filmstrip panel.
pub enum SlideDeckAction {
    None,
    SelectSlide(u64),
    AddBlankSlide { duration: f64 },
    DuplicateSlide(u64),
    DeleteSlide(u64),
    MoveSlideUp(usize),
    MoveSlideDown(usize),
    AdjustSlideDuration { clip_id: u64, delta_secs: f64 },
    DropFilesOnSlide { clip_id: u64, paths: Vec<PathBuf> },
}

pub struct SlideDeckView;

impl SlideDeckView {
    pub fn render(ui: &mut Ui, app: &mut VideoEditorApp) -> SlideDeckAction {
        let mut action = SlideDeckAction::None;

        let video_track = app
            .project
            .timeline
            .tracks
            .iter()
            .find(|t| t.kind == TrackKind::Video)
            .cloned();

        let clips: Vec<Clip> = match video_track {
            Some(t) => t.clips,
            None => Vec::new(),
        };

        ui.vertical(|ui| {
            // Header: Slide Deck Title & Add Button
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("🖼 Slide Deck ({})", clips.len()))
                        .size(13.5)
                        .strong()
                        .color(AppTheme::accent_yellow()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            Button::new(RichText::new("➕ New Slide").size(12.0).strong().color(Color32::WHITE))
                                .fill(AppTheme::accent_green())
                                .min_size(egui::vec2(80.0, 26.0)),
                        )
                        .on_hover_text("Add a new blank slide to the slideshow")
                        .clicked()
                    {
                        action = SlideDeckAction::AddBlankSlide { duration: 5.0 };
                    }
                });
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            if clips.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(30.0);
                    ui.label(RichText::new("No slides created yet").size(13.0).color(AppTheme::text_secondary()));
                    ui.add_space(8.0);
                    if ui
                        .add(
                            Button::new(RichText::new("➕ Create First Slide").size(13.0).strong().color(Color32::WHITE))
                                .fill(AppTheme::accent_green())
                                .min_size(egui::vec2(160.0, 32.0)),
                        )
                        .clicked()
                    {
                        action = SlideDeckAction::AddBlankSlide { duration: 5.0 };
                    }
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("Or drag photos/videos directly from File Explorer onto this panel!")
                            .size(11.0)
                            .color(AppTheme::text_muted()),
                    );
                });
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let active_id = app.active_slide().map(|c| c.id);

                        for (idx, clip) in clips.iter().enumerate() {
                            let is_active = Some(clip.id) == active_id || clip.is_selected;
                            let slide_num = idx + 1;

                            let card_action = Self::render_slide_card(
                                ui,
                                idx,
                                slide_num,
                                clip,
                                is_active,
                                clips.len(),
                            );

                            if !matches!(card_action, SlideDeckAction::None) {
                                action = card_action;
                            }

                            ui.add_space(6.0);
                        }
                    });
            }
        });

        action
    }

    fn render_slide_card(
        ui: &mut Ui,
        idx: usize,
        slide_num: usize,
        clip: &Clip,
        is_active: bool,
        total_slides: usize,
    ) -> SlideDeckAction {
        let mut action = SlideDeckAction::None;

        let border_color = if is_active {
            AppTheme::accent_cyan()
        } else {
            Color32::from_white_alpha(30)
        };

        let bg_color = if is_active {
            Color32::from_rgb(26, 32, 44)
        } else {
            AppTheme::bg_card()
        };

        Frame::none()
            .fill(bg_color)
            .rounding(Rounding::same(6.0))
            .stroke(Stroke::new(if is_active { 2.0 } else { 1.0 }, border_color))
            .inner_margin(egui::Margin::same(6.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // 1. Slide Number Badge
                    let num_label = Button::new(
                        RichText::new(format!("{:02}", slide_num))
                            .size(13.0)
                            .strong()
                            .color(if is_active { Color32::WHITE } else { AppTheme::text_secondary() }),
                    )
                    .fill(if is_active { AppTheme::accent_blue() } else { AppTheme::bg_panel() })
                    .min_size(Vec2::new(26.0, 48.0));

                    if ui.add(num_label).on_hover_text("Click to select this slide").clicked() {
                        action = SlideDeckAction::SelectSlide(clip.id);
                    }

                    ui.add_space(4.0);

                    // 2. Visual Miniature Thumbnail of Slide
                    let thumb_size = Vec2::new(76.0, 48.0);
                    let (rect, response) = ui.allocate_exact_size(thumb_size, Sense::click());

                    let painter = ui.painter();
                    // Background color fill
                    let bg_fill = match &clip.background {
                        Some(SlideBackground::Solid(c)) => *c,
                        _ => Color32::from_rgb(18, 20, 24),
                    };
                    painter.rect_filled(rect, Rounding::same(3.0), bg_fill);
                    painter.rect_stroke(rect, Rounding::same(3.0), Stroke::new(1.0, Color32::from_white_alpha(40)));

                    // Miniature element badges/icons on thumbnail
                    let num_elements = clip.elements.len();
                    if num_elements > 0 {
                        let mut icon_summary = String::new();
                        for el in &clip.elements {
                            match el {
                                SlideElement::Text(_) => icon_summary.push_str("🔤"),
                                SlideElement::Calendar(_) => icon_summary.push_str("📅"),
                                SlideElement::Picture { .. } => icon_summary.push_str("🖼"),
                                SlideElement::Video { .. } => icon_summary.push_str("🎬"),
                                SlideElement::Audio { .. } => icon_summary.push_str("🎵"),
                                SlideElement::Placeholder { .. } => icon_summary.push_str("➕"),
                            }
                        }
                        painter.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            icon_summary,
                            egui::FontId::proportional(10.0),
                            Color32::WHITE,
                        );
                    }

                    if response.clicked() {
                        action = SlideDeckAction::SelectSlide(clip.id);
                    }

// (OS file drag-and-drop handled globally at top-level update loop in app.rs)

                    ui.add_space(4.0);

                    // 3. Slide Metadata & Controls
                    ui.vertical(|ui| {
                        let title_resp = ui.add(
                            egui::Label::new(
                                RichText::new(&clip.name)
                                    .size(12.0)
                                    .strong()
                                    .color(if is_active { AppTheme::accent_yellow() } else { Color32::WHITE }),
                            )
                            .sense(Sense::click()),
                        );
                        if title_resp.clicked() {
                            action = SlideDeckAction::SelectSlide(clip.id);
                        }

                        // Duration Stepper (- / +)
                        ui.horizontal(|ui| {
                            let dur_secs = clip.duration().as_secs_f64();
                            ui.label(RichText::new(format!("⏱ {:.1}s", dur_secs)).size(11.0).color(AppTheme::text_secondary()));

                            if ui.button(RichText::new("-").size(10.0)).on_hover_text("Shorten slide by 0.5s").clicked() {
                                action = SlideDeckAction::AdjustSlideDuration {
                                    clip_id: clip.id,
                                    delta_secs: -0.5,
                                };
                            }
                            if ui.button(RichText::new("+").size(10.0)).on_hover_text("Extend slide by 0.5s").clicked() {
                                action = SlideDeckAction::AdjustSlideDuration {
                                    clip_id: clip.id,
                                    delta_secs: 0.5,
                                };
                            }
                        });

                        // Reorder & Action buttons
                        ui.horizontal(|ui| {
                            if idx > 0 {
                                if ui.button("▲").on_hover_text("Move slide earlier in show").clicked() {
                                    action = SlideDeckAction::MoveSlideUp(idx);
                                }
                            }
                            if idx + 1 < total_slides {
                                if ui.button("▼").on_hover_text("Move slide later in show").clicked() {
                                    action = SlideDeckAction::MoveSlideDown(idx);
                                }
                            }
                            if ui.button("📋").on_hover_text("Duplicate this slide").clicked() {
                                action = SlideDeckAction::DuplicateSlide(clip.id);
                            }
                            if ui.button("🗑").on_hover_text("Delete this slide").clicked() {
                                action = SlideDeckAction::DeleteSlide(clip.id);
                            }
                        });
                    });
                });
            });

        action
    }

    /// Renders a horizontal filmstrip tray of slides for Slideshow Studio mode.
    pub fn render_horizontal_filmstrip(ui: &mut Ui, app: &mut VideoEditorApp) -> SlideDeckAction {
        let mut action = SlideDeckAction::None;

        let video_track = app
            .project
            .timeline
            .tracks
            .iter()
            .find(|t| t.kind == TrackKind::Video)
            .cloned();

        let clips: Vec<Clip> = match video_track {
            Some(t) => t.clips,
            None => Vec::new(),
        };

        if clips.is_empty() {
            ui.horizontal_centered(|ui| {
                ui.add_space(20.0);
                ui.label(RichText::new("No slides yet.").size(13.0).color(AppTheme::text_secondary()));
                if ui
                    .add(
                        Button::new(RichText::new("➕ Create First Slide").size(12.5).strong().color(Color32::WHITE))
                            .fill(AppTheme::accent_green())
                            .min_size(Vec2::new(150.0, 32.0)),
                    )
                    .clicked()
                {
                    action = SlideDeckAction::AddBlankSlide { duration: 5.0 };
                }
            });
            return action;
        }

        let active_id = app.active_slide().map(|c| c.id);
        let total_slides = clips.len();

        egui::ScrollArea::horizontal()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(4.0);

                    for (idx, clip) in clips.iter().enumerate() {
                        let is_active = Some(clip.id) == active_id || clip.is_selected;
                        let slide_num = idx + 1;

                        let card_action = Self::render_horizontal_slide_card(
                            ui,
                            app,
                            idx,
                            slide_num,
                            clip,
                            is_active,
                            total_slides,
                        );

                        if !matches!(card_action, SlideDeckAction::None) {
                            action = card_action;
                        }

                        ui.add_space(8.0);
                    }

                    // "+ Add Slide" Quick Card at the end of the filmstrip
                    let add_card_size = Vec2::new(120.0, 112.0);
                    let (add_rect, add_resp) = ui.allocate_exact_size(add_card_size, Sense::click());
                    let painter = ui.painter();
                    let is_hovered = add_resp.hovered();

                    let add_bg = if is_hovered {
                        Color32::from_rgba_premultiplied(35, 55, 80, 200)
                    } else {
                        Color32::from_rgba_premultiplied(22, 26, 36, 160)
                    };

                    let add_stroke = if is_hovered {
                        Stroke::new(1.8, AppTheme::accent_yellow())
                    } else {
                        Stroke::new(1.2, Color32::from_white_alpha(50))
                    };

                    painter.rect_filled(add_rect, Rounding::same(6.0), add_bg);
                    painter.rect_stroke(add_rect, Rounding::same(6.0), add_stroke);

                    painter.text(
                        add_rect.center() - Vec2::new(0.0, 10.0),
                        egui::Align2::CENTER_CENTER,
                        "➕",
                        egui::FontId::proportional(22.0),
                        if is_hovered { AppTheme::accent_yellow() } else { AppTheme::accent_cyan() },
                    );

                    painter.text(
                        add_rect.center() + Vec2::new(0.0, 14.0),
                        egui::Align2::CENTER_CENTER,
                        "Add Slide",
                        egui::FontId::proportional(12.0),
                        if is_hovered { Color32::WHITE } else { AppTheme::text_secondary() },
                    );

                    if add_resp.clicked() {
                        action = SlideDeckAction::AddBlankSlide { duration: 5.0 };
                    }

                    ui.add_space(8.0);
                });
            });

        action
    }

    fn render_horizontal_slide_card(
        ui: &mut Ui,
        app: &VideoEditorApp,
        idx: usize,
        slide_num: usize,
        clip: &Clip,
        is_active: bool,
        total_slides: usize,
    ) -> SlideDeckAction {
        let mut action = SlideDeckAction::None;

        let card_w = 160.0;
        let card_h = 112.0;

        let border_stroke = if is_active {
            Stroke::new(2.5, AppTheme::accent_yellow())
        } else {
            Stroke::new(1.0, Color32::from_rgb(45, 50, 65))
        };

        let card_bg = if is_active {
            Color32::from_rgb(26, 36, 54)
        } else {
            AppTheme::bg_card()
        };

        Frame::none()
            .fill(card_bg)
            .rounding(Rounding::same(6.0))
            .stroke(border_stroke)
            .inner_margin(egui::Margin::symmetric(6.0, 5.0))
            .show(ui, |ui| {
                ui.set_width(card_w);
                ui.set_height(card_h);

                ui.vertical(|ui| {
                    // 1. Top Header Row: Slide # badge, duration, and quick reorder
                    ui.horizontal(|ui| {
                        let num_badge = Button::new(
                            RichText::new(format!("#{}", slide_num))
                                .size(11.5)
                                .strong()
                                .color(if is_active { Color32::WHITE } else { AppTheme::text_secondary() }),
                        )
                        .fill(if is_active { AppTheme::accent_blue() } else { Color32::from_rgb(20, 24, 32) })
                        .min_size(Vec2::new(28.0, 20.0));

                        if ui.add(num_badge).clicked() {
                            action = SlideDeckAction::SelectSlide(clip.id);
                        }

                        ui.label(
                            RichText::new(format!("{:.1}s", clip.duration().as_secs_f64()))
                                .size(11.0)
                                .color(AppTheme::text_secondary()),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(Button::new(RichText::new("×").size(12.0).color(Color32::from_rgb(255, 140, 140))).min_size(Vec2::new(16.0, 16.0)))
                                .on_hover_text("Delete slide")
                                .clicked()
                            {
                                action = SlideDeckAction::DeleteSlide(clip.id);
                            }

                            if idx + 1 < total_slides {
                                if ui.add(Button::new("▶").min_size(Vec2::new(16.0, 16.0))).on_hover_text("Move right").clicked() {
                                    action = SlideDeckAction::MoveSlideDown(idx);
                                }
                            }
                            if idx > 0 {
                                if ui.add(Button::new("◀").min_size(Vec2::new(16.0, 16.0))).on_hover_text("Move left").clicked() {
                                    action = SlideDeckAction::MoveSlideUp(idx);
                                }
                            }
                        });
                    });

                    ui.add_space(2.0);

                    // 2. 16:9 Aspect Ratio Thumbnail Box (148 x 72 px)
                    let thumb_size = Vec2::new(card_w - 12.0, 72.0);
                    let (thumb_rect, thumb_resp) = ui.allocate_exact_size(thumb_size, Sense::click());
                    let painter = ui.painter();

                    let bg_color = match &clip.background {
                        Some(SlideBackground::Solid(c)) => *c,
                        _ => Color32::from_rgb(18, 22, 28),
                    };
                    painter.rect_filled(thumb_rect, Rounding::same(4.0), bg_color);
                    painter.rect_stroke(thumb_rect, Rounding::same(4.0), Stroke::new(1.0, Color32::from_white_alpha(35)));

                    // If the slide has pictures and cached texture, draw the primary photo on thumbnail
                    let mut drawn_pic = false;
                    for el in &clip.elements {
                        if let SlideElement::Picture { path, .. } = el {
                            if let Some(tex) = app.slide_textures.get(path) {
                                painter.image(
                                    tex.id(),
                                    thumb_rect,
                                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0)),
                                    Color32::WHITE,
                                );
                                drawn_pic = true;
                                break;
                            }
                        }
                    }

                    // Element badges overlay on thumbnail
                    let elem_count = clip.elements.len();
                    if elem_count > 0 && !drawn_pic {
                        let mut badge_text = String::new();
                        for el in &clip.elements {
                            match el {
                                SlideElement::Text(t) => {
                                    if !t.text.trim().is_empty() && badge_text.len() < 14 {
                                        badge_text = format!("🔤 {}", t.text.lines().next().unwrap_or(""));
                                    }
                                }
                                SlideElement::Calendar(c) => {
                                    badge_text = format!("📅 Calendar {}", c.year);
                                }
                                SlideElement::Picture { .. } => {
                                    if badge_text.is_empty() {
                                        badge_text = "🖼 Photo Slide".to_string();
                                    }
                                }
                                SlideElement::Video { .. } => {
                                    badge_text = "🎬 Video Slide".to_string();
                                }
                                SlideElement::Placeholder { label, .. } => {
                                    badge_text = format!("➕ {}", label);
                                }
                                _ => {}
                            }
                        }

                        if badge_text.is_empty() {
                            badge_text = format!("{} Elements", elem_count);
                        }

                        painter.text(
                            thumb_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            badge_text,
                            egui::FontId::proportional(11.0),
                            Color32::WHITE,
                        );
                    } else if elem_count == 0 && !drawn_pic {
                        painter.text(
                            thumb_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "Blank Slide",
                            egui::FontId::proportional(10.5),
                            AppTheme::text_muted(),
                        );
                    }

                    if thumb_resp.clicked() {
                        action = SlideDeckAction::SelectSlide(clip.id);
                    }
                });
            });

        action
    }

}
