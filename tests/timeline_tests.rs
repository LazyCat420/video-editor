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
