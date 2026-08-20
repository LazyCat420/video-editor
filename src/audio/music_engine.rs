use crate::core::time::TimeCode;
use crate::core::timeline::Timeline;
use std::time::Duration;

/// Where inside the song file playback must start when the timeline playhead
/// sits somewhere over the clip: offset = (playhead - clip start) + source_in.
/// Pure math — unit-tested without any audio device.
pub fn music_source_offset(
    clip_start: TimeCode,
    source_in: TimeCode,
    playhead: TimeCode,
) -> Duration {
    let micros = (playhead.micros - clip_start.micros).max(0) + source_in.micros.max(0);
    Duration::from_micros(micros as u64)
}

#[cfg(feature = "audio-playback")]
mod real {
    use super::*;
    use crate::core::clip::Clip;
    use crate::core::track::TrackKind;
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::PathBuf;

    struct ActiveSong {
        clip_id: u64,
        path: PathBuf,
        clip_start: TimeCode,
        source_in: TimeCode,
        /// Where in the file the sink's own clock starts (skip_duration builds).
        offset_at_start: Duration,
    }

    /// Plays the music track's clips during preview. The wall-clock playhead is
    /// the master clock; `sync` reconciles the sink to it every frame.
    pub struct MusicEngine {
        // OutputStream is !Send; the app lives on the main thread only. The
        // stream must outlive its sinks, so both are owned here together.
        stream: Option<(rodio::OutputStream, rodio::OutputStreamHandle)>,
        /// Latched after a failed device open so we don't retry every frame.
        device_failed: bool,
        sink: Option<rodio::Sink>,
        active: Option<ActiveSong>,
        /// Files that failed to decode — each error is surfaced once.
        failed_decodes: HashSet<PathBuf>,
        pending_error: Option<String>,
    }

    impl Default for MusicEngine {
        fn default() -> Self {
            Self {
                stream: None,
                device_failed: false,
                sink: None,
                active: None,
                failed_decodes: HashSet::new(),
                pending_error: None,
            }
        }
    }

    impl MusicEngine {
        pub fn new() -> Self {
            Self::default()
        }

        fn ensure_stream(&mut self) -> bool {
            if self.stream.is_some() {
                return true;
            }
            if self.device_failed {
                return false;
            }
            match rodio::OutputStream::try_default() {
                Ok(pair) => {
                    self.stream = Some(pair);
                    true
                }
                Err(e) => {
                    self.device_failed = true;
                    self.pending_error = Some(format!(
                        "No sound device found — music won't play here, but it WILL be in the exported video. ({e})"
                    ));
                    false
                }
            }
        }

        /// Per-frame reconciler while previewing. Silence when not playing, in a
        /// gap after the last song, or when the track is missing.
        pub fn sync(&mut self, timeline: &Timeline, playhead: TimeCode, is_playing: bool) {
            let track = timeline.tracks.iter().find(|t| t.kind == TrackKind::Audio);

            let clip = if is_playing {
                track.and_then(|t| {
                    // contains_timeline_time is end-inclusive, so at an exact
                    // boundary two clips match; `playhead < end` picks the next
                    // song, matching how the video side resolves boundaries.
                    t.clips.iter().find(|c| {
                        c.has_audio
                            && c.contains_timeline_time(playhead)
                            && playhead < c.timeline_end()
                    })
                })
            } else {
                None
            };

            let (Some(track), Some(clip)) = (track, clip) else {
                self.stop();
                return;
            };

            if track.is_muted {
                if let Some(s) = &self.sink {
                    s.set_volume(0.0);
                }
                return;
            }

            let needs_start = match &self.active {
                None => true,
                Some(a) => {
                    a.clip_id != clip.id
                        || a.path != clip.source_path
                        || a.clip_start != clip.timeline_start
                        || a.source_in != clip.source_in
                }
            };

            if needs_start {
                self.start_song(clip, playhead);
            } else if let Some(sink) = &self.sink {
                // Drift correction between the wall-clock playhead and the audio clock.
                let expected = music_source_offset(clip.timeline_start, clip.source_in, playhead);
                let offset_at_start = self
                    .active
                    .as_ref()
                    .map(|a| a.offset_at_start)
                    .unwrap_or(Duration::ZERO);
                let actual = offset_at_start + sink.get_pos();
                let drift = if expected > actual {
                    expected - actual
                } else {
                    actual - expected
                };
                if drift > Duration::from_millis(300) {
                    self.start_song(clip, playhead);
                    return;
                }
                if sink.is_paused() {
                    sink.play();
                }
            }

            if let Some(sink) = &self.sink {
                sink.set_volume(track.volume.clamp(0.0, 2.0));
            }
        }

        fn start_song(&mut self, clip: &Clip, playhead: TimeCode) {
            self.active = None;
            if let Some(s) = self.sink.take() {
                s.stop();
            }
            if self.failed_decodes.contains(&clip.source_path) {
                return;
            }
            if !self.ensure_stream() {
                return;
            }

            let file = match File::open(&clip.source_path) {
                Ok(f) => f,
                Err(e) => {
                    self.failed_decodes.insert(clip.source_path.clone());
                    self.pending_error =
                        Some(format!("Couldn't open \"{}\": {}", clip.name, e));
                    return;
                }
            };
            let source = match rodio::Decoder::new(BufReader::new(file)) {
                Ok(s) => s,
                Err(e) => {
                    self.failed_decodes.insert(clip.source_path.clone());
                    self.pending_error = Some(format!(
                        "\"{}\" can't be played in the preview ({e}). It will still be in the exported video.",
                        clip.name
                    ));
                    return;
                }
            };

            let handle = &self.stream.as_ref().expect("ensured above").1;
            let sink = match rodio::Sink::try_new(handle) {
                Ok(s) => s,
                Err(e) => {
                    self.pending_error = Some(format!("Audio error: {e}"));
                    return;
                }
            };

            // skip_duration decodes-and-discards up to the offset (simple and
            // codec-agnostic; the cost is a brief burst on far seeks), then
            // take_duration stops the source at the clip's trimmed end.
            use rodio::Source;
            let offset = music_source_offset(clip.timeline_start, clip.source_in, playhead);
            let clip_len = Duration::from_micros(
                (clip.source_out.micros - clip.source_in.micros).max(0) as u64,
            );
            let in_clip = offset.saturating_sub(Duration::from_micros(
                clip.source_in.micros.max(0) as u64,
            ));
            let remaining = clip_len.saturating_sub(in_clip);
            if remaining.is_zero() {
                return;
            }
            sink.append(source.skip_duration(offset).take_duration(remaining));
            sink.play();

            self.sink = Some(sink);
            self.active = Some(ActiveSong {
                clip_id: clip.id,
                path: clip.source_path.clone(),
                clip_start: clip.timeline_start,
                source_in: clip.source_in,
                offset_at_start: offset,
            });
        }

        pub fn pause(&mut self) {
            if let Some(s) = &self.sink {
                s.pause();
            }
        }

        pub fn stop(&mut self) {
            if let Some(s) = self.sink.take() {
                s.stop();
            }
            self.active = None;
        }

        /// After a seek/undo/redo/project switch: the next playing sync()
        /// restarts the right song at the right offset.
        pub fn invalidate(&mut self) {
            self.stop();
        }

        /// Queued error for the app to toast (drained once per frame).
        pub fn take_error(&mut self) -> Option<String> {
            self.pending_error.take()
        }
    }
}

#[cfg(feature = "audio-playback")]
pub use real::MusicEngine;

/// Inert stand-in with the identical API, so `--no-default-features` builds
/// (and any machine where cpal can't link) still compile and run silently.
#[cfg(not(feature = "audio-playback"))]
#[derive(Default)]
pub struct MusicEngine;

#[cfg(not(feature = "audio-playback"))]
impl MusicEngine {
    pub fn new() -> Self {
        Self
    }
    pub fn sync(&mut self, _timeline: &Timeline, _playhead: TimeCode, _is_playing: bool) {}
    pub fn pause(&mut self) {}
    pub fn stop(&mut self) {}
    pub fn invalidate(&mut self) {}
    pub fn take_error(&mut self) -> Option<String> {
        None
    }
}
