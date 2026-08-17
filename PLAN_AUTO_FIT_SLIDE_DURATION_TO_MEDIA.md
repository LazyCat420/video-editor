# Implementation Plan: Auto-Fit Blank Slide Duration to Longest Media Clip

**Repository:** `video-editor` (`LazyCat420/video-editor`)  
**Status:** Plan for review — **awaiting user approval before implementation.**

---

## 1. Feature Overview & Design

### Goal
Make blank slides smart enough to automatically end at the exact duration of the longest media clip (video or audio) added to the slide, rather than remaining stuck at the default 3-second duration.

### Desired Behavior
1. **Dynamic Expansion on Drop / Addition**:
   - When a video or audio clip is dragged and dropped (or added via inspector) onto a blank slide, compute `max_duration` across all media elements on that slide.
   - Automatically update the slide duration (`clip.source_duration`, `clip.source_out`, and `clip.volume_envelope`) to match `max_duration`.
2. **Timeline Ripple Safety**:
   - If the slide duration expands (e.g. from 3.0s to 18.5s), automatically shift any downstream clips on the same timeline track forward by `delta` so existing clips are never overlapped or cut off.
3. **Element Removal Adaptation**:
   - When a media element is removed from the slide, recalculate the duration based on the remaining longest media element (or retain standard default 3.0s if only text/pictures remain).

---

## 2. Proposed Changes

### 1. `src/app.rs`
- Add helper method `auto_adjust_slide_duration_to_media(&mut self, slide_id: u64)`:
  - Scans all `SlideElement::Video` and `SlideElement::Audio` elements in `clip.elements`.
  - Checks asset duration in `project.media_assets` or probes the media file via `crate::media::probe::probe_media_file`.
  - Sets `clip.source_duration = new_duration`, `clip.source_out = new_duration`, `clip.volume_envelope = VolumeEnvelope::default_for_duration(new_duration)`.
  - Shifts subsequent clips on the same track if duration increased.
- Call `auto_adjust_slide_duration_to_media` in:
  - `drop_media_asset_on_canvas`
  - `drop_files_on_canvas`
  - `place_pending_element`
  - `delete_slide_element`
  - `SlideBinAction::AddAudioElement`
  - `SlideBinAction::RemoveElement`

---

## 3. Verification Plan

### Automated Tests
- In `tests/slide_dnd_tests.rs`:
  - `test_blank_slide_auto_fits_duration_to_longest_video`:
    1. Create a 3.0s blank slide.
    2. Drop a 10.0s video onto the slide.
    3. Assert slide duration is now 10.0s.
    4. Drop a 15.0s audio track onto the slide.
    5. Assert slide duration is now 15.0s.
    6. Remove the 15.0s audio track.
    7. Assert slide duration adapts back to 10.0s.
  - `test_blank_slide_expansion_shifts_subsequent_clips`:
    1. Place a blank slide at 00:00 (3s) and a video clip at 00:03.
    2. Drop a 12s video onto the slide.
    3. Assert the blank slide is 12s and the subsequent clip shifted to 00:12.
- Run `cargo test`.

### Manual Verification
1. Run `cargo run --release`.
2. Add a blank slide to the timeline.
3. Drag a 15-second video onto the slide canvas.
4. Verify the slide block on the timeline expands to 15 seconds.
5. Hit **PLAY** and verify the slide plays for the full duration of the video.
