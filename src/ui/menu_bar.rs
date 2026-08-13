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
}

impl MenuBarView {
    pub fn render(ui: &mut Ui, project: &mut Project) -> MenuAction {
        let mut action = MenuAction::None;

        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Project").clicked() {
                    action = MenuAction::NewProject;
                    ui.close_menu();
                }
                if ui.button("Open Project...").clicked() {
                    action = MenuAction::OpenProject;
                    ui.close_menu();
                }
                if ui.button("Save Project").clicked() {
                    action = MenuAction::SaveProject;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Import Media...").clicked() {
                    action = MenuAction::ImportMedia;
                    ui.close_menu();
                }
                if ui.button("Export Video... (Ctrl+E)").clicked() {
                    action = MenuAction::OpenExportDialog;
                    ui.close_menu();
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Split at Playhead (S)").clicked() {
                    action = MenuAction::SplitAtPlayhead;
                    ui.close_menu();
                }
                if ui.button("Delete Selected (Del)").clicked() {
                    action = MenuAction::DeleteSelected;
                    ui.close_menu();
                }
                if ui.button("Clear Selection").clicked() {
                    project.timeline.clear_selection();
                    ui.close_menu();
                }
            });

            ui.separator();

            // Quick Toolbar Action Buttons
            if ui
                .add(
                    Button::new(RichText::new("📁 Import Media").color(Color32::WHITE))
                        .fill(AppTheme::ACCENT_BLUE),
                )
                .clicked()
            {
                action = MenuAction::ImportMedia;
            }

            if ui
                .add(
                    Button::new(RichText::new("✂ Split (S)").color(Color32::WHITE))
                        .fill(AppTheme::BG_CARD),
                )
                .on_hover_text("Split clip at current playhead position (Hotkey: S)")
                .clicked()
            {
                action = MenuAction::SplitAtPlayhead;
            }

            if ui
                .add(
                    Button::new(RichText::new("🗑 Delete").color(Color32::WHITE))
                        .fill(AppTheme::BG_CARD),
                )
                .on_hover_text("Delete selected clip(s) (Hotkey: Delete / Backspace)")
                .clicked()
            {
                action = MenuAction::DeleteSelected;
            }

            ui.separator();

            // Magnetic Snapping Toggle
            let snap_text = if project.timeline.snapping_enabled {
                "🧲 Snap: ON"
            } else {
                "🧲 Snap: OFF"
            };
            let snap_btn = Button::new(snap_text).fill(if project.timeline.snapping_enabled {
                AppTheme::BG_HOVER
            } else {
                AppTheme::BG_CARD
            });
            if ui.add(snap_btn).clicked() {
                project.timeline.snapping_enabled = !project.timeline.snapping_enabled;
            }

            // Timeline Zoom Slider
            ui.add_space(10.0);
            ui.label(RichText::new("🔍 Zoom:").size(12.0).color(AppTheme::TEXT_SECONDARY));
            ui.add(
                egui::Slider::new(&mut project.timeline.zoom_pps, 15.0..=300.0)
                    .logarithmic(true)
                    .show_value(false),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        Button::new(RichText::new("🚀 Export Video").color(Color32::WHITE).strong())
                            .fill(AppTheme::ACCENT_GREEN),
                    )
                    .clicked()
                {
                    action = MenuAction::OpenExportDialog;
                }

                ui.label(
                    RichText::new(format!("⏱ Total: {}", project.timeline.duration()))
                        .color(AppTheme::TEXT_MUTED)
                        .size(12.0),
                );
            });
        });

        action
    }
}
