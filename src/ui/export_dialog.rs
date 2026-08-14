use crate::export::filter_graph::{EncoderType, ExportConfig};
use crate::export::renderer::RenderProgress;
use crate::ui::theme::AppTheme;
use egui::{Button, Color32, ProgressBar, RichText, Window};

pub struct ExportDialog {
    pub is_open: bool,
    pub config: ExportConfig,
    pub progress_rx: Option<tokio::sync::watch::Receiver<RenderProgress>>,
    pub status_message: String,
}

impl Default for ExportDialog {
    fn default() -> Self {
        Self {
            is_open: false,
            config: ExportConfig::default(),
            progress_rx: None,
            status_message: String::new(),
        }
    }
}

pub enum ExportDialogAction {
    None,
    StartExport(ExportConfig),
    Close,
}

impl ExportDialog {
    pub fn render(&mut self, ctx: &egui::Context) -> ExportDialogAction {
        if !self.is_open {
            return ExportDialogAction::None;
        }

        let mut action = ExportDialogAction::None;
        let mut is_open_window = true;
        let mut should_close = false;

        Window::new("🚀 Export Video")
            .open(&mut is_open_window)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_width(380.0);

                // Resolution Presets
                ui.heading(RichText::new("Export Settings").size(15.0).color(AppTheme::text_primary()));
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label("Resolution Preset:");
                    egui::ComboBox::from_label("")
                        .selected_text(format!("{}x{}", self.config.width, self.config.height))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.config.width, 1920, "1080p FHD (1920x1080)");
                            if self.config.width == 1920 { self.config.height = 1080; }

                            ui.selectable_value(&mut self.config.width, 1280, "720p HD (1280x720)");
                            if self.config.width == 1280 { self.config.height = 720; }

                            ui.selectable_value(&mut self.config.width, 854, "480p SD (854x480)");
                            if self.config.width == 854 { self.config.height = 480; }

                            ui.selectable_value(&mut self.config.width, 3840, "4K UHD (3840x2160)");
                            if self.config.width == 3840 { self.config.height = 2160; }
                        });
                });

                // Frame Rate
                ui.horizontal(|ui| {
                    ui.label("Frame Rate (FPS):");
                    egui::ComboBox::from_label(" ")
                        .selected_text(format!("{:.0} fps", self.config.fps))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.config.fps, 24.0, "24 fps (Cinematic)");
                            ui.selectable_value(&mut self.config.fps, 30.0, "30 fps (Standard)");
                            ui.selectable_value(&mut self.config.fps, 60.0, "60 fps (Smooth)");
                        });
                });

                // Encoder
                ui.horizontal(|ui| {
                    ui.label("Video Encoder:");
                    let encoder_name = match self.config.encoder {
                        EncoderType::Libx264 => "libx264 (CPU / Universal Fallback)",
                        EncoderType::VaapiH264 => "VAAPI (Linux / Intel / AMD HW)",
                        EncoderType::QsvH264 => "QuickSync (Intel HW)",
                        EncoderType::NvencH264 => "NVENC (Nvidia HW)",
                    };

                    egui::ComboBox::from_label("  ")
                        .selected_text(encoder_name)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.config.encoder,
                                EncoderType::Libx264,
                                "libx264 (CPU / Universal Fallback)",
                            );
                            ui.selectable_value(
                                &mut self.config.encoder,
                                EncoderType::VaapiH264,
                                "VAAPI (Linux / Intel / AMD HW)",
                            );
                            ui.selectable_value(
                                &mut self.config.encoder,
                                EncoderType::QsvH264,
                                "QuickSync (Intel HW)",
                            );
                            ui.selectable_value(
                                &mut self.config.encoder,
                                EncoderType::NvencH264,
                                "NVENC (Nvidia HW)",
                            );
                        });
                });

                // Bitrate
                ui.horizontal(|ui| {
                    ui.label("Video Bitrate (kbps):");
                    ui.add(egui::DragValue::new(&mut self.config.video_bitrate_kbps).range(500..=50000).speed(100));
                });

                // Output Path
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Destination:");
                    let path_str = self.config.output_path.to_str().unwrap_or("output.mp4");
                    ui.label(RichText::new(path_str).monospace().size(11.0).color(AppTheme::accent_cyan()));
                    if ui.button("Browse...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("MP4 Video", &["mp4"])
                            .set_file_name("render.mp4")
                            .save_file()
                        {
                            self.config.output_path = path;
                        }
                    }
                });

                // Progress Bar Display
                if let Some(ref rx) = self.progress_rx {
                    let progress = rx.borrow().clone();
                    match progress {
                        RenderProgress::Rendering { progress_pct, current_time_secs, fps } => {
                            ui.add_space(8.0);
                            ui.add(ProgressBar::new(progress_pct / 100.0).text(format!("{:.1}% (render time: {:.1}s)", progress_pct, current_time_secs)));
                            ui.label(RichText::new(format!("Rendering at {:.1} FPS...", fps)).size(11.0).color(AppTheme::text_secondary()));
                        }
                        RenderProgress::Completed { output_path } => {
                            ui.add_space(8.0);
                            ui.label(RichText::new("✅ Render Complete!").color(AppTheme::accent_green()).strong());
                            ui.label(RichText::new(format!("Saved to: {}", output_path.display())).size(11.0));
                        }
                        RenderProgress::Failed { error } => {
                            ui.add_space(8.0);
                            ui.label(RichText::new(format!("❌ Render Failed: {}", error)).color(AppTheme::accent_red()));
                        }
                        _ => {}
                    }
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    let is_rendering = self.progress_rx.as_ref().map_or(false, |rx| {
                        matches!(*rx.borrow(), RenderProgress::Rendering { .. })
                    });

                    let render_btn = Button::new(RichText::new("Start Render").color(Color32::WHITE).strong())
                        .fill(AppTheme::accent_green());

                    if ui.add_enabled(!is_rendering, render_btn).clicked() {
                        action = ExportDialogAction::StartExport(self.config.clone());
                    }

                    if ui.button("Close").clicked() {
                        should_close = true;
                        action = ExportDialogAction::Close;
                    }
                });
            });

        if !is_open_window || should_close {
            self.is_open = false;
        }

        action
    }
}
