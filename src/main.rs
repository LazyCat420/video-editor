pub mod app;
pub mod audio;
pub mod core;
pub mod export;
pub mod media;
pub mod ui;

use app::VideoEditorApp;
use eframe::NativeOptions;
use egui::Vec2;

fn main() -> eframe::Result<()> {
    // Initialize Tokio async runtime for background workers
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to initialize Tokio runtime");
    let _guard = rt.enter();

    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Video Editor (Rust + FFmpeg)")
            .with_inner_size(Vec2::new(1280.0, 800.0))
            .with_min_inner_size(Vec2::new(960.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Video Editor",
        native_options,
        Box::new(|cc| Ok(Box::new(VideoEditorApp::new(cc)))),
    )
}
