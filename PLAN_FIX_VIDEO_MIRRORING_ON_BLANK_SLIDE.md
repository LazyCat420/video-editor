# Implementation Plan: Fix Video Mirroring in Background on Blank Slides

**Repository:** `video-editor` (`LazyCat420/video-editor`)  
**Status:** Plan for review — **awaiting user approval before implementation.**

---

## 1. Problem Statement & Root Cause

### Symptom
- When dragging the first video into a blank slide (or playing it), the video appears both in its intended foreground bounding box AND duplicated/mirrored full-screen across the entire canvas background.

### Root Cause
1. **Background Overwrite from Stream Decoder**:
   - In `src/app.rs`, `self.current_frame` controls the base full-screen canvas background.
   - When a video was dragged onto a blank slide, the playback/stream decoder fed decoded video frames directly into `self.current_frame`, rendering the entire video full-screen in the background.
   - Concurrently, `slide_visuals` rendered the video element inside its smaller foreground draggable box, causing the video to appear mirrored/duplicated twice.
2. **Stale Texture Retention on Slide Creation**:
   - If `base_frame_for` returned `None` when `clip.background` was uninitialized or when switching to a slide, `PreviewPlayerView`'s texture cache retained the previous video frame, leaving the old video stuck behind the slide.

---

## 2. Proposed Changes

### `src/app.rs`
1. **Guaranteed Solid Background Generation for Slides**:
   - Update `base_frame_for`: For any static slide (`clip.is_static_slide()`), ensure it ALWAYS returns a clean solid color frame (e.g. RGB `18, 18, 24` or the user-selected background color/image), never `None`.
2. **Strict Background Isolation during Playback**:
   - In the streaming frame loop, explicitly guard `self.current_frame`:
     - If `clip.is_static_slide()`, **never** assign the raw streaming video frame to `self.current_frame`.
     - The streaming decoder will only be used for audio playback and feeding `slide_visuals` for the foreground box.
3. **Drop Handler Clean State Refresh**:
   - When dropping a video (`drop_media_asset_on_canvas` and `drop_files_on_canvas`), immediately ensure `refresh_preview_frame` loads the solid slide background so the background is clean before the first video frame renders.

---

## 3. Verification Plan

### Automated Tests
- Test in `tests/slide_dnd_tests.rs`:
  - Add test verifying that when a video element is added to a blank slide, `base_frame_for` produces the solid background frame and `is_static_slide()` prevents stream frames from polluting `current_frame`.
- Run full test suite: `cargo test`.

### Manual Verification
1. Run `cargo run --release`.
2. Click **➕ Add Blank Page** next to the zoom slider on the timeline.
3. Drag a video from the media bin (or file browser) onto the blank slide.
4. Verify:
   - The canvas background remains clean solid dark background (no full-screen video in background).
   - The video renders solely inside its resizable/movable box on the slide.
   - Hitting **PLAY** plays the video inside its box with sound, without mirroring to the background.
