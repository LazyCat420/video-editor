//! Reproduces the "play only renders the selected slide" defect.
//!
//! With several slides on the timeline, pressing Play should walk the deck from
//! the playhead forward. Today the preview is driven by `active_slide()`, which
//! prefers whichever clip carries `is_selected` over the clip under the playhead,
//! so playback visually pins to the selected slide.

use video_editor::core::clip::Clip;
use video_editor::core::time::TimeCode;
use video_editor::core::track::TrackKind;
use video_editor::VideoEditorApp;

/// Build an app with three 5s blank slides laid end to end on the video track.
fn app_with_three_slides() -> (VideoEditorApp, Vec<u64>) {
    let mut app = VideoEditorApp::default();
    let track_id = app
        .project
        .timeline
        .tracks
        .iter()
        .find(|t| t.kind == TrackKind::Video)
        .map(|t| t.id)
        .expect("default timeline has a video track");

    let mut ids = Vec::new();
    for i in 0..3 {
        let id = app.project.timeline.next_id();
        let mut clip = Clip::new_blank_slide(id, track_id, format!("Slide {}", i + 1), 5.0);
        clip.timeline_start = TimeCode::from_secs_f64(i as f64 * 5.0);
        clip.is_selected = false;
        app.project
            .timeline
            .get_track_mut(track_id)
            .unwrap()
            .add_clip(clip);
        ids.push(id);
    }
    (app, ids)
}

#[test]
fn playhead_picks_the_slide_it_is_over_when_nothing_is_selected() {
    let (mut app, ids) = app_with_three_slides();

    app.project.timeline.playhead = TimeCode::from_secs_f64(1.0);
    assert_eq!(app.active_slide().map(|c| c.id), Some(ids[0]));

    app.project.timeline.playhead = TimeCode::from_secs_f64(6.0);
    assert_eq!(app.active_slide().map(|c| c.id), Some(ids[1]));

    app.project.timeline.playhead = TimeCode::from_secs_f64(11.0);
    assert_eq!(app.active_slide().map(|c| c.id), Some(ids[2]));
}

#[test]
fn playback_follows_the_playhead_not_the_selection() {
    let (mut app, ids) = app_with_three_slides();

    // The user clicked slide 3 at some point, then rewound to zero and hit Play.
    app.project.timeline.select_clip(ids[2]);
    app.project.timeline.playhead = TimeCode::ZERO;
    app.project.timeline.is_playing = true;

    // Frame 1 of playback: the playhead is over slide 1, so slide 1 must render.
    assert_eq!(
        app.active_slide().map(|c| c.id),
        Some(ids[0]),
        "while playing, the preview must follow the playhead, not the selected slide"
    );

    // ...and it must advance as the playhead crosses each slide boundary.
    app.project.timeline.playhead = TimeCode::from_secs_f64(7.5);
    assert_eq!(
        app.active_slide().map(|c| c.id),
        Some(ids[1]),
        "playback must advance to slide 2"
    );

    app.project.timeline.playhead = TimeCode::from_secs_f64(12.5);
    assert_eq!(
        app.active_slide().map(|c| c.id),
        Some(ids[2]),
        "playback must advance to slide 3"
    );
}

#[test]
fn timeline_duration_spans_every_slide() {
    let (app, _) = app_with_three_slides();
    assert_eq!(
        app.project.timeline.duration(),
        TimeCode::from_secs_f64(15.0),
        "three 5s slides must give a 15s timeline so playback does not stop early"
    );
}

#[test]
fn selection_still_drives_the_preview_while_paused() {
    let (mut app, ids) = app_with_three_slides();

    // Paused editing: clicking slide 3 in the deck should preview slide 3
    // even though the playhead sits at zero.
    app.project.timeline.select_clip(ids[2]);
    app.project.timeline.playhead = TimeCode::ZERO;
    app.project.timeline.is_playing = false;

    assert_eq!(
        app.active_slide().map(|c| c.id),
        Some(ids[2]),
        "while paused, clicking a slide must preview that slide"
    );
}
