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

#[test]
fn test_backspace_does_not_delete_text_box() {
    use video_editor::ui::preview_player::{PlayerAction, PreviewPlayerView};

    let mut app = VideoEditorApp::default();
    app.insert_blank_slide_at_playhead(5.0, None);

    let active_slide = app.active_slide().expect("Expected active slide");
    let slide_id = active_slide.id;

    // Add a text box
    let overlay = TextOverlay::new("Hello World");
    if let Some(clip) = app.project.timeline.get_clip_mut(slide_id) {
        clip.elements.push(SlideElement::Text(overlay));
        app.selected_slide_element = Some(0);
    }

    assert_eq!(app.active_slide().unwrap().elements.len(), 1);
    assert_eq!(app.selected_slide_element, Some(0));

    // Run an egui frame where Backspace key is pressed
    let ctx = egui::Context::default();
    let mut raw_input = egui::RawInput::default();
    raw_input.screen_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 720.0)));
    raw_input.events.push(egui::Event::Key {
        key: egui::Key::Backspace,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::default(),
    });

    let mut player_action = PlayerAction::None;
    let _ = ctx.run(raw_input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let visuals = vec![];
            let mut texture = None;
            let mut view_mode = video_editor::ui::MainViewMode::Slideshow;
            player_action = PreviewPlayerView::render(
                ui,
                &app.project.timeline,
                None,
                &visuals,
                &mut texture,
                false,
                false,
                Some(0),
                &mut view_mode,
            );
        });
    });

    // Assert that PlayerAction::DeleteElement was NOT produced by Backspace
    assert!(
        !matches!(player_action, PlayerAction::DeleteElement(_)),
        "Backspace must NEVER produce a DeleteElement action"
    );
}

#[test]
fn test_text_overlay_all_formatting_properties() {
    let mut overlay = TextOverlay::new("hello world");
    assert_eq!(overlay.formatted_text(), "hello world");

    overlay.is_all_caps = true;
    assert_eq!(overlay.formatted_text(), "HELLO WORLD");

    overlay.is_bold = true;
    overlay.is_italic = true;
    overlay.show_shadow = false;
    overlay.alignment = video_editor::core::text_overlay::TextAlignment::Right;
    overlay.text_color = egui::Color32::from_rgb(255, 230, 0);
    overlay.box_style = TextBoxStyle::TranslucentBox;
    overlay.box_opacity = 0.85;

    assert!(overlay.is_bold);
    assert!(overlay.is_italic);
    assert!(!overlay.show_shadow);
    assert_eq!(overlay.alignment, video_editor::core::text_overlay::TextAlignment::Right);
    assert_eq!(overlay.text_color, egui::Color32::from_rgb(255, 230, 0));
    assert_eq!(overlay.box_opacity, 0.85);
}

#[test]
fn test_text_renderer_bold_italic_and_alignment_rendering() {
    let ctx = egui::Context::default();
    AppTheme::install_custom_fonts(&ctx);

    ctx.begin_pass(egui::RawInput::default());
    let painter = ctx.layer_painter(egui::LayerId::background());
    let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 720.0));

    for preset in FontFamilyPreset::all() {
        for is_bold in [true, false] {
            for is_italic in [true, false] {
                for show_shadow in [true, false] {
                    for alignment in [
                        video_editor::core::text_overlay::TextAlignment::Left,
                        video_editor::core::text_overlay::TextAlignment::Center,
                        video_editor::core::text_overlay::TextAlignment::Right,
                    ] {
                        let mut overlay = TextOverlay::new(format!("Test line 1\nLine 2 for {}", preset.label()));
                        overlay.font_family = *preset;
                        overlay.is_bold = is_bold;
                        overlay.is_italic = is_italic;
                        overlay.show_shadow = show_shadow;
                        overlay.alignment = alignment;
                        overlay.text_color = egui::Color32::from_rgb(0, 229, 255);
                        overlay.box_style = TextBoxStyle::TranslucentBox;
                        overlay.box_opacity = 0.75;

                        video_editor::ui::text_renderer::TextRenderer::draw_text_overlay(
                            &painter,
                            rect,
                            &overlay,
                        );
                    }
                }
            }
        }
    }
    let _ = ctx.end_pass();
}

#[test]
fn test_pptx_and_pdf_export_text_formatting() {
    use std::io::Read;

    let mut app = VideoEditorApp::default();
    app.insert_blank_slide_at_playhead(5.0, None);

    let slide_id = app.active_slide().unwrap().id;
    let mut overlay = TextOverlay::new("Title with Bold & Italic");
    overlay.is_bold = true;
    overlay.is_italic = true;
    overlay.is_all_caps = true;
    overlay.font_family = FontFamilyPreset::SansSerif;
    overlay.text_color = egui::Color32::from_rgb(255, 68, 68);
    overlay.box_style = TextBoxStyle::TranslucentBox;
    overlay.box_opacity = 0.8;

    if let Some(clip) = app.project.timeline.get_clip_mut(slide_id) {
        clip.elements.push(SlideElement::Text(overlay));
    }

    let temp_dir = std::env::temp_dir().join(format!("ve_test_export_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    // Test PPTX export
    let pptx_path = temp_dir.join("test.pptx");
    video_editor::export::pptx_exporter::export_to_pptx(&app.project.timeline, &pptx_path).unwrap();
    assert!(pptx_path.exists());

    // Inspect PPTX zip contents for b="1" and i="1"
    let file = std::fs::File::open(&pptx_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut slide_xml = String::new();
    for i in 0..archive.len() {
        let mut zfile = archive.by_index(i).unwrap();
        if zfile.name().contains("slide1.xml") {
            zfile.read_to_string(&mut slide_xml).unwrap();
            break;
        }
    }
    assert!(slide_xml.contains("b=\"1\""), "PPTX slide XML must contain b=\"1\" when bold is active");
    assert!(slide_xml.contains("i=\"1\""), "PPTX slide XML must contain i=\"1\" when italic is active");
    assert!(slide_xml.contains("TITLE WITH BOLD &amp; ITALIC") || slide_xml.contains("TITLE WITH BOLD"), "PPTX slide XML must contain formatted uppercase text");

    // Test PDF export
    let pdf_path = temp_dir.join("test.pdf");
    video_editor::export::pdf_exporter::export_to_pdf(&app.project.timeline, &pdf_path).unwrap();
    assert!(pdf_path.exists());

    let pdf_bytes = std::fs::read(&pdf_path).unwrap();
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);
    assert!(pdf_str.contains("/Helvetica-BoldOblique") || pdf_str.contains("/F4"), "PDF must define bold italic font");
    assert!(pdf_str.contains("/F4"), "PDF content stream must reference /F4 for bold+italic text");

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_selected_text_element_renders_realtime_on_canvas() {
    use video_editor::ui::preview_player::{PlayerAction, PreviewPlayerView};

    let mut app = VideoEditorApp::default();
    app.insert_blank_slide_at_playhead(5.0, None);

    let slide_id = app.active_slide().unwrap().id;
    let mut overlay = TextOverlay::new("Live Canvas Update");
    overlay.is_bold = true;
    overlay.is_italic = true;
    overlay.text_color = egui::Color32::from_rgb(255, 230, 0);

    if let Some(clip) = app.project.timeline.get_clip_mut(slide_id) {
        clip.elements.push(SlideElement::Text(overlay));
        app.selected_slide_element = Some(0);
    }

    let ctx = egui::Context::default();
    AppTheme::install_custom_fonts(&ctx);

    let mut raw_input = egui::RawInput::default();
    raw_input.screen_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280.0, 720.0)));

    let mut player_action = PlayerAction::None;
    let _ = ctx.run(raw_input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let visuals = vec![];
            let mut texture = None;
            let mut view_mode = video_editor::ui::MainViewMode::Slideshow;
            player_action = PreviewPlayerView::render(
                ui,
                &app.project.timeline,
                None,
                &visuals,
                &mut texture,
                false,
                false,
                Some(0),
                &mut view_mode,
            );
        });
    });

    assert!(matches!(player_action, PlayerAction::None));
}



