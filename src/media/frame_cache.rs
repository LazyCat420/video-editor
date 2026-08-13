use egui::ColorImage;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

/// Find ffmpeg executable path on Windows / Linux / WSL.
pub fn find_ffmpeg_executable() -> PathBuf {
    // 1. Check if ffmpeg.exe exists in current directory or ./ffmpeg/bin
    let local_paths = [
        PathBuf::from("ffmpeg.exe"),
        PathBuf::from("ffmpeg"),
        PathBuf::from("bin/ffmpeg.exe"),
        PathBuf::from("ffmpeg/bin/ffmpeg.exe"),
        PathBuf::from("C:/ffmpeg/bin/ffmpeg.exe"),
    ];

    for p in &local_paths {
        if p.exists() {
            return p.clone();
        }
    }

    // 2. Default to PATH resolution
    if cfg!(target_os = "windows") {
        PathBuf::from("ffmpeg.exe")
    } else {
        PathBuf::from("ffmpeg")
    }
}

/// Bounded LRU-style frame cache for real-time video preview with async worker.
pub struct FrameCache {
    cache: Arc<Mutex<HashMap<(PathBuf, i64), (ColorImage, u64)>>>,
    access_counter: Arc<Mutex<u64>>,
    pending_requests: Arc<Mutex<HashMap<(PathBuf, i64), bool>>>,
    max_frames: usize,
    ffmpeg_bin: PathBuf,
}

impl Default for FrameCache {
    fn default() -> Self {
        Self::new(100) // 100 frames @ 360p ~= 90MB RAM
    }
}

impl FrameCache {
    pub fn new(max_frames: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            access_counter: Arc::new(Mutex::new(0)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            max_frames,
            ffmpeg_bin: find_ffmpeg_executable(),
        }
    }

    /// Retrieve a frame if cached in memory.
    pub fn get_cached<P: AsRef<Path>>(&self, path: P, time_secs: f64) -> Option<ColorImage> {
        let key = (path.as_ref().to_path_buf(), (time_secs * 10.0).round() as i64); // 100ms bucket
        let mut lock = self.cache.lock().ok()?;
        if let Some((img, counter)) = lock.get_mut(&key) {
            let mut access = self.access_counter.lock().ok()?;
            *access += 1;
            *counter = *access;
            return Some(img.clone());
        }

        // If exact key is not found, check closest neighbor within ±300ms for instant scrub response
        let target_bucket = (time_secs * 10.0).round() as i64;
        let mut closest = None;
        let mut min_diff = i64::MAX;
        let path_buf = path.as_ref().to_path_buf();

        for (k, (img, _)) in lock.iter() {
            if k.0 == path_buf {
                let diff = (k.1 - target_bucket).abs();
                if diff < min_diff && diff <= 3 {
                    min_diff = diff;
                    closest = Some(img.clone());
                }
            }
        }

        closest
    }

    /// Fetch and cache a frame. Returns immediately from cache or queues background extraction.
    pub fn fetch_frame<P: AsRef<Path>>(&self, path: P, time_secs: f64) -> Option<ColorImage> {
        let p = path.as_ref().to_path_buf();
        if let Some(cached) = self.get_cached(&p, time_secs) {
            return Some(cached);
        }

        let key = (p.clone(), (time_secs * 10.0).round() as i64);

        // Check if already being fetched in background
        if let Ok(mut pending) = self.pending_requests.lock() {
            if *pending.get(&key).unwrap_or(&false) {
                return None;
            }
            pending.insert(key.clone(), true);
        }

        // Spawn background worker to extract frame without freezing UI
        let cache_arc = Arc::clone(&self.cache);
        let access_arc = Arc::clone(&self.access_counter);
        let pending_arc = Arc::clone(&self.pending_requests);
        let max_cap = self.max_frames;
        let ffmpeg_cmd = self.ffmpeg_bin.clone();

        thread::spawn(move || {
            let ts_str = format!("{:.3}", time_secs.max(0.0));
            let output_res = Command::new(&ffmpeg_cmd)
                .args([
                    "-ss",
                    &ts_str,
                    "-i",
                    p.to_str().unwrap_or_default(),
                    "-vframes",
                    "1",
                    "-f",
                    "image2pipe",
                    "-vcodec",
                    "jpeg",
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
                    }
                }
            }

            if let Ok(mut pending) = pending_arc.lock() {
                pending.remove(&key);
            }
        });

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
