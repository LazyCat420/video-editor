use crate::VideoEditorApp;
use crate::core::text_overlay::{SlideBackground, SlideElement, TextBoxStyle};
use crate::ui::theme::AppTheme;
use crate::ui::{SlideBinAction, SlideElementDrag};
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

                    // Add Tools row
                    Self::render_add_tools(ui, app, &mut action);
                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // Context-aware inspector: check if an element is selected on canvas
                    let sel_idx = app.selected_slide_element;
                    if let Some(idx) = sel_idx {
                        if let Some(element) = clip.elements.get(idx) {
                            Self::render_selected_element_inspector(ui, idx, element, &mut action);
                        } else {
                            Self::render_slide_background_and_overview(ui, &clip, app, &mut action);
                        }
                    } else {
                        Self::render_slide_background_and_overview(ui, &clip, app, &mut action);
                    }
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
                            RichText::new("Click 'Add Blank Slide' above, or select a clip on the timeline, to start editing.")
                                .size(12.0)
                                .color(AppTheme::text_muted()),
                        );
                    });
                }
            }
        });

        action
    }

    fn render_add_tools(ui: &mut Ui, app: &mut VideoEditorApp, action: &mut SlideBinAction) {
        ui.label(
            RichText::new("Slide Tools:").size(12.5).strong().color(Color32::WHITE),
        );

        // Click to add text: arm placement
        if ui
            .add(
                Button::new(RichText::new("✏️  Add Text Box").size(12.5).strong())
                    .min_size(egui::vec2(ui.available_width(), 30.0))
                    .fill(if app.pending_place.is_some() { AppTheme::accent_blue() } else { AppTheme::bg_card() }),
            )
            .on_hover_text("Click here, then click anywhere on the slide canvas to place text")
            .clicked()
        {
            let mut overlay = app.text_draft.clone();
            if overlay.text.trim().is_empty() {
                overlay.text = "Click to edit text".to_string();
            }
            *action = SlideBinAction::ArmPlace(crate::ui::PendingElement::Text(overlay));
        }

        ui.horizontal(|ui| {
            if ui
                .add(Button::new(RichText::new("🖼 Pick Photo").size(12.0)).min_size(egui::vec2((ui.available_width() - 6.0) / 2.0, 28.0)))
                .clicked()
            {
                if let Some(p) = crate::media::probe::create_media_file_dialog().pick_file() {
                    *action = SlideBinAction::ArmPlace(crate::ui::PendingElement::Picture(p));
                }
            }
            if ui
                .add(Button::new(RichText::new("🎞 Pick Video").size(12.0)).min_size(egui::vec2(ui.available_width(), 28.0)))
                .clicked()
            {
                if let Some(p) = crate::media::probe::create_media_file_dialog().pick_file() {
                    *action = SlideBinAction::ArmPlace(crate::ui::PendingElement::Video(p));
                }
            }
        });

        ui.horizontal(|ui| {
            if ui
                .add(Button::new(RichText::new("🎵 Add Audio").size(12.0)).min_size(egui::vec2(ui.available_width(), 28.0)))
                .on_hover_text("Add a music/sound file that plays during this slide")
                .clicked()
            {
                if let Some(p) = crate::media::probe::create_media_file_dialog().pick_file() {
                    *action = SlideBinAction::AddAudioElement(p);
                }
            }
        });

        if app.pending_place.is_some() {
            ui.add_space(4.0);
            ui.label(
                RichText::new("👆 Now click on the preview canvas to place it.")
                    .size(12.0)
                    .color(AppTheme::accent_cyan()),
            );
        }
    }

    fn render_selected_element_inspector(
        ui: &mut Ui,
        idx: usize,
        element: &SlideElement,
        action: &mut SlideBinAction,
    ) {
        match element {
            SlideElement::Text(overlay) => {
                let mut updated = overlay.clone();
                let mut changed = false;

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("✏️ Selected Text")
                            .size(13.0)
                            .strong()
                            .color(AppTheme::accent_cyan()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✖ Deselect").clicked() {
                            *action = SlideBinAction::SelectElement(None);
                        }
                    });
                });
                ui.add_space(4.0);

                ui.label(RichText::new("Text Words:").size(11.5).color(AppTheme::text_secondary()));
                let text_resp = ui.add_sized(
                    [ui.available_width(), 50.0],
                    egui::TextEdit::multiline(&mut updated.text).hint_text("Type words..."),
                );
                if text_resp.changed() {
                    changed = true;
                }

                ui.add_space(4.0);
                egui::ComboBox::from_id_salt("sel_slide_text_font")
                    .selected_text(RichText::new(format!("🔤 {}", updated.font_family.label())).size(12.0))
                    .width(ui.available_width() - 8.0)
                    .show_ui(ui, |ui| {
                        for f in crate::core::text_overlay::FontFamilyPreset::all() {
                            let is_sel = updated.font_family == *f;
                            if ui.selectable_label(is_sel, format!("{}  -  {}", f.label(), f.preview_sample())).clicked() {
                                updated.font_family = *f;
                                changed = true;
                            }
                        }
                    });

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Size:").size(11.5).color(AppTheme::text_secondary()));
                    crate::ui::small_slider(ui, 12.0, |ui| {
                        if ui.add_sized([90.0, 12.0], egui::Slider::new(&mut updated.font_size, 14.0..=120.0).step_by(2.0)).changed() {
                            changed = true;
                        }
                    });
                });

                ui.horizontal(|ui| {
                    if ui
                        .add(Button::new(RichText::new("B Bold").size(11.5).strong()).fill(if updated.is_bold { AppTheme::accent_blue() } else { AppTheme::bg_panel() }))
                        .clicked()
                    {
                        updated.is_bold = !updated.is_bold;
                        changed = true;
                    }
                    if ui
                        .add(Button::new(RichText::new("I Italic").size(11.5)).fill(if updated.is_italic { AppTheme::accent_blue() } else { AppTheme::bg_panel() }))
                        .clicked()
                    {
                        updated.is_italic = !updated.is_italic;
                        changed = true;
                    }
                    ui.add_space(4.0);
                    ui.label(RichText::new("Color:").size(11.5).color(AppTheme::text_secondary()));
                    for c in [Color32::WHITE, Color32::from_rgb(15, 15, 15), Color32::from_rgb(0, 230, 255), Color32::from_rgb(255, 215, 90)] {
                        if ui
                            .add(Button::new("").fill(c).min_size(egui::vec2(18.0, 18.0)))
                            .on_hover_text("Text colour")
                            .clicked()
                        {
                            updated.text_color = c;
                            changed = true;
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Background:").size(11.5).color(AppTheme::text_secondary()));
                    for style in TextBoxStyle::all() {
                        let is_sel = updated.box_style == *style;
                        if ui
                            .add(Button::new(RichText::new(style.label()).size(11.0)).fill(if is_sel { AppTheme::accent_blue() } else { AppTheme::bg_panel() }))
                            .clicked()
                        {
                            updated.box_style = *style;
                            changed = true;
                        }
                    }
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("⬆ Move Up").clicked() {
                        *action = SlideBinAction::ReorderElement { idx, dir: -1 };
                    }
                    if ui.button("⬇ Move Down").clicked() {
                        *action = SlideBinAction::ReorderElement { idx, dir: 1 };
                    }
                    if ui.button("🗑 Delete Text").clicked() {
                        *action = SlideBinAction::RemoveElement(idx);
                    }
                });

                if changed {
                    *action = SlideBinAction::UpdateElement {
                        idx,
                        element: SlideElement::Text(updated),
                    };
                }
            }
            SlideElement::Picture { path, .. } => {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("🖼 Picture: {}", file_label(path)))
                            .size(13.0)
                            .strong()
                            .color(AppTheme::accent_cyan()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✖ Deselect").clicked() {
                            *action = SlideBinAction::SelectElement(None);
                        }
                    });
                });
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    if ui.button("⛶ Fill Entire Slide").clicked() {
                        *action = SlideBinAction::FullSlide(idx);
                    }
                    if ui.button("🖼 Set as Slide Background").clicked() {
                        *action = SlideBinAction::SetElementAsBackground(idx);
                    }
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("⬆ Move Up").clicked() {
                        *action = SlideBinAction::ReorderElement { idx, dir: -1 };
                    }
                    if ui.button("⬇ Move Down").clicked() {
                        *action = SlideBinAction::ReorderElement { idx, dir: 1 };
                    }
                    if ui.button("🗑 Delete Picture").clicked() {
                        *action = SlideBinAction::RemoveElement(idx);
                    }
                });
            }
            SlideElement::Video { path, .. } => {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("🎞 Video: {}", file_label(path)))
                            .size(13.0)
                            .strong()
                            .color(AppTheme::accent_cyan()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✖ Deselect").clicked() {
                            *action = SlideBinAction::SelectElement(None);
                        }
                    });
                });
                ui.add_space(6.0);

                if ui.button("⛶ Fill Entire Slide").clicked() {
                    *action = SlideBinAction::FullSlide(idx);
                }

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("⬆ Move Up").clicked() {
                        *action = SlideBinAction::ReorderElement { idx, dir: -1 };
                    }
                    if ui.button("⬇ Move Down").clicked() {
                        *action = SlideBinAction::ReorderElement { idx, dir: 1 };
                    }
                    if ui.button("🗑 Delete Video").clicked() {
                        *action = SlideBinAction::RemoveElement(idx);
                    }
                });
            }
            SlideElement::Audio { path, volume } => {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("🎵 Audio: {}", file_label(path)))
                            .size(13.0)
                            .strong()
                            .color(AppTheme::accent_cyan()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✖ Deselect").clicked() {
                            *action = SlideBinAction::SelectElement(None);
                        }
                    });
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Volume:").size(11.5).color(AppTheme::text_secondary()));
                    let mut v = *volume;
                    crate::ui::small_slider(ui, 12.0, |ui| {
                        if ui.add_sized([100.0, 12.0], egui::Slider::new(&mut v, 0.0..=2.0)).changed() {
                            *action = SlideBinAction::UpdateAudioVolume { idx, volume: v };
                        }
                    });
                });
                if ui.button("🗑 Remove Audio").clicked() {
                    *action = SlideBinAction::RemoveElement(idx);
                }
            }
        }
    }

    fn render_slide_background_and_overview(
        ui: &mut Ui,
        clip: &crate::core::clip::Clip,
        app: &mut VideoEditorApp,
        action: &mut SlideBinAction,
    ) {
        Self::render_background(ui, action);
        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);
        Self::render_element_list(ui, clip, app, action);
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
        ui.add_space(4.0);
        ui.label(
            RichText::new("💡 Tip: Drag images/videos from Files panel directly onto the canvas.")
                .size(11.0)
                .color(AppTheme::text_muted()),
        );
    }

    fn render_element_list(
        ui: &mut Ui,
        clip: &crate::core::clip::Clip,
        app: &VideoEditorApp,
        action: &mut SlideBinAction,
    ) {
        ui.label(
            RichText::new("Items on this slide:").size(12.5).strong().color(Color32::WHITE),
        );
        if clip.elements.is_empty() {
            ui.label(
                RichText::new("Nothing here yet — click 'Add Text Box' or drag files onto the canvas.")
                    .size(12.0)
                    .color(AppTheme::text_muted()),
            );
            return;
        }

        let active_drag_idx = egui::DragAndDrop::payload::<SlideElementDrag>(ui.ctx()).map(|p| p.0);

        egui::ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
            let render_slide_drop_slot = |ui: &mut Ui, target_idx: usize, action: &mut SlideBinAction| {
                if let Some(_drag_from) = active_drag_idx {
                    let (slot_rect, slot_resp) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 22.0),
                        egui::Sense::hover(),
                    );
                    let is_hovered = slot_resp.dnd_hover_payload::<SlideElementDrag>().is_some()
                        || (slot_resp.hovered() && active_drag_idx.is_some());
                    let sp = ui.painter_at(slot_rect);
                    if is_hovered {
                        sp.rect_filled(
                            slot_rect,
                            egui::Rounding::same(4.0),
                            Color32::from_rgba_premultiplied(0, 140, 255, 45),
                        );
                        sp.rect_stroke(
                            slot_rect,
                            egui::Rounding::same(4.0),
                            egui::Stroke::new(1.5, AppTheme::accent_blue()),
                        );
                        sp.text(
                            slot_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("⬇ Move to Layer #{}", target_idx + 1),
                            egui::FontId::proportional(10.5),
                            AppTheme::accent_blue(),
                        );
                    } else {
                        sp.hline(
                            slot_rect.x_range(),
                            slot_rect.center().y,
                            egui::Stroke::new(1.0, Color32::from_rgba_premultiplied(0, 140, 255, 50)),
                        );
                    }
                    if let Some(released) = slot_resp.dnd_release_payload::<SlideElementDrag>() {
                        if released.0 != target_idx {
                            *action = SlideBinAction::ReorderElementTo {
                                from_idx: released.0,
                                to_idx: target_idx,
                            };
                        }
                    }
                }
            };

            for (idx, el) in clip.elements.iter().enumerate() {
                render_slide_drop_slot(ui, idx, action);

                let is_sel = app.selected_slide_element == Some(idx);
                let is_this_dragged = active_drag_idx == Some(idx);

                ui.horizontal(|ui| {
                    let label = match el {
                        SlideElement::Text(o) => format!("✏️ Text: {}", o.text.lines().next().unwrap_or("")),
                        SlideElement::Picture { path, .. } => format!("🖼 {}", file_label(path)),
                        SlideElement::Video { path, .. } => format!("🎞 {}", file_label(path)),
                        SlideElement::Audio { path, .. } => format!("🎵 {}", file_label(path)),
                    };

                    let handle_resp = ui.add(
                        Button::new(RichText::new("⠿").size(12.0).color(if is_this_dragged { Color32::from_gray(80) } else { AppTheme::text_secondary() }))
                            .frame(false)
                    );
                    let handle_resp = handle_resp.interact(egui::Sense::click_and_drag());
                    handle_resp.dnd_set_drag_payload(SlideElementDrag(idx));
                    if handle_resp.dragged() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                    } else if handle_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                    }

                    let btn = Button::new(
                        RichText::new(label)
                            .size(11.5)
                            .color(if is_this_dragged { Color32::from_gray(100) } else if is_sel { AppTheme::accent_yellow() } else { Color32::WHITE }),
                    )
                    .fill(if is_this_dragged { Color32::from_rgba_premultiplied(20, 24, 32, 140) } else if is_sel { AppTheme::bg_hover() } else { AppTheme::bg_card() });

                    if ui.add(btn).on_hover_text("Click to select and format this item").clicked() {
                        *action = SlideBinAction::SelectElement(Some(idx));
                    }
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
            }

            if !clip.elements.is_empty() {
                let last = clip.elements.len();
                render_slide_drop_slot(ui, last, action);
            }

            // Floating drag ghost for slide elements
            if let Some(drag_idx) = active_drag_idx {
                if let Some(dragged_el) = clip.elements.get(drag_idx) {
                    if let Some(pointer_pos) = ui.input(|i| i.pointer.hover_pos()) {
                        let ghost_size = egui::vec2(190.0, 32.0);
                        let ghost_rect = egui::Rect::from_min_size(pointer_pos + egui::vec2(10.0, 10.0), ghost_size);
                        let painter = ui.ctx().layer_painter(egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("dnd_ghost_slide_el")));
                        painter.rect_filled(ghost_rect, egui::Rounding::same(6.0), Color32::from_rgba_premultiplied(24, 29, 40, 235));
                        painter.rect_stroke(ghost_rect, egui::Rounding::same(6.0), egui::Stroke::new(1.5, AppTheme::accent_blue()));
                        let ghost_label = match dragged_el {
                            SlideElement::Text(o) => format!("✏️ Text: {}", o.text.lines().next().unwrap_or("")),
                            SlideElement::Picture { path, .. } => format!("🖼 {}", file_label(path)),
                            SlideElement::Video { path, .. } => format!("🎞 {}", file_label(path)),
                            SlideElement::Audio { path, .. } => format!("🎵 {}", file_label(path)),
                        };
                        painter.text(
                            egui::pos2(ghost_rect.min.x + 8.0, ghost_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            ghost_label,
                            egui::FontId::proportional(11.5),
                            AppTheme::accent_blue(),
                        );
                    }
                }
            }
        });
    }
}

fn file_label(p: &Path) -> String {
    p.file_name().and_then(|n| n.to_str()).unwrap_or("File").to_string()
}

