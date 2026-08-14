use std::path::PathBuf;
use video_editor::core::clip::Clip;
use video_editor::core::project::Project;
use video_editor::core::time::TimeCode;
use video_editor::core::timeline::Timeline;
use video_editor::core::track::TrackKind;

#[test]
fn test_timecode_conversions() {
    let tc = TimeCode::from_secs_f64(65.5);
    assert_eq!(tc.to_timecode_str(), "00:01:05.500");
    assert_eq!(tc.to_smpte_str(30.0), "00:01:05:15");

    let frames = tc.as_frames(30.0);
    assert_eq!(frames, 1965);

    let from_frames = TimeCode::from_frames(1965, 30.0);
    assert_eq!(from_frames.as_secs_f64(), 65.5);
}

#[test]
fn test_senior_friendly_default_tracks() {
    let timeline = Timeline::new(30.0);
    assert_eq!(timeline.tracks.len(), 2);
    assert_eq!(timeline.tracks[0].name, "🎬 Video Track");
    assert_eq!(timeline.tracks[0].kind, TrackKind::Video);
    assert_eq!(timeline.tracks[1].name, "🎵 Music & Sound");
    assert_eq!(timeline.tracks[1].kind, TrackKind::Audio);
}

#[test]
fn test_clip_splitting() {
    let mut clip = Clip::new(
        1,
        100,
        "Test Video".to_string(),
        PathBuf::from("/path/to/video.mp4"),
        TimeCode::from_secs_f64(10.0),
        true,
        true,
    );
    clip.timeline_start = TimeCode::from_secs_f64(5.0);

    // Split at timeline timestamp 8.0s (3.0s into clip)
    let second_half = clip.split_at(TimeCode::from_secs_f64(8.0), 2);
    assert!(second_half.is_some());
    let second = second_half.unwrap();

    // First half assertions
    assert_eq!(clip.id, 1);
    assert_eq!(clip.timeline_start.as_secs_f64(), 5.0);
    assert_eq!(clip.duration().as_secs_f64(), 3.0);
    assert_eq!(clip.source_in.as_secs_f64(), 0.0);
    assert_eq!(clip.source_out.as_secs_f64(), 3.0);

    // Second half assertions
    assert_eq!(second.id, 2);
    assert_eq!(second.timeline_start.as_secs_f64(), 8.0);
    assert_eq!(second.duration().as_secs_f64(), 7.0);
    assert_eq!(second.source_in.as_secs_f64(), 3.0);
    assert_eq!(second.source_out.as_secs_f64(), 10.0);
}

#[test]
fn test_timeline_magnetic_snapping() {
    let mut timeline = Timeline::new(30.0);
    let track_id = timeline.tracks[0].id;

    let mut clip1 = Clip::new(
        1,
        track_id,
        "Clip 1".to_string(),
        PathBuf::from("clip1.mp4"),
        TimeCode::from_secs_f64(5.0),
        true,
        false,
    );
    clip1.timeline_start = TimeCode::ZERO;

    if let Some(track) = timeline.get_track_mut(track_id) {
        track.add_clip(clip1);
    }

    let pps = 50.0;
    // Snap threshold is 10px / 50pps = 0.2s
    // Target 4.9s should snap to 5.0s (end of clip 1)
    let snap1 = timeline.find_snap_point(TimeCode::from_secs_f64(4.9), pps);
    assert_eq!(snap1.as_secs_f64(), 5.0);

    // Target 0.1s should snap to 0.0s (origin)
    let snap2 = timeline.find_snap_point(TimeCode::from_secs_f64(0.1), pps);
    assert_eq!(snap2.as_secs_f64(), 0.0);

    // Target 2.5s (far from boundaries) should remain 2.5s
    let snap3 = timeline.find_snap_point(TimeCode::from_secs_f64(2.5), pps);
    assert_eq!(snap3.as_secs_f64(), 2.5);
}

#[test]
fn test_project_save_load_json() {
    let mut project = Project::new("Test NLE Project".to_string());
    let track_id = project.timeline.tracks[0].id;
    let clip = Clip::new(
        1,
        track_id,
        "Clip A".to_string(),
        PathBuf::from("test.mp4"),
        TimeCode::from_secs_f64(12.0),
        true,
        true,
    );
    if let Some(t) = project.timeline.get_track_mut(track_id) {
        t.add_clip(clip);
    }

    let temp_file = std::env::temp_dir().join("test_video_editor_project.vproj");
    let save_res = project.save_to_file(&temp_file);
    assert!(save_res.is_ok());

    let loaded = Project::load_from_file(&temp_file);
    assert!(loaded.is_ok());
    let loaded_proj = loaded.unwrap();
    assert_eq!(loaded_proj.name, "Test NLE Project");
    assert_eq!(loaded_proj.timeline.tracks[0].clips.len(), 1);
    assert_eq!(loaded_proj.timeline.tracks[0].clips[0].name, "Clip A");

    let _ = std::fs::remove_file(temp_file);
}

#[test]
fn test_drag_math_to_zero() {
    let mut clip = Clip::new(
        1,
        1,
        "Test".to_string(),
        PathBuf::from("video.mp4"),
        TimeCode::from_secs_f64(10.0),
        true,
        true,
    );
    clip.timeline_start = TimeCode::from_secs_f64(4.0);

    let pps = 50.0;
    // Dragging left by -250px (-5.0s)
    let delta_x: f32 = -250.0;
    let cur_secs = clip.timeline_start.as_secs_f64();
    let delta_secs = (delta_x / pps) as f64;
    let new_secs = (cur_secs + delta_secs).max(0.0);
    let new_start = TimeCode::from_secs_f64(new_secs);

    assert_eq!(new_start.as_secs_f64(), 0.0);
}

#[test]
fn test_universal_format_filters() {
    use video_editor::media::probe::{
        SUPPORTED_AUDIO_EXTENSIONS, SUPPORTED_IMAGE_EXTENSIONS, SUPPORTED_VIDEO_EXTENSIONS,
    };
    assert!(SUPPORTED_VIDEO_EXTENSIONS.contains(&"mp4"));
    assert!(SUPPORTED_VIDEO_EXTENSIONS.contains(&"MP4"));
    assert!(SUPPORTED_VIDEO_EXTENSIONS.contains(&"mov"));
    assert!(SUPPORTED_VIDEO_EXTENSIONS.contains(&"wmv"));
    assert!(SUPPORTED_VIDEO_EXTENSIONS.contains(&"mts"));
    assert!(SUPPORTED_AUDIO_EXTENSIONS.contains(&"mp3"));
    assert!(SUPPORTED_AUDIO_EXTENSIONS.contains(&"wav"));
    assert!(SUPPORTED_AUDIO_EXTENSIONS.contains(&"m4a"));
    assert!(SUPPORTED_IMAGE_EXTENSIONS.contains(&"jpg"));
    assert!(SUPPORTED_IMAGE_EXTENSIONS.contains(&"png"));
}

#[test]
fn test_timeline_history_undo_redo() {
    use video_editor::core::history::TimelineHistory;

    let mut history = TimelineHistory::new(50);
    let mut timeline = Timeline::new(30.0);
    let track_id = timeline.tracks[0].id;

    // Initial state: 0 clips
    assert_eq!(timeline.tracks[0].clips.len(), 0);
    assert!(!history.can_undo());

    // Action 1: Add a clip
    history.push_snapshot(&timeline);
    let clip = Clip::new(
        1,
        track_id,
        "Clip 1".to_string(),
        PathBuf::from("video.mp4"),
        TimeCode::from_secs_f64(10.0),
        true,
        true,
    );
    timeline.tracks[0].add_clip(clip);
    assert_eq!(timeline.tracks[0].clips.len(), 1);
    assert!(history.can_undo());

    // Action 2: Split the clip
    history.push_snapshot(&timeline);
    timeline.divide_clip_in_half(1);
    assert_eq!(timeline.tracks[0].clips.len(), 2);

    // Test Undo 1: Revert split -> back to 1 clip
    if let Some(prev) = history.undo(&timeline) {
        timeline = prev;
    }
    assert_eq!(timeline.tracks[0].clips.len(), 1);
    assert!(history.can_redo());

    // Test Undo 2: Revert add -> back to 0 clips
    if let Some(prev) = history.undo(&timeline) {
        timeline = prev;
    }
    assert_eq!(timeline.tracks[0].clips.len(), 0);
    assert!(!history.can_undo());

    // Test Redo 1: Back to 1 clip
    if let Some(next) = history.redo(&timeline) {
        timeline = next;
    }
    assert_eq!(timeline.tracks[0].clips.len(), 1);

    // Test Redo 2: Back to 2 clips
    if let Some(next) = history.redo(&timeline) {
        timeline = next;
    }
    assert_eq!(timeline.tracks[0].clips.len(), 2);
}

#[test]
fn test_divide_clip_in_half() {
    let mut timeline = Timeline::new(30.0);
    let track_id = timeline.tracks[0].id;

    let clip = Clip::new(
        1,
        track_id,
        "Clip 10s".to_string(),
        PathBuf::from("video.mp4"),
        TimeCode::from_secs_f64(10.0),
        true,
        true,
    );
    timeline.tracks[0].add_clip(clip);

    let divided = timeline.divide_clip_in_half(1);
    assert!(divided);
    assert_eq!(timeline.tracks[0].clips.len(), 2);
    assert_eq!(timeline.tracks[0].clips[0].duration().as_secs_f64(), 5.0);
    assert_eq!(timeline.tracks[0].clips[1].duration().as_secs_f64(), 5.0);
    assert_eq!(timeline.tracks[0].clips[1].timeline_start.as_secs_f64(), 5.0);
}

#[test]
fn test_trim_clip_to_playhead() {
    let mut timeline = Timeline::new(30.0);
    let track_id = timeline.tracks[0].id;

    let mut clip = Clip::new(
        1,
        track_id,
        "Clip 10s".to_string(),
        PathBuf::from("video.mp4"),
        TimeCode::from_secs_f64(10.0),
        true,
        true,
    );
    clip.timeline_start = TimeCode::ZERO;
    timeline.tracks[0].add_clip(clip);

    // Set playhead to 3.0s and trim start
    timeline.playhead = TimeCode::from_secs_f64(3.0);
    assert!(timeline.trim_clip_start_to_playhead(1));
    let c = timeline.get_clip(1).unwrap();
    assert_eq!(c.timeline_start.as_secs_f64(), 3.0);
    assert_eq!(c.source_in.as_secs_f64(), 3.0);
    assert_eq!(c.duration().as_secs_f64(), 7.0);

    // Set playhead to 8.0s and trim end
    timeline.playhead = TimeCode::from_secs_f64(8.0);
    assert!(timeline.trim_clip_end_to_playhead(1));
    let c2 = timeline.get_clip(1).unwrap();
    assert_eq!(c2.timeline_start.as_secs_f64(), 3.0);
    assert_eq!(c2.source_out.as_secs_f64(), 8.0);
    assert_eq!(c2.duration().as_secs_f64(), 5.0);
}

#[test]
fn test_close_gaps_magnet() {
    let mut timeline = Timeline::new(30.0);
    let track_id = timeline.tracks[0].id;

    let mut clip1 = Clip::new(
        1,
        track_id,
        "Clip 1".to_string(),
        PathBuf::from("1.mp4"),
        TimeCode::from_secs_f64(4.0),
        true,
        false,
    );
    clip1.timeline_start = TimeCode::from_secs_f64(2.0); // 2s gap at start

    let mut clip2 = Clip::new(
        2,
        track_id,
        "Clip 2".to_string(),
        PathBuf::from("2.mp4"),
        TimeCode::from_secs_f64(6.0),
        true,
        false,
    );
    clip2.timeline_start = TimeCode::from_secs_f64(10.0); // 4s gap between clips

    timeline.tracks[0].add_clip(clip1);
    timeline.tracks[0].add_clip(clip2);

    // Close all gaps
    timeline.close_gaps(None);

    let t = &timeline.tracks[0];
    assert_eq!(t.clips[0].timeline_start.as_secs_f64(), 0.0);
    assert_eq!(t.clips[0].duration().as_secs_f64(), 4.0);
    assert_eq!(t.clips[1].timeline_start.as_secs_f64(), 4.0);
    assert_eq!(t.clips[1].duration().as_secs_f64(), 6.0);
    assert_eq!(timeline.duration().as_secs_f64(), 10.0);
}

#[test]
fn test_paste_clip() {
    let mut timeline = Timeline::new(30.0);
    let track_id = timeline.tracks[0].id;

    let clip = Clip::new(
        1,
        track_id,
        "Original".to_string(),
        PathBuf::from("source.mp4"),
        TimeCode::from_secs_f64(8.0),
        true,
        true,
    );

    let pasted_id = timeline.paste_clip(clip, track_id, TimeCode::from_secs_f64(15.0));
    // paste_clip assigns the next free timeline id (default Timeline uses ids 1 and 2
    // for its two starter tracks, so the next one is 3).
    assert_eq!(pasted_id, 3);
    assert_eq!(timeline.tracks[0].clips.len(), 1);
    let pasted = timeline.get_clip(pasted_id).unwrap();
    assert_eq!(pasted.timeline_start.as_secs_f64(), 15.0);
    assert_eq!(pasted.duration().as_secs_f64(), 8.0);
    assert!(pasted.is_selected);
}

#[test]
fn test_remove_track_deletes_track_and_clips() {
    let mut timeline = Timeline::new(30.0);

    // Add a second video track and put a clip on the first track.
    let second_id = timeline.add_track("Video 2".to_string(), TrackKind::Video);
    let first_id = timeline.tracks[0].id;
    let clip = Clip::new(
        1,
        first_id,
        "Clip".to_string(),
        PathBuf::from("a.mp4"),
        TimeCode::from_secs_f64(5.0),
        true,
        false,
    );
    timeline.tracks[0].add_clip(clip);

    assert!(timeline.remove_track(first_id));
    // Default timeline has Video + Audio; removing Video leaves Audio + Video 2.
    assert_eq!(timeline.tracks.len(), 2);
    assert!(!timeline.tracks.iter().any(|t| t.id == first_id));
    assert!(timeline.tracks.iter().any(|t| t.id == second_id));

    // Track is gone, so its clips are gone with it.
    assert!(timeline.get_clip(1).is_none());
}

#[test]
fn test_reorder_track() {
    let mut timeline = Timeline::new(30.0);
    let v_id = timeline.tracks[0].id;
    let a_id = timeline.tracks[1].id;
    let v2_id = timeline.add_track("Video 2".to_string(), TrackKind::Video);

    // Move the audio track (currently index 1) so it ends up first.
    timeline.reorder_track(a_id, 0);
    assert_eq!(timeline.tracks[0].id, a_id);
    assert_eq!(timeline.tracks[1].id, v_id);
    assert_eq!(timeline.tracks[2].id, v2_id);

    // Moving to a too-large index clamps to the end.
    timeline.reorder_track(v_id, 99);
    assert_eq!(timeline.tracks.last().unwrap().id, v_id);

    // Drag downward: move a track from the top all the way to the bottom.
    let mut t2 = Timeline::new(30.0);
    let first = t2.tracks[0].id;
    let last_idx = t2.tracks.len() - 1;
    t2.reorder_track(first, last_idx);
    assert_eq!(t2.tracks.last().unwrap().id, first);
}

#[test]
fn test_scan_folder_recursive_and_dedup() {
    use video_editor::media::probe::{is_supported_media, scan_folder_for_media};

    let dir = std::env::temp_dir().join(format!("ve_scan_test_{}", std::process::id()));
    let nested = dir.join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(dir.join("a.MP4"), b"x").unwrap();
    std::fs::write(dir.join("b.mp3"), b"x").unwrap();
    std::fs::write(dir.join("ignored.txt"), b"x").unwrap();
    std::fs::write(nested.join("c.png"), b"x").unwrap();

    assert!(is_supported_media(&dir.join("a.MP4")));
    assert!(is_supported_media(&dir.join("c.png")));
    assert!(!is_supported_media(&dir.join("ignored.txt")));

    let found = scan_folder_for_media(&dir);
    assert_eq!(found.len(), 3);
    assert!(found.iter().any(|p| p.ends_with("a.MP4")));
    assert!(found.iter().any(|p| p.ends_with("b.mp3")));
    assert!(found.iter().any(|p| p.ends_with("c.png")));

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_thumbnail_extraction_and_frame_cache() {
    use video_editor::media::frame_cache::FrameCache;
    use video_editor::media::thumbnail::{downscale, extract_thumbnail};

    // Generate a small real video with ffmpeg (2s of a colored test pattern).
    let dir = std::env::temp_dir().join(format!("ve_thumb_test_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let video = dir.join("testsrc.mp4");
    let status = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc=duration=2:size=640x360:rate=30",
            "-pix_fmt",
            "yuv420p",
            video.to_str().unwrap(),
        ])
        .status();
    assert!(status.is_ok(), "ffmpeg must be installed to run this test");
    assert!(status.unwrap().success(), "ffmpeg failed to make test video");

    // 1. Standalone thumbnail extraction produces a real image.
    let thumb = extract_thumbnail(&video, 1.0);
    assert!(thumb.is_ok(), "thumbnail extraction failed: {:?}", thumb.err());
    let thumb = thumb.unwrap();
    assert!(thumb.size[0] > 0 && thumb.size[1] > 0);

    // 2. Frame cache: initial frame is cached and retrievable at 0.0s.
    let cache = FrameCache::new(40);
    let initial = cache.extract_initial_frame(&video);
    assert!(initial.is_some(), "initial frame extraction failed");
    let initial = initial.unwrap();
    assert!(cache.get_cached(&video, 0.0).is_some());

    // 3. Downscale actually shrinks it (the media-bin thumbnail source).
    let small = downscale(&initial, 192, 108);
    assert!(small.size[0] <= 192 && small.size[1] <= 108);
    assert!(small.size[0] > 0);

    let _ = std::fs::remove_dir_all(dir);
}

