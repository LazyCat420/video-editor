use crate::app::VideoEditorApp;
use crate::core::time::TimeCode;
use egui::Context;
use std::path::{Path, PathBuf};

impl VideoEditorApp {
    /// Append audio files to the music track end-to-end: probe (with a visible
    /// error on failure), register the asset (feeds the Timeline editor's
    /// waveform machinery), and land the song right after the last one so songs
    /// never overlap. One undo snapshot covers the whole batch.
    pub fn add_music_files(&mut self, paths: Vec<PathBuf>, ctx: Option<&Context>) {
        if paths.is_empty() {
            return;
        }
        self.snapshot_timeline();
        let mut any_added = false;
        for p in paths {
            match self.append_music_clip_inner(&p) {
                Ok(_) => any_added = true,
                Err(e) => self.show_error(e),
            }
        }
        if !any_added {
            self.history.discard_last_snapshot();
        }
        self.refresh_preview_frame(ctx);
    }

    /// No-snapshot worker so batch callers control undo granularity.
    fn append_music_clip_inner(&mut self, path: &Path) -> Result<u64, String> {
        let asset_id = self.add_media_to_bin(path)?;
        let asset = self
            .project
            .media_assets
            .iter()
            .find(|a| a.id == asset_id)
            .cloned()
            .ok_or_else(|| "Something went wrong adding that song.".to_string())?;
        if !asset.has_audio {
            return Err(format!("\"{}\" has no sound in it.", asset.name));
        }
        Ok(self.project.timeline.append_music_clip(
            asset.name,
            asset.path,
            TimeCode::from_secs_f64(asset.duration_secs),
        ))
    }

    /// 🗑 on a music chip: remove the song and close the gap it leaves.
    pub fn remove_music_clip(&mut self, clip_id: u64, ctx: Option<&Context>) {
        self.snapshot_timeline();
        if !self.project.timeline.remove_music_clip(clip_id) {
            self.history.discard_last_snapshot();
        }
        self.refresh_preview_frame(ctx);
    }

    /// Router for "📂 Open Files" and OS drag-drop alike: music goes to the
    /// music track, photos/videos go onto the current slide.
    pub fn import_files(&mut self, paths: Vec<PathBuf>, ctx: Option<&Context>) {
        // drop_files_on_canvas partitions audio out itself; this thin wrapper
        // exists so call sites read as intent ("import whatever was picked").
        self.drop_files_on_canvas(paths, 0.5, 0.5, ctx);
    }
}
