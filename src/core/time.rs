use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Nanosecond-accurate timeline time representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct TimeCode {
    /// Total duration in microseconds to avoid floating point drift.
    pub micros: i64,
}

impl TimeCode {
    pub const ZERO: Self = Self { micros: 0 };

    #[inline]
    pub fn from_micros(micros: i64) -> Self {
        Self {
            micros: micros.max(0),
        }
    }

    #[inline]
    pub fn from_secs_f64(secs: f64) -> Self {
        Self {
            micros: (secs.max(0.0) * 1_000_000.0).round() as i64,
        }
    }

    #[inline]
    pub fn from_frames(frames: i64, fps: f64) -> Self {
        if fps <= 0.0 {
            return Self::ZERO;
        }
        let secs = frames.max(0) as f64 / fps;
        Self::from_secs_f64(secs)
    }

    #[inline]
    pub fn as_secs_f64(&self) -> f64 {
        self.micros as f64 / 1_000_000.0
    }

    #[inline]
    pub fn as_frames(&self, fps: f64) -> i64 {
        if fps <= 0.0 {
            return 0;
        }
        (self.as_secs_f64() * fps).round() as i64
    }

    /// Format as HH:MM:SS.mmm
    pub fn to_timecode_str(&self) -> String {
        let total_ms = self.micros / 1000;
        let ms = total_ms % 1000;
        let total_secs = total_ms / 1000;
        let secs = total_secs % 60;
        let total_mins = total_secs / 60;
        let mins = total_mins % 60;
        let hours = total_mins / 60;

        format!("{:02}:{:02}:{:02}.{:03}", hours, mins, secs, ms)
    }

    /// Format as HH:MM:SS:FF (SMPTE timecode)
    pub fn to_smpte_str(&self, fps: f64) -> String {
        let safe_fps = if fps <= 0.0 { 30.0 } else { fps };
        let total_frames = self.as_frames(safe_fps);
        let fps_int = safe_fps.round() as i64;
        let frames = total_frames % fps_int.max(1);
        let total_secs = total_frames / fps_int.max(1);
        let secs = total_secs % 60;
        let total_mins = total_secs / 60;
        let mins = total_mins % 60;
        let hours = total_mins / 60;

        format!("{:02}:{:02}:{:02}:{:02}", hours, mins, secs, frames)
    }

    /// Convert to timeline X pixel position given pixels-per-second zoom scale.
    #[inline]
    pub fn to_pixels(&self, pps: f32) -> f32 {
        (self.as_secs_f64() as f32) * pps
    }

    /// Convert timeline X pixel position back to TimeCode.
    #[inline]
    pub fn from_pixels(pixels: f32, pps: f32) -> Self {
        if pps <= 0.0 {
            return Self::ZERO;
        }
        let secs = (pixels.max(0.0) / pps) as f64;
        Self::from_secs_f64(secs)
    }
}

impl Add for TimeCode {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            micros: self.micros + rhs.micros,
        }
    }
}

impl AddAssign for TimeCode {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.micros += rhs.micros;
    }
}

impl Sub for TimeCode {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            micros: (self.micros - rhs.micros).max(0),
        }
    }
}

impl SubAssign for TimeCode {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.micros = (self.micros - rhs.micros).max(0);
    }
}

impl fmt::Display for TimeCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_timecode_str())
    }
}
