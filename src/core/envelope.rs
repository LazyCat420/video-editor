use crate::core::time::TimeCode;
use serde::{Deserialize, Serialize};

/// Type of curve interpolation between two volume nodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurveType {
    Linear,
    SmoothBezier,
    EaseInOut,
    Hold,
}

impl Default for CurveType {
    fn default() -> Self {
        Self::Linear
    }
}

/// A single keyframe node on the volume envelope line graph.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VolumeNode {
    pub id: u64,
    /// Time offset relative to clip start.
    pub time_offset: TimeCode,
    /// Linear gain factor: 0.0 (silent / -inf dB), 1.0 (unity / 0 dB), 2.0 (+6 dB).
    pub gain: f32,
    /// Interpolation type leading TO the next node.
    pub curve: CurveType,
}

impl VolumeNode {
    pub fn new(id: u64, time_offset: TimeCode, gain: f32, curve: CurveType) -> Self {
        Self {
            id,
            time_offset,
            gain: gain.clamp(0.0, 4.0),
            curve,
        }
    }

    /// Convert linear gain to decibels (dB)
    #[inline]
    pub fn gain_to_db(&self) -> f32 {
        if self.gain <= 0.0001 {
            -60.0
        } else {
            20.0 * self.gain.log10()
        }
    }

    /// Create node from dB value (-60dB to +12dB)
    #[inline]
    pub fn from_db(id: u64, time_offset: TimeCode, db: f32, curve: CurveType) -> Self {
        let gain = if db <= -59.9 {
            0.0
        } else {
            10.0f32.powf(db / 20.0)
        };
        Self::new(id, time_offset, gain, curve)
    }
}

/// A collection of volume automation nodes representing the envelope curve over a clip.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VolumeEnvelope {
    pub nodes: Vec<VolumeNode>,
    pub enabled: bool,
    next_node_id: u64,
}

impl Default for VolumeEnvelope {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            enabled: true,
            next_node_id: 1,
        }
    }
}

impl VolumeEnvelope {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a default flat envelope at 1.0 (unity gain) for the clip duration.
    pub fn default_for_duration(duration: TimeCode) -> Self {
        let mut env = Self::new();
        env.add_node(TimeCode::ZERO, 1.0, CurveType::Linear);
        env.add_node(duration, 1.0, CurveType::Linear);
        env
    }

    /// Insert or replace a node at the given time offset.
    pub fn add_node(&mut self, time_offset: TimeCode, gain: f32, curve: CurveType) -> u64 {
        let id = self.next_node_id;
        self.next_node_id += 1;

        let new_node = VolumeNode::new(id, time_offset, gain, curve);

        // Check if a node already exists within 5 milliseconds; if so, replace it
        if let Some(existing) = self
            .nodes
            .iter_mut()
            .find(|n| (n.time_offset.micros - time_offset.micros).abs() < 5_000)
        {
            existing.gain = new_node.gain;
            existing.curve = new_node.curve;
            return existing.id;
        }

        self.nodes.push(new_node);
        self.sort_nodes();
        id
    }

    /// Remove a node by its unique ID.
    pub fn remove_node(&mut self, id: u64) -> bool {
        let initial_len = self.nodes.len();
        self.nodes.retain(|n| n.id != id);
        self.nodes.len() != initial_len
    }

    /// Update an existing node's time and gain.
    pub fn update_node(&mut self, id: u64, new_time: TimeCode, new_gain: f32) -> bool {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) {
            node.time_offset = new_time;
            node.gain = new_gain.clamp(0.0, 4.0);
            self.sort_nodes();
            true
        } else {
            false
        }
    }

    /// Update node gain only.
    pub fn update_node_gain(&mut self, id: u64, new_gain: f32) -> bool {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) {
            node.gain = new_gain.clamp(0.0, 4.0);
            true
        } else {
            false
        }
    }

    /// Update node curve type.
    pub fn update_node_curve(&mut self, id: u64, curve: CurveType) -> bool {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) {
            node.curve = curve;
            true
        } else {
            false
        }
    }

    #[inline]
    fn sort_nodes(&mut self) {
        self.nodes.sort_by_key(|n| n.time_offset.micros);
    }

    /// Evaluate the continuous interpolated gain factor at any time offset $t$.
    pub fn eval_gain(&self, time_offset: TimeCode) -> f32 {
        if !self.enabled || self.nodes.is_empty() {
            return 1.0;
        }

        // Before first node
        if time_offset <= self.nodes[0].time_offset {
            return self.nodes[0].gain;
        }

        // After last node
        let last_idx = self.nodes.len() - 1;
        if time_offset >= self.nodes[last_idx].time_offset {
            return self.nodes[last_idx].gain;
        }

        // Find bounding nodes N0 and N1 such that N0.time <= time_offset < N1.time
        for i in 0..last_idx {
            let n0 = &self.nodes[i];
            let n1 = &self.nodes[i + 1];

            if time_offset >= n0.time_offset && time_offset <= n1.time_offset {
                let dt = (n1.time_offset.micros - n0.time_offset.micros) as f32;
                if dt <= 0.0 {
                    return n0.gain;
                }

                let t = ((time_offset.micros - n0.time_offset.micros) as f32) / dt;
                let t_clamped = t.clamp(0.0, 1.0);

                return match n0.curve {
                    CurveType::Linear => n0.gain + (n1.gain - n0.gain) * t_clamped,
                    CurveType::SmoothBezier | CurveType::EaseInOut => {
                        // Smooth S-curve (cubic Hermite / smoothstep)
                        let smooth_t = t_clamped * t_clamped * (3.0 - 2.0 * t_clamped);
                        n0.gain + (n1.gain - n0.gain) * smooth_t
                    }
                    CurveType::Hold => n0.gain,
                };
            }
        }

        1.0
    }

    /// Generate an FFmpeg `volume=eval=frame:volume='...'` expression for export rendering.
    pub fn to_ffmpeg_volume_expression(&self) -> Option<String> {
        if !self.enabled || self.nodes.is_empty() {
            return None;
        }

        if self.nodes.len() == 1 {
            return Some(format!("volume={:.4}", self.nodes[0].gain));
        }

        // Check if all nodes have unity gain
        if self.nodes.iter().all(|n| (n.gain - 1.0).abs() < 0.001) {
            return None;
        }

        let mut parts = Vec::new();
        let last_idx = self.nodes.len() - 1;

        // Leading portion before first node
        let first = &self.nodes[0];
        let t0 = first.time_offset.as_secs_f64();
        parts.push(format!("if(lte(t,{:.3}),{:.4}", t0, first.gain));

        // Intermediate segments
        for i in 0..last_idx {
            let n0 = &self.nodes[i];
            let n1 = &self.nodes[i + 1];
            let start_t = n0.time_offset.as_secs_f64();
            let end_t = n1.time_offset.as_secs_f64();
            let g0 = n0.gain;
            let g1 = n1.gain;
            let dt = end_t - start_t;

            if dt > 0.0001 {
                match n0.curve {
                    CurveType::Linear => {
                        let slope = (g1 - g0) / (dt as f32);
                        parts.push(format!(
                            ",if(between(t,{:.3},{:.3}),{:.4}+{:.4}*(t-{:.3})",
                            start_t, end_t, g0, slope, start_t
                        ));
                    }
                    CurveType::SmoothBezier | CurveType::EaseInOut => {
                        // Approximate cubic smoothstep for ffmpeg
                        parts.push(format!(
                            ",if(between(t,{:.3},{:.3}),{:.4}+({:.4}-{:.4})*(3*pow((t-{:.3})/{:.3},2)-2*pow((t-{:.3})/{:.3},3))",
                            start_t, end_t, g0, g1, g0, start_t, dt, start_t, dt
                        ));
                    }
                    CurveType::Hold => {
                        parts.push(format!(
                            ",if(between(t,{:.3},{:.3}),{:.4}",
                            start_t, end_t, g0
                        ));
                    }
                }
            }
        }

        // Trailing portion after last node
        let last = &self.nodes[last_idx];
        parts.push(format!(",{:.4}", last.gain));

        // Close all nested `if(` parentheses
        let close_parens = ")".repeat(parts.len() - 1);
        let joined = parts.join("") + &close_parens;

        Some(format!("volume=eval=frame:volume='{}'", joined))
    }
}
