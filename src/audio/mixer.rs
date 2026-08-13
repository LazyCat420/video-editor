use crate::core::timeline::Timeline;
use crate::core::time::TimeCode;

pub struct AudioMixer {
    pub sample_rate: u32,
    pub channels: usize,
    pub master_volume: f32,
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            master_volume: 1.0,
        }
    }
}

impl AudioMixer {
    pub fn new(sample_rate: u32, channels: usize) -> Self {
        Self {
            sample_rate,
            channels,
            master_volume: 1.0,
        }
    }

    /// Evaluates the total effective gain for a track and clip at a specific timeline timestamp.
    pub fn calculate_effective_gain(
        &self,
        timeline: &Timeline,
        track_id: u64,
        clip_id: u64,
        timeline_time: TimeCode,
    ) -> f32 {
        let has_solo = timeline.tracks.iter().any(|t| t.is_solo);

        let track = match timeline.get_track(track_id) {
            Some(t) => t,
            None => return 0.0,
        };

        if track.is_muted {
            return 0.0;
        }

        if has_solo && !track.is_solo {
            return 0.0;
        }

        let clip = match track.clips.iter().find(|c| c.id == clip_id) {
            Some(c) => c,
            None => return 0.0,
        };

        if !clip.contains_timeline_time(timeline_time) {
            return 0.0;
        }

        let clip_offset = timeline_time - clip.timeline_start;
        let envelope_gain = clip.volume_envelope.eval_gain(clip_offset);

        (self.master_volume * track.volume * envelope_gain).clamp(0.0, 4.0)
    }
}
