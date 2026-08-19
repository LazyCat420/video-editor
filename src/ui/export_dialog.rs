use crate::export::filter_graph::{EncoderType, ExportConfig};
use crate::export::renderer::RenderProgress;
use crate::ui::theme::AppTheme;
use egui::{Button, Color32, ProgressBar, RichText, Window};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormatTab {
    VideoMp4,
    PresentationPdf,
}

pub struct ExportDialog {
    pub is_open: bool,
    pub active_tab: ExportFormatTab,
    pub config: ExportConfig,
    pub pdf_output_path: PathBuf,
    pub progress_rx: Option<tokio::sync::watch::Receiver<RenderProgress>>,
    pub export_status: Option<Result<PathBuf, String>>,
}

impl Default for ExportDialog {
    fn default() -> Self {
        let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).unwrap_or_else(|_| ".".to_string());
        let default_dir = PathBuf::from(home).join("Desktop");
        Self {
            is_open: false,
            active_tab: ExportFormatTab::VideoMp4,
            config: ExportConfig::default(),
            pdf_output_path: default_dir.join("slideshow.pdf"),
            progress_rx: None,
            export_status: None,
        }
    }
}

pub enum ExportDialogAction {
    None,
    StartExportVideo(ExportConfig),
    StartExportPdf(PathBuf),
    Close,
}

impl ExportDialog {
    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn render(&mut self, ctx: &egui::Context) -> ExportDialogAction {
        if !self.is_open {
            return ExportDialogAction::None;
        }

        let mut action = ExportDialogAction::None;
        let mut is_open_window = true;
        let mut should_close = false;

        Window::new("🚀 Export Slideshow & Video")
            .open(&mut is_open_window)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_width(450.0);

                // Format Tabs
                ui.horizontal(|ui| {
                    let video_btn = egui::SelectableLabel::new(
                        self.active_tab == ExportFormatTab::VideoMp4,
                        RichText::new("🎬 Video (.mp4)").size(13.0).strong(),
                    );
                    if ui.add(video_btn).on_hover_text("Export animated movie with audio & transitions").clicked() {
                        self.active_tab = ExportFormatTab::VideoMp4;
                    }

                    let pdf_btn = egui::SelectableLabel::new(
                        self.active_tab == ExportFormatTab::PresentationPdf,
                        RichText::new("📄 Printable PDF (.pdf)").size(13.0).strong(),
                    );
                    if ui.add(pdf_btn).on_hover_text("Export 16:9 landscape presentation PDF for viewing & printing").clicked() {
                        self.active_tab = ExportFormatTab::PresentationPdf;
                    }
                });

                ui.separator();
                ui.add_space(6.0);

                match self.active_tab {
                    ExportFormatTab::VideoMp4 => {
                        ui.heading(RichText::new("Video Export Settings").size(15.0).color(AppTheme::text_primary()));
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
                                    ui.selectable_value(&mut self.config.encoder, EncoderType::Libx264, "libx264 (CPU / Universal Fallback)");
                                    ui.selectable_value(&mut self.config.encoder, EncoderType::VaapiH264, "VAAPI (Linux / Intel / AMD HW)");
                                    ui.selectable_value(&mut self.config.encoder, EncoderType::QsvH264, "QuickSync (Intel HW)");
                                    ui.selectable_value(&mut self.config.encoder, EncoderType::NvencH264, "NVENC (Nvidia HW)");
                                });
                        });

                        ui.horizontal(|ui| {
                            ui.label("Video Bitrate (kbps):");
                            ui.add(egui::DragValue::new(&mut self.config.video_bitrate_kbps).range(500..=50000).speed(100));
                        });

                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("Destination:");
                            let path_str = self.config.output_path.to_str().unwrap_or("output.mp4");
                            ui.label(RichText::new(path_str).monospace().size(11.0).color(AppTheme::accent_cyan()));
                            if ui.button("Browse...").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("MP4 Video", &["mp4"])
                                    .set_file_name("slideshow.mp4")
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
                                    ui.label(RichText::new("✅ Video Render Complete!").color(AppTheme::accent_green()).strong());
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

                            let render_btn = Button::new(RichText::new("🎬 Start Video Render").color(Color32::WHITE).strong())
                                .fill(AppTheme::accent_green());

                            if ui.add_enabled(!is_rendering, render_btn).clicked() {
                                action = ExportDialogAction::StartExportVideo(self.config.clone());
                            }

                            if ui.button("Close").clicked() {
                                should_close = true;
                                action = ExportDialogAction::Close;
                            }
                        });
                    }

                    ExportFormatTab::PresentationPdf => {
                        ui.heading(RichText::new("Printable Presentation PDF Export").size(15.0).color(AppTheme::accent_cyan()));
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("Creates a 16:9 widescreen landscape PDF presentation document, ideal for viewing, printing, sharing, or photo albums.")
                                .color(AppTheme::text_secondary())
                                .size(12.0)
                        );
                        ui.add_space(8.0);

                        ui.label(RichText::new("• 16:9 Landscape High-Resolution Document").size(12.0));
                        ui.label(RichText::new("• High-Quality Vector Typography & Sharp Artwork").size(12.0));
                        ui.label(RichText::new("• Embedded Photos, Video Frames & Vector Calendar Grids").size(12.0));
                        ui.label(RichText::new("• Universal compatibility with Acrobat, Web Browsers & Printers").size(12.0));

                        ui.add_space(8.0);
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("Save File To:");
                            let path_str = self.pdf_output_path.to_str().unwrap_or("slideshow.pdf");
                            ui.label(RichText::new(path_str).monospace().size(11.0).color(AppTheme::accent_cyan()));
                            if ui.button("Browse...").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("PDF Document", &["pdf"])
                                    .set_file_name("slideshow.pdf")
                                    .save_file()
                                {
                                    self.pdf_output_path = path;
                                }
                            }
                        });

                        if let Some(ref res) = self.export_status {
                            match res {
                                Ok(p) => {
                                    ui.add_space(8.0);
                                    ui.label(RichText::new("✅ PDF Document Created Successfully!").color(AppTheme::accent_green()).strong());
                                    ui.label(RichText::new(format!("Saved to: {}", p.display())).size(11.0));
                                }
                                Err(e) => {
                                    ui.add_space(8.0);
                                    ui.label(RichText::new(format!("❌ Export Failed: {}", e)).color(AppTheme::accent_red()));
                                }
                            }
                        }

                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            let export_btn = Button::new(RichText::new("📄 Export Presentation PDF (.pdf)").color(Color32::WHITE).strong())
                                .fill(AppTheme::accent_green());

                            if ui.add(export_btn).clicked() {
                                action = ExportDialogAction::StartExportPdf(self.pdf_output_path.clone());
                            }

                            if ui.button("Close").clicked() {
                                should_close = true;
                                action = ExportDialogAction::Close;
                            }
                        });
                    }
                }
            });

        if !is_open_window || should_close {
            self.is_open = false;
        }

        action
    }
}
