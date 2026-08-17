# Implementation Plan: Slide Video Playback, Timeline Media Visuals & Media Bin Reorder Fix

**Repository:** `video-editor` (`LazyCat420/video-editor`)  
**Status:** Plan for review — **not to be implemented until approved.**

---

## 1. Root Cause Summary

1. **Embedded Video Playback on Slide**:
   - `get_active_video_clip_info` bypassed slides (`clip.has_video == false`), stopping the `stream_player`.
   - `slide_visuals` hardcoded frame fetch at `0.0` and cached a static texture, never updating during playback.
2. **Timeline Visuals for Slides**:
   - The timeline rendered a plain rectangle for the slide without showing what videos, audios, pictures, or text layers are attached.
3. **Sidebar Media Reordering**:
   - `ReorderAsset { from_id, to_index }` had an off-by-one displacement when `from < to_index` after removing the item from the list.

---

## 2. Checklist of Everything To Do

- [ ] **1. Slide Video & Audio Playback Engine** ([`src/app.rs`](file:///home/lazycat/github/projects/sun/video-editor/src/app.rs))
  - Support video/audio element lookup in `get_active_video_clip_info` with local time offset `(playhead - slide.timeline_start)`.
  - Dynamically fetch frames at `(playhead - slide.timeline_start)` in `slide_visuals` and update textures on every frame during playback.
  - Enable continuous audio playback for embedded video and audio elements on the slide.

- [ ] **2. Timeline Media Element Badges on Blank Slides** ([`src/ui/timeline_view.rs`](file:///home/lazycat/github/projects/sun/video-editor/src/ui/timeline_view.rs))
  - Render mini layered element tags/badges on the slide timeline block:
    - `🎞 [Video Name]`
    - `🖼 [Picture Name]`
    - `✏️ [Text Preview]`
    - `🎵 [Audio Name]`
  - Display element count and layer indicators directly on the clip rectangle.

- [ ] **3. Sidebar Media Bin Drag-and-Drop Fix** ([`src/ui/media_bin.rs`](file:///home/lazycat/github/projects/sun/video-editor/src/ui/media_bin.rs) & [`src/app.rs`](file:///home/lazycat/github/projects/sun/video-editor/src/app.rs))
  - Fix index calculation when moving items down the list (`from < to_index`).
  - Ensure reordering updates `project.media_assets` and reflects immediately in the sidebar.

- [ ] **4. Testing & Verification**
  - Add automated tests in `tests/slide_dnd_tests.rs`.
  - Verify all 33+ tests pass via `cargo test`.
