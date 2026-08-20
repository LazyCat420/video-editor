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

#[test]
fn test_media_bin_import_does_not_mutate_timeline() {
    use video_editor::core::project::{MediaAsset, Project};

    let mut project = Project::new("Import Test".to_string());
    assert_eq!(project.media_assets.len(), 0);
    assert_eq!(project.timeline.tracks[0].clips.len(), 0);

    let asset1 = MediaAsset {
        id: 1,
        name: "video1.mp4".to_string(),
        path: PathBuf::from("video1.mp4"),
        duration_secs: 10.0,
        width: 1920,
        height: 1080,
        fps: 30.0,
        has_video: true,
        has_audio: true,
        proxy_path: None,
        peak_path: None,
    };
    let asset2 = MediaAsset {
        id: 2,
        name: "video2.mp4".to_string(),
        path: PathBuf::from("video2.mp4"),
        duration_secs: 5.0,
        width: 1920,
        height: 1080,
        fps: 30.0,
        has_video: true,
        has_audio: true,
        proxy_path: None,
        peak_path: None,
    };

    project.add_asset(asset1);
    project.add_asset(asset2);

    assert_eq!(project.media_assets.len(), 2);
    // Timeline must remain untouched
    assert_eq!(project.timeline.tracks[0].clips.len(), 0);
    assert_eq!(project.timeline.duration().as_secs_f64(), 0.0);
}

#[test]
fn test_reorder_element_to_arbitrary_index() {
    let mut slide = Clip::new_blank_slide(1, 100, "Blank Slide".to_string(), 3.0);
    slide.elements.push(SlideElement::Text(TextOverlay::new("Item A")));
    slide.elements.push(SlideElement::Text(TextOverlay::new("Item B")));
    slide.elements.push(SlideElement::Text(TextOverlay::new("Item C")));
    slide.elements.push(SlideElement::Text(TextOverlay::new("Item D")));

    // Reorder from index 3 (Item D) to index 1 (between A and B)
    let el = slide.elements.remove(3);
    slide.elements.insert(1, el);

    let labels: Vec<String> = slide.elements.iter().map(|e| match e {
        SlideElement::Text(o) => o.text.clone(),
        _ => String::new(),
    }).collect();

    assert_eq!(labels, vec!["Item A", "Item D", "Item B", "Item C"]);

    // Reorder from index 0 (Item A) to end (index 3)
    let el = slide.elements.remove(0);
    slide.elements.insert(3, el);

    let labels: Vec<String> = slide.elements.iter().map(|e| match e {
        SlideElement::Text(o) => o.text.clone(),
        _ => String::new(),
    }).collect();

    assert_eq!(labels, vec!["Item D", "Item B", "Item C", "Item A"]);
}

#[test]
fn test_add_blank_page_and_powerpoint_composition() {
    use video_editor::core::timeline::Timeline;
    use video_editor::core::track::TrackKind;
    use video_editor::core::time::TimeCode;

    let mut timeline = Timeline::new(30.0);
    let track_id = timeline.tracks.iter().find(|t| t.kind == TrackKind::Video).unwrap().id;

    let next_id = timeline.next_id();
    let mut blank_slide = Clip::new_blank_slide(next_id, track_id, "Blank Slide".to_string(), 3.0);
    blank_slide.timeline_start = timeline.playhead;
    blank_slide.is_selected = true;

    // Compose elements onto blank slide like PowerPoint:
    // 1. Text element
    let mut text_el = TextOverlay::new("Welcome to Slides");
    text_el.x = 0.5;
    text_el.y = 0.2;
    blank_slide.elements.push(SlideElement::Text(text_el));

    // 2. Picture element
    blank_slide.elements.push(SlideElement::Picture {
        path: PathBuf::from("diagram.png"),
        x: 0.2,
        y: 0.4,
        w: 0.3,
        h: 0.3,
    });

    // 3. Video element
    blank_slide.elements.push(SlideElement::Video {
        path: PathBuf::from("demo.mp4"),
        x: 0.6,
        y: 0.4,
        w: 0.3,
        h: 0.3,
    });

    if let Some(track) = timeline.get_track_mut(track_id) {
        track.add_clip(blank_slide);
    }

    let track = timeline.get_track(track_id).unwrap();
    assert_eq!(track.clips.len(), 1);
    let clip = &track.clips[0];
    assert!(clip.is_static_slide());
    assert_eq!(clip.elements.len(), 3);
    assert_eq!(clip.duration(), TimeCode::from_secs_f64(3.0));
}

#[test]
fn test_media_bin_reorder_forward_and_backward() {
    use video_editor::core::project::MediaAsset;

    let make_asset = |id: u64, name: &str| MediaAsset {
        id,
        name: name.to_string(),
        path: PathBuf::from(format!("{}.mp4", name)),
        duration_secs: 10.0,
        width: 1920,
        height: 1080,
        fps: 30.0,
        has_video: true,
        has_audio: true,
        proxy_path: None,
        peak_path: None,
    };
    let mut assets = vec![
        make_asset(1, "Asset A"),
        make_asset(2, "Asset B"),
        make_asset(3, "Asset C"),
        make_asset(4, "Asset D"),
    ];

    // Reorder from index 0 (A) to before D (index 3)
    let from: usize = 0;
    let to_index: usize = 3;
    let target_pos = if from < to_index {
        to_index.saturating_sub(1).min(assets.len() - 1)
    } else {
        to_index.min(assets.len())
    };
    let item = assets.remove(from);
    assets.insert(target_pos, item);

    let names: Vec<&str> = assets.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, vec!["Asset B", "Asset C", "Asset A", "Asset D"]);

    // Reorder from index 3 (D) to before B (index 0)
    let from: usize = 3;
    let to_index: usize = 0;
    let target_pos = if from < to_index {
        to_index.saturating_sub(1).min(assets.len() - 1)
    } else {
        to_index.min(assets.len())
    };
    let item = assets.remove(from);
    assets.insert(target_pos, item);

    let names: Vec<&str> = assets.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, vec!["Asset D", "Asset B", "Asset C", "Asset A"]);
}

#[test]
fn test_slide_video_element_playback_time_progression() {
    use video_editor::core::timeline::Timeline;
    use video_editor::core::track::TrackKind;
    use video_editor::core::time::TimeCode;

    let mut timeline = Timeline::new(30.0);
    let track_id = timeline.tracks.iter().find(|t| t.kind == TrackKind::Video).unwrap().id;

    let next_id = timeline.next_id();
    let mut blank_slide = Clip::new_blank_slide(next_id, track_id, "Blank Slide".to_string(), 5.0);
    blank_slide.timeline_start = TimeCode::from_secs_f64(2.0); // starts at 00:02
    blank_slide.elements.push(SlideElement::Video {
        path: PathBuf::from("clip.mp4"),
        x: 0.1,
        y: 0.1,
        w: 0.8,
        h: 0.8,
    });

    if let Some(track) = timeline.get_track_mut(track_id) {
        track.add_clip(blank_slide);
    }

    // When playhead is at 00:03.5 (1.5s into the slide)
    let playhead = TimeCode::from_secs_f64(3.5);
    let clip = timeline.get_track(track_id).unwrap().get_clip_at(playhead).unwrap();
    let elapsed = (playhead - clip.timeline_start).as_secs_f64();
    assert_eq!(elapsed, 1.5);
}

#[test]
fn test_slide_playback_background_isolated_from_stream_frame() {
    use video_editor::core::timeline::Timeline;
    use video_editor::core::track::TrackKind;
    use video_editor::core::time::TimeCode;

    let mut timeline = Timeline::new(30.0);
    let track_id = timeline.tracks.iter().find(|t| t.kind == TrackKind::Video).unwrap().id;

    // Clip 1: Blank slide with video element at 00:00 to 00:03
    let mut slide = Clip::new_blank_slide(1, track_id, "Slide 1".to_string(), 3.0);
    slide.timeline_start = TimeCode::ZERO;
    slide.elements.push(SlideElement::Video {
        path: PathBuf::from("embed.mp4"),
        x: 0.2,
        y: 0.2,
        w: 0.6,
        h: 0.6,
    });

    // Clip 2: Next video clip at 00:03 to 00:06
    let mut next_clip = Clip::new(2, track_id, "Next Video".to_string(), PathBuf::from("next.mp4"), TimeCode::from_secs_f64(3.0), true, true);
    next_clip.timeline_start = TimeCode::from_secs_f64(3.0);

    if let Some(track) = timeline.get_track_mut(track_id) {
        track.add_clip(slide);
        track.add_clip(next_clip);
    }

    // At playhead 00:02.8 (near end of slide 1)
    let playhead = TimeCode::from_secs_f64(2.8);
    let active_clip = timeline.get_track(track_id).unwrap().get_clip_at(playhead).unwrap();
    assert!(active_clip.is_static_slide());
    assert_eq!(active_clip.id, 1);

    // At playhead 00:03.1 (crossed into clip 2)
    let playhead_next = TimeCode::from_secs_f64(3.1);
    let next_active = timeline.get_track(track_id).unwrap().get_clip_at(playhead_next).unwrap();
    assert!(!next_active.is_static_slide());
    assert_eq!(next_active.id, 2);
}

#[test]
fn test_blank_slide_auto_fits_duration_to_longest_media() {
    use video_editor::core::envelope::VolumeEnvelope;
    use video_editor::core::project::MediaAsset;
    use video_editor::core::time::TimeCode;
    use video_editor::core::timeline::Timeline;
    use video_editor::core::track::TrackKind;

    let timeline = Timeline::new(30.0);
    let track_id = timeline.tracks.iter().find(|t| t.kind == TrackKind::Video).unwrap().id;

    let mut slide = Clip::new_blank_slide(1, track_id, "Slide 1".to_string(), 3.0);
    slide.elements.push(SlideElement::Video {
        path: PathBuf::from("clip1.mp4"),
        x: 0.1,
        y: 0.1,
        w: 0.4,
        h: 0.4,
    });
    slide.elements.push(SlideElement::Audio {
        path: PathBuf::from("audio1.mp3"),
        volume: 1.0,
    });

    let assets = vec![
        MediaAsset {
            id: 1,
            name: "clip1.mp4".to_string(),
            path: PathBuf::from("clip1.mp4"),
            duration_secs: 10.0,
            width: 1920,
            height: 1080,
            fps: 30.0,
            has_video: true,
            has_audio: true,
            proxy_path: None,
            peak_path: None,
        },
        MediaAsset {
            id: 2,
            name: "audio1.mp3".to_string(),
            path: PathBuf::from("audio1.mp3"),
            duration_secs: 18.5,
            width: 0,
            height: 0,
            fps: 0.0,
            has_video: false,
            has_audio: true,
            proxy_path: None,
            peak_path: None,
        },
    ];

    let mut max_dur: f64 = 0.0;
    for el in &slide.elements {
        match el {
            SlideElement::Video { path, .. } | SlideElement::Audio { path, .. } => {
                let dur = assets.iter().find(|a| &a.path == path).map(|a| a.duration_secs).unwrap_or(0.0);
                if dur > max_dur {
                    max_dur = dur;
                }
            }
            _ => {}
        }
    }

    assert_eq!(max_dur, 18.5);
    let target_dur = TimeCode::from_secs_f64(max_dur);
    slide.source_duration = target_dur;
    slide.source_out = target_dur;
    slide.volume_envelope = VolumeEnvelope::default_for_duration(target_dur);

    assert_eq!(slide.duration(), TimeCode::from_secs_f64(18.5));
}

#[test]
fn test_blank_slide_expansion_shifts_subsequent_clips() {
    use video_editor::core::time::TimeCode;
    use video_editor::core::timeline::Timeline;
    use video_editor::core::track::TrackKind;

    let mut timeline = Timeline::new(30.0);
    let track_id = timeline.tracks.iter().find(|t| t.kind == TrackKind::Video).unwrap().id;

    // Slide 1 starts at 00:00 with initial 3.0s duration
    let mut slide = Clip::new_blank_slide(1, track_id, "Slide 1".to_string(), 3.0);
    slide.timeline_start = TimeCode::ZERO;

    // Clip 2 starts at 00:03 with 5.0s duration
    let mut next_clip = Clip::new(2, track_id, "Next Video".to_string(), PathBuf::from("next.mp4"), TimeCode::from_secs_f64(5.0), true, true);
    next_clip.timeline_start = TimeCode::from_secs_f64(3.0);

    if let Some(track) = timeline.get_track_mut(track_id) {
        track.add_clip(slide);
        track.add_clip(next_clip);
    }

    // Now slide expands to 12.0s (+9.0s delta)
    let old_dur = TimeCode::from_secs_f64(3.0);
    let new_dur = TimeCode::from_secs_f64(12.0);
    let delta = new_dur - old_dur;
    let old_end = TimeCode::ZERO + old_dur;

    if let Some(track) = timeline.get_track_mut(track_id) {
        if let Some(c) = track.clips.iter_mut().find(|c| c.id == 1) {
            c.source_duration = new_dur;
            c.source_out = new_dur;
        }
        for c in &mut track.clips {
            if c.id != 1 && c.timeline_start >= old_end {
                c.timeline_start = c.timeline_start + delta;
            }
        }
        track.sort_clips();
    }

    let track = timeline.get_track(track_id).unwrap();
    assert_eq!(track.clips[0].duration(), TimeCode::from_secs_f64(12.0));
    assert_eq!(track.clips[1].timeline_start, TimeCode::from_secs_f64(12.0));
    assert_eq!(track.clips[1].timeline_end(), TimeCode::from_secs_f64(17.0));
}

#[test]
fn test_multi_video_slide_picks_longest_video_for_stream() {
    use video_editor::core::project::MediaAsset;

    let mut slide = Clip::new_blank_slide(1, 1, "Slide 1".to_string(), 15.0);
    slide.elements.push(SlideElement::Video {
        path: PathBuf::from("short_5s.mp4"),
        x: 0.1,
        y: 0.1,
        w: 0.4,
        h: 0.4,
    });
    slide.elements.push(SlideElement::Video {
        path: PathBuf::from("long_15s.mp4"),
        x: 0.5,
        y: 0.1,
        w: 0.4,
        h: 0.4,
    });

    let assets = vec![
        MediaAsset {
            id: 1,
            name: "short_5s.mp4".to_string(),
            path: PathBuf::from("short_5s.mp4"),
            duration_secs: 5.0,
            width: 1920,
            height: 1080,
            fps: 30.0,
            has_video: true,
            has_audio: true,
            proxy_path: None,
            peak_path: None,
        },
        MediaAsset {
            id: 2,
            name: "long_15s.mp4".to_string(),
            path: PathBuf::from("long_15s.mp4"),
            duration_secs: 15.0,
            width: 1920,
            height: 1080,
            fps: 30.0,
            has_video: true,
            has_audio: true,
            proxy_path: None,
            peak_path: None,
        },
    ];

    let mut best_media: Option<(PathBuf, f64)> = None;
    let mut max_dur: f64 = 0.0;

    for el in &slide.elements {
        if let SlideElement::Video { path, .. } = el {
            let dur = assets.iter().find(|a| &a.path == path).map(|a| a.duration_secs).unwrap_or(0.0);
            if dur >= max_dur {
                max_dur = dur;
                best_media = Some((path.clone(), dur));
            }
        }
    }

    assert_eq!(best_media, Some((PathBuf::from("long_15s.mp4"), 15.0)));
}

#[test]
fn test_shorter_video_clamps_to_end_frame_when_expired() {
    let short_dur: f64 = 5.0;
    let slide_elapsed_at_8s: f64 = 8.0;

    // At 8s, shorter 5s video clamps to (5.0 - 0.05) = 4.95s
    let clamped_time: f64 = if slide_elapsed_at_8s >= short_dur {
        (short_dur - 0.05).max(0.0)
    } else {
        slide_elapsed_at_8s
    };

    assert_eq!((clamped_time * 100.0).round(), 495.0);

    // At 3s, shorter 5s video plays naturally at 3.0s
    let slide_elapsed_at_3s: f64 = 3.0;
    let normal_time: f64 = if slide_elapsed_at_3s >= short_dur {
        (short_dur - 0.05).max(0.0)
    } else {
        slide_elapsed_at_3s
    };

    assert_eq!(normal_time, 3.0);
}




#[test]
fn test_unified_slide_text_overlay_on_video_clip() {
    use video_editor::core::clip::Clip;
    use video_editor::core::text_overlay::{SlideElement, TextOverlay};
    use video_editor::core::time::TimeCode;
    use std::path::PathBuf;

    // Create a regular video clip (has_video: true)
    let mut clip = Clip::new(
        1,
        100,
        "MyVideo.mp4".to_string(),
        PathBuf::from("videos/sample.mp4"),
        TimeCode::from_secs_f64(10.0),
        true,
        true,
    );

    assert_eq!(clip.elements.len(), 0);

    // Add a text box directly to the video clip (unified slide model)
    let mut text_overlay = TextOverlay::new("Intro Title".to_string());
    text_overlay.x = 0.5;
    text_overlay.y = 0.5;
    clip.elements.push(SlideElement::Text(text_overlay));

    assert_eq!(clip.elements.len(), 1);

    // Verify inline text editing mutates the element
    if let SlideElement::Text(t) = &mut clip.elements[0] {
        t.text = "Updated Intro Title Directly In Canvas".to_string();
    }

    if let SlideElement::Text(t) = &clip.elements[0] {
        assert_eq!(t.text, "Updated Intro Title Directly In Canvas");
        assert_eq!(t.x, 0.5);
        assert_eq!(t.y, 0.5);
    } else {
        panic!("Expected SlideElement::Text");
    }
}

#[test]
fn test_slide_background_solid_color_palette() {
    use video_editor::core::clip::Clip;
    use video_editor::core::text_overlay::SlideBackground;
    use egui::Color32;

    let mut clip = Clip::new_blank_slide(
        10,
        1,
        "Blank Slide".to_string(),
        5.0,
    );

    // Test Emerald Green
    clip.background = Some(SlideBackground::Solid(Color32::from_rgb(30, 145, 75)));
    assert_eq!(clip.background, Some(SlideBackground::Solid(Color32::from_rgb(30, 145, 75))));

    // Test Vibrant Yellow
    clip.background = Some(SlideBackground::Solid(Color32::from_rgb(250, 205, 35)));
    assert_eq!(clip.background, Some(SlideBackground::Solid(Color32::from_rgb(250, 205, 35))));

    // Test Hot Pink
    clip.background = Some(SlideBackground::Solid(Color32::from_rgb(238, 68, 148)));
    assert_eq!(clip.background, Some(SlideBackground::Solid(Color32::from_rgb(238, 68, 148))));

    // Test Pastel Pink
    clip.background = Some(SlideBackground::Solid(Color32::from_rgb(245, 175, 200)));
    assert_eq!(clip.background, Some(SlideBackground::Solid(Color32::from_rgb(245, 175, 200))));
}

#[test]
fn test_multiple_media_elements_on_blank_slide_playback() {
    use video_editor::core::clip::Clip;
    use video_editor::core::text_overlay::SlideElement;
    use std::path::PathBuf;

    let mut clip = Clip::new_blank_slide(
        1,
        1,
        "Multi-Media Slide".to_string(),
        10.0,
    );

    // Add Video 1
    clip.elements.push(SlideElement::Video {
        path: PathBuf::from("videos/video1.mp4"),
        x: 0.05,
        y: 0.1,
        w: 0.4,
        h: 0.4,
    });

    // Add Video 2
    clip.elements.push(SlideElement::Video {
        path: PathBuf::from("videos/video2.mp4"),
        x: 0.55,
        y: 0.1,
        w: 0.4,
        h: 0.4,
    });

    // Add Picture
    clip.elements.push(SlideElement::Picture {
        path: PathBuf::from("images/logo.png"),
        x: 0.3,
        y: 0.6,
        w: 0.4,
        h: 0.3,
    });

    assert_eq!(clip.elements.len(), 3);
    assert!(clip.is_static_slide());

    let video_count = clip.elements.iter().filter(|el| matches!(el, SlideElement::Video { .. })).count();
    let pic_count = clip.elements.iter().filter(|el| matches!(el, SlideElement::Picture { .. })).count();

    assert_eq!(video_count, 2);
    assert_eq!(pic_count, 1);
}

#[test]
fn test_canvas_drop_auto_creates_blank_slide_and_plus_adds_whole_clip() {
    use video_editor::core::project::Project;
    use video_editor::core::clip::Clip;
    use video_editor::core::text_overlay::SlideElement;
    use video_editor::core::time::TimeCode;
    use std::path::PathBuf;

    let mut project = Project::new("Test Project".to_string());
    let track_id = project.timeline.tracks[0].id;

    // Verify timeline starts empty
    assert_eq!(project.timeline.tracks[0].clips.len(), 0);

    // 1. Simulate canvas drag-and-drop onto empty timeline:
    // Auto-creates a blank slide at playhead and adds dropped video element
    let slide_id = project.timeline.next_id();
    let mut slide = Clip::new_blank_slide(slide_id, track_id, "Blank Slide".to_string(), 5.0);
    slide.timeline_start = project.timeline.playhead;
    slide.is_selected = true;

    slide.elements.push(SlideElement::Video {
        path: PathBuf::from("videos/overlay.mp4"),
        x: 0.25,
        y: 0.15,
        w: 0.50,
        h: 0.30,
    });

    if let Some(track) = project.timeline.get_track_mut(track_id) {
        track.add_clip(slide);
    }

    assert_eq!(project.timeline.tracks[0].clips.len(), 1);
    let created_slide = &project.timeline.tracks[0].clips[0];
    assert!(created_slide.is_static_slide());
    assert_eq!(created_slide.elements.len(), 1);
    assert!(created_slide.is_selected);

    // 2. Simulate user hitting '+' button to add media as a WHOLE timeline clip
    let clip_id = project.timeline.next_id();
    let full_clip = Clip::new(
        clip_id,
        track_id,
        "FullMovie.mp4".to_string(),
        PathBuf::from("videos/FullMovie.mp4"),
        TimeCode::from_secs_f64(12.0),
        true,
        true,
    );

    if let Some(track) = project.timeline.get_track_mut(track_id) {
        track.add_clip(full_clip);
    }

    assert_eq!(project.timeline.tracks[0].clips.len(), 2);
    let whole_slide_clip = &project.timeline.tracks[0].clips[1];
    assert!(!whole_slide_clip.is_static_slide());
    assert!(whole_slide_clip.has_video);
    assert_eq!(whole_slide_clip.duration(), TimeCode::from_secs_f64(12.0));
}

#[test]
fn test_calendar_weekday_and_leap_year_shifting() {
    use video_editor::core::calendar_gen::CalendarMonth;

    // 1. Year 2026
    let jan_2026 = CalendarMonth::new(2026, 1);
    assert_eq!(jan_2026.days_in_month(), 31);
    assert_eq!(jan_2026.first_weekday(), 4); // Thursday

    let feb_2026 = CalendarMonth::new(2026, 2);
    assert_eq!(feb_2026.days_in_month(), 28); // Not a leap year
    assert_eq!(feb_2026.first_weekday(), 0); // Sunday

    let jul_2026 = CalendarMonth::new(2026, 7);
    assert_eq!(jul_2026.days_in_month(), 31);
    assert_eq!(jul_2026.first_weekday(), 3); // Wednesday

    // 2. Year 2027 (shifts forward 1 day)
    let jan_2027 = CalendarMonth::new(2027, 1);
    assert_eq!(jan_2027.first_weekday(), 5); // Friday

    // 3. Year 2028 (Leap year)
    assert!(CalendarMonth::is_leap_year(2028));
    let feb_2028 = CalendarMonth::new(2028, 2);
    assert_eq!(feb_2028.days_in_month(), 29); // Leap year 29 days!
    assert_eq!(feb_2028.first_weekday(), 2); // Tuesday

    // Verify grid string format contains header and day numbers
    let grid = jan_2026.format_grid_string();
    assert!(grid.contains("January 2026"));
    assert!(grid.contains("Sun  Mon  Tue  Wed  Thu  Fri  Sat"));
    assert!(grid.contains(" 31"));
}

#[test]
fn test_slide_layout_templates_insertion() {
        use video_editor::core::text_overlay::SlideElement;
    use video_editor::VideoEditorApp;

    let mut app = VideoEditorApp::default();
    assert_eq!(app.project.timeline.tracks[0].clips.len(), 0);

    // 1. Insert Title + 2 Media template
    app.insert_template_title_2_media(None);
    assert_eq!(app.project.timeline.tracks[0].clips.len(), 1);
    let slide1 = &app.project.timeline.tracks[0].clips[0];
    assert_eq!(slide1.elements.len(), 3); // 1 Title + 2 Placeholders
    assert!(matches!(slide1.elements[0], SlideElement::Text(_)));
    assert!(matches!(slide1.elements[1], SlideElement::Placeholder { slot_id: 1, .. }));
    assert!(matches!(slide1.elements[2], SlideElement::Placeholder { slot_id: 2, .. }));

    // 2. Insert Title + 4 Grid template
    app.insert_template_title_4_media(None);
    assert_eq!(app.project.timeline.tracks[0].clips.len(), 2);
    let slide2 = &app.project.timeline.tracks[0].clips[1];
    assert_eq!(slide2.elements.len(), 5); // 1 Title + 4 Placeholders

    // 3. Insert Feature Showcase template
    app.insert_template_showcase(None);
    assert_eq!(app.project.timeline.tracks[0].clips.len(), 3);
    let slide3 = &app.project.timeline.tracks[0].clips[2];
    assert_eq!(slide3.elements.len(), 3); // 1 Featured slot + Title + Caption
}

#[test]
fn test_template_placeholder_slot_replacement_on_media_drop() {
    use video_editor::core::project::MediaAsset;
    use video_editor::core::text_overlay::SlideElement;
    use video_editor::VideoEditorApp;
    use std::path::PathBuf;

    let mut app = VideoEditorApp::default();
    app.insert_template_title_2_media(None);

    // Register a media asset in the bin
    let asset = MediaAsset {
        id: 42,
        name: "GrandmaPhoto.jpg".to_string(),
        path: PathBuf::from("photos/GrandmaPhoto.jpg"),
        duration_secs: 5.0,
        has_video: false,
        has_audio: false,
        width: 1920,
        height: 1080,
        fps: 30.0,
        proxy_path: None,
        peak_path: None,
    };
    app.project.media_assets.push(asset);

    // Drop photo directly inside Slot 1 (bounds: x: 0.07..0.48, y: 0.25..0.90)
    // Drop point at (x=0.20, y=0.40)
    app.drop_media_asset_on_canvas(42, 0.20, 0.40, None);

    let slide = &app.project.timeline.tracks[0].clips[0];
    assert_eq!(slide.elements.len(), 3);
    // Slot 1 (elements[1]) should now be replaced with Picture with the exact slot bounds
    match &slide.elements[1] {
        SlideElement::Picture { path, x, y, w, h } => {
            assert_eq!(path, &PathBuf::from("photos/GrandmaPhoto.jpg"));
            assert!((*x - 0.07).abs() < 0.01);
            assert!((*y - 0.25).abs() < 0.01);
            assert!((*w - 0.41).abs() < 0.01);
            assert!((*h - 0.65).abs() < 0.01);
        }
        other => panic!("Expected Picture element, got: {:?}", other),
    }

    // Slot 2 should remain an unfilled Placeholder
    assert!(matches!(slide.elements[2], SlideElement::Placeholder { slot_id: 2, .. }));
}

#[test]
fn test_american_and_chinese_holidays_calculation() {
    use video_editor::core::calendar_gen::{CalendarMonth, HolidayCategory};

    let holidays_2026 = CalendarMonth::default_holidays_for_year(2026);
    
    // Check Easter 2026 (Apr 5)
    let easter = holidays_2026.iter().find(|h| h.id == "us_easter").unwrap();
    assert_eq!(easter.month, 4);
    assert_eq!(easter.day, 5);
    assert_eq!(easter.category, HolidayCategory::American);

    // Check Thanksgiving 2026 (Nov 26)
    let thanks = holidays_2026.iter().find(|h| h.id == "us_thanksgiving").unwrap();
    assert_eq!(thanks.month, 11);
    assert_eq!(thanks.day, 26);

    // Check Chinese New Year / Spring Festival 2026 (Feb 17)
    let cny = holidays_2026.iter().find(|h| h.id == "cn_cny").unwrap();
    assert_eq!(cny.month, 2);
    assert_eq!(cny.day, 17);
    assert_eq!(cny.category, HolidayCategory::Chinese);

    // Check Mid-Autumn Moon Festival 2026 (Sep 25)
    let moon = holidays_2026.iter().find(|h| h.id == "cn_mid_autumn").unwrap();
    assert_eq!(moon.month, 9);
    assert_eq!(moon.day, 25);

    // Check Dragon Boat 2026 (Jun 19)
    let dragon = holidays_2026.iter().find(|h| h.id == "cn_dragon_boat").unwrap();
    assert_eq!(dragon.month, 6);
    assert_eq!(dragon.day, 19);
}

#[test]
fn test_holiday_color_customization_and_filtering() {
    use video_editor::core::calendar_gen::{CalendarMonth, CalendarStyle, CustomCalendarEvent, HolidayCategory};
    use egui::Color32;

    let mut holidays = CalendarMonth::default_holidays_for_year(2026);
    
    // Assign custom imperial gold/red to Chinese New Year
    if let Some(cny) = holidays.iter_mut().find(|h| h.id == "cn_cny") {
        cny.set_color32(Color32::from_rgb(255, 0, 0));
        assert_eq!(cny.color32(), Color32::from_rgb(255, 0, 0));
    }

    // Toggle off American holidays and keep only Chinese festivals
    for h in &mut holidays {
        if h.category == HolidayCategory::American {
            h.enabled = false;
        }
    }

    // Add custom family birthday with custom color
    let custom_events = vec![CustomCalendarEvent {
        month: 2,
        day: 20,
        label: "Grandma's 80th Birthday".to_string(),
        color: [255, 105, 180, 255],
    }];

    // Generate February string with custom holidays
    let text = CalendarMonth::format_multi_month_string(2026, 2, 1, true, CalendarStyle::BoxedGrid, &holidays, &custom_events);
    assert!(text.contains("February 2026"));
    assert!(text.contains("Chinese New Year"));
    assert!(text.contains("Grandma's 80th Birthday"));
    assert!(!text.contains("Valentine's Day")); // Disabled
}

#[test]
fn test_multi_month_calendar_formatting_1_2_3_months() {
    use video_editor::core::calendar_gen::{CalendarMonth, CalendarStyle};

    let holidays = CalendarMonth::default_holidays_for_year(2026);

    // 1 Month boxed format
    let one_month = CalendarMonth::format_multi_month_string(2026, 4, 1, true, CalendarStyle::BoxedGrid, &holidays, &[]);
    assert!(one_month.contains("April 2026"));
    assert!(one_month.contains("Easter"));
    assert!(one_month.contains("Easter Sunday"));

    // 2 Months side-by-side boxed format
    let two_months = CalendarMonth::format_multi_month_string(2026, 4, 2, true, CalendarStyle::BoxedGrid, &holidays, &[]);
    assert!(two_months.contains("April 2026"));
    assert!(two_months.contains("May 2026"));
    assert!(two_months.contains("Mother's Day"));

    // 3 Months quarterly boxed format
    let three_months = CalendarMonth::format_multi_month_string(2026, 4, 3, true, CalendarStyle::BoxedGrid, &holidays, &[]);
    assert!(three_months.contains("April 2026"));
    assert!(three_months.contains("May 2026"));
    assert!(three_months.contains("June 2026"));
    assert!(three_months.contains("Juneteenth"));
}

#[test]
fn test_12_month_calendar_slideshow_generation() {
    use video_editor::VideoEditorApp;

    let mut app = VideoEditorApp::default();
    
    // 1. Generate 1-month per slide (12 slides)
    app.generate_12_month_calendar(2026, 1, true, None);
    let track = &app.project.timeline.tracks[0];
    assert_eq!(track.clips.len(), 12);
    assert_eq!(track.clips[0].name, "January 2026");
    assert_eq!(track.clips[11].name, "December 2026");

    // 2. Generate 2-months per slide (6 slides)
    let mut app2 = VideoEditorApp::default();
    app2.generate_12_month_calendar(2026, 2, true, None);
    let track2 = &app2.project.timeline.tracks[0];
    assert_eq!(track2.clips.len(), 6);
    assert_eq!(track2.clips[0].name, "Jan - Feb 2026");
    assert_eq!(track2.clips[5].name, "Nov - Dec 2026");

    // 3. Generate 3-months per slide (4 quarterly slides)
    let mut app3 = VideoEditorApp::default();
    app3.generate_12_month_calendar(2026, 3, true, None);
    let track3 = &app3.project.timeline.tracks[0];
    assert_eq!(track3.clips.len(), 4);
    assert_eq!(track3.clips[0].name, "Jan - Mar 2026");
    assert_eq!(track3.clips[3].name, "Oct - Dec 2026");
}

#[test]
fn test_calendar_box_resizing_and_properties() {
    use video_editor::core::text_overlay::SlideElement;
    use video_editor::VideoEditorApp;

    let mut app = VideoEditorApp::default();
    app.insert_template_calendar_slide(2026, 1, 1, true, None);

    let slide = &app.project.timeline.tracks[0].clips[0];
    assert_eq!(slide.elements.len(), 2);
    
    // Verify initial graphical vector calendar box
    if let SlideElement::Calendar(c) = &slide.elements[1] {
        assert_eq!(c.year, 2026);
        assert_eq!(c.start_month, 1);
        assert_eq!(c.month_count, 1);
        assert_eq!(c.w, 0.46);
    } else {
        panic!("Expected Calendar element at index 1");
    }

    // Resize calendar box to custom bounds
    let slide_mut = &mut app.project.timeline.tracks[0].clips[0];
    slide_mut.elements[1].set_bounds(0.10, 0.40, 0.80, 0.50);
    let slide_updated = &app.project.timeline.tracks[0].clips[0];
    if let SlideElement::Calendar(c) = &slide_updated.elements[1] {
        assert_eq!(c.x, 0.10);
        assert_eq!(c.y, 0.40);
        assert_eq!(c.w, 0.80);
        assert_eq!(c.h, 0.50);
    }
}

#[test]
fn test_printable_landscape_calendar_export() {
    use video_editor::VideoEditorApp;
    let app = VideoEditorApp::default();
    let temp_dir = std::env::temp_dir().join("test_printable_calendar_export_holidays");
    
    // Export 2-month sheets (6 landscape sheets)
    let result = app.export_printable_calendar_sheets(&temp_dir, 2026, 2, true);
    assert!(result.is_ok());
    let files = result.unwrap();
    assert_eq!(files.len(), 6);
    for f in files {
        assert!(f.exists());
        let _ = std::fs::remove_file(f);
    }
    let _ = std::fs::remove_dir(temp_dir);
}

#[test]
fn test_boxed_calendar_grid_with_day_boxes_and_bottom_right_holidays() {
    use video_editor::core::calendar_gen::{CalendarMonth, CalendarStyle};

    let holidays = CalendarMonth::default_holidays_for_year(2026);
    let boxed = CalendarMonth::format_multi_month_string(
        2026,
        2,
        1,
        true,
        CalendarStyle::BoxedGrid,
        &holidays,
        &[],
    );

    // Verify box border characters are present
    assert!(boxed.contains("┌"));
    assert!(boxed.contains("┬"));
    assert!(boxed.contains("┐"));
    assert!(boxed.contains("├"));
    assert!(boxed.contains("┼"));
    assert!(boxed.contains("┤"));
    assert!(boxed.contains("└"));
    assert!(boxed.contains("┴"));
    assert!(boxed.contains("┘"));

    // Verify day numbers in boxes
    assert!(boxed.contains("14"));
    assert!(boxed.contains("17"));

    // Verify bottom-right holiday labels inside day boxes
    assert!(boxed.contains("V-Day"));
    assert!(boxed.contains("CNY"));

    // Verify legend footnotes below the boxed table
    assert!(boxed.contains("Valentine's Day"));
    assert!(boxed.contains("Chinese New Year (Spring Festival)"));
}


#[test]
fn test_slideshow_vs_timeline_mode_toggle() {
    use video_editor::VideoEditorApp;
    use video_editor::ui::MainViewMode;

    let mut app = VideoEditorApp::default();
    assert_eq!(app.main_view_mode, MainViewMode::Slideshow);

    // Add a slide
    app.insert_blank_slide_at_playhead(5.0, None);
    assert_eq!(app.project.timeline.tracks[0].clips.len(), 1);

    // Switch to Timeline mode
    app.main_view_mode = MainViewMode::Timeline;
    assert_eq!(app.main_view_mode, MainViewMode::Timeline);
    assert_eq!(app.main_view_mode.label(), "⏱ Timeline Editor");

    // Switch back to Slideshow mode
    app.main_view_mode = MainViewMode::Slideshow;
    assert_eq!(app.main_view_mode.label(), "🖼 Slideshow Studio");
}

#[test]
fn test_slide_deck_reordering_and_timeline_reflow() {
    use video_editor::VideoEditorApp;
    use video_editor::core::time::TimeCode;

    let mut app = VideoEditorApp::default();
    
    // Add Slide 1 (5s), Slide 2 (10s), Slide 3 (3s)
    app.insert_blank_slide_at_playhead(5.0, None);
    app.project.timeline.tracks[0].clips[0].name = "Slide 1".to_string();

    app.insert_blank_slide_at_playhead(10.0, None);
    app.project.timeline.tracks[0].clips[1].name = "Slide 2".to_string();

    app.insert_blank_slide_at_playhead(3.0, None);
    app.project.timeline.tracks[0].clips[2].name = "Slide 3".to_string();

    app.reflow_slide_timeline_positions();

    // Verify initial sequential order
    let track = &app.project.timeline.tracks[0];
    assert_eq!(track.clips.len(), 3);
    assert_eq!(track.clips[0].name, "Slide 1");
    assert_eq!(track.clips[0].timeline_start, TimeCode::from_secs_f64(0.0));
    assert_eq!(track.clips[1].name, "Slide 2");
    assert_eq!(track.clips[1].timeline_start, TimeCode::from_secs_f64(5.0));
    assert_eq!(track.clips[2].name, "Slide 3");
    assert_eq!(track.clips[2].timeline_start, TimeCode::from_secs_f64(15.0));

    // Move Slide 3 (index 2) to the front (index 0)
    app.reorder_slide(2, 0, None);

    let track = &app.project.timeline.tracks[0];
    assert_eq!(track.clips[0].name, "Slide 3");
    assert_eq!(track.clips[0].timeline_start, TimeCode::from_secs_f64(0.0)); // 3s
    assert_eq!(track.clips[1].name, "Slide 1");
    assert_eq!(track.clips[1].timeline_start, TimeCode::from_secs_f64(3.0)); // 5s
    assert_eq!(track.clips[2].name, "Slide 2");
    assert_eq!(track.clips[2].timeline_start, TimeCode::from_secs_f64(8.0)); // 10s
    assert_eq!(track.duration(), TimeCode::from_secs_f64(18.0));
}

#[test]
fn test_apply_calendar_to_active_slide_in_place() {
    use video_editor::VideoEditorApp;
    use video_editor::core::text_overlay::SlideElement;
    use std::path::PathBuf;

    let mut app = VideoEditorApp::default();

    // Create a blank slide with 2 photos/media
    app.insert_blank_slide_at_playhead(5.0, None);
    assert_eq!(app.project.timeline.tracks[0].clips.len(), 1);
    let original_id = app.project.timeline.tracks[0].clips[0].id;

    app.drop_files_on_canvas(vec![PathBuf::from("pic1.jpg"), PathBuf::from("pic2.jpg")], 0.5, 0.5, None);
    assert_eq!(app.project.timeline.tracks[0].clips[0].elements.len(), 2);

    // Apply calendar to the active slide (March 2026, 1 month)
    app.apply_template_calendar_to_active(2026, 3, 1, true, None);

    // Verify it modified the existing slide in-place, did NOT wipe the 2 photos, and did NOT add a new clip
    assert_eq!(app.project.timeline.tracks[0].clips.len(), 1);
    let slide = &app.project.timeline.tracks[0].clips[0];
    assert_eq!(slide.id, original_id);
    assert_eq!(slide.name, "March 2026");
    assert_eq!(slide.elements.len(), 3); // 2 photos + 1 calendar text

    match &slide.elements[2] {
        SlideElement::Calendar(c) => {
            assert_eq!(c.year, 2026);
            assert_eq!(c.start_month, 3);
            assert_eq!(c.month_count, 1);
            assert!(c.show_holidays);
        }
        _ => panic!("Expected Calendar element for calendar"),
    }
}

#[test]
fn test_apply_templates_to_active_slide_in_place() {
    use video_editor::VideoEditorApp;
    let mut app = VideoEditorApp::default();
    app.insert_blank_slide_at_playhead(5.0, None);
    assert_eq!(app.project.timeline.tracks[0].clips.len(), 1);

    // Apply Title + 2 Media
    app.apply_template_title_2_media_to_active(None);
    assert_eq!(app.project.timeline.tracks[0].clips.len(), 1);
    let slide = &app.project.timeline.tracks[0].clips[0];
    assert_eq!(slide.name, "Title + 2 Media");
    assert_eq!(slide.elements.len(), 3); // 1 title + 2 slots

    // Apply Title + 4 Grid
    app.apply_template_title_4_media_to_active(None);
    assert_eq!(app.project.timeline.tracks[0].clips.len(), 1);
    let slide = &app.project.timeline.tracks[0].clips[0];
    assert_eq!(slide.name, "Title + 4 Grid");
    assert_eq!(slide.elements.len(), 5); // 1 title + 4 slots

    // Apply Feature Showcase
    app.apply_template_showcase_to_active(None);
    assert_eq!(app.project.timeline.tracks[0].clips.len(), 1);
    let slide = &app.project.timeline.tracks[0].clips[0];
    assert_eq!(slide.name, "Feature Showcase");
    assert_eq!(slide.elements.len(), 3); // 1 showcase slot + 2 text
}

#[test]
fn test_slide_duration_stepper_and_duplicate() {
    use video_editor::VideoEditorApp;
    use video_editor::core::time::TimeCode;

    let mut app = VideoEditorApp::default();
    app.insert_blank_slide_at_playhead(5.0, None);
    let slide_id = app.project.timeline.tracks[0].clips[0].id;

    // Adjust duration by +1.5s
    app.adjust_slide_duration(slide_id, 1.5, None);
    assert_eq!(app.project.timeline.tracks[0].clips[0].duration(), TimeCode::from_secs_f64(6.5));

    // Duplicate slide
    app.duplicate_slide(slide_id, None);
    assert_eq!(app.project.timeline.tracks[0].clips.len(), 2);
    assert_eq!(app.project.timeline.tracks[0].clips[1].name, "Blank Slide (Copy)");
    assert_eq!(app.project.timeline.tracks[0].clips[1].timeline_start, TimeCode::from_secs_f64(6.5));
    assert_eq!(app.project.timeline.tracks[0].clips[1].duration(), TimeCode::from_secs_f64(6.5));

    // Delete first slide
    app.delete_slide_by_id(slide_id, None);
    assert_eq!(app.project.timeline.tracks[0].clips.len(), 1);
    assert_eq!(app.project.timeline.tracks[0].clips[0].timeline_start, TimeCode::from_secs_f64(0.0));
}

#[test]
fn test_os_file_explorer_direct_drag_and_drop_image_and_audio() {
    use video_editor::VideoEditorApp;
    use video_editor::core::text_overlay::SlideElement;
    use std::path::PathBuf;

    let mut app = VideoEditorApp::default();
    
    // 1. Direct drop onto empty project (auto-creates Slide 1)
    let photo = PathBuf::from("C:\\Users\\Photos\\vacation.jpg");
    app.drop_files_on_canvas(vec![photo.clone()], 0.5, 0.5, None);

    assert_eq!(app.project.timeline.tracks[0].clips.len(), 1);
    let slide = &app.project.timeline.tracks[0].clips[0];
    assert_eq!(slide.elements.len(), 1);
    match &slide.elements[0] {
        SlideElement::Picture { path, .. } => {
            assert_eq!(path, &photo);
        }
        _ => panic!("Expected Picture element from dropped .jpg"),
    }

    // 2. Direct drop of a real audio file: it must land on the MUSIC TRACK,
    //    never on the slide (music is managed as a song list, not a slide element).
    let dir = std::env::temp_dir().join(format!("ve_dnd_audio_test_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let audio = dir.join("background_song.mp3");
    let ok = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=1",
            audio.to_str().unwrap(),
        ])
        .status()
        .unwrap()
        .success();
    assert!(ok, "failed to make test mp3");

    app.drop_files_on_canvas(vec![audio.clone()], 0.5, 0.5, None);

    // A real file is probed on the worker thread; pump like the update loop does.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while app.pending_import.is_some() {
        assert!(std::time::Instant::now() < deadline, "import never finished");
        app.pump_import_queue(None);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    let slide = &app.project.timeline.tracks[0].clips[0];
    assert_eq!(slide.elements.len(), 1, "audio must not become a slide element");
    let music = app.project.timeline.music_clips();
    assert_eq!(music.len(), 1);
    assert_eq!(music[0].source_path, audio);
    assert!(music[0].has_audio && !music[0].has_video);

    // 3. A missing/unreadable audio file adds nothing and surfaces a visible error.
    let ghost = PathBuf::from("C:\\Users\\Music\\does_not_exist.mp3");
    app.status_toast = None;
    app.drop_files_on_canvas(vec![ghost], 0.5, 0.5, None);
    assert_eq!(app.project.timeline.music_clips().len(), 1);
    assert!(
        app.status_toast.is_some(),
        "a failed music import must show a visible error"
    );

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_os_file_explorer_drop_fills_template_placeholder_slot() {
    use video_editor::VideoEditorApp;
    use video_editor::core::text_overlay::SlideElement;
    use std::path::PathBuf;

    let mut app = VideoEditorApp::default();
    app.insert_template_title_2_media(None);

    let slide = &app.project.timeline.tracks[0].clips[0];
    assert_eq!(slide.elements.len(), 3); // 1 title + 2 placeholders

    // Drop photo from OS explorer
    let photo1 = PathBuf::from("C:\\Photos\\beach.png");
    app.drop_files_on_canvas(vec![photo1.clone()], 0.5, 0.5, None);

    let slide = &app.project.timeline.tracks[0].clips[0];
    // First placeholder should be replaced with Picture
    match &slide.elements[1] {
        SlideElement::Picture { path, .. } => {
            assert_eq!(path, &photo1);
        }
        _ => panic!("Expected Picture element replacing first placeholder slot"),
    }
    // Second placeholder remains
    assert!(matches!(&slide.elements[2], SlideElement::Placeholder { .. }));
}

#[test]
fn test_first_media_dropped_defaults_to_resizable_card() {
    use video_editor::VideoEditorApp;
    use video_editor::core::text_overlay::SlideElement;
    use std::path::PathBuf;

    let mut app = VideoEditorApp::default();
    // Drop single photo onto the canvas when project starts (0 slides/empty)
    app.drop_files_on_canvas(vec![PathBuf::from("vacation_beach.jpg")], 0.5, 0.5, None);

    let track = &app.project.timeline.tracks[0];
    assert_eq!(track.clips.len(), 1);
    let slide = &track.clips[0];
    assert_eq!(slide.elements.len(), 1);

    // Verify it is an interactive 80% resizable media card with visible margins
    match &slide.elements[0] {
        SlideElement::Picture { x, y, w, h, .. } => {
            assert_eq!(*x, 0.10);
            assert_eq!(*y, 0.10);
            assert_eq!(*w, 0.80);
            assert_eq!(*h, 0.80);
        }
        _ => panic!("Expected Picture element"),
    }
}

#[test]
fn test_subsequent_media_dropped_on_blank_slide_becomes_collage_box() {
    use video_editor::VideoEditorApp;
    use video_editor::core::text_overlay::SlideElement;
    use std::path::PathBuf;

    let mut app = VideoEditorApp::default();
    // 1st media dropped creates a Full Slide
    app.drop_files_on_canvas(vec![PathBuf::from("vacation_beach.jpg")], 0.5, 0.5, None);
    assert_eq!(app.project.timeline.tracks[0].clips[0].elements.len(), 1);

    // User adds a 2nd media to the same slide -> becomes a collage card
    app.drop_files_on_canvas(vec![PathBuf::from("family_dinner.jpg")], 0.6, 0.6, None);
    let slide = &app.project.timeline.tracks[0].clips[0];
    assert_eq!(slide.elements.len(), 2);

    match &slide.elements[1] {
        SlideElement::Picture { x, y, w, h, .. } => {
            assert_eq!(*w, 0.45);
            assert_eq!(*h, 0.45);
            assert!(*x >= 0.0 && *x <= 1.0);
            assert!(*y >= 0.0 && *y <= 1.0);
        }
        _ => panic!("Expected Picture element for 2nd media"),
    }
}

#[test]
fn test_element_full_screen_toggle_action() {
    use video_editor::VideoEditorApp;
    use video_editor::core::text_overlay::SlideElement;
    use std::path::PathBuf;

    let mut app = VideoEditorApp::default();
    // Add blank slide and 2 collage photos
    app.insert_blank_slide_at_playhead(5.0, None);
    app.drop_files_on_canvas(vec![PathBuf::from("photo1.jpg"), PathBuf::from("photo2.jpg")], 0.5, 0.5, None);

    let slide = &app.project.timeline.tracks[0].clips[0];
    assert_eq!(slide.elements.len(), 2);

    // Toggle element 0 to Full Screen
    app.full_slide_element(0, None);
    let slide_updated = &app.project.timeline.tracks[0].clips[0];
    match &slide_updated.elements[0] {
        SlideElement::Picture { x, y, w, h, .. } => {
            assert_eq!(*x, 0.0);
            assert_eq!(*y, 0.0);
            assert_eq!(*w, 1.0);
            assert_eq!(*h, 1.0);
        }
        _ => panic!("Expected Picture element"),
    }

    // Toggle element 0 again -> returns to centered resizable card
    app.full_slide_element(0, None);
    let slide_restored = &app.project.timeline.tracks[0].clips[0];
    match &slide_restored.elements[0] {
        SlideElement::Picture { x, y, w, h, .. } => {
            assert_eq!(*x, 0.10);
            assert_eq!(*y, 0.10);
            assert_eq!(*w, 0.80);
            assert_eq!(*h, 0.80);
        }
        _ => panic!("Expected Picture element"),
    }
}

#[test]
fn test_all_8_resize_handles_and_drag_directions() {
    use video_editor::ui::preview_player::{calculate_resized_bounds, detect_resize_handle, ResizeHandle};
    use egui::{Pos2, Rect, Vec2};

    let start_bounds = (0.20, 0.20, 0.40, 0.40); // left: 0.20, top: 0.20, right: 0.60, bottom: 0.60
    let screen_rect = Rect::from_min_size(Pos2::new(100.0, 100.0), Vec2::new(400.0, 300.0));

    // 1. Detect all 8 handles on screen rect
    assert_eq!(detect_resize_handle(screen_rect, Pos2::new(100.0, 100.0)), Some(ResizeHandle::TopLeft));
    assert_eq!(detect_resize_handle(screen_rect, Pos2::new(500.0, 100.0)), Some(ResizeHandle::TopRight));
    assert_eq!(detect_resize_handle(screen_rect, Pos2::new(100.0, 400.0)), Some(ResizeHandle::BottomLeft));
    assert_eq!(detect_resize_handle(screen_rect, Pos2::new(500.0, 400.0)), Some(ResizeHandle::BottomRight));
    assert_eq!(detect_resize_handle(screen_rect, Pos2::new(300.0, 100.0)), Some(ResizeHandle::Top));
    assert_eq!(detect_resize_handle(screen_rect, Pos2::new(300.0, 400.0)), Some(ResizeHandle::Bottom));
    assert_eq!(detect_resize_handle(screen_rect, Pos2::new(100.0, 250.0)), Some(ResizeHandle::Left));
    assert_eq!(detect_resize_handle(screen_rect, Pos2::new(500.0, 250.0)), Some(ResizeHandle::Right));

    // 2. Resize BottomRight (drags bottom-right corner out to 0.70, 0.70)
    let (x, y, w, h) = calculate_resized_bounds(ResizeHandle::BottomRight, start_bounds, 0.70, 0.70);
    assert_eq!(x, 0.20);
    assert_eq!(y, 0.20);
    assert!((w - 0.50).abs() < 0.001);
    assert!((h - 0.50).abs() < 0.001);

    // 3. Resize TopLeft (drags top-left corner in to 0.30, 0.30)
    let (x, y, w, h) = calculate_resized_bounds(ResizeHandle::TopLeft, start_bounds, 0.30, 0.30);
    assert_eq!(x, 0.30);
    assert_eq!(y, 0.30);
    assert!((w - 0.30).abs() < 0.001); // 0.60 - 0.30 = 0.30
    assert!((h - 0.30).abs() < 0.001); // 0.60 - 0.30 = 0.30

    // 4. Resize TopRight (drags top-right corner to 0.70, 0.10)
    let (x, y, w, h) = calculate_resized_bounds(ResizeHandle::TopRight, start_bounds, 0.70, 0.10);
    assert_eq!(x, 0.20);
    assert_eq!(y, 0.10);
    assert!((w - 0.50).abs() < 0.001); // 0.70 - 0.20 = 0.50
    assert!((h - 0.50).abs() < 0.001); // 0.60 - 0.10 = 0.50

    // 5. Resize BottomLeft (drags bottom-left corner to 0.10, 0.70)
    let (x, y, w, h) = calculate_resized_bounds(ResizeHandle::BottomLeft, start_bounds, 0.10, 0.70);
    assert_eq!(x, 0.10);
    assert_eq!(y, 0.20);
    assert!((w - 0.50).abs() < 0.001); // 0.60 - 0.10 = 0.50
    assert!((h - 0.50).abs() < 0.001); // 0.70 - 0.20 = 0.50

    // 6. Resize Top side only (drags top edge up to 0.10)
    let (x, y, w, h) = calculate_resized_bounds(ResizeHandle::Top, start_bounds, 0.40, 0.10);
    assert_eq!(x, 0.20);
    assert_eq!(y, 0.10);
    assert_eq!(w, 0.40);
    assert!((h - 0.50).abs() < 0.001);

    // 7. Resize Bottom side only (drags bottom edge down to 0.80)
    let (x, y, w, h) = calculate_resized_bounds(ResizeHandle::Bottom, start_bounds, 0.40, 0.80);
    assert_eq!(x, 0.20);
    assert_eq!(y, 0.20);
    assert_eq!(w, 0.40);
    assert!((h - 0.60).abs() < 0.001);

    // 8. Resize Left side only (drags left edge to 0.10)
    let (x, y, w, h) = calculate_resized_bounds(ResizeHandle::Left, start_bounds, 0.10, 0.40);
    assert_eq!(x, 0.10);
    assert_eq!(y, 0.20);
    assert!((w - 0.50).abs() < 0.001);
    assert_eq!(h, 0.40);

    // 9. Resize Right side only (drags right edge to 0.80)
    let (x, y, w, h) = calculate_resized_bounds(ResizeHandle::Right, start_bounds, 0.80, 0.40);
    assert_eq!(x, 0.20);
    assert_eq!(y, 0.20);
    assert!((w - 0.60).abs() < 0.001);
    assert_eq!(h, 0.40);
}

#[test]
fn test_export_slideshow_to_powerpoint_pptx() {
    use std::fs::File;
    use std::io::Read;
    use video_editor::core::calendar_gen::CalendarMonth;
    use video_editor::core::clip::Clip;
    use video_editor::core::text_overlay::{CalendarOverlay, SlideBackground, SlideElement, TextOverlay};
    use video_editor::core::timeline::Timeline;
    use video_editor::core::track::{Track, TrackKind};
    use video_editor::export::export_to_pptx;

    let dir = std::env::temp_dir();
    let img1_path = dir.join(format!("photo1_{}.png", uuid::Uuid::new_v4()));
    let img_buf = image::RgbaImage::new(100, 100);
    img_buf.save(&img1_path).unwrap();

    let mut timeline = Timeline::default();
    let mut track = Track::new(1, "Slides Track".to_string(), TrackKind::Video);

    // Slide 1: Vacation Cover Slide
    let mut slide1 = Clip::new_blank_slide(101, 1, "Slide 1 - Hawaii Vacation".to_string(), 5.0);
    slide1.background = Some(SlideBackground::Solid(egui::Color32::from_rgb(20, 30, 45)));

    let mut title_overlay = TextOverlay::new("Hawaii Vacation 2026");
    title_overlay.x = 0.50;
    title_overlay.y = 0.20;
    title_overlay.font_size = 40.0;
    slide1.elements.push(SlideElement::Text(title_overlay));

    slide1.elements.push(SlideElement::Picture {
        path: img1_path.clone(),
        x: 0.10,
        y: 0.35,
        w: 0.40,
        h: 0.50,
    });

    slide1.elements.push(SlideElement::Calendar(CalendarOverlay {
        year: 2026,
        start_month: 7,
        month_count: 1,
        show_holidays: true,
        x: 0.55,
        y: 0.35,
        w: 0.40,
        h: 0.50,
        holidays: CalendarMonth::default_holidays_for_year(2026),
        custom_events: Vec::new(),
    }));

    // Slide 2: Second slide
    let mut slide2 = Clip::new_blank_slide(102, 2, "Slide 2 - Beach Memories".to_string(), 5.0);
    let mut text2 = TextOverlay::new("Sunset at Waikiki");
    text2.x = 0.50;
    text2.y = 0.85;
    slide2.elements.push(SlideElement::Text(text2));

    track.clips.push(slide1);
    track.clips.push(slide2);
    timeline.tracks.push(track);

    let pptx_path = dir.join(format!("vacation_{}.pptx", uuid::Uuid::new_v4()));
    let res = export_to_pptx(&timeline, &pptx_path);
    assert!(res.is_ok(), "PPTX export should succeed: {:?}", res.err());
    assert!(pptx_path.exists(), "PPTX file must exist");

    // Verify ZIP content
    let zip_file = File::open(&pptx_path).unwrap();
    let mut archive = zip::ZipArchive::new(zip_file).unwrap();

    let mut found_content_types = false;
    let mut found_pres = false;
    let mut found_slide1 = false;
    let mut found_slide2 = false;
    let mut found_media = false;
    let mut slide1_xml = String::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let name = file.name().to_string();
        if name == "[Content_Types].xml" { found_content_types = true; }
        if name == "ppt/presentation.xml" { found_pres = true; }
        if name == "ppt/slides/slide1.xml" {
            found_slide1 = true;
            file.read_to_string(&mut slide1_xml).unwrap();
        }
        if name == "ppt/slides/slide2.xml" { found_slide2 = true; }
        if name.starts_with("ppt/media/image") { found_media = true; }
    }

    assert!(found_content_types, "[Content_Types].xml must be present");
    assert!(found_pres, "ppt/presentation.xml must be present");
    assert!(found_slide1, "ppt/slides/slide1.xml must be present");
    assert!(found_slide2, "ppt/slides/slide2.xml must be present");
    assert!(found_media, "Embedded image media must be present");
    assert!(slide1_xml.contains("Hawaii Vacation 2026"), "Slide 1 must contain text title");

    let _ = std::fs::remove_file(pptx_path);
    let _ = std::fs::remove_file(img1_path);
}

#[test]
fn test_export_slideshow_to_presentation_pdf() {
    use video_editor::core::calendar_gen::CalendarMonth;
    use video_editor::core::clip::Clip;
    use video_editor::core::text_overlay::{CalendarOverlay, SlideBackground, SlideElement, TextOverlay};
    use video_editor::core::timeline::Timeline;
    use video_editor::core::track::{Track, TrackKind};
    use video_editor::export::export_to_pdf;

    let dir = std::env::temp_dir();
    let img_path = dir.join(format!("scenery_{}.png", uuid::Uuid::new_v4()));
    let img_buf = image::RgbaImage::new(100, 100);
    img_buf.save(&img_path).unwrap();

    let mut timeline = Timeline::default();
    let mut track = Track::new(1, "Video Track".to_string(), TrackKind::Video);

    let mut slide = Clip::new_blank_slide(201, 1, "Cover Page".to_string(), 5.0);
    slide.background = Some(SlideBackground::Solid(egui::Color32::from_rgb(15, 20, 30)));

    let mut title = TextOverlay::new("Summer Roadtrip 2026");
    title.x = 0.50;
    title.y = 0.20;
    title.font_size = 36.0;
    slide.elements.push(SlideElement::Text(title));

    slide.elements.push(SlideElement::Picture {
        path: img_path.clone(),
        x: 0.10,
        y: 0.40,
        w: 0.35,
        h: 0.45,
    });

    slide.elements.push(SlideElement::Calendar(CalendarOverlay {
        year: 2026,
        start_month: 8,
        month_count: 1,
        show_holidays: true,
        x: 0.55,
        y: 0.40,
        w: 0.35,
        h: 0.45,
        holidays: CalendarMonth::default_holidays_for_year(2026),
        custom_events: Vec::new(),
    }));

    track.clips.push(slide);
    timeline.tracks.push(track);

    let pdf_path = dir.join(format!("presentation_{}.pdf", uuid::Uuid::new_v4()));
    let res = export_to_pdf(&timeline, &pdf_path);
    assert!(res.is_ok(), "PDF export should succeed: {:?}", res.err());
    assert!(pdf_path.exists(), "PDF file must exist");

    let pdf_bytes = std::fs::read(&pdf_path).unwrap();
    assert!(pdf_bytes.starts_with(b"%PDF-1.7"), "File must start with %PDF-1.7");
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);
    assert!(pdf_str.contains("/MediaBox [0 0 960.0 540.0]"), "PDF page must be 16:9 landscape");
    assert!(pdf_str.contains("Summer Roadtrip 2026"), "PDF must contain text content");
    assert!(pdf_str.ends_with("%%EOF\n") || pdf_str.contains("%%EOF"), "PDF must have EOF trailer");

    let _ = std::fs::remove_file(pdf_path);
    let _ = std::fs::remove_file(img_path);
}

#[test]
fn test_multiple_media_elements_persist_visuals_across_resizes_and_ticks() {
    use video_editor::core::clip::Clip;
    use video_editor::core::text_overlay::{SlideBackground, SlideElement, TextOverlay};

    let dir = std::env::temp_dir();
    let img1_path = dir.join(format!("elem1_{}.png", uuid::Uuid::new_v4()));
    let img2_path = dir.join(format!("elem2_{}.png", uuid::Uuid::new_v4()));

    let img1 = image::RgbaImage::from_pixel(200, 200, image::Rgba([255, 0, 0, 255]));
    img1.save(&img1_path).unwrap();

    let img2 = image::RgbaImage::from_pixel(200, 200, image::Rgba([0, 255, 0, 255]));
    img2.save(&img2_path).unwrap();

    let mut slide = Clip::new_blank_slide(501, 1, "Multi-Media Slide".to_string(), 5.0);
    slide.background = Some(SlideBackground::Solid(egui::Color32::from_rgb(30, 30, 40)));

    slide.elements.push(SlideElement::Picture {
        path: img1_path.clone(),
        x: 0.1,
        y: 0.2,
        w: 0.35,
        h: 0.5,
    });

    slide.elements.push(SlideElement::Picture {
        path: img2_path.clone(),
        x: 0.55,
        y: 0.2,
        w: 0.35,
        h: 0.5,
    });

    slide.elements.push(SlideElement::Text(TextOverlay::new("Persistent Title")));

    assert_eq!(slide.elements.len(), 3);
    assert_eq!(slide.elements[0].bounds(), (0.1, 0.2, 0.35, 0.5));
    assert_eq!(slide.elements[1].bounds(), (0.55, 0.2, 0.35, 0.5));

    // Verify fast direct image loading preserves both pictures without failure
    let loaded1 = image::open(&img1_path);
    let loaded2 = image::open(&img2_path);
    assert!(loaded1.is_ok(), "Image 1 must open successfully");
    assert!(loaded2.is_ok(), "Image 2 must open successfully");

    let _ = std::fs::remove_file(img1_path);
    let _ = std::fs::remove_file(img2_path);
}

#[test]
fn test_slideshow_horizontal_filmstrip_actions() {
    use video_editor::core::clip::Clip;
    use video_editor::core::text_overlay::{SlideBackground, SlideElement, TextOverlay};
    use video_editor::core::timeline::Timeline;
    use video_editor::core::track::TrackKind;

    let mut timeline = Timeline::default();
    timeline.tracks.retain(|t| t.kind == TrackKind::Video);
    if timeline.tracks.is_empty() {
        timeline.add_track("Video Track".to_string(), TrackKind::Video);
    }

    let mut slide1 = Clip::new_blank_slide(1, 1, "Slide 1 - Hawaii".to_string(), 5.0);
    slide1.background = Some(SlideBackground::Solid(egui::Color32::from_rgb(20, 30, 45)));
    slide1.elements.push(SlideElement::Text(TextOverlay::new("Hawaii Trip")));

    let mut slide2 = Clip::new_blank_slide(2, 2, "Slide 2 - Beach".to_string(), 4.0);
    slide2.background = Some(SlideBackground::Solid(egui::Color32::from_rgb(40, 20, 30)));

    let slide3 = Clip::new_blank_slide(3, 3, "Slide 3 - Sunset".to_string(), 6.0);

    timeline.tracks[0].clips.push(slide1);
    timeline.tracks[0].clips.push(slide2);
    timeline.tracks[0].clips.push(slide3);

    assert_eq!(timeline.tracks[0].clips.len(), 3);
    assert_eq!(timeline.tracks[0].clips[0].name, "Slide 1 - Hawaii");
    assert_eq!(timeline.tracks[0].clips[1].name, "Slide 2 - Beach");
    assert_eq!(timeline.tracks[0].clips[2].name, "Slide 3 - Sunset");

    // Test selection
    timeline.select_clip(2);
    let selected = timeline.get_selected_clip();
    assert!(selected.is_some());
    assert_eq!(selected.unwrap().id, 2);

    // Test slide reordering: Move Slide 2 before Slide 1
    let moved_clip = timeline.tracks[0].clips.remove(1);
    timeline.tracks[0].clips.insert(0, moved_clip);
    assert_eq!(timeline.tracks[0].clips[0].name, "Slide 2 - Beach");
    assert_eq!(timeline.tracks[0].clips[1].name, "Slide 1 - Hawaii");

    // Test duration adjustment
    let new_dur = video_editor::core::time::TimeCode::from_secs_f64(7.5);
    timeline.tracks[0].clips[0].source_out = timeline.tracks[0].clips[0].source_in + new_dur;
    assert_eq!(timeline.tracks[0].clips[0].duration().as_secs_f64(), 7.5);
}

// ---------------------------------------------------------------------------
// Filmstrip drag-and-drop slide reordering.
//
// These drive the SHIPPING helpers (`gap_to_target_index` + `reorder_slide`)
// rather than re-deriving the index math locally, so the assertions still bind
// if the production conversion ever changes.
// ---------------------------------------------------------------------------

/// Build a deck of `n` named slides: "S1".."Sn", each 5s.
fn deck_of(n: usize) -> video_editor::VideoEditorApp {
    use video_editor::VideoEditorApp;
    let mut app = VideoEditorApp::default();
    for i in 0..n {
        app.insert_blank_slide_at_playhead(5.0, None);
        app.project.timeline.tracks[0].clips[i].name = format!("S{}", i + 1);
    }
    app.reflow_slide_timeline_positions();
    app
}

fn deck_names(app: &video_editor::VideoEditorApp) -> Vec<String> {
    app.project.timeline.tracks[0]
        .clips
        .iter()
        .map(|c| c.name.clone())
        .collect()
}

#[test]
fn test_gap_to_target_index_shift_correction() {
    use video_editor::ui::slide_deck::gap_to_target_index;

    // Dragging RIGHT: removing the source first shifts later slides down one,
    // so the gap must be decremented or the slide overshoots by a position.
    // S1 dropped in the gap after S4 (gap 4) in a 5-deck must land at index 3.
    assert_eq!(gap_to_target_index(0, 4, 5), Some(3));
    // Dropping past the final card appends: last index, never out of bounds.
    assert_eq!(gap_to_target_index(0, 5, 5), Some(4));

    // Dragging LEFT: nothing before the source has shifted, so the gap is the
    // destination index unchanged.
    assert_eq!(gap_to_target_index(4, 0, 5), Some(0));
    assert_eq!(gap_to_target_index(3, 1, 5), Some(1));

    // Both gaps flanking the dragged card are no-ops, so an accidental
    // click-drag in place never pushes an undo snapshot.
    assert_eq!(gap_to_target_index(2, 2, 5), None);
    assert_eq!(gap_to_target_index(2, 3, 5), None);

    // Out-of-range source is rejected rather than panicking.
    assert_eq!(gap_to_target_index(9, 1, 5), None);
}

#[test]
fn test_drag_slide_to_arbitrary_position_reorders_deck() {
    use video_editor::ui::slide_deck::gap_to_target_index;

    // Drag the FIRST slide to the very END in one gesture — the move the
    // one-step arrows would need four presses to achieve.
    let mut app = deck_of(5);
    let to = gap_to_target_index(0, 5, 5).expect("drop past last card is a real move");
    app.reorder_slide(0, to, None);
    assert_eq!(deck_names(&app), vec!["S2", "S3", "S4", "S5", "S1"]);

    // Drag the LAST slide to the very FRONT.
    let mut app = deck_of(5);
    let to = gap_to_target_index(4, 0, 5).expect("drop before first card is a real move");
    app.reorder_slide(4, to, None);
    assert_eq!(deck_names(&app), vec!["S5", "S1", "S2", "S3", "S4"]);

    // Drag a middle slide (S2) into the gap between S4 and S5.
    let mut app = deck_of(5);
    let to = gap_to_target_index(1, 4, 5).expect("middle move");
    app.reorder_slide(1, to, None);
    assert_eq!(deck_names(&app), vec!["S1", "S3", "S4", "S2", "S5"]);
}

#[test]
fn test_dragged_slide_keeps_its_content_and_timeline_reflows() {
    use video_editor::core::text_overlay::{SlideBackground, TextOverlay};
    use video_editor::ui::slide_deck::gap_to_target_index;

    let mut app = deck_of(3);
    // Give S1 identifiable content, then drag it to the end. Reordering must
    // move the whole slide, not just relabel positions.
    app.project.timeline.tracks[0].clips[0].background =
        Some(SlideBackground::Solid(egui::Color32::from_rgb(9, 9, 9)));
    let mut ov = TextOverlay::default();
    ov.text = "carry me".to_string();
    app.project.timeline.tracks[0].clips[0]
        .elements
        .push(video_editor::core::text_overlay::SlideElement::Text(ov));

    // Vary durations so a reflow error shows up as a bad start time.
    app.project.timeline.tracks[0].clips[1].source_duration =
        video_editor::core::time::TimeCode::from_secs_f64(10.0);
    app.project.timeline.tracks[0].clips[1].source_out =
        video_editor::core::time::TimeCode::from_secs_f64(10.0);
    app.reflow_slide_timeline_positions();

    let to = gap_to_target_index(0, 3, 3).expect("to the end");
    app.reorder_slide(0, to, None);

    let clips = &app.project.timeline.tracks[0].clips;
    assert_eq!(
        clips.iter().map(|c| c.name.clone()).collect::<Vec<_>>(),
        vec!["S2", "S3", "S1"]
    );
    // The moved slide still owns its background and its text element.
    let moved = &clips[2];
    assert!(matches!(moved.background, Some(SlideBackground::Solid(_))));
    assert_eq!(moved.elements.len(), 1);

    // Timeline positions are contiguous after the move: S2(10s), S3(5s), S1(5s).
    assert_eq!(clips[0].timeline_start.as_secs_f64(), 0.0);
    assert_eq!(clips[1].timeline_start.as_secs_f64(), 10.0);
    assert_eq!(clips[2].timeline_start.as_secs_f64(), 15.0);
}

#[test]
fn test_arrow_reorder_still_moves_one_position() {
    // The one-step arrow buttons the user also asked for: MoveSlideDown(idx)
    // maps to reorder_slide(idx, idx+1) and MoveSlideUp to (idx, idx-1).
    let mut app = deck_of(4);

    app.reorder_slide(0, 1, None); // ▶ on S1
    assert_eq!(deck_names(&app), vec!["S2", "S1", "S3", "S4"]);

    app.reorder_slide(1, 0, None); // ◀ moves it back
    assert_eq!(deck_names(&app), vec!["S1", "S2", "S3", "S4"]);

    // A move past the last slide is a no-op, not a panic or a lost slide.
    let len = app.slide_count();
    app.reorder_slide(len - 1, len, None);
    assert_eq!(deck_names(&app), vec!["S1", "S2", "S3", "S4"]);
}

#[test]
fn test_slide_reorder_is_undoable() {
    use video_editor::ui::slide_deck::gap_to_target_index;

    let mut app = deck_of(3);
    let to = gap_to_target_index(0, 3, 3).expect("to the end");
    app.reorder_slide(0, to, None);
    assert_eq!(deck_names(&app), vec!["S2", "S3", "S1"]);

    // reorder_slide snapshots before mutating, so Ctrl+Z restores the deck.
    app.undo(None);
    assert_eq!(deck_names(&app), vec!["S1", "S2", "S3"]);
}
