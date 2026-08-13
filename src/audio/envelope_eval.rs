use crate::core::envelope::VolumeEnvelope;
use crate::core::time::TimeCode;

/// Modulates a chunk of audio samples in-place according to a VolumeEnvelope.
/// `start_time` is the clip-relative time of the first sample.
/// `sample_rate` is typically 44100 or 48000.
/// `channels` is typically 1 (mono) or 2 (stereo).
pub fn apply_volume_envelope(
    samples: &mut [f32],
    start_time: TimeCode,
    sample_rate: u32,
    channels: usize,
    envelope: &VolumeEnvelope,
) {
    if !envelope.enabled || envelope.nodes.is_empty() {
        return;
    }

    if channels == 0 || sample_rate == 0 {
        return;
    }

    let frame_count = samples.len() / channels;
    let dt_per_frame_micros = 1_000_000.0 / sample_rate as f64;

    for frame_idx in 0..frame_count {
        let frame_time_micros =
            start_time.micros + (frame_idx as f64 * dt_per_frame_micros).round() as i64;
        let gain = envelope.eval_gain(TimeCode::from_micros(frame_time_micros));

        let base_idx = frame_idx * channels;
        for ch in 0..channels {
            samples[base_idx + ch] *= gain;
        }
    }
}
