use eframe::egui;
use egui::{RichText, Vec2};
use video_editor::ui::theme::{AppTheme, ThemeKind};
use video_editor::ui::small_slider;

struct ShotApp {
    cur_scale: f32, // mirrors the settings slider
    v14: f32,
    v12: f32,
    done: bool,
    frames: u32,
    configured: bool,
}

impl Default for ShotApp {
    fn default() -> Self {
        Self {
            cur_scale: 1.0,
            v14: 1.0,
            v12: 1.0,
            done: false,
            frames: 0,
            configured: false,
        }
    }
}

impl eframe::App for ShotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frames += 1;
        if !self.configured {
            AppTheme::configure(ctx, ThemeKind::Dark, 1.0);
            self.configured = true;
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(24.0);
            ui.heading("Slider knob comparison (circle you drag)");
            ui.add_space(12.0);

            ui.label("A) CURRENT (small_slider 18, add_sized [150,18])");
            small_slider(ui, 18.0, |ui| {
                ui.add_sized(
                    [150.0, 18.0],
                    egui::Slider::new(&mut self.cur_scale, 0.7..=1.15)
                        .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                        .step_by(0.05),
                )
            });
            ui.add_space(16.0);

            ui.label("B) small_slider 14, add_sized [150,14]");
            small_slider(ui, 14.0, |ui| {
                ui.add_sized(
                    [150.0, 14.0],
                    egui::Slider::new(&mut self.v14, 0.7..=1.15)
                        .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                        .step_by(0.05),
                )
            });
            ui.add_space(16.0);

            ui.label("C) small_slider 12, add_sized [150,12]");
            small_slider(ui, 12.0, |ui| {
                ui.add_sized(
                    [150.0, 12.0],
                    egui::Slider::new(&mut self.v12, 0.7..=1.15)
                        .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                        .step_by(0.05),
                )
            });
            ui.add_space(16.0);

            ui.label("D) PLAIN slider (no small_slider) -- shows original big knob");
            ui.add(egui::Slider::new(&mut self.cur_scale, 0.7..=1.15).fixed_decimals(2));
        });

        if self.frames > 120 && !self.done {
            self.done = true;
            std::process::exit(0);
        }
        std::thread::sleep(std::time::Duration::from_millis(16));
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(760.0, 420.0)),
        ..Default::default()
    };
    eframe::run_native("Slider Shot", opts, Box::new(|_| Ok(Box::new(ShotApp::default()))))
}
