# Implementation Plan: PowerPoint-Style Drag-and-Drop Slide Canvas & Context-Aware Sidebar

**Repository:** `video-editor` (`LazyCat420/video-editor`)  
**Status:** Plan for review — **not to be implemented until approved.**

---

## 1. Executive Summary & Problem Analysis

In the current video editor:
1. **Drag-and-Drop to Canvas is missing**: `MediaBinView` enables dragging assets via `MediaAssetDrag`, but `PreviewPlayerView` does not accept drops. Dragging images or videos onto the blank slide preview canvas has no effect.
2. **Text creation requires pre-filling sidebar forms**: Users must navigate form inputs before placing text. If draft text is empty, placing text creates an invisible element that is suppressed by rendering guards (`!o.text.trim().is_empty()`).
3. **Sidebar is cluttered with static forms**: Rather than acting as a fast PowerPoint-style property palette that binds to the selected element on canvas, the sidebar requires filling out disconnected fields.

---

## 2. Locked Design Decisions (from `/grill-me` interview)

| Decision Area | Agreed Design Behavior |
|---|---|
| **Canvas Drag & Drop** | Dragging an image or video from the Files bin (or dropping files from OS explorer) onto the canvas drops it as a placed, movable, and resizable element (`SlideElement::Picture` / `SlideElement::Video`) centered at the drop point `(x, y)`. |
| **Click-to-Add Text** | Clicking the canvas with the Text tool armed creates a text box at `(x, y)` with default placeholder text (`"Click to edit text"`) and immediately selects it for dragging, inline editing, and sidebar styling. |
| **Context-Aware Sidebar** | The left sidebar functions as a styling palette that automatically binds to whichever element is currently selected on the canvas (or slide background when none is selected). |
| **Canvas Element Manipulation** | Click to select, drag to reposition, drag corner to resize, right-click for quick actions (`Fill Slide`, `Set as Slide Background`, `Bring to Front`, `Send to Back`, `Delete`), and `Delete`/`Backspace` key to remove. |

---

## 3. Verified Claims Matrix (VCPM Standard)

- **[C1 - Verified Fact]**: `src/ui/preview_player.rs:95` allocates canvas size with `Sense::click_and_drag()` but has zero `dnd_hover_payload` or `dnd_release_payload` calls. *(Source: `preview_player.rs:95`)*
- **[C2 - Verified Fact]**: `src/ui/media_bin.rs:179` sets `card_resp.dnd_set_drag_payload(MediaAssetDrag(asset.id))` which is currently only handled by timeline tracks. *(Source: `timeline_view.rs:614-640`)*
- **[C3 - Verified Fact]**: `src/app.rs:543` checks `!o.text.trim().is_empty()`, causing blank draft text to be omitted from visual rendering. *(Source: `app.rs:543`)*
- **[C4 - Testable Claim]**: Implementing `dnd_release_payload::<MediaAssetDrag>` and OS file drop on preview canvas allows dropping media assets directly into the active slide at normalized `(x, y)` coordinates.
- **[C5 - Testable Claim]**: Maintaining `selected_slide_element: Option<usize>` enables bidirectional synchronization between canvas hit-testing and the sidebar inspector.
- **[C6 - Assumption-1]**: Default element dimensions of `w: 0.4, h: 0.3` for pictures and `w: 0.5, h: 0.3` for videos provide an optimal starting layout on a 16:9 canvas prior to user resizing. *(Risk: Low; users can immediately drag corner to resize or click 'Fill Slide')*

---

## 4. Detailed Component Implementation Plan

### 4.1 `src/ui/preview_player.rs` — Canvas Drag-and-Drop & PowerPoint-Style Interaction
1. **DND Drop Target**:
   - Check `response.dnd_hover_payload::<MediaAssetDrag>()`: draw illuminated cyan/blue drop border and badge `"➕ Drop onto slide"`.
   - Check `response.dnd_release_payload::<MediaAssetDrag>()`: emit `PlayerAction::DropMediaAsset { asset_id, x, y }`.
   - Check `ui.input(|i| i.raw.dropped_files)`: emit `PlayerAction::DropFiles { paths, x, y }`.
2. **Selection & Handle Rendering**:
   - Render selection outline with corner resize handles around the currently active `selected_element: Option<usize>`.
   - When hovering over element body: cursor becomes `Move` / `Grab`.
   - When hovering over corner resize handle: cursor becomes `ResizeSouthEast`.
3. **Click & Drag State Machine**:
   - Click on element: selects element.
   - Click on empty canvas: deselects element (`selected_element = None`).
   - Drag element: moves element `(x, y)` in real time.
   - Drag corner handle: resizes element `(w, h)` in real time.
4. **Context Menu & Hotkeys**:
   - Secondary click (right-click) on element opens context menu:
     - `⛶ Fill Slide`
     - `🖼 Set as Slide Background` (for picture elements)
     - `⬆ Bring Forward` / `⬇ Send Backward`
     - `🗑 Delete Element`
   - Key handlers: `Delete` / `Backspace` deletes selected element; `Escape` deselects.

### 4.2 `src/ui/slide_bin.rs` — Context-Aware Property Palette & Tools
1. **Quick Tool Bar**:
   - `➕ Add Blank Slide`
   - `✏️ Add Text Box` (arms text tool for 1-click canvas placement)
   - `🖼 Add Picture` / `🎞 Add Video` (file dialog or prompt to drag from files bin)
   - `🎵 Add Audio`
2. **Context-Sensitive Inspector (Active when `selected_element` is `Some(idx)`)**:
   - **When Text is Selected**:
     - `TextEdit::multiline` bound to element text for direct live editing.
     - Font family dropdown (10 presets).
     - Font size slider (14 to 120 pt).
     - Bold (`B`) / Italic (`I`) toggles.
     - Text Color swatches + Box background style (`No Background`, `Tight Box`, `Full Banner`).
     - Layer order buttons (`⬆`, `⬇`) and `🗑 Delete`.
   - **When Picture / Video is Selected**:
     - Element filename and thumbnail.
     - `⛶ Fill Slide` button and `🖼 Set as Slide Background` button.
     - Layer order buttons (`⬆`, `⬇`) and `🗑 Delete`.
3. **Slide Background Inspector (Active when `selected_element` is `None`)**:
   - Solid color palette swatches.
   - `🖼 Pick Background Photo` button.
   - Element layer manager showing all items on slide.

### 4.3 `src/app.rs` — App State & Dispatch Wiring
1. Store `selected_slide_element: Option<usize>`.
2. Add handlers for:
   - `DropMediaAsset { asset_id, x, y }`:
     - Resolves active slide (or creates new blank slide at playhead if none).
     - Locates `MediaAsset` by `asset_id`.
     - Adds `SlideElement::Picture` (if image) or `SlideElement::Video` (if video) at normalized `(x, y)`.
     - Sets `selected_slide_element = Some(new_idx)`.
     - Triggers preview refresh.
   - `DropFiles { paths, x, y }`:
     - Imports files to `project.media_assets`.
     - Inserts visual slide elements at `(x, y)`.
   - `SelectElement(Option<usize>)`:
     - Synchronizes selection state.
   - `DeleteSelectedElement`:
     - Removes element from active slide and snapshots timeline for undo.
   - `SetAsBackground(usize)`:
     - Converts image element to `SlideBackground::Picture` and removes element from list.
3. Update `place_pending_element`:
   - If placed text is empty, auto-populate `"Click to edit text"`.
   - Set `selected_slide_element = Some(new_idx)`.

---

## 5. Verification & Testing Strategy

### 5.1 Automated Tests (`cargo test`)
1. `test_slide_element_drop_placement`: Assert that dropping an asset at `(0.3, 0.4)` adds `SlideElement::Picture` or `SlideElement::Video` with bounds matching those coordinates.
2. `test_click_to_add_text_default_placeholder`: Assert that placing text with empty draft populates `"Click to edit text"` and is returned in `slide_visuals`.
3. `test_element_selection_and_deletion`: Assert that deleting an element at index `idx` updates `clip.elements` and creates an undo history snapshot.
4. `test_set_element_as_background`: Assert that promoting a picture element sets `clip.background = Some(SlideBackground::Picture(...))` and removes the element.

### 5.2 Manual Verification Steps
1. Add a blank slide on timeline.
2. Drag an image from the left `Files` bin and drop it directly onto the slide preview canvas -> verify it lands at mouse position with resize handles.
3. Drag a video from `Files` bin onto the canvas -> verify both video and picture render on canvas.
4. Click `✏️ Add Text Box` -> click canvas -> verify text box appears with `"Click to edit text"` and is selected.
5. In left sidebar, type text, pick a font preset, change color, toggle tight box background -> verify real-time updates on canvas.
6. Drag elements around canvas to position them; drag bottom-right corner to resize.
7. Right-click an element -> test `Fill Slide` and `Set as Slide Background`.
