use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

/// Precomputed waveform peaks for instantaneous timeline rendering without scanning raw audio.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WaveformPeaks {
    /// Pairs of [min_sample, max_sample] normalized to [-1.0, 1.0].
    pub peaks: Vec<[f32; 2]>,
    /// Number of peak pairs per second (typically 100).
    pub points_per_sec: f32,
    pub total_duration_secs: f64,
}

impl WaveformPeaks {
    pub fn get_peak_at_sec(&self, sec: f64) -> [f32; 2] {
        if self.peaks.is_empty() || sec < 0.0 {
            return [0.0, 0.0];
        }
        let idx = (sec * self.points_per_sec as f64).round() as usize;
        if idx >= self.peaks.len() {
            return [0.0, 0.0];
        }
        self.peaks[idx]
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        let header = b"PEAKv001";
        file.write_all(header)?;
        file.write_all(&self.points_per_sec.to_le_bytes())?;
        file.write_all(&self.total_duration_secs.to_le_bytes())?;
        file.write_all(&(self.peaks.len() as u32).to_le_bytes())?;

        for [min, max] in &self.peaks {
            file.write_all(&min.to_le_bytes())?;
            file.write_all(&max.to_le_bytes())?;
        }
        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut header = [0u8; 8];
        file.read_exact(&mut header)?;
        if &header != b"PEAKv001" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid peak file header",
            ));
        }

        let mut buf4 = [0u8; 4];
        let mut buf8 = [0u8; 8];

        file.read_exact(&mut buf4)?;
        let pps = f32::from_le_bytes(buf4);

        file.read_exact(&mut buf8)?;
        let dur = f64::from_le_bytes(buf8);

        file.read_exact(&mut buf4)?;
        let count = u32::from_le_bytes(buf4) as usize;

        let mut peaks = Vec::with_capacity(count);
        for _ in 0..count {
            file.read_exact(&mut buf4)?;
            let min = f32::from_le_bytes(buf4);
            file.read_exact(&mut buf4)?;
            let max = f32::from_le_bytes(buf4);
            peaks.push([min, max]);
        }

        Ok(Self {
            peaks,
            points_per_sec: pps,
            total_duration_secs: dur,
        })
    }
}

/// Extract waveform peaks from an audio or video file using FFmpeg.
pub fn extract_peaks<P: AsRef<Path>>(
    media_path: P,
    duration_secs: f64,
) -> Result<WaveformPeaks, String> {
    let path = media_path.as_ref();
    let cache_dir = std::env::temp_dir().join("video_editor_peaks");
    let _ = std::fs::create_dir_all(&cache_dir);

    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("audio");
    let peak_file = cache_dir.join(format!("{}.peaks", stem));

    // Try loading existing cached peaks
    if let Ok(cached) = WaveformPeaks::load_from_file(&peak_file) {
        if (cached.total_duration_secs - duration_secs).abs() < 1.0 && !cached.peaks.is_empty() {
            return Ok(cached);
        }
    }

    let target_sample_rate = 8000;
    let points_per_sec = 100.0f32;
    let samples_per_bucket = (target_sample_rate as f32 / points_per_sec).round() as usize;

    let ffmpeg_bin = crate::media::frame_cache::find_ffmpeg_executable();
    let mut child = Command::new(&ffmpeg_bin)
        .args([
            "-v",
            "error",
            "-i",
            path.to_str().unwrap_or_default(),
            "-vn",
            "-ac",
            "1",
            "-ar",
            &target_sample_rate.to_string(),
            "-f",
            "f32le",
            "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn ffmpeg for peak extraction: {}", e))?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to capture ffmpeg stdout".to_string())?;

    let mut peaks = Vec::new();
    let mut raw_buf = [0u8; 4096];
    let mut current_min = 0.0f32;
    let mut current_max = 0.0f32;
    let mut bucket_count = 0;

    loop {
        let bytes_read = match stdout.read(&mut raw_buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => return Err(format!("Error reading ffmpeg audio stream: {}", e)),
        };

        let sample_count = bytes_read / 4;
        for i in 0..sample_count {
            let offset = i * 4;
            let sample = f32::from_le_bytes([
                raw_buf[offset],
                raw_buf[offset + 1],
                raw_buf[offset + 2],
                raw_buf[offset + 3],
            ]);

            current_min = current_min.min(sample);
            current_max = current_max.max(sample);
            bucket_count += 1;

            if bucket_count >= samples_per_bucket {
                peaks.push([current_min.clamp(-1.0, 1.0), current_max.clamp(-1.0, 1.0)]);
                current_min = 0.0;
                current_max = 0.0;
                bucket_count = 0;
            }
        }
    }

    let _ = child.wait();

    let waveform = WaveformPeaks {
        peaks,
        points_per_sec,
        total_duration_secs: duration_secs,
    };

    let _ = waveform.save_to_file(&peak_file);

    Ok(waveform)
}
