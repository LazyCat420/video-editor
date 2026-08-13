use egui::ColorImage;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

/// Bounded LRU-style frame cache for real-time video preview.
pub struct FrameCache {
    cache: Arc<Mutex<HashMap<(PathBuf, i64), (ColorImage, u64)>>>,
    access_counter: Arc<Mutex<u64>>,
    max_frames: usize,
}

impl Default for FrameCache {
    fn default() -> Self {
        Self::new(60) // 60 frames @ 360p ~= 55MB RAM
    }
}

impl FrameCache {
    pub fn new(max_frames: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            access_counter: Arc::new(Mutex::new(0)),
            max_frames,
        }
    }

    /// Retrieve a frame if cached in memory.
    pub fn get_cached<P: AsRef<Path>>(&self, path: P, time_secs: f64) -> Option<ColorImage> {
        let key = (path.as_ref().to_path_buf(), (time_secs * 20.0).round() as i64); // 50ms bucket
        let mut lock = self.cache.lock().ok()?;
        if let Some((img, counter)) = lock.get_mut(&key) {
            let mut access = self.access_counter.lock().ok()?;
            *access += 1;
            *counter = *access;
            return Some(img.clone());
        }
        None
    }

    /// Fetch and cache a frame synchronously or from fast proxy.
    pub fn fetch_frame<P: AsRef<Path>>(&self, path: P, time_secs: f64) -> Option<ColorImage> {
        let p = path.as_ref();
        if let Some(cached) = self.get_cached(p, time_secs) {
            return Some(cached);
        }

        let ts_str = format!("{:.3}", time_secs.max(0.0));
        let output = Command::new("ffmpeg")
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
                "3",
                "-",
            ])
            .output()
            .ok()?;

        if !output.status.success() || output.stdout.is_empty() {
            return None;
        }

        let dyn_img = image::load_from_memory(&output.stdout).ok()?;
        let rgba = dyn_img.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let color_img = ColorImage::from_rgba_unmultiplied(size, &rgba.into_raw());

        self.insert(p, time_secs, color_img.clone());
        Some(color_img)
    }

    pub fn insert<P: AsRef<Path>>(&self, path: P, time_secs: f64, img: ColorImage) {
        let key = (path.as_ref().to_path_buf(), (time_secs * 20.0).round() as i64);
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

        // Evict oldest if exceeding max capacity
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
