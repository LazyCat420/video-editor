use crate::core::clip::Clip;
use crate::core::time::TimeCode;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackKind {
    Video,
    Audio,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: u64,
    pub name: String,
    pub kind: TrackKind,
    pub clips: Vec<Clip>,
    pub is_muted: bool,
    pub is_solo: bool,
    pub is_locked: bool,
    pub volume: f32,
}

impl Track {
    pub fn new(id: u64, name: String, kind: TrackKind) -> Self {
        Self {
            id,
            name,
            kind,
            clips: Vec::new(),
            is_muted: false,
            is_solo: false,
            is_locked: false,
            volume: 1.0,
        }
    }

    /// Add a clip to this track.
    pub fn add_clip(&mut self, mut clip: Clip) {
        clip.track_id = self.id;
        self.clips.push(clip);
        self.sort_clips();
    }

    /// Remove a clip by ID.
    pub fn remove_clip(&mut self, clip_id: u64) -> Option<Clip> {
        if let Some(pos) = self.clips.iter().position(|c| c.id == clip_id) {
            Some(self.clips.remove(pos))
        } else {
            None
        }
    }

    /// Find the clip at a specific timeline timestamp.
    pub fn get_clip_at(&self, time: TimeCode) -> Option<&Clip> {
        self.clips.iter().find(|c| c.contains_timeline_time(time))
    }

    /// Find mutable reference to clip at a specific timeline timestamp.
    pub fn get_clip_at_mut(&mut self, time: TimeCode) -> Option<&mut Clip> {
        self.clips
            .iter_mut()
            .find(|c| c.contains_timeline_time(time))
    }

    /// Total duration of all clips on this track.
    pub fn duration(&self) -> TimeCode {
        self.clips
            .iter()
            .map(|c| c.timeline_end())
            .max()
            .unwrap_or(TimeCode::ZERO)
    }

    /// Sort all clips chronologically by their timeline start point.
    #[inline]
    pub fn sort_clips(&mut self) {
        self.clips.sort_by_key(|c| c.timeline_start.micros);
    }

    /// Split the clip intersecting `split_time`.
    pub fn split_clip_at(&mut self, split_time: TimeCode, next_clip_id: &mut u64) -> bool {
        let mut split_result = None;

        for (idx, clip) in self.clips.iter_mut().enumerate() {
            if clip.contains_timeline_time(split_time) {
                let new_id = *next_clip_id;
                *next_clip_id += 1;
                if let Some(second_half) = clip.split_at(split_time, new_id) {
                    split_result = Some((idx + 1, second_half));
                }
                break;
            }
        }

        if let Some((insert_idx, new_clip)) = split_result {
            self.clips.insert(insert_idx, new_clip);
            self.sort_clips();
            true
        } else {
            false
        }
    }
}
