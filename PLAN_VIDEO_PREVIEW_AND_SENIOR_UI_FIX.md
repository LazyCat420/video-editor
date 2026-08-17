# Implementation Plan: Video Preview Fix & Senior-Friendly UI Simplification

## Problem Statement

1. **Video Preview Not Visible:** When the user imports a video (`Biker_with_toucan_head_202606070202.mp4`, duration 10.0s), the preview window remains black displaying `"Video Preview"`. The timeline playhead starts or ends at `00:00:11.17` (past the 10.0s clip), causing `get_clip_at(playhead)` to return `None`. In addition, synchronous `ffmpeg` process calls on the UI thread cause frame drops if `ffmpeg.exe` resolution has any latency on Windows.
2. **UI Too Complicated for Older Users:** The current interface has small technical buttons (`Split (S)`, `Snap: ON`, `M`, `S`, `⏮ ⏪ ⏸ ⏩ ⏹`), small 12px fonts, confusing technical tracks (`Video 2 (Overlay)`, `Audio 1 (Dialogue)`, `Audio 2 (Music / BGM)`), and SMPTE timecodes (`00:00:11:17`). Older users need an intuitive, large-font, step-by-step workflow with high contrast and clear plain-English buttons.

---

## User Review Required

> [!IMPORTANT]
> **Key Simplifications Proposed for Older Users:**
> 1. **3-Step Action Header:** Replace the cluttered menu and toolbar with a prominent, numbered workflow:
>    - **Step 1: 📂 Open Video or Music** (Large blue button, 42px height)
>    - **Step 2: ✂ Cut / Split** & **🗑 Remove Clip** (Large clear action buttons)
>    - **Step 3: 🚀 Save & Export Finished Video** (Large vibrant green button)
> 2. **Simplified 2-Track System:** 
>    - 🎬 **Video Track (Movies & Clips)**
>    - 🎵 **Music & Sound (Songs & Voice)**
> 3. **Large-Format Preview Controls:**
>    - Big, friendly play button (`▶ PLAY` / `⏸ PAUSE`) with large text (18px).
>    - Plain-English time readout: `0:05 / 0:10 (5 seconds of 10s total)`.
>    - Auto-Rewind to `0:00` when a video is loaded so the first frame appears instantly.
> 4. **Embedded Guidance / Tips Banner:**
>    - Bottom status bar providing friendly hints: *"Click '✂ Cut' to split video where the red line is", "Drag the yellow line on audio to lower music volume"*.

---

## Open Questions

> [!NOTE]
> 1. **Default Track Setup:** Should we start with 2 tracks (1 Video + 1 Music/Audio) by default, and allow adding more only if requested via an `+ Add Another Track` button?
> 2. **Timecode Format:** Would you prefer simple seconds `0:05 / 0:10` as the primary display, with full SMPTE timecode toggleable in settings?
> 3. **Automatic Auto-Play vs Pause on Load:** When you drop/add a video, should it stay paused on frame 0, or immediately start previewing?

---

## Proposed Changes

Grouped by component and layer:

### 1. Media & Preview Subsystem (`src/media/` & `src/core/`)

#### [MODIFY] [`frame_cache.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/media/frame_cache.rs)
- Add non-blocking background frame pre-fetching and async decoding channel so UI never stutters.
- Add robust `ffmpeg.exe` executable discovery (checking current working dir, local `./ffmpeg`, system PATH, Windows standard directories).
- Add fast fallback: store the first frame / thumbnail on import so the preview is never blank even before full seeking occurs.

#### [MODIFY] [`clip.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/core/clip.rs)
- Ensure clip boundary checks clamp queries within `[source_in, source_out]` safely.

---

### 2. UI Simplification & Senior-Friendly Redesign (`src/ui/`)

#### [MODIFY] [`theme.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/ui/theme.rs)
- Increase base typography scale: button text 16px–18px, headings 20px, labels 15px.
- Increase minimum button padding to $12\text{px} \times 8\text{px}$ for easy mouse targeting.
- Enhance contrast with warm high-visibility colors (High-contrast Yellow playhead, Vibrant Green export button, Clear Cyan audio envelope line).

#### [MODIFY] [`preview_player.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/ui/preview_player.rs)
- Redesign the transport bar with large, easy-to-read buttons:
  - `[ ⏮ Rewind to Start ]`
  - `[ ▶ PLAY ]` (Large 40px pill button)
  - `[ ⏪ -1 Sec ]` `[ ⏩ +1 Sec ]`
- Replace raw SMPTE with friendly human-readable time format: `0:04 / 0:10 (4s of 10s)`.
- If the playhead is beyond the last video clip, display a helpful message: *"End of video reached. Click '⏮ Rewind to Start' to watch again."*

#### [MODIFY] [`timeline_view.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/ui/timeline_view.rs)
- Enlarge track row heights from 52px to 72px for comfortable visibility.
- Simplify track controls:
  - Replace tiny `M`/`S` and rotary knobs with clear `🔊 Volume: 100%` slider and a big `Mute` toggle button.
  - Large track icons: 🎬 **Video Track** and 🎵 **Music & Audio Track**.
- Add intuitive node graph overlay on the audio track with a direct tooltip: *"Click anywhere on the yellow line to adjust volume"*.

#### [MODIFY] [`menu_bar.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/ui/menu_bar.rs)
- Replace complex menus with an intuitive top banner:
  - **Step 1: 📂 Open Video / Music** (Large blue button)
  - **Step 2: ✂ Cut Video** (Large scissor button)
  - **Step 3: 🗑 Delete Selected**
  - **Step 4: 🚀 Export Finished Video** (Large green button)
  - **Help / Tutorial Button (`❓ How to Use`)**: Opens an instant popup with simple 3-step instructions.

#### [MODIFY] [`app.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/app.rs)
- On `import_file` or `add_asset_to_timeline`: automatically reset `playhead = TimeCode::ZERO` and trigger an immediate preview refresh so the user instantly sees their video frame on screen.
- Auto-clamp playhead to the end of timeline during playback (stop cleanly at duration end instead of running into black space).

---

## Verification Plan

### Automated Tests
1. **Clip Boundary & Clamping Unit Tests:**
   - Verify `timeline_to_source_time` returns `None` outside clip and correct frame time inside clip.
2. **Timeline Playhead Reset Tests:**
   - Verify `add_asset_to_timeline` correctly sets duration and bounds playhead.
3. **Audio Node Graph Tests:**
   - Verify audio envelope gain calculation and keyframe manipulation.

Run command:
```bash
cargo test
```

### Windows Build & Run Verification
1. Compile Windows release executable:
   ```bash
   cargo build --release --target x86_64-pc-windows-gnullvm
   ```
2. Verify binary exists and passes PE32+ checks:
   ```bash
   file target/x86_64-pc-windows-gnullvm/release/video-editor.exe
   ```
3. Launch and verify UI:
   - Import `Biker_with_toucan_head_202606070202.mp4`
   - Confirm first video frame is immediately visible in the preview player.
   - Confirm large friendly buttons, clear track labels, and step-by-step layout.
