use crate::core::project::Project;
use crate::ui::theme::AppTheme;
use egui::{Button, Color32, RichText, Ui};

pub struct MenuBarView;

pub enum MenuAction {
    None,
    NewProject,
    OpenProject,
    SaveProject,
    ImportMedia,
    SplitAtPlayhead,
    DeleteSelected,
    OpenTransitions,
    OpenExportDialog,
    OpenSettings,
}

impl MenuBarView {
    pub fn render(ui: &mut Ui, project: &mut Project, is_timeline: bool) -> MenuAction {
        let mut action = MenuAction::None;

        ui.horizontal(|ui| {
            ui.add_space(6.0);

            // Step 1: Open Video / Music
            let open_btn = Button::new(
                RichText::new("📂 Open Files")
                    .size(14.0)
                    .color(AppTheme::text_primary()),
            )
            .min_size(egui::vec2(120.0, 36.0))
            .fill(AppTheme::bg_card());

            if ui
                .add(open_btn)
                .on_hover_text("Select photos, videos, or music files from your computer")
                .clicked()
            {
                action = MenuAction::ImportMedia;
            }

            // Step 2: Cut tool (shown in Timeline mode)
            if is_timeline {
                let split_btn = Button::new(
                    RichText::new("✂ Cut (C)")
                        .size(14.0)
                        .color(AppTheme::text_primary()),
                )
                .min_size(egui::vec2(90.0, 36.0))
                .fill(AppTheme::bg_card());

                if ui
                    .add(split_btn)
                    .on_hover_text("Cut the video or slide at the playhead (Hotkey: C)")
                    .clicked()
                {
                    action = MenuAction::SplitAtPlayhead;
                }
            }

            let del_btn = Button::new(
                RichText::new("🗑 Delete")
                    .size(14.0)
                    .color(AppTheme::text_primary()),
            )
            .min_size(egui::vec2(90.0, 36.0))
            .fill(AppTheme::bg_card());

            if ui
                .add(del_btn)
                .on_hover_text("Remove the selected slide or piece of video")
                .clicked()
            {
                action = MenuAction::DeleteSelected;
            }

            // Zoom Buttons (+ / -) only in Timeline mode
            if is_timeline {
                ui.add_space(4.0);
                if ui
                    .add(Button::new("🔍 -").min_size(egui::vec2(36.0, 36.0)))
                    .on_hover_text("Zoom out")
                    .clicked()
                {
                    project.timeline.zoom_pps = (project.timeline.zoom_pps * 0.75).max(15.0);
                }
                if ui
                    .add(Button::new("🔍 +").min_size(egui::vec2(36.0, 36.0)))
                    .on_hover_text("Zoom in")
                    .clicked()
                {
                    project.timeline.zoom_pps = (project.timeline.zoom_pps * 1.33).min(300.0);
                }
            }

            // Right-aligned: Export & Help
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(6.0);

                // Step 3: Export Button (Prominent Green)
                let export_btn = Button::new(
                    RichText::new("🚀 Export Video")
                        .size(14.5)
                        .strong()
                        .color(Color32::WHITE),
                )
                .min_size(egui::vec2(160.0, 36.0))
                .fill(AppTheme::accent_green());

                if ui
                    .add(export_btn)
                    .on_hover_text("Save your finished slideshow or video to a file")
                    .clicked()
                {
                    action = MenuAction::OpenExportDialog;
                }



                // Settings button (theme + text size)
                let settings_btn = Button::new(
                    RichText::new("⚙ Settings")
                        .size(14.0)
                        .color(AppTheme::text_primary()),
                )
                .min_size(egui::vec2(95.0, 36.0))
                .fill(AppTheme::bg_card());

                if ui
                    .add(settings_btn)
                    .on_hover_text("Change the colors and text size")
                    .clicked()
                {
                    action = MenuAction::OpenSettings;
                }

                // Project Menu (File/Save)
                egui::menu::menu_button(ui, "📁 Project", |ui| {
                    if ui.button("New Project").clicked() {
                        action = MenuAction::NewProject;
                        ui.close_menu();
                    }
                    if ui.button("Open Project...").clicked() {
                        action = MenuAction::OpenProject;
                        ui.close_menu();
                    }
                    if ui.button("Save Project...").clicked() {
                        action = MenuAction::SaveProject;
                        ui.close_menu();
                    }
                });
            });
        });

        action
    }
}
