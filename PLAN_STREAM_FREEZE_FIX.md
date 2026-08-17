# Implementation Plan: Fix 1-2 Second Playback Freeze (Pipe Deadlock & Rate Pacing)

## Problem Statement

The video preview starts playing smoothly, but freezes after approximately 1–2 seconds.

### Root Cause Analysis (First Principles & Evidence):

1. **OS Stderr Pipe Buffer Deadlock (The Primary Blocker):**
   - In `StreamVideoPlayer::start()`, the FFmpeg child process is spawned with `.stderr(Stdio::piped())`.
   - Because no thread ever reads from `stderr`, after ~1–2 seconds (30–60 frames), FFmpeg writes warning/diagnostic logs that exceed the OS kernel pipe buffer (4KB–64KB).
   - Once the pipe buffer is full, the operating system pauses the FFmpeg process on `write(stderr)`. FFmpeg halts and stops outputting frames to `stdout`, causing playback to freeze permanently.
2. **Unregulated 60Hz Queue Drain:**
   - The UI update loop ticks at 60 FPS and called `buf.pop_front()` unconditionally on every tick.
   - Because FFmpeg generates frames at 30 FPS, the UI consumed frames at $2\times$ speed, exhausting the buffer in less than a second.

---

## User Review Required

> [!IMPORTANT]
> **Proposed Fixes:**
> 1. **Eliminate Stderr Pipe Deadlock:**
>    - Change `.stderr(Stdio::null())` so FFmpeg never blocks on unread stderr data, allowing indefinite, uninterrupted continuous video playback.
> 2. **Clock-Synchronized Frame Pacing:**
>    - Deliver decoded frames synchronized to the timeline clock so playback matches the exact 1.0x real-time speed.
> 3. **Expanded Frame Buffer (60 frames / 2s lookahead):**
>    - Increase ring buffer size from 30 to 60 frames to absorb any CPU scheduling jitter.

---

## Open Questions

> [!NOTE]
> 1. **Loop Playback:** When playback reaches the end of the video clip, should it automatically loop back to the beginning or stop at the end? (Currently stops at the end).

---

## Proposed Changes

### 1. Stream Engine (`src/media/stream_player.rs`)

#### [MODIFY] [`stream_player.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/media/stream_player.rs)
- Set `.stderr(Stdio::null())` to eliminate OS pipe buffer deadlock.
- Increase buffer capacity to 60 frames.
- Add real-time stream synchronization so frames match the playback clock.

---

## Verification Plan

### Automated Tests
1. **Long Playback Stream Test:**
   - Test streaming 150+ frames (>5 seconds) through the reader loop to verify no pipe deadlock occurs.

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
   - Play 10s MP4 clip (`Biker_with_toucan_head_...mp4`).
   - Confirm video plays continuously for the full 10 seconds without freezing.
