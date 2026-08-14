# Implementation Plan: Drag-and-Drop Reorder Animation & Push-Down Feedback

**Repository:** `video-editor` (`LazyCat420/video-editor`)  
**Status:** Plan for review — **not to be implemented until approved.**

---

## 1. Problem Statement & Root Cause

### Expected Behavior
- When clicking and dragging a box/card:
  1. A floating preview follows the mouse cursor with an accent glow and shadow.
  2. The target list dynamically pushes adjacent boxes down to create an insertion slot.
  3. The user clearly sees the physical movement of boxes before releasing the mouse.
  4. On drop, the item settles cleanly into the new slot.

### Actual Behavior
- Cards remain rigid and stationary during drag.
- Hovering over other boxes does not push them down or display any drop slot indicator.
- Upon release, the items array swaps instantly in place, making it appear as if only the text changed rather than boxes physically moving.

### Root Cause
- In [`src/ui/media_bin.rs:186-198`](file:///home/lazycat/github/projects/sun/video-editor/src/ui/media_bin.rs#L186-L198), `dnd_hover_payload` is not handled to create spacing or push cards down.
- No floating layer painter is used to render the dragged box moving across the screen with the cursor.
- Slide items in [`src/ui/slide_bin.rs`](file:///home/lazycat/github/projects/sun/video-editor/src/ui/slide_bin.rs) only have up/down buttons with instant text swap and no drag-and-drop animation.

---

## 2. Proposed Changes

### 1. `src/ui/media_bin.rs`
- Add dynamic drop insertion slots: when dragging a card over another card, dynamically insert a 32px glowing insertion drop zone with `"⬇ Insert Here"`, physically pushing down subsequent cards in the layout.
- Render a floating translucent card preview attached to the mouse pointer (`LayerId::new(Order::Tooltip, ...)`).
- Render the source card as a dimmed placeholder (dashed outline) while being dragged.

### 2. `src/ui/slide_bin.rs`
- Add drag-and-drop handles (`⠿`) to slide elements in the sidebar items list with the same push-down drop slot animation.

---

## 3. Verification & Testing Strategy

### Automated Tests
- Unit tests in `tests/slide_dnd_tests.rs` for drag-and-drop reorder slot calculation and array mutation.

### Manual Verification
1. Drag a video card in the Files bin over another card.
2. Confirm the floating card follows cursor and the hovered card slides down to open the insertion gap.
3. Drop the card and confirm it lands precisely in the opened slot.
