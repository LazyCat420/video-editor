# Implementation Plan: 'Add Blank Page' Toolbar Button & PowerPoint-Style Media Canvas

**Repository:** `video-editor` (`LazyCat420/video-editor`)  
**Status:** Plan for review — **not to be implemented until approved.**

---

## 1. Problem Statement & User Goal

### User Goal
- Add a dedicated **"➕ Add Blank Page"** button on the bar directly above the timeline (next to the Zoom slider).
- Make sure clicking this button creates a blank page where the user can drag & drop pictures, videos, audio, and click to add text just like PowerPoint.

---

## 2. Proposed Changes

### 1. `src/ui/timeline_view.rs`
- Add `TimelineAction::AddBlankSlide { duration: f64 }`.
- In the top timeline control toolbar, place a prominent green button:
  `➕ Add Blank Page` right beside the Zoom slider.

### 2. `src/app.rs`
- In `TimelineAction` handler:
  - When `TimelineAction::AddBlankSlide` triggers:
    - Save timeline undo snapshot.
    - Insert blank slide at playhead.
    - Select the new blank slide and set playhead over it.
    - Switch sidebar tab to `Titles & Text` (Slide Inspector).
    - Refresh preview frame so the blank page canvas is immediately interactive.

### 3. `src/ui/preview_player.rs`
- Ensure dragging media onto the canvas over a blank slide drops visual elements at cursor `(x, y)` with resize grab handles and context-aware sidebar binding.

---

## 3. Verification & Testing Strategy

### Automated Tests
- Unit test in `tests/slide_dnd_tests.rs` for toolbar blank page insertion, element composition, and selection.

### Manual Verification
1. Run application.
2. Click **"➕ Add Blank Page"** next to Zoom slider.
3. Confirm blank page appears on timeline and in preview canvas.
4. Drag images and videos from Files bin onto canvas.
5. Click "Add Text Box" and click on canvas to type text.
