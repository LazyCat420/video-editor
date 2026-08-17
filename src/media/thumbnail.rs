use egui::ColorImage;
use std::path::Path;
use std::process::Command;

/// Downscale a [`ColorImage`] to fit within `max_w` x `max_h`, preserving aspect ratio.
/// Returns a clone unchanged if already small enough.
pub fn downscale(img: &ColorImage, max_w: usize, max_h: usize) -> ColorImage {
    let (w, h) = (img.size[0], img.size[1]);
    if w <= max_w && h <= max_h {
        return img.clone();
    }

    let scale = ((max_w as f32 / w as f32).min(max_h as f32 / h as f32)).min(1.0);
    let nw = ((w as f32 * scale).round() as u32).max(1);
    let nh = ((h as f32 * scale).round() as u32).max(1);

    let raw: Vec<u8> = img
        .pixels
        .iter()
        .flat_map(|c| [c.r(), c.g(), c.b(), c.a()])
        .collect();

    if let Some(rgba) = image::RgbaImage::from_raw(w as u32, h as u32, raw) {
        let dyn_img = image::DynamicImage::ImageRgba8(rgba)
            .resize(nw, nh, image::imageops::FilterType::Lanczos3);
        let rgba = dyn_img.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        ColorImage::from_rgba_unmultiplied(size, &rgba.into_raw())
    } else {
        img.clone()
    }
}

/// Extract a compact thumbnail for an asset at the given timestamp (default 1.0s or 0.0s).
pub fn extract_thumbnail<P: AsRef<Path>>(
    media_path: P,
    timestamp_secs: f64,
) -> Result<ColorImage, String> {
    let path = media_path.as_ref();
    let ts_str = format!("{:.3}", timestamp_secs.max(0.0));

    let ffmpeg_bin = crate::media::frame_cache::find_ffmpeg_executable();
    let output = Command::new(&ffmpeg_bin)
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
