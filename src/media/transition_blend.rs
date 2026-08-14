use crate::core::transition::TransitionKind;
use egui::{Color32, ColorImage};

/// Blend two video frames using the specified visual transition style and normalized progress (0.0..=1.0).
pub fn blend_transition(
    frame_a: &ColorImage,
    frame_b: &ColorImage,
    kind: TransitionKind,
    progress: f32,
) -> ColorImage {
    let t = progress.clamp(0.0, 1.0);
    if t <= 0.001 {
        return frame_a.clone();
    }
    if t >= 0.999 {
        return frame_b.clone();
    }

    let w = frame_b.size[0].min(frame_a.size[0]);
    let h = frame_b.size[1].min(frame_a.size[1]);
    if w == 0 || h == 0 {
        return frame_b.clone();
    }

    let mut out_pixels = Vec::with_capacity(w * h);

    let get_pixel_a = |x: usize, y: usize| -> Color32 {
        let idx = y * frame_a.size[0] + x;
        if idx < frame_a.pixels.len() {
            frame_a.pixels[idx]
        } else {
            Color32::BLACK
        }
    };

    let get_pixel_b = |x: usize, y: usize| -> Color32 {
        let idx = y * frame_b.size[0] + x;
        if idx < frame_b.pixels.len() {
            frame_b.pixels[idx]
        } else {
            Color32::BLACK
        }
    };

    match kind {
        TransitionKind::CrossFade => {
            let inv_t = 1.0 - t;
            for y in 0..h {
                for x in 0..w {
                    let a = get_pixel_a(x, y);
                    let b = get_pixel_b(x, y);
                    out_pixels.push(Color32::from_rgba_premultiplied(
                        (a.r() as f32 * inv_t + b.r() as f32 * t) as u8,
                        (a.g() as f32 * inv_t + b.g() as f32 * t) as u8,
                        (a.b() as f32 * inv_t + b.b() as f32 * t) as u8,
                        (a.a() as f32 * inv_t + b.a() as f32 * t) as u8,
                    ));
                }
            }
        }
        TransitionKind::DipToBlack => {
            if t < 0.5 {
                let factor = 1.0 - (t * 2.0);
                for y in 0..h {
                    for x in 0..w {
                        let a = get_pixel_a(x, y);
                        out_pixels.push(Color32::from_rgba_premultiplied(
                            (a.r() as f32 * factor) as u8,
                            (a.g() as f32 * factor) as u8,
                            (a.b() as f32 * factor) as u8,
                            255,
                        ));
                    }
                }
            } else {
                let factor = (t - 0.5) * 2.0;
                for y in 0..h {
                    for x in 0..w {
                        let b = get_pixel_b(x, y);
                        out_pixels.push(Color32::from_rgba_premultiplied(
                            (b.r() as f32 * factor) as u8,
                            (b.g() as f32 * factor) as u8,
                            (b.b() as f32 * factor) as u8,
                            255,
                        ));
                    }
                }
            }
        }
        TransitionKind::DipToWhite => {
            if t < 0.5 {
                let factor = t * 2.0;
                for y in 0..h {
                    for x in 0..w {
                        let a = get_pixel_a(x, y);
                        out_pixels.push(Color32::from_rgba_premultiplied(
                            (a.r() as f32 + (255.0 - a.r() as f32) * factor) as u8,
                            (a.g() as f32 + (255.0 - a.g() as f32) * factor) as u8,
                            (a.b() as f32 + (255.0 - a.b() as f32) * factor) as u8,
                            255,
                        ));
                    }
                }
            } else {
                let factor = 1.0 - (t - 0.5) * 2.0;
                for y in 0..h {
                    for x in 0..w {
                        let b = get_pixel_b(x, y);
                        out_pixels.push(Color32::from_rgba_premultiplied(
                            (b.r() as f32 + (255.0 - b.r() as f32) * factor) as u8,
                            (b.g() as f32 + (255.0 - b.g() as f32) * factor) as u8,
                            (b.b() as f32 + (255.0 - b.b() as f32) * factor) as u8,
                            255,
                        ));
                    }
                }
            }
        }
        TransitionKind::WipeLeft => {
            let split_x = ((1.0 - t) * w as f32) as usize;
            for y in 0..h {
                for x in 0..w {
                    out_pixels.push(if x < split_x {
                        get_pixel_a(x, y)
                    } else {
                        get_pixel_b(x, y)
                    });
                }
            }
        }
        TransitionKind::WipeRight => {
            let split_x = (t * w as f32) as usize;
            for y in 0..h {
                for x in 0..w {
                    out_pixels.push(if x < split_x {
                        get_pixel_b(x, y)
                    } else {
                        get_pixel_a(x, y)
                    });
                }
            }
        }
        TransitionKind::WipeUp => {
            let split_y = ((1.0 - t) * h as f32) as usize;
            for y in 0..h {
                for x in 0..w {
                    out_pixels.push(if y < split_y {
                        get_pixel_a(x, y)
                    } else {
                        get_pixel_b(x, y)
                    });
                }
            }
        }
        TransitionKind::WipeDown => {
            let split_y = (t * h as f32) as usize;
            for y in 0..h {
                for x in 0..w {
                    out_pixels.push(if y < split_y {
                        get_pixel_b(x, y)
                    } else {
                        get_pixel_a(x, y)
                    });
                }
            }
        }
        TransitionKind::SlideLeft => {
            let shift_x = (t * w as f32) as usize;
            for y in 0..h {
                for x in 0..w {
                    if x + shift_x < w {
                        out_pixels.push(get_pixel_a(x + shift_x, y));
                    } else {
                        out_pixels.push(get_pixel_b(x + shift_x - w, y));
                    }
                }
            }
        }
        TransitionKind::SlideRight => {
            let shift_x = (t * w as f32) as usize;
            for y in 0..h {
                for x in 0..w {
                    if x < shift_x {
                        out_pixels.push(get_pixel_b(w - shift_x + x, y));
                    } else {
                        out_pixels.push(get_pixel_a(x - shift_x, y));
                    }
                }
            }
        }
        TransitionKind::SlideUp => {
            let shift_y = (t * h as f32) as usize;
            for y in 0..h {
                for x in 0..w {
                    if y + shift_y < h {
                        out_pixels.push(get_pixel_a(x, y + shift_y));
                    } else {
                        out_pixels.push(get_pixel_b(x, y + shift_y - h));
                    }
                }
            }
        }
        TransitionKind::SlideDown => {
            let shift_y = (t * h as f32) as usize;
            for y in 0..h {
                for x in 0..w {
                    if y < shift_y {
                        out_pixels.push(get_pixel_b(x, h - shift_y + y));
                    } else {
                        out_pixels.push(get_pixel_a(x, y - shift_y));
                    }
                }
            }
        }
        TransitionKind::SmoothLeft => {
            let center_x = (1.0 - t) * w as f32;
            let feather = (w as f32 * 0.12).max(8.0);
            for y in 0..h {
                for x in 0..w {
                    let diff = x as f32 - center_x;
                    let blend_b = ((diff / feather) + 0.5).clamp(0.0, 1.0);
                    let blend_a = 1.0 - blend_b;
                    let a = get_pixel_a(x, y);
                    let b = get_pixel_b(x, y);
                    out_pixels.push(Color32::from_rgba_premultiplied(
                        (a.r() as f32 * blend_a + b.r() as f32 * blend_b) as u8,
                        (a.g() as f32 * blend_a + b.g() as f32 * blend_b) as u8,
                        (a.b() as f32 * blend_a + b.b() as f32 * blend_b) as u8,
                        255,
                    ));
                }
            }
        }
        TransitionKind::CircleOpen => {
            let cx = w as f32 / 2.0;
            let cy = h as f32 / 2.0;
            let max_r = (cx * cx + cy * cy).sqrt();
            let current_r = max_r * t;
            let r_sq = current_r * current_r;
            for y in 0..h {
                let dy = y as f32 - cy;
                let dy_sq = dy * dy;
                for x in 0..w {
                    let dx = x as f32 - cx;
                    let dist_sq = dx * dx + dy_sq;
                    out_pixels.push(if dist_sq <= r_sq {
                        get_pixel_b(x, y)
                    } else {
                        get_pixel_a(x, y)
                    });
                }
            }
        }
        TransitionKind::CircleClose => {
            let cx = w as f32 / 2.0;
            let cy = h as f32 / 2.0;
            let max_r = (cx * cx + cy * cy).sqrt();
            let current_r = max_r * (1.0 - t);
            let r_sq = current_r * current_r;
            for y in 0..h {
                let dy = y as f32 - cy;
                let dy_sq = dy * dy;
                for x in 0..w {
                    let dx = x as f32 - cx;
                    let dist_sq = dx * dx + dy_sq;
                    out_pixels.push(if dist_sq <= r_sq {
                        get_pixel_a(x, y)
                    } else {
                        get_pixel_b(x, y)
                    });
                }
            }
        }
        TransitionKind::Radial => {
            let cx = w as f32 / 2.0;
            let cy = h as f32 / 2.0;
            let target_angle = t * std::f32::consts::TAU;
            for y in 0..h {
                let dy = y as f32 - cy;
                for x in 0..w {
                    let dx = x as f32 - cx;
                    let mut angle = dy.atan2(dx) + std::f32::consts::PI; // 0..TAU
                    if angle >= std::f32::consts::TAU {
                        angle -= std::f32::consts::TAU;
                    }
                    out_pixels.push(if angle <= target_angle {
                        get_pixel_b(x, y)
                    } else {
                        get_pixel_a(x, y)
                    });
                }
            }
        }
        TransitionKind::ZoomIn => {
            let zoom = 0.5 + 0.5 * t;
            let cx = w as f32 / 2.0;
            let cy = h as f32 / 2.0;
            for y in 0..h {
                for x in 0..w {
                    let src_x = ((x as f32 - cx) / zoom + cx).clamp(0.0, (w - 1) as f32) as usize;
                    let src_y = ((y as f32 - cy) / zoom + cy).clamp(0.0, (h - 1) as f32) as usize;
                    let a = get_pixel_a(x, y);
                    let b = get_pixel_b(src_x, src_y);
                    out_pixels.push(Color32::from_rgba_premultiplied(
                        (a.r() as f32 * (1.0 - t) + b.r() as f32 * t) as u8,
                        (a.g() as f32 * (1.0 - t) + b.g() as f32 * t) as u8,
                        (a.b() as f32 * (1.0 - t) + b.b() as f32 * t) as u8,
                        255,
                    ));
                }
            }
        }
        TransitionKind::SqueezeHorizontal => {
            let squeeze = (1.0 - t).max(0.01);
            for y in 0..h {
                for x in 0..w {
                    let src_x = (x as f32 / squeeze) as usize;
                    if src_x < w {
                        out_pixels.push(get_pixel_a(src_x, y));
                    } else {
                        out_pixels.push(get_pixel_b(x, y));
                    }
                }
            }
        }
        TransitionKind::Pixelate => {
            let block_size = if t < 0.5 {
                (t * 40.0).max(1.0) as usize
            } else {
                ((1.0 - t) * 40.0).max(1.0) as usize
            };
            for y in 0..h {
                let block_y = (y / block_size) * block_size;
                for x in 0..w {
                    let block_x = (x / block_size) * block_size;
                    let a = get_pixel_a(block_x, block_y);
                    let b = get_pixel_b(block_x, block_y);
                    out_pixels.push(Color32::from_rgba_premultiplied(
                        (a.r() as f32 * (1.0 - t) + b.r() as f32 * t) as u8,
                        (a.g() as f32 * (1.0 - t) + b.g() as f32 * t) as u8,
                        (a.b() as f32 * (1.0 - t) + b.b() as f32 * t) as u8,
                        255,
                    ));
                }
            }
        }
    }

    ColorImage {
        size: [w, h],
        pixels: out_pixels,
    }
}

/// Blend a single video frame for leading edge fade-ins (Dip to Black / Dip to White).
pub fn blend_fade_in(frame: &ColorImage, kind: TransitionKind, progress: f32) -> ColorImage {
    let t = progress.clamp(0.0, 1.0);
    if t >= 0.999 {
        return frame.clone();
    }

    let w = frame.size[0];
    let h = frame.size[1];
    let mut out_pixels = Vec::with_capacity(w * h);

    match kind {
        TransitionKind::DipToWhite => {
            let factor = 1.0 - t;
            for p in &frame.pixels {
                out_pixels.push(Color32::from_rgba_premultiplied(
                    (p.r() as f32 + (255.0 - p.r() as f32) * factor) as u8,
                    (p.g() as f32 + (255.0 - p.g() as f32) * factor) as u8,
                    (p.b() as f32 + (255.0 - p.b() as f32) * factor) as u8,
                    255,
                ));
            }
        }
        _ => {
            // Default to fading in from black
            for p in &frame.pixels {
                out_pixels.push(Color32::from_rgba_premultiplied(
                    (p.r() as f32 * t) as u8,
                    (p.g() as f32 * t) as u8,
                    (p.b() as f32 * t) as u8,
                    255,
                ));
            }
        }
    }

    ColorImage {
        size: [w, h],
        pixels: out_pixels,
    }
}
