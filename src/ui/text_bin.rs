use crate::core::clip::Clip;
use crate::core::text_overlay::{TextOverlay, TextPosition, TextStylePreset, TitleCardTheme};
use crate::core::timeline::Timeline;
use crate::core::transition::TransitionKind;
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
        title: String,
        subtitle: Option<String>,
        theme: TitleCardTheme,
        duration_secs: f64,
        at_start: bool,
    },
    CreateVacationSlideshow {
        paths: Vec<PathBuf>,
        title: String,
        subtitle: Option<String>,
        outro: String,
        theme: TitleCardTheme,
        photo_duration_secs: f64,
        transition: Option<TransitionKind>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TextBinTab {
    Wizard,
    TitleCards,
    SelectedCaption,
}

impl TextBinView {
    pub fn render(ui: &mut Ui, timeline: &mut Timeline) -> TextBinAction {
        let mut action = TextBinAction::None;

        let tab_id = Id::new("text_bin_subtab");
        let mut current_tab: TextBinTab = ui
            .data_mut(|d| d.get_temp(tab_id))
            .unwrap_or(TextBinTab::Wizard);

        let selected_clip: Option<Clip> = timeline.get_selected_clip().cloned();

        ui.vertical(|ui| {
            ui.add_space(4.0);

            // Subtab Header
            ui.horizontal(|ui| {
                let is_wiz = current_tab == TextBinTab::Wizard;
                let is_cards = current_tab == TextBinTab::TitleCards;
                let is_caption = current_tab == TextBinTab::SelectedCaption;

                let wiz_btn = Button::new(
                    RichText::new("🌴 Slideshow Wizard")
                        .size(12.0)
                        .strong()
                        .color(if is_wiz { Color32::WHITE } else { AppTheme::text_secondary() }),
                )
                .fill(if is_wiz { AppTheme::accent_blue() } else { AppTheme::bg_card() })
                .min_size(egui::vec2(130.0, 28.0));

                if ui.add(wiz_btn).on_hover_text("1-Click Vacation Slideshow Creator").clicked() {
                    current_tab = TextBinTab::Wizard;
                    ui.data_mut(|d| d.insert_temp(tab_id, current_tab));
                }

                let cards_btn = Button::new(
                    RichText::new("🎬 Title Cards")
                        .size(12.0)
                        .strong()
                        .color(if is_cards { Color32::WHITE } else { AppTheme::text_secondary() }),
                )
                .fill(if is_cards { AppTheme::accent_blue() } else { AppTheme::bg_card() })
                .min_size(egui::vec2(95.0, 28.0));

                if ui.add(cards_btn).on_hover_text("Add Opening or Ending Title Cards").clicked() {
                    current_tab = TextBinTab::TitleCards;
                    ui.data_mut(|d| d.insert_temp(tab_id, current_tab));
                }

                let cap_btn = Button::new(
                    RichText::new("💬 Captions")
                        .size(12.0)
                        .strong()
                        .color(if is_caption { Color32::WHITE } else { AppTheme::text_secondary() }),
                )
                .fill(if is_caption { AppTheme::accent_blue() } else { AppTheme::bg_card() })
                .min_size(egui::vec2(80.0, 28.0));

                if ui.add(cap_btn).on_hover_text("Add text overlay to the selected picture").clicked() {
                    current_tab = TextBinTab::SelectedCaption;
                    ui.data_mut(|d| d.insert_temp(tab_id, current_tab));
                }
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            ScrollArea::vertical().show(ui, |ui| {
                match current_tab {
                    TextBinTab::Wizard => {
                        Self::render_wizard(ui, &mut action);
                    }
                    TextBinTab::TitleCards => {
                        Self::render_title_cards(ui, &mut action);
                    }
                    TextBinTab::SelectedCaption => {
                        Self::render_caption_editor(ui, selected_clip.as_ref(), &mut action);
                    }
                }
            });
        });

        action
    }

    fn render_wizard(ui: &mut Ui, action: &mut TextBinAction) {
        Frame::none()
            .fill(AppTheme::bg_card())
            .rounding(Rounding::same(8.0))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("🌴 1-Click Vacation Slideshow Wizard")
                        .size(15.0)
                        .strong()
                        .color(AppTheme::accent_yellow()),
                );
                ui.label(
                    RichText::new("Turn a folder of vacation photos into a movie with titles & smooth transitions.")
                        .size(12.0)
                        .color(AppTheme::text_secondary()),
                );

                ui.add_space(10.0);

                // Step 1: Photos storage in UI state
                let photos_id = Id::new("wizard_selected_photos");
                let mut photos: Vec<PathBuf> = ui.data_mut(|d| d.get_temp(photos_id)).unwrap_or_default();

                ui.label(
                    RichText::new("Step 1: Pick Vacation Photos")
                        .size(13.0)
                        .strong()
                        .color(Color32::WHITE),
                );

                let pick_btn = Button::new(
                    RichText::new(format!("📂 Select Photos ({} chosen)", photos.len()))
                        .size(13.5)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(AppTheme::accent_blue())
                .min_size(egui::vec2(ui.available_width(), 32.0));

                if ui.add(pick_btn).clicked() {
                    if let Some(files) = crate::media::probe::create_media_file_dialog().pick_files() {
                        photos = files;
                        ui.data_mut(|d| d.insert_temp(photos_id, photos.clone()));
                    }
                }

                ui.add_space(10.0);

                // Step 2: Vacation Titles
                let title_id = Id::new("wizard_title_text");
                let sub_id = Id::new("wizard_subtitle_text");
                let outro_id = Id::new("wizard_outro_text");

                let mut title_text: String = ui.data_mut(|d| d.get_temp(title_id)).unwrap_or_else(|| "Our Hawaii Vacation 2026".to_string());
                let mut sub_text: String = ui.data_mut(|d| d.get_temp(sub_id)).unwrap_or_else(|| "Family Memories".to_string());
                let mut outro_text: String = ui.data_mut(|d| d.get_temp(outro_id)).unwrap_or_else(|| "The End ❤️".to_string());

                ui.label(
                    RichText::new("Step 2: Vacation Title & Names")
                        .size(13.0)
                        .strong()
                        .color(Color32::WHITE),
                );

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Title:").size(12.0).color(AppTheme::text_secondary()));
                    if ui.text_edit_singleline(&mut title_text).changed() {
                        ui.data_mut(|d| d.insert_temp(title_id, title_text.clone()));
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Subtitle:").size(12.0).color(AppTheme::text_secondary()));
                    if ui.text_edit_singleline(&mut sub_text).changed() {
                        ui.data_mut(|d| d.insert_temp(sub_id, sub_text.clone()));
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Outro:").size(12.0).color(AppTheme::text_secondary()));
                    if ui.text_edit_singleline(&mut outro_text).changed() {
                        ui.data_mut(|d| d.insert_temp(outro_id, outro_text.clone()));
                    }
                });

                ui.add_space(10.0);

                // Step 3: Theme & Pace
                let theme_id = Id::new("wizard_theme");
                let dur_id = Id::new("wizard_photo_dur");

                let mut chosen_theme: TitleCardTheme = ui.data_mut(|d| d.get_temp(theme_id)).unwrap_or(TitleCardTheme::SunsetGlow);
                let mut photo_dur: f64 = ui.data_mut(|d| d.get_temp(dur_id)).unwrap_or(4.0);

                ui.label(
                    RichText::new("Step 3: Theme & Speed")
                        .size(13.0)
                        .strong()
                        .color(Color32::WHITE),
                );

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Theme:").size(12.0).color(AppTheme::text_secondary()));
                    egui::ComboBox::from_id_salt("wizard_theme_cb")
                        .selected_text(chosen_theme.label())
                        .show_ui(ui, |ui| {
                            for th in TitleCardTheme::all() {
                                if ui.selectable_value(&mut chosen_theme, *th, th.label()).clicked() {
                                    ui.data_mut(|d| d.insert_temp(theme_id, chosen_theme));
                                }
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Pace:").size(12.0).color(AppTheme::text_secondary()));
                    if crate::ui::small_slider(ui, 12.0, |ui| {
                        ui.add_sized(
                            [120.0, 12.0],
                            egui::Slider::new(&mut photo_dur, 2.0..=8.0)
                                .custom_formatter(|v, _| format!("{:.1}s / photo", v))
                                .step_by(0.5),
                        )
                    })
                    .changed()
                    {
                        ui.data_mut(|d| d.insert_temp(dur_id, photo_dur));
                    }
                });

                ui.add_space(14.0);

                // Step 4: Big Action Button
                let can_create = !photos.is_empty() && !title_text.trim().is_empty();
                let create_btn = Button::new(
                    RichText::new("✨ CREATE VACATION SLIDESHOW")
                        .size(14.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(if can_create { AppTheme::accent_green() } else { AppTheme::bg_panel() })
                .min_size(egui::vec2(ui.available_width(), 38.0));

                if ui
                    .add_enabled(can_create, create_btn)
                    .on_hover_text("Build full slideshow with opening title, photos with crossfades, and ending card")
                    .clicked()
                {
                    *action = TextBinAction::CreateVacationSlideshow {
                        paths: photos,
                        title: title_text,
                        subtitle: if sub_text.trim().is_empty() { None } else { Some(sub_text) },
                        outro: outro_text,
                        theme: chosen_theme,
                        photo_duration_secs: photo_dur,
                        transition: Some(TransitionKind::CrossFade),
                    };
                }
            });
    }

    fn render_title_cards(ui: &mut Ui, action: &mut TextBinAction) {
        Frame::none()
            .fill(AppTheme::bg_card())
            .rounding(Rounding::same(8.0))
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("🎬 Create Opening / Ending Title Cards")
                        .size(14.0)
                        .strong()
                        .color(AppTheme::accent_yellow()),
                );

                ui.add_space(8.0);

                let title_id = Id::new("card_title_text");
                let sub_id = Id::new("card_subtitle_text");
                let theme_id = Id::new("card_theme");

                let mut title_text: String = ui.data_mut(|d| d.get_temp(title_id)).unwrap_or_else(|| "Vacation Memories".to_string());
                let mut sub_text: String = ui.data_mut(|d| d.get_temp(sub_id)).unwrap_or_else(|| "Summer 2026".to_string());
                let mut theme: TitleCardTheme = ui.data_mut(|d| d.get_temp(theme_id)).unwrap_or(TitleCardTheme::OceanBlue);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Title:").size(12.0).color(AppTheme::text_secondary()));
                    if ui.text_edit_singleline(&mut title_text).changed() {
                        ui.data_mut(|d| d.insert_temp(title_id, title_text.clone()));
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Subtitle:").size(12.0).color(AppTheme::text_secondary()));
                    if ui.text_edit_singleline(&mut sub_text).changed() {
                        ui.data_mut(|d| d.insert_temp(sub_id, sub_text.clone()));
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Theme:").size(12.0).color(AppTheme::text_secondary()));
                    egui::ComboBox::from_id_salt("card_theme_cb")
                        .selected_text(theme.label())
                        .show_ui(ui, |ui| {
                            for th in TitleCardTheme::all() {
                                if ui.selectable_value(&mut theme, *th, th.label()).clicked() {
                                    ui.data_mut(|d| d.insert_temp(theme_id, theme));
                                }
                            }
                        });
                });

                ui.add_space(10.0);

                let start_btn = Button::new(
                    RichText::new("➕ Insert Opening Card at Start")
                        .size(12.5)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(AppTheme::accent_blue())
                .min_size(egui::vec2(ui.available_width(), 30.0));

                if ui.add(start_btn).clicked() {
                    *action = TextBinAction::InsertTitleCard {
                        title: title_text.clone(),
                        subtitle: if sub_text.trim().is_empty() { None } else { Some(sub_text.clone()) },
                        theme,
                        duration_secs: 4.0,
                        at_start: true,
                    };
                }

                ui.add_space(4.0);

                let end_btn = Button::new(
                    RichText::new("➕ Insert Ending Card at Finish")
                        .size(12.5)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(60, 45, 80))
                .min_size(egui::vec2(ui.available_width(), 30.0));

                if ui.add(end_btn).clicked() {
                    *action = TextBinAction::InsertTitleCard {
                        title: title_text,
                        subtitle: if sub_text.trim().is_empty() { None } else { Some(sub_text) },
                        theme,
                        duration_secs: 4.0,
                        at_start: false,
                    };
                }
            });
    }

    fn render_caption_editor(ui: &mut Ui, selected_clip: Option<&Clip>, action: &mut TextBinAction) {
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

                    let mut overlay = clip.text_overlay.clone().unwrap_or_default();
                    let has_overlay = clip.text_overlay.is_some();

                    ui.label(
                        RichText::new("On-Screen Caption / Location:")
                            .size(12.5)
                            .strong()
                            .color(Color32::WHITE),
                    );

                    let changed = ui
                        .add(
                            egui::TextEdit::singleline(&mut overlay.text)
                                .hint_text("e.g. Snorkeling at Turtle Bay — Day 2"),
                        )
                        .changed();

                    ui.add_space(6.0);

                    // Position Picker
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Position:").size(12.0).color(AppTheme::text_secondary()));
                        egui::ComboBox::from_id_salt("caption_pos_cb")
                            .selected_text(overlay.position.label())
                            .show_ui(ui, |ui| {
                                for pos in TextPosition::all() {
                                    if ui.selectable_value(&mut overlay.position, *pos, pos.label()).clicked() {
                                        *action = TextBinAction::SetClipTextOverlay {
                                            clip_id: clip.id,
                                            overlay: Some(overlay.clone()),
                                        };
                                    }
                                }
                            });
                    });

                    // Style Picker
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Style:").size(12.0).color(AppTheme::text_secondary()));
                        egui::ComboBox::from_id_salt("caption_style_cb")
                            .selected_text(overlay.style.label())
                            .show_ui(ui, |ui| {
                                for st in TextStylePreset::all() {
                                    if ui.selectable_value(&mut overlay.style, *st, st.label()).clicked() {
                                        *action = TextBinAction::SetClipTextOverlay {
                                            clip_id: clip.id,
                                            overlay: Some(overlay.clone()),
                                        };
                                    }
                                }
                            });
                    });

                    // Size Slider
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Size:").size(12.0).color(AppTheme::text_secondary()));
                        if crate::ui::small_slider(ui, 12.0, |ui| {
                            ui.add_sized(
                                [110.0, 12.0],
                                egui::Slider::new(&mut overlay.font_size, 18.0..=56.0)
                                    .custom_formatter(|v, _| format!("{:.0} pt", v))
                                    .step_by(2.0),
                            )
                        })
                        .changed()
                        {
                            *action = TextBinAction::SetClipTextOverlay {
                                clip_id: clip.id,
                                overlay: Some(overlay.clone()),
                            };
                        }
                    });

                    // Legibility dark backing box toggle
                    ui.horizontal(|ui| {
                        if ui
                            .checkbox(&mut overlay.show_box, "High-Contrast Background Box")
                            .on_hover_text("Adds a semi-transparent dark backing box for 100% legibility on sunny photos")
                            .changed()
                        {
                            *action = TextBinAction::SetClipTextOverlay {
                                clip_id: clip.id,
                                overlay: Some(overlay.clone()),
                            };
                        }
                    });

                    if changed {
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
                        ui.add_space(8.0);
                        let remove_btn = Button::new(
                            RichText::new("❌ Remove Caption")
                                .size(12.0)
                                .color(Color32::from_rgb(255, 130, 130)),
                        )
                        .min_size(egui::vec2(ui.available_width(), 26.0))
                        .fill(Color32::from_rgb(55, 25, 25));

                        if ui.add(remove_btn).clicked() {
                            *action = TextBinAction::SetClipTextOverlay {
                                clip_id: clip.id,
                                overlay: None,
                            };
                        }
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("ℹ No Picture Selected")
                                .strong()
                                .size(13.0)
                                .color(AppTheme::text_secondary()),
                        );
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new("Click a photo or video on the timeline to add captions.")
                                .size(12.0)
                                .color(AppTheme::text_muted()),
                        );
                    });
                }
            });
    }
}
