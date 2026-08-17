use crate::VideoEditorApp;
use crate::core::clip::Clip;
use crate::core::text_overlay::{SlideBackground, SlideElement};
use crate::core::track::TrackKind;
use crate::ui::SlideReorderDrag;
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
    /// Drag-and-drop reorder: move the slide at `from_idx` into the insertion
    /// gap `to_gap` (0..=len), where gap `g` means "land between the slides
    /// currently at g-1 and g". The handler converts the gap to a final index.
    ReorderSlideToGap { from_idx: usize, to_gap: usize },
    AdjustSlideDuration { clip_id: u64, delta_secs: f64 },
    DropFilesOnSlide { clip_id: u64, paths: Vec<PathBuf> },
}

/// Convert a drop *gap* into the index the slide ends up at after the move.
///
/// A gap is a slot *between* cards: gap `g` in a deck of `len` slides means
/// "land between the slides currently at `g - 1` and `g`", so gaps run `0..=len`.
/// Reordering is `remove(from)` then `insert(target)`; removing first shifts
/// every later element down by one, so a gap to the right of `from` overshoots
/// by exactly one and must be decremented. Returns `None` when the move is a
/// no-op (dropping a slide back into either gap flanking its own position),
/// which keeps a stray click-drag from pushing an undo snapshot.
pub fn gap_to_target_index(from_idx: usize, to_gap: usize, len: usize) -> Option<usize> {
    if from_idx >= len {
        return None;
    }
    // Both gaps touching the dragged card leave the deck exactly as it was.
    if to_gap == from_idx || to_gap == from_idx + 1 {
        return None;
    }
    let target = if to_gap > from_idx { to_gap - 1 } else { to_gap };
    Some(target.min(len - 1))
}

pub struct SlideDeckView;

impl SlideDeckView {
    /// Paint a real miniature of the slide into `rect`: background first, then
    /// every element mapped through its normalized (0..1) canvas coordinates.
    /// Returns true if at least one element was drawn, so callers can fall back
    /// to a text badge for slides whose textures haven't loaded yet.
    fn paint_slide_mini(
        painter: &egui::Painter,
        rect: egui::Rect,
        clip: &Clip,
        app: &VideoEditorApp,
    ) -> bool {
        let uv = egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1.0, 1.0));

        // Background: solid color, picture texture, or the dark canvas default.
        match &clip.background {
            Some(SlideBackground::Solid(c)) => {
                painter.rect_filled(rect, Rounding::same(3.0), *c);
            }
            Some(SlideBackground::Picture(p)) => {
                if let Some(tex) = app.slide_textures.get(p) {
                    painter.image(tex.id(), rect, uv, Color32::WHITE);
                } else {
                    painter.rect_filled(rect, Rounding::same(3.0), Color32::from_rgb(18, 20, 24));
                }
            }
            None => {
                painter.rect_filled(rect, Rounding::same(3.0), Color32::from_rgb(18, 20, 24));
            }
        }

        // Map normalized slide coordinates into the thumbnail rect.
        let sub = |x: f32, y: f32, w: f32, h: f32| {
            egui::Rect::from_min_size(
                rect.min + Vec2::new(x * rect.width(), y * rect.height()),
                Vec2::new(w * rect.width(), h * rect.height()),
            )
        };

        let mut drew = false;
        for el in &clip.elements {
            match el {
                SlideElement::Picture { path, x, y, w, h }
                | SlideElement::Sticker { path, x, y, w, h, .. }
                | SlideElement::Video { path, x, y, w, h } => {
                    let r = sub(*x, *y, *w, *h).intersect(rect);
                    if let Some(tex) = app.slide_textures.get(path) {
                        painter.image(tex.id(), r, uv, Color32::WHITE);
                    } else {
                        // Texture still loading: a placeholder block keeps the
                        // layout honest instead of showing nothing.
                        painter.rect_filled(r, Rounding::same(2.0), Color32::from_rgb(40, 44, 54));
                    }
                    drew = true;
                }
                SlideElement::Text(o) => {
                    if !o.text.trim().is_empty() {
                        // Text x/y anchor the CENTER of the text box on the canvas.
                        let pos = rect.min + Vec2::new(o.x * rect.width(), o.y * rect.height());
                        let scaled = (o.font_size * rect.height() / 360.0).clamp(6.0, 16.0);
                        painter.text(
                            pos,
                            egui::Align2::CENTER_CENTER,
                            o.text.lines().next().unwrap_or(""),
                            egui::FontId::proportional(scaled),
                            o.text_color,
                        );
                        drew = true;
                    }
                }
                SlideElement::Calendar(c) => {
                    let r = sub(c.x, c.y, c.w, c.h).intersect(rect);
                    painter.rect_filled(r, Rounding::same(2.0), Color32::from_white_alpha(20));
                    painter.text(
                        r.center(),
                        egui::Align2::CENTER_CENTER,
                        "📅",
                        egui::FontId::proportional((r.height() * 0.6).clamp(8.0, 18.0)),
                        Color32::WHITE,
                    );
                    drew = true;
                }
                SlideElement::Placeholder { x, y, w, h, .. } => {
                    let r = sub(*x, *y, *w, *h).intersect(rect);
                    painter.rect_stroke(r, Rounding::same(2.0), Stroke::new(1.0, Color32::from_white_alpha(60)));
                }
                SlideElement::Audio { .. } => {}
            }
        }

        painter.rect_stroke(rect, Rounding::same(3.0), Stroke::new(1.0, Color32::from_white_alpha(40)));
        drew
    }

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
                                app,
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
        app: &VideoEditorApp,
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
                    let drew = Self::paint_slide_mini(painter, rect, clip, app);

                    // Miniature element badges/icons on thumbnail (fallback while textures load)
                    let num_elements = clip.elements.len();
                    if num_elements > 0 && !drew {
                        let mut icon_summary = String::new();
                        for el in &clip.elements {
                            match el {
                                SlideElement::Text(_) => icon_summary.push_str("🔤"),
                                SlideElement::Calendar(_) => icon_summary.push_str("📅"),
                                SlideElement::Picture { .. } => icon_summary.push_str("🖼"),
                                SlideElement::Sticker { .. } => icon_summary.push_str("🎀"),
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

                    // Which slide (if any) is mid-drag right now. Used to dim the
                    // card being carried and to arm the drop-indicator line.
                    let dragging_idx =
                        egui::DragAndDrop::payload::<SlideReorderDrag>(ui.ctx()).map(|p| p.0);

                    // Card rects, collected in layout order so the drop gap can be
                    // resolved from the pointer's x position after the row is built.
                    let mut card_rects: Vec<egui::Rect> = Vec::with_capacity(clips.len());

                    for (idx, clip) in clips.iter().enumerate() {
                        let is_active = Some(clip.id) == active_id || clip.is_selected;
                        let slide_num = idx + 1;

                        let (card_action, card_rect) = Self::render_horizontal_slide_card(
                            ui,
                            app,
                            idx,
                            slide_num,
                            clip,
                            is_active,
                            total_slides,
                            dragging_idx,
                        );
                        card_rects.push(card_rect);

                        if !matches!(card_action, SlideDeckAction::None) {
                            action = card_action;
                        }

                        ui.add_space(8.0);
                    }

                    // Resolve the drop: map pointer-x onto an insertion gap by
                    // comparing against each card's centre, so releasing over a
                    // card's left half lands before it and the right half after.
                    // Dropping past the last card appends, which is how a slide
                    // reaches the end of the deck in a single gesture.
                    if let Some(from_idx) = dragging_idx {
                        let pointer = ui.input(|i| i.pointer.interact_pos());
                        if let Some(pos) = pointer {
                            let gap = card_rects
                                .iter()
                                .position(|r| pos.x < r.center().x)
                                .unwrap_or(card_rects.len());

                            // Live indicator: a vertical bar sitting in the gap the
                            // slide would land in, so the target is visible before release.
                            if gap_to_target_index(from_idx, gap, card_rects.len()).is_some() {
                                let x = match card_rects.get(gap) {
                                    Some(r) => r.left() - 4.0,
                                    None => card_rects
                                        .last()
                                        .map(|r| r.right() + 4.0)
                                        .unwrap_or(ui.max_rect().left()),
                                };
                                let (top, bottom) = card_rects
                                    .first()
                                    .map(|r| (r.top(), r.bottom()))
                                    .unwrap_or((ui.max_rect().top(), ui.max_rect().bottom()));
                                ui.painter().line_segment(
                                    [egui::pos2(x, top), egui::pos2(x, bottom)],
                                    Stroke::new(3.0, AppTheme::accent_yellow()),
                                );
                            }

                            // egui clears the payload on release; take it here so the
                            // drop resolves once, on the frame the pointer comes up.
                            if ui.input(|i| i.pointer.any_released()) {
                                if let Some(payload) =
                                    egui::DragAndDrop::take_payload::<SlideReorderDrag>(ui.ctx())
                                {
                                    action = SlideDeckAction::ReorderSlideToGap {
                                        from_idx: payload.0,
                                        to_gap: gap,
                                    };
                                }
                            }
                        }
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
        dragging_idx: Option<usize>,
    ) -> (SlideDeckAction, egui::Rect) {
        let mut action = SlideDeckAction::None;

        let card_w = 175.0;
        let card_h = 114.0;

        let is_being_dragged = dragging_idx == Some(idx);

        let border_stroke = if is_being_dragged {
            // The carried card reads as a "hole" the deck will close up.
            Stroke::new(2.0, AppTheme::accent_cyan())
        } else if is_active {
            Stroke::new(2.5, AppTheme::accent_yellow())
        } else {
            Stroke::new(1.0, Color32::from_rgb(45, 50, 65))
        };

        let card_bg = if is_being_dragged {
            // Dim the card being carried so the deck reads as a gap opening up
            // rather than the slide being in two places at once.
            Color32::from_rgb(16, 22, 30)
        } else if is_active {
            Color32::from_rgb(26, 36, 54)
        } else {
            AppTheme::bg_card()
        };

        let card_rect = Frame::none()
            .fill(card_bg)
            .rounding(Rounding::same(6.0))
            .stroke(border_stroke)
            .inner_margin(egui::Margin::symmetric(6.0, 5.0))
            .show(ui, |ui| {
                ui.set_width(card_w);
                ui.set_height(card_h);

                ui.vertical(|ui| {
                    // 1. Top Header Row: Slide # badge, duration, and quick reorder (clean spacing so arrows never overlap)
                    ui.horizontal(|ui| {
                        let num_badge = Button::new(
                            RichText::new(format!("#{}", slide_num))
                                .size(11.0)
                                .strong()
                                .color(if is_active { Color32::WHITE } else { AppTheme::text_secondary() }),
                        )
                        .fill(if is_active { AppTheme::accent_blue() } else { Color32::from_rgb(20, 24, 32) })
                        .min_size(Vec2::new(26.0, 18.0));

                        if ui.add(num_badge).clicked() {
                            action = SlideDeckAction::SelectSlide(clip.id);
                        }

                        ui.add_space(2.0);
                        ui.label(
                            RichText::new(format!("{:.1}s", clip.duration().as_secs_f64()))
                                .size(10.5)
                                .color(AppTheme::text_secondary()),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            ui.spacing_mut().button_padding = Vec2::new(2.0, 1.0);

                            if ui
                                .add(Button::new(RichText::new("×").size(11.5).color(Color32::from_rgb(255, 140, 140))).min_size(Vec2::new(16.0, 16.0)))
                                .on_hover_text("Delete slide")
                                .clicked()
                            {
                                action = SlideDeckAction::DeleteSlide(clip.id);
                            }

                            if idx + 1 < total_slides {
                                if ui.add(Button::new(RichText::new("▶").size(9.5)).min_size(Vec2::new(16.0, 16.0))).on_hover_text("Move right").clicked() {
                                    action = SlideDeckAction::MoveSlideDown(idx);
                                }
                            }
                            if idx > 0 {
                                if ui.add(Button::new(RichText::new("◀").size(9.5)).min_size(Vec2::new(16.0, 16.0))).on_hover_text("Move left").clicked() {
                                    action = SlideDeckAction::MoveSlideUp(idx);
                                }
                            }
                        });
                    });

                    ui.add_space(2.0);

                    // 2. 16:9 Aspect Ratio Thumbnail Box (148 x 72 px):
                    // a real miniature of the slide (background + elements laid
                    // out at their true positions), not an icon summary.
                    // The thumbnail doubles as the drag handle: click still selects,
                    // but holding and moving picks the slide up for reordering. The
                    // header buttons keep their own click senses and are unaffected.
                    let thumb_size = Vec2::new(card_w - 12.0, 72.0);
                    let (thumb_rect, thumb_resp) =
                        ui.allocate_exact_size(thumb_size, Sense::click_and_drag());

                    if thumb_resp.drag_started() {
                        egui::DragAndDrop::set_payload(ui.ctx(), SlideReorderDrag(idx));
                    }
                    if thumb_resp.dragged() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                    } else if thumb_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                    }

                    let painter = ui.painter();

                    let drawn_pic = Self::paint_slide_mini(painter, thumb_rect, clip, app);

                    // Element badges overlay on thumbnail (fallback while textures load)
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

                    // `clicked()` is already false for the release that ends a drag,
                    // so a reorder gesture never doubles as a selection.
                    if thumb_resp.clicked() {
                        action = SlideDeckAction::SelectSlide(clip.id);
                    }
                });
            })
            .response
            .rect;

        (action, card_rect)
    }

}
