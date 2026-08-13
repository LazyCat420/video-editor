use egui::ColorImage;
use std::path::Path;
use std::process::Command;

/// Extract a compact thumbnail for an asset at the given timestamp (default 1.0s or 0.0s).
pub fn extract_thumbnail<P: AsRef<Path>>(
    media_path: P,
    timestamp_secs: f64,
) -> Result<ColorImage, String> {
    let path = media_path.as_ref();
    let ts_str = format!("{:.3}", timestamp_secs.max(0.0));

    let output = Command::new("ffmpeg")
        .args([
            "-ss",
            &ts_str,
            "-i",
            path.to_str().unwrap_or_default(),
            "-vframes",
            "1",
            "-vf",
            "scale=160:-1",
            "-f",
            "image2pipe",
            "-vcodec",
            "png",
            "-",
        ])
        .output()
        .map_err(|e| format!("Failed to spawn ffmpeg for thumbnail: {}", e))?;

    if !output.status.success() || output.stdout.is_empty() {
        return Err("Failed to extract thumbnail image".to_string());
    }

    let dyn_img = image::load_from_memory(&output.stdout)
        .map_err(|e| format!("Failed to decode thumbnail PNG: {}", e))?;

    let rgba = dyn_img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let pixels = rgba.into_raw();

    Ok(ColorImage::from_rgba_unmultiplied(size, &pixels))
}
