use egui::Color32;
use std::path::PathBuf;
use video_editor::core::clip::Clip;
use video_editor::core::stickers::{StickerCatalog, StickerCategory};
use video_editor::core::text_overlay::{FontFamilyPreset, SlideBackground, SlideElement, TextBoxStyle, TextOverlay};
use video_editor::core::text_paint::TextPaint;
use video_editor::core::timeline::Timeline;
use video_editor::export::filter_graph::{build_ffmpeg_export_command, EncoderType, ExportConfig};
use video_editor::export::pdf_exporter::export_to_pdf;
use video_editor::media::ffmpeg_locator::{find_ffmpeg_executable, find_ffprobe_executable};

#[test]
fn test_text_paint_color_parity() {
    // 1. Fully opaque pure orange
    let orange = Color32::from_rgb(255, 128, 0);
    let paint_orange = TextPaint::from_color32(orange);
    assert_eq!(paint_orange.r, 255);
    assert_eq!(paint_orange.g, 128);
    assert_eq!(paint_orange.b, 0);
    assert_eq!(paint_orange.a, 255);
    assert_eq!(paint_orange.to_ffmpeg_fontcolor(), "0xFF8000");
    let (pdf_r, pdf_g, pdf_b) = paint_orange.to_pdf_rgb();
    assert!((pdf_r - 1.0).abs() < 1e-4);
    assert!((pdf_g - (128.0 / 255.0)).abs() < 1e-4);
    assert!((pdf_b - 0.0).abs() < 1e-4);

    // 2. Translucent Cyan
    let cyan_half = Color32::from_rgba_unmultiplied(0, 200, 255, 128);
    let paint_cyan = TextPaint::from_color32(cyan_half);
    assert_eq!(paint_cyan.r, 0);
    assert!((paint_cyan.g as i32 - 200).abs() <= 1);
    assert_eq!(paint_cyan.b, 255);
    assert_eq!(paint_cyan.a, 128);
    assert!(paint_cyan.to_ffmpeg_fontcolor().starts_with("0x00C"));
}

#[test]
fn test_ffmpeg_command_contains_overwrite_and_progress_flags() {
    let mut timeline = Timeline::new(30.0);
    let v_track_id = timeline.tracks[0].id;

    let mut clip = Clip::new_blank_slide(1, v_track_id, "Slide 1".to_string(), 5.0);
    clip.background = Some(SlideBackground::Solid(Color32::from_rgb(20, 24, 30)));

    let mut overlay = TextOverlay::new("Export Test Overlay");
    overlay.text_color = Color32::from_rgb(255, 200, 50);
    overlay.font_family = FontFamilyPreset::Serif;
    overlay.font_size = 28.0;
    overlay.box_style = TextBoxStyle::TranslucentBox;
    clip.elements.push(SlideElement::Text(overlay));

    if let Some(t) = timeline.get_track_mut(v_track_id) {
        t.add_clip(clip);
    }

    let config = ExportConfig {
        output_path: PathBuf::from("my_export.mp4"),
        width: 1920,
        height: 1080,
        fps: 30.0,
        video_bitrate_kbps: 8000,
        audio_bitrate_kbps: 192,
        encoder: EncoderType::Libx264,
    };

    let args = build_ffmpeg_export_command(&timeline, &config).expect("Failed to build ffmpeg command");
    let full_cmd = args.join(" ");

    // Critical assertions: Overwrite flag, progress pipe, drawtext color
    assert!(args.contains(&"-y".to_string()), "Command must include -y flag to avoid prompt deadlocks");
    assert!(args.contains(&"-progress".to_string()), "Command must include -progress");
    assert!(args.contains(&"pipe:1".to_string()), "Progress must be piped to stdout");
    assert!(full_cmd.contains("fontcolor=0xFFC832"), "Drawtext filter must contain formatted fontcolor");
    assert!(full_cmd.contains("my_export.mp4"), "Command must output to specified path");
}

#[test]
fn test_stickers_transparent_generation() {
    let all_stickers = StickerCatalog::all_stickers();
    assert!(all_stickers.len() >= 30);

    for item in all_stickers {
        let img = StickerCatalog::generate_procedural_sticker_image(item);
        assert_eq!(img.width(), 256);
        assert_eq!(img.height(), 256);

        // Check that corner pixel (0, 0) is completely transparent (alpha = 0)
        let top_left = img.get_pixel(0, 0);
        assert_eq!(top_left[3], 0, "Top-left corner of sticker {} must be transparent", item.id);

        let bottom_right = img.get_pixel(255, 255);
        assert_eq!(bottom_right[3], 0, "Bottom-right corner of sticker {} must be transparent", item.id);

        // Check center pixel is opaque
        let center = img.get_pixel(128, 128);
        assert!(center[3] > 200, "Center of sticker {} must be rendered and visible", item.id);
    }
}

#[test]
fn test_pdf_static_export_with_various_elements() {
    let mut timeline = Timeline::new(30.0);
    let v_track_id = timeline.tracks[0].id;

    let mut clip = Clip::new_blank_slide(1, v_track_id, "Slide With Text & Sticker".to_string(), 4.0);
    clip.background = Some(SlideBackground::Solid(Color32::from_rgb(30, 30, 40)));

    let mut overlay = TextOverlay::new("PDF Slide Title");
    overlay.text_color = Color32::from_rgb(255, 255, 255);
    clip.elements.push(SlideElement::Text(overlay));

    clip.elements.push(SlideElement::Sticker {
        path: PathBuf::from("assets/stickers/star_gold.png"),
        name: "Gold Star".to_string(),
        category: StickerCategory::EverydayFun,
        x: 0.1,
        y: 0.1,
        w: 0.2,
        h: 0.2,
    });

    if let Some(t) = timeline.get_track_mut(v_track_id) {
        t.add_clip(clip);
    }

    let temp_pdf = std::env::temp_dir().join("test_export_presentation.pdf");
    let res = export_to_pdf(&timeline, &temp_pdf);
    assert!(res.is_ok(), "PDF export must succeed");

    let metadata = std::fs::metadata(&temp_pdf).expect("PDF file must exist");
    assert!(metadata.len() > 100, "PDF file must not be empty");

    let pdf_bytes = std::fs::read(&temp_pdf).expect("Read PDF bytes");
    assert!(pdf_bytes.starts_with(b"%PDF-1.7"), "Must be a valid PDF 1.7 header");
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);
    assert!(pdf_str.contains("PDF Slide Title"), "PDF must contain the slide text");
}

#[test]
fn test_ffmpeg_locator_discovery() {
    let ffmpeg = find_ffmpeg_executable();
    assert!(!ffmpeg.as_os_str().is_empty());

    let ffprobe = find_ffprobe_executable();
    assert!(!ffprobe.as_os_str().is_empty());
}
