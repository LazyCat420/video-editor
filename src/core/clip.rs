use crate::core::envelope::VolumeEnvelope;
use crate::core::time::TimeCode;
use crate::core::transition::Transition;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A video or audio clip on a timeline track.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Clip {
    pub id: u64,
    pub track_id: u64,
    pub name: String,
    /// Absolute path to original full-resolution master media.
    pub source_path: PathBuf,
    /// Optional path to low-res 360p intraframe proxy for real-time scrubbing.
    pub proxy_path: Option<PathBuf>,
    /// Optional path to cached binary waveform peak file (`.peaks`).
    pub peak_path: Option<PathBuf>,
    /// Total duration of the original media source.
    pub source_duration: TimeCode,
    /// Trim in-point within source media.
    pub source_in: TimeCode,
    /// Trim out-point within source media.
    pub source_out: TimeCode,
    /// Start timestamp on the timeline.
    pub timeline_start: TimeCode,
    /// Playback speed multiplier (1.0 = normal).
    pub speed: f32,
    /// Volume automation envelope (node line graph).
    pub volume_envelope: VolumeEnvelope,
    /// Transition at the beginning (In) of the clip (e.g. Fade In, Wipe In, Dip to Black).
    #[serde(default)]
    pub transition_in: Option<Transition>,
    /// Transition at the end (Out) of the clip (e.g. Fade Out, Crossfade to next clip, Dip to Black).
    #[serde(default)]
    pub transition_out: Option<Transition>,
    /// Legacy transition field kept for backwards-compatibility with existing saved projects.
    #[serde(default)]
    pub transition: Option<Transition>,
    /// On-screen text/caption overlay (e.g. "Hawaii Beach Day 1", "The End")
    /// LEGACY: kept only to load old project files. Migrated into `elements` on load.
    #[serde(default)]
    pub text_overlay: Option<crate::core::text_overlay::TextOverlay>,
    /// LEGACY: kept only to load old project files. Migrated into `background` on load.
    #[serde(default)]
    pub is_title_card: bool,
    #[serde(default)]
    pub title_card_bg: Option<crate::core::text_overlay::TitleCardBackground>,
    /// Background of a blank slide (no incoming video stream). Rendered behind `elements`.
    #[serde(default)]
    pub background: Option<crate::core::text_overlay::SlideBackground>,
    /// Placed slide elements (text / extra pictures / videos / audio) drawn over the base.
    #[serde(default)]
    pub elements: Vec<crate::core::text_overlay::SlideElement>,
    pub has_video: bool,
    pub has_audio: bool,
    /// Is the clip currently selected in the UI?
    #[serde(skip)]
    pub is_selected: bool,
}

impl Clip {
    pub fn new(
        id: u64,
        track_id: u64,
        name: String,
        source_path: PathBuf,
        source_duration: TimeCode,
        has_video: bool,
        has_audio: bool,
    ) -> Self {
        let volume_envelope = VolumeEnvelope::default_for_duration(source_duration);
        Self {
            id,
            track_id,
            name,
            source_path,
            proxy_path: None,
            peak_path: None,
            source_duration,
            source_in: TimeCode::ZERO,
            source_out: source_duration,
            timeline_start: TimeCode::ZERO,
            speed: 1.0,
            volume_envelope,
            transition_in: None,
            transition_out: None,
            transition: None,
            text_overlay: None,
            is_title_card: false,
            title_card_bg: None,
            background: None,
            elements: Vec::new(),
            has_video,
            has_audio,
            is_selected: false,
        }
    }

    /// Create a slide with a background and (optionally) one text element already on it.
    /// Used for both legacy title cards (migrated) and new blank slides.
    pub fn new_title_card(
        id: u64,
        track_id: u64,
        name: String,
        overlay: crate::core::text_overlay::TextOverlay,
        bg: crate::core::text_overlay::TitleCardBackground,
        duration_secs: f64,
    ) -> Self {
        let mut clip = Self::new_blank_slide(id, track_id, name, duration_secs);
        clip.background = Some(match bg {
            crate::core::text_overlay::TitleCardBackground::SolidColor(c) => {
                crate::core::text_overlay::SlideBackground::Solid(c)
            }
            crate::core::text_overlay::TitleCardBackground::Picture(p) => {
                crate::core::text_overlay::SlideBackground::Picture(p)
            }
        });
        if !overlay.text.trim().is_empty() {
            clip.elements
                .push(crate::core::text_overlay::SlideElement::Text(overlay));
        }
        clip
    }

    /// A blank slide: an empty canvas (solid background) the user fills with elements.
    pub fn new_blank_slide(id: u64, track_id: u64, name: String, duration_secs: f64) -> Self {
        let dur = TimeCode::from_secs_f64(duration_secs);
        let volume_envelope = VolumeEnvelope::default_for_duration(dur);
        Self {
            id,
            track_id,
            name,
            source_path: PathBuf::new(),
            proxy_path: None,
            peak_path: None,
            source_duration: dur,
            source_in: TimeCode::ZERO,
            source_out: dur,
            timeline_start: TimeCode::ZERO,
            speed: 1.0,
            volume_envelope,
            transition_in: None,
            transition_out: None,
            transition: None,
            text_overlay: None,
            is_title_card: false,
            title_card_bg: None,
            background: Some(crate::core::text_overlay::SlideBackground::Solid(
                egui::Color32::from_rgb(18, 18, 24),
            )),
            elements: Vec::new(),
            has_video: false,
            has_audio: false,
            is_selected: false,
        }
    }

    /// True when this clip is a static slide (blank slide or title card) rather than a
    /// streamed media clip.
    pub fn is_static_slide(&self) -> bool {
        !self.has_video && self.background.is_some()
    }

    /// Effective beginning/start transition for this clip.
    pub fn start_transition(&self) -> Option<&Transition> {
        self.transition_in.as_ref().or(self.transition.as_ref())
    }

    /// Effective ending/out transition for this clip.
    pub fn end_transition(&self) -> Option<&Transition> {
        self.transition_out.as_ref()
    }

    /// Duration of the trimmed clip on the timeline.
    #[inline]
    pub fn duration(&self) -> TimeCode {
        let trimmed_micros = (self.source_out.micros - self.source_in.micros).max(0);
        let speed = if self.speed <= 0.01 { 1.0 } else { self.speed as f64 };
        let scaled_micros = (trimmed_micros as f64 / speed).round() as i64;
        TimeCode::from_micros(scaled_micros)
    }

    /// End timestamp on the timeline.
    #[inline]
    pub fn timeline_end(&self) -> TimeCode {
        self.timeline_start + self.duration()
    }

    /// Check if a given timeline timestamp falls within this clip.
    #[inline]
    pub fn contains_timeline_time(&self, time: TimeCode) -> bool {
        time >= self.timeline_start && time <= self.timeline_end()
    }

    /// Convert timeline timestamp to source media offset.
    pub fn timeline_to_source_time(&self, timeline_time: TimeCode) -> Option<TimeCode> {
        if !self.contains_timeline_time(timeline_time) {
            return None;
        }
        let offset = timeline_time - self.timeline_start;
        let speed = if self.speed <= 0.01 { 1.0 } else { self.speed as f64 };
        let source_offset = (offset.as_secs_f64() * speed * 1_000_000.0).round() as i64;
        Some(self.source_in + TimeCode::from_micros(source_offset))
    }

    /// Split this clip at the given timeline timestamp, producing two adjacent clips.
    pub fn split_at(&mut self, split_time: TimeCode, new_clip_id: u64) -> Option<Clip> {
        if split_time <= self.timeline_start || split_time >= self.timeline_end() {
            return None;
        }

        let first_part_duration = split_time - self.timeline_start;
        let speed = if self.speed <= 0.01 { 1.0 } else { self.speed as f64 };
        let source_delta_micros =
            (first_part_duration.as_secs_f64() * speed * 1_000_000.0).round() as i64;
        let split_source_point = self.source_in + TimeCode::from_micros(source_delta_micros);

        // Build second clip
        let mut second_clip = self.clone();
        second_clip.id = new_clip_id;
        second_clip.source_in = split_source_point;
        second_clip.timeline_start = split_time;
        second_clip.is_selected = false;

        // Adjust envelope for second clip
        let mut second_env = VolumeEnvelope::new();
        for node in &self.volume_envelope.nodes {
            if node.time_offset >= first_part_duration {
                let shifted_time = node.time_offset - first_part_duration;
                second_env.add_node(shifted_time, node.gain, node.curve);
            }
        }
        if second_env.nodes.is_empty() {
            second_env.add_node(TimeCode::ZERO, 1.0, crate::core::envelope::CurveType::Linear);
            second_env.add_node(
                second_clip.duration(),
                1.0,
                crate::core::envelope::CurveType::Linear,
            );
        }
        second_clip.volume_envelope = second_env;

        // Trim first clip
        self.source_out = split_source_point;
        self.volume_envelope
            .nodes
            .retain(|n| n.time_offset <= first_part_duration);
        if self.volume_envelope.nodes.is_empty() {
            self.volume_envelope
                .add_node(TimeCode::ZERO, 1.0, crate::core::envelope::CurveType::Linear);
            self.volume_envelope.add_node(
                self.duration(),
                1.0,
                crate::core::envelope::CurveType::Linear,
            );
        }

        Some(second_clip)
    }

    /// Active media path to use for preview (uses proxy if available, else source).
    #[inline]
    pub fn active_preview_path(&self) -> &PathBuf {
        self.proxy_path.as_ref().unwrap_or(&self.source_path)
    }
}
