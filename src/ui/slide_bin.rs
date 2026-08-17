use crate::VideoEditorApp;
use crate::core::calendar_gen::{CalendarMonth, CustomCalendarEvent, HolidayCategory};
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
                    .on_hover_text("Insert a blank 5-second slide to fill with pictures, videos, text and audio")
                    .clicked()
                {
                    action = SlideBinAction::AddBlankSlide { duration: 5.0 };
                }
            });

            ui.add_space(8.0);
            ui.label(
                RichText::new("📋 Preset Slide Layouts")
                    .size(12.5)
                    .strong()
                    .color(AppTheme::accent_cyan()),
            );
            ui.add_space(2.0);

            ui.columns(3, |cols| {
                let btn_t2 = Button::new(RichText::new("📰 Title+2").size(10.5).strong())
                    .fill(AppTheme::bg_card())
                    .min_size(egui::vec2(cols[0].available_width(), 26.0));
                if cols[0].add(btn_t2).on_hover_text("Apply Title + 2 Media layout to the active slide").clicked() {
                    action = SlideBinAction::ApplyTemplateTitle2MediaToActive;
                }

                let btn_t4 = Button::new(RichText::new("🪟 4 Grid").size(10.5).strong())
                    .fill(AppTheme::bg_card())
                    .min_size(egui::vec2(cols[1].available_width(), 26.0));
                if cols[1].add(btn_t4).on_hover_text("Apply Title + 4 Grid layout to the active slide").clicked() {
                    action = SlideBinAction::ApplyTemplateTitle4MediaToActive;
                }

                let btn_sc = Button::new(RichText::new("🌟 Show").size(10.5).strong())
                    .fill(AppTheme::bg_card())
                    .min_size(egui::vec2(cols[2].available_width(), 26.0));
                if cols[2].add(btn_sc).on_hover_text("Apply Feature Showcase layout to the active slide").clicked() {
                    action = SlideBinAction::ApplyTemplateShowcaseToActive;
                }
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                egui::CollapsingHeader::new(RichText::new("⚙️ Calendar & Holiday Settings").size(12.5).strong().color(AppTheme::accent_yellow()))
                    .default_open(true)
                    .show(ui, |ui| {
                        Self::render_calendar_config_panel(ui, app, &mut action, false);
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

                        Self::render_add_tools(ui, app, &mut action);
                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);

                        let sel_idx = app.selected_slide_element;
                        if let Some(idx) = sel_idx {
                            if let Some(element) = clip.elements.get(idx) {
                                Self::render_selected_element_inspector(ui, idx, element, app, &mut action);
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
        });

        action
    }

    /// One holiday row: a checkbox label with the colour swatch pinned to the right edge.
    ///
    /// Laid out right-to-left so the swatch is placed first and the label is confined to the
    /// space left over. A long name ("Columbus / Indigenous Peoples Day") then stays inside
    /// the row instead of growing it; an overflowing row widens the whole SidePanel, which
    /// paints as a dead gap between the sidebar and the preview.
    fn holiday_row(ui: &mut Ui, name: &str, enabled: &mut bool, color: &mut Color32) -> bool {
        let mut changed = false;
        // Allocate one row's worth of height explicitly: a bare right-to-left layout would
        // claim all the height still available in the panel, leaving one holiday per screen.
        let row_h = ui.spacing().interact_size.y.max(18.0);
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), row_h),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                if ui.color_edit_button_srgba(color).changed() {
                    changed = true;
                }
                ui.add_space(4.0);
                // Draw the checkbox left-aligned in whatever the swatch left behind, so the
                // names still form a straight left edge under the section header.
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    if ui
                        .checkbox(enabled, RichText::new(name).size(11.5))
                        .changed()
                    {
                        changed = true;
                    }
                });
            },
        );
        changed
    }

    /// Comprehensive Calendar and Holiday Configuration Panel
    pub fn render_calendar_config_panel(
        ui: &mut Ui,
        app: &mut VideoEditorApp,
        action: &mut SlideBinAction,
        in_element_inspector: bool,
    ) {
        ui.vertical(|ui| {
            // 1. Months in Slide Selector (Clean row layout, never cut off)
            ui.label(RichText::new("Months in Slide:").size(12.0).strong().color(AppTheme::text_primary()));
            ui.add_space(2.0);
            ui.columns(3, |cols| {
                for (idx, count) in [1, 2, 3].iter().enumerate() {
                    let label = format!("{} Month{}", count, if *count > 1 { "s" } else { "" });
                    let is_active = app.calendar_month_count == *count;
                    let btn = Button::new(RichText::new(label).size(11.0).strong())
                        .fill(if is_active { AppTheme::accent_blue() } else { AppTheme::bg_card() })
                        .min_size(egui::vec2(cols[idx].available_width(), 26.0));
                    if cols[idx].add(btn).clicked() {
                        app.calendar_month_count = *count;
                        if in_element_inspector {
                            *action = SlideBinAction::UpdateActiveCalendarSlide;
                        }
                    }
                }
            });

            ui.add_space(5.0);

            // 2. Direct Add Calendar to Slide (Full-width button, NEVER cut off)
            let add_btn = Button::new(
                RichText::new("➕ Add Calendar to Slide")
                    .size(12.5)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(AppTheme::accent_green())
            .min_size(egui::vec2(ui.available_width(), 30.0));

            if ui.add(add_btn)
                .on_hover_text("Add or update the calendar grid on the active slide")
                .clicked()
            {
                *action = SlideBinAction::ApplyTemplateCalendarSlideToActive {
                    year: app.calendar_year,
                    start_month: app.calendar_start_month,
                    month_count: app.calendar_month_count,
                    show_holidays: app.calendar_show_holidays,
                };
            }

            ui.add_space(5.0);

            // 3. Full-Year 12-Month Calendar & Print Buttons (2 equal columns)
            ui.columns(2, |cols| {
                let full_year_btn = Button::new(RichText::new("📅 12-Month").size(11.0).strong().color(Color32::WHITE))
                    .fill(Color32::from_rgb(40, 70, 120))
                    .min_size(egui::vec2(cols[0].available_width(), 26.0));
                if cols[0].add(full_year_btn)
                    .on_hover_text("Generate calendar slides covering the entire year based on Months in Slide")
                    .clicked()
                {
                    *action = SlideBinAction::Generate12MonthCalendar {
                        year: app.calendar_year,
                        month_count: app.calendar_month_count,
                        show_holidays: app.calendar_show_holidays,
                    };
                }

                let print_btn = Button::new(RichText::new("🖨 Print").size(11.0).color(AppTheme::accent_green()))
                    .fill(AppTheme::bg_card())
                    .min_size(egui::vec2(cols[1].available_width(), 26.0));
                if cols[1].add(print_btn)
                    .on_hover_text("Export high-resolution landscape printable wall calendar sheets")
                    .clicked()
                {
                    *action = SlideBinAction::OpenCalendarExportDialog;
                }
            });

            ui.add_space(6.0);

            // 2. Year & Start Month Selectors
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Year:").size(11.5).color(AppTheme::text_secondary()));
                if ui.button("◀").clicked() {
                    app.calendar_year -= 1;
                    app.calendar_holidays = CalendarMonth::default_holidays_for_year(app.calendar_year);
                    if in_element_inspector {
                        *action = SlideBinAction::UpdateActiveCalendarSlide;
                    }
                }
                ui.label(RichText::new(format!("{}", app.calendar_year)).size(13.0).strong().color(Color32::WHITE));
                if ui.button("▶").clicked() {
                    app.calendar_year += 1;
                    app.calendar_holidays = CalendarMonth::default_holidays_for_year(app.calendar_year);
                    if in_element_inspector {
                        *action = SlideBinAction::UpdateActiveCalendarSlide;
                    }
                }
            });

            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Start:").size(11.5).color(AppTheme::text_secondary()));
                egui::ComboBox::from_id_salt("cal_start_month_select")
                    .selected_text(RichText::new(CalendarMonth::name_for_month(app.calendar_start_month)).size(11.5))
                    .width(ui.available_width().min(160.0))
                    .show_ui(ui, |ui| {
                        for m in 1..=12 {
                            let is_sel = app.calendar_start_month == m;
                            if ui.selectable_label(is_sel, CalendarMonth::name_for_month(m)).clicked() {
                                app.calendar_start_month = m;
                                if in_element_inspector {
                                    *action = SlideBinAction::UpdateActiveCalendarSlide;
                                }
                            }
                        }
                    });
            });

            // 3. Master Holiday Toggle & Quick Actions
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.checkbox(&mut app.calendar_show_holidays, RichText::new("Show Holidays on Calendar").size(12.0).strong().color(AppTheme::accent_yellow())).changed() {
                    if in_element_inspector {
                        *action = SlideBinAction::UpdateActiveCalendarSlide;
                    }
                }
            });

            if app.calendar_show_holidays {
                ui.add_space(4.0);
                ui.columns(3, |cols| {
                    if cols[0].add(Button::new(RichText::new("🇺🇸 All US").size(10.5)).min_size(egui::vec2(cols[0].available_width(), 22.0))).clicked() {
                        for h in &mut app.calendar_holidays {
                            if h.category == HolidayCategory::American {
                                h.enabled = true;
                            }
                        }
                        if in_element_inspector {
                            *action = SlideBinAction::UpdateActiveCalendarSlide;
                        }
                    }
                    if cols[1].add(Button::new(RichText::new("🧧 All Chinese").size(10.5)).min_size(egui::vec2(cols[1].available_width(), 22.0))).clicked() {
                        for h in &mut app.calendar_holidays {
                            if h.category == HolidayCategory::Chinese {
                                h.enabled = true;
                            }
                        }
                        if in_element_inspector {
                            *action = SlideBinAction::UpdateActiveCalendarSlide;
                        }
                    }
                    if cols[2].add(Button::new(RichText::new("🔄 Reset").size(10.5)).min_size(egui::vec2(cols[2].available_width(), 22.0))).clicked() {
                        app.calendar_holidays = CalendarMonth::default_holidays_for_year(app.calendar_year);
                        if in_element_inspector {
                            *action = SlideBinAction::UpdateActiveCalendarSlide;
                        }
                    }
                });
                ui.add_space(2.0);
                if ui.add(Button::new(RichText::new("✖ Clear All").size(10.5)).min_size(egui::vec2(ui.available_width(), 22.0))).clicked() {
                    for h in &mut app.calendar_holidays {
                        h.enabled = false;
                    }
                    if in_element_inspector {
                        *action = SlideBinAction::UpdateActiveCalendarSlide;
                    }
                }

                ui.add_space(4.0);

                // Collapsible 🇺🇸 American Holidays
                egui::CollapsingHeader::new(RichText::new("🇺🇸 American Holidays").size(11.5).strong().color(Color32::from_rgb(140, 180, 255)))
                    .default_open(true)
                    .show(ui, |ui| {
                        for h in &mut app.calendar_holidays {
                            if h.category == HolidayCategory::American {
                                let mut col = h.color32();
                                if Self::holiday_row(ui, &h.name, &mut h.enabled, &mut col) {
                                    h.set_color32(col);
                                    if in_element_inspector {
                                        *action = SlideBinAction::UpdateActiveCalendarSlide;
                                    }
                                }
                            }
                        }
                    });

                // Collapsible 🧧 Chinese Festivals
                egui::CollapsingHeader::new(RichText::new("🧧 Chinese Festivals & Holidays").size(11.5).strong().color(Color32::from_rgb(255, 100, 100)))
                    .default_open(true)
                    .show(ui, |ui| {
                        for h in &mut app.calendar_holidays {
                            if h.category == HolidayCategory::Chinese {
                                let mut col = h.color32();
                                if Self::holiday_row(ui, &h.name, &mut h.enabled, &mut col) {
                                    h.set_color32(col);
                                    if in_element_inspector {
                                        *action = SlideBinAction::UpdateActiveCalendarSlide;
                                    }
                                }
                            }
                        }
                    });

                // Collapsible ⭐ Custom Family Events
                egui::CollapsingHeader::new(RichText::new("⭐ Custom Family Events & Birthdays").size(11.5).strong().color(Color32::from_rgb(255, 215, 0)))
                    .default_open(false)
                    .show(ui, |ui| {
                        let mut to_remove = None;
                        for (i, ev) in app.calendar_custom_events.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("{} {}: {}", CalendarMonth::short_name_for_month(ev.month), ev.day, ev.label)).size(11.0));
                                let mut c = Color32::from_rgba_premultiplied(ev.color[0], ev.color[1], ev.color[2], ev.color[3]);
                                if ui.color_edit_button_srgba(&mut c).changed() {
                                    ev.color = [c.r(), c.g(), c.b(), c.a()];
                                    if in_element_inspector {
                                        *action = SlideBinAction::UpdateActiveCalendarSlide;
                                    }
                                }
                                if ui.button("🗑").clicked() {
                                    to_remove = Some(i);
                                }
                            });
                        }
                        if let Some(i) = to_remove {
                            app.calendar_custom_events.remove(i);
                            if in_element_inspector {
                                *action = SlideBinAction::UpdateActiveCalendarSlide;
                            }
                        }

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label("Month:");
                            ui.add(egui::DragValue::new(&mut app.new_custom_event_month).range(1..=12));
                            ui.label("Day:");
                            ui.add(egui::DragValue::new(&mut app.new_custom_event_day).range(1..=31));
                        });
                        ui.horizontal(|ui| {
                            // Leave room for the colour swatch + gap: a
                            // full-available-width TextEdit pushed the swatch
                            // past the width budget (dead-gap overflow).
                            ui.add(
                                egui::TextEdit::singleline(&mut app.new_custom_event_label)
                                    .hint_text("Grandma's Birthday")
                                    .desired_width(ui.available_width() - 44.0),
                            );
                            let mut c = Color32::from_rgba_premultiplied(app.new_custom_event_color[0], app.new_custom_event_color[1], app.new_custom_event_color[2], app.new_custom_event_color[3]);
                            if ui.color_edit_button_srgba(&mut c).changed() {
                                app.new_custom_event_color = [c.r(), c.g(), c.b(), c.a()];
                            }
                        });
                        if ui.button("➕ Add Custom Event").clicked() && !app.new_custom_event_label.is_empty() {
                            app.calendar_custom_events.push(CustomCalendarEvent {
                                month: app.new_custom_event_month,
                                day: app.new_custom_event_day,
                                label: app.new_custom_event_label.clone(),
                                color: app.new_custom_event_color,
                            });
                            app.new_custom_event_label = "Family Event".to_string();
                            if in_element_inspector {
                                *action = SlideBinAction::UpdateActiveCalendarSlide;
                            }
                        }
                    });
            }

            // 4. Update / Re-render Action Button
            if in_element_inspector {
                ui.add_space(6.0);
                let update_btn = Button::new(RichText::new("🔄 Re-render Calendar on Slide").strong().color(Color32::WHITE))
                    .fill(AppTheme::accent_blue())
                    .min_size(egui::vec2(ui.available_width(), 28.0));
                if ui.add(update_btn).clicked() {
                    *action = SlideBinAction::UpdateActiveCalendarSlide;
                }
            }
        });
    }

    fn render_add_tools(ui: &mut Ui, _app: &mut VideoEditorApp, action: &mut SlideBinAction) {
        ui.horizontal_wrapped(|ui| {
            if ui
                .add(Button::new(RichText::new("🔤 Add Text Box").size(12.0).strong().color(Color32::WHITE)).fill(AppTheme::accent_blue()))
                .on_hover_text("Add words or titles to the current slide")
                .clicked()
            {
                let mut overlay = crate::core::text_overlay::TextOverlay::new("Type text here");
                overlay.x = 0.5;
                overlay.y = 0.5;
                overlay.font_size = 28.0;
                overlay.alignment = crate::core::text_overlay::TextAlignment::Center;
                overlay.box_style = TextBoxStyle::TranslucentBox;
                *action = SlideBinAction::AddTextElement(overlay);
            }
        });
    }

    fn render_slide_background_and_overview(
        ui: &mut Ui,
        clip: &crate::core::Clip,
        _app: &mut VideoEditorApp,
        action: &mut SlideBinAction,
    ) {
        ui.label(RichText::new("🎨 Slide Background Color").size(12.5).strong().color(AppTheme::accent_cyan()));
        ui.add_space(4.0);

        let colors: [(&str, Color32); 16] = [
            ("Dark Slate", Color32::from_rgb(18, 20, 24)),
            ("Pure Black", Color32::BLACK),
            ("Charcoal", Color32::from_rgb(32, 34, 38)),
            ("Deep Navy", Color32::from_rgb(15, 23, 42)),
            ("Midnight Blue", Color32::from_rgb(10, 15, 30)),
            ("Emerald Green", Color32::from_rgb(6, 78, 59)),
            ("Forest Green", Color32::from_rgb(20, 83, 45)),
            ("Wine Red", Color32::from_rgb(136, 19, 55)),
            ("Crimson", Color32::from_rgb(159, 18, 57)),
            ("Deep Purple", Color32::from_rgb(88, 28, 135)),
            ("Royal Violet", Color32::from_rgb(107, 33, 168)),
            ("Hot Pink", Color32::from_rgb(190, 24, 93)),
            ("Pastel Pink", Color32::from_rgb(244, 114, 182)),
            ("Amber Gold", Color32::from_rgb(180, 83, 9)),
            ("Bright Yellow", Color32::from_rgb(234, 179, 8)),
            ("Clean White", Color32::WHITE),
        ];

        ui.horizontal_wrapped(|ui| {
            for (label, col) in colors {
                let is_current = match &clip.background {
                    Some(SlideBackground::Solid(c)) => *c == col,
                    _ => false,
                };
                let btn = Button::new("").fill(col).min_size(egui::vec2(22.0, 22.0)).stroke(egui::Stroke::new(
                    if is_current { 2.5 } else { 0.5 },
                    if is_current { AppTheme::accent_cyan() } else { Color32::from_white_alpha(40) },
                ));
                if ui.add(btn).on_hover_text(label).clicked() {
                    *action = SlideBinAction::SetActiveBackground(SlideBackground::Solid(col));
                }
            }
        });

        ui.add_space(8.0);
        ui.label(RichText::new(format!("📑 Slide Elements ({})", clip.elements.len())).size(12.5).strong().color(AppTheme::accent_cyan()));
        ui.add_space(4.0);

        if clip.elements.is_empty() {
            ui.label(RichText::new("Drag & drop photos or videos from Files panel onto the preview canvas to add them here.")
                .size(11.5).color(AppTheme::text_muted()));
        } else {
            for (idx, el) in clip.elements.iter().enumerate() {
                ui.horizontal(|ui| {
                    // Snippets are truncated and the button text sized down:
                    // these rows sit in a non-wrapping horizontal layout, so an
                    // unbounded label (long filename, long text line) is exactly
                    // what used to widen the panel and open the dead gap.
                    let desc = match el {
                        SlideElement::Text(t) => format!("🔤 Text: \"{}\"", truncate_chars(t.text.lines().next().unwrap_or(""), 18)),
                        SlideElement::Calendar(c) => format!("📅 Calendar: {} {}", CalendarMonth::name_for_month(c.start_month), c.year),
                        SlideElement::Picture { path, .. } => format!("🖼 Picture: {}", file_label(path)),
                        SlideElement::Video { path, .. } => format!("🎬 Video: {}", file_label(path)),
                        SlideElement::Audio { path, .. } => format!("🎵 Audio: {}", file_label(path)),
                        SlideElement::Placeholder { slot_id, label, .. } => format!("➕ Slot #{}: {}", slot_id, label),
                    };
                    if ui.button(RichText::new(desc).size(12.0)).clicked() {
                        *action = SlideBinAction::SelectElement(Some(idx));
                    }
                });
            }
        }
    }

    /// Inspector header: fixed title + Deselect on one row, then the filename
    /// on its own line, where the vertical layout wraps it instead of letting
    /// it extend the panel (a filename inside the header row was one of the
    /// dead-gap causes).
    fn inspector_media_header(
        ui: &mut Ui,
        title: &str,
        color: Color32,
        path: &Path,
        action: &mut SlideBinAction,
    ) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(title).size(13.0).strong().color(color));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✖ Deselect").clicked() {
                    *action = SlideBinAction::SelectElement(None);
                }
            });
        });
        ui.label(RichText::new(file_label(path)).size(10.5).color(AppTheme::text_muted()));
    }

    fn render_selected_element_inspector(
        ui: &mut Ui,
        idx: usize,
        element: &SlideElement,
        app: &mut VideoEditorApp,
        action: &mut SlideBinAction,
    ) {
        match element {
            SlideElement::Calendar(cal) => {
                let mut updated = cal.clone();
                let mut changed = false;

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("📅 Selected Calendar Box")
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

                // Months in Slide: 1, 2, 3 — tight padding, the default-padded
                // row measured over the 254px budget (dead-gap overflow).
                ui.scope(|ui| {
                    ui.spacing_mut().button_padding.x = 6.0;
                    ui.spacing_mut().item_spacing.x = 6.0;
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Months:").size(11.5).strong().color(AppTheme::text_secondary()));
                        for count in [1, 2, 3] {
                            let label = format!("{} Month{}", count, if count > 1 { "s" } else { "" });
                            let is_active = updated.month_count == count;
                            let btn = Button::new(RichText::new(label).size(10.5).strong())
                                .fill(if is_active { AppTheme::accent_blue() } else { AppTheme::bg_card() });
                            if ui.add(btn).clicked() {
                                updated.month_count = count;
                                changed = true;
                            }
                        }
                    });
                });
                ui.add_space(2.0);

                // Year and Starting Month on separate rows: combined they
                // measured ~345px against the 254px budget (dead gap).
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Year:").size(11.5).color(AppTheme::text_secondary()));
                    if ui.button("◀").clicked() {
                        updated.year -= 1;
                        updated.holidays = CalendarMonth::default_holidays_for_year(updated.year);
                        changed = true;
                    }
                    ui.label(RichText::new(format!("{}", updated.year)).size(12.5).strong().color(Color32::WHITE));
                    if ui.button("▶").clicked() {
                        updated.year += 1;
                        updated.holidays = CalendarMonth::default_holidays_for_year(updated.year);
                        changed = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Start:").size(11.5).color(AppTheme::text_secondary()));
                    egui::ComboBox::from_id_salt("cal_insp_month_combo")
                        .selected_text(CalendarMonth::name_for_month(updated.start_month))
                        .width(160.0)
                        .show_ui(ui, |ui| {
                            for m in 1..=12 {
                                let is_sel = updated.start_month == m;
                                if ui.selectable_label(is_sel, CalendarMonth::name_for_month(m)).clicked() {
                                    updated.start_month = m;
                                    changed = true;
                                }
                            }
                        });
                });
                ui.add_space(2.0);

                // Toggle Holidays
                if ui.checkbox(&mut updated.show_holidays, "Show Holidays on Calendar").changed() {
                    changed = true;
                }
                ui.add_space(4.0);

                // Position & Size Steppers
                ui.horizontal(|ui| {
                    ui.label(RichText::new("X:").size(11.0).color(AppTheme::text_muted()));
                    if ui.add(egui::DragValue::new(&mut updated.x).speed(0.01).range(0.0..=1.0)).changed() {
                        changed = true;
                    }
                    ui.label(RichText::new("Y:").size(11.0).color(AppTheme::text_muted()));
                    if ui.add(egui::DragValue::new(&mut updated.y).speed(0.01).range(0.0..=1.0)).changed() {
                        changed = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("W:").size(11.0).color(AppTheme::text_muted()));
                    if ui.add(egui::DragValue::new(&mut updated.w).speed(0.01).range(0.05..=1.0)).changed() {
                        changed = true;
                    }
                    ui.label(RichText::new("H:").size(11.0).color(AppTheme::text_muted()));
                    if ui.add(egui::DragValue::new(&mut updated.h).speed(0.01).range(0.05..=1.0)).changed() {
                        changed = true;
                    }
                });

                if changed {
                    *action = SlideBinAction::UpdateElement {
                        idx,
                        element: SlideElement::Calendar(updated),
                    };
                }
            }
            SlideElement::Text(overlay) => {
                let mut updated = overlay.clone();
                let mut changed = false;

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("✏️ Selected Text / Calendar")
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

                // Quick Sizing Presets & Slider — tightened padding: at the
                // theme's default button padding this row measured ~303px
                // against the 254px budget (dead-gap overflow).
                ui.scope(|ui| {
                    ui.spacing_mut().button_padding.x = 6.0;
                    ui.spacing_mut().item_spacing.x = 6.0;
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Size:").size(11.5).strong().color(AppTheme::text_secondary()));
                        for (label, sz) in [("S (14)", 14.0), ("M (18)", 18.0), ("L (24)", 24.0), ("XL (32)", 32.0)] {
                            let is_active = (updated.font_size - sz).abs() < 1.0;
                            if ui.add(Button::new(RichText::new(label).size(10.5)).fill(if is_active { AppTheme::accent_blue() } else { AppTheme::bg_card() })).clicked() {
                                updated.font_size = sz;
                                changed = true;
                            }
                        }
                    });
                });

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Scale Slider:").size(11.0).color(AppTheme::text_muted()));
                    crate::ui::small_slider(ui, 12.0, |ui| {
                        if ui.add_sized([120.0, 12.0], egui::Slider::new(&mut updated.font_size, 10.0..=60.0).step_by(1.0)).changed() {
                            changed = true;
                        }
                    });
                });

                ui.add_space(4.0);
                ui.label(RichText::new("Text Words / Grid:").size(11.5).color(AppTheme::text_secondary()));
                let text_resp = ui.add_sized(
                    // −8: TextEdit adds its own 2×4px margin on top of the
                    // given size, which nudged this over the width budget.
                    [ui.available_width() - 8.0, 60.0],
                    egui::TextEdit::multiline(&mut updated.text).hint_text("Type words..."),
                );
                if text_resp.changed() {
                    changed = true;
                }

                // If this text element appears to be a calendar grid, show the full Calendar & Holiday settings panel
                let looks_like_calendar = updated.text.contains("Sun") && updated.text.contains("Mon");
                if looks_like_calendar {
                    ui.add_space(6.0);
                    egui::CollapsingHeader::new(RichText::new("🗓 Calendar & Holiday Controls").size(12.0).strong().color(AppTheme::accent_yellow()))
                        .default_open(true)
                        .show(ui, |ui| {
                            Self::render_calendar_config_panel(ui, app, action, true);
                        });
                }

                ui.add_space(4.0);
                egui::ComboBox::from_id_salt("sel_slide_text_font")
                    .selected_text(RichText::new(format!("🔤 {}", updated.font_family.label())).size(12.0))
                    // ComboBox::width sets the INNER width; the button frame
                    // adds 2× button_padding (28px) on top, so leave room.
                    .width(ui.available_width() - 36.0)
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
                    ui.color_edit_button_srgba(&mut updated.text_color);
                });

                // Label on its own line + short button labels: the one-line
                // form measured ~364px against the 254px budget (dead gap).
                ui.label(RichText::new("Background:").size(11.5).color(AppTheme::text_secondary()));
                ui.horizontal(|ui| {
                    for style in TextBoxStyle::all() {
                        let is_sel = updated.box_style == *style;
                        let short = match style {
                            TextBoxStyle::None => "None",
                            TextBoxStyle::TranslucentBox => "Tight Box",
                            TextBoxStyle::SolidBanner => "Banner",
                        };
                        if ui
                            .add(Button::new(RichText::new(short).size(11.0)).fill(if is_sel { AppTheme::accent_blue() } else { AppTheme::bg_panel() }))
                            .on_hover_text(style.label())
                            .clicked()
                        {
                            updated.box_style = *style;
                            changed = true;
                        }
                    }
                });

                ui.add_space(6.0);
                // Short labels at 11.5: the 15px "Move Up / Move Down /
                // Delete Text" row measured ~359px against the 254px budget.
                ui.horizontal(|ui| {
                    if ui.add(Button::new(RichText::new("⬆ Up").size(11.5))).on_hover_text("Move this text earlier in the layer order").clicked() {
                        *action = SlideBinAction::ReorderElement { idx, dir: -1 };
                    }
                    if ui.add(Button::new(RichText::new("⬇ Down").size(11.5))).on_hover_text("Move this text later in the layer order").clicked() {
                        *action = SlideBinAction::ReorderElement { idx, dir: 1 };
                    }
                    if ui.add(Button::new(RichText::new("🗑 Delete").size(11.5))).on_hover_text("Delete this text element").clicked() {
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
                Self::inspector_media_header(ui, "🖼 Picture", AppTheme::accent_cyan(), path, action);
                ui.add_space(4.0);
                let is_full = match element {
                    SlideElement::Picture { x, y, w, h, .. } => *x == 0.0 && *y == 0.0 && *w == 1.0 && *h == 1.0,
                    _ => false,
                };
                // Two rows: three 15px buttons on one line measured ~410px
                // against the 254px budget — the single widest dead-gap row.
                ui.horizontal(|ui| {
                    let full_lbl = if is_full { "🗗 Centered Box" } else { "⛶ Full Slide" };
                    let full_btn = Button::new(RichText::new(full_lbl).size(12.0).strong().color(Color32::WHITE))
                        .fill(if is_full { AppTheme::accent_blue() } else { AppTheme::accent_green() });
                    if ui.add(full_btn).on_hover_text("Toggle between 100% full slide and centered collage box").clicked() {
                        *action = SlideBinAction::FullSlide(idx);
                    }
                    if ui.add(Button::new(RichText::new("🖼 Background").size(11.5)))
                        .on_hover_text("Use this picture as the slide background")
                        .clicked()
                    {
                        *action = SlideBinAction::SetElementAsBackground(idx);
                    }
                });
                ui.horizontal(|ui| {
                    if ui.add(Button::new(RichText::new("🗑 Delete").size(11.5))).clicked() {
                        *action = SlideBinAction::RemoveElement(idx);
                    }
                });
            }
            SlideElement::Video { path, .. } => {
                Self::inspector_media_header(ui, "🎬 Video", AppTheme::accent_cyan(), path, action);
                ui.add_space(4.0);
                let is_full = match element {
                    SlideElement::Video { x, y, w, h, .. } => *x == 0.0 && *y == 0.0 && *w == 1.0 && *h == 1.0,
                    _ => false,
                };
                ui.horizontal(|ui| {
                    let full_lbl = if is_full { "🗗 Centered Box" } else { "⛶ Full Slide" };
                    let full_btn = Button::new(RichText::new(full_lbl).size(12.0).strong().color(Color32::WHITE))
                        .fill(if is_full { AppTheme::accent_blue() } else { AppTheme::accent_green() });
                    if ui.add(full_btn).on_hover_text("Toggle between 100% full slide and centered collage box").clicked() {
                        *action = SlideBinAction::FullSlide(idx);
                    }
                    if ui.add(Button::new(RichText::new("🗑 Delete").size(11.5))).clicked() {
                        *action = SlideBinAction::RemoveElement(idx);
                    }
                });
            }
            SlideElement::Audio { path, volume } => {
                let mut vol = *volume;
                Self::inspector_media_header(ui, "🎵 Audio", AppTheme::accent_cyan(), path, action);
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Volume:").size(11.5).color(AppTheme::text_secondary()));
                    if ui.add(egui::Slider::new(&mut vol, 0.0..=2.0).step_by(0.05)).changed() {
                        *action = SlideBinAction::UpdateAudioVolume { idx, volume: vol };
                    }
                });
                ui.add_space(4.0);
                if ui.button("🗑 Delete Audio").clicked() {
                    *action = SlideBinAction::RemoveElement(idx);
                }
            }
            SlideElement::Placeholder { slot_id, label, .. } => {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("➕ Slot #{}: {}", slot_id, label))
                            .size(13.0)
                            .strong()
                            .color(AppTheme::accent_yellow()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✖ Deselect").clicked() {
                            *action = SlideBinAction::SelectElement(None);
                        }
                    });
                });
                ui.add_space(4.0);
                ui.label(RichText::new("Drag any photo or video from the Files tab and drop it directly onto this slot on the canvas.")
                    .size(11.5).color(AppTheme::text_muted()));
                ui.add_space(4.0);
                if ui.button("🗑 Delete Slot").clicked() {
                    *action = SlideBinAction::RemoveElement(idx);
                }
            }
        }
    }
}

/// Filename for sidebar rows, capped so a long name can never widen the panel
/// (labels in non-wrapping horizontal rows extend instead of wrapping, and the
/// sidebar budget is 264px — the dead-gap bug was filename-driven).
fn file_label(p: &Path) -> String {
    let name = p
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| p.to_string_lossy().to_string());
    truncate_chars(&name, 22)
}

/// Truncate to `max` characters with a trailing ellipsis.
fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{cut}…")
    }
}
