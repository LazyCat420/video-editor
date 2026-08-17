use crate::ui::theme::AppTheme;
use egui::{Button, Color32, Frame, Margin, RichText, Rounding, Sense, Stroke, Ui, Vec2};

/// Modular LEGO Action Row Card (Icon + Title/Desc + Action Button).
pub struct ActionRowCard;

impl ActionRowCard {
    pub fn render(
        ui: &mut Ui,
        icon: &str,
        title: &str,
        desc: &str,
        is_active: bool,
    ) -> bool {
        let mut clicked = false;
        let bg_color = if is_active {
            Color32::from_rgb(32, 45, 64)
        } else {
            AppTheme::bg_card()
        };
        let border_stroke = if is_active {
            Stroke::new(1.5, AppTheme::accent_yellow())
        } else {
            Stroke::new(1.0, Color32::from_rgb(45, 48, 60))
        };

        let resp = Frame::none()
            .fill(bg_color)
            .rounding(Rounding::same(6.0))
            .stroke(border_stroke)
            .inner_margin(Margin::symmetric(8.0, 6.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // 1. Icon Badge (fixed 28x24 px)
                    Frame::none()
                        .fill(Color32::from_rgb(18, 22, 30))
                        .rounding(Rounding::same(4.0))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(40, 48, 65)))
                        .inner_margin(Margin::symmetric(5.0, 3.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new(icon).size(13.0));
                        });

                    ui.add_space(2.0);

                    // 2. Middle Text Column: Dynamically takes all remaining space
                    let btn_w = 54.0;
                    let text_w = (ui.available_width() - btn_w - 6.0).max(40.0);
                    ui.allocate_ui_with_layout(
                        Vec2::new(text_w, 30.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(title)
                                        .strong()
                                        .size(11.5)
                                        .color(if is_active {
                                            AppTheme::accent_yellow()
                                        } else {
                                            Color32::WHITE
                                        }),
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
                            ui.label(
                                RichText::new(desc)
                                    .size(10.0)
                                    .color(AppTheme::text_muted()),
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

        if resp.response.interact(Sense::click()).clicked() {
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
