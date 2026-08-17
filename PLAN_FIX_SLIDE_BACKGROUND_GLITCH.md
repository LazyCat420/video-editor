# Implementation Plan: Fix Background Glitch on Blank Slide Playback

**Repository:** `video-editor` (`LazyCat420/video-editor`)  
**Status:** Plan for review — **not to be implemented until approved.**

---

## 1. Problem Statement & Root Cause

### Symptom
- When playing a blank slide that has video elements, the next clip or full-screen video appears in the background canvas behind the slide elements.

### Root Cause
1. During playback, decoded stream frames from `stream_player` were being assigned directly to `self.current_frame` (which represents the full-screen canvas background).
2. For a blank slide, `self.current_frame` should remain the slide's solid color background, while the video element is drawn in its own bounded box via `slide_visuals`.
3. Near the end of the slide, lookahead pre-warming prepared the next clip in Deck B, causing `stream_player` frames from the next clip to be assigned to `self.current_frame`.

---

## 2. Proposed Changes

### `src/app.rs`
1. When consuming decoded frames in the playback loop:
   - If the active clip is a slide (`clip.is_static_slide()`), do not overwrite `self.current_frame` with the full-screen stream frame; maintain the slide's solid/image background.
   - If the active clip is a standard video, update `self.current_frame` normally.
2. In `refresh_preview_frame`:
   - Keep the slide background frame clean and separate from video element overlays.

---

## 3. Verification Plan

### Automated Tests
- Test in `tests/slide_dnd_tests.rs` for background isolation during slide playback.
- `cargo test`.

### Manual Verification
1. Run `cargo run --release`.
2. Play a blank slide with embedded videos next to another clip.
3. Confirm background remains solid with no video bleed or next-slide flicker.
