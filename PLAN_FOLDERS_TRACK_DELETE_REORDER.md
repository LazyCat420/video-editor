# Implementation Plan: Folder Bins, Drag-to-Timeline, Track Delete & Reorder (Senior-Friendly)

## Goal & Problem Analysis

Everything below is **net-new UI** wired onto hooks that already exist in the model:

1. **Delete tracks is already implemented in the engine** (`Timeline::remove_track` — `src/core/timeline.rs:59`) but **nothing in the UI calls it**. There is no way to remove a track today.
2. **The media panel ("📁 Your Files") is a flat pile**: a single list, an "Add Video/Music" button, and a "▶ Put on Timeline" button per file. No folders, no ability to pull in a whole folder at once, and **no drag-from-panel-onto-track** (drag only works on clips already on the timeline).
3. **Tracks are fixed in order** — no way to reorder them.

---

## Locked Design Decisions (from user)

| Topic | Decision taken |
|---|---|
| Folder model | **Import same-folder files together** — one "Add Entire Folder" button; files from a folder group under an open/close folder header. No manual bin management. |
| Drag from panel | **Yes** — drag a file from the files panel straight onto a timeline track (Premiere-style); keep the "Put on Timeline" button as fallback. |
| Delete-track guard | **Always keep at least 1 track**; Undo restores a deleted track. |
| Track reorder | **Yes** — drag a track header up/down to reorder rows. |

---

## Proposed Changes

### 1. Delete Tracks (`src/ui/timeline_view.rs` + `src/app.rs`)

#### [MODIFY] `src/ui/timeline_view.rs`
- Add `DeleteTrack(u64)` to `TimelineAction`.
- In the **track header column** (left fixed column, the loop over `timeline.tracks`), add a small but clear `🗑` button (with `on_hover_text("Remove this whole row")`) next to the track name. Click deletes immediately — Undo covers mistakes.
- Add a `Remove this Row` item to the track-header context menu for discoverability.
- When only one track remains, ignore delete (guard lives in the model handler).

#### [MODIFY] `src/app.rs` (in the `TimelineAction` match)
```rust
TimelineAction::DeleteTrack(id) => {
    if self.project.timeline.tracks.len() > 1 {
        self.snapshot_timeline();
        self.project.timeline.remove_track(id);
        self.refresh_preview_frame(Some(ctx));
    }
}
```
- Keeps ≥1 track (guard), and `snapshot_timeline` makes Undo restore the deleted track.

### 2. Folder Bins — "Import Entire Folder" + grouped list (`src/ui/media_bin.rs`, `src/core/project.rs`, `src/app.rs`, `src/media/probe.rs`)

#### [MODIFY] `src/media/probe.rs`
- Add `scan_folder_for_media(path) -> Vec<PathBuf>` that walks a chosen folder and returns every path matching `SUPPORTED_VIDEO/AUDIO/IMAGE_EXTENSIONS` (reuse existing constant lists).

#### [MODIFY] `src/core/project.rs`
- Add a helper `add_imported_folder(&mut self, path, files)` (or reuse `import_file` per file) that imports each qualifying file **with dedup by `asset.path`** so re-importing a folder never duplicates entries.
- Keep `media_assets: Vec<MediaAsset>` flat for serialization compatibility (`.vproj` unchanged) — **folder grouping is derived from each asset's parent directory** at render time, so old project files still load.

#### [MODIFY] `src/ui/media_bin.rs`
- Add `ImportFolder(PathBuf)` to `MediaBinAction`.
- Add a second big button **`+ Add Entire Folder`** (below the existing `+ Add Video / Music`), opens `rfd::FileDialog::new().pick_folder()`.
- Render the flat list **grouped by folder**: each distinct parent-directory name becomes a big toggleable folder header (`📁 FolderName` with a `▸/▾` chevron). Files under that folder appear indented beneath it. A non-folder file (no parent grouping) shows as before.
- Track open/close state in app via a persistent `media_bin_collapsed: HashSet<String>` (folder-name keys) so the user's toggles survive frames and project loads.

#### [MODIFY] `src/app.rs`
- Store `pub media_bin_collapsed: HashSet<String>`; pass `&mut` into `MediaBinView::render`.
- Handle `ImportFolder(path)`: `scan_folder_for_media` → dedup-import each → `refresh_preview_frame`.

### 3. Drag a File from Panel onto a Timeline Track (`src/ui/media_bin.rs`, `src/ui/timeline_view.rs`, `src/app.rs`)

- In `media_bin.rs`, wrap each asset row (or the file-name label) in an **egui drag source** using the `DragAndDrop` helper (`dnd_drag_source`), carrying a cheap payload = the **asset id** (`asset.id`). Payload via `ui.ctx().data_mut(|d| d.insert_temp(...))`.
- In `timeline_view.rs`, over each track's clip canvas, if `DragAndDrop::has_any_payload()` of the media-asset type, draw a subtle **"Drop here" highlight** over the hovered track. On drop (pointer released over a track), resolve the mouse x → `TimeCode`, and emit a new action:
  ```
  AddMediaToTimeline { asset_id, track_id, start: TimeCode }
  ```
- In `app.rs`: look up the asset, place it on the requested track at that time (reuse `add_asset_to_timeline` logic generalized to accept `(asset, track_id, start)`), `snapshot_timeline`, refresh. Auto-fallback:
  - Audio asset → nearest Audio track if dropped on a Video track (and vice-versa), else create one (reuse existing logic in `add_asset_to_timeline`).

### 4. Drag to Reorder Tracks (`src/ui/timeline_view.rs`, `src/core/timeline.rs`, `src/app.rs`)

- [MODIFY] `src/core/timeline.rs`: add `reorder_track(&mut self, from_id: u64, to_index: usize)`.
- [MODIFY] `src/ui/timeline_view.rs`: make each **track header** a drag source (`dnd_drag_source`) with payload = `track.id`; when the dragged id is hovered over another track's header, emit `ReorderTrack { from_id, to_index }`.
- [MODIFY] `src/app.rs`: `snapshot_timeline()` then call `reorder_track`; refresh.

### 5. Senior-Friendly Wording & Plain Labels
- Keep the already-simplified app language (it's in good shape). Concretely:
  - New buttons read plainly: `🗑 Delete Track` header button hover = *"Remove this whole row"*; `+ Add Entire Folder` = *"Bring in all videos & music from a folder"*.
  - Empty-folder / empty-bin helper text stays plain-English.
- No technical words (no "track", "bin", "layer", "decode") in visible UI copy; these names exist only in code.

---

## Open Questions / Notes
- **Drag payload type:** using the asset's numeric `id` as payload keeps serialization and cross-panel ownership clean; the receiving timeline resolves the id via `project.media_assets`.
- **Folder depth:** `scan_folder_for_media` will scan **one level deep** (the chosen folder's files); sub-subfolders are flattened into the same group unless the user picks them. Keeps the UI simple for seniors. *(Flagged for confirmation.)*
- Deleting a track that holds clips deletes those clips too — recoverable via Undo.

## Verification Plan (Manual + Smoke)
1. `cargo build` (in `/home/lazycat/github/projects/sun/video-editor`).
2. **Delete track:** add a track, delete it, confirm ≥1 track remains and clips vanish; Undo restores it.
3. **Folder import:** `+ Add Entire Folder` → files appear grouped under one folder header; clicking the header collapses/expands.
4. **Drag to timeline:** drag a file from panel onto an empty area of a track → clip appears centered on drop x; drag an audio file onto a video track → lands on the audio track.
5. **Track reorder:** drag a track header above/below another → rows swap; clips stay with their row.
6. `cargo test` added for `reorder_track`, dedup import, and keep-≥1-track invariants.
