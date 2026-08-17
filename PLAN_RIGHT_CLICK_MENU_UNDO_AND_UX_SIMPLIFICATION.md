# Implementation Plan: Right-Click Context Menu, Undo/Redo Engine & Simplified Editing UX

## Overview & Goal

Enhance the video editor with intuitive mouse interactions, robust Undo/Redo history, and high-impact UX simplifications designed to make video editing effortless for non-technical and older users without removing any core power features.

---

## 1. Core Requested Features

### A. Right-Click Context Menu on Timeline & Clips
- Right-clicking directly on any clip or timeline track opens a large, easy-to-read menu:
  - **`✂️ Cut / Split Here`**: Slices the clip at the exact clicked timestamp.
  - **`➗ Divide into 2 Halves`**: Splits the clip exactly in the middle.
  - **`📋 Copy Clip`**: Copies the selected clip with all its trim points and volume nodes.
  - **`📋 Paste Clip at Cursor`**: Pastes the copied clip at the right-click timestamp.
  - **`🗑️ Delete Clip`**: Removes the clip from the timeline.
  - **`🔇 Mute / Unmute Clip`**: Toggles clip sound on/off.
  - **`📈 Auto-Fade In (1s) / Auto-Fade Out (1s)`**: Automatically creates smooth volume ramp nodes.

### B. Undo / Redo Toolbar Above Timeline
- Dedicated toolbar located directly above the timeline tracks:
  - **`[ ↩️ Undo (Ctrl+Z) ]`** (Large button with active/disabled state).
  - **`[ ↪️ Redo (Ctrl+Y) ]`** (Large button with active/disabled state).
  - **`[ 🧲 Auto-Snap Gaps ]`** (Toggles automatic closing of black gaps between clips).
- Timeline History Stack: Stores up to 50 undo snapshots (`Vec<Timeline>`) captured before any destructive or mutating operation (splits, moves, trims, deletes, volume adjustments).

---

## 2. Additional Ideas to Make Editing Easier & Less Complicated

### Idea 1: "Gap Closer / Magnet" (Eliminates Accidental Black Screens)
- **The Problem:** Older adults frequently leave small 0.5s–2s empty spaces between clips when dragging, resulting in accidental black screens during playback.
- **The Solution:** A `🧲 Close All Gaps` button on the timeline toolbar that magnetically pulls all clips on a track together so they touch edge-to-edge.

### Idea 2: 1-Click "Trim to Marker" Buttons
- **The Problem:** Dragging the tiny 4px edges of clips to trim can be difficult for shaky hands or trackpads.
- **The Solution:** Two large buttons in the toolbar / context menu:
  - `[ ✂️ Cut Off Everything Before Marker ]` (Trims start of clip to playhead).
  - `[ ✂️ Cut Off Everything After Marker ]` (Trims end of clip to playhead).

### Idea 3: 1-Click Volume Fade Presets
- **The Problem:** Clicking to add tiny yellow dots on the volume curve requires fine motor precision.
- **The Solution:** One-click presets:
  - `[ 📉 Fade In (Start Soft) ]`
  - `[ 📈 Fade Out (End Soft) ]`
  - `[ 🗣️ Duck Music (Softer during Talking) ]`

### Idea 4: Direct "Add Music" Quick-Button on Audio Track
- In the empty space of `🎵 Music & Sound Track`, show a large friendly button: `[ 🎵 + Choose Music for Video ]` that directly opens the audio file picker and drops the song onto the audio track.

### Idea 5: Plain-English Export Presets
- In the Export Dialog, replace technical bitrates with 3 clear cards:
  - `📱 For Phone & Texting (Small File, Fast)`
  - `🖥️ Standard TV & YouTube (1080p - Recommended)`
  - `🌟 Highest Quality (Best)`

---

## User Review Required

> [!IMPORTANT]
> **Proposed Implementation Phases:**
> 1. **Phase 1 (Requested Core):**
>    - Full Timeline History Stack (`undo_stack` & `redo_stack`).
>    - Large `[ ↩️ Undo ]` and `[ ↪️ Redo ]` buttons directly above timeline.
>    - Right-click context popup on clips & tracks (`Cut`, `Divide`, `Delete`, `Copy`, `Paste`, `Mute`, `Fade In/Out`).
> 2. **Phase 2 (UX Enhancements):**
>    - `🧲 Close Gaps` button.
>    - `Trim to Playhead` action buttons.
>    - 1-Click Audio Fade Presets.

---

## Open Questions

> [!NOTE]
> 1. **Right-Click Paste Target:** When you right-click on an empty track area, should `Paste` paste the copied clip at the exact mouse timestamp? (Recommended: Yes).
> 2. **Undo Depth:** Is 50 levels of Undo history sufficient? (Uses $< 2\text{MB}$ of memory).
> 3. **Which additional ideas would you like included first?** (e.g. `Close All Gaps`, `1-Click Fades`, `Trim to Marker`).

---

## Proposed Changes

### 1. Core Timeline & History Engine (`src/core/`)

#### [NEW] [`src/core/history.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/core/history.rs)
- Implement `TimelineHistory`:
  - `push_snapshot(&mut self, timeline: &Timeline)`
  - `undo(&mut self, current: &Timeline) -> Option<Timeline>`
  - `redo(&mut self, current: &Timeline) -> Option<Timeline>`
  - `can_undo()`, `can_redo()`

#### [MODIFY] [`src/core/timeline.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/core/timeline.rs)
- Add `close_gaps(&mut self, track_id: u64)` method.
- Add clipboard support (`clipboard_clip: Option<Clip>`).

---

### 2. Timeline UI & Context Menu (`src/ui/timeline_view.rs`)

#### [MODIFY] [`timeline_view.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/ui/timeline_view.rs)
- Add timeline toolbar with large `[ ↩️ Undo ]`, `[ ↪️ Redo ]`, `[ 🧲 Close Gaps ]`.
- Add `clip_resp.context_menu(...)` with large action items:
  - `✂️ Cut / Split Here`
  - `➗ Divide in Half`
  - `📋 Copy Clip`
  - `📋 Paste Clip`
  - `📈 Auto Fade In (1s)`
  - `📉 Auto Fade Out (1s)`
  - `🗑️ Delete Clip`

---

### 3. App Integration (`src/app.rs`)

#### [MODIFY] [`app.rs`](file:///home/lazycat/github/projects/sun/.worktrees/video-editor/src/app.rs)
- Integrate `TimelineHistory` with global hotkeys `Ctrl+Z` and `Ctrl+Y`.
- Snapshot timeline state before mutating actions.

---

## Verification Plan

### Automated Tests
1. **Undo / Redo History Test:**
   - Test sequence: Add clip -> Split -> Undo -> Verify single clip -> Redo -> Verify split.
2. **Copy / Paste Test:**
   - Test copying clip from Track 1 at 0s and pasting to Track 1 at 5s.
3. **Close Gaps Test:**
   - Test multiple clips with 2s gaps collapsing into contiguous clips.

Run command:
```bash
cargo test
```

### Windows Release Verification
1. Build Windows release binary:
   ```bash
   cargo build --release
   ```
2. Test Right-Click & Undo in UI:
   - Right-click a clip and test `Cut Here`, `Divide`, `Copy`, `Paste`, `Delete`.
   - Click `[ ↩️ Undo ]` and verify action reverses cleanly.
