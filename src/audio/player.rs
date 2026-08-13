use crate::core::time::TimeCode;
use std::time::Instant;

pub struct AudioPlayer {
    pub is_playing: bool,
    last_update: Option<Instant>,
    pub playback_speed: f64,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self {
            is_playing: false,
            last_update: None,
            playback_speed: 1.0,
        }
    }
}

impl AudioPlayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn play(&mut self) {
        self.is_playing = true;
        self.last_update = Some(Instant::now());
    }

    pub fn pause(&mut self) {
        self.is_playing = false;
        self.last_update = None;
    }

    pub fn toggle(&mut self) -> bool {
        if self.is_playing {
            self.pause();
            false
        } else {
            self.play();
            true
        }
    }

    /// Advance the playhead based on elapsed time since the last frame update.
    /// Returns the updated playhead position.
    pub fn update_playhead(&mut self, current_playhead: TimeCode, max_duration: TimeCode) -> TimeCode {
        if !self.is_playing {
            return current_playhead;
        }

        let now = Instant::now();
        let elapsed = if let Some(last) = self.last_update {
            now.duration_since(last).as_secs_f64()
        } else {
            0.0
        };
        self.last_update = Some(now);

        let delta_micros = (elapsed * self.playback_speed * 1_000_000.0).round() as i64;
        let new_micros = current_playhead.micros + delta_micros;

        if max_duration.micros > 0 && new_micros >= max_duration.micros {
            self.pause();
            return max_duration;
        }

        TimeCode::from_micros(new_micros)
    }
}
