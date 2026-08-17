# Root-Cause & Implementation Plan: Cut Clips Playback Fix

## Problem Statement

When the user cuts a video into multiple pieces (or trims/deletes parts), the cut segments do not play or freeze when the playhead enters them.

---

## Root Cause Analysis (First Principles & Evidence):

### 1. The `-re` + `-ss` FFmpeg Stall Bug:
- **The Bug:** FFmpeg's `-re` flag instructs FFmpeg to read the source file at native 1.0x speed starting from timestamp `0.0`.
- When `-ss <offset>` is used with `-re` (e.g. `start_secs = 5.0s` for the second cut clip), FFmpeg **idles in real time for 5.0 seconds** before outputting the first frame at the cut point.
- If a clip starts at 30 seconds into the file, FFmpeg sits idle for 30 real-world seconds before sending any video data, making cut clips appear completely dead/frozen.

### 2. Rust Application Clock vs FFmpeg Rate Limiting:
- In our editor, the playback clock is already driven by `Instant::now()` in Rust (`AudioPlayer::update_playhead`).
- Rust's reader thread already has bounded backpressure (`if buf.len() >= 30 { sleep(15ms) }`), which guarantees that memory never exceeds 30 frames ($20\text{MB}$).
- Therefore, FFmpeg should **never** have `-re` passed to it. FFmpeg must seek instantly ($< 5\text{ms}$) to the exact cut timestamp and immediately provide frames into the 30-frame buffer.

### 3. Missing Duration Limits (`-t` / Segment Bounds):
- When a cut clip has a specific duration (e.g. 4.0s from 5.0s to 9.0s), FFmpeg should be spawned with `-ss 5.0 -t 4.0` so it decodes only the exact trimmed slice belonging to that clip.

---

## User Review Required

> [!IMPORTANT]
> **Proposed Fixes:**
> 1. **Remove `-re` from FFmpeg Spawner:** Allows instant sub-10ms seeking to any cut timestamp without real-time stalls.
> 2. **Pass Exact Segment Duration (`-t`):** Tell FFmpeg the exact duration of the cut clip (`clip.duration()`) so it streams only the valid cut portion.
> 3. **Instant Buffer Priming:** When switching to a new cut clip, the first frame is available immediately to prevent any visual stutter.

---

## Open Questions

> [!NOTE]
> 1. **Cross-Clip Audio Playback:** When playing across cut clips, should the audio player also seek and sync with each cut's `source_in` offset? (Recommended: Yes, ensuring audio and video remain in 100% sync across all cuts).

---

## Proposed Changes

### 1. Stream Video Player (`src/media/stream_player.rs`)

#### [MODIFY] [`stream_player.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/media/stream_player.rs)
- Remove `-re` argument.
- Accept optional `duration_secs: Option<f64>` and pass `-t <duration>` to FFmpeg.
- Ensure reader thread fills the initial lookahead buffer instantly.

---

### 2. App & Clip Info (`src/app.rs`)

#### [MODIFY] [`app.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/app.rs)
- In `get_active_video_clip_info`, return `(clip_id, path, source_time, remaining_duration)`.
- Pass the remaining duration to `stream_player.start()`.

---

## Verification Plan

### Automated Tests
1. **Cut Clip Playback Test:**
   - Create a 10s clip, split at 3.0s and 7.0s.
   - Verify all 3 cut clips calculate correct `source_in` offsets and durations.
2. **Instant Seek & Stream Test:**
   - Verify `StreamVideoPlayer` produces frames for a cut starting at 15.0s within $< 50\text{ms}$ (zero stall).

Run command:
```bash
cargo test
```

### Windows Release Verification
1. Build Windows release binary:
   ```bash
   cargo build --release
   ```
2. Test in UI:
   - Cut a video into 3 pieces (e.g. 0–3s, 3–7s, 7–10s).
   - Click Play from 0:00 and watch playback proceed through all 3 cut pieces without pausing or freezing.
   - Delete the middle piece, move piece 3 to the start, and verify piece 3 plays instantly.
