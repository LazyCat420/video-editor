# Implementation Plan: Fix Multi-Video Slide Playback Duration & Independent Completion

**Repository:** `video-editor` (`LazyCat420/video-editor`)  
**Status:** Plan for review — **awaiting user approval before implementation.**

---

## 1. Problem Statement & Root Cause

### Symptom
- When a blank slide contains multiple videos of different durations (e.g. Video A is 5s, Video B is 15s), playback stops prematurely when the shorter video ends (at 5s) instead of continuing until the longest video finishes (at 15s).

### Root Cause
1. **First-Element Selection in `get_active_video_clip_info`**:
   - In `src/app.rs`, when scanning `clip.elements` on a static slide, `get_active_video_clip_info` returned on the **first** video/audio element found in the vector.
   - If the first element was the shorter video (5s), the stream player attached to that 5s file and signaled EOF/stopped at 5s, halting playback for the entire slide.
2. **Missing Per-Element Duration Clamping in `slide_visuals`**:
   - In `slide_visuals`, `fetch_frame` was requested with raw `slide_elapsed` for every video without clamping to each individual video's duration.
   - When `slide_elapsed` surpassed the shorter video's duration, ffmpeg failed to seek past EOF, causing frame fetch failures.

---

## 2. Proposed Changes

### `src/app.rs`
1. **Select Longest Media Element in `get_active_video_clip_info`**:
   - When resolving active video/audio for a static slide, inspect all `SlideElement::Video` and `SlideElement::Audio` elements.
   - Select the media element with the **longest duration**, ensuring `stream_player` stays active and plays sound/sync across the entire slide length.
2. **Independent Per-Video Completion & Clamping in `slide_visuals`**:
   - For each `SlideElement::Video { path, .. }`:
     - Query its duration `elem_dur` from `project.media_assets` or probe info.
     - If `slide_elapsed < elem_dur`: fetch real-time frame at `slide_elapsed`.
     - If `slide_elapsed >= elem_dur`: clamp frame time to `(elem_dur - 0.05).max(0.0)` (the video stops and remains cleanly frozen on its final frame while longer videos continue playing).

---

## 3. Verification Plan

### Automated Tests
- In `tests/slide_dnd_tests.rs`:
  - `test_multi_video_slide_picks_longest_video_for_stream`
  - `test_shorter_video_clamps_to_end_frame_when_expired`
- Run `cargo test`.

### Manual Verification
1. Run `cargo run --release`.
2. Add a blank slide.
3. Drag in a short video (e.g. 5s) and a long video (e.g. 15s).
4. Hit **PLAY** from the start of the slide.
5. Verify:
   - Both videos play simultaneously.
   - At 5s, the short video finishes and stops cleanly.
   - The long video keeps playing smoothly until 15s.
