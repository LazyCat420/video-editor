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
        persist_window: false,
        viewport: egui::ViewportBuilder::default()
            .with_title("Video Editor (Rust + FFmpeg)")
            .with_maximized(true)
            .with_inner_size(Vec2::new(1440.0, 900.0))
            .with_min_inner_size(Vec2::new(1024.0, 680.0))
            .with_resizable(true)
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "Video Editor",
        native_options,
        Box::new(|cc| Ok(Box::new(VideoEditorApp::new(cc)))),
    )
}
