use egui::{Color32, Pos2, Rect, Shape, Stroke};
use serde::{Deserialize, Serialize};

/// Celebration & PowerPoint-style screen visual effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlideEffectKind {
    /// Colorful explosive fireworks with sparkling light trails
    Fireworks,
    /// Festive tumbling confetti paper raining downward
    Confetti,
    /// Colorful party balloons floating upward with string sways
    Balloons,
    /// Graceful flock of birds soaring across the sky
    Birds,
    /// Cheerful pulsing clapping hands with celebration bursts
    Clapping,
    /// Luminous shooting stars streaking diagonally across the screen
    ShootingStar,
}

impl SlideEffectKind {
    pub fn all() -> &'static [SlideEffectKind] {
        &[
            SlideEffectKind::Fireworks,
            SlideEffectKind::Confetti,
            SlideEffectKind::Balloons,
            SlideEffectKind::Birds,
            SlideEffectKind::Clapping,
            SlideEffectKind::ShootingStar,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Fireworks => "Fireworks",
            Self::Confetti => "Confetti",
            Self::Balloons => "Floating Balloons",
            Self::Birds => "Flying Birds",
            Self::Clapping => "Clapping Applause",
            Self::ShootingStar => "Shooting Star",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Fireworks => "🎆",
            Self::Confetti => "🎊",
            Self::Balloons => "🎈",
            Self::Birds => "🕊️",
            Self::Clapping => "👏",
            Self::ShootingStar => "🌠",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Fireworks => "Vibrant colorful bursts with sparkling light trails",
            Self::Confetti => "Celebratory confetti ribbons raining down",
            Self::Balloons => "Party balloons gently floating up the screen",
            Self::Birds => "Flock of birds gracefully soaring across",
            Self::Clapping => "Rhythmic cheering applause with sparkle pops",
            Self::ShootingStar => "Magical glowing stars streaking across the sky",
        }
    }
}

/// A configured active slide effect instance.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SlideEffect {
    pub kind: SlideEffectKind,
    #[serde(default = "default_intensity")]
    pub intensity: f32, // 0.2 to 2.0 (density/particle count)
    #[serde(default = "default_speed")]
    pub speed: f32,     // 0.5 to 2.0 (animation speed multiplier)
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_intensity() -> f32 {
    1.0
}

fn default_speed() -> f32 {
    1.0
}

fn default_enabled() -> bool {
    true
}

impl SlideEffect {
    pub fn new(kind: SlideEffectKind) -> Self {
        Self {
            kind,
            intensity: 1.0,
            speed: 1.0,
            enabled: true,
        }
    }
}

// =============================================================================
// PARTICLE SIMULATION ENGINE
// =============================================================================

/// High-performance deterministic particle and animation simulation engine for live egui preview.
///
/// Uses analytical trajectory computation with ghost-trail rendering for smooth,
/// stateless motion blur. All positions are computed from `t` using deterministic
/// math, so the animation is scrub-friendly and requires no stored state.
pub struct EffectParticleSimulator;

impl EffectParticleSimulator {
    /// Render all active slide effects on top of the given preview rectangle at time `t_secs`.
    pub fn render_preview(
        painter: &egui::Painter,
        rect: Rect,
        t_secs: f64,
        effects: &[SlideEffect],
    ) {
        for effect in effects {
            if !effect.enabled {
                continue;
            }
            let speed = effect.speed.clamp(0.2, 3.0) as f64;
            let intensity = effect.intensity.clamp(0.2, 3.0);
            let sim_time = t_secs * speed;

            match effect.kind {
                SlideEffectKind::Fireworks => {
                    Self::draw_fireworks(painter, rect, sim_time, intensity);
                }
                SlideEffectKind::Confetti => {
                    Self::draw_confetti(painter, rect, sim_time, intensity);
                }
                SlideEffectKind::Balloons => {
                    Self::draw_balloons(painter, rect, sim_time, intensity);
                }
                SlideEffectKind::Birds => {
                    Self::draw_birds(painter, rect, sim_time, intensity);
                }
                SlideEffectKind::Clapping => {
                    Self::draw_clapping(painter, rect, sim_time, intensity);
                }
                SlideEffectKind::ShootingStar => {
                    Self::draw_shooting_stars(painter, rect, sim_time, intensity);
                }
            }
        }
    }

    // =========================================================================
    // HELPER FUNCTIONS — Hash, Glow, Sparkle, Bezier, Rotation
    // =========================================================================

    /// Fast deterministic integer hash (lowbias32). Produces well-distributed
    /// pseudo-random values from any seed, used for particle property variation.
    #[inline]
    fn hash(mut x: u32) -> u32 {
        x ^= x >> 16;
        x = x.wrapping_mul(0x45d9f3b);
        x ^= x >> 16;
        x = x.wrapping_mul(0x45d9f3b);
        x ^= x >> 16;
        x
    }

    /// Hash to float in [0.0, 1.0).
    #[inline]
    fn hash_f(seed: u32) -> f32 {
        (Self::hash(seed) & 0xFFFF) as f32 / 65536.0
    }

    /// Hash to float in [min, max).
    #[inline]
    fn hash_range(seed: u32, min: f32, max: f32) -> f32 {
        min + Self::hash_f(seed) * (max - min)
    }

    /// Draw multi-layer glow: 3 concentric alpha-blended circles simulating bloom.
    /// The outer halo is large and faint, the inner core is small and bright.
    fn draw_glow(painter: &egui::Painter, pos: Pos2, radius: f32, color: Color32, alpha: f32) {
        let a = (alpha * 255.0).clamp(0.0, 255.0);
        // Outer halo
        painter.circle_filled(
            pos,
            radius * 3.2,
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), (a * 0.08) as u8),
        );
        // Mid glow
        painter.circle_filled(
            pos,
            radius * 1.8,
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), (a * 0.25) as u8),
        );
        // Bright core
        painter.circle_filled(
            pos,
            radius,
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a as u8),
        );
    }

    /// Draw a 4-pointed sparkle cross with optional diagonal arms (8-point).
    fn draw_sparkle(painter: &egui::Painter, pos: Pos2, size: f32, color: Color32, eight_point: bool) {
        let s = Stroke::new(1.2, color);
        painter.line_segment(
            [Pos2::new(pos.x - size, pos.y), Pos2::new(pos.x + size, pos.y)],
            s,
        );
        painter.line_segment(
            [Pos2::new(pos.x, pos.y - size), Pos2::new(pos.x, pos.y + size)],
            s,
        );
        if eight_point {
            let d = size * 0.7;
            let s2 = Stroke::new(0.8, color);
            painter.line_segment(
                [Pos2::new(pos.x - d, pos.y - d), Pos2::new(pos.x + d, pos.y + d)],
                s2,
            );
            painter.line_segment(
                [Pos2::new(pos.x + d, pos.y - d), Pos2::new(pos.x - d, pos.y + d)],
                s2,
            );
        }
    }

    /// Evaluate a cubic bezier at parameter t ∈ [0, 1].
    fn bezier(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, t: f32) -> Pos2 {
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let t2 = t * t;
        Pos2::new(
            mt2 * mt * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t2 * t * p3.x,
            mt2 * mt * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t2 * t * p3.y,
        )
    }

    /// Project a 3D-rotated rectangle to 2D screen coordinates.
    /// `theta` = tilt angle (rotation around local X), `phi` = spin angle (around Z).
    /// Returns 4 corners of the projected parallelogram (always convex).
    fn rotated_rect_corners(
        center: Pos2,
        hw: f32,
        hh: f32,
        theta: f32,
        phi: f32,
    ) -> [Pos2; 4] {
        let ct = theta.cos();
        let cp = phi.cos();
        let sp = phi.sin();
        let a = hw * cp;
        let b = hh * ct * sp;
        let c = hw * sp;
        let d = hh * ct * cp;
        [
            Pos2::new(center.x - a + b, center.y - c - d),
            Pos2::new(center.x + a + b, center.y + c - d),
            Pos2::new(center.x + a - b, center.y + c + d),
            Pos2::new(center.x - a - b, center.y - c + d),
        ]
    }

    /// Compute a 3D firework spark position projected to 2D with depth scaling.
    /// `vx`, `vy`, `vz` are 3D velocities in pixels/sec.
    /// Returns `(screen_pos, depth_scale)`.
    fn volumetric_spark_pos(
        center: Pos2,
        vx: f32,
        vy: f32,
        vz: f32,
        t: f32,
        gravity: f32,
        drag: f32,
    ) -> (Pos2, f32) {
        let pos_factor = if drag > 0.01 {
            (1.0 - (-drag * t).exp()) / drag
        } else {
            t
        };
        let x = center.x + vx * pos_factor;
        let y = center.y + vy * pos_factor + 0.5 * gravity * t * t;
        let z = vz * pos_factor;
        let depth_scale = (260.0 / (260.0 + z)).clamp(0.45, 1.75);
        (Pos2::new(x, y), depth_scale)
    }

    /// Slightly vary a color's RGB channels by a deterministic offset for organic variation.
    fn vary_color(color: Color32, seed: u32, range: i16) -> Color32 {
        let dr = (Self::hash_range(seed, -range as f32, range as f32)) as i16;
        let dg = (Self::hash_range(seed.wrapping_add(1), -range as f32, range as f32)) as i16;
        let db = (Self::hash_range(seed.wrapping_add(2), -range as f32, range as f32)) as i16;
        Color32::from_rgb(
            (color.r() as i16 + dr).clamp(0, 255) as u8,
            (color.g() as i16 + dg).clamp(0, 255) as u8,
            (color.b() as i16 + db).clamp(0, 255) as u8,
        )
    }

    /// Generate points for an ellipse (n-sided polygon approximation).
    fn ellipse_points(center: Pos2, rx: f32, ry: f32, n: usize) -> Vec<Pos2> {
        let tau = std::f32::consts::TAU;
        (0..n)
            .map(|i| {
                let angle = (i as f32) * tau / (n as f32);
                Pos2::new(center.x + rx * angle.cos(), center.y + ry * angle.sin())
            })
            .collect()
    }

    /// Scale factor to normalize visual sizes relative to a 400px-tall reference frame.
    #[inline]
    fn scale(rect: Rect) -> f32 {
        rect.height() / 400.0
    }

    // =========================================================================
    // 1. PROCEDURAL FIREWORKS — 3D Volumetric Shells, Streamer Trails, Strobe
    // =========================================================================
    fn draw_fireworks(painter: &egui::Painter, rect: Rect, t: f64, intensity: f32) {
        let burst_period = 2.0;
        let burst_count = (5.0 * intensity).round() as usize;
        let scale = Self::scale(rect);

        // Core pyrotechnic chemical color palettes
        let color_palettes = [
            // Palette 0: Crimson Strontium + Electric Gold
            (Color32::from_rgb(255, 45, 65), Color32::from_rgb(255, 215, 35)),
            // Palette 1: Azure Copper Cyan + Bright Silver
            (Color32::from_rgb(30, 220, 255), Color32::from_rgb(240, 250, 255)),
            // Palette 2: Emerald Barium + Neon Lime
            (Color32::from_rgb(45, 255, 115), Color32::from_rgb(180, 255, 80)),
            // Palette 3: Royal Purple + Flamingo Pink
            (Color32::from_rgb(195, 60, 255), Color32::from_rgb(255, 110, 200)),
            // Palette 4: Golden Willow / Kamuro (Deep Amber + Bright Gold)
            (Color32::from_rgb(255, 175, 40), Color32::from_rgb(255, 235, 120)),
        ];

        for i in 0..burst_count {
            let seed_base = (i as u32).wrapping_mul(7919).wrapping_add(47);
            let offset = Self::hash_range(seed_base, 0.0, burst_period as f32) as f64;
            let local_t = (t + offset) % burst_period;
            let progress = (local_t / burst_period) as f32;

            // Burst center apex in upper 65% of screen
            let cx_norm = Self::hash_range(seed_base.wrapping_add(100), 0.12, 0.88);
            let cy_norm = Self::hash_range(seed_base.wrapping_add(200), 0.15, 0.58);
            let center = Pos2::new(
                rect.min.x + cx_norm * rect.width(),
                rect.min.y + cy_norm * rect.height(),
            );

            let (primary_color, secondary_color) = color_palettes[i % color_palettes.len()];
            let shell_style = (i % 4) as u8;
            let rocket_phase = 0.24;

            if progress < rocket_phase {
                // =============================================================
                // PHASE 1: ROCKET LAUNCH ASCENT (Smooth Quadratic Deceleration)
                // =============================================================
                let rp = progress / rocket_phase;
                // Ease-out launch trajectory
                let launch_y = rect.max.y + 10.0;
                let rocket_y = launch_y - (launch_y - center.y) * (1.0 - (1.0 - rp).powi(2));
                let sway = (t * 16.0 + offset * 5.0).sin() as f32 * 2.5 * scale;
                let r_pos = Pos2::new(center.x + sway, rocket_y);

                // Streaming smoke and glowing spark tail
                let smoke_steps = 10;
                for st in 1..=smoke_steps {
                    let back_p = (rp - st as f32 * 0.025).max(0.0);
                    let back_y = launch_y - (launch_y - center.y) * (1.0 - (1.0 - back_p).powi(2));
                    let back_sway = (t * 16.0 + offset * 5.0 - st as f64 * 0.1).sin() as f32 * 2.5 * scale;
                    let back_pos = Pos2::new(center.x + back_sway, back_y);
                    let tail_alpha = (1.0 - st as f32 / smoke_steps as f32).powf(1.6);

                    // Glowing golden sparks
                    painter.circle_filled(
                        back_pos,
                        (2.6 - st as f32 * 0.2).max(0.6) * scale,
                        Color32::from_rgba_unmultiplied(255, 190, 60, (tail_alpha * 200.0) as u8),
                    );

                    // Smoke puff
                    if st % 2 == 0 {
                        painter.circle_filled(
                            back_pos,
                            (3.5 + st as f32 * 0.4) * scale,
                            Color32::from_rgba_unmultiplied(200, 180, 160, (tail_alpha * 45.0) as u8),
                        );
                    }
                }

                // Rocket head with intense white-hot bloom
                Self::draw_glow(painter, r_pos, 3.0 * scale, Color32::from_rgb(255, 245, 200), 1.0);
                painter.circle_filled(r_pos, 1.8 * scale, Color32::WHITE);
            } else {
                // =============================================================
                // PHASE 2 & 3: VOLUMETRIC DETONATION & 3D PARTICLE DISPERSION
                // =============================================================
                let burst_p = (progress - rocket_phase) / (1.0 - rocket_phase);
                let alpha_factor = (1.0 - burst_p).powf(0.75);

                // --- 1. DETONATION FLASH & EXPANDING BLAST SHOCKWAVE ---
                if burst_p < 0.20 {
                    let flash_p = burst_p / 0.20;
                    let flash_alpha = (1.0 - flash_p).powf(2.0);
                    let flash_radius = (35.0 * scale) * flash_p.sqrt();

                    // White-hot core bloom
                    painter.circle_filled(
                        center,
                        flash_radius,
                        Color32::from_rgba_unmultiplied(255, 255, 255, (flash_alpha * 220.0) as u8),
                    );
                    painter.circle_filled(
                        center,
                        flash_radius * 0.45,
                        Color32::from_rgba_unmultiplied(255, 250, 190, (flash_alpha * 255.0) as u8),
                    );

                    // Expanding shockwave ring
                    let shockwave_r = 50.0 * scale * flash_p;
                    painter.circle_stroke(
                        center,
                        shockwave_r,
                        Stroke::new(
                            (2.0 * (1.0 - flash_p)).max(0.5) * scale,
                            Color32::from_rgba_unmultiplied(255, 230, 140, (flash_alpha * 160.0) as u8),
                        ),
                    );
                }

                // --- 2. 3D VOLUMETRIC PARTICLE SHELLS ---
                // Configure shell properties based on pyrotechnic archetype
                let (num_sparks, base_speed, gravity, drag, trail_len, has_pistil, is_strobe) = match shell_style {
                    // Kamuro / Willow: Heavy weeping golden trails, low drag, high gravity
                    0 => (75, 78.0 * scale, 58.0 * scale, 1.2_f32, 8, false, false),
                    // Chrysanthemum with Pistil: Dense outer color sphere + inner white-hot core
                    1 => (65, 85.0 * scale, 42.0 * scale, 1.7_f32, 6, true, false),
                    // Strobe / Glitter: Flickering stars with fast random twinkling
                    2 => (80, 80.0 * scale, 38.0 * scale, 1.8_f32, 5, false, true),
                    // Peony: High-speed vivid spherical burst
                    _ => (70, 92.0 * scale, 44.0 * scale, 1.9_f32, 6, false, false),
                };

                let spark_t = burst_p * 1.35; // trajectory elapsed time

                for s in 0..num_sparks {
                    let spark_seed = seed_base
                        .wrapping_mul(313)
                        .wrapping_add(s as u32)
                        .wrapping_add(100);

                    // Generate uniform 3D spherical direction vector (Fibonacci/Spherical coordinates)
                    let u = Self::hash_range(spark_seed, -1.0, 1.0);
                    let theta = (s as f32) * 2.3999632 // Golden ratio angle
                        + Self::hash_range(spark_seed.wrapping_add(1), -0.15, 0.15);
                    let radius_3d = (1.0 - u * u).max(0.0).sqrt();

                    let dir_x = radius_3d * theta.cos();
                    let dir_y = u;
                    let dir_z = radius_3d * theta.sin();

                    let speed_variation = Self::hash_range(spark_seed.wrapping_add(2), 0.65, 1.25);
                    let vx = dir_x * base_speed * speed_variation;
                    let vy = dir_y * base_speed * speed_variation;
                    let vz = dir_z * base_speed * speed_variation;

                    // Compute head position and depth
                    let (head_pos, depth_scale) =
                        Self::volumetric_spark_pos(center, vx, vy, vz, spark_t, gravity, drag);

                    // Culling if out of bounds
                    if head_pos.x < rect.min.x - 40.0
                        || head_pos.x > rect.max.x + 40.0
                        || head_pos.y < rect.min.y - 40.0
                        || head_pos.y > rect.max.y + 40.0
                    {
                        continue;
                    }

                    // Strobe flickering calculation
                    let strobe_alpha = if is_strobe && burst_p > 0.25 {
                        let freq = 36.0 + Self::hash_range(spark_seed.wrapping_add(3), 0.0, 15.0);
                        let strobe_phase = (spark_t * freq as f32 + s as f32 * 1.2).sin();
                        if strobe_phase > 0.1 { 1.0 } else { 0.15 }
                    } else {
                        1.0
                    };

                    let spark_alpha = (alpha_factor * strobe_alpha).clamp(0.0, 1.0);
                    let spark_color = Self::vary_color(primary_color, spark_seed, 20);
                    let spark_size = (3.0 * (1.0 - burst_p * 0.45) * depth_scale).max(0.8) * scale;

                    // --- DRAW CONNECTED GRADIENT STREAMER TRAILS ---
                    let dt = 0.024;
                    for seg in 0..trail_len {
                        let t_a = (spark_t - seg as f32 * dt).max(0.0);
                        let t_b = (spark_t - (seg + 1) as f32 * dt).max(0.0);

                        let (p_a, depth_a) =
                            Self::volumetric_spark_pos(center, vx, vy, vz, t_a, gravity, drag);
                        let (p_b, _) =
                            Self::volumetric_spark_pos(center, vx, vy, vz, t_b, gravity, drag);

                        let seg_p = seg as f32 / trail_len as f32;
                        let seg_alpha = spark_alpha * (1.0 - seg_p).powf(1.4);
                        let seg_width = (spark_size * (1.0 - seg_p * 0.65) * depth_a).max(0.5);

                        let seg_color = if seg == 0 {
                            Color32::from_rgba_unmultiplied(255, 255, 240, (seg_alpha * 255.0) as u8)
                        } else if seg < 3 {
                            Color32::from_rgba_unmultiplied(
                                spark_color.r(),
                                spark_color.g(),
                                spark_color.b(),
                                (seg_alpha * 255.0) as u8,
                            )
                        } else {
                            // Warm golden amber ember trail
                            Color32::from_rgba_unmultiplied(
                                255,
                                175,
                                45,
                                (seg_alpha * 190.0) as u8,
                            )
                        };

                        painter.line_segment([p_a, p_b], Stroke::new(seg_width, seg_color));
                    }

                    // --- DRAW SPARK HEAD & GLOW ---
                    Self::draw_glow(painter, head_pos, spark_size, spark_color, spark_alpha);
                    painter.circle_filled(
                        head_pos,
                        (spark_size * 0.6).max(0.5),
                        Color32::from_rgba_unmultiplied(255, 255, 255, (spark_alpha * 255.0) as u8),
                    );

                    // Occasional 8-point sparkle cross on bright leading sparks
                    if s % 5 == 0 && burst_p < 0.65 {
                        let sparkle_a = spark_alpha * (1.0 - burst_p / 0.65);
                        Self::draw_sparkle(
                            painter,
                            head_pos,
                            spark_size * 2.6,
                            Color32::from_rgba_unmultiplied(255, 255, 220, (sparkle_a * 220.0) as u8),
                            true,
                        );
                    }
                }

                // --- 3. INNER PISTIL CORE SHELL (For Chrysanthemum Style) ---
                if has_pistil && burst_p < 0.70 {
                    let pistil_p = burst_p / 0.70;
                    let pistil_alpha = (1.0 - pistil_p).powf(1.2);
                    let pistil_sparks = 30;
                    let pistil_speed = base_speed * 0.52;

                    for ps in 0..pistil_sparks {
                        let p_seed = seed_base.wrapping_mul(541).wrapping_add(ps as u32);
                        let u = Self::hash_range(p_seed, -1.0, 1.0);
                        let theta = (ps as f32) * 2.3999632;
                        let r3d = (1.0 - u * u).max(0.0).sqrt();

                        let p_vx = r3d * theta.cos() * pistil_speed;
                        let p_vy = u * pistil_speed;
                        let p_vz = r3d * theta.sin() * pistil_speed;

                        let (p_pos, p_depth) = Self::volumetric_spark_pos(
                            center,
                            p_vx,
                            p_vy,
                            p_vz,
                            spark_t,
                            gravity * 0.8,
                            drag * 1.3,
                        );

                        let p_col = secondary_color;
                        let p_size = (2.2 * (1.0 - pistil_p * 0.5) * p_depth).max(0.6) * scale;

                        painter.circle_filled(
                            p_pos,
                            p_size,
                            Color32::from_rgba_unmultiplied(
                                p_col.r(),
                                p_col.g(),
                                p_col.b(),
                                (pistil_alpha * 240.0) as u8,
                            ),
                        );
                        painter.circle_filled(
                            p_pos,
                            p_size * 0.5,
                            Color32::from_rgba_unmultiplied(255, 255, 255, (pistil_alpha * 255.0) as u8),
                        );
                    }
                }
            }
        }
    }

    // =========================================================================
    // 2. CONFETTI — 3D Tumbling Projection, Depth Layers, Wind, Ribbons
    // =========================================================================
    fn draw_confetti(painter: &egui::Painter, rect: Rect, t: f64, intensity: f32) {
        let piece_count = (40.0 * intensity).round() as usize;
        let scale = Self::scale(rect);
        let colors = [
            Color32::from_rgb(255, 50, 85),   // Hot Pink
            Color32::from_rgb(255, 200, 30),  // Marigold
            Color32::from_rgb(30, 200, 100),  // Emerald
            Color32::from_rgb(30, 160, 255),  // Sky Blue
            Color32::from_rgb(190, 70, 255),  // Violet
            Color32::from_rgb(255, 120, 40),  // Tangerine
            Color32::from_rgb(50, 230, 220),  // Teal
            Color32::from_rgb(255, 255, 255), // White
        ];

        // Global wind: slow sinusoidal horizontal force
        let wind = (t * 0.7).sin() as f32 * 0.03;

        // Draw 3 depth layers: back (small/muted), mid, front (large/vivid)
        for layer in 0..3_u32 {
            let layer_scale = match layer {
                0 => 0.55,  // back
                1 => 0.85,  // mid
                _ => 1.2,   // front
            };
            let layer_alpha = match layer {
                0 => 0.5,
                1 => 0.8,
                _ => 1.0,
            };
            let layer_speed = match layer {
                0 => 0.65,
                1 => 0.85,
                _ => 1.1,
            };
            let layer_offset = layer * piece_count as u32 / 3;

            let start = (layer_offset) as usize;
            let end = (start + piece_count / 3).min(piece_count);

            for i in start..end {
                let seed = Self::hash((i as u32).wrapping_mul(97).wrapping_add(23).wrapping_add(layer * 10000));
                let speed_mult = 0.6 + Self::hash_f(seed) * 0.8;
                let fall_period = (3.5 / (speed_mult * layer_speed)) as f64;
                let phase = Self::hash_f(seed.wrapping_add(1)) as f64 * fall_period;
                let local_t = (t + phase) % fall_period;
                let progress = (local_t / fall_period) as f32;

                // Position with wind drift and sinusoidal sway
                let x_base = Self::hash_range(seed.wrapping_add(2), 0.02, 0.98);
                let sway = (local_t * 3.2 + Self::hash_f(seed.wrapping_add(3)) as f64 * 10.0).sin() as f32 * 0.04;
                let px = rect.min.x + (x_base + sway + wind * progress).clamp(0.01, 0.99) * rect.width();
                let py = rect.min.y + progress * (rect.height() + 40.0 * scale) - 20.0 * scale;

                // 3D rotation angles (different rates for tumbling effect)
                let theta = (local_t * (4.0 + Self::hash_f(seed.wrapping_add(4)) as f64 * 3.0)) as f32;
                let phi = (local_t * (2.5 + Self::hash_f(seed.wrapping_add(5)) as f64 * 4.0)) as f32;

                // Piece dimensions
                let piece_type = Self::hash(seed.wrapping_add(6)) % 4; // 0-1: rect, 2: circle, 3: ribbon
                let color = colors[i % colors.len()];
                let a_mult = layer_alpha * (1.0 - (progress - 0.85).max(0.0) / 0.15); // fade at bottom
                let alpha = (a_mult * 255.0).clamp(0.0, 255.0) as u8;
                let piece_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
                let center = Pos2::new(px, py);

                match piece_type {
                    0 | 1 => {
                        // Tumbling rectangle — 3D projected parallelogram
                        let hw = (4.5 + (i % 3) as f32 * 1.5) * scale * layer_scale;
                        let hh = (3.0 + (i % 2) as f32 * 1.0) * scale * layer_scale;
                        let corners = Self::rotated_rect_corners(center, hw, hh, theta, phi);

                        painter.add(Shape::convex_polygon(
                            corners.to_vec(),
                            piece_color,
                            Stroke::NONE,
                        ));

                        // Glint highlight when roughly face-on (large projected area)
                        let projected_area = theta.cos().abs() * phi.cos().abs();
                        if projected_area > 0.85 {
                            let glint_a = ((projected_area - 0.85) / 0.15 * 120.0) as u8;
                            painter.add(Shape::convex_polygon(
                                corners.to_vec(),
                                Color32::from_rgba_unmultiplied(255, 255, 255, (glint_a as f32 * layer_alpha) as u8),
                                Stroke::NONE,
                            ));
                        }
                    }
                    2 => {
                        // Round confetti dot
                        let r = (3.0 + (i % 3) as f32 * 1.5) * scale * layer_scale;
                        painter.circle_filled(center, r, piece_color);
                        // Tiny highlight
                        painter.circle_filled(
                            Pos2::new(center.x - r * 0.3, center.y - r * 0.3),
                            r * 0.35,
                            Color32::from_rgba_unmultiplied(255, 255, 255, (alpha as f32 * 0.4) as u8),
                        );
                    }
                    _ => {
                        // Ribbon / streamer — wavy multi-segment line
                        let ribbon_len = (18.0 + Self::hash_f(seed.wrapping_add(7)) * 12.0) * scale * layer_scale;
                        let segments = 6;
                        let stroke_w = (2.0 + Self::hash_f(seed.wrapping_add(8)) * 1.5) * scale * layer_scale;
                        for seg in 0..segments {
                            let t0 = seg as f32 / segments as f32;
                            let t1 = (seg + 1) as f32 / segments as f32;
                            let wave0 = (t0 * 4.0 + theta).sin() * 4.0 * scale * layer_scale;
                            let wave1 = (t1 * 4.0 + theta).sin() * 4.0 * scale * layer_scale;
                            let p0 = Pos2::new(center.x + wave0, center.y + t0 * ribbon_len);
                            let p1 = Pos2::new(center.x + wave1, center.y + t1 * ribbon_len);
                            let seg_alpha = alpha as f32 * (1.0 - t1 * 0.3);
                            painter.line_segment(
                                [p0, p1],
                                Stroke::new(
                                    stroke_w * (1.0 - t1 * 0.3),
                                    Color32::from_rgba_unmultiplied(
                                        color.r(), color.g(), color.b(),
                                        seg_alpha.clamp(0.0, 255.0) as u8,
                                    ),
                                ),
                            );
                        }
                    }
                }
            }
        }
    }

    // =========================================================================
    // 3. FLOATING BALLOONS — Ellipse Polygon, 3-Layer Shading, Bezier String
    // =========================================================================
    fn draw_balloons(painter: &egui::Painter, rect: Rect, t: f64, intensity: f32) {
        let balloon_count = (10.0 * intensity).round() as usize;
        let scale = Self::scale(rect);
        let colors = [
            Color32::from_rgb(235, 40, 60),   // Cherry Red
            Color32::from_rgb(35, 150, 245),  // Sky Blue
            Color32::from_rgb(245, 185, 15),  // Sunny Yellow
            Color32::from_rgb(45, 200, 105),  // Mint Green
            Color32::from_rgb(180, 55, 230),  // Violet
            Color32::from_rgb(245, 110, 35),  // Orange
            Color32::from_rgb(245, 95, 170),  // Hot Pink
            Color32::from_rgb(95, 210, 230),  // Aqua
        ];

        for i in 0..balloon_count {
            let seed = Self::hash((i as u32).wrapping_mul(137).wrapping_add(43));
            let rise_period = (4.5 + Self::hash_f(seed) * 2.5) as f64;
            let phase = Self::hash_f(seed.wrapping_add(1)) as f64 * rise_period;
            let local_t = (t + phase) % rise_period;
            let progress = (local_t / rise_period) as f32;

            // Position with sway and bobbing
            let x_base = Self::hash_range(seed.wrapping_add(2), 0.08, 0.92);
            let sway = (local_t * 1.5 + Self::hash_f(seed.wrapping_add(3)) as f64 * 6.0).sin() as f32 * 0.04;
            let bob = (local_t * 3.5 + Self::hash_f(seed.wrapping_add(4)) as f64 * 4.0).sin() as f32 * 3.0 * scale;
            let px = rect.min.x + (x_base + sway).clamp(0.05, 0.95) * rect.width();
            let py = rect.max.y - progress * (rect.height() + 80.0 * scale) + 40.0 * scale + bob;

            // Size variation for depth illusion
            let size_mult = Self::hash_range(seed.wrapping_add(5), 0.7, 1.3);
            let rx = 13.0 * scale * size_mult;
            let ry = rx * 1.28; // slightly taller than wide

            // Tilt angle from sway
            let tilt = (local_t * 1.5 + Self::hash_f(seed.wrapping_add(3)) as f64 * 6.0).cos() as f32 * 0.12;

            let col = colors[i % colors.len()];
            let balloon_center = Pos2::new(px, py);

            // Layer 1: Dark shadow base (slightly offset down-right)
            let dark_col = Color32::from_rgb(
                (col.r() as f32 * 0.5) as u8,
                (col.g() as f32 * 0.5) as u8,
                (col.b() as f32 * 0.5) as u8,
            );
            let shadow_pts = Self::ellipse_points(
                Pos2::new(balloon_center.x + 1.5 * scale, balloon_center.y + 1.5 * scale),
                rx, ry, 20,
            );
            painter.add(Shape::convex_polygon(shadow_pts, dark_col, Stroke::NONE));

            // Layer 2: Main balloon body (ellipse polygon)
            // Apply tilt by slightly skewing the ellipse x-coordinates
            let body_pts: Vec<Pos2> = (0..20)
                .map(|j| {
                    let angle = (j as f32) * std::f32::consts::TAU / 20.0;
                    let bx = rx * angle.cos();
                    let by = ry * angle.sin();
                    // Apply tilt rotation
                    let tx = bx * tilt.cos() - by * tilt.sin();
                    let ty = bx * tilt.sin() + by * tilt.cos();
                    Pos2::new(balloon_center.x + tx, balloon_center.y + ty)
                })
                .collect();
            painter.add(Shape::convex_polygon(body_pts, col, Stroke::NONE));

            // Layer 3: Specular highlight (smaller ellipse, offset top-left, white with alpha)
            let hl_offset_x = -rx * 0.3;
            let hl_offset_y = -ry * 0.35;
            let hl_center = Pos2::new(balloon_center.x + hl_offset_x, balloon_center.y + hl_offset_y);
            let hl_pts = Self::ellipse_points(hl_center, rx * 0.4, ry * 0.35, 12);
            painter.add(Shape::convex_polygon(
                hl_pts,
                Color32::from_rgba_unmultiplied(255, 255, 255, 110),
                Stroke::NONE,
            ));

            // Balloon knot (small triangle at bottom)
            let knot_y = balloon_center.y + ry * 0.9;
            let knot_pts = vec![
                Pos2::new(px - 2.5 * scale, knot_y),
                Pos2::new(px + 2.5 * scale, knot_y),
                Pos2::new(px, knot_y + 4.0 * scale),
            ];
            painter.add(Shape::convex_polygon(knot_pts, col, Stroke::NONE));

            // Curved string using cubic bezier (sampled as line segments)
            let string_start = Pos2::new(px, knot_y + 4.0 * scale);
            let string_sway = (local_t * 2.5 + Self::hash_f(seed.wrapping_add(6)) as f64 * 5.0).sin() as f32;
            let string_len = 28.0 * scale;
            let string_end = Pos2::new(px + string_sway * 6.0 * scale, string_start.y + string_len);
            let cp1 = Pos2::new(
                px + string_sway * 3.0 * scale,
                string_start.y + string_len * 0.35,
            );
            let cp2 = Pos2::new(
                px + string_sway * 5.0 * scale,
                string_start.y + string_len * 0.7,
            );

            let string_segments = 6;
            let string_stroke = Stroke::new(
                0.9 * scale,
                Color32::from_rgba_unmultiplied(200, 200, 220, 160),
            );
            for seg in 0..string_segments {
                let t0 = seg as f32 / string_segments as f32;
                let t1 = (seg + 1) as f32 / string_segments as f32;
                let p0 = Self::bezier(string_start, cp1, cp2, string_end, t0);
                let p1 = Self::bezier(string_start, cp1, cp2, string_end, t1);
                painter.line_segment([p0, p1], string_stroke);
            }
        }
    }

    // =========================================================================
    // 4. FLYING BIRDS — Multi-Segment Curved Wings, Body, Depth Variation
    // =========================================================================
    fn draw_birds(painter: &egui::Painter, rect: Rect, t: f64, intensity: f32) {
        let flock_size = (8.0 * intensity).round() as usize;
        let loop_period = 7.0;
        let scale = Self::scale(rect);

        for i in 0..flock_size {
            let seed = Self::hash((i as u32).wrapping_mul(71).wrapping_add(19));
            let phase = Self::hash_f(seed) as f64 * loop_period;
            let local_t = (t + phase) % loop_period;
            let progress = (local_t / loop_period) as f32;

            // Depth layer: smaller/fainter birds in the back
            let depth = Self::hash_range(seed.wrapping_add(1), 0.5, 1.2);
            let bird_alpha = (depth * 220.0).clamp(80.0, 240.0) as u8;

            // Position with undulating flight path
            let y_base = Self::hash_range(seed.wrapping_add(2), 0.08, 0.75);
            let bob = (local_t * 2.2 + Self::hash_f(seed.wrapping_add(3)) as f64 * 5.0).sin() as f32 * 0.025;
            let px = rect.min.x - 50.0 * scale + progress * (rect.width() + 100.0 * scale);
            let py = rect.min.y + (y_base + bob).clamp(0.03, 0.92) * rect.height();

            // Wing span scales with depth
            let wing_span = (16.0 + Self::hash_f(seed.wrapping_add(4)) * 8.0) * scale * depth;

            // Smooth wing flap: sinusoidal with asymmetric up/down timing
            let flap_raw = (local_t * 6.0 + Self::hash_f(seed.wrapping_add(5)) as f64 * 3.0).sin() as f32;
            let flap = flap_raw * 0.7; // amplitude: ±0.7 radians

            let center = Pos2::new(px, py);
            let bird_color = Color32::from_rgba_unmultiplied(
                (35.0 + depth * 25.0) as u8,
                (35.0 + depth * 30.0) as u8,
                (45.0 + depth * 35.0) as u8,
                bird_alpha,
            );
            let wing_stroke = Stroke::new((2.0 + depth * 0.8) * scale, bird_color);

            // Multi-segment curved wings (3 segments per wing, with flap displacement)
            // Left wing: tip → mid → inner → body
            let tip_y_offset = -flap * wing_span * 0.55;
            let mid_y_offset = -flap * wing_span * 0.3;
            let inner_y_offset = -flap * wing_span * 0.1;

            let left_pts = [
                Pos2::new(center.x - wing_span, center.y + tip_y_offset),
                Pos2::new(center.x - wing_span * 0.62, center.y + mid_y_offset + 1.5 * scale),
                Pos2::new(center.x - wing_span * 0.28, center.y + inner_y_offset + 0.5 * scale),
                center,
            ];
            let right_pts = [
                center,
                Pos2::new(center.x + wing_span * 0.28, center.y + inner_y_offset + 0.5 * scale),
                Pos2::new(center.x + wing_span * 0.62, center.y + mid_y_offset + 1.5 * scale),
                Pos2::new(center.x + wing_span, center.y + tip_y_offset),
            ];

            // Draw wing curves as connected segments
            for seg in 0..3 {
                painter.line_segment([left_pts[seg], left_pts[seg + 1]], wing_stroke);
                painter.line_segment([right_pts[seg], right_pts[seg + 1]], wing_stroke);
            }

            // Body oval (small ellipse at center)
            let body_r = wing_span * 0.08;
            painter.circle_filled(center, body_r, bird_color);

            // Head / beak (tiny forward protrusion)
            let head_pos = Pos2::new(center.x + wing_span * 0.12, center.y - body_r * 0.3);
            painter.circle_filled(head_pos, body_r * 0.6, bird_color);

            // Tail feathers (small V behind body)
            let tail_len = wing_span * 0.15;
            let tail_base = Pos2::new(center.x - wing_span * 0.1, center.y);
            painter.line_segment(
                [tail_base, Pos2::new(tail_base.x - tail_len, tail_base.y - tail_len * 0.4)],
                Stroke::new(wing_stroke.width * 0.6, bird_color),
            );
            painter.line_segment(
                [tail_base, Pos2::new(tail_base.x - tail_len, tail_base.y + tail_len * 0.4)],
                Stroke::new(wing_stroke.width * 0.6, bird_color),
            );
        }
    }

    // =========================================================================
    // 5. CLAPPING APPLAUSE — Palm Polygons, Shockwave, Sparkle Burst, Pulse
    // =========================================================================
    fn draw_clapping(painter: &egui::Painter, rect: Rect, t: f64, intensity: f32) {
        let clap_pairs = (5.0 * intensity).round() as usize;
        let clap_period = 0.85;
        let scale = Self::scale(rect);

        for i in 0..clap_pairs {
            let seed = Self::hash((i as u32).wrapping_mul(89).wrapping_add(31));
            let offset = Self::hash_f(seed) as f64 * clap_period;
            let local_t = (t + offset) % clap_period;
            let clap_p = (local_t / clap_period) as f32;

            let cx_norm = Self::hash_range(seed.wrapping_add(1), 0.15, 0.85);
            let cy_norm = Self::hash_range(seed.wrapping_add(2), 0.30, 0.80);
            let center = Pos2::new(
                rect.min.x + cx_norm * rect.width(),
                rect.min.y + cy_norm * rect.height(),
            );

            // Impact timing: quick clap at start, slow release
            let impact = (1.0 - (clap_p * 3.0).min(1.0)).powi(2);
            let size = (24.0 + impact * 6.0) * scale;
            let hand_sep = (1.0 - impact) * 14.0 * scale;

            // Scale pulse: hands enlarge briefly on impact
            let pulse = if impact > 0.5 { 1.0 + (impact - 0.5) * 0.3 } else { 1.0 };
            let palm_r = size * 0.42 * pulse;

            let left_center = Pos2::new(center.x - hand_sep, center.y);
            let right_center = Pos2::new(center.x + hand_sep, center.y);

            // Palm ovals (slightly taller than wide)
            let palm_col = Color32::from_rgb(255, 218, 170);
            let palm_dark = Color32::from_rgb(220, 185, 145);

            // Left palm: dark base then lighter overlay
            let left_base = Self::ellipse_points(
                Pos2::new(left_center.x + 0.5 * scale, left_center.y + 0.5 * scale),
                palm_r * 0.95, palm_r * 1.1, 12,
            );
            painter.add(Shape::convex_polygon(left_base, palm_dark, Stroke::NONE));
            let left_pts = Self::ellipse_points(left_center, palm_r * 0.95, palm_r * 1.1, 12);
            painter.add(Shape::convex_polygon(left_pts, palm_col, Stroke::NONE));

            // Right palm
            let right_base = Self::ellipse_points(
                Pos2::new(right_center.x + 0.5 * scale, right_center.y + 0.5 * scale),
                palm_r * 0.95, palm_r * 1.1, 12,
            );
            painter.add(Shape::convex_polygon(right_base, palm_dark, Stroke::NONE));
            let right_pts = Self::ellipse_points(right_center, palm_r * 0.95, palm_r * 1.1, 12);
            painter.add(Shape::convex_polygon(right_pts, palm_col, Stroke::NONE));

            // Finger stubs on each palm (3 small circles on top)
            for f in 0..3 {
                let fx = (f as f32 - 1.0) * palm_r * 0.5;
                let fy = -palm_r * 0.9;
                let finger_r = palm_r * 0.22;
                // Left hand fingers
                painter.circle_filled(
                    Pos2::new(left_center.x + fx, left_center.y + fy),
                    finger_r,
                    palm_col,
                );
                // Right hand fingers
                painter.circle_filled(
                    Pos2::new(right_center.x + fx, right_center.y + fy),
                    finger_r,
                    palm_col,
                );
            }

            // Impact effects (only during clap contact)
            if impact > 0.1 {
                // Expanding shockwave ring
                let ring_progress = 1.0 - impact;
                let ring_r = size * 0.6 + ring_progress * 25.0 * scale;
                let ring_alpha = (impact * 200.0) as u8;
                painter.circle_stroke(
                    center,
                    ring_r,
                    Stroke::new(
                        (2.5 * impact).max(0.5) * scale,
                        Color32::from_rgba_unmultiplied(255, 230, 80, ring_alpha),
                    ),
                );

                // Second ring (delayed, fainter)
                if ring_progress > 0.15 {
                    let ring2_r = size * 0.6 + (ring_progress - 0.15) * 30.0 * scale;
                    let ring2_a = (impact * 100.0) as u8;
                    painter.circle_stroke(
                        center,
                        ring2_r,
                        Stroke::new(
                            (1.5 * impact).max(0.3) * scale,
                            Color32::from_rgba_unmultiplied(255, 255, 200, ring2_a),
                        ),
                    );
                }

                // Sparkle burst: 8 particles flying outward
                for r in 0..8 {
                    let angle = (r as f32) * (std::f32::consts::TAU / 8.0)
                        + Self::hash_range(seed.wrapping_add(r as u32 + 100), -0.2, 0.2)
                        + (t * 1.5) as f32;
                    let dist = (size * 0.5 + (1.0 - impact) * 18.0 * scale)
                        * Self::hash_range(seed.wrapping_add(r as u32 + 200), 0.7, 1.3);
                    let spark_pos = Pos2::new(
                        center.x + angle.cos() * dist,
                        center.y + angle.sin() * dist,
                    );
                    let spark_a = impact * 255.0;
                    let spark_r = (2.0 * impact).max(0.5) * scale;
                    painter.circle_filled(
                        spark_pos,
                        spark_r,
                        Color32::from_rgba_unmultiplied(255, 240, 100, spark_a.clamp(0.0, 255.0) as u8),
                    );
                }

                // Sparkle cross at impact center
                if impact > 0.3 {
                    Self::draw_sparkle(
                        painter,
                        center,
                        size * 0.5 * impact,
                        Color32::from_rgba_unmultiplied(255, 255, 180, (impact * 200.0) as u8),
                        true,
                    );
                }
            }

            // Musical note glyphs floating upward (after clap)
            if clap_p > 0.3 && clap_p < 0.9 {
                let note_p = (clap_p - 0.3) / 0.6;
                let note_alpha = ((1.0 - note_p) * 200.0) as u8;
                let notes = ["♪", "♫"];
                for n in 0..2_u32 {
                    let nx = center.x + Self::hash_range(seed.wrapping_add(n + 300), -15.0, 15.0) * scale;
                    let ny = center.y - note_p * 30.0 * scale - n as f32 * 8.0 * scale;
                    let note_sway = (t * 3.0 + n as f64 * 1.5).sin() as f32 * 4.0 * scale;
                    painter.text(
                        Pos2::new(nx + note_sway, ny),
                        egui::Align2::CENTER_CENTER,
                        notes[n as usize % 2],
                        egui::FontId::proportional(11.0 * scale),
                        Color32::from_rgba_unmultiplied(255, 230, 100, note_alpha),
                    );
                }
            }
        }
    }

    // =========================================================================
    // 6. SHOOTING STARS — Tapered Trail, Glow Halo, Sparkle Scatter, Arc
    // =========================================================================
    fn draw_shooting_stars(painter: &egui::Painter, rect: Rect, t: f64, intensity: f32) {
        let star_count = (4.0 * intensity).round() as usize;
        let star_period = 2.8;
        let scale = Self::scale(rect);

        for i in 0..star_count {
            let seed = Self::hash((i as u32).wrapping_mul(73).wrapping_add(11));
            let offset = Self::hash_f(seed) as f64 * star_period;
            let local_t = (t + offset) % star_period;
            let progress = (local_t / star_period) as f32;

            // Only visible for first 70% of period (rest is gap)
            if progress > 0.70 {
                continue;
            }
            let streak_p = progress / 0.70;

            // Start position and trajectory (slight arc)
            let start_x = Self::hash_range(seed.wrapping_add(1), 0.08, 0.65);
            let start_y = Self::hash_range(seed.wrapping_add(2), 0.03, 0.30);
            let streak_len = Self::hash_range(seed.wrapping_add(3), 0.30, 0.45);

            // Acceleration (star speeds up as it falls)
            let accel_p = streak_p * (1.0 + streak_p * 0.4); // quadratic acceleration
            let arc_curvature = Self::hash_range(seed.wrapping_add(4), 0.02, 0.08);

            let head_x = start_x + accel_p * streak_len;
            let head_y = start_y + accel_p * (streak_len * 0.6) + accel_p.powi(2) * arc_curvature;
            let head = Pos2::new(
                rect.min.x + head_x * rect.width(),
                rect.min.y + head_y * rect.height(),
            );

            let alpha_factor = (1.0 - streak_p).powf(0.6);

            // Tapered multi-segment trail (15 segments, decreasing width + alpha + color shift)
            let trail_segments = 15;
            let _trail_len = 75.0 * scale;
            for seg in 0..trail_segments {
                let t0 = seg as f32 / trail_segments as f32;
                let t1 = (seg + 1) as f32 / trail_segments as f32;

                // Trail position: compute backwards along trajectory
                let back0 = accel_p - t0 * (accel_p * 0.25);
                let back1 = accel_p - t1 * (accel_p * 0.25);
                let tx0 = start_x + back0 * streak_len;
                let ty0 = start_y + back0 * (streak_len * 0.6) + back0.powi(2) * arc_curvature;
                let tx1 = start_x + back1 * streak_len;
                let ty1 = start_y + back1 * (streak_len * 0.6) + back1.powi(2) * arc_curvature;

                let p0 = Pos2::new(
                    rect.min.x + tx0 * rect.width(),
                    rect.min.y + ty0 * rect.height(),
                );
                let p1 = Pos2::new(
                    rect.min.x + tx1 * rect.width(),
                    rect.min.y + ty1 * rect.height(),
                );

                // Width tapers from thick at head to thin at tail
                let width = (3.5 * (1.0 - t1 * 0.85)).max(0.3) * scale;
                // Color shifts from white → cyan → transparent
                let seg_alpha = (alpha_factor * (1.0 - t1).powf(1.2) * 255.0).clamp(0.0, 255.0) as u8;
                let r = (255.0 * (1.0 - t1 * 0.55)) as u8;
                let g = (255.0 * (1.0 - t1 * 0.15)) as u8;
                let b = 255_u8;
                let trail_color = Color32::from_rgba_unmultiplied(r, g, b, seg_alpha);

                painter.line_segment([p0, p1], Stroke::new(width, trail_color));
            }

            // Glow halo at star head (3 layers)
            Self::draw_glow(painter, head, 3.5 * scale, Color32::from_rgb(200, 240, 255), alpha_factor);
            // Bright white core
            painter.circle_filled(
                head,
                2.5 * scale,
                Color32::from_rgba_unmultiplied(255, 255, 255, (alpha_factor * 255.0) as u8),
            );

            // Sparkle scatter: tiny particles shed perpendicular to trail
            let scatter_count = 6;
            for sc in 0..scatter_count {
                let sc_seed = seed.wrapping_mul(17).wrapping_add(sc);
                let sc_t = Self::hash_range(sc_seed, 0.05, 0.5);
                let sc_perp = Self::hash_range(sc_seed.wrapping_add(1), -1.0, 1.0) * 8.0 * scale;
                let sc_age = (streak_p - sc_t * streak_p).max(0.0);

                if sc_age < 0.01 || sc_age > 0.3 {
                    continue;
                }

                let sc_back = accel_p - sc_t * (accel_p * 0.25);
                let sc_base_x = start_x + sc_back * streak_len;
                let sc_base_y = start_y + sc_back * (streak_len * 0.6) + sc_back.powi(2) * arc_curvature;

                // Offset perpendicular to trail direction + drift downward with age
                let sc_pos = Pos2::new(
                    rect.min.x + sc_base_x * rect.width() + sc_perp,
                    rect.min.y + sc_base_y * rect.height() + sc_age * 15.0 * scale,
                );

                let sc_alpha = ((1.0 - sc_age / 0.3) * alpha_factor * 200.0) as u8;
                let sc_r = (1.5 * (1.0 - sc_age / 0.3)).max(0.4) * scale;
                painter.circle_filled(
                    sc_pos,
                    sc_r,
                    Color32::from_rgba_unmultiplied(220, 240, 255, sc_alpha),
                );
            }

            // Twinkling endpoint sparkle cross
            if streak_p < 0.3 {
                let twinkle = ((t * 12.0 + offset * 5.0).sin() as f32 * 0.5 + 0.5) * alpha_factor;
                Self::draw_sparkle(
                    painter,
                    head,
                    6.0 * scale * twinkle,
                    Color32::from_rgba_unmultiplied(255, 255, 255, (twinkle * 200.0) as u8),
                    true,
                );
            }
        }
    }
}
