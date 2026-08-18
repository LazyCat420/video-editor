use video_editor::app::VideoEditorApp;
use video_editor::core::text_overlay::{FontFamilyPreset, SlideElement, TextBoxStyle, TextOverlay};
use video_editor::ui::theme::AppTheme;
use egui::{FontFamily, FontId, Pos2, Rect, Vec2};
use std::sync::Arc;

#[test]
fn test_all_10_font_presets_layout_without_panic() {
    let ctx = egui::Context::default();
    AppTheme::install_custom_fonts(&ctx);

    ctx.begin_pass(egui::RawInput::default());
    
    let painter = ctx.layer_painter(egui::LayerId::background());
    let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));

    for preset in FontFamilyPreset::all() {
        let mut overlay = TextOverlay::new(format!("Sample text for {}", preset.label()));
        overlay.font_family = *preset;
        overlay.font_size = 24.0;
        overlay.box_style = TextBoxStyle::TranslucentBox;

        video_editor::ui::text_renderer::TextRenderer::draw_text_overlay(
            &painter,
            rect,
            &overlay,
        );

        // Also test direct galley layout
        let fam = FontFamily::Name(Arc::from(preset.preview_family()));
        let font_id = FontId::new(20.0, fam);
        let galley = painter.layout_no_wrap(
            format!("Direct layout test for {}", preset.label()),
            font_id,
            egui::Color32::WHITE,
        );
        assert!(galley.size().x > 0.0);
    }

    let _ = ctx.end_pass();
}

#[test]
fn test_add_text_box_slide_bin_action() {
    let mut app = VideoEditorApp::default();
    app.insert_blank_slide_at_playhead(5.0, None);

    let active_slide = app.active_slide().expect("Expected active slide");
    let slide_id = active_slide.id;
    assert_eq!(active_slide.elements.len(), 0);

    // Create a new text overlay
    let mut overlay = TextOverlay::new("Grandma's Birthday");
    overlay.font_size = 32.0;
    overlay.font_family = FontFamilyPreset::Handwritten;
    overlay.box_style = TextBoxStyle::SolidBanner;

    // Simulate adding text element action
    if let Some(clip) = app.project.timeline.get_clip_mut(slide_id) {
        clip.elements.push(SlideElement::Text(overlay));
        app.selected_slide_element = Some(clip.elements.len() - 1);
    }

    let updated_slide = app.active_slide().expect("Expected active slide");
    assert_eq!(updated_slide.elements.len(), 1);
    assert_eq!(app.selected_slide_element, Some(0));

    if let SlideElement::Text(t) = &updated_slide.elements[0] {
        assert_eq!(t.text, "Grandma's Birthday");
        assert_eq!(t.font_family, FontFamilyPreset::Handwritten);
        assert_eq!(t.box_style, TextBoxStyle::SolidBanner);
        assert_eq!(t.font_size, 32.0);
    } else {
        panic!("Expected SlideElement::Text");
    }
}
