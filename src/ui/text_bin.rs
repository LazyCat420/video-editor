use crate::core::clip::Clip;
use crate::core::text_overlay::{
    FontFamilyPreset, TextAlignment, TextBoxStyle, TextOverlay, TextPosition, TitleCardBackground,
};
use crate::core::timeline::Timeline;
use crate::ui::theme::AppTheme;
use egui::{Button, Color32, Frame, Id, RichText, Rounding, ScrollArea, Ui};
use std::path::PathBuf;

pub struct TextBinView;

pub enum TextBinAction {
    None,
    SetClipTextOverlay {
        clip_id: u64,
        overlay: Option<TextOverlay>,
    },
    InsertTitleCard {
        name: String,
        overlay: TextOverlay,
        bg: TitleCardBackground,
        duration_secs: f64,
        at_start: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TextBinTab {
    TitleCardBuilder,
    SelectedClipText,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BackgroundMode {
    SolidColor,
    Picture,
}

impl TextBinView {
    pub fn render(ui: &mut Ui, timeline: &mut Timeline) -> TextBinAction {
        let mut action = TextBinAction::None;

        let tab_id = Id::new("text_bin_tab_selection");
        let mut current_tab: TextBinTab = ui
            .data_mut(|d| d.get_temp(tab_id))
            .unwrap_or(TextBinTab::TitleCardBuilder);

        let selected_clip: Option<Clip> = timeline.get_selected_clip().cloned();

        ui.vertical(|ui| {
            ui.add_space(4.0);

            // Tab Selector Header
            ui.horizontal(|ui| {
                let is_builder = current_tab == TextBinTab::TitleCardBuilder;
                let is_clip_text = current_tab == TextBinTab::SelectedClipText;

                let builder_btn = Button::new(
                    RichText::new("🎬 Title Card Builder")
                        .size(12.5)
                        .strong()
                        .color(if is_builder { Color32::WHITE } else { AppTheme::text_secondary() }),
                )
                .fill(if is_builder { AppTheme::accent_blue() } else { AppTheme::bg_card() })
                .min_size(egui::vec2(140.0, 30.0));

                if ui.add(builder_btn).on_hover_text("Create standalone title card with solid color or photo background").clicked() {
                    current_tab = TextBinTab::TitleCardBuilder;
                    ui.data_mut(|d| d.insert_temp(tab_id, current_tab));
                }

                let clip_btn = Button::new(
                    RichText::new("💬 Text on Clip")
                        .size(12.5)
                        .strong()
                        .color(if is_clip_text { Color32::WHITE } else { AppTheme::text_secondary() }),
                )
                .fill(if is_clip_text { AppTheme::accent_blue() } else { AppTheme::bg_card() })
                .min_size(egui::vec2(110.0, 30.0));

                if ui.add(clip_btn).on_hover_text("Add text/captions directly on top of selected video/photo").clicked() {
                    current_tab = TextBinTab::SelectedClipText;
                    ui.data_mut(|d| d.insert_temp(tab_id, current_tab));
                }
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            ScrollArea::vertical().show(ui, |ui| {
                match current_tab {
                    TextBinTab::TitleCardBuilder => {
                        Self::render_title_card_builder(ui, &mut action);
                    }
                    TextBinTab::SelectedClipText => {
                        Self::render_clip_text_editor(ui, selected_clip.as_ref(), &mut action);
                    }
                }
            });
        });

        action
    }

    /// Standalone Title Card Builder
    fn render_title_card_builder(ui: &mut Ui, action: &mut TextBinAction) {
        Frame::none()
            .fill(AppTheme::bg_card())
            .rounding(Rounding::same(8.0))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("🎬 Create Standalone Title Card")
                        .size(14.5)
                        .strong()
                        .color(AppTheme::accent_yellow()),
                );
                ui.label(
                    RichText::new("Add opening titles, chapters, or photo slides with formatted text.")
                        .size(12.0)
                        .color(AppTheme::text_secondary()),
                );

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // 1. Background Selection: Solid Color vs Picture
                let bg_mode_id = Id::new("builder_bg_mode");
                let solid_col_id = Id::new("builder_solid_color");
                let pic_path_id = Id::new("builder_picture_path");

                let mut bg_mode: BackgroundMode = ui
                    .data_mut(|d| d.get_temp(bg_mode_id))
                    .unwrap_or(BackgroundMode::SolidColor);
                let mut solid_color: Color32 = ui
                    .data_mut(|d| d.get_temp(solid_col_id))
                    .unwrap_or_else(|| Color32::from_rgb(18, 18, 26));
                let mut pic_path: Option<PathBuf> = ui.data_mut(|d| d.get_temp(pic_path_id));

                ui.label(
                    RichText::new("1. Card Background:")
                        .size(13.0)
                        .strong()
                        .color(Color32::WHITE),
                );

                ui.horizontal(|ui| {
                    if ui.selectable_value(&mut bg_mode, BackgroundMode::SolidColor, "🎨 Solid Color").clicked() {
                        ui.data_mut(|d| d.insert_temp(bg_mode_id, bg_mode));
                    }
                    if ui.selectable_value(&mut bg_mode, BackgroundMode::Picture, "🖼 Picture / Photo").clicked() {
                        ui.data_mut(|d| d.insert_temp(bg_mode_id, bg_mode));
                    }
                });

                ui.add_space(4.0);

                match bg_mode {
                    BackgroundMode::SolidColor => {
                        ui.horizontal_wrapped(|ui| {
                            let swatches = [
                                ("⬛ Black", Color32::from_rgb(12, 12, 16)),
                                ("🟦 Navy", Color32::from_rgb(15, 30, 60)),
                                ("🟩 Forest", Color32::from_rgb(18, 48, 28)),
                                ("🟫 Sand", Color32::from_rgb(55, 42, 28)),
                                ("🟥 Crimson", Color32::from_rgb(55, 18, 25)),
                                ("⬜ White", Color32::from_rgb(240, 240, 245)),
                            ];

                            for (lbl, col) in swatches {
                                let is_active = solid_color == col;
                                if ui
                                    .add(
                                        Button::new(
                                            RichText::new(lbl)
                                                .size(11.5)
                                                .color(if is_active { Color32::WHITE } else { AppTheme::text_secondary() }),
                                        )
                                        .fill(if is_active { AppTheme::accent_blue() } else { AppTheme::bg_panel() }),
                                    )
                                    .clicked()
                                {
                                    solid_color = col;
                                    ui.data_mut(|d| d.insert_temp(solid_col_id, solid_color));
                                }
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Custom Color:").size(12.0).color(AppTheme::text_secondary()));
                            let mut rgb = [solid_color.r(), solid_color.g(), solid_color.b()];
                            if ui.color_edit_button_srgb(&mut rgb).changed() {
                                solid_color = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                                ui.data_mut(|d| d.insert_temp(solid_col_id, solid_color));
                            }
                        });
                    }
                    BackgroundMode::Picture => {
                        ui.horizontal(|ui| {
                            let pic_lbl = if let Some(ref p) = pic_path {
                                p.file_name().and_then(|n| n.to_str()).unwrap_or("Picture")
                            } else {
                                "No picture selected"
                            };

                            ui.label(RichText::new(pic_lbl).size(12.0).color(AppTheme::accent_cyan()));

                            if ui
                                .add(
                                    Button::new(RichText::new("📂 Pick Photo...").size(12.0).strong().color(Color32::WHITE))
                                        .fill(AppTheme::accent_blue()),
                                )
                                .clicked()
                            {
                                if let Some(path) = crate::media::probe::create_media_file_dialog().pick_file() {
                                    pic_path = Some(path.clone());
                                    ui.data_mut(|d| d.insert_temp(pic_path_id, Some(path)));
                                }
                            }
                        });
                    }
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                // 2. Text Content & Typography Controls
                let text_id = Id::new("builder_text_overlay");
                let mut overlay: TextOverlay = ui.data_mut(|d| d.get_temp(text_id)).unwrap_or_else(|| {
                    let mut o = TextOverlay::new("VACATION MEMORIES\nSummer 2026");
                    o.font_size = 44.0;
                    o.position = TextPosition::Center;
                    o
                });

                ui.label(
                    RichText::new("2. Title Text & Words:")
                        .size(13.0)
                        .strong()
                        .color(Color32::WHITE),
                );

                let mut text_buf = overlay.text.clone();
                if ui
                    .add(
                        egui::TextEdit::multiline(&mut text_buf)
                            .hint_text("Type your title here...\n(Press Enter for second line)")
                            .desired_rows(2)
                            .desired_width(ui.available_width()),
                    )
                    .changed()
                {
                    overlay.text = text_buf;
                    ui.data_mut(|d| d.insert_temp(text_id, overlay.clone()));
                }

                ui.add_space(8.0);

                // Render Typography and Formatting Controls
                if Self::render_typography_controls(ui, &mut overlay, "builder") {
                    ui.data_mut(|d| d.insert_temp(text_id, overlay.clone()));
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                // 3. Duration & Insertion
                let dur_id = Id::new("builder_card_dur");
                let mut card_dur: f64 = ui.data_mut(|d| d.get_temp(dur_id)).unwrap_or(4.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Duration:").size(12.5).strong().color(Color32::WHITE));
                    if crate::ui::small_slider(ui, 12.0, |ui| {
                        ui.add_sized(
                            [120.0, 12.0],
                            egui::Slider::new(&mut card_dur, 1.0..=20.0)
                                .custom_formatter(|v, _| format!("{:.1} seconds", v))
                                .step_by(0.5),
                        )
                    })
                    .changed()
                    {
                        ui.data_mut(|d| d.insert_temp(dur_id, card_dur));
                    }
                });

                ui.add_space(12.0);

                let bg = match bg_mode {
                    BackgroundMode::SolidColor => TitleCardBackground::SolidColor(solid_color),
                    BackgroundMode::Picture => {
                        if let Some(p) = pic_path {
                            TitleCardBackground::Picture(p)
                        } else {
                            TitleCardBackground::SolidColor(solid_color)
                        }
                    }
                };

                let card_name = if overlay.text.trim().is_empty() {
                    "Title Card".to_string()
                } else {
                    overlay.text.lines().next().unwrap_or("Title Card").to_string()
                };

                ui.horizontal(|ui| {
                    let at_playhead_btn = Button::new(
                        RichText::new("➕ Insert at Playhead")
                            .size(13.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(AppTheme::accent_green())
                    .min_size(egui::vec2((ui.available_width() - 8.0) / 2.0, 34.0));

                    if ui.add(at_playhead_btn).on_hover_text("Insert this title card at the current playhead").clicked() {
                        *action = TextBinAction::InsertTitleCard {
                            name: card_name.clone(),
                            overlay: overlay.clone(),
                            bg: bg.clone(),
                            duration_secs: card_dur,
                            at_start: false,
                        };
                    }

                    let at_start_btn = Button::new(
                        RichText::new("⏮ Insert at Start")
                            .size(13.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(AppTheme::accent_blue())
                    .min_size(egui::vec2(ui.available_width(), 34.0));

                    if ui.add(at_start_btn).on_hover_text("Insert this title card at the beginning of video (00:00)").clicked() {
                        *action = TextBinAction::InsertTitleCard {
                            name: card_name,
                            overlay: overlay.clone(),
                            bg,
                            duration_secs: card_dur,
                            at_start: true,
                        };
                    }
                });
            });
    }

    /// Direct On-Clip Text Overlay Editor
    fn render_clip_text_editor(
        ui: &mut Ui,
        selected_clip: Option<&Clip>,
        action: &mut TextBinAction,
    ) {
        Frame::none()
            .fill(AppTheme::bg_card())
            .rounding(Rounding::same(8.0))
            .inner_margin(12.0)
            .show(ui, |ui| {
                if let Some(clip) = selected_clip {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("🎬 Selected:")
                                .strong()
                                .size(13.0)
                                .color(AppTheme::accent_blue()),
                        );
                        ui.label(
                            RichText::new(&clip.name)
                                .strong()
                                .size(13.0)
                                .color(Color32::WHITE),
                        );
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    let mut overlay = clip.text_overlay.clone().unwrap_or_default();
                    let has_overlay = clip.text_overlay.is_some();

                    ui.label(
                        RichText::new("Words on Top of this Clip:")
                            .size(13.0)
                            .strong()
                            .color(Color32::WHITE),
                    );

                    let mut text_buf = overlay.text.clone();
                    let text_changed = ui
                        .add(
                            egui::TextEdit::multiline(&mut text_buf)
                                .hint_text("Type words to show directly on this video or photo...")
                                .desired_rows(2)
                                .desired_width(ui.available_width()),
                        )
                        .changed();

                    if text_changed {
                        overlay.text = text_buf;
                    }

                    ui.add_space(8.0);

                    let styling_changed = Self::render_typography_controls(ui, &mut overlay, "clip");

                    if text_changed || styling_changed {
                        if overlay.text.trim().is_empty() {
                            *action = TextBinAction::SetClipTextOverlay {
                                clip_id: clip.id,
                                overlay: None,
                            };
                        } else {
                            *action = TextBinAction::SetClipTextOverlay {
                                clip_id: clip.id,
                                overlay: Some(overlay.clone()),
                            };
                        }
                    }

                    if has_overlay {
                        ui.add_space(12.0);
                        let remove_btn = Button::new(
                            RichText::new("❌ Remove Words from this Clip")
                                .size(12.5)
                                .color(Color32::from_rgb(255, 140, 140)),
                        )
                        .min_size(egui::vec2(ui.available_width(), 28.0))
                        .fill(Color32::from_rgb(60, 25, 25));

                        if ui.add(remove_btn).clicked() {
                            *action = TextBinAction::SetClipTextOverlay {
                                clip_id: clip.id,
                                overlay: None,
                            };
                        }
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(
                            RichText::new("ℹ No Clip Selected on Timeline")
                                .strong()
                                .size(14.0)
                                .color(AppTheme::text_secondary()),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("Click any video clip or picture on the timeline to add words directly on top.")
                                .size(12.0)
                                .color(AppTheme::text_muted()),
                        );
                        ui.add_space(20.0);
                    });
                }
            });
    }

    /// Common Typography & Formatting Controls
    fn render_typography_controls(ui: &mut Ui, overlay: &mut TextOverlay, id_prefix: &str) -> bool {
        let mut changed = false;

        // 1. Visual Font Picker with Previews
        ui.label(
            RichText::new("Font Style:")
                .size(12.5)
                .strong()
                .color(Color32::WHITE),
        );

        let combo_id = format!("{}_font_combo", id_prefix);
        egui::ComboBox::from_id_salt(combo_id)
            .selected_text(
                RichText::new(format!("🔤 {}  ({})", overlay.font_family.label(), overlay.font_family.preview_sample()))
                    .size(13.0)
                    .strong(),
            )
            .width(ui.available_width() - 8.0)
            .show_ui(ui, |ui| {
                for font in FontFamilyPreset::all() {
                    let is_sel = overlay.font_family == *font;
                    let text = format!("{}  -  {}", font.preview_sample(), font.description());

                    let mut rich = RichText::new(text).size(13.0);
                    if is_sel {
                        rich = rich.strong().color(AppTheme::accent_yellow());
                    }

                    if ui.selectable_label(is_sel, rich).clicked() {
                        overlay.font_family = *font;
                        changed = true;
                    }
                }
            });

        ui.add_space(6.0);

        // 2. Formatting Toggles (Bold, Italic, ALL CAPS)
        ui.horizontal(|ui| {
            let bold_btn = Button::new(
                RichText::new("B Bold")
                    .size(12.0)
                    .strong()
                    .color(if overlay.is_bold { Color32::WHITE } else { AppTheme::text_secondary() }),
            )
            .fill(if overlay.is_bold { AppTheme::accent_blue() } else { AppTheme::bg_panel() })
            .min_size(egui::vec2(60.0, 24.0));

            if ui.add(bold_btn).clicked() {
                overlay.is_bold = !overlay.is_bold;
                changed = true;
            }

            let italic_btn = Button::new(
                RichText::new("I Italic")
                    .size(12.0)
                    .color(if overlay.is_italic { Color32::WHITE } else { AppTheme::text_secondary() }),
            )
            .fill(if overlay.is_italic { AppTheme::accent_blue() } else { AppTheme::bg_panel() })
            .min_size(egui::vec2(60.0, 24.0));

            if ui.add(italic_btn).clicked() {
                overlay.is_italic = !overlay.is_italic;
                changed = true;
            }

            let caps_btn = Button::new(
                RichText::new("ALL CAPS")
                    .size(11.5)
                    .strong()
                    .color(if overlay.is_all_caps { Color32::WHITE } else { AppTheme::text_secondary() }),
            )
            .fill(if overlay.is_all_caps { AppTheme::accent_blue() } else { AppTheme::bg_panel() })
            .min_size(egui::vec2(75.0, 24.0));

            if ui.add(caps_btn).clicked() {
                overlay.is_all_caps = !overlay.is_all_caps;
                changed = true;
            }
        });

        ui.add_space(6.0);

        // 3. Alignment Buttons
        ui.horizontal(|ui| {
            ui.label(RichText::new("Align:").size(12.0).color(AppTheme::text_secondary()));
            for align in TextAlignment::all() {
                let is_sel = overlay.alignment == *align;
                let btn_text = match align {
                    TextAlignment::Left => "⇤ Left",
                    TextAlignment::Center => "≡ Center",
                    TextAlignment::Right => "⇥ Right",
                };
                if ui
                    .add(
                        Button::new(
                            RichText::new(btn_text)
                                .size(11.5)
                                .color(if is_sel { Color32::WHITE } else { AppTheme::text_secondary() }),
                        )
                        .fill(if is_sel { AppTheme::accent_blue() } else { AppTheme::bg_panel() }),
                    )
                    .clicked()
                {
                    overlay.alignment = *align;
                    changed = true;
                }
            }
        });

        ui.add_space(6.0);

        // 4. Position Picker
        ui.horizontal(|ui| {
            ui.label(RichText::new("Position:").size(12.0).color(AppTheme::text_secondary()));
            let pos_combo = format!("{}_pos_combo", id_prefix);
            egui::ComboBox::from_id_salt(pos_combo)
                .selected_text(overlay.position.label())
                .show_ui(ui, |ui| {
                    for pos in TextPosition::all() {
                        if ui.selectable_value(&mut overlay.position, *pos, pos.label()).clicked() {
                            changed = true;
                        }
                    }
                });
        });

        ui.add_space(6.0);

        // 5. Size Slider
        ui.horizontal(|ui| {
            ui.label(RichText::new("Font Size:").size(12.0).color(AppTheme::text_secondary()));
            if crate::ui::small_slider(ui, 12.0, |ui| {
                ui.add_sized(
                    [120.0, 12.0],
                    egui::Slider::new(&mut overlay.font_size, 14.0..=100.0)
                        .custom_formatter(|v, _| format!("{:.0} pt", v))
                        .step_by(2.0),
                )
            })
            .changed()
            {
                changed = true;
            }
        });

        ui.add_space(6.0);

        // 6. Text Color Swatches & Custom
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Color:").size(12.0).color(AppTheme::text_secondary()));
            let colors = [
                ("⚪ White", Color32::WHITE),
                ("🟡 Gold", Color32::from_rgb(255, 215, 90)),
                ("🔵 Cyan", Color32::from_rgb(0, 230, 255)),
                ("🔴 Coral", Color32::from_rgb(255, 140, 110)),
                ("🟢 Lime", Color32::from_rgb(140, 255, 120)),
                ("⚫ Black", Color32::from_rgb(15, 15, 15)),
            ];

            for (lbl, c) in colors {
                let is_sel = overlay.text_color == c;
                if ui
                    .add(
                        Button::new(
                            RichText::new(lbl)
                                .size(11.0)
                                .color(if is_sel { Color32::WHITE } else { AppTheme::text_secondary() }),
                        )
                        .fill(if is_sel { AppTheme::accent_blue() } else { AppTheme::bg_panel() }),
                    )
                    .clicked()
                {
                    overlay.text_color = c;
                    changed = true;
                }
            }

            let mut rgb = [overlay.text_color.r(), overlay.text_color.g(), overlay.text_color.b()];
            if ui.color_edit_button_srgb(&mut rgb).changed() {
                overlay.text_color = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                changed = true;
            }
        });

        ui.add_space(6.0);

        // 7. Backing Box / Pill
        ui.horizontal(|ui| {
            ui.label(RichText::new("Backing:").size(12.0).color(AppTheme::text_secondary()));
            let box_combo = format!("{}_box_combo", id_prefix);
            egui::ComboBox::from_id_salt(box_combo)
                .selected_text(overlay.box_style.label())
                .show_ui(ui, |ui| {
                    for st in TextBoxStyle::all() {
                        if ui.selectable_value(&mut overlay.box_style, *st, st.label()).clicked() {
                            changed = true;
                        }
                    }
                });
        });

        if overlay.box_style != TextBoxStyle::None {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Box Opacity:").size(11.5).color(AppTheme::text_secondary()));
                if crate::ui::small_slider(ui, 12.0, |ui| {
                    ui.add_sized(
                        [100.0, 12.0],
                        egui::Slider::new(&mut overlay.box_opacity, 0.1..=1.0)
                            .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                            .step_by(0.05),
                    )
                })
                .changed()
                {
                    changed = true;
                }
            });
        }

        changed
    }
}
