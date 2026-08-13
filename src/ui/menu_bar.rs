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
    OpenExportDialog,
    ToggleHelp,
}

impl MenuBarView {
    pub fn render(ui: &mut Ui, project: &mut Project) -> MenuAction {
        let mut action = MenuAction::None;

        ui.horizontal(|ui| {
            ui.add_space(6.0);

            // Step 1: Open Video Button (Prominent Blue)
            let open_btn = Button::new(
                RichText::new("1. 📂 Open Video / Music")
                    .size(15.0)
                    .strong()
                    .color(Color32::WHITE),
            )
            .min_size(egui::vec2(190.0, 36.0))
            .fill(AppTheme::ACCENT_BLUE);

            if ui
                .add(open_btn)
                .on_hover_text("Click to select a video or music file from your computer")
                .clicked()
            {
                action = MenuAction::ImportMedia;
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Step 2: Cut & Remove Tools
            let split_btn = Button::new(
                RichText::new("2. ✂ Cut Video (S)")
                    .size(15.0)
                    .strong()
                    .color(Color32::WHITE),
            )
            .min_size(egui::vec2(140.0, 36.0))
            .fill(AppTheme::BG_CARD);

            if ui
                .add(split_btn)
                .on_hover_text("Cut the video at the red line marker (Hotkey: S)")
                .clicked()
            {
                action = MenuAction::SplitAtPlayhead;
            }

            let del_btn = Button::new(
                RichText::new("🗑 Delete Clip")
                    .size(14.0)
                    .color(AppTheme::TEXT_PRIMARY),
            )
            .min_size(egui::vec2(120.0, 36.0))
            .fill(AppTheme::BG_CARD);

            if ui
                .add(del_btn)
                .on_hover_text("Remove the selected piece of video or audio")
                .clicked()
            {
                action = MenuAction::DeleteSelected;
            }

            // Easy Zoom Buttons (+ / -)
            ui.add_space(4.0);
            if ui
                .add(Button::new("🔍 -").min_size(egui::vec2(42.0, 36.0)))
                .on_hover_text("Zoom out timeline")
                .clicked()
            {
                project.timeline.zoom_pps = (project.timeline.zoom_pps * 0.75).max(15.0);
            }
            if ui
                .add(Button::new("🔍 +").min_size(egui::vec2(42.0, 36.0)))
                .on_hover_text("Zoom in timeline")
                .clicked()
            {
                project.timeline.zoom_pps = (project.timeline.zoom_pps * 1.33).min(300.0);
            }

            // Right-aligned: Export & Help
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(6.0);

                // Step 3: Export Button (Prominent Green)
                let export_btn = Button::new(
                    RichText::new("3. 🚀 Export Finished Video")
                        .size(15.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .min_size(egui::vec2(210.0, 36.0))
                .fill(AppTheme::ACCENT_GREEN);

                if ui
                    .add(export_btn)
                    .on_hover_text("Save your finished video to a file")
                    .clicked()
                {
                    action = MenuAction::OpenExportDialog;
                }

                // Help Button
                let help_btn = Button::new(
                    RichText::new("❓ Help")
                        .size(14.0)
                        .color(AppTheme::TEXT_PRIMARY),
                )
                .min_size(egui::vec2(75.0, 36.0))
                .fill(AppTheme::BG_CARD);

                if ui
                    .add(help_btn)
                    .on_hover_text("View simple step-by-step instructions")
                    .clicked()
                {
                    action = MenuAction::ToggleHelp;
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
