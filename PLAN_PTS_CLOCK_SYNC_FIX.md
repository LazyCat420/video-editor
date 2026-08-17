# Root-Cause & Implementation Plan: PTS Clock Synchronization & 2x Drain Fix

## Problem Statement

- **Symptom 1:** Video freezes halfway through playback (e.g. at 5.0s on a 10s video).
- **Symptom 2:** On small cut clips (e.g. 3s), the second half / end of the clip is completely frozen.

---

## First-Principles Mathematical Proof of the Bug

### 1. The 60Hz UI vs 30 FPS Video Drain Mismatch:
- The `egui` UI renders at **60 FPS** (every 16.6ms).
- In `app.rs`, `self.stream_player.get_next_frame()` was called unconditionally on **every single UI render tick**.
- In `stream_player.rs`, `get_next_frame()` popped one frame from the front of the buffer every time it was called:
  $$\text{Consumption Rate} = 60 \text{ frames/second}$$
  $$\text{Source Video Rate} = 30 \text{ frames/second}$$
- **The Result:** The UI was draining frames at **2.0x speed** while the playhead clock moved at 1.0x speed.
  - On a 10-second clip (300 frames): all 300 frames were drained in $\frac{300}{60} = 5.0\text{ seconds}$ (exactly halfway through the video).
  - On a 3-second cut clip (90 frames): all 90 frames were drained in $\frac{90}{60} = 1.5\text{ seconds}$ (leaving the remaining 1.5s frozen at the end).

---

## The Solution: PTS (Presentation TimeStamp) Pacing

### 1. Attach PTS (Presentation Timestamp) to Each Decoded Frame:
In `StreamVideoPlayer`:
- When spawning FFmpeg at `start_secs`, the reader thread tags each incoming 30 FPS frame with its exact presentation timestamp:
  $$\text{PTS}_N = \text{start\_secs} + \frac{N}{30.0}$$
- The ring buffer stores `(f64, ColorImage)` (the timestamp and frame image).

### 2. Time-Synchronized Frame Retrieval:
Instead of popping blindly on every UI render:
`pub fn get_frame_for_time(&mut self, current_source_time_secs: f64) -> Option<ColorImage>`
- As long as the head of the buffer has $\text{PTS} \le \text{current\_source\_time}$, we pop and advance to that frame.
- If the next frame is in the future ($\text{PTS} > \text{current\_source\_time}$), we keep it in the buffer and display the current frame.
- If the buffer is empty, we hold the current frame.

### 3. Mathematical Outcome:
- A 10.0-second video advances at exact 1.0x real time, playing frame 0 at 0.0s, frame 150 at 5.0s, and frame 300 at 10.0s.
- Cut clips of any size (1s, 3s, 5s) play from start to finish with **zero frozen ends**.
- Memory is strictly bounded to $< 30$ frames ($20\text{MB}$) via producer backpressure.

---

## User Review Required

> [!IMPORTANT]
> **Proposed Changes:**
> 1. Update `StreamVideoPlayer` to attach `pts_secs` to all decoded frames.
> 2. Replace blind `get_next_frame()` with `get_frame_for_time(source_time)` synced to `AudioPlayer` clock.
> 3. Add regression tests validating 1.0x real-time pacing across multiple cut clips.

---

## Open Questions

> [!NOTE]
> 1. **Display Refresh Rate Independence:** With PTS pacing, video playback will stay perfectly in sync whether running on 60Hz, 120Hz, or 144Hz monitors.

---

## Verification Plan

### Automated Tests
1. **Pacing Synchronization Test:**
   - Verify 300 frames over 10.0s advance at exact 30 FPS without premature queue exhaustion.
2. **Cut Clip Boundary Test:**
   - Verify a 3.0s cut clip (90 frames) plays continuously from 0.0s to 3.0s with all frames rendered.

Run command:
```bash
cargo test
```

### Windows Release Verification
1. Build Windows release executable:
   ```bash
   cargo build --release
   ```
2. Test Playback:
   - Play 10s video from start to finish; verify it never freezes halfway.
   - Cut video into 2s and 3s small clips and verify all parts play to the very end with zero frozen tails.
