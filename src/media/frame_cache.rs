use egui::{ColorImage, Context};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

pub use crate::media::ffmpeg_locator::{find_ffmpeg_executable, find_ffprobe_executable};

/// Helper to configure Command with no console window on Windows.
#[cfg(target_os = "windows")]
fn configure_command(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(target_os = "windows"))]
fn configure_command(_cmd: &mut Command) {}

/// Bounded LRU-style frame cache for real-time video preview with async worker and repaint trigger.
pub struct FrameCache {
    cache: Arc<Mutex<HashMap<(PathBuf, i64), (ColorImage, u64)>>>,
    access_counter: Arc<Mutex<u64>>,
    pending_requests: Arc<Mutex<HashMap<(PathBuf, i64), bool>>>,
    active_workers: Arc<AtomicUsize>,
    max_frames: usize,
    ffmpeg_bin: PathBuf,
}

impl Default for FrameCache {
    fn default() -> Self {
        Self::new(120) // 120 frames @ 360p ~= 100MB RAM
    }
}

impl FrameCache {
    pub fn new(max_frames: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            access_counter: Arc::new(Mutex::new(0)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            active_workers: Arc::new(AtomicUsize::new(0)),
            max_frames,
            ffmpeg_bin: find_ffmpeg_executable(),
        }
    }

    /// Retrieve a frame if cached in memory (bucketed to 100ms with neighbor tolerance).
    pub fn get_cached<P: AsRef<Path>>(&self, path: P, time_secs: f64) -> Option<ColorImage> {
        let target_bucket = (time_secs * 10.0).round() as i64;
        let key = (path.as_ref().to_path_buf(), target_bucket);
        let mut lock = self.cache.lock().ok()?;

        // 1. Exact match
        if let Some((img, counter)) = lock.get_mut(&key) {
            let mut access = self.access_counter.lock().ok()?;
            *access += 1;
            *counter = *access;
            return Some(img.clone());
        }

        // 2. Nearest neighbor match within ±500ms (5 buckets) for instant playback continuity
        let path_buf = path.as_ref().to_path_buf();
        let mut closest = None;
        let mut min_diff = i64::MAX;

        for (k, (img, _)) in lock.iter() {
            if k.0 == path_buf {
                let diff = (k.1 - target_bucket).abs();
                if diff < min_diff && diff <= 5 {
                    min_diff = diff;
                    closest = Some(img.clone());
                }
            }
        }

        closest
    }

    /// Fetch a frame. Returns immediately from cache or queues background extraction with UI repaint callback.
    pub fn fetch_frame<P: AsRef<Path>>(
        &self,
        path: P,
        time_secs: f64,
        ctx: Option<&Context>,
    ) -> Option<ColorImage> {
        let p = path.as_ref().to_path_buf();
        let target_bucket = (time_secs * 10.0).round() as i64;

        // If already cached, return immediately
        if let Some(cached) = self.get_cached(&p, time_secs) {
            // Also prefetch next 300ms in background if not yet in cache
            self.prefetch_ahead(&p, time_secs + 0.3, ctx);
            return Some(cached);
        }

        let key = (p.clone(), target_bucket);

        // Check if already in progress
        if let Ok(mut pending) = self.pending_requests.lock() {
            if *pending.get(&key).unwrap_or(&false) {
                return None;
            }
            pending.insert(key.clone(), true);
        }

        // Spawn background worker
        // Bound concurrent background FFmpeg processes to 2 max to protect CPU
        if self.active_workers.load(Ordering::SeqCst) >= 2 {
            if let Ok(mut pending) = self.pending_requests.lock() {
                pending.remove(&key);
            }
            return None;
        }
        self.active_workers.fetch_add(1, Ordering::SeqCst);

        let cache_arc = Arc::clone(&self.cache);
        let access_arc = Arc::clone(&self.access_counter);
        let pending_arc = Arc::clone(&self.pending_requests);
        let workers_arc = Arc::clone(&self.active_workers);
        let max_cap = self.max_frames;
        let ffmpeg_cmd = self.ffmpeg_bin.clone();
        let ctx_clone = ctx.cloned();

        thread::spawn(move || {
            let ts_str = format!("{:.3}", time_secs.max(0.0));
            let mut cmd = Command::new(&ffmpeg_cmd);
            configure_command(&mut cmd);

            // Accurate output seeking with 360p scaling for maximum speed
            let output_res = cmd
                .args([
                    "-i",
                    p.to_str().unwrap_or_default(),
                    "-ss",
                    &ts_str,
                    "-vframes",
                    "1",
                    "-vf",
                    "scale=-2:360",
                    "-f",
                    "image2pipe",
                    "-vcodec",
                    "mjpeg",
                    "-q:v",
                    "2",
                    "-",
                ])
                .output();

            if let Ok(output) = output_res {
                if output.status.success() && !output.stdout.is_empty() {
                    if let Ok(dyn_img) = image::load_from_memory(&output.stdout) {
                        let rgba = dyn_img.to_rgba8();
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        let color_img = ColorImage::from_rgba_unmultiplied(size, &rgba.into_raw());

                        // Insert into cache
                        if let (Ok(mut lock), Ok(mut access)) = (cache_arc.lock(), access_arc.lock()) {
                            *access += 1;
                            let counter_val = *access;

                            if lock.len() >= max_cap {
                                if let Some((oldest_key, _)) = lock
                                    .iter()
                                    .min_by_key(|(_, (_, counter))| *counter)
                                    .map(|(k, v)| (k.clone(), v.clone()))
                                {
                                    lock.remove(&oldest_key);
                                }
                            }
                            lock.insert(key.clone(), (color_img, counter_val));
                        }

                        // Trigger UI repaint immediately so frame displays on screen
                        if let Some(c) = ctx_clone {
                            c.request_repaint();
                        }
                    }
                }
            }

            workers_arc.fetch_sub(1, Ordering::SeqCst);
            if let Ok(mut pending) = pending_arc.lock() {
                pending.remove(&key);
            }
        });

        None
    }

    /// Prefetch upcoming frames ahead of the playhead.
    fn prefetch_ahead<P: AsRef<Path>>(&self, path: P, time_secs: f64, ctx: Option<&Context>) {
        let p = path.as_ref().to_path_buf();
        let target_bucket = (time_secs * 10.0).round() as i64;
        let key = (p.clone(), target_bucket);

        if let Ok(lock) = self.cache.lock() {
            if lock.contains_key(&key) {
                return;
            }
        }

        if let Ok(mut pending) = self.pending_requests.lock() {
            if *pending.get(&key).unwrap_or(&false) {
                return;
            }
            pending.insert(key.clone(), true);
        }

        if self.active_workers.load(Ordering::SeqCst) >= 2 {
            if let Ok(mut pending) = self.pending_requests.lock() {
                pending.remove(&key);
            }
            return;
        }
        self.active_workers.fetch_add(1, Ordering::SeqCst);

        let cache_arc = Arc::clone(&self.cache);
        let access_arc = Arc::clone(&self.access_counter);
        let pending_arc = Arc::clone(&self.pending_requests);
        let workers_arc = Arc::clone(&self.active_workers);
        let max_cap = self.max_frames;
        let ffmpeg_cmd = self.ffmpeg_bin.clone();
        let ctx_clone = ctx.cloned();

        thread::spawn(move || {
            let ts_str = format!("{:.3}", time_secs.max(0.0));
            let mut cmd = Command::new(&ffmpeg_cmd);
            configure_command(&mut cmd);

            let output_res = cmd
                .args([
                    "-i",
                    p.to_str().unwrap_or_default(),
                    "-ss",
                    &ts_str,
                    "-vframes",
                    "1",
                    "-vf",
                    "scale=-2:360",
                    "-f",
                    "image2pipe",
                    "-vcodec",
                    "mjpeg",
                    "-q:v",
                    "2",
                    "-",
                ])
                .output();

            if let Ok(output) = output_res {
                if output.status.success() && !output.stdout.is_empty() {
                    if let Ok(dyn_img) = image::load_from_memory(&output.stdout) {
                        let rgba = dyn_img.to_rgba8();
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        let color_img = ColorImage::from_rgba_unmultiplied(size, &rgba.into_raw());

                        if let (Ok(mut lock), Ok(mut access)) = (cache_arc.lock(), access_arc.lock()) {
                            *access += 1;
                            let counter_val = *access;

                            if lock.len() >= max_cap {
                                if let Some((oldest_key, _)) = lock
                                    .iter()
                                    .min_by_key(|(_, (_, counter))| *counter)
                                    .map(|(k, v)| (k.clone(), v.clone()))
                                {
                                    lock.remove(&oldest_key);
                                }
                            }
                            lock.insert(key.clone(), (color_img, counter_val));
                        }

                        if let Some(c) = ctx_clone {
                            c.request_repaint();
                        }
                    }
                }
            }

            workers_arc.fetch_sub(1, Ordering::SeqCst);
            if let Ok(mut pending) = pending_arc.lock() {
                pending.remove(&key);
            }
        });
    }

    /// Synchronously extract frame 0.0s on import so preview is immediately ready.
    pub fn extract_initial_frame<P: AsRef<Path>>(&self, path: P) -> Option<ColorImage> {
        let p = path.as_ref().to_path_buf();
        let mut cmd = Command::new(&self.ffmpeg_bin);
        configure_command(&mut cmd);

        let output = cmd
            .args([
                "-i",
                p.to_str().unwrap_or_default(),
                "-ss",
                "0.000",
                "-vframes",
                "1",
                "-vf",
                "scale=-2:360",
                "-f",
                "image2pipe",
                "-vcodec",
                "mjpeg",
                "-q:v",
                "2",
                "-",
            ])
            .output()
            .ok()?;

        if output.status.success() && !output.stdout.is_empty() {
            if let Ok(dyn_img) = image::load_from_memory(&output.stdout) {
                let rgba = dyn_img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let color_img = ColorImage::from_rgba_unmultiplied(size, &rgba.into_raw());

                self.insert(&p, 0.0, color_img.clone());
                return Some(color_img);
            }
        }
        None
    }

    pub fn insert<P: AsRef<Path>>(&self, path: P, time_secs: f64, img: ColorImage) {
        let key = (path.as_ref().to_path_buf(), (time_secs * 10.0).round() as i64);
        let mut access = match self.access_counter.lock() {
            Ok(a) => a,
            Err(_) => return,
        };
        *access += 1;
        let counter_val = *access;

        let mut lock = match self.cache.lock() {
            Ok(l) => l,
            Err(_) => return,
        };

        if lock.len() >= self.max_frames {
            if let Some((oldest_key, _)) = lock
                .iter()
                .min_by_key(|(_, (_, counter))| *counter)
                .map(|(k, v)| (k.clone(), v.clone()))
            {
                lock.remove(&oldest_key);
            }
        }

        lock.insert(key, (img, counter_val));
    }

    pub fn clear(&self) {
        if let Ok(mut lock) = self.cache.lock() {
            lock.clear();
        }
    }
}
