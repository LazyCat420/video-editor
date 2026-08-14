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
    buffer: Arc<Mutex<VecDeque<ColorImage>>>,
    is_running: Arc<AtomicBool>,
    active_path: Option<PathBuf>,
    child_process: Option<Child>,
    pub last_error: Arc<Mutex<Option<String>>>,
    ffmpeg_bin: PathBuf,
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
        }
    }
}

impl StreamVideoPlayer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start streaming decoded 30 FPS video frames from `start_secs`.
    pub fn start<P: AsRef<Path>>(&mut self, path: P, start_secs: f64, ctx: Option<&Context>) {
        self.stop();

        let path_buf = path.as_ref().to_path_buf();
        self.active_path = Some(path_buf.clone());
        self.is_running.store(true, Ordering::SeqCst);

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

        cmd.args([
            "-ss",
            &ts_str,
            "-i",
            path_buf.to_str().unwrap_or_default(),
            "-vf",
            &vf_filter,
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "-v",
            "error",
            "-",
        ])
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

        // Spawn background reader thread
        thread::spawn(move || {
            let mut raw_buf = vec![0u8; STREAM_BYTES_PER_FRAME];

            while is_running_arc.load(Ordering::SeqCst) {
                match stdout.read_exact(&mut raw_buf) {
                    Ok(_) => {
                        let color_img = ColorImage::from_rgb(
                            [STREAM_WIDTH, STREAM_HEIGHT],
                            &raw_buf,
                        );

                        if let Ok(mut buf) = buffer_arc.lock() {
                            // Maintain a 30-frame buffer (~1 second lookahead)
                            if buf.len() >= 30 {
                                buf.pop_front();
                            }
                            buf.push_back(color_img);
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

    /// Retrieve the next decoded video frame from the stream buffer.
    pub fn get_next_frame(&self) -> Option<ColorImage> {
        if let Ok(mut buf) = self.buffer.lock() {
            if buf.len() > 1 {
                return buf.pop_front();
            } else if let Some(f) = buf.front() {
                return Some(f.clone());
            }
        }
        None
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
    }

    pub fn is_active(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}
