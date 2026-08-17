use egui::{
    Button, Color32, Frame, Margin, RichText, Rounding, Sense, Stroke, Ui, Vec2,
};
use crate::ui::theme::AppTheme;

pub struct ActionRowCard;

impl ActionRowCard {
    pub fn render(
        ui: &mut Ui,
        icon: &str,
        title: &str,
        desc: &str,
        is_active: bool,
        budget_w: f32,
    ) -> bool {
        Self::render_with_image(ui, None, icon, title, desc, is_active, budget_w)
    }

    pub fn render_with_image(
        ui: &mut Ui,
        texture: Option<&egui::TextureHandle>,
        fallback_icon: &str,
        title: &str,
        desc: &str,
        is_active: bool,
        budget_w: f32,
    ) -> bool {
        let mut clicked = false;

        let outer_margin_x = 4.0;
        let card_w = (budget_w - 2.0 * outer_margin_x).max(180.0);

        let bg_color = if is_active {
            Color32::from_rgb(25, 40, 60)
        } else {
            AppTheme::bg_card()
        };

        let stroke_color = if is_active {
            AppTheme::accent_yellow()
        } else {
            Color32::from_rgb(45, 52, 68)
        };

        let resp = Frame::none()
            .fill(bg_color)
            .rounding(Rounding::same(6.0))
            .stroke(Stroke::new(if is_active { 1.5 } else { 1.0 }, stroke_color))
            .inner_margin(Margin::symmetric(6.0, 5.0))
            .show(ui, |ui| {
                ui.set_max_width(card_w);
                let inner_w = card_w - 2.0 * 6.0;

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;

                    // 1. Left Thumbnail / Icon Badge
                    const ICON_GLYPH_W: f32 = 28.0;
                    const ICON_GLYPH_H: f32 = 28.0;

                    Frame::none()
                        .fill(if is_active {
                            Color32::from_rgb(30, 55, 80)
                        } else {
                            Color32::from_rgb(20, 24, 34)
                        })
                        .rounding(Rounding::same(4.0))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(40, 48, 65)))
                        .inner_margin(Margin::same(2.0))
                        .show(ui, |ui| {
                            ui.allocate_ui_with_layout(
                                Vec2::new(ICON_GLYPH_W, ICON_GLYPH_H),
                                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                                |ui| {
                                    ui.set_clip_rect(ui.clip_rect().intersect(ui.max_rect()));
                                    if let Some(tex) = texture {
                                        ui.image((tex.id(), Vec2::new(ICON_GLYPH_W - 2.0, ICON_GLYPH_H - 2.0)));
                                    } else {
                                        ui.add(
                                            egui::Label::new(RichText::new(fallback_icon).size(13.0))
                                                .selectable(false),
                                        );
                                    }
                                },
                            );
                        });

                    ui.add_space(2.0);

                    // 2. Middle Text Column
                    let btn_w = 54.0;
                    let gap = ui.spacing().item_spacing.x;
                    let badge_w = ICON_GLYPH_W + 4.0;
                    let text_w = (inner_w - badge_w - btn_w - 2.0 - 2.0 * gap - 2.0).max(40.0);
                    ui.allocate_ui_with_layout(
                        Vec2::new(text_w, 34.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_max_width(text_w);
                            ui.set_clip_rect(ui.clip_rect().intersect(ui.max_rect()));
                            ui.spacing_mut().item_spacing.y = 1.0;
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(title)
                                            .strong()
                                            .size(11.5)
                                            .color(if is_active {
                                                AppTheme::accent_yellow()
                                            } else {
                                                Color32::WHITE
                                            }),
                                    )
                                    .truncate(),
                                );
                                if is_active {
                                    ui.label(
                                        RichText::new("✓")
                                            .size(9.5)
                                            .strong()
                                            .color(AppTheme::accent_yellow()),
                                    );
                                }
                            });
                            ui.add(
                                egui::Label::new(
                                    RichText::new(desc)
                                        .size(10.0)
                                        .color(AppTheme::text_muted()),
                                )
                                .truncate(),
                            );
                        },
                    );

                    // 3. Right Pinned Button (fixed 54x22 px)
                    let apply_label = if is_active { "Active" } else { "Apply" };
                    let btn = Button::new(
                        RichText::new(apply_label)
                            .size(11.0)
                            .color(Color32::WHITE)
                            .strong(),
                    )
                    .fill(if is_active {
                        AppTheme::accent_green()
                    } else {
                        AppTheme::accent_blue()
                    })
                    .min_size(Vec2::new(btn_w, 22.0));

                    if ui.add(btn).clicked() {
                        clicked = true;
                    }
                });
            });

        if resp
            .response
            .on_hover_text(desc)
            .interact(Sense::click())
            .clicked()
        {
            clicked = true;
        }

        clicked
    }
}

/// Modular LEGO Sidebar Tab Bar (2-column and 3-column).
pub struct SidebarTabs;

impl SidebarTabs {
    pub fn render_2_tabs(
        ui: &mut Ui,
        tab1_label: &str,
        tab1_active: bool,
        tab2_label: &str,
        tab2_active: bool,
    ) -> (bool, bool) {
        let mut t1_clicked = false;
        let mut t2_clicked = false;

        ui.columns(2, |cols| {
            let btn1 = Button::new(
                RichText::new(tab1_label)
                    .size(12.5)
                    .strong()
                    .color(if tab1_active { Color32::WHITE } else { AppTheme::text_secondary() }),
            )
            .fill(if tab1_active { AppTheme::accent_blue() } else { AppTheme::bg_card() })
            .min_size(Vec2::new(cols[0].available_width(), 28.0));

            if cols[0].add(btn1).clicked() {
                t1_clicked = true;
            }

            let btn2 = Button::new(
                RichText::new(tab2_label)
                    .size(12.5)
                    .strong()
                    .color(if tab2_active { Color32::WHITE } else { AppTheme::text_secondary() }),
            )
            .fill(if tab2_active { AppTheme::accent_blue() } else { AppTheme::bg_card() })
            .min_size(Vec2::new(cols[1].available_width(), 28.0));

            if cols[1].add(btn2).clicked() {
                t2_clicked = true;
            }
        });

        (t1_clicked, t2_clicked)
    }

    pub fn render_3_tabs(
        ui: &mut Ui,
        tab1_label: &str,
        tab1_active: bool,
        tab2_label: &str,
        tab2_active: bool,
        tab3_label: &str,
        tab3_active: bool,
    ) -> (bool, bool, bool) {
        let mut t1_clicked = false;
        let mut t2_clicked = false;
        let mut t3_clicked = false;

        ui.columns(3, |cols| {
            let btn1 = Button::new(
                RichText::new(tab1_label)
                    .size(11.5)
                    .strong()
                    .color(if tab1_active { Color32::WHITE } else { AppTheme::text_secondary() }),
            )
            .fill(if tab1_active { AppTheme::accent_blue() } else { AppTheme::bg_card() })
            .min_size(Vec2::new(cols[0].available_width(), 28.0));

            if cols[0].add(btn1).clicked() {
                t1_clicked = true;
            }

            let btn2 = Button::new(
                RichText::new(tab2_label)
                    .size(11.5)
                    .strong()
                    .color(if tab2_active { Color32::WHITE } else { AppTheme::text_secondary() }),
            )
            .fill(if tab2_active { AppTheme::accent_blue() } else { AppTheme::bg_card() })
            .min_size(Vec2::new(cols[1].available_width(), 28.0));

            if cols[1].add(btn2).clicked() {
                t2_clicked = true;
            }

            let btn3 = Button::new(
                RichText::new(tab3_label)
                    .size(11.5)
                    .strong()
                    .color(if tab3_active { Color32::WHITE } else { AppTheme::text_secondary() }),
            )
            .fill(if tab3_active { AppTheme::accent_blue() } else { AppTheme::bg_card() })
            .min_size(Vec2::new(cols[2].available_width(), 28.0));

            if cols[2].add(btn3).clicked() {
                t3_clicked = true;
            }
        });

        (t1_clicked, t2_clicked, t3_clicked)
    }
}
