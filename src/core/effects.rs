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

                        // Micro-glitter stardust shedding along trail
                        if seg > 1 && seg % 2 == 0 {
                            let glitter_seed = spark_seed.wrapping_mul(97).wrapping_add(seg as u32);
                            let lateral_x = Self::hash_range(glitter_seed, -4.0, 4.0) * scale;
                            let lateral_y = Self::hash_range(glitter_seed.wrapping_add(1), -2.0, 4.0) * scale;
                            let glitter_pos = Pos2::new(p_a.x + lateral_x, p_a.y + lateral_y);
                            let strobe_twinkle = (t * 50.0 + glitter_seed as f64 * 1.5).sin() as f32 * 0.5 + 0.5;
                            let glitter_alpha = (seg_alpha * strobe_twinkle * 220.0).clamp(0.0, 255.0) as u8;
                            painter.circle_filled(
                                glitter_pos,
                                0.9 * scale * depth_a,
                                Color32::from_rgba_unmultiplied(255, 235, 160, glitter_alpha),
                            );
                        }
                    }

                    // --- DRAW SPARK HEAD & GLOW ---
                    Self::draw_glow(painter, head_pos, spark_size, spark_color, spark_alpha);
                    painter.circle_filled(
                        head_pos,
                        (spark_size * 0.65).max(0.6),
                        Color32::from_rgba_unmultiplied(255, 255, 255, (spark_alpha * 255.0) as u8),
                    );

                    // Optical 8-point diffraction spike on leading bright stars
                    if s % 4 == 0 && burst_p < 0.60 {
                        let sparkle_a = spark_alpha * (1.0 - burst_p / 0.60);
                        Self::draw_sparkle(
                            painter,
                            head_pos,
                            spark_size * 3.0,
                            Color32::from_rgba_unmultiplied(255, 255, 225, (sparkle_a * 235.0) as u8),
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
    // 2. CONFETTI — 3D Tumbling Projection, Metallic Foil, Spiral Serpentine
    // =========================================================================
    fn draw_confetti(painter: &egui::Painter, rect: Rect, t: f64, intensity: f32) {
        let piece_count = (48.0 * intensity).round() as usize;
        let scale = Self::scale(rect);

        // Rich palette including metallic foil & party colors
        let colors = [
            Color32::from_rgb(255, 215, 0),   // Metallic Gold Foil
            Color32::from_rgb(230, 240, 255), // Platinum Silver Foil
            Color32::from_rgb(255, 45, 85),   // Hot Crimson Pink
            Color32::from_rgb(30, 210, 110),  // Emerald Green
            Color32::from_rgb(30, 160, 255),  // Electric Cyan
            Color32::from_rgb(195, 65, 255),  // Holographic Violet
            Color32::from_rgb(255, 130, 30),  // Tangerine
            Color32::from_rgb(255, 255, 255), // Bright White
        ];

        // Global wind: sinusoidal horizontal drift with micro-vortices
        let wind = (t * 0.75).sin() as f32 * 0.035;

        // Draw 3 depth layers: back (small/muted), mid, front (large/vivid)
        for layer in 0..3_u32 {
            let layer_scale = match layer {
                0 => 0.55,  // back
                1 => 0.85,  // mid
                _ => 1.25,  // front
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

            let start = layer_offset as usize;
            let end = (start + piece_count / 3).min(piece_count);

            for i in start..end {
                let seed = Self::hash((i as u32).wrapping_mul(97).wrapping_add(23).wrapping_add(layer * 10000));
                let speed_mult = 0.6 + Self::hash_f(seed) * 0.8;
                let fall_period = (3.4 / (speed_mult * layer_speed)) as f64;
                let phase = Self::hash_f(seed.wrapping_add(1)) as f64 * fall_period;
                let local_t = (t + phase) % fall_period;
                let progress = (local_t / fall_period) as f32;

                // Position with wind drift and sinusoidal sway
                let x_base = Self::hash_range(seed.wrapping_add(2), 0.02, 0.98);
                let sway = (local_t * 3.4 + Self::hash_f(seed.wrapping_add(3)) as f64 * 10.0).sin() as f32 * 0.045;
                let px = rect.min.x + (x_base + sway + wind * progress).clamp(0.01, 0.99) * rect.width();
                let py = rect.min.y + progress * (rect.height() + 45.0 * scale) - 20.0 * scale;

                // 3D rotation angles
                let theta = (local_t * (4.2 + Self::hash_f(seed.wrapping_add(4)) as f64 * 3.2)) as f32;
                let phi = (local_t * (2.8 + Self::hash_f(seed.wrapping_add(5)) as f64 * 4.0)) as f32;

                let piece_type = Self::hash(seed.wrapping_add(6)) % 4; // 0-1: rect, 2: circle, 3: spiral ribbon
                let color = colors[i % colors.len()];
                let a_mult = layer_alpha * (1.0 - (progress - 0.86).max(0.0) / 0.14);
                let alpha = (a_mult * 255.0).clamp(0.0, 255.0) as u8;
                let piece_color = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha);
                let center = Pos2::new(px, py);

                match piece_type {
                    0 | 1 => {
                        // 3D Tumbling rectangle — projected parallelogram
                        let hw = (4.8 + (i % 3) as f32 * 1.6) * scale * layer_scale;
                        let hh = (3.2 + (i % 2) as f32 * 1.1) * scale * layer_scale;
                        let corners = Self::rotated_rect_corners(center, hw, hh, theta, phi);

                        painter.add(Shape::convex_polygon(
                            corners.to_vec(),
                            piece_color,
                            Stroke::NONE,
                        ));

                        // Metallic foil glint flare when face-on
                        let projected_area = theta.cos().abs() * phi.cos().abs();
                        if projected_area > 0.82 {
                            let glint_a = ((projected_area - 0.82) / 0.18 * 160.0) as u8;
                            painter.add(Shape::convex_polygon(
                                corners.to_vec(),
                                Color32::from_rgba_unmultiplied(255, 255, 255, (glint_a as f32 * layer_alpha) as u8),
                                Stroke::NONE,
                            ));
                            // Tiny sparkle star on center of foil
                            if layer == 2 && projected_area > 0.92 {
                                Self::draw_sparkle(
                                    painter,
                                    center,
                                    hw * 1.5,
                                    Color32::from_rgba_unmultiplied(255, 255, 240, (glint_a as f32 * layer_alpha) as u8),
                                    false,
                                );
                            }
                        }
                    }
                    2 => {
                        // Round metallic foil dot
                        let r = (3.2 + (i % 3) as f32 * 1.5) * scale * layer_scale;
                        painter.circle_filled(center, r, piece_color);
                        // Specular gloss dot
                        painter.circle_filled(
                            Pos2::new(center.x - r * 0.32, center.y - r * 0.32),
                            r * 0.35,
                            Color32::from_rgba_unmultiplied(255, 255, 255, (alpha as f32 * 0.55) as u8),
                        );
                    }
                    _ => {
                        // 3D Curling spiral serpentine streamer
                        let ribbon_len = (22.0 + Self::hash_f(seed.wrapping_add(7)) * 14.0) * scale * layer_scale;
                        let segments = 8;
                        let stroke_w = (2.2 + Self::hash_f(seed.wrapping_add(8)) * 1.6) * scale * layer_scale;
                        let coil_radius = 5.0 * scale * layer_scale;

                        for seg in 0..segments {
                            let t0 = seg as f32 / segments as f32;
                            let t1 = (seg + 1) as f32 / segments as f32;

                            // 3D helical coil
                            let angle0 = t0 * 6.28 + theta;
                            let angle1 = t1 * 6.28 + theta;
                            let wave0 = angle0.sin() * coil_radius;
                            let wave1 = angle1.sin() * coil_radius;

                            let p0 = Pos2::new(center.x + wave0, center.y + t0 * ribbon_len);
                            let p1 = Pos2::new(center.x + wave1, center.y + t1 * ribbon_len);

                            let seg_alpha = alpha as f32 * (1.0 - t1 * 0.25);
                            let depth_factor = (angle0.cos() * 0.3 + 0.7).clamp(0.4, 1.0);

                            painter.line_segment(
                                [p0, p1],
                                Stroke::new(
                                    stroke_w * depth_factor,
                                    Color32::from_rgba_unmultiplied(
                                        color.r(),
                                        color.g(),
                                        color.b(),
                                        (seg_alpha * depth_factor).clamp(0.0, 255.0) as u8,
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
    // 3. FLOATING BALLOONS — Glossy Latex 3D Material, Teardrop, Bounce Rim
    // =========================================================================
    fn draw_balloons(painter: &egui::Painter, rect: Rect, t: f64, intensity: f32) {
        let balloon_count = (10.0 * intensity).round() as usize;
        let scale = Self::scale(rect);

        // Rich, saturated party balloon colors with dark shadow tones
        let colors = [
            (Color32::from_rgb(235, 30, 50), Color32::from_rgb(140, 10, 25)),   // Ruby Red
            (Color32::from_rgb(30, 145, 255), Color32::from_rgb(15, 75, 160)),  // Electric Royal Blue
            (Color32::from_rgb(255, 195, 10), Color32::from_rgb(165, 115, 5)),  // Bright Gold
            (Color32::from_rgb(40, 205, 100), Color32::from_rgb(15, 110, 50)),  // Emerald Green
            (Color32::from_rgb(185, 50, 240), Color32::from_rgb(105, 20, 150)), // Vibrant Purple
            (Color32::from_rgb(255, 105, 30), Color32::from_rgb(160, 50, 10)),  // Tangerine Orange
            (Color32::from_rgb(255, 80, 170), Color32::from_rgb(160, 25, 95)),  // Hot Magenta
            (Color32::from_rgb(40, 215, 230), Color32::from_rgb(15, 115, 130)), // Cyan Aqua
        ];

        for i in 0..balloon_count {
            let seed = Self::hash((i as u32).wrapping_mul(137).wrapping_add(43));
            let rise_period = (4.8 + Self::hash_f(seed) * 2.2) as f64;
            let phase = Self::hash_f(seed.wrapping_add(1)) as f64 * rise_period;
            let local_t = (t + phase) % rise_period;
            let progress = (local_t / rise_period) as f32;

            // Lateral sway & harmonic buoyancy bobbing
            let x_base = Self::hash_range(seed.wrapping_add(2), 0.06, 0.94);
            let sway = (local_t * 1.4 + Self::hash_f(seed.wrapping_add(3)) as f64 * 6.0).sin() as f32 * 0.038;
            let bob = (local_t * 3.2 + Self::hash_f(seed.wrapping_add(4)) as f64 * 4.0).sin() as f32 * 3.5 * scale;
            let px = rect.min.x + (x_base + sway).clamp(0.04, 0.96) * rect.width();
            let py = rect.max.y - progress * (rect.height() + 90.0 * scale) + 45.0 * scale + bob;

            let size_mult = Self::hash_range(seed.wrapping_add(5), 0.75, 1.25);
            let rx = 14.0 * scale * size_mult;
            let ry = rx * 1.30;

            // Tilt angle from aerodynamic sway
            let tilt = (local_t * 1.4 + Self::hash_f(seed.wrapping_add(3)) as f64 * 6.0).cos() as f32 * 0.12;

            let (main_col, dark_col) = colors[i % colors.len()];
            let balloon_center = Pos2::new(px, py);

            // 1. Teardrop balloon geometry (24 perimeter points: wide top dome, pinched neck)
            let num_pts = 24;
            let body_pts: Vec<Pos2> = (0..num_pts)
                .map(|j| {
                    let angle = (j as f32) * std::f32::consts::TAU / (num_pts as f32);
                    let sin_a = angle.sin();
                    let cos_a = angle.cos();

                    // Lower hemisphere tapers into teardrop neck
                    let r_mod = if sin_a > 0.0 {
                        rx * (1.0 - 0.22 * sin_a)
                    } else {
                        rx
                    };

                    let bx = r_mod * cos_a;
                    let by = ry * sin_a;

                    // Apply tilt skew
                    let tx = bx * tilt.cos() - by * tilt.sin();
                    let ty = bx * tilt.sin() + by * tilt.cos();
                    Pos2::new(balloon_center.x + tx, balloon_center.y + ty)
                })
                .collect();

            // Layer 1: Dark base shadow (ambient occlusion on lower-right)
            let shadow_pts: Vec<Pos2> = body_pts
                .iter()
                .map(|p| Pos2::new(p.x + 1.2 * scale, p.y + 1.5 * scale))
                .collect();
            painter.add(Shape::convex_polygon(shadow_pts, dark_col, Stroke::NONE));

            // Layer 2: Glossy balloon body polygon with translucent Fresnel rim stroke
            painter.add(Shape::convex_polygon(
                body_pts.clone(),
                main_col,
                Stroke::new(1.2 * scale, Color32::from_rgba_unmultiplied(255, 255, 255, 55)),
            ));

            // Layer 3: Curved Specular Crescent Highlight (Glossy Studio Light Reflection)
            let hl_center = Pos2::new(balloon_center.x - rx * 0.32, balloon_center.y - ry * 0.36);
            let hl_pts = Self::ellipse_points(hl_center, rx * 0.38, ry * 0.32, 16);
            painter.add(Shape::convex_polygon(
                hl_pts,
                Color32::from_rgba_unmultiplied(255, 255, 255, 175),
                Stroke::NONE,
            ));
            // Secondary small pinpoint glint
            painter.circle_filled(
                Pos2::new(hl_center.x - rx * 0.12, hl_center.y - ry * 0.14),
                rx * 0.16,
                Color32::from_rgba_unmultiplied(255, 255, 255, 240),
            );

            // Layer 4: Secondary Ambient Bounce Glint (Soft rim light on lower-right edge)
            let bounce_center = Pos2::new(balloon_center.x + rx * 0.30, balloon_center.y + ry * 0.35);
            let bounce_pts = Self::ellipse_points(bounce_center, rx * 0.28, ry * 0.22, 12);
            painter.add(Shape::convex_polygon(
                bounce_pts,
                Color32::from_rgba_unmultiplied(255, 255, 255, 60),
                Stroke::NONE,
            ));

            // Layer 5: Gathered Latex Neck Bead & Tied Knot
            let knot_y = balloon_center.y + ry * 0.95;
            let bead_center = Pos2::new(px, knot_y);
            // Rolled rubber bead lip
            painter.circle_filled(bead_center, 3.2 * scale, dark_col);
            painter.circle_filled(bead_center, 2.5 * scale, main_col);

            // Tied triangular knot
            let knot_pts = vec![
                Pos2::new(px - 3.2 * scale, knot_y + 1.5 * scale),
                Pos2::new(px + 3.2 * scale, knot_y + 1.5 * scale),
                Pos2::new(px, knot_y + 5.5 * scale),
            ];
            painter.add(Shape::convex_polygon(knot_pts, main_col, Stroke::NONE));

            // Layer 6: Cubic Bezier dangling curly ribbon string
            let string_start = Pos2::new(px, knot_y + 5.5 * scale);
            let string_sway = (local_t * 2.8 + Self::hash_f(seed.wrapping_add(6)) as f64 * 5.0).sin() as f32;
            let string_len = 32.0 * scale;
            let string_end = Pos2::new(px + string_sway * 7.0 * scale, string_start.y + string_len);
            let cp1 = Pos2::new(
                px + string_sway * 3.5 * scale,
                string_start.y + string_len * 0.33,
            );
            let cp2 = Pos2::new(
                px - string_sway * 4.0 * scale,
                string_start.y + string_len * 0.68,
            );

            let string_segments = 7;
            let string_stroke = Stroke::new(
                1.0 * scale,
                Color32::from_rgba_unmultiplied(220, 220, 235, 175),
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
    // =========================================================================
    // 4. FLYING BIRDS — Aerodynamic V-Formation Flock & Wingbeat Physics
    // =========================================================================
    fn draw_birds(painter: &egui::Painter, rect: Rect, t: f64, intensity: f32) {
        let scale = Self::scale(rect);

        // 2 Coordinated Flocks:
        // Flock 0: Primary foreground V-formation (7–9 birds, crisp & prominent)
        // Flock 1: Distant background secondary V-formation (5 birds, faint & higher in sky)
        let flock_configs = [
            // (flock_idx, bird_count, loop_period, phase_offset, y_base, depth_scale, color)
            (
                0_u32,
                (7.0 * intensity).round().clamp(5.0, 9.0) as usize,
                7.2_f64,
                0.0_f64,
                0.26_f32,
                1.0_f32,
                Color32::from_rgba_unmultiplied(28, 34, 48, 235),
            ),
            (
                1_u32,
                (5.0 * intensity).round().clamp(3.0, 5.0) as usize,
                9.4_f64,
                3.8_f64,
                0.14_f32,
                0.58_f32,
                Color32::from_rgba_unmultiplied(75, 92, 122, 160),
            ),
        ];

        for &(flock_idx, bird_count, loop_period, phase_offset, y_base, depth_scale, bird_color) in &flock_configs {
            let flock_seed = (flock_idx + 1).wrapping_mul(317);
            let local_t = (t + phase_offset) % loop_period;
            let progress = (local_t / loop_period) as f32;

            // Collective flock flight corridor (smooth swooping undulation across the sky)
            let undulate_y = ((t * 1.1 + flock_idx as f64 * 1.4).sin() as f32 * 12.0
                + (t * 0.45).cos() as f32 * 8.0)
                * scale
                * depth_scale;

            // Apex Leader Position (spans across the screen with margin)
            let leader_x = rect.min.x - 140.0 * scale * depth_scale
                + progress * (rect.width() + 280.0 * scale * depth_scale);
            let leader_y = rect.min.y + y_base * rect.height() + undulate_y;

            // Flap-Glide Duty Cycle:
            // 2.4s total cycle: 1.5s active powerstroke flapping, 0.9s flat-winged soaring glide
            let glide_period = 2.4;
            let glide_cycle_t = (t + flock_idx as f64 * 0.7) % glide_period;
            let glide_envelope = if glide_cycle_t < 1.5 {
                1.0_f32
            } else {
                // Smooth transition into flat-winged aerodynamic glide
                ((1.0 - (glide_cycle_t - 1.5) / 0.9) * std::f64::consts::PI).sin().powi(2) as f32 * 0.35 + 0.05
            };

            // Flapping frequency
            let flap_freq = 19.5_f64; // ~3.1 flaps/sec
            let phase_lag_step = 0.42_f64; // phase lag per echelon step down the V

            for k in 0..bird_count {
                let k_seed = flock_seed.wrapping_mul(79).wrapping_add(k as u32);

                // --- 1. AERODYNAMIC V-FORMATION (ECHELON DRAFTING GEOMETRY) ---
                // Leader is k=0 at apex.
                // k=1,3,5 are left echelon arm; k=2,4,6 are right echelon arm.
                let (echelon_step, arm_side) = if k == 0 {
                    (0_f32, 0.0_f32)
                } else if k % 2 == 1 {
                    (((k + 1) / 2) as f32, -1.0_f32) // Left arm
                } else {
                    ((k / 2) as f32, 1.0_f32) // Right arm
                };

                // Aerodynamic spacing: 25px back, 14px out per echelon step
                let dx_spacing = 25.0 * scale * depth_scale;
                let dy_spacing = 14.0 * scale * depth_scale;

                // Micro-turbulence drafting adjustments
                let turb_x = Self::hash_range(k_seed.wrapping_add(1), -2.0, 2.0) * scale * depth_scale;
                let turb_y = Self::hash_range(k_seed.wrapping_add(2), -1.5, 1.5) * scale * depth_scale;

                let bird_x = leader_x - echelon_step * dx_spacing + turb_x;
                let bird_y = leader_y + arm_side * echelon_step * dy_spacing + turb_y;
                let bird_center = Pos2::new(bird_x, bird_y);

                // Skip drawing if completely off-screen
                if bird_x < rect.min.x - 40.0 * scale || bird_x > rect.max.x + 40.0 * scale {
                    continue;
                }

                // --- 2. PHASE-LAG WINGBEAT WAVE SYNCHRONIZATION ---
                let bird_phase = (t * flap_freq - (echelon_step as f64) * phase_lag_step) as f32;
                let raw_flap = bird_phase.sin();
                let flap = raw_flap * glide_envelope * 0.72; // amplitude in radians

                let wing_span = 20.0 * scale * depth_scale;
                let body_len = 10.0 * scale * depth_scale;

                // --- 3. STREAMLINED AVIAN ANATOMY RENDERING ---

                // A. Aerodynamic Torso Fuselage
                let torso_pts = vec![
                    Pos2::new(bird_center.x - body_len * 0.55, bird_center.y),
                    Pos2::new(bird_center.x - body_len * 0.1, bird_center.y + 1.8 * scale * depth_scale),
                    Pos2::new(bird_center.x + body_len * 0.45, bird_center.y + 0.6 * scale * depth_scale),
                    Pos2::new(bird_center.x + body_len * 0.65, bird_center.y),
                    Pos2::new(bird_center.x + body_len * 0.45, bird_center.y - 0.6 * scale * depth_scale),
                    Pos2::new(bird_center.x - body_len * 0.1, bird_center.y - 1.8 * scale * depth_scale),
                ];
                painter.add(Shape::convex_polygon(torso_pts, bird_color, Stroke::NONE));

                // B. Forward Head & Aerodynamic Sharp Beak
                let head_pos = Pos2::new(bird_center.x + body_len * 0.65, bird_center.y);
                let beak_tip = Pos2::new(bird_center.x + body_len * 1.05, bird_center.y - 0.2 * scale * depth_scale);
                painter.circle_filled(head_pos, 2.2 * scale * depth_scale, bird_color);
                let beak_pts = vec![
                    Pos2::new(head_pos.x, head_pos.y - 1.2 * scale * depth_scale),
                    beak_tip,
                    Pos2::new(head_pos.x, head_pos.y + 1.2 * scale * depth_scale),
                ];
                painter.add(Shape::convex_polygon(beak_pts, bird_color, Stroke::NONE));

                // C. Feathered Tail Fan (Rudder)
                let tail_base = Pos2::new(bird_center.x - body_len * 0.55, bird_center.y);
                let tail_pts = vec![
                    tail_base,
                    Pos2::new(tail_base.x - body_len * 0.55, tail_base.y - 2.8 * scale * depth_scale),
                    Pos2::new(tail_base.x - body_len * 0.65, tail_base.y),
                    Pos2::new(tail_base.x - body_len * 0.55, tail_base.y + 2.8 * scale * depth_scale),
                ];
                painter.add(Shape::convex_polygon(tail_pts, bird_color, Stroke::NONE));

                // D. Cambered Multi-Segment Feathered Wings
                // Upstroke & Downstroke dynamic joint displacement
                let tip_y = -flap * wing_span * 0.62;
                let elbow_y = -flap * wing_span * 0.32 + (if flap > 0.0 { 1.5 } else { -0.5 }) * scale * depth_scale;
                let wrist_y = -flap * wing_span * 0.48;

                // Left Wing Cambered Polygon (Body -> Elbow -> Wrist -> Wingtip)
                let left_wing_pts = vec![
                    Pos2::new(bird_center.x, bird_center.y - 1.2 * scale * depth_scale),
                    Pos2::new(bird_center.x - wing_span * 0.28, bird_center.y - wing_span * 0.32 + elbow_y),
                    Pos2::new(bird_center.x - wing_span * 0.45, bird_center.y - wing_span * 0.68 + wrist_y),
                    Pos2::new(bird_center.x - wing_span * 0.55, bird_center.y - wing_span + tip_y),
                    Pos2::new(bird_center.x - wing_span * 0.35, bird_center.y - wing_span * 0.65 + wrist_y + 1.8 * scale * depth_scale),
                    Pos2::new(bird_center.x - wing_span * 0.15, bird_center.y - wing_span * 0.28 + elbow_y + 2.4 * scale * depth_scale),
                    Pos2::new(bird_center.x - body_len * 0.3, bird_center.y),
                ];
                painter.add(Shape::convex_polygon(
                    left_wing_pts,
                    bird_color,
                    Stroke::new(1.2 * scale * depth_scale, bird_color),
                ));

                // Right Wing Cambered Polygon (Body -> Elbow -> Wrist -> Wingtip)
                let right_wing_pts = vec![
                    Pos2::new(bird_center.x, bird_center.y + 1.2 * scale * depth_scale),
                    Pos2::new(bird_center.x - wing_span * 0.28, bird_center.y + wing_span * 0.32 - elbow_y),
                    Pos2::new(bird_center.x - wing_span * 0.45, bird_center.y + wing_span * 0.68 - wrist_y),
                    Pos2::new(bird_center.x - wing_span * 0.55, bird_center.y + wing_span - tip_y),
                    Pos2::new(bird_center.x - wing_span * 0.35, bird_center.y + wing_span * 0.65 - wrist_y - 1.8 * scale * depth_scale),
                    Pos2::new(bird_center.x - wing_span * 0.15, bird_center.y + wing_span * 0.28 - elbow_y - 2.4 * scale * depth_scale),
                    Pos2::new(bird_center.x - body_len * 0.3, bird_center.y),
                ];
                painter.add(Shape::convex_polygon(
                    right_wing_pts,
                    bird_color,
                    Stroke::new(1.2 * scale * depth_scale, bird_color),
                ));
            }
        }
    }

    // =========================================================================
    // 5. CLAPPING APPLAUSE — Clean Minimalist Silhouette Crowd with Articulated Arms
    // =========================================================================
    fn draw_clapping(painter: &egui::Painter, rect: Rect, t: f64, intensity: f32) {
        let scale = Self::scale(rect);
        let bottom_y = rect.max.y;
        let width = rect.width();

        // ---------------------------------------------------------------------
        // LAYER 1: BACK-ROW CHEERING SILHOUETTES
        // ---------------------------------------------------------------------
        let back_count = (10.0 * intensity).round() as usize;
        let back_color = Color32::from_rgba_unmultiplied(20, 26, 38, 215);

        for b in 0..back_count {
            let b_seed = (b as u32).wrapping_mul(173).wrapping_add(19);
            let b_x_norm = 0.04 + (b as f32 + Self::hash_range(b_seed, -0.2, 0.2)) / (back_count as f32) * 0.92;
            let b_x = rect.min.x + b_x_norm * width;

            // Enthusiastic bobbing up and down
            let bob = (t * 5.4 + b as f64 * 1.3).sin() as f32 * 4.0 * scale;
            let head_y = bottom_y - (38.0 * scale) + bob;
            let head_r = 8.5 * scale;

            // Head circle
            painter.circle_filled(Pos2::new(b_x, head_y), head_r, back_color);

            // Shoulders / upper torso
            let shoulder_w = 14.0 * scale;
            let torso_pts = vec![
                Pos2::new(b_x - shoulder_w, bottom_y),
                Pos2::new(b_x - shoulder_w * 0.8, head_y + head_r * 0.8),
                Pos2::new(b_x + shoulder_w * 0.8, head_y + head_r * 0.8),
                Pos2::new(b_x + shoulder_w, bottom_y),
            ];
            painter.add(Shape::convex_polygon(torso_pts, back_color, Stroke::NONE));

            // Cheering raised arms waving in the air
            let wave_left = (t * 6.0 + b as f64 * 1.5).sin() as f32 * 5.0 * scale;
            let wave_right = (t * 6.0 + b as f64 * 1.5 + 1.0).sin() as f32 * 5.0 * scale;

            let arm_stroke = Stroke::new(3.0 * scale, back_color);
            painter.line_segment(
                [
                    Pos2::new(b_x - shoulder_w * 0.7, head_y + head_r),
                    Pos2::new(b_x - shoulder_w * 1.1 + wave_left, head_y - 12.0 * scale),
                ],
                arm_stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(b_x + shoulder_w * 0.7, head_y + head_r),
                    Pos2::new(b_x + shoulder_w * 1.1 + wave_right, head_y - 12.0 * scale),
                ],
                arm_stroke,
            );
        }

        // ---------------------------------------------------------------------
        // LAYER 2: FRONT-ROW CLEAN ARTICULATED CLAPPING SILHOUETTES
        // ---------------------------------------------------------------------
        let front_count = (6.0 * intensity).round().max(4.0) as usize;

        // Character Archetypes: (Shirt Color, Collar Color, Skin Tone, Hair Color, Hair Style)
        let character_archetypes = [
            // 0: Crimson Shirt, White Collar, Tan Skin, Brunette Hair
            (
                Color32::from_rgb(190, 40, 55),
                Color32::from_rgb(245, 245, 250),
                Color32::from_rgb(240, 195, 155),
                Color32::from_rgb(45, 30, 22),
                0,
            ),
            // 1: Emerald Green, Deep Green Collar, Fair Skin, Dark Curly Hair
            (
                Color32::from_rgb(28, 148, 90),
                Color32::from_rgb(18, 95, 58),
                Color32::from_rgb(255, 220, 185),
                Color32::from_rgb(24, 20, 18),
                1,
            ),
            // 2: Royal Blue, Gold Rim, Warm Tan Skin, Golden Blonde Hair
            (
                Color32::from_rgb(32, 90, 190),
                Color32::from_rgb(255, 210, 50),
                Color32::from_rgb(245, 200, 160),
                Color32::from_rgb(215, 165, 70),
                2,
            ),
            // 3: Warm Amber, Dark Trim, Deep Brown Skin, Dark Crop
            (
                Color32::from_rgb(220, 145, 28),
                Color32::from_rgb(140, 85, 12),
                Color32::from_rgb(155, 100, 70),
                Color32::from_rgb(28, 18, 14),
                3,
            ),
            // 4: Plum Purple, Soft Lavender, Olive Skin, Dark Hair
            (
                Color32::from_rgb(155, 48, 170),
                Color32::from_rgb(220, 180, 230),
                Color32::from_rgb(210, 155, 115),
                Color32::from_rgb(35, 24, 20),
                0,
            ),
            // 5: Ocean Teal, Dark Teal, Fair Skin, Chestnut Hair
            (
                Color32::from_rgb(42, 160, 190),
                Color32::from_rgb(25, 105, 125),
                Color32::from_rgb(255, 218, 185),
                Color32::from_rgb(95, 45, 25),
                2,
            ),
        ];

        for k in 0..front_count {
            let (shirt_col, collar_col, skin_col, hair_col, hair_style) =
                character_archetypes[k % character_archetypes.len()];

            let k_x_norm = 0.08 + (k as f32 / (front_count - 1) as f32) * 0.84;
            let k_x = rect.min.x + k_x_norm * width;

            // Rhythmic body bounce
            let bounce = (t * 6.2 + k as f64 * 1.1).sin() as f32 * 5.0 * scale;
            let head_center = Pos2::new(k_x, bottom_y - 48.0 * scale + bounce);
            let head_r = 9.5 * scale;
            let shoulder_y = head_center.y + head_r * 0.95;
            let shoulder_w = 19.0 * scale;

            // 1. Torso & Shirt with Clean Shoulder Contours
            let torso_pts = vec![
                Pos2::new(k_x - shoulder_w * 1.18, bottom_y),
                Pos2::new(k_x - shoulder_w, shoulder_y + 2.0 * scale),
                Pos2::new(k_x - shoulder_w * 0.6, shoulder_y - 1.0 * scale),
                Pos2::new(k_x + shoulder_w * 0.6, shoulder_y - 1.0 * scale),
                Pos2::new(k_x + shoulder_w, shoulder_y + 2.0 * scale),
                Pos2::new(k_x + shoulder_w * 1.18, bottom_y),
            ];
            painter.add(Shape::convex_polygon(torso_pts, shirt_col, Stroke::NONE));

            // Clean Minimalist Shirt Collar Notch
            let collar_pts = vec![
                Pos2::new(k_x - 5.0 * scale, shoulder_y - 1.0 * scale),
                Pos2::new(k_x, shoulder_y + 5.5 * scale),
                Pos2::new(k_x + 5.0 * scale, shoulder_y - 1.0 * scale),
            ];
            painter.add(Shape::convex_polygon(
                collar_pts,
                collar_col,
                Stroke::NONE,
            ));

            // 2. Clean Minimalist Head Silhouette (No Facial Clutter)
            painter.circle_filled(head_center, head_r, skin_col);

            // 3. Sleek Hair Silhouette Caps
            match hair_style {
                1 => {
                    // Textured curly cap
                    for c in 0..5 {
                        let c_ang = (c as f32) * std::f32::consts::PI / 4.0 + std::f32::consts::PI;
                        let cx = head_center.x + c_ang.cos() * head_r * 0.92;
                        let cy = head_center.y + c_ang.sin() * head_r * 0.92;
                        painter.circle_filled(Pos2::new(cx, cy), 4.2 * scale, hair_col);
                    }
                    painter.circle_filled(Pos2::new(head_center.x, head_center.y - head_r * 0.65), 5.0 * scale, hair_col);
                }
                2 => {
                    // Wavy hair silhouette
                    let hair_pts = vec![
                        Pos2::new(head_center.x - head_r * 1.08, head_center.y + head_r * 0.3),
                        Pos2::new(head_center.x - head_r * 1.05, head_center.y - head_r * 0.6),
                        Pos2::new(head_center.x, head_center.y - head_r * 1.25),
                        Pos2::new(head_center.x + head_r * 1.05, head_center.y - head_r * 0.6),
                        Pos2::new(head_center.x + head_r * 1.08, head_center.y + head_r * 0.3),
                        Pos2::new(head_center.x + head_r * 0.75, head_center.y - head_r * 0.35),
                        Pos2::new(head_center.x - head_r * 0.75, head_center.y - head_r * 0.35),
                    ];
                    painter.add(Shape::convex_polygon(hair_pts, hair_col, Stroke::NONE));
                }
                3 => {
                    // Spiky hair cap
                    let hair_pts = vec![
                        Pos2::new(head_center.x - head_r, head_center.y),
                        Pos2::new(head_center.x - head_r * 0.75, head_center.y - head_r * 0.8),
                        Pos2::new(head_center.x - head_r * 0.35, head_center.y - head_r * 1.35),
                        Pos2::new(head_center.x, head_center.y - head_r * 1.05),
                        Pos2::new(head_center.x + head_r * 0.35, head_center.y - head_r * 1.35),
                        Pos2::new(head_center.x + head_r * 0.75, head_center.y - head_r * 0.8),
                        Pos2::new(head_center.x + head_r, head_center.y),
                        Pos2::new(head_center.x, head_center.y - head_r * 0.4),
                    ];
                    painter.add(Shape::convex_polygon(hair_pts, hair_col, Stroke::NONE));
                }
                _ => {
                    // Classic side-part sleek cap
                    let hair_pts = vec![
                        Pos2::new(head_center.x - head_r * 1.02, head_center.y + head_r * 0.1),
                        Pos2::new(head_center.x - head_r * 0.85, head_center.y - head_r * 0.75),
                        Pos2::new(head_center.x - head_r * 0.2, head_center.y - head_r * 1.2),
                        Pos2::new(head_center.x + head_r * 0.85, head_center.y - head_r * 0.8),
                        Pos2::new(head_center.x + head_r * 1.02, head_center.y + head_r * 0.1),
                        Pos2::new(head_center.x + head_r * 0.45, head_center.y - head_r * 0.4),
                        Pos2::new(head_center.x - head_r * 0.45, head_center.y - head_r * 0.4),
                    ];
                    painter.add(Shape::convex_polygon(hair_pts, hair_col, Stroke::NONE));
                }
            }

            // 4. Clapping Cycle & 2-Segment Articulated Arms (Bending Elbows)
            let clap_period = 0.52;
            let offset = k as f64 * 0.12;
            let local_t = (t + offset) % clap_period;
            let clap_p = (local_t / clap_period) as f32;

            // Arm movement & hand separation
            let clap_sin = (clap_p * std::f32::consts::TAU).sin().abs();
            let hand_sep = clap_sin * 15.0 * scale;
            let overhead_y = head_center.y - 25.0 * scale;

            let left_shoulder = Pos2::new(k_x - shoulder_w * 0.85, shoulder_y);
            let right_shoulder = Pos2::new(k_x + shoulder_w * 0.85, shoulder_y);

            let left_hand = Pos2::new(k_x - hand_sep, overhead_y);
            let right_hand = Pos2::new(k_x + hand_sep, overhead_y);

            // Inverse Kinematics: Elbows flare outward naturally when hands meet
            let elbow_flare = (1.0 - clap_sin) * 8.0 * scale + 5.0 * scale;
            let left_elbow = Pos2::new(
                (left_shoulder.x + left_hand.x) * 0.5 - elbow_flare,
                (left_shoulder.y + left_hand.y) * 0.5 + 2.0 * scale,
            );
            let right_elbow = Pos2::new(
                (right_shoulder.x + right_hand.x) * 0.5 + elbow_flare,
                (right_shoulder.y + right_hand.y) * 0.5 + 2.0 * scale,
            );

            // Upper arms (Shoulder -> Elbow in shirt sleeve color)
            let upper_arm_stroke = Stroke::new(4.5 * scale, shirt_col);
            painter.line_segment([left_shoulder, left_elbow], upper_arm_stroke);
            painter.line_segment([right_shoulder, right_elbow], upper_arm_stroke);

            // Forearms (Elbow -> Hand in skin tone)
            let forearm_stroke = Stroke::new(3.6 * scale, skin_col);
            painter.line_segment([left_elbow, left_hand], forearm_stroke);
            painter.line_segment([right_elbow, right_hand], forearm_stroke);

            // Detailed Hands: Palm + Thumb pads
            let hand_r = 3.6 * scale;
            painter.circle_filled(left_hand, hand_r, skin_col);
            painter.circle_filled(right_hand, hand_r, skin_col);
            painter.circle_filled(Pos2::new(left_hand.x + 1.8 * scale, left_hand.y + 1.2 * scale), hand_r * 0.55, skin_col);
            painter.circle_filled(Pos2::new(right_hand.x - 1.8 * scale, right_hand.y + 1.2 * scale), hand_r * 0.55, skin_col);

            // 5. Celebration Impact FX Overhead
            let is_impact = hand_sep < 2.5 * scale;
            if is_impact {
                let impact_center = Pos2::new(k_x, overhead_y);

                // Golden shockwave ring
                painter.circle_stroke(
                    impact_center,
                    9.0 * scale,
                    Stroke::new(1.8 * scale, Color32::from_rgba_unmultiplied(255, 225, 80, 230)),
                );

                // 4-point celebration sparkle cross
                Self::draw_sparkle(
                    painter,
                    impact_center,
                    7.0 * scale,
                    Color32::from_rgba_unmultiplied(255, 245, 170, 250),
                    true,
                );

                // Micro celebration sparks
                for sp in 0..4 {
                    let sp_ang = (sp as f32) * (std::f32::consts::TAU / 4.0) + (t * 2.0) as f32;
                    let sp_pos = Pos2::new(
                        impact_center.x + sp_ang.cos() * 10.0 * scale,
                        impact_center.y + sp_ang.sin() * 10.0 * scale,
                    );
                    painter.circle_filled(
                        sp_pos,
                        1.4 * scale,
                        Color32::from_rgba_unmultiplied(255, 235, 95, 230),
                    );
                }
            }

            // Floating celebration glyphs rising from crowd
            if k % 2 == 0 {
                let note_progress = (clap_p + 0.3) % 1.0;
                let note_alpha = ((1.0 - note_progress) * 220.0) as u8;
                let note_y = overhead_y - note_progress * 38.0 * scale;
                let note_sway = (t * 3.5 + k as f64 * 1.5).sin() as f32 * 6.0 * scale;
                let note_pos = Pos2::new(k_x + note_sway, note_y);

                let symbols = ["♪", "♫", "✨", "⭐", "👏"];
                let symbol = symbols[(k + (t * 2.0) as usize) % symbols.len()];

                painter.text(
                    note_pos,
                    egui::Align2::CENTER_CENTER,
                    symbol,
                    egui::FontId::proportional(12.0 * scale),
                    Color32::from_rgba_unmultiplied(255, 230, 90, note_alpha),
                );
            }
        }
    }

    // =========================================================================
    // 6. SHOOTING STARS — Cinematic Glowing Plasma Bolides with Volumetric Bloom
    // =========================================================================
    fn draw_shooting_stars(painter: &egui::Painter, rect: Rect, t: f64, intensity: f32) {
        let star_count = (3.0 * intensity).round().max(2.0) as usize;
        let star_period = 3.2; // time between periodic streaks
        let scale = Self::scale(rect);

        for i in 0..star_count {
            let seed = Self::hash((i as u32).wrapping_mul(83).wrapping_add(17));
            let offset = Self::hash_f(seed) as f64 * star_period;
            let local_t = (t + offset) % star_period;

            // Swift celestial streak window: active for 0.46s
            let streak_duration = 0.46_f64;
            if local_t > streak_duration + 0.85 {
                // Completely inactive
                continue;
            }

            let is_streaking = local_t <= streak_duration;
            let streak_p = if is_streaking {
                (local_t / streak_duration) as f32
            } else {
                1.0_f32
            };

            // Trajectory parameters (diagonal entry across sky)
            let start_x = Self::hash_range(seed.wrapping_add(1), 0.05, 0.55);
            let start_y = Self::hash_range(seed.wrapping_add(2), 0.02, 0.30);
            let streak_len_x = Self::hash_range(seed.wrapping_add(3), 0.40, 0.55);
            let streak_len_y = streak_len_x * 0.48;

            // Hypervelocity entry acceleration
            let accel_p = streak_p * (1.0 + streak_p * 0.4);

            let head_x = start_x + accel_p * streak_len_x;
            let head_y = start_y + accel_p * streak_len_y;
            let head = Pos2::new(
                rect.min.x + head_x * rect.width(),
                rect.min.y + head_y * rect.height(),
            );

            // Fade intensity based on flight lifecycle
            let base_alpha = if is_streaking {
                // Fade in rapidly, peak at 60%, fade out towards end
                if streak_p < 0.15 {
                    streak_p / 0.15
                } else {
                    (1.0 - streak_p).powf(0.5)
                }
            } else {
                // Lingering phosphorescent wake fading out
                let fade_p = ((local_t - streak_duration) / 0.85) as f32;
                (1.0 - fade_p).powi(2) * 0.35
            };

            if base_alpha <= 0.01 {
                continue;
            }

            // -----------------------------------------------------------------
            // 1. SEAMLESS SUB-PIXEL CONTINUOUS PLASMA RIBBON (65 Steps, No Dashes)
            // -----------------------------------------------------------------
            let trail_steps = 65;
            let trail_span = 0.32_f32; // span behind the head

            for seg in 0..trail_steps {
                let s0 = seg as f32 / trail_steps as f32;
                let s1 = (seg + 1) as f32 / trail_steps as f32;

                let back0 = (accel_p - s0 * (accel_p * trail_span)).max(0.0);
                let back1 = (accel_p - s1 * (accel_p * trail_span)).max(0.0);

                let p0 = Pos2::new(
                    rect.min.x + (start_x + back0 * streak_len_x) * rect.width(),
                    rect.min.y + (start_y + back0 * streak_len_y) * rect.height(),
                );
                let p1 = Pos2::new(
                    rect.min.x + (start_x + back1 * streak_len_x) * rect.width(),
                    rect.min.y + (start_y + back1 * streak_len_y) * rect.height(),
                );

                let seg_decay = (1.0 - s1).powf(1.6);
                let seg_alpha = (base_alpha * seg_decay * 255.0).clamp(0.0, 255.0) as u8;

                // LAYER A: Wide Outer Translucent Glow Aura
                let aura_width = (8.5 * (1.0 - s1).powf(1.2) + 1.2) * scale;
                let aura_alpha = (base_alpha * seg_decay * 65.0).clamp(0.0, 255.0) as u8;
                let aura_color = Color32::from_rgba_unmultiplied(45, 175, 255, aura_alpha);
                painter.line_segment([p0, p1], Stroke::new(aura_width, aura_color));

                // LAYER B: Middle Electric Cyan Ionization Sheath
                let sheath_width = (4.0 * (1.0 - s1).powf(1.4) + 0.6) * scale;
                let sheath_alpha = (base_alpha * seg_decay * 165.0).clamp(0.0, 255.0) as u8;
                let sheath_color = Color32::from_rgba_unmultiplied(160, 245, 255, sheath_alpha);
                painter.line_segment([p0, p1], Stroke::new(sheath_width, sheath_color));

                // LAYER C: Center Razor-Sharp White-Hot Core Filament
                let core_width = (1.5 * (1.0 - s1 * 0.7)).max(0.6) * scale;
                let core_color = Color32::from_rgba_unmultiplied(255, 255, 255, seg_alpha);
                painter.line_segment([p0, p1], Stroke::new(core_width, core_color));
            }

            // -----------------------------------------------------------------
            // 2. 8-LAYER VOLUMETRIC GAUSSIAN BLOOM AROUND METEOR HEAD
            // -----------------------------------------------------------------
            if is_streaking {
                let bloom_radii = [
                    24.0 * scale,
                    17.0 * scale,
                    12.0 * scale,
                    8.5 * scale,
                    6.0 * scale,
                    4.2 * scale,
                    2.8 * scale,
                    1.6 * scale,
                ];
                let bloom_alphas = [
                    (base_alpha * 22.0) as u8,
                    (base_alpha * 38.0) as u8,
                    (base_alpha * 60.0) as u8,
                    (base_alpha * 95.0) as u8,
                    (base_alpha * 140.0) as u8,
                    (base_alpha * 190.0) as u8,
                    (base_alpha * 235.0) as u8,
                    (base_alpha * 255.0) as u8,
                ];
                let bloom_colors = [
                    Color32::from_rgba_unmultiplied(35, 120, 255, bloom_alphas[0]),  // Outer Celestial Blue
                    Color32::from_rgba_unmultiplied(50, 160, 255, bloom_alphas[1]),
                    Color32::from_rgba_unmultiplied(75, 195, 255, bloom_alphas[2]),  // Electric Cyan
                    Color32::from_rgba_unmultiplied(120, 225, 255, bloom_alphas[3]),
                    Color32::from_rgba_unmultiplied(180, 240, 255, bloom_alphas[4]), // Ice Blue
                    Color32::from_rgba_unmultiplied(225, 250, 255, bloom_alphas[5]),
                    Color32::from_rgba_unmultiplied(255, 255, 255, bloom_alphas[6]), // Pure White Core
                    Color32::from_rgba_unmultiplied(255, 255, 255, bloom_alphas[7]),
                ];

                for b in 0..8 {
                    painter.circle_filled(head, bloom_radii[b], bloom_colors[b]);
                }

                // -----------------------------------------------------------------
                // 3. ANAMORPHIC AXIAL LENS FLARE (Sharp Needle of Light Along Path)
                // -----------------------------------------------------------------
                let dir_x = streak_len_x * rect.width();
                let dir_y = streak_len_y * rect.height();
                let dir_len = (dir_x * dir_x + dir_y * dir_y).sqrt().max(1.0);
                let norm_dx = dir_x / dir_len;
                let norm_dy = dir_y / dir_len;

                let flare_len = 22.0 * scale * base_alpha;
                let flare_p0 = Pos2::new(head.x - norm_dx * flare_len * 0.6, head.y - norm_dy * flare_len * 0.6);
                let flare_p1 = Pos2::new(head.x + norm_dx * flare_len * 1.0, head.y + norm_dy * flare_len * 1.0);

                painter.line_segment(
                    [flare_p0, flare_p1],
                    Stroke::new(
                        1.6 * scale,
                        Color32::from_rgba_unmultiplied(255, 255, 255, (base_alpha * 240.0) as u8),
                    ),
                );

                // Perpendicular cross glint
                let perp_dx = -norm_dy;
                let perp_dy = norm_dx;
                let cross_len = 8.0 * scale * base_alpha;
                let cross_p0 = Pos2::new(head.x - perp_dx * cross_len, head.y - perp_dy * cross_len);
                let cross_p1 = Pos2::new(head.x + perp_dx * cross_len, head.y + perp_dy * cross_len);

                painter.line_segment(
                    [cross_p0, cross_p1],
                    Stroke::new(
                        1.2 * scale,
                        Color32::from_rgba_unmultiplied(210, 245, 255, (base_alpha * 210.0) as u8),
                    ),
                );
            }
        }
    }
}


