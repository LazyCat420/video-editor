//! Playback / slide-deck sequencing tests.
//!
//! Originally written to reproduce the "play only renders the selected slide"
//! defect: the preview was driven by the selection-first `active_slide()`, so
//! playback visually pinned to whichever slide the user last clicked. The
//! render paths now go through `slide_to_render()` (playhead-first while
//! playing, selection-first while paused) — verified against the real call
//! sites in `src/app/mod.rs` and `src/app/playback.rs`.

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

/// Starts (in secs) of every video-track clip, in track order.
fn slide_starts(app: &VideoEditorApp) -> Vec<f64> {
    app.project
        .timeline
        .tracks
        .iter()
        .filter(|t| t.kind == TrackKind::Video)
        .flat_map(|t| t.clips.iter())
        .map(|c| c.timeline_start.as_secs_f64())
        .collect()
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
        app.slide_to_render().map(|c| c.id),
        Some(ids[0]),
        "while playing, the preview must follow the playhead, not the selected slide"
    );

    // ...and it must advance as the playhead crosses each slide boundary.
    app.project.timeline.playhead = TimeCode::from_secs_f64(7.5);
    assert_eq!(app.slide_to_render().map(|c| c.id), Some(ids[1]));

    app.project.timeline.playhead = TimeCode::from_secs_f64(12.5);
    assert_eq!(app.slide_to_render().map(|c| c.id), Some(ids[2]));
}

#[test]
fn playback_walks_every_slide_in_order_with_no_backtracking() {
    let (mut app, ids) = app_with_three_slides();
    app.project.timeline.select_clip(ids[2]); // adversarial selection
    app.project.timeline.is_playing = true;

    let mut seen: Vec<u64> = Vec::new();
    let mut t = 0.0;
    while t < 15.0 {
        app.project.timeline.playhead = TimeCode::from_secs_f64(t);
        let id = app
            .slide_to_render()
            .map(|c| c.id)
            .expect("a contiguous deck always has a slide under the playhead");
        if seen.last() != Some(&id) {
            seen.push(id);
        }
        t += 0.5;
    }
    assert_eq!(
        seen, ids,
        "playback must visit slide 1, 2, 3 exactly once each, in order"
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
        app.slide_to_render().map(|c| c.id),
        Some(ids[2]),
        "while paused, clicking a slide must preview that slide"
    );
}

#[test]
fn selection_follows_the_playhead_while_playing_only() {
    let (mut app, ids) = app_with_three_slides();
    app.project.timeline.select_clip(ids[2]);

    // Paused: sync must NOT move the selection out from under the user.
    app.project.timeline.playhead = TimeCode::from_secs_f64(1.0);
    app.project.timeline.is_playing = false;
    app.sync_selection_to_playhead();
    assert_eq!(
        app.project.timeline.get_selected_clip().map(|c| c.id),
        Some(ids[2])
    );

    // Playing: the deck highlight follows the slide being played.
    app.project.timeline.is_playing = true;
    app.sync_selection_to_playhead();
    assert_eq!(
        app.project.timeline.get_selected_clip().map(|c| c.id),
        Some(ids[0])
    );

    app.project.timeline.playhead = TimeCode::from_secs_f64(7.5);
    app.sync_selection_to_playhead();
    assert_eq!(
        app.project.timeline.get_selected_clip().map(|c| c.id),
        Some(ids[1])
    );
}

#[test]
fn rewind_then_play_renders_slide_one() {
    let (mut app, ids) = app_with_three_slides();
    let ctx = egui::Context::default();

    // Watch to the middle of slide 3, stop (rewinds to zero), play again.
    app.project.timeline.select_clip(ids[2]);
    app.project.timeline.playhead = TimeCode::from_secs_f64(12.0);
    app.start_playback(&ctx);
    app.stop_playback(&ctx);
    assert_eq!(app.project.timeline.playhead, TimeCode::ZERO);

    app.start_playback(&ctx);
    assert!(app.project.timeline.is_playing);
    assert_eq!(
        app.slide_to_render().map(|c| c.id),
        Some(ids[0]),
        "after Rewind + Play the first rendered slide must be slide 1"
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
fn inserting_a_slide_never_overlaps_existing_slides() {
    let mut app = VideoEditorApp::default();

    // Three inserts with the playhead parked at ZERO every time — the exact
    // state after hitting Rewind. Without the insert-path reflow this stacked
    // every new slide on top of slide 1 at t=0.
    for _ in 0..3 {
        app.project.timeline.playhead = TimeCode::ZERO;
        app.insert_blank_slide_at_playhead(5.0, None);
    }

    assert_eq!(slide_starts(&app), vec![0.0, 5.0, 10.0], "slides must be packed end to end");
    assert_eq!(
        app.project.timeline.duration(),
        TimeCode::from_secs_f64(15.0),
        "three 5s slides must yield a 15s show"
    );
}

#[test]
fn inserting_mid_deck_lands_after_the_playhead_slide() {
    let (mut app, _) = app_with_three_slides();

    // Playhead inside slide 1 → the new slide slots in after it, and the deck
    // stays contiguous.
    app.project.timeline.playhead = TimeCode::from_secs_f64(2.0);
    app.insert_blank_slide_at_playhead(5.0, None);

    assert_eq!(slide_starts(&app), vec![0.0, 5.0, 10.0, 15.0]);
    assert_eq!(app.project.timeline.duration(), TimeCode::from_secs_f64(20.0));
}

#[test]
fn pause_and_seek_discard_slide_video_decoders() {
    let (mut app, _) = app_with_three_slides();
    let ctx = egui::Context::default();

    // A stopped-but-kept decoder entry is exactly the state that used to make
    // rewound slide videos never restart — both paths must DROP entries.
    app.slide_video_players.insert(
        std::path::PathBuf::from("clip.mp4"),
        video_editor::media::stream_player::StreamVideoPlayer::new(),
    );
    app.pause_playback();
    assert!(
        app.slide_video_players.is_empty(),
        "pause must discard slide decoders so the next play re-seeks"
    );

    app.slide_video_players.insert(
        std::path::PathBuf::from("clip.mp4"),
        video_editor::media::stream_player::StreamVideoPlayer::new(),
    );
    app.seek_to(TimeCode::from_secs_f64(1.0), &ctx);
    assert!(
        app.slide_video_players.is_empty(),
        "seek must discard slide decoders so they restart at the new position"
    );
}
