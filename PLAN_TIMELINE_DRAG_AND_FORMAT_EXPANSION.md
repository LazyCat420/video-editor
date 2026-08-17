# Implementation Plan: Timeline Clip Dragging Fix & Comprehensive Format Support

## Problem Statement

1. **Clip Dragging Blocked When Moving Left / Towards `0:00`:**
   - When the user tries to drag a video clip left towards `0:00`, `TimeCode::from_pixels(delta_x, pps)` clamped `pixels.max(0.0)`. For negative drag deltas (moving left), `delta_time` became `0.0`, making it impossible to drag clips to the left or to `0:00`.
2. **Format Filter Restrictions:**
   - The file dialog previously only allowed a small subset of lowercase extensions (`["mp4", "mkv", "mov", "avi", "webm", "mp3", "wav", "flac", "aac"]`), hiding common camera, iPhone, Windows, and camcorder formats (`.MP4`, `.MTS`, `.M2TS`, `.WMV`, `.M4V`, `.3GP`, `.M4A`, `.WMA`, `.JPG`, `.PNG`) and lacking an `"All Files (*.*)"` option.

---

## User Review Required

> [!IMPORTANT]
> **Proposed Fixes:**
> 1. **Signed Delta Float Math for Clip Dragging:**
>    - Calculate drag delta as signed floating-point seconds: `new_secs = (cur_secs + (delta_px / pps)).max(0.0)`.
>    - Add automatic magnetic snap to `0:00` (within 15px threshold) so clips snap to the start of the timeline.
> 2. **Universal Format Support in File Pickers:**
>    - Add comprehensive format filters:
>      - **All Videos:** `mp4`, `mov`, `m4v`, `mkv`, `avi`, `wmv`, `webm`, `flv`, `ts`, `mts`, `m2ts`, `3gp`, `mpg`, `mpeg`, `vob` (both lowercase and uppercase).
>      - **All Audio:** `mp3`, `wav`, `m4a`, `aac`, `flac`, `ogg`, `wma`, `opus`, `aiff`, `alac`.
>      - **Photos / Images:** `jpg`, `jpeg`, `png`, `webp`, `bmp`.
>      - **`All Files (*.*)` filter:** To allow opening any file on the computer.

---

## Open Questions

> [!NOTE]
> 1. **Magnetic Snap Strength:** When dragging near `0:00` (e.g. within 0.2 seconds), should the clip snap to the origin? (Recommended: Yes).
> 2. **Track Dragging Between Tracks:** Would you like to be able to drag clips vertically between video tracks (e.g., from Video 1 to Video 2)?

---

## Proposed Changes

### 1. Timeline & Drag Engine (`src/ui/` & `src/core/`)

#### [MODIFY] [`timeline_view.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/ui/timeline_view.rs)
- Fix drag delta calculation using signed float seconds:
  ```rust
  let cur_secs = clip.timeline_start.as_secs_f64();
  let delta_secs = (clip_resp.drag_delta().x / pps) as f64;
  let new_secs = (cur_secs + delta_secs).max(0.0);
  let snapped = snap_fn(TimeCode::from_secs_f64(new_secs));
  ```
- Ensure snapping candidate at `0.0s` has a clear snap radius.
- Restrict audio envelope click-interact so top title bar of clip handles full horizontal dragging without gesture interference.

#### [MODIFY] [`time.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/core/time.rs)
- Support signed seconds conversion `from_signed_secs_f64` or signed pixel mapping if needed.

---

### 2. Format Ingestion Expansion (`src/ui/` & `src/media/`)

#### [MODIFY] [`menu_bar.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/ui/menu_bar.rs)
- Update `rfd::FileDialog` with comprehensive video, audio, image, and All Files filters.

#### [MODIFY] [`media_bin.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/ui/media_bin.rs)
- Update file dialog and drag-and-drop ingestion to support all extensions and image formats.

---

## Verification Plan

### Automated Tests
1. **Clip Drag Math Test:**
   - Test moving clip from `5.0s` with `-5.0s` delta reaches `0.0s`.
   - Test snapping to `0.0s` from `0.1s`.
2. **Format List Test:**
   - Verify all file extensions are supported.

Run command:
```bash
cargo test
```

### Windows Release Verification
1. Build Windows release executable:
   ```bash
   cargo build --release
   ```
2. Verify Drag to 0:00:
   - Import video.
   - Drag clip from 5s left to 0:00 and verify it moves and snaps to 0:00.
