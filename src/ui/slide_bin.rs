use crate::VideoEditorApp;
use crate::core::text_overlay::{SlideBackground, SlideElement, TextBoxStyle};
use crate::ui::theme::AppTheme;
use crate::ui::SlideBinAction;
use egui::{Button, Color32, RichText, Ui};
use std::path::Path;

pub struct SlideBinView;

impl SlideBinView {
    pub fn render(ui: &mut Ui, app: &mut VideoEditorApp) -> SlideBinAction {
        let mut action = SlideBinAction::None;

        ui.vertical(|ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui
                    .add(
                        Button::new(RichText::new("➕ Add Blank Slide").size(13.0).strong().color(Color32::WHITE))
                            .fill(AppTheme::accent_green())
                            .min_size(egui::vec2(ui.available_width(), 34.0)),
                    )
                    .on_hover_text("Insert a blank 3-second slide to fill with pictures, videos, text and audio")
                    .clicked()
                {
                    action = SlideBinAction::AddBlankSlide { duration: 3.0 };
                }
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            let active = app.active_slide().cloned();
            match active {
                Some(clip) => {
                    ui.label(
                        RichText::new(format!("🎬 Editing slide: {}", clip.name))
                            .size(13.0)
                            .strong()
                            .color(AppTheme::accent_yellow()),
                    );
                    ui.add_space(6.0);
                    Self::render_background(ui, &mut action);
                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);
                    Self::render_add_buttons(ui, app, &mut action);
                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);
                    Self::render_element_list(ui, &clip, &mut action);
                }
                None => {
                    ui.add_space(16.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("🎨 No slide selected")
                                .size(14.0)
                                .strong()
                                .color(AppTheme::text_secondary()),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("Click 'Add Blank Slide' above, or select a clip on the timeline, to start filling a slide.")
                                .size(12.0)
                                .color(AppTheme::text_muted()),
                        );
                    });
                }
            }
        });

        action
    }

    fn render_background(ui: &mut Ui, action: &mut SlideBinAction) {
        ui.label(
            RichText::new("Slide Background:").size(12.5).strong().color(Color32::WHITE),
        );
        let swatches = [
            ("⬛", Color32::from_rgb(12, 12, 16)),
            ("⬜", Color32::from_rgb(240, 240, 245)),
            ("🟦", Color32::from_rgb(15, 30, 60)),
            ("🟨", Color32::from_rgb(40, 34, 12)),
            ("🟥", Color32::from_rgb(55, 18, 25)),
        ];
        ui.horizontal_wrapped(|ui| {
            for (lbl, col) in swatches {
                if ui
                    .add(Button::new(RichText::new(lbl).size(13.0)).fill(col).min_size(egui::vec2(30.0, 30.0)))
                    .on_hover_text("Solid background colour")
                    .clicked()
                {
                    *action = SlideBinAction::SetActiveBackground(SlideBackground::Solid(col));
                }
            }
        });
        ui.horizontal(|ui| {
            if ui
                .add(Button::new(RichText::new("🖼 Pick Background Photo").size(12.0)))
                .clicked()
            {
                if let Some(p) = crate::media::probe::create_media_file_dialog().pick_file() {
                    *action = SlideBinAction::SetActiveBackground(SlideBackground::Picture(p));
                }
            }
        });
        ui.add_space(6.0);
    }

    fn render_add_buttons(ui: &mut Ui, app: &mut VideoEditorApp, action: &mut SlideBinAction) {
        ui.label(
            RichText::new("Add to this slide:").size(12.5).strong().color(Color32::WHITE),
        );

        // Text: arm placement with the current draft styling; the user then clicks the frame.
        if ui
            .add(Button::new(RichText::new("✏️  Text").size(13.0).strong()).min_size(egui::vec2((ui.available_width() - 6.0) / 2.0, 30.0)))
            .on_hover_text("Click 'Text', then click the frame to place the words where you want them")
            .clicked()
        {
            let overlay = app.text_draft.clone();
            *action = SlideBinAction::ArmPlace(crate::ui::PendingElement::Text(overlay));
        }

        ui.horizontal(|ui| {
            if ui.add(Button::new(RichText::new("🖼 Picture").size(12.5)).min_size(egui::vec2((ui.available_width() - 6.0) / 2.0, 30.0))).clicked() {
                if let Some(p) = crate::media::probe::create_media_file_dialog().pick_file() {
                    *action = SlideBinAction::ArmPlace(crate::ui::PendingElement::Picture(p));
                }
            }
            if ui.add(Button::new(RichText::new("🎞 Video").size(12.5)).min_size(egui::vec2(ui.available_width(), 30.0))).clicked() {
                if let Some(p) = crate::media::probe::create_media_file_dialog().pick_file() {
                    *action = SlideBinAction::ArmPlace(crate::ui::PendingElement::Video(p));
                }
            }
        });

        if ui
            .add(Button::new(RichText::new("🎵 Audio").size(13.0)).min_size(egui::vec2(ui.available_width(), 30.0)))
            .on_hover_text("Add a music/sound file that plays during this slide")
            .clicked()
        {
            if let Some(p) = crate::media::probe::create_media_file_dialog().pick_file() {
                *action = SlideBinAction::AddAudioElement(p);
            }
        }

        if app.pending_place.is_some() {
            ui.add_space(4.0);
            ui.label(
                RichText::new("👆 Now click on the preview frame to place it.")
                    .size(12.0)
                    .color(AppTheme::accent_cyan()),
            );
        }

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);
        ui.label(
            RichText::new("Text style (applies to new text):").size(12.0).color(AppTheme::text_secondary()),
        );
        Self::render_text_style(ui, app);
    }

    fn render_text_style(ui: &mut Ui, app: &mut VideoEditorApp) {
        let overlay = &mut app.text_draft;

        egui::ComboBox::from_id_salt("slide_text_font")
            .selected_text(RichText::new(format!("🔤 {}", overlay.font_family.label())).size(12.0))
            .width(ui.available_width() - 8.0)
            .show_ui(ui, |ui| {
                for f in crate::core::text_overlay::FontFamilyPreset::all() {
                    let is_sel = overlay.font_family == *f;
                    if ui.selectable_label(is_sel, format!("{}  -  {}", f.label(), f.preview_sample())).clicked() {
                        overlay.font_family = *f;
                    }
                }
            });

        ui.horizontal(|ui| {
            ui.label(RichText::new("Size:").size(11.5).color(AppTheme::text_secondary()));
            crate::ui::small_slider(ui, 12.0, |ui| {
                ui.add_sized([90.0, 12.0], egui::Slider::new(&mut overlay.font_size, 14.0..=120.0).step_by(2.0))
            });
        });

        ui.horizontal(|ui| {
            if ui
                .add(Button::new(RichText::new("B Bold").size(11.5).strong()).fill(if overlay.is_bold { AppTheme::accent_blue() } else { AppTheme::bg_panel() }))
                .clicked()
            {
                overlay.is_bold = !overlay.is_bold;
            }
            if ui
                .add(Button::new(RichText::new("I Italic").size(11.5)).fill(if overlay.is_italic { AppTheme::accent_blue() } else { AppTheme::bg_panel() }))
                .clicked()
            {
                overlay.is_italic = !overlay.is_italic;
            }
            ui.add_space(4.0);
            ui.label(RichText::new("Color:").size(11.5).color(AppTheme::text_secondary()));
            for c in [Color32::WHITE, Color32::from_rgb(15, 15, 15), Color32::from_rgb(0, 230, 255), Color32::from_rgb(255, 215, 90)] {
                if ui
                    .add(Button::new("").fill(c).min_size(egui::vec2(18.0, 18.0)))
                    .on_hover_text("Text colour")
                    .clicked()
                {
                    overlay.text_color = c;
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label(RichText::new("Background:").size(11.5).color(AppTheme::text_secondary()));
            for style in TextBoxStyle::all() {
                let is_sel = overlay.box_style == *style;
                if ui
                    .add(Button::new(RichText::new(style.label()).size(11.0)).fill(if is_sel { AppTheme::accent_blue() } else { AppTheme::bg_panel() }))
                    .clicked()
                {
                    overlay.box_style = *style;
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label(RichText::new("Words:").size(11.5).color(AppTheme::text_secondary()));
            ui.add_sized(
                [ui.available_width(), 26.0],
                egui::TextEdit::singleline(&mut overlay.text).hint_text("Type words..."),
            );
        });
    }

    fn render_element_list(ui: &mut Ui, clip: &crate::core::clip::Clip, action: &mut SlideBinAction) {
        ui.label(
            RichText::new("Items on this slide:").size(12.5).strong().color(Color32::WHITE),
        );
        if clip.elements.is_empty() {
            ui.label(
                RichText::new("Nothing here yet — add text, pictures, videos or audio above.")
                    .size(12.0)
                    .color(AppTheme::text_muted()),
            );
            return;
        }

        egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
            for (idx, el) in clip.elements.iter().enumerate() {
                ui.horizontal(|ui| {
                    let label = match el {
                        SlideElement::Text(o) => format!("✏️ Text: {}", o.text.lines().next().unwrap_or("")),
                        SlideElement::Picture { path, .. } => format!("🖼 {}", file_label(path)),
                        SlideElement::Video { path, .. } => format!("🎞 {}", file_label(path)),
                        SlideElement::Audio { path, .. } => format!("🎵 {}", file_label(path)),
                    };
                    ui.label(RichText::new(label).size(11.5).color(Color32::WHITE));
                    if ui.add(Button::new("⬆").frame(false)).clicked() {
                        *action = SlideBinAction::ReorderElement { idx, dir: -1 };
                    }
                    if ui.add(Button::new("⬇").frame(false)).clicked() {
                        *action = SlideBinAction::ReorderElement { idx, dir: 1 };
                    }
                    if ui.add(Button::new("🗑").frame(false)).clicked() {
                        *action = SlideBinAction::RemoveElement(idx);
                    }
                });
                if let SlideElement::Audio { volume, .. } = el {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Vol:").size(11.0).color(AppTheme::text_secondary()));
                        let mut v = *volume;
                        crate::ui::small_slider(ui, 12.0, |ui| {
                            if ui.add_sized([100.0, 12.0], egui::Slider::new(&mut v, 0.0..=2.0)).changed() {
                                *action = SlideBinAction::UpdateAudioVolume { idx, volume: v };
                            }
                        });
                    });
                }
            }
        });
    }
}

fn file_label(p: &Path) -> String {
    p.file_name().and_then(|n| n.to_str()).unwrap_or("File").to_string()
}
