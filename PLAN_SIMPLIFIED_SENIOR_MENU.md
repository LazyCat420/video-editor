# Implementation Plan: Ultra-Simplified Senior-Friendly Video Controls

## Goal & Problem Analysis

### 1. Issues Identified from User Screenshot:
1. **Missing Character Boxes (`□`):** Complex emoji symbols rendered as broken boxes on Windows default font.
2. **Menu Overload:** 9 options with technical audio/trim concepts caused cognitive overload for older users.
3. **Too Small / Cluttered:** Small text and tight margins made clicking difficult.

---

## 2. Ultra-Simplified Design (Only Essential Functions)

### A. Dead-Simple Right-Click Menu (5 Big Clean Choices):
Remove all trims, fades, and sub-menus. Keep ONLY the core actions in large 15px text:
1. **`✂ Cut Video Here`** (or `Split Video Here`)
2. **`➗ Divide in Half`**
3. **`📋 Copy Clip`**
4. **`📋 Paste Clip`**
5. **`🗑 Delete Clip`**

### B. Clean Font & Standard Symbols (No Broken `□` Boxes):
- Replace multi-byte symbol combinations with clean Unicode/ASCII characters guaranteed to render crisply on Windows without missing glyph boxes.

### C. Big 3-Button Timeline Bar:
Directly above the timeline tracks, display only 3 large, obvious buttons:
- `[ ↩ Undo ]` (Revert any mistake)
- `[ ✂ Cut Video (S) ]` (Big friendly slice button)
- `[ 🗑 Delete Selected ]` (Big trash button)

---

## User Review Required

> [!IMPORTANT]
> **What is being removed:**
> - Removed "Trim Start to Red Marker" and "Trim End to Red Marker" (redundant with Cut + Delete).
> - Removed "Auto Fade In" and "Auto Fade Out" from the context menu.
> - Removed technical track-level menus.
>
> **What remains:**
> - `Cut Video Here`
> - `Divide in Half`
> - `Copy Clip`
> - `Paste Clip`
> - `Delete Clip`
> - Big `[ ↩ Undo ]` button above tracks.

---

## Open Questions

> [!NOTE]
> 1. **Menu Wording:** Would you prefer the top option to be labeled **`Cut Video Here`** or **`Split Video Here`**? (Both do the exact same action).
> 2. **Divide in Half:** Is the `Divide in Half` button helpful for you, or would you prefer *only* `Cut`, `Copy`, `Paste`, and `Delete`?

---

## Proposed Changes

### 1. Timeline View (`src/ui/timeline_view.rs`)
- Replace the context menu with large 15px items: `Cut Video Here`, `Divide in Half`, `Copy Clip`, `Paste Clip`, `Delete Clip`.
- Fix all symbol strings to eliminate `□` square box artifacts on Windows.
- Make the top toolbar simple: `[ ↩ Undo ]`, `[ ✂ Cut Video ]`, `[ 🗑 Delete ]`, and zoom.

---

## Verification Plan

### Manual & Visual Verification
1. Build Windows executable:
   ```bash
   cargo build --release
   ```
2. Test Right-Click Menu:
   - Right-click any video clip and verify large, clean 5-item menu with zero broken character boxes.
   - Click `Cut Video Here`, `Divide in Half`, `Copy`, `Paste`, `Delete`, and verify each works smoothly and reverts cleanly with `Undo`.
