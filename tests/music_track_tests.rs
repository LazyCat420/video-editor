use std::path::{Path, PathBuf};
use video_editor::core::time::TimeCode;
use video_editor::core::timeline::Timeline;
use video_editor::core::track::TrackKind;

#[test]
fn append_two_songs_second_starts_at_end_of_first() {
    let mut tl = Timeline::new(30.0);
    tl.append_music_clip(
        "a.mp3".to_string(),
        PathBuf::from("a.mp3"),
        TimeCode::from_secs_f64(120.0),
    );
    tl.append_music_clip(
        "b.mp3".to_string(),
        PathBuf::from("b.mp3"),
        TimeCode::from_secs_f64(30.5),
    );
    let clips = tl.music_clips();
    assert_eq!(clips.len(), 2);
    assert_eq!(clips[0].timeline_start, TimeCode::ZERO);
    assert_eq!(clips[1].timeline_start, TimeCode::from_secs_f64(120.0));
    assert_eq!(clips[1].timeline_end(), TimeCode::from_secs_f64(150.5));
}

#[test]
fn append_creates_music_track_if_missing() {
    let mut tl = Timeline::new(30.0);
    let audio_ids: Vec<u64> = tl
        .tracks
        .iter()
        .filter(|t| t.kind == TrackKind::Audio)
        .map(|t| t.id)
        .collect();
    for id in audio_ids {
        tl.remove_track(id);
    }
    assert!(tl.tracks.iter().all(|t| t.kind != TrackKind::Audio));

    tl.append_music_clip(
        "a.mp3".to_string(),
        PathBuf::from("a.mp3"),
        TimeCode::from_secs_f64(10.0),
    );
    let track = tl
        .tracks
        .iter()
        .find(|t| t.kind == TrackKind::Audio)
        .expect("music track re-created");
    assert_eq!(track.name, "🎵 Music & Sound");
    assert_eq!(tl.music_clips().len(), 1);
}

#[test]
fn remove_middle_song_reflows_contiguously_from_zero() {
    let mut tl = Timeline::new(30.0);
    tl.append_music_clip("a".into(), PathBuf::from("a.mp3"), TimeCode::from_secs_f64(10.0));
    let middle = tl.append_music_clip("b".into(), PathBuf::from("b.mp3"), TimeCode::from_secs_f64(20.0));
    tl.append_music_clip("c".into(), PathBuf::from("c.mp3"), TimeCode::from_secs_f64(30.0));

    assert!(tl.remove_music_clip(middle));

    let clips = tl.music_clips();
    assert_eq!(clips.len(), 2);
    assert_eq!(clips[0].timeline_start, TimeCode::ZERO);
    assert_eq!(clips[0].duration(), TimeCode::from_secs_f64(10.0));
    assert_eq!(clips[1].timeline_start, TimeCode::from_secs_f64(10.0));
    assert_eq!(clips[1].timeline_end(), TimeCode::from_secs_f64(40.0));
}

#[test]
fn remove_nonexistent_music_clip_is_noop() {
    let mut tl = Timeline::new(30.0);
    tl.append_music_clip("a".into(), PathBuf::from("a.mp3"), TimeCode::from_secs_f64(10.0));
    let before = tl.music_clips().to_vec();
    assert!(!tl.remove_music_clip(999_999));
    assert_eq!(tl.music_clips(), before.as_slice());
}

#[test]
fn source_offset_math() {
    use video_editor::audio::music_engine::music_source_offset;
    // Song starts at 60s on the timeline, trimmed in by 10s; playhead at 75s
    // → decode from 25s into the file.
    let off = music_source_offset(
        TimeCode::from_secs_f64(60.0),
        TimeCode::from_secs_f64(10.0),
        TimeCode::from_secs_f64(75.0),
    );
    assert_eq!(off, std::time::Duration::from_secs(25));
    // A playhead before the clip clamps to source_in, never goes negative.
    let off = music_source_offset(
        TimeCode::from_secs_f64(60.0),
        TimeCode::ZERO,
        TimeCode::from_secs_f64(59.0),
    );
    assert_eq!(off, std::time::Duration::ZERO);
}

#[test]
fn audio_routing_is_extension_based() {
    use video_editor::media::probe::is_audio_path;
    assert!(is_audio_path(Path::new("song.mp3")));
    assert!(is_audio_path(Path::new("SONG.MP3")));
    assert!(is_audio_path(Path::new("Song.Mp3")));
    assert!(is_audio_path(Path::new("track.m4a")));
    assert!(is_audio_path(Path::new("track.flac")));
    assert!(!is_audio_path(Path::new("movie.mp4")));
    assert!(!is_audio_path(Path::new("photo.png")));
    assert!(!is_audio_path(Path::new("noext")));
}

#[test]
fn add_media_to_bin_uses_probe_cache_without_ffprobe() {
    use video_editor::media::probe::MediaMetadata;
    use video_editor::VideoEditorApp;

    let mut app = VideoEditorApp::default();
    // The path does not exist, so a real ffprobe would fail: success proves
    // the cached result was used instead of a fresh probe.
    let fake = PathBuf::from("Z:\\definitely\\missing\\song.mp3");
    app.probe_cache.insert(
        fake.clone(),
        Ok(MediaMetadata {
            duration_secs: 33.0,
            has_audio: true,
            ..Default::default()
        }),
    );
    app.add_music_files(vec![fake.clone()], None);

    let music = app.project.timeline.music_clips();
    assert_eq!(music.len(), 1, "cached probe must be honored");
    assert_eq!(music[0].duration(), TimeCode::from_secs_f64(33.0));
    assert_eq!(music[0].source_path, fake);
}

#[test]
fn import_queue_probes_in_background_then_applies_batch() {
    use video_editor::VideoEditorApp;

    let dir = std::env::temp_dir().join(format!("ve_import_queue_test_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mp3 = dir.join("song.mp3");
    let ok = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=1",
            mp3.to_str().unwrap(),
        ])
        .status()
        .unwrap()
        .success();
    assert!(ok, "failed to make test mp3");

    let mut app = VideoEditorApp::default();
    app.drop_files_on_canvas(vec![mp3.clone()], 0.5, 0.5, None);

    // The call must return without applying anything: probing is in flight.
    assert!(app.pending_import.is_some(), "unprobed files must queue");
    assert_eq!(app.project.timeline.music_clips().len(), 0);

    // Pump like the update loop does, until the worker finishes (bounded wait).
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while app.pending_import.is_some() {
        assert!(
            std::time::Instant::now() < deadline,
            "import worker never finished"
        );
        app.pump_import_queue(None);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    let music = app.project.timeline.music_clips();
    assert_eq!(music.len(), 1, "batch must apply after probing completes");
    assert_eq!(music[0].source_path, mp3);
    assert!(app.probe_cache.is_empty(), "cache must be drained after apply");

    let _ = std::fs::remove_dir_all(dir);
}
