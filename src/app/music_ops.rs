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

    /// Send every not-yet-probed file to a worker thread. Returns true when a
    /// batch was queued (the caller must return and wait for pump_import_queue
    /// to replay it) and false when everything is already known — the caller
    /// can proceed synchronously with zero ffprobe cost.
    pub(crate) fn queue_unprobed_files(&mut self, paths: &[PathBuf], x: f32, y: f32) -> bool {
        let mut unprobed: Vec<PathBuf> = Vec::new();
        for p in paths {
            if self.probe_cache.contains_key(p)
                || self.project.media_assets.iter().any(|a| &a.path == p)
            {
                continue;
            }
            if p.exists() {
                unprobed.push(p.clone());
            } else {
                // A missing file needs no worker: fail it instantly so the
                // batch stays synchronous when nothing real needs probing.
                self.probe_cache.insert(
                    p.clone(),
                    Err("that file can't be found — was it moved or deleted?".to_string()),
                );
            }
        }
        if unprobed.is_empty() {
            return false;
        }
        if self.pending_import.is_some() {
            self.show_error("Still adding the last files — one moment, then try again.");
            return true;
        }
        let (tx, rx) = crossbeam_channel::unbounded();
        let total = unprobed.len();
        std::thread::spawn(move || {
            for p in unprobed {
                let res = crate::media::probe::probe_media_file(&p);
                if tx.send((p, res)).is_err() {
                    return; // app dropped the batch; stop probing
                }
            }
        });
        self.pending_import = Some(crate::app::PendingImport {
            rx,
            total,
            done: 0,
            paths: paths.to_vec(),
            x,
            y,
        });
        true
    }

    /// Called every frame: collect finished probes, and when the whole batch is
    /// in, replay the original drop with a hot cache (so it applies instantly).
    pub fn pump_import_queue(&mut self, ctx: Option<&Context>) {
        let mut finished: Option<(Vec<PathBuf>, f32, f32)> = None;
        if let Some(pending) = &mut self.pending_import {
            while let Ok((p, res)) = pending.rx.try_recv() {
                self.probe_cache.insert(p, res);
                pending.done += 1;
            }
            if pending.done >= pending.total {
                finished = Some((std::mem::take(&mut pending.paths), pending.x, pending.y));
            }
        }
        if let Some((paths, x, y)) = finished {
            self.pending_import = None;
            self.drop_files_on_canvas(paths, x, y, ctx);
            // The batch has been consumed into assets/clips; don't let stale
            // probe results shadow a file the user later replaces on disk.
            self.probe_cache.clear();
        }
    }
}
