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
