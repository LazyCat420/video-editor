use std::path::PathBuf;
use video_editor::core::clip::Clip;
use video_editor::core::effects::{SlideEffect, SlideEffectKind};
use video_editor::core::stickers::{StickerCatalog, StickerCategory};
use video_editor::core::text_overlay::SlideElement;
use video_editor::core::timeline::Timeline;
use video_editor::export::build_ffmpeg_export_command;
use video_editor::export::export_to_pdf;
use video_editor::export::export_to_pptx;
use video_editor::export::filter_graph::EncoderType;
use video_editor::export::ExportConfig;

#[test]
fn test_powerpoint_effects_catalog_and_toggles() {
    let mut clip = Clip::new_blank_slide(1, 1, "Celebration Slide".to_string(), 5.0);
    assert_eq!(clip.effects.len(), 0);

    // Test all 6 effects
    let kinds = SlideEffectKind::all();
    assert_eq!(kinds.len(), 6);

    for kind in kinds {
        assert!(!kind.label().is_empty());
        assert!(!kind.icon().is_empty());
        assert!(!kind.description().is_empty());
        clip.toggle_effect(*kind);
        assert!(clip.has_effect(*kind));
    }
    assert_eq!(clip.effects.len(), 6);

    // Toggling Fireworks again turns it off
    clip.toggle_effect(SlideEffectKind::Fireworks);
    assert!(!clip.has_effect(SlideEffectKind::Fireworks));
    assert_eq!(clip.effects.len(), 5);

    // Clear all effects
    clip.clear_effects();
    assert_eq!(clip.effects.len(), 0);
}

#[test]
fn test_effect_particle_simulator_render_all_effects() {
    use video_editor::core::effects::EffectParticleSimulator;
    use egui::{Pos2, Rect, Vec2};

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        let painter = ctx.layer_painter(egui::LayerId::background());
        let all_kinds = SlideEffectKind::all();
        for kind in all_kinds {
            let effect = SlideEffect::new(*kind);
            let effects_list = vec![effect];

            // Render at various time steps across cycle
            for t in [0.0, 0.25, 0.5, 1.2, 2.5, 5.0] {
                EffectParticleSimulator::render_preview(
                    &painter,
                    Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 450.0)),
                    t,
                    &effects_list,
                );
            }
        }
    });
}

#[test]
fn test_holiday_stickers_catalog_and_asset_generation() {
    let temp_dir = std::env::temp_dir().join("ve_sticker_test_assets");
    let _ = std::fs::create_dir_all(&temp_dir);

    // Ensure assets exist in temp directory
    StickerCatalog::ensure_sticker_assets_exist(&temp_dir);

    let all_stickers = StickerCatalog::all_stickers();
    assert!(all_stickers.len() >= 30);

    let categories = StickerCategory::all_filter_categories();
    assert_eq!(categories.len(), 12);

    for item in &all_stickers {
        assert!(!item.id.is_empty());
        assert!(!item.name.is_empty());
        assert!(!item.emoji.is_empty());
        let asset_path = StickerCatalog::sticker_asset_path(&temp_dir, &item.id);
        assert!(asset_path.exists(), "Sticker asset {} must exist", item.id);

        let img = image::open(&asset_path).expect("Must open sticker PNG");
        assert_eq!(img.width(), 256);
        assert_eq!(img.height(), 256);
    }
}

#[test]
fn test_slide_sticker_element_transformations_and_bounds() {
    let path = PathBuf::from("assets/stickers/pumpkin.png");
    let mut element = SlideElement::Sticker {
        path: path.clone(),
        name: "Jack-o'-Lantern".to_string(),
        category: StickerCategory::Halloween,
        x: 0.2,
        y: 0.3,
        w: 0.25,
        h: 0.25,
    };

    assert!(element.is_visual());
    assert_eq!(element.bounds(), (0.2, 0.3, 0.25, 0.25));

    // Move & Resize sticker
    element.set_bounds(0.4, 0.5, 0.35, 0.35);
    assert_eq!(element.bounds(), (0.4, 0.5, 0.35, 0.35));

    // Bounds clamp
    element.set_bounds(1.5, -0.5, 2.0, 0.0);
    let (x, y, w, h) = element.bounds();
    assert!(x <= 1.0 && x >= 0.0);
    assert!(y <= 1.0 && y >= 0.0);
    assert!(w <= 1.0 && w >= 0.01);
    assert!(h <= 1.0 && h >= 0.01);
}

#[test]
fn test_export_slide_with_stickers_to_pptx_and_pdf() {
    let temp_dir = std::env::temp_dir().join("ve_export_sticker_test");
    let _ = std::fs::create_dir_all(&temp_dir);

    // Ensure sticker assets exist
    StickerCatalog::ensure_sticker_assets_exist(&temp_dir);

    let mut timeline = Timeline::new(30.0);
    let track_id = timeline.tracks[0].id;

    let mut clip = Clip::new_blank_slide(100, track_id, "Holiday Card".to_string(), 4.0);
    let sticker_path = StickerCatalog::sticker_asset_path(&temp_dir, "xmas_tree");

    clip.elements.push(SlideElement::Sticker {
        path: sticker_path,
        name: "Christmas Tree".to_string(),
        category: StickerCategory::Christmas,
        x: 0.35,
        y: 0.30,
        w: 0.30,
        h: 0.30,
    });
    clip.effects.push(SlideEffect::new(SlideEffectKind::Fireworks));

    timeline.tracks[0].add_clip(clip);

    // 1. Export PPTX
    let pptx_path = temp_dir.join("holiday_collage.pptx");
    let pptx_res = export_to_pptx(&timeline, &pptx_path);
    assert!(pptx_res.is_ok(), "PPTX export must succeed: {:?}", pptx_res);
    assert!(pptx_path.exists());
    assert!(std::fs::metadata(&pptx_path).unwrap().len() > 100);

    // 2. Export PDF
    let pdf_path = temp_dir.join("holiday_collage.pdf");
    let pdf_res = export_to_pdf(&timeline, &pdf_path);
    assert!(pdf_res.is_ok(), "PDF export must succeed: {:?}", pdf_res);
    assert!(pdf_path.exists());
    assert!(std::fs::metadata(&pdf_path).unwrap().len() > 100);
}

#[test]
fn test_filter_graph_compilation_with_stickers() {
    let temp_dir = std::env::temp_dir().join("ve_filter_graph_sticker_test");
    let _ = std::fs::create_dir_all(&temp_dir);
    StickerCatalog::ensure_sticker_assets_exist(&temp_dir);

    let mut timeline = Timeline::new(30.0);
    let track_id = timeline.tracks[0].id;

    let mut clip = Clip::new_blank_slide(200, track_id, "Sticker Video Slide".to_string(), 3.0);
    let sticker_path = StickerCatalog::sticker_asset_path(&temp_dir, "red_heart");

    clip.elements.push(SlideElement::Sticker {
        path: sticker_path,
        name: "Love Heart".to_string(),
        category: StickerCategory::Valentine,
        x: 0.40,
        y: 0.40,
        w: 0.20,
        h: 0.20,
    });

    timeline.tracks[0].add_clip(clip);

    let config = ExportConfig {
        output_path: temp_dir.join("out.mp4"),
        width: 1280,
        height: 720,
        fps: 30.0,
        video_bitrate_kbps: 2000,
        audio_bitrate_kbps: 192,
        encoder: EncoderType::Libx264,
    };

    let cmd_res = build_ffmpeg_export_command(&timeline, &config);
    assert!(cmd_res.is_ok(), "FFmpeg export command must build: {:?}", cmd_res);
    let cmd = cmd_res.unwrap();
    let cmd_str = cmd.join(" ");
    assert!(cmd_str.contains("filter_complex"));
    assert!(cmd_str.contains("overlay"));
}
