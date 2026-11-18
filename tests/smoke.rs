//! Smoke test: the end-to-end project lifecycle the app performs on every
//! edit session — build a small project in memory (asset + video track clips
//! + music bed), save it through the real JSON serializer, load it back,
//! and drive the timeline model (duration maths, boundary hit-tests, and a
//! playhead split) — asserting concrete final state at each step. This is
//! the headless layer under the egui UI: the UI needs a window/GPU and is
//! out of scope here.

use std::path::PathBuf;
use video_editor::core::clip::Clip;
use video_editor::core::project::{MediaAsset, Project};
use video_editor::core::time::TimeCode;

fn sec(s: f64) -> TimeCode {
    TimeCode::from_secs_f64(s)
}

fn tiny_project() -> (Project, u64) {
    let mut p = Project::new("smoke".to_string());
    let asset = MediaAsset {
        id: p.next_asset_id(),
        name: "clip.mp4".to_string(),
        path: PathBuf::from("clip.mp4"),
        duration_secs: 30.0,
        width: 640,
        height: 360,
        fps: 30.0,
        has_video: true,
        has_audio: true,
        proxy_path: None,
        peak_path: None,
    };
    p.add_asset(asset);

    let video_track = p.timeline.tracks[0].id;
    let mut start = TimeCode::ZERO;
    for (name, dur) in [("a", 10.0), ("b", 5.0), ("c", 7.0)] {
        let id = p.timeline.next_id();
        let mut clip = Clip::new(
            id,
            video_track,
            name.to_string(),
            PathBuf::from("clip.mp4"),
            sec(dur),
            true,
            true,
        );
        clip.timeline_start = start;
        start = start + dur_to_tc(dur);
        p.timeline
            .get_track_mut(video_track)
            .unwrap()
            .add_clip(clip);
    }
    p.timeline
        .append_music_clip("song.mp3".to_string(), PathBuf::from("song.mp3"), sec(12.0));
    (p, video_track)
}

/// `TimeCode` addition through f64 (no Add impl on TimeCode today).
fn dur_to_tc(s: f64) -> TimeCode {
    TimeCode::from_micros((s * 1_000_000.0).round() as i64)
}

#[test]
fn save_load_then_playhead_split_end_to_end() {
    let (project, video_track) = tiny_project();

    // Timeline maths on the constructed state.
    assert_eq!(
        project.timeline.duration(),
        sec(22.0),
        "video 10+5+7 s must dominate the 12 s music bed"
    );
    let vt = project.timeline.get_track(video_track).unwrap();
    assert_eq!(vt.clips.len(), 3);
    assert_eq!(
        vt.get_clip_at(sec(12.5)).map(|c| c.name.as_str()),
        Some("b")
    );
    assert_eq!(
        vt.get_clip_at(sec(22.5)),
        None,
        "past the end must hit nothing"
    );
    assert_eq!(project.timeline.music_clips().len(), 1);

    // Real serializer round trip: save, load back, structural equality.
    let dir = std::env::temp_dir().join(format!("ve-smoke-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("project.json");

    project.save_to_file(&file).expect("save");
    let loaded = Project::load_from_file(&file).expect("load");
    assert_eq!(loaded, project, "save/load must be lossless");

    // Edit, save again, reload: playhead split at 12.5 s inside clip "b".
    let mut p = loaded;
    p.timeline.playhead = sec(12.5);
    assert!(
        p.timeline.split_at_playhead(),
        "split inside clip b must apply"
    );
    let vt = p.timeline.get_track(video_track).unwrap();
    assert_eq!(vt.clips.len(), 4, "split must add exactly one clip");
    let mut starts: Vec<TimeCode> = vt.clips.iter().map(|c| c.timeline_start).collect();
    starts.sort();
    assert_eq!(
        starts,
        vec![sec(0.0), sec(10.0), sec(12.5), sec(15.0)],
        "split halves misplaced"
    );
    let total_after: f64 = vt.clips.iter().map(|c| c.duration().as_secs_f64()).sum();
    assert!(
        (total_after - 22.0).abs() < 1e-6,
        "split must preserve total duration: {total_after}"
    );

    p.save_to_file(&file).expect("re-save");
    assert_eq!(Project::load_from_file(&file).expect("re-load"), p);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn empty_project_round_trips_and_has_the_default_rig() {
    let p = Project::default();
    assert_eq!(
        p.timeline.tracks.len(),
        2,
        "default rig: 1 video + 1 music track"
    );
    assert_eq!(p.timeline.duration(), TimeCode::ZERO);
    let dir = std::env::temp_dir().join(format!("ve-smoke-empty-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("p.json");
    p.save_to_file(&file).unwrap();
    assert_eq!(Project::load_from_file(&file).unwrap(), p);
    let _ = std::fs::remove_dir_all(&dir);
}
