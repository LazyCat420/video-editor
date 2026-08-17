# Implementation Plan: Smooth Playback & Bounded Memory Architecture

## Selected Focus Task
**Single Task:** Eliminate video playback freezing and guarantee lightweight, crash-free memory management ($< 60\text{MB}$ RAM) on older Dell PCs.

---

## Problem & Root Cause Breakdown

### 1. Why Video Freezes After 1–2 Seconds:
- **OS Stderr Buffer Deadlock:** FFmpeg was spawned with `.stderr(Stdio::piped())`. Because `stderr` was not read, the OS pipe buffer filled up with FFmpeg stream logs after 30–60 frames (1–2 seconds), permanently pausing the FFmpeg process.
- **Fix:** Set `.stderr(Stdio::null())` to allow continuous streaming for hours without blocking.

### 2. Why Memory Spikes & Causes Jitter:
- **Per-Frame GPU Texture Re-upload:** In `preview_player.rs`, `texture.set(frame.clone(), ...)` was re-uploading a 1MB texture to the GPU/RAM 60 times a second ($55\text{MB/sec}$ of continuous churn), even when the frame hadn't changed.
- **Fix:** Track frame version/pointer and only re-upload to GPU when a new decoded frame actually arrives.

### 3. Strict Memory Ceiling ($< 60\text{MB}$ Total RAM):
- **Stream Ring Buffer:** Fixed at 30 frames ($20.7\text{MB}$).
- **Static Seek LRU Cache:** Fixed at 40 frames ($27.6\text{MB}$).
- **Single GPU Texture Handle:** Reused continuously ($0.9\text{MB}$).
- **Total Footprint:** $< 60\text{MB}$ RAM baseline, zero memory leaks, zero GC pauses.

---

## User Review Required

> [!IMPORTANT]
> **Proposed Changes:**
> 1. Fix the `.stderr(Stdio::null())` pipe deadlock in `StreamVideoPlayer`.
> 2. Implement dirty-tracking in `preview_player.rs` so textures are only uploaded when frames change, eliminating $55\text{MB/sec}$ of CPU/GPU churn.
> 3. Cap all internal ring buffers and LRU caches to guarantee memory stays strictly below $60\text{MB}$ RAM.

---

## Open Questions

> [!NOTE]
> 1. **Playback Framerate:** Is 30 FPS preview suitable for low-spec PCs, or would you like 24 FPS / 60 FPS options? (30 FPS provides optimal smoothness and low CPU usage).

---

## Proposed Changes

### 1. Streaming Engine (`src/media/stream_player.rs`)
- Set `.stderr(Stdio::null())`.
- Buffer size fixed at 30 frames with automatic FIFO discard on overrun.

### 2. Preview Renderer (`src/ui/preview_player.rs`)
- Add dirty check before `texture.set()` to prevent redundant allocations and GPU bus traffic.

### 3. Frame Cache (`src/media/frame_cache.rs`)
- Reduce static cache capacity to 40 frames ($< 28\text{MB}$).

---

## Verification Plan

### Automated Tests
1. **Long Stream Test (150+ frames):**
   - Verify uninterrupted stream reading past 5+ seconds.
2. **Bounded Buffer Test:**
   - Verify buffer never exceeds 30 frames under heavy production.

Run command:
```bash
cargo test
```

### Windows Release Verification
1. Build Windows executable:
   ```bash
   cargo build --release
   ```
2. Test Playback:
   - Play 10s and 30s MP4 files.
   - Confirm video plays continuously from start to finish with no freezing and $< 60\text{MB}$ RAM.
