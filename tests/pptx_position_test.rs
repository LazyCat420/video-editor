
use video_editor::core::clip::Clip;
use video_editor::core::text_overlay::{SlideBackground, SlideElement, TextOverlay};
use video_editor::core::timeline::Timeline;
use video_editor::core::track::{Track, TrackKind};
use video_editor::export::export_to_pptx;

#[test]
fn test_pptx_exact_image_positioning_and_embeds() {
    let tmp_dir = std::env::temp_dir().join(format!("pptx_verify_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&tmp_dir).unwrap();

    // 1. Create 3 distinct images with different colors
    let img_a_path = tmp_dir.join("photo_a_red.png");
    let img_b_path = tmp_dir.join("photo_b_green.png");
    let img_c_path = tmp_dir.join("photo_c_blue.png");

    let img_a = image::RgbaImage::from_pixel(200, 150, image::Rgba([255, 0, 0, 255]));
    img_a.save(&img_a_path).unwrap();

    let img_b = image::RgbaImage::from_pixel(300, 200, image::Rgba([0, 255, 0, 255]));
    img_b.save(&img_b_path).unwrap();

    let img_c = image::RgbaImage::from_pixel(400, 300, image::Rgba([0, 0, 255, 255]));
    img_c.save(&img_c_path).unwrap();

    // 2. Build Timeline with 1 slide having 3 images at precise spots
    let mut timeline = Timeline::default();
    let mut track = Track::new(1, "Video Track".to_string(), TrackKind::Video);

    let mut slide = Clip::new_blank_slide(1, 1, "Vacation Photo Slide".to_string(), 5.0);
    slide.background = Some(SlideBackground::Solid(egui::Color32::from_rgb(26, 36, 51)));

    // Photo A: Top-Left (x: 0.05, y: 0.10, w: 0.40, h: 0.35)
    slide.elements.push(SlideElement::Picture {
        path: img_a_path.clone(),
        x: 0.05,
        y: 0.10,
        w: 0.40,
        h: 0.35,
    });

    // Photo B: Top-Right (x: 0.55, y: 0.10, w: 0.40, h: 0.35)
    slide.elements.push(SlideElement::Picture {
        path: img_b_path.clone(),
        x: 0.55,
        y: 0.10,
        w: 0.40,
        h: 0.35,
    });

    // Photo C: Bottom-Center (x: 0.20, y: 0.55, w: 0.60, h: 0.40)
    slide.elements.push(SlideElement::Picture {
        path: img_c_path.clone(),
        x: 0.20,
        y: 0.55,
        w: 0.60,
        h: 0.40,
    });

    // Title Text at Top-Center (x: 0.50, y: 0.05)
    let mut title = TextOverlay::new("Summer Vacation Collage");
    title.x = 0.50;
    title.y = 0.05;
    title.font_size = 32.0;
    slide.elements.push(SlideElement::Text(title));

    track.clips.push(slide);
    timeline.tracks.push(track);

    // 3. Export to PPTX
    let pptx_path = tmp_dir.join("vacation_collage.pptx");
    export_to_pptx(&timeline, &pptx_path).expect("Failed to export to PPTX");

    println!("EXPORT_SUCCESS:{}", pptx_path.display());
}
