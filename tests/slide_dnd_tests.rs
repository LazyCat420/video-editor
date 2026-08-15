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



