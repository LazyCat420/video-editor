use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MediaMetadata {
    pub duration_secs: f64,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub has_video: bool,
    pub has_audio: bool,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
}

pub const SUPPORTED_VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "MP4", "mov", "MOV", "m4v", "M4V", "mkv", "MKV", "avi", "AVI",
    "wmv", "WMV", "webm", "WEBM", "flv", "FLV", "ts", "TS", "mts", "MTS",
    "m2ts", "M2TS", "3gp", "3GP", "mpg", "MPG", "mpeg", "MPEG", "vob", "VOB",
];

pub const SUPPORTED_AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "MP3", "wav", "WAV", "m4a", "M4A", "aac", "AAC", "flac", "FLAC",
    "ogg", "OGG", "wma", "WMA", "opus", "OPUS", "aiff", "AIFF", "alac", "ALAC",
];

pub const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "JPG", "jpeg", "JPEG", "png", "PNG", "webp", "WEBP", "bmp", "BMP",
];

/// True if the file's extension is one of the supported video/audio/image formats.
pub fn is_supported_media(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    SUPPORTED_VIDEO_EXTENSIONS.contains(&ext)
        || SUPPORTED_AUDIO_EXTENSIONS.contains(&ext)
        || SUPPORTED_IMAGE_EXTENSIONS.contains(&ext)
}

/// Walk a folder (recursively) and return every media file it contains.
pub fn scan_folder_for_media(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&d) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if is_supported_media(&path) {
                    out.push(path);
                }
            }
        }
    }
    out
}

/// Returns a pre-configured FileDialog with all supported media formats.
pub fn create_media_file_dialog() -> rfd::FileDialog {
    let mut all_media = Vec::new();
    all_media.extend_from_slice(SUPPORTED_VIDEO_EXTENSIONS);
    all_media.extend_from_slice(SUPPORTED_AUDIO_EXTENSIONS);
    all_media.extend_from_slice(SUPPORTED_IMAGE_EXTENSIONS);

    rfd::FileDialog::new()
        .add_filter("All Media (Videos, Music, Photos)", &all_media)
        .add_filter("Video Files", SUPPORTED_VIDEO_EXTENSIONS)
        .add_filter("Audio / Music Files", SUPPORTED_AUDIO_EXTENSIONS)
        .add_filter("Photos & Images", SUPPORTED_IMAGE_EXTENSIONS)
        .add_filter("All Files (*.*)", &["*"])
}

#[derive(Deserialize)]
struct FFprobeOutput {
    streams: Option<Vec<FFprobeStream>>,
    format: Option<FFprobeFormat>,
}

#[derive(Deserialize)]
struct FFprobeStream {
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    r_frame_rate: Option<String>,
    sample_rate: Option<String>,
    channels: Option<u32>,
    duration: Option<String>,
}

#[derive(Deserialize)]
struct FFprobeFormat {
    duration: Option<String>,
}

/// Inspects a media file using ffprobe and extracts technical metadata.
pub fn probe_media_file<P: AsRef<Path>>(path: P) -> Result<MediaMetadata, String> {
    let path_str = path
        .as_ref()
        .to_str()
        .ok_or_else(|| "Invalid file path".to_string())?;

    let ffprobe_bin = crate::media::frame_cache::find_ffprobe_executable();
    let output = Command::new(&ffprobe_bin)
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration:stream=codec_type,codec_name,width,height,r_frame_rate,sample_rate,channels,duration",
            "-of",
            "json",
            path_str,
        ])
        .output()
        .map_err(|e| format!("Failed to execute ffprobe: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffprobe error: {}", err));
    }

    let parsed: FFprobeOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse ffprobe json output: {}", e))?;

    let mut meta = MediaMetadata::default();

    // 1. Duration from format header
    if let Some(ref format) = parsed.format {
        if let Some(ref dur_str) = format.duration {
            if let Ok(dur) = dur_str.parse::<f64>() {
                meta.duration_secs = dur;
            }
        }
    }

    // 2. Streams inspection
    if let Some(streams) = parsed.streams {
        for stream in streams {
            if let Some(ref codec_type) = stream.codec_type {
                match codec_type.as_str() {
                    "video" => {
                        meta.has_video = true;
                        meta.video_codec = stream.codec_name;
                        meta.width = stream.width.unwrap_or(0);
                        meta.height = stream.height.unwrap_or(0);

                        if let Some(ref r_rate) = stream.r_frame_rate {
                            meta.fps = parse_fraction(r_rate).unwrap_or(30.0);
                        }

                        if meta.duration_secs == 0.0 {
                            if let Some(ref dur_str) = stream.duration {
                                if let Ok(dur) = dur_str.parse::<f64>() {
                                    meta.duration_secs = dur;
                                }
                            }
                        }
                    }
                    "audio" => {
                        meta.has_audio = true;
                        meta.audio_codec = stream.codec_name;
                        if let Some(ref sr) = stream.sample_rate {
                            meta.sample_rate = sr.parse().ok();
                        }
                        meta.channels = stream.channels;

                        if meta.duration_secs == 0.0 {
                            if let Some(ref dur_str) = stream.duration {
                                if let Ok(dur) = dur_str.parse::<f64>() {
                                    meta.duration_secs = dur;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Fallback if fps was zero
    if meta.fps == 0.0 && meta.has_video {
        meta.fps = 30.0;
    }

    // Fallback if duration was zero (e.g. still photos like JPG/PNG)
    if meta.duration_secs <= 0.01 {
        meta.duration_secs = 5.0;
    }

    Ok(meta)
}

fn parse_fraction(frac: &str) -> Option<f64> {
    let parts: Vec<&str> = frac.split('/').collect();
    if parts.len() == 2 {
        let num: f64 = parts[0].parse().ok()?;
        let den: f64 = parts[1].parse().ok()?;
        if den != 0.0 {
            return Some(num / den);
        }
    } else if parts.len() == 1 {
        return parts[0].parse().ok();
    }
    None
}
