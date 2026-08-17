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

#[test]
fn test_transition_export_xfade() {
    use video_editor::core::transition::{Transition, TransitionKind};
    use video_editor::export::filter_graph::{build_ffmpeg_export_command, ExportConfig};

    let dir = std::env::temp_dir().join(format!("ve_xfade_test_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();

    // Three short, single-colour source clips (2s each).
    let mut sources = Vec::new();
    for (i, color) in ["red", "green", "blue"].iter().enumerate() {
        let p = dir.join(format!("c{}.mp4", i));
        let ok = std::process::Command::new("ffmpeg")
            .args([
                "-y",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                &format!("color=c={}:s=320x240:d=2:r=25", color),
                "-an",
                p.to_str().unwrap(),
            ])
            .status()
            .unwrap()
            .success();
        assert!(ok, "failed to make test clip {}", i);
        sources.push(p);
    }

    // Timeline: one video track, three clips in a row, with transitions on clips 2 and 3.
    let mut timeline = video_editor::core::timeline::Timeline::new(25.0);
    let track_id = timeline.tracks[0].id;
    let mut clips = Vec::new();
    for (i, src) in sources.iter().enumerate() {
        let mut clip = video_editor::core::clip::Clip::new(
            (i + 1) as u64,
            track_id,
            format!("Clip {}", i),
            src.clone(),
            video_editor::core::time::TimeCode::from_secs_f64(2.0),
            true,
            false,
        );
        if i == 1 {
            clip.transition = Some(Transition::new(TransitionKind::WipeLeft)); // 0.5s
        } else if i == 2 {
            clip.transition = Some(Transition::new(TransitionKind::CrossFade)); // 0.5s
        }
        timeline.tracks[0].add_clip(clip);
        clips.push(src.clone());
    }

    let out = dir.join("out.mp4");
    let config = ExportConfig {
        output_path: out.clone(),
        width: 320,
        height: 240,
        fps: 25.0,
        ..ExportConfig::default()
    };
    let cmd = build_ffmpeg_export_command(&timeline, &config).expect("build command");

    // Sanity: the generated graph must use xfade with the two chosen transitions.
    let fc_idx = cmd.iter().position(|a| a == "-filter_complex").unwrap();
    let fc = &cmd[fc_idx + 1];
    assert!(fc.contains("xfade=transition=wipeleft"));
    assert!(fc.contains("xfade=transition=dissolve"));

    // Run it end to end and confirm the output exists with the expected duration
    // (2+2+2 - 0.5 - 0.5 = 5.0s).
    let st = std::process::Command::new("ffmpeg").args(&cmd).status().unwrap();
    assert!(st.success(), "ffmpeg render failed");
    let out_meta = std::fs::metadata(&out).expect("output file");
    assert!(out_meta.len() > 0);

    let duration = std::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=nw=1:nk=1",
            out.to_str().unwrap(),
        ])
        .output()
        .ok();
    if let Some(dur) = duration {
        let s = String::from_utf8_lossy(&dur.stdout);
        let v: f64 = s.trim().parse().unwrap_or(0.0);
        assert!((v - 5.0).abs() < 0.2, "duration was {}", v);
    }

    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_transition_kind_catalog_completeness() {
    use std::collections::HashSet;
    use video_editor::core::transition::TransitionKind;

    let all = TransitionKind::all();
    assert_eq!(all.len(), 18, "Expected 18 transition kinds");

    let mut labels = HashSet::new();
    let mut xfade_names = HashSet::new();

    for kind in all {
        let label = kind.label();
        let xfade = kind.to_xfade();

        assert!(!label.is_empty(), "Label must not be empty");
        assert!(!xfade.is_empty(), "xfade name must not be empty");

        assert!(
            labels.insert(label),
            "Duplicate transition label: {}",
            label
        );
        assert!(
            xfade_names.insert(xfade),
            "Duplicate xfade transition name: {}",
            xfade
        );
    }
}

#[test]
fn test_transition_attachment_and_duration_mutation() {
    use std::path::PathBuf;
    use video_editor::core::clip::Clip;
    use video_editor::core::history::TimelineHistory;
    use video_editor::core::time::TimeCode;
    use video_editor::core::timeline::Timeline;
    use video_editor::core::transition::{Transition, TransitionKind};

    let mut timeline = Timeline::new(30.0);
    let track_id = timeline.tracks[0].id;

    let clip = Clip::new(
        101,
        track_id,
        "Scene A".to_string(),
        PathBuf::from("/mock/video.mp4"),
        TimeCode::from_secs_f64(5.0),
        true,
        false,
    );
    timeline.tracks[0].add_clip(clip);

    let mut history = TimelineHistory::new(20);

    // Initial state: no transition
    assert_eq!(timeline.tracks[0].clips[0].transition, None);

    // Snapshot before setting transition
    history.push_snapshot(&timeline);

    // Apply Dip to Black (0.5s)
    let clip_ref = timeline.get_clip_mut(101).unwrap();
    clip_ref.transition = Some(Transition::new(TransitionKind::DipToBlack));
    assert_eq!(
        timeline.tracks[0].clips[0].transition,
        Some(Transition {
            kind: TransitionKind::DipToBlack,
            duration_secs: 0.5,
        })
    );

    // Snapshot before changing duration
    history.push_snapshot(&timeline);

    // Change duration to 1.2s
    let clip_ref = timeline.get_clip_mut(101).unwrap();
    if let Some(tr) = clip_ref.transition.as_mut() {
        tr.duration_secs = 1.2;
    }
    assert_eq!(
        timeline.tracks[0].clips[0].transition.unwrap().duration_secs,
        1.2
    );

    // Snapshot before removing transition
    history.push_snapshot(&timeline);

    // Remove transition (hard cut)
    let clip_ref = timeline.get_clip_mut(101).unwrap();
    clip_ref.transition = None;
    assert_eq!(timeline.tracks[0].clips[0].transition, None);

    // Undo removal -> restores 1.2s transition
    timeline = history.undo(&timeline).expect("undo remove");
    assert_eq!(
        timeline.tracks[0].clips[0].transition,
        Some(Transition {
            kind: TransitionKind::DipToBlack,
            duration_secs: 1.2,
        })
    );

    // Undo duration change -> restores 0.5s transition
    timeline = history.undo(&timeline).expect("undo duration");
    assert_eq!(
        timeline.tracks[0].clips[0].transition,
        Some(Transition {
            kind: TransitionKind::DipToBlack,
            duration_secs: 0.5,
        })
    );

    // Undo application -> restores None
    timeline = history.undo(&timeline).expect("undo apply");
    assert_eq!(timeline.tracks[0].clips[0].transition, None);
}

#[test]
fn test_small_slider_scopes_and_restores_spacing() {
    use video_editor::ui::small_slider;

    let ctx = egui::Context::default();
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let initial_interact = ui.spacing().interact_size;
            let initial_rail = ui.spacing().slider_rail_height;

            small_slider(ui, 12.0, |ui| {
                assert_eq!(
                    ui.spacing().interact_size.y,
                    12.0,
                    "interact_size.y must be 12.0 inside small_slider"
                );
                assert_eq!(
                    ui.spacing().slider_rail_height,
                    4.0,
                    "slider_rail_height must be 4.0 inside small_slider"
                );
            });

            assert_eq!(
                ui.spacing().interact_size,
                initial_interact,
                "interact_size must be restored"
            );
            assert_eq!(
                ui.spacing().slider_rail_height,
                initial_rail,
                "slider_rail_height must be restored"
            );
        });
    });
}

#[test]
fn test_settings_font_scale_granularity() {
    use video_editor::ui::theme::{AppTheme, ThemeKind};

    let ctx = egui::Context::default();

    // Verify fine-grained stepping across 65% to 140% in 1% increments
    for i in 65..=140 {
        let scale = i as f32 / 100.0;
        AppTheme::configure(&ctx, ThemeKind::Dark, scale);
        let current = AppTheme::font_scale_now();
        assert!(
            (current - scale).abs() < 1e-4,
            "Font scale must match exactly at {:.2} (got {:.2})",
            scale,
            current
        );
    }
}

#[test]
fn test_font_scaling_modifies_text_styles_without_pixels_per_point_mutation() {
    use video_editor::ui::theme::{AppTheme, ThemeKind};

    let ctx = egui::Context::default();
    let initial_ppp = ctx.pixels_per_point();

    AppTheme::configure(&ctx, ThemeKind::Dark, 1.25);

    // pixels_per_point should remain the initial window DPI scale to keep mouse coordinates stable
    assert_eq!(
        ctx.pixels_per_point(),
        initial_ppp,
        "pixels_per_point must not change dynamically"
    );

    let style = ctx.style();
    let body_font = style.text_styles.get(&egui::TextStyle::Body).unwrap();
    assert_eq!(
        body_font.size,
        15.0 * 1.25,
        "Body font size must scale to 15.0 * 1.25"
    );
}

#[test]
fn test_transition_blend_all_18_kinds_matrix() {
    use egui::{Color32, ColorImage};
    use video_editor::core::transition::TransitionKind;
    use video_editor::media::{blend_fade_in, blend_transition};

    let w = 64;
    let h = 36;
    let frame_a = ColorImage {
        size: [w, h],
        pixels: vec![Color32::BLACK; w * h],
    };
    let frame_b = ColorImage {
        size: [w, h],
        pixels: vec![Color32::WHITE; w * h],
    };

    let all_kinds = [
        TransitionKind::CrossFade,
        TransitionKind::DipToBlack,
        TransitionKind::DipToWhite,
        TransitionKind::WipeLeft,
        TransitionKind::WipeRight,
        TransitionKind::WipeUp,
        TransitionKind::WipeDown,
        TransitionKind::SlideLeft,
        TransitionKind::SlideRight,
        TransitionKind::SlideUp,
        TransitionKind::SlideDown,
        TransitionKind::SmoothLeft,
        TransitionKind::CircleOpen,
        TransitionKind::CircleClose,
        TransitionKind::Radial,
        TransitionKind::ZoomIn,
        TransitionKind::SqueezeHorizontal,
        TransitionKind::Pixelate,
    ];

    assert_eq!(all_kinds.len(), 18, "Must test all 18 catalog transitions");

    for kind in all_kinds {
        for progress in [0.0f32, 0.25, 0.5, 0.75, 1.0] {
            let blended = blend_transition(&frame_a, &frame_b, kind, progress);
            assert_eq!(
                blended.size,
                [w, h],
                "Blended frame size must match input size for {:?} at t={}",
                kind,
                progress
            );
            assert_eq!(
                blended.pixels.len(),
                w * h,
                "Blended pixel count must match width*height for {:?} at t={}",
                kind,
                progress
            );

            if progress == 0.0 {
                assert_eq!(
                    blended.pixels[0],
                    Color32::BLACK,
                    "t=0.0 must be frame A for {:?}",
                    kind
                );
            } else if progress == 1.0 {
                assert_eq!(
                    blended.pixels[0],
                    Color32::WHITE,
                    "t=1.0 must be frame B for {:?}",
                    kind
                );
            }
        }
    }

    // Test fade-in helper
    let fade_black = blend_fade_in(&frame_b, TransitionKind::DipToBlack, 0.5);
    assert_eq!(fade_black.size, [w, h]);
    assert_eq!(fade_black.pixels[0].r(), 127);

    let fade_white = blend_fade_in(&frame_a, TransitionKind::DipToWhite, 0.5);
    assert_eq!(fade_white.size, [w, h]);
    assert_eq!(fade_white.pixels[0].r(), 127);
}

#[test]
fn test_export_filtergraph_single_clip_fade_in() {
    use std::path::PathBuf;
    use video_editor::core::clip::Clip;
    use video_editor::core::time::TimeCode;
    use video_editor::core::timeline::Timeline;
    use video_editor::core::transition::{Transition, TransitionKind};
    use video_editor::export::filter_graph::{build_ffmpeg_export_command, ExportConfig};

    let mut timeline = Timeline::new(30.0);
    let track_id = timeline.tracks[0].id;
    let mut clip = Clip::new(
        1,
        track_id,
        "Intro".to_string(),
        PathBuf::from("intro.mp4"),
        TimeCode::from_secs_f64(5.0),
        true,
        false,
    );
    clip.transition = Some(Transition::new(TransitionKind::DipToBlack));
    timeline.tracks[0].add_clip(clip);

    let config = ExportConfig {
        output_path: PathBuf::from("output.mp4"),
        width: 1280,
        height: 720,
        fps: 30.0,
        ..ExportConfig::default()
    };

    let cmd = build_ffmpeg_export_command(&timeline, &config).expect("build command");
    let fc_idx = cmd.iter().position(|a| a == "-filter_complex").unwrap();
    let fc = &cmd[fc_idx + 1];

    assert!(
        fc.contains("fade=t=in:st=0:d=0.500:color=black"),
        "Filter complex must contain leading fade in: {}",
        fc
    );
}

#[test]
fn test_clip_dual_transition_in_and_out_slots() {
    use std::path::PathBuf;
    use video_editor::core::clip::Clip;
    use video_editor::core::time::TimeCode;
    use video_editor::core::transition::{Transition, TransitionKind};

    let mut clip = Clip::new(
        10,
        1,
        "TestClip".to_string(),
        PathBuf::from("test.mp4"),
        TimeCode::from_secs_f64(10.0),
        true,
        true,
    );

    // Initial state: no transitions
    assert!(clip.start_transition().is_none());
    assert!(clip.end_transition().is_none());

    // Apply Beginning (In) Transition
    clip.transition_in = Some(Transition::new(TransitionKind::DipToBlack));
    assert_eq!(clip.start_transition().unwrap().kind, TransitionKind::DipToBlack);
    assert!(clip.end_transition().is_none());

    // Apply Ending (Out) Transition
    clip.transition_out = Some(Transition::new(TransitionKind::CrossFade));
    assert_eq!(clip.start_transition().unwrap().kind, TransitionKind::DipToBlack);
    assert_eq!(clip.end_transition().unwrap().kind, TransitionKind::CrossFade);

    // Remove In Transition independently
    clip.transition_in = None;
    assert!(clip.start_transition().is_none());
    assert_eq!(clip.end_transition().unwrap().kind, TransitionKind::CrossFade);
}

#[test]
fn test_export_filtergraph_single_clip_fade_in_and_fade_out() {
    use std::path::PathBuf;
    use video_editor::core::clip::Clip;
    use video_editor::core::time::TimeCode;
    use video_editor::core::timeline::Timeline;
    use video_editor::core::transition::{Transition, TransitionKind};
    use video_editor::export::filter_graph::{build_ffmpeg_export_command, ExportConfig};

    let mut timeline = Timeline::new(30.0);
    let track_id = timeline.tracks[0].id;
    let mut clip = Clip::new(
        1,
        track_id,
        "Intro".to_string(),
        PathBuf::from("intro.mp4"),
        TimeCode::from_secs_f64(6.0),
        true,
        false,
    );
    clip.transition_in = Some(Transition {
        kind: TransitionKind::DipToBlack,
        duration_secs: 1.0,
    });
    clip.transition_out = Some(Transition {
        kind: TransitionKind::DipToWhite,
        duration_secs: 1.5,
    });
    timeline.tracks[0].add_clip(clip);

    let config = ExportConfig {
        output_path: PathBuf::from("output.mp4"),
        width: 1280,
        height: 720,
        fps: 30.0,
        ..ExportConfig::default()
    };

    let cmd = build_ffmpeg_export_command(&timeline, &config).expect("build command");
    let fc_idx = cmd.iter().position(|a| a == "-filter_complex").unwrap();
    let fc = &cmd[fc_idx + 1];

    assert!(
        fc.contains("fade=t=in:st=0:d=1.000:color=black"),
        "Filter complex must contain leading fade in: {}",
        fc
    );
    assert!(
        fc.contains("fade=t=out:st=4.500:d=1.500:color=white"),
        "Filter complex must contain trailing fade out: {}",
        fc
    );
}

#[test]
fn test_export_filtergraph_multi_clip_in_and_out_xfade() {
    use std::path::PathBuf;
    use video_editor::core::clip::Clip;
    use video_editor::core::time::TimeCode;
    use video_editor::core::timeline::Timeline;
    use video_editor::core::transition::{Transition, TransitionKind};
    use video_editor::export::filter_graph::{build_ffmpeg_export_command, ExportConfig};

    let mut timeline = Timeline::new(30.0);
    let track_id = timeline.tracks[0].id;

    let mut clip1 = Clip::new(
        1,
        track_id,
        "Clip1".to_string(),
        PathBuf::from("c1.mp4"),
        TimeCode::from_secs_f64(4.0),
        true,
        false,
    );
    clip1.transition_in = Some(Transition {
        kind: TransitionKind::DipToBlack,
        duration_secs: 0.5,
    });

    let mut clip2 = Clip::new(
        2,
        track_id,
        "Clip2".to_string(),
        PathBuf::from("c2.mp4"),
        TimeCode::from_secs_f64(4.0),
        true,
        false,
    );
    clip2.timeline_start = TimeCode::from_secs_f64(4.0);
    clip2.transition_in = Some(Transition {
        kind: TransitionKind::WipeLeft,
        duration_secs: 1.0,
    });
    clip2.transition_out = Some(Transition {
        kind: TransitionKind::DipToBlack,
        duration_secs: 0.5,
    });

    timeline.tracks[0].add_clip(clip1);
    timeline.tracks[0].add_clip(clip2);

    let config = ExportConfig {
        output_path: PathBuf::from("output.mp4"),
        width: 1280,
        height: 720,
        fps: 30.0,
        ..ExportConfig::default()
    };

    let cmd = build_ffmpeg_export_command(&timeline, &config).expect("build command");
    let fc_idx = cmd.iter().position(|a| a == "-filter_complex").unwrap();
    let fc = &cmd[fc_idx + 1];

    assert!(fc.contains("fade=t=in:st=0:d=0.500:color=black"));
    assert!(fc.contains("xfade=transition=wipeleft:duration=1.000"));
    assert!(fc.contains("fade=t=out:"));
}

#[test]
fn test_text_overlay_model_and_card_creation() {
    use egui::Color32;
    use video_editor::core::clip::Clip;
    use video_editor::core::text_overlay::{
        FontFamilyPreset, TextAlignment, TextBoxStyle, TextOverlay, TitleCardBackground,
    };

    let mut overlay = TextOverlay::new("Our Hawaii Trip 2026
Summer Memories");
    overlay.font_family = FontFamilyPreset::Serif;
    overlay.alignment = TextAlignment::Center;
    overlay.is_bold = true;
    overlay.is_italic = true;
    overlay.font_size = 48.0;
    overlay.box_style = TextBoxStyle::TranslucentBox;
    overlay.box_opacity = 0.7;
    overlay.x = 0.25;
    overlay.y = 0.75;

    assert_eq!(overlay.text, "Our Hawaii Trip 2026
Summer Memories");
    assert_eq!(overlay.font_family, FontFamilyPreset::Serif);
    assert_eq!(overlay.alignment, TextAlignment::Center);
    assert_eq!(overlay.x, 0.25);
    assert_eq!(overlay.y, 0.75);
    assert!(overlay.is_bold);
    assert!(overlay.is_italic);

    let solid_bg = TitleCardBackground::SolidColor(Color32::from_rgb(15, 30, 60));
    let card = Clip::new_title_card(
        100,
        1,
        "Hawaii 2026".to_string(),
        overlay.clone(),
        solid_bg,
        4.0,
    );

    assert_eq!(card.duration().as_secs_f64(), 4.0);
    // A title card is now a static slide: background holds the colour, the text rides in elements.
    assert!(!card.has_video);
    assert!(!card.has_audio);
    assert!(!card.is_title_card);
    assert_eq!(
        card.background,
        Some(video_editor::core::text_overlay::SlideBackground::Solid(
            Color32::from_rgb(15, 30, 60)
        ))
    );
    assert_eq!(card.elements.len(), 1);
    match &card.elements[0] {
        video_editor::core::text_overlay::SlideElement::Text(o) => {
            assert_eq!(o.text, "Our Hawaii Trip 2026
Summer Memories")
        }
        _ => panic!("expected a text element on the card"),
    }
}
#[test]
fn test_generate_title_card_solid_color_frame() {
    use egui::Color32;
    use video_editor::core::text_overlay::TitleCardBackground;
    use video_editor::media::{generate_solid_color_frame, generate_title_card_frame};

    let color = Color32::from_rgb(20, 40, 80);
    let frame = generate_solid_color_frame(color, 320, 180);
    assert_eq!(frame.size, [320, 180]);
    assert_eq!(frame.pixels.len(), 320 * 180);
    assert_eq!(frame.pixels[0], color);
    assert_eq!(frame.pixels[frame.pixels.len() - 1], color);

    let bg = TitleCardBackground::SolidColor(color);
    let frame2 = generate_title_card_frame(&bg, 320, 180);
    assert_eq!(frame2.pixels[0], color);
}

#[test]
fn test_export_filtergraph_title_card_with_drawtext() {
    use egui::Color32;
    use std::path::PathBuf;
    use video_editor::core::clip::Clip;
    use video_editor::core::text_overlay::{
        FontFamilyPreset, TextAlignment, TextBoxStyle, TextOverlay, TitleCardBackground,
    };
    use video_editor::core::timeline::Timeline;
    use video_editor::export::filter_graph::{build_ffmpeg_export_command, ExportConfig};

    let mut timeline = Timeline::new(30.0);
    let track_id = timeline.tracks[0].id;

    let mut overlay = TextOverlay::new("Welcome to Hawaii\nTrip of a Lifetime");
    overlay.font_family = FontFamilyPreset::SansSerif;
    overlay.alignment = TextAlignment::Center;
    overlay.box_style = TextBoxStyle::TranslucentBox;
    overlay.box_opacity = 0.65;

    let bg = TitleCardBackground::SolidColor(Color32::from_rgb(18, 24, 36));
    let intro_card = Clip::new_title_card(
        1,
        track_id,
        "Welcome Card".to_string(),
        overlay,
        bg,
        4.0,
    );
    timeline.tracks[0].add_clip(intro_card);

    let config = ExportConfig {
        output_path: PathBuf::from("slideshow.mp4"),
        width: 1280,
        height: 720,
        fps: 30.0,
        ..ExportConfig::default()
    };

    let cmd = build_ffmpeg_export_command(&timeline, &config).expect("export command");
    let fc_idx = cmd.iter().position(|a| a == "-filter_complex").unwrap();
    let fc = &cmd[fc_idx + 1];

    assert!(
        fc.contains("color=c="),
        "Must use color generator for title card: {}",
        fc
    );
    assert!(
        fc.contains("drawtext=text='Welcome to Hawaii'"),
        "Must have drawtext line 1: {}",
        fc
    );
    assert!(
        fc.contains("drawtext=text='Trip of a Lifetime'"),
        "Must have drawtext line 2: {}",
        fc
    );
}

#[test]
fn test_blank_slide_and_element_bounds() {
    use std::path::PathBuf;
    use video_editor::core::clip::Clip;
    use video_editor::core::text_overlay::{SlideBackground, SlideElement};

    let mut slide = Clip::new_blank_slide(1, 1, "Blank".to_string(), 3.0);
    assert_eq!(slide.duration().as_secs_f64(), 3.0);
    assert!(slide.is_static_slide());
    assert!(!slide.has_video);
    assert!(!slide.has_audio);
    assert_eq!(
        slide.background,
        Some(SlideBackground::Solid(egui::Color32::from_rgb(18, 18, 24)))
    );

    slide.elements.push(SlideElement::Picture {
        path: PathBuf::from("x.png"),
        x: 0.1,
        y: 0.2,
        w: 0.5,
        h: 0.4,
    });
    let el = slide.elements.last_mut().unwrap();
    el.set_bounds(0.0, 0.0, 1.0, 1.0);
    assert_eq!(el.bounds(), (0.0, 0.0, 1.0, 1.0));

    let audio = SlideElement::Audio {
        path: PathBuf::from("a.mp3"),
        volume: 0.5,
    };
    assert!(!audio.is_visual());
}

#[test]
fn test_legacy_clip_migration_on_load() {
    use video_editor::core::clip::Clip;
    use video_editor::core::project::Project;
    use video_editor::core::text_overlay::{
        SlideBackground, TextOverlay, TitleCardBackground,
    };
    use video_editor::core::time::TimeCode;

    let mut project = Project::new("Migrate".to_string());
    let mut clip = Clip::new(
        1,
        0,
        "Old Title".to_string(),
        std::path::PathBuf::new(),
        TimeCode::from_secs_f64(4.0),
        false,
        false,
    );
    clip.is_title_card = true;
    clip.title_card_bg = Some(TitleCardBackground::SolidColor(egui::Color32::BLUE));
    clip.text_overlay = Some(TextOverlay::new("Hi"));
    project.timeline.tracks[0].add_clip(clip);

    let dir = std::env::temp_dir().join(format!("ve_mig_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("p.vproj");
    project.save_to_file(&path).unwrap();

    let loaded = Project::load_from_file(&path).unwrap();
    let c = &loaded.timeline.tracks[0].clips[0];
    assert!(!c.is_title_card);
    assert!(!c.has_video);
    assert_eq!(c.background, Some(SlideBackground::Solid(egui::Color32::BLUE)));
    assert_eq!(c.elements.len(), 1);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_action_row_card_respects_parent_clip_rect() {
    use video_editor::ui::components::ActionRowCard;

    let ctx = egui::Context::default();
    let mut captured_clip_rects = Vec::new();

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let scroll_viewport = egui::Rect::from_min_max(
                egui::Pos2::new(0.0, 100.0),
                egui::Pos2::new(300.0, 300.0),
            );
            ui.set_clip_rect(scroll_viewport);

            // Allocate a child ui with that clip rect
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(scroll_viewport), |ui| {
                ActionRowCard::render(ui, "🎬", "Wipe Left", "New clip reveals by wiping in", false, 260.0);
                captured_clip_rects.push(ui.clip_rect());
            });
        });
    });

    for clip in captured_clip_rects {
        assert!(
            clip.min.y >= 100.0,
            "Clip rect ({:?}) must never expand above scroll_viewport min.y (100.0)",
            clip
        );
    }
}
