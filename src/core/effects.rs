use egui::{Color32, Pos2, Rect, Stroke, Vec2};
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

/// High-performance deterministic particle and animation simulation engine for live egui preview.
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

    // -------------------------------------------------------------
    // 1. FIREWORKS SIMULATION
    // -------------------------------------------------------------
    fn draw_fireworks(painter: &egui::Painter, rect: Rect, t: f64, intensity: f32) {
        let burst_period = 1.6;
        let burst_count = (4.0 * intensity).round() as usize;

        let colors = [
            Color32::from_rgb(255, 60, 60),   // Crimson Red
            Color32::from_rgb(255, 210, 40),  // Gold Yellow
            Color32::from_rgb(50, 220, 255),  // Electric Cyan
            Color32::from_rgb(180, 70, 255),  // Purple
            Color32::from_rgb(70, 255, 120),  // Neon Green
            Color32::from_rgb(255, 130, 220), // Bright Pink
        ];

        for i in 0..burst_count {
            let offset = (i as f64) * 0.42;
            let local_t = (t + offset) % burst_period;
            let progress = (local_t / burst_period) as f32; // 0.0 to 1.0

            let cx_norm = 0.20 + (((i * 73 + 17) % 65) as f32) / 100.0;
            let cy_norm = 0.25 + (((i * 41 + 13) % 45) as f32) / 100.0;
            let center = Pos2::new(
                rect.min.x + cx_norm * rect.width(),
                rect.min.y + cy_norm * rect.height(),
            );

            let color_base = colors[i % colors.len()];
            let num_sparks = 18;

            if progress < 0.25 {
                // Rocket rising phase
                let rocket_p = progress / 0.25;
                let rocket_y = rect.max.y - (rect.max.y - center.y) * rocket_p;
                let r_pos = Pos2::new(center.x, rocket_y);
                painter.circle_filled(r_pos, 2.5, Color32::from_rgb(255, 240, 180));
                painter.line_segment(
                    [r_pos, Pos2::new(r_pos.x, r_pos.y + 12.0)],
                    Stroke::new(1.5, Color32::from_rgba_unmultiplied(255, 180, 50, 120)),
                );
            } else {
                // Explosion phase
                let burst_p = (progress - 0.25) / 0.75;
                let alpha = ((1.0 - burst_p) * 255.0).clamp(0.0, 255.0) as u8;
                let spark_color = Color32::from_rgba_unmultiplied(
                    color_base.r(),
                    color_base.g(),
                    color_base.b(),
                    alpha,
                );

                let max_radius = 55.0 * rect.height() / 400.0;
                let cur_radius = max_radius * burst_p.sqrt();

                for s in 0..num_sparks {
                    let angle = (s as f32) * (std::f32::consts::TAU / num_sparks as f32);
                    let gravity_drop = burst_p.powi(2) * 14.0;
                    let spark_pos = Pos2::new(
                        center.x + angle.cos() * cur_radius,
                        center.y + angle.sin() * cur_radius + gravity_drop,
                    );
                    painter.circle_filled(spark_pos, (3.0 * (1.0 - burst_p)).max(1.0), spark_color);

                    // Tiny sparkle trail
                    let trail_pos = Pos2::new(
                        center.x + angle.cos() * (cur_radius * 0.85),
                        center.y + angle.sin() * (cur_radius * 0.85) + gravity_drop * 0.8,
                    );
                    painter.circle_filled(
                        trail_pos,
                        1.2,
                        Color32::from_rgba_unmultiplied(255, 255, 200, (alpha as f32 * 0.6) as u8),
                    );
                }
            }
        }
    }

    // -------------------------------------------------------------
    // 2. CONFETTI SIMULATION
    // -------------------------------------------------------------
    fn draw_confetti(painter: &egui::Painter, rect: Rect, t: f64, intensity: f32) {
        let piece_count = (32.0 * intensity).round() as usize;
        let colors = [
            Color32::from_rgb(255, 60, 90),
            Color32::from_rgb(255, 190, 30),
            Color32::from_rgb(40, 210, 100),
            Color32::from_rgb(30, 170, 255),
            Color32::from_rgb(200, 80, 255),
            Color32::from_rgb(255, 255, 255),
        ];

        for i in 0..piece_count {
            let seed = (i * 97 + 23) as f64;
            let speed_mult = 0.7 + ((seed % 100.0) / 100.0) * 0.6;
            let fall_period = 3.2 / speed_mult;
            let phase = (seed * 0.13) % fall_period;
            let local_t = (t + phase) % fall_period;
            let progress = (local_t / fall_period) as f32;

            let x_base = (((i * 47 + 13) % 100) as f32) / 100.0;
            let sway = ((local_t * 3.5 + seed).sin() as f32) * 0.04;
            let px = rect.min.x + (x_base + sway).clamp(0.02, 0.98) * rect.width();
            let py = rect.min.y + progress * (rect.height() + 30.0) - 15.0;

            let flutter = (local_t * 6.0 + seed).cos() as f32;
            let w = (8.0 + (i % 4) as f32 * 2.0) * rect.height() / 450.0;
            let h = (w * 0.5 * flutter.abs()).max(2.0);

            let color = colors[i % colors.len()];
            let c_rect = Rect::from_center_size(Pos2::new(px, py), Vec2::new(w, h));
            painter.rect_filled(c_rect, 1.5, color);

            // Occasional round confetti
            if i % 3 == 0 {
                painter.circle_filled(
                    Pos2::new(px + 4.0, py + 4.0),
                    w * 0.35,
                    colors[(i + 2) % colors.len()],
                );
            }
        }
    }

    // -------------------------------------------------------------
    // 3. FLOATING BALLOONS SIMULATION
    // -------------------------------------------------------------
    fn draw_balloons(painter: &egui::Painter, rect: Rect, t: f64, intensity: f32) {
        let balloon_count = (9.0 * intensity).round() as usize;
        let colors = [
            Color32::from_rgb(255, 50, 70),   // Cherry Red
            Color32::from_rgb(40, 160, 255),  // Sky Blue
            Color32::from_rgb(255, 195, 20),  // Sunny Yellow
            Color32::from_rgb(50, 210, 110),  // Mint Green
            Color32::from_rgb(190, 60, 240),  // Violet
            Color32::from_rgb(255, 120, 40),  // Orange
            Color32::from_rgb(255, 105, 180), // Hot Pink
        ];

        for i in 0..balloon_count {
            let seed = (i * 137 + 43) as f64;
            let rise_period = 4.5 + ((seed % 100.0) / 100.0) * 2.0;
            let phase = (seed * 0.17) % rise_period;
            let local_t = (t + phase) % rise_period;
            let progress = (local_t / rise_period) as f32; // 0.0 to 1.0 (bottom to top)

            let x_base = 0.08 + (((i * 83 + 29) % 84) as f32) / 100.0;
            let sway = ((local_t * 1.8 + seed).sin() as f32) * 0.035;
            let px = rect.min.x + (x_base + sway).clamp(0.04, 0.96) * rect.width();
            let py = rect.max.y - progress * (rect.height() + 70.0) + 35.0;

            let radius_x = (14.0 + (i % 3) as f32 * 3.0) * rect.height() / 450.0;
            let radius_y = radius_x * 1.25;

            let col = colors[i % colors.len()];

            // Balloon oval
            let balloon_center = Pos2::new(px, py);
            painter.circle_filled(balloon_center, radius_x, col);

            // Shading highlight (sun glare top-left)
            let highlight_pos = Pos2::new(px - radius_x * 0.35, py - radius_y * 0.35);
            painter.circle_filled(
                highlight_pos,
                radius_x * 0.3,
                Color32::from_rgba_unmultiplied(255, 255, 255, 140),
            );

            // Balloon knot & string
            let knot_pos = Pos2::new(px, py + radius_y * 0.85);
            painter.circle_filled(knot_pos, 2.5, col);
            let string_end = Pos2::new(px + ((local_t * 3.0).sin() as f32) * 4.0, knot_pos.y + 24.0);
            painter.line_segment(
                [knot_pos, string_end],
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(200, 200, 220, 160)),
            );
        }
    }

    // -------------------------------------------------------------
    // 4. FLYING BIRDS SIMULATION
    // -------------------------------------------------------------
    fn draw_birds(painter: &egui::Painter, rect: Rect, t: f64, intensity: f32) {
        let flock_size = (6.0 * intensity).round() as usize;
        let loop_period = 6.0;

        for i in 0..flock_size {
            let seed = (i * 71 + 19) as f64;
            let phase = (seed * 0.23) % loop_period;
            let local_t = (t + phase) % loop_period;
            let progress = (local_t / loop_period) as f32; // 0.0 to 1.0 (left to right)

            let y_base = 0.12 + (((i * 59 + 7) % 55) as f32) / 100.0;
            let bob = ((local_t * 2.8 + seed).sin() as f32) * 0.02;

            let px = rect.min.x - 40.0 + progress * (rect.width() + 80.0);
            let py = rect.min.y + (y_base + bob).clamp(0.05, 0.90) * rect.height();

            // Wing flap angle
            let flap = ((local_t * 7.5 + seed).sin() as f32) * 0.7; // -0.7 to 0.7
            let wing_span = (14.0 + (i % 3) as f32 * 3.0) * rect.height() / 450.0;

            let center = Pos2::new(px, py);
            let left_tip = Pos2::new(center.x - wing_span, center.y - flap * wing_span * 0.6);
            let right_tip = Pos2::new(center.x + wing_span, center.y - flap * wing_span * 0.6);
            let beak = Pos2::new(center.x + 3.0, center.y + 1.0);

            let bird_color = Color32::from_rgba_unmultiplied(240, 245, 255, 220);
            let wing_color = Color32::from_rgba_unmultiplied(220, 230, 255, 240);

            painter.line_segment([left_tip, center], Stroke::new(2.2, wing_color));
            painter.line_segment([center, right_tip], Stroke::new(2.2, wing_color));
            painter.circle_filled(beak, 1.8, bird_color);
        }
    }

    // -------------------------------------------------------------
    // 5. CLAPPING APPLAUSE SIMULATION
    // -------------------------------------------------------------
    fn draw_clapping(painter: &egui::Painter, rect: Rect, t: f64, intensity: f32) {
        let clap_pairs = (4.0 * intensity).round() as usize;
        let clap_period = 0.8;

        for i in 0..clap_pairs {
            let offset = (i as f64) * 0.21;
            let local_t = (t + offset) % clap_period;
            let clap_p = (local_t / clap_period) as f32; // 0.0 to 1.0

            let cx_norm = 0.20 + (((i * 89 + 31) % 65) as f32) / 100.0;
            let cy_norm = 0.35 + (((i * 53 + 23) % 45) as f32) / 100.0;
            let center = Pos2::new(
                rect.min.x + cx_norm * rect.width(),
                rect.min.y + cy_norm * rect.height(),
            );

            let impact = (1.0 - (clap_p * 2.5).min(1.0)).powi(2);
            let size = (22.0 + impact * 8.0) * rect.height() / 450.0;

            let hand_sep = (1.0 - impact) * 12.0;
            let left_hand = Pos2::new(center.x - hand_sep, center.y);
            let right_hand = Pos2::new(center.x + hand_sep, center.y);

            let palm_col = Color32::from_rgb(255, 218, 170);
            painter.circle_filled(left_hand, size * 0.45, palm_col);
            painter.circle_filled(right_hand, size * 0.45, palm_col);

            if impact > 0.1 {
                let spark_col = Color32::from_rgba_unmultiplied(255, 230, 80, (impact * 255.0) as u8);
                let num_rays = 6;
                let ray_len = (size * 0.7) + (1.0 - impact) * 16.0;

                for r in 0..num_rays {
                    let ang = (r as f32) * (std::f32::consts::TAU / num_rays as f32) + (t * 2.0) as f32;
                    let p1 = Pos2::new(center.x + ang.cos() * (size * 0.5), center.y + ang.sin() * (size * 0.5));
                    let p2 = Pos2::new(center.x + ang.cos() * ray_len, center.y + ang.sin() * ray_len);
                    painter.line_segment([p1, p2], Stroke::new(1.8 * impact, spark_col));
                }
            }
        }
    }

    // -------------------------------------------------------------
    // 6. SHOOTING STARS SIMULATION
    // -------------------------------------------------------------
    fn draw_shooting_stars(painter: &egui::Painter, rect: Rect, t: f64, intensity: f32) {
        let star_count = (3.0 * intensity).round() as usize;
        let star_period = 2.4;

        for i in 0..star_count {
            let offset = (i as f64) * 0.85;
            let local_t = (t + offset) % star_period;
            let progress = (local_t / star_period) as f32;

            if progress > 0.65 {
                continue;
            }

            let streak_p = progress / 0.65;
            let start_x = 0.10 + (((i * 73 + 11) % 60) as f32) / 100.0;
            let start_y = 0.05 + (((i * 47 + 19) % 35) as f32) / 100.0;

            let streak_len_norm = 0.35;
            let px_norm = start_x + streak_p * streak_len_norm;
            let py_norm = start_y + streak_p * (streak_len_norm * 0.65);

            let head = Pos2::new(
                rect.min.x + px_norm * rect.width(),
                rect.min.y + py_norm * rect.height(),
            );

            let tail_len = 65.0 * rect.height() / 450.0;
            let tail = Pos2::new(head.x - tail_len * 0.85, head.y - tail_len * 0.55);

            let alpha = ((1.0 - streak_p) * 255.0).clamp(0.0, 255.0) as u8;
            let star_color = Color32::from_rgba_unmultiplied(255, 255, 240, alpha);
            let glow_color = Color32::from_rgba_unmultiplied(120, 220, 255, (alpha as f32 * 0.6) as u8);

            painter.line_segment([tail, head], Stroke::new(2.5, glow_color));
            painter.line_segment([tail, head], Stroke::new(1.0, star_color));

            painter.circle_filled(head, 3.5, star_color);
            painter.circle_filled(head, 7.0, Color32::from_rgba_unmultiplied(255, 255, 255, (alpha as f32 * 0.35) as u8));
        }
    }
}
