use crate::core::clip::Clip;
use crate::core::time::TimeCode;
use crate::core::track::{Track, TrackKind};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Timeline {
    pub tracks: Vec<Track>,
    pub playhead: TimeCode,
    pub fps: f64,
    pub zoom_pps: f32, // Pixels per second (e.g. 40.0 to 200.0)
    pub scroll_offset_x: f32,
    pub snapping_enabled: bool,
    pub snap_threshold_pixels: f32,
    next_id: u64,
    #[serde(skip)]
    pub is_playing: bool,
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new(30.0)
    }
}

impl Timeline {
    pub fn new(fps: f64) -> Self {
        let mut tl = Self {
            tracks: Vec::new(),
            playhead: TimeCode::ZERO,
            fps: if fps <= 0.0 { 30.0 } else { fps },
            zoom_pps: 60.0,
            scroll_offset_x: 0.0,
            snapping_enabled: true,
            snap_threshold_pixels: 10.0,
            next_id: 1,
            is_playing: false,
        };

        // Initialize with simple, senior-friendly tracks: 1 Video + 1 Music/Audio
        tl.add_track("🎬 Video Track".to_string(), TrackKind::Video);
        tl.add_track("🎵 Music & Sound".to_string(), TrackKind::Audio);

        tl
    }

    pub fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add_track(&mut self, name: String, kind: TrackKind) -> u64 {
        let id = self.next_id();
        self.tracks.push(Track::new(id, name, kind));
        id
    }

    pub fn remove_track(&mut self, track_id: u64) -> bool {
        let initial_len = self.tracks.len();
        self.tracks.retain(|t| t.id != track_id);
        self.tracks.len() != initial_len
    }

    pub fn get_track(&self, track_id: u64) -> Option<&Track> {
        self.tracks.iter().find(|t| t.id == track_id)
    }

    pub fn get_track_mut(&mut self, track_id: u64) -> Option<&mut Track> {
        self.tracks.iter_mut().find(|t| t.id == track_id)
    }

    /// Overall timeline duration (maximum duration across all tracks).
    pub fn duration(&self) -> TimeCode {
        self.tracks
            .iter()
            .map(|t| t.duration())
            .max()
            .unwrap_or(TimeCode::ZERO)
    }

    /// Find snap point: snaps to playhead, origin, and other clip boundaries within pixel threshold.
    pub fn find_snap_point(&self, target_time: TimeCode, pps: f32) -> TimeCode {
        self.find_snap_point_excluding(target_time, pps, None)
    }

    /// Find snap point while excluding a specific clip (e.g. the one currently being dragged).
    pub fn find_snap_point_excluding(
        &self,
        target_time: TimeCode,
        pps: f32,
        exclude_clip_id: Option<u64>,
    ) -> TimeCode {
        if !self.snapping_enabled || pps <= 0.0 {
            return target_time;
        }

        let threshold_secs = (self.snap_threshold_pixels / pps) as f64;
        let mut candidates = Vec::new();

        // 1. Playhead
        candidates.push(self.playhead);

        // 2. Timeline origin (0.0)
        candidates.push(TimeCode::ZERO);

        // 3. Other clip boundaries
        for track in &self.tracks {
            for clip in &track.clips {
                if Some(clip.id) != exclude_clip_id {
                    candidates.push(clip.timeline_start);
                    candidates.push(clip.timeline_end());
                }
            }
        }

        let target_secs = target_time.as_secs_f64();
        let mut closest = target_time;
        let mut min_diff = threshold_secs;

        for cand in candidates {
            let diff = (cand.as_secs_f64() - target_secs).abs();
            if diff < min_diff {
                min_diff = diff;
                closest = cand;
            }
        }

        closest
    }

    /// Split clip(s) intersecting the playhead.
    pub fn split_at_playhead(&mut self) -> bool {
        let split_time = self.playhead;
        let mut any_split = false;

        // If a clip is selected, split only the selected clip
        let mut selected_found = false;
        for track in &mut self.tracks {
            for clip in &mut track.clips {
                if clip.is_selected && clip.contains_timeline_time(split_time) {
                    selected_found = true;
                    break;
                }
            }
        }

        let mut next_id = self.next_id;
        for track in &mut self.tracks {
            if selected_found {
                if track.clips.iter().any(|c| c.is_selected) {
                    if track.split_clip_at(split_time, &mut next_id) {
                        any_split = true;
                    }
                }
            } else {
                if track.split_clip_at(split_time, &mut next_id) {
                    any_split = true;
                }
            }
        }
        self.next_id = next_id;
        any_split
    }

    /// Delete all currently selected clips.
    pub fn delete_selected_clips(&mut self) -> bool {
        let mut any_deleted = false;
        for track in &mut self.tracks {
            let count_before = track.clips.len();
            track.clips.retain(|c| !c.is_selected);
            if track.clips.len() != count_before {
                any_deleted = true;
            }
        }
        any_deleted
    }

    /// Clear selection flags on all clips.
    pub fn clear_selection(&mut self) {
        for track in &mut self.tracks {
            for clip in &mut track.clips {
                clip.is_selected = false;
            }
        }
    }

    /// Select a single clip exclusively.
    pub fn select_clip(&mut self, clip_id: u64) {
        self.clear_selection();
        for track in &mut self.tracks {
            for clip in &mut track.clips {
                if clip.id == clip_id {
                    clip.is_selected = true;
                    return;
                }
            }
        }
    }

    /// Retrieve the currently selected clip (if exactly one is selected).
    pub fn get_selected_clip(&self) -> Option<&Clip> {
        for track in &self.tracks {
            for clip in &track.clips {
                if clip.is_selected {
                    return Some(clip);
                }
            }
        }
        None
    }

    /// Retrieve mutable reference to the currently selected clip.
    pub fn get_selected_clip_mut(&mut self) -> Option<&mut Clip> {
        for track in &mut self.tracks {
            for clip in &mut track.clips {
                if clip.is_selected {
                    return Some(clip);
                }
            }
        }
        None
    }

    /// Move or reposition a clip across tracks or timestamps.
    pub fn move_clip(
        &mut self,
        clip_id: u64,
        dest_track_id: u64,
        new_timeline_start: TimeCode,
    ) -> bool {
        let mut extracted_clip = None;

        for track in &mut self.tracks {
            if let Some(pos) = track.clips.iter().position(|c| c.id == clip_id) {
                let mut clip = track.clips.remove(pos);
                clip.timeline_start = new_timeline_start;
                clip.track_id = dest_track_id;
                extracted_clip = Some(clip);
                break;
            }
        }

        if let Some(clip) = extracted_clip {
            if let Some(dest_track) = self.get_track_mut(dest_track_id) {
                dest_track.add_clip(clip);
                return true;
            }
        }

        false
    }
}
