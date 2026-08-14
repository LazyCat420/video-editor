# Implementation Plan: Prevent Auto-Adding Imported Files/Folders to Timeline

**Repository:** `video-editor` (`LazyCat420/video-editor`)  
**Status:** Plan for review — **not to be implemented until approved.**

---

## 1. Problem Statement & Root Cause Analysis

### Expected Behavior
- When importing media (e.g. clicking **"📁 Add Entire Folder"**, **"+ Add Video / Music"**, or importing via Menu/Drag & Drop), the files should only appear in the **Files panel** (`project.media_assets`).
- The timeline tracks should not be modified automatically.

### Actual Behavior
- `import_file(...)` calls `self.add_asset_to_timeline(asset)` immediately for each imported file, automatically dumping every video sequentially onto Track 1.

### Root Cause
- [`src/app.rs:194-203`](file:///home/lazycat/github/projects/sun/video-editor/src/app.rs#L194-L203) links `add_media_to_bin` directly with `add_asset_to_timeline`.
- [`src/app.rs:1071-1083`](file:///home/lazycat/github/projects/sun/video-editor/src/app.rs#L1071-L1083) calls `self.import_file` inside `MediaBinAction::ImportFiles` and `MediaBinAction::ImportFolder`.

---

## 2. Proposed Changes

### `src/app.rs`
- In `MediaBinAction::ImportFiles(paths)`: call `self.add_media_to_bin(path)` (media bin only).
- In `MediaBinAction::ImportFolder(dir)`: call `self.add_media_to_bin(file)` (media bin only).
- In `MenuAction::ImportMedia`: call `self.add_media_to_bin(file)` (media bin only).
- Keep `add_asset_to_timeline` for explicit user timeline additions (clicking "+" on a card or dragging to timeline).

---

## 3. Verification & Testing Strategy

### Automated Regression Test
- Add unit test asserting that importing multiple files populates `project.media_assets` without adding clips to `project.timeline.tracks`.

### Manual Verification
1. Click **"📁 Add Entire Folder"** with a test directory.
2. Confirm files populate in the Files panel.
3. Confirm Timeline stays completely untouched and empty until dragged.
