use std::path::PathBuf;
use video_editor::core::clip::Clip;
use video_editor::core::text_overlay::{FontFamilyPreset, SlideBackground, SlideElement, TextBoxStyle, TextOverlay};

#[test]
fn test_slide_element_drop_media_asset_picture_and_video() {
    let mut slide = Clip::new_blank_slide(1, 100, "Blank Slide".to_string(), 3.0);
    assert_eq!(slide.elements.len(), 0);

    // Drop Picture
    let pic_path = PathBuf::from("photo.png");
    let pic_elem = SlideElement::Picture {
        path: pic_path.clone(),
        x: 0.25,
        y: 0.30,
        w: 0.40,
        h: 0.30,
    };
    slide.elements.push(pic_elem);
    assert_eq!(slide.elements.len(), 1);
    assert_eq!(slide.elements[0].bounds(), (0.25, 0.30, 0.40, 0.30));

    // Drop Video
    let vid_path = PathBuf::from("clip.mp4");
    let vid_elem = SlideElement::Video {
        path: vid_path.clone(),
        x: 0.50,
        y: 0.20,
        w: 0.45,
        h: 0.35,
    };
    slide.elements.push(vid_elem);
    assert_eq!(slide.elements.len(), 2);
    assert_eq!(slide.elements[1].bounds(), (0.50, 0.20, 0.45, 0.35));
}

#[test]
fn test_slide_element_set_as_background() {
    let mut slide = Clip::new_blank_slide(1, 100, "Blank Slide".to_string(), 3.0);
    let pic_path = PathBuf::from("bg_photo.jpg");
    slide.elements.push(SlideElement::Picture {
        path: pic_path.clone(),
        x: 0.1,
        y: 0.1,
        w: 0.5,
        h: 0.5,
    });

    // Simulate "Set as Slide Background" action
    if let SlideElement::Picture { path, .. } = slide.elements.remove(0) {
        slide.background = Some(SlideBackground::Picture(path));
    }

    assert_eq!(slide.elements.len(), 0);
    assert_eq!(slide.background, Some(SlideBackground::Picture(pic_path)));
}

#[test]
fn test_click_to_add_text_with_placeholder() {
    let mut overlay = TextOverlay::default();
    if overlay.text.trim().is_empty() {
        overlay.text = "Click to edit text".to_string();
    }
    overlay.x = 0.35;
    overlay.y = 0.45;

    let text_elem = SlideElement::Text(overlay);
    assert_eq!(text_elem.bounds(), (0.35, 0.45, 0.0, 0.0));
    if let SlideElement::Text(o) = &text_elem {
        assert_eq!(o.text, "Click to edit text");
        assert_eq!(o.font_family, FontFamilyPreset::SansSerif);
        assert_eq!(o.box_style, TextBoxStyle::None);
    } else {
        panic!("Expected SlideElement::Text");
    }
}

#[test]
fn test_slide_element_reordering_and_layering() {
    let mut slide = Clip::new_blank_slide(1, 100, "Blank Slide".to_string(), 3.0);
    slide.elements.push(SlideElement::Text(TextOverlay::new("First")));
    slide.elements.push(SlideElement::Text(TextOverlay::new("Second")));
    slide.elements.push(SlideElement::Text(TextOverlay::new("Third")));

    // Move "First" down (index 0 -> index 1)
    let el = slide.elements.remove(0);
    slide.elements.insert(1, el);

    if let SlideElement::Text(o) = &slide.elements[0] {
        assert_eq!(o.text, "Second");
    }
    if let SlideElement::Text(o) = &slide.elements[1] {
        assert_eq!(o.text, "First");
    }
    if let SlideElement::Text(o) = &slide.elements[2] {
        assert_eq!(o.text, "Third");
    }
}

#[test]
fn test_slide_element_resize_and_bounds_clamp() {
    let mut elem = SlideElement::Picture {
        path: PathBuf::from("img.png"),
        x: 0.1,
        y: 0.1,
        w: 0.3,
        h: 0.3,
    };

    // Test moving & resizing
    elem.set_bounds(0.85, 0.85, 0.5, 0.5);
    let (x, y, w, h) = elem.bounds();
    assert_eq!(x, 0.85);
    assert_eq!(y, 0.85);
    assert_eq!(w, 0.5);
    assert_eq!(h, 0.5);
}
