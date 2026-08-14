use egui::{ColorImage, Context};
use std::collections::VecDeque;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;
use std::thread;

use crate::media::frame_cache::find_ffmpeg_executable;

pub const STREAM_WIDTH: usize = 640;
pub const STREAM_HEIGHT: usize = 360;
pub const STREAM_BYTES_PER_FRAME: usize = STREAM_WIDTH * STREAM_HEIGHT * 3;

/// Build the ffmpeg CLI args for a rawvideo stream starting at `start_secs`.
///
/// Deliberately does **not** pass `-t`: a continuous stream must run to end-of-file
/// so that a preserved deck keeps decoding across adjacent timeline cuts of the same
/// source instead of silently EOF-ing at the first sub-clip's boundary (which froze
/// every subsequent cut in the multi-cut lag bug). Decode output is bounded in the
/// consumer by the lookahead buffer backpressure and the explicit `stop()` calls at
/// gaps/jump cuts, not by a producer-side duration cap.
fn build_stream_args(path: &str, start_secs: f64) -> Vec<String> {
    let ts_str = format!("{:.3}", start_secs.max(0.0));
    let vf_filter = format!(
        "scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2:color=black,fps=30",
        STREAM_WIDTH, STREAM_HEIGHT, STREAM_WIDTH, STREAM_HEIGHT
    );
    vec![
        "-ss".to_string(),
        ts_str,
        "-i".to_string(),
        path.to_string(),
        "-vf".to_string(),
        vf_filter,
        "-f".to_string(),
        "rawvideo".to_string(),
        "-pix_fmt".to_string(),
        "rgb24".to_string(),
        "-v".to_string(),
        "error".to_string(),
        "-".to_string(),
    ]
}

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
            (source_time - last).abs() < 1.0
        } else {
            true
        }
    }

    /// Stop the continuous video playback stream without blocking the UI thread.
    pub fn stop(&mut self) {
        self.is_running.store(false, Ordering::SeqCst);

        if let Some(mut child) = self.child_process.take() {
            // Discard process on background worker thread to prevent UI micro-stalls
            thread::spawn(move || {
                let _ = child.kill();
                let _ = child.wait();
            });
        }

        self.buffer.lock().clear();

        self.active_path = None;
        self.current_frame = None;
        self.last_pts = None;
    }

    /// Start streaming decoded 30 FPS video frames from `start_secs` with optional segment duration.
    pub fn start<P: AsRef<Path>>(
        &mut self,
        path: P,
        start_secs: f64,
        _duration_secs: Option<f64>,
        ctx: Option<&Context>,
    ) {
        self.stop();

        let path_buf = path.as_ref().to_path_buf();
        self.active_path = Some(path_buf.clone());
        self.is_running.store(true, Ordering::SeqCst);
        self.current_frame = None;
        self.last_pts = Some(start_secs);

        let mut err_lock = self.last_error.lock();
        *err_lock = None;
        drop(err_lock);

        let buffer_arc = Arc::clone(&self.buffer);
        let is_running_arc = Arc::clone(&self.is_running);
        let err_arc = Arc::clone(&self.last_error);
        let ffmpeg_bin = self.ffmpeg_bin.clone();
        let ctx_clone = ctx.cloned();

        let stream_args = build_stream_args(
            path_buf.to_str().unwrap_or_default(),
            start_secs,
        );

        let mut cmd = Command::new(&ffmpeg_bin);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }

        cmd.args(&stream_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let msg = format!("Failed to start FFmpeg playback stream: {}", e);
                *err_arc.lock() = Some(msg);
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
                let buf = buffer_arc.lock();
                if buf.len() >= 30 {
                    drop(buf);
                    thread::sleep(std::time::Duration::from_millis(15));
                    continue;
                }
                drop(buf);

                match stdout.read_exact(&mut raw_buf) {
                    Ok(_) => {
                        let pts = start_secs + (frame_idx as f64 * frame_duration);
                        frame_idx += 1;

                        let color_img = ColorImage::from_rgb(
                            [STREAM_WIDTH, STREAM_HEIGHT],
                            &raw_buf,
                        );

                        buffer_arc.lock().push_back((pts, color_img));

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
    /// Pops all frames whose PTS <= current_source_time.
    ///
    /// Returns `(had_new_frame, frame)`: `had_new_frame` is true ONLY when a brand-new
    /// decoded frame was popped during this call. The caller must only re-upload the
    /// texture (and clone the ~691 KB `ColorImage`) when that flag is true, otherwise a
    /// 60 FPS UI re-clones the frame at 2x the 30 FPS video rate for no visible gain.
    pub fn get_frame_for_time(&mut self, current_source_time: f64) -> (bool, Option<ColorImage>) {
        let mut buf = self.buffer.lock();
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
            return (true, self.current_frame.clone());
        }
        (false, None)
    }
}

impl Drop for StreamVideoPlayer {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Dual-deck lookahead playback engine that pre-warms upcoming cut clips in the background
/// for 0ms instantaneous cut transitions.
pub struct DualDeckPlayer {
    deck_a: StreamVideoPlayer,
    deck_b: StreamVideoPlayer,
    active_is_a: bool,
    pub active_clip_id: Option<u64>,
    pub prewarmed_clip_id: Option<u64>,
}

impl Default for DualDeckPlayer {
    fn default() -> Self {
        Self {
            deck_a: StreamVideoPlayer::new(),
            deck_b: StreamVideoPlayer::new(),
            active_is_a: true,
            active_clip_id: None,
            prewarmed_clip_id: None,
        }
    }
}

impl DualDeckPlayer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if active deck is already continuously decoding this stream
    pub fn is_active_continuous_with(&self, path: &Path, source_time: f64) -> bool {
        let active_deck = if self.active_is_a {
            &self.deck_a
        } else {
            &self.deck_b
        };
        active_deck.is_continuous_with(path, source_time)
    }

    /// Pre-warm the standby deck on an upcoming cut clip in the background only if it's a discontinuity.
    pub fn prewarm<P: AsRef<Path>>(
        &mut self,
        clip_id: u64,
        path: P,
        start_secs: f64,
        duration_secs: Option<f64>,
        ctx: Option<&Context>,
    ) {
        let path_ref = path.as_ref();

        // If the active deck already continuously streams this upcoming section, NEVER pre-warm!
        if self.is_active_continuous_with(path_ref, start_secs) {
            return;
        }

        if self.prewarmed_clip_id == Some(clip_id) || self.active_clip_id == Some(clip_id) {
            return;
        }

        let standby = if self.active_is_a {
            &mut self.deck_b
        } else {
            &mut self.deck_a
        };

        standby.start(path, start_secs, duration_secs, ctx);
        self.prewarmed_clip_id = Some(clip_id);
    }

    /// Switch active playback to a clip, preserving the active stream across same-file continuous cuts.
    pub fn switch_to_clip<P: AsRef<Path>>(
        &mut self,
        clip_id: u64,
        path: P,
        start_secs: f64,
        duration_secs: Option<f64>,
        ctx: Option<&Context>,
    ) {
        if self.active_clip_id == Some(clip_id) {
            return;
        }

        let path_ref = path.as_ref();

        // 1. If currently active deck is already continuous, preserve the stream with 0 processes!
        if self.is_active_continuous_with(path_ref, start_secs) {
            self.active_clip_id = Some(clip_id);
            if self.prewarmed_clip_id.is_some() {
                let standby = if self.active_is_a {
                    &mut self.deck_b
                } else {
                    &mut self.deck_a
                };
                standby.stop();
                self.prewarmed_clip_id = None;
            }
            return;
        }

        // 2. Check if the standby deck was pre-warmed for this clip (jump cut or file change)
        if self.prewarmed_clip_id == Some(clip_id) {
            // Instant 0ms deck swap!
            self.active_is_a = !self.active_is_a;
            self.active_clip_id = Some(clip_id);
            self.prewarmed_clip_id = None;

            // Stop the retired standby deck in background
            let old_deck = if self.active_is_a {
                &mut self.deck_b
            } else {
                &mut self.deck_a
            };
            old_deck.stop();
            return;
        }

        // 3. Fallback cold start on active deck
        let active_deck = if self.active_is_a {
            &mut self.deck_a
        } else {
            &mut self.deck_b
        };
        active_deck.start(path_ref, start_secs, duration_secs, ctx);
        self.active_clip_id = Some(clip_id);
        self.prewarmed_clip_id = None;
    }

    /// Retrieve the current video frame from the active deck synchronized to PTS.
    /// Returns `(had_new_frame, frame)`; see `StreamVideoPlayer::get_frame_for_time`.
    pub fn get_frame_for_time(
        &mut self,
        current_source_time: f64,
    ) -> (bool, Option<ColorImage>) {
        if self.active_is_a {
            self.deck_a.get_frame_for_time(current_source_time)
        } else {
            self.deck_b.get_frame_for_time(current_source_time)
        }
    }

    /// Stop both player decks.
    pub fn stop(&mut self) {
        self.deck_a.stop();
        self.deck_b.stop();
        self.active_clip_id = None;
        self.prewarmed_clip_id = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_frame(pts: f64) -> (f64, ColorImage) {
        let px = vec![0u8; 4 * 4 * 3];
        (pts, ColorImage::from_rgb([4, 4], &px))
    }

    /// Pins the multi-cut lag fix: a continuous (preserved) stream must NOT carry a
    /// producer-side `-t` cap, otherwise it EOFs at the first sub-clip's boundary and
    /// every subsequent adjacent cut freezes.
    #[test]
    fn continuous_stream_args_have_no_duration_cap() {
        let args = build_stream_args("movie.mp4", 1.5);
        assert!(
            !args.iter().any(|a| a == "-t"),
            "stream args must not contain '-t': {args:?}"
        );
        // Sanity: the seek point and input path are still present.
        assert!(args.contains(&"-ss".to_string()));
        assert!(args.contains(&"1.500".to_string()));
        assert!(args.contains(&"movie.mp4".to_string()));
    }

    /// Pins the Step-2 contract: the caller learns whether a genuinely NEW frame was
    /// decoded, so the UI only re-clones/re-uploads when it must (not every 60 FPS tick).
    #[test]
    fn get_frame_for_time_reports_new_frame_advanced() {
        let mut p = StreamVideoPlayer::default();
        {
            let mut buf = p.buffer.lock();
            buf.push_back(small_frame(0.000));
            buf.push_back(small_frame(0.033));
            buf.push_back(small_frame(0.066));
        }

        // First pull advances past 0.0 and 0.033 -> a new frame is available.
        let (adv, frame) = p.get_frame_for_time(0.05);
        assert!(adv, "a new frame should have been decoded for t=0.05");
        assert!(frame.is_some());

        // Same playhead again: nothing new to pop -> no clone/re-upload needed.
        let (adv, frame) = p.get_frame_for_time(0.05);
        assert!(!adv, "no new frame for a repeated playhead");
        assert!(frame.is_none());

        // Advancing past the last frame pops it; beyond that there is nothing left.
        let (adv, _) = p.get_frame_for_time(0.066);
        assert!(adv);
        let (adv, _) = p.get_frame_for_time(1.0);
        assert!(!adv);
    }
}
