# Root-Cause & Implementation Plan: Real-Time Stream Pacing & Seamless Clip Transitions

## Problem Statement

1. **Mid-Clip Freeze:** When playing a video clip, playback freezes halfway through (or after 1–2 seconds) because FFmpeg runs unthrottled, decodes the entire clip in ~1.5s, overflows the buffer, and exits.
2. **Next-Clip Transition Freeze:** When the playhead crosses into the next clip on the timeline, playback does not switch to the next clip.

---

## Root Cause Analysis (First Principles & Evidence):

### 1. The Fast-Producer Buffer Eviction Race:
- Without the `-re` (real-time read) flag, FFmpeg runs at maximum CPU speed (150–300 FPS), decoding a 10-second video in ~1.5 seconds.
- In `stream_player.rs`, when the 30-frame buffer filled up, the reader thread called `buf.pop_front()` to make room for newer incoming frames.
- **The Consequence:** In the first 1.5 seconds of wall-clock time, FFmpeg decoded all 300 frames of the video, discarded frames 0–270 from the buffer, kept only the final frames (271–300), and terminated on EOF.
- When the UI playhead was only at second 2, the buffer was exhausted, FFmpeg was dead, and playback froze.

### 2. Producer Backpressure Missing:
- The reader thread should **never** discard unconsumed frames from the front of the queue. If the buffer has reached its target lookahead (e.g. 30 frames), the producer thread must **pause reading** (backpressure) and wait until the UI consumes frames at the real-time playback rate.

### 3. Missing Clip Boundary Transition:
- During playback, `stream_player` was only initialized once on the initial clip. When the timeline playhead crossed into an adjacent clip, `stream_player` was not instructed to switch to the new clip.

---

## User Review Required

> [!IMPORTANT]
> **Proposed Fixes:**
>
> 1. **FFmpeg Real-Time Rate Flag (`-re`):**
>    - Add `-re` before `-i` in the FFmpeg command line. This forces FFmpeg to pace frame generation to exactly 1.0x native video speed (30 frames/sec), matching the real-time clock.
> 2. **Producer Backpressure (Zero Frame Discard):**
>    - When the ring buffer reaches 30 frames, the background reader thread sleeps briefly ($15\text{ms}$) instead of dropping earlier frames. Frames are ONLY consumed when the UI clock requests them.
> 3. **Automatic Multi-Clip Transitioning:**
>    - In `app.rs`, track `current_playing_clip_id`. When the playhead crosses from Clip A to Clip B, automatically start streaming Clip B seamlessly.

---

## Open Questions

> [!NOTE]
> 1. **Gap Handling:** If there is a gap on the timeline between two video clips (e.g., 2 seconds of empty space before the next clip), should the player display black and continue playing until it reaches the next clip, or stop? (Recommended: Display black and continue playing to the next clip).

---

## Proposed Changes

### 1. Streaming Decoder Engine (`src/media/stream_player.rs`)

#### [MODIFY] [`stream_player.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/media/stream_player.rs)
- Add `-re` input flag to throttle FFmpeg to real-time speed.
- Implement producer backpressure:
  ```rust
  while is_running_arc.load(Ordering::SeqCst) {
      if buffer_arc.lock().unwrap().len() >= 30 {
          thread::sleep(Duration::from_millis(15));
          continue;
      }
      // read exact frame bytes...
  }
  ```

---

### 2. App & Multi-Clip Timeline Management (`src/app.rs`)

#### [MODIFY] [`app.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/app.rs)
- Track active playing clip ID.
- Detect clip transitions on playhead advance and switch `stream_player` to the new clip automatically.

---

## Verification Plan

### Automated Tests
1. **Producer Backpressure Test:**
   - Verify reader thread pauses when buffer is full and produces frames only as buffer is drained.
2. **Multi-Clip Timeline Transition Test:**
   - Verify timeline transitions between adjacent clips at boundary timestamps.

Run command:
```bash
cargo test
```

### Windows Release Verification
1. Build Windows release binary:
   ```bash
   cargo build --release
   ```
2. Test Playback:
   - Play a 10s MP4 clip from 0:00 to 0:10 continuously without freezing.
   - Put two clips side-by-side on the timeline and verify playback continues seamlessly from Clip 1 into Clip 2.
