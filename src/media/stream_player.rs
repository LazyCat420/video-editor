use egui::{ColorImage, Context};
use std::collections::VecDeque;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::media::frame_cache::find_ffmpeg_executable;

pub const STREAM_WIDTH: usize = 640;
pub const STREAM_HEIGHT: usize = 360;
pub const STREAM_BYTES_PER_FRAME: usize = STREAM_WIDTH * STREAM_HEIGHT * 3;

/// Continuous video playback decoder using a single lightweight FFmpeg rawvideo stream.
pub struct StreamVideoPlayer {
    buffer: Arc<Mutex<VecDeque<(f64, ColorImage)>>>,
    is_running: Arc<AtomicBool>,
    active_path: Option<PathBuf>,
    child_process: Option<Child>,
    pub last_error: Arc<Mutex<Option<String>>>,
    ffmpeg_bin: PathBuf,
    current_frame: Option<ColorImage>,
    last_pts: Option<f64>,
}

impl Default for StreamVideoPlayer {
    fn default() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(30))),
            is_running: Arc::new(AtomicBool::new(false)),
            active_path: None,
            child_process: None,
            last_error: Arc::new(Mutex::new(None)),
            ffmpeg_bin: find_ffmpeg_executable(),
            current_frame: None,
            last_pts: None,
        }
    }
}

impl StreamVideoPlayer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the currently running stream already covers the given file and timestamp seamlessly.
    pub fn is_continuous_with(&self, path: &Path, source_time: f64) -> bool {
        if !self.is_running.load(Ordering::SeqCst) {
            return false;
        }
        if self.active_path.as_deref() != Some(path) {
            return false;
        }
        if let Some(last) = self.last_pts {
            (source_time - last).abs() < 0.35
        } else {
            false
        }
    }

    /// Start streaming decoded 30 FPS video frames from `start_secs` with optional segment duration.
    pub fn start<P: AsRef<Path>>(
        &mut self,
        path: P,
        start_secs: f64,
        duration_secs: Option<f64>,
        ctx: Option<&Context>,
    ) {
        self.stop();

        let path_buf = path.as_ref().to_path_buf();
        self.active_path = Some(path_buf.clone());
        self.is_running.store(true, Ordering::SeqCst);
        self.current_frame = None;
        self.last_pts = Some(start_secs);

        let mut err_lock = self.last_error.lock().unwrap();
        *err_lock = None;
        drop(err_lock);

        let buffer_arc = Arc::clone(&self.buffer);
        let is_running_arc = Arc::clone(&self.is_running);
        let err_arc = Arc::clone(&self.last_error);
        let ffmpeg_bin = self.ffmpeg_bin.clone();
        let ctx_clone = ctx.cloned();

        let ts_str = format!("{:.3}", start_secs.max(0.0));
        let vf_filter = format!(
            "scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2:color=black,fps=30",
            STREAM_WIDTH, STREAM_HEIGHT, STREAM_WIDTH, STREAM_HEIGHT
        );

        let mut cmd = Command::new(&ffmpeg_bin);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }

        let mut args = vec![
            "-ss".to_string(),
            ts_str,
            "-i".to_string(),
            path_buf.to_str().unwrap_or_default().to_string(),
        ];

        if let Some(dur) = duration_secs {
            if dur > 0.0 {
                args.push("-t".to_string());
                args.push(format!("{:.3}", dur));
            }
        }

        args.extend([
            "-vf".to_string(),
            vf_filter,
            "-f".to_string(),
            "rawvideo".to_string(),
            "-pix_fmt".to_string(),
            "rgb24".to_string(),
            "-v".to_string(),
            "error".to_string(),
            "-".to_string(),
        ]);

        cmd.args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let msg = format!("Failed to start FFmpeg playback stream: {}", e);
                if let Ok(mut l) = err_arc.lock() {
                    *l = Some(msg);
                }
                self.is_running.store(false, Ordering::SeqCst);
                return;
            }
        };

        let mut stdout = child.stdout.take().expect("Failed to open stdout pipe");

        // Spawn background reader thread with producer backpressure and PTS tagging
        thread::spawn(move || {
            let mut raw_buf = vec![0u8; STREAM_BYTES_PER_FRAME];
            let mut frame_idx: u64 = 0;
            let frame_duration = 1.0 / 30.0;

            while is_running_arc.load(Ordering::SeqCst) {
                // Backpressure: pause reader when lookahead buffer has 30 frames
                if let Ok(buf) = buffer_arc.lock() {
                    if buf.len() >= 30 {
                        drop(buf);
                        thread::sleep(std::time::Duration::from_millis(15));
                        continue;
                    }
                }

                match stdout.read_exact(&mut raw_buf) {
                    Ok(_) => {
                        let pts = start_secs + (frame_idx as f64 * frame_duration);
                        frame_idx += 1;

                        let color_img = ColorImage::from_rgb(
                            [STREAM_WIDTH, STREAM_HEIGHT],
                            &raw_buf,
                        );

                        if let Ok(mut buf) = buffer_arc.lock() {
                            buf.push_back((pts, color_img));
                        }

                        if let Some(ref c) = ctx_clone {
                            c.request_repaint();
                        }
                    }
                    Err(_) => {
                        // EOF or stream broken
                        break;
                    }
                }
            }

            is_running_arc.store(false, Ordering::SeqCst);
        });

        self.child_process = Some(child);
    }

    /// Retrieve the video frame corresponding to the current source playback time.
    /// Pops all frames whose PTS <= current_source_time, returning the most up-to-date frame.
    pub fn get_frame_for_time(&mut self, current_source_time: f64) -> Option<ColorImage> {
        if let Ok(mut buf) = self.buffer.lock() {
            let mut advanced = false;
            while let Some((pts, _)) = buf.front() {
                if *pts <= current_source_time {
                    let (pts_val, frame) = buf.pop_front().unwrap();
                    self.current_frame = Some(frame);
                    self.last_pts = Some(pts_val);
                    advanced = true;
                } else {
                    break;
                }
            }
            if advanced {
                return self.current_frame.clone();
            }
        }
        self.current_frame.clone()
    }

    /// Stop the continuous video playback stream.
    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);

        if let Some(mut child) = self.child_process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }

        if let Ok(mut buf) = self.buffer.lock() {
            buf.clear();
        }

        self.active_path = None;
        self.current_frame = None;
        self.last_pts = None;
    }
}

impl Drop for StreamVideoPlayer {
    fn drop(&mut self) {
        self.stop();
    }
}
