use crate::core::text_overlay::{SlideBackground, SlideElement, TextBoxStyle, TextOverlay};
use crate::core::timeline::Timeline;
use crate::core::track::TrackKind;
use crate::core::transition::Transition;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportConfig {
    pub output_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub video_bitrate_kbps: u32,
    pub audio_bitrate_kbps: u32,
    pub encoder: EncoderType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncoderType {
    Libx264,
    VaapiH264,
    QsvH264,
    NvencH264,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            output_path: PathBuf::from("output.mp4"),
            width: 1920,
            height: 1080,
            fps: 30.0,
            video_bitrate_kbps: 8000,
            audio_bitrate_kbps: 192,
            encoder: EncoderType::Libx264,
        }
    }
}

/// Collect every path that must become an input (`-i`): each clip's media, each slide
/// picture/video element, each audio element, and picture backgrounds.
fn collect_sources(
    timeline: &Timeline,
) -> (Vec<PathBuf>, HashMap<PathBuf, usize>) {
    let mut unique: Vec<PathBuf> = Vec::new();
    let mut map: HashMap<PathBuf, usize> = HashMap::new();
    for track in &timeline.tracks {
        for clip in &track.clips {
            if !clip.source_path.as_os_str().is_empty() {
                if !map.contains_key(&clip.source_path) {
                    let i = unique.len();
                    map.insert(clip.source_path.clone(), i);
                    unique.push(clip.source_path.clone());
                }
            }
            if let Some(SlideBackground::Picture(p)) = &clip.background {
                if !map.contains_key(p) {
                    let i = unique.len();
                    map.insert(p.clone(), i);
                    unique.push(p.clone());
                }
            }
            for el in &clip.elements {
                let p = match el {
                    SlideElement::Picture { path, .. }
                    | SlideElement::Sticker { path, .. }
                    | SlideElement::Video { path, .. }
                    | SlideElement::Audio { path, .. } => Some(path),
                    _ => None,
                };
                if let Some(p) = p {
                    if !map.contains_key(p) {
                        let i = unique.len();
                        map.insert(p.clone(), i);
                        unique.push(p.clone());
                    }
                }
            }
        }
    }
    (unique, map)
}

/// How long a looped still input must run to satisfy every clip that reads it.
///
/// A base-layer image clip is cut with `trim=start=..:end=source_out`, so the input has to
/// carry frames all the way to the largest `source_out` among the clips using it. Overlay
/// uses need no such reach (framesync repeats the last frame of a secondary input), but the
/// timeline duration is kept as a floor so a still always outlasts what it is composited on.
fn image_input_duration(timeline: &Timeline, src: &PathBuf) -> f64 {
    let mut needed = timeline.duration().as_secs_f64();
    for track in &timeline.tracks {
        for clip in &track.clips {
            if &clip.source_path == src {
                needed = needed.max(clip.source_out.as_secs_f64());
            }
        }
    }
    needed.max(0.04)
}

/// Builds the complete FFmpeg CLI command arguments for rendering the timeline project.
pub fn build_ffmpeg_export_command(
    timeline: &Timeline,
    config: &ExportConfig,
) -> Result<Vec<String>, String> {
    if timeline.duration().micros == 0 {
        return Err("Timeline is empty. Add media clips before exporting.".to_string());
    }

    let mut args = Vec::new();
    args.push("-y".to_string());

    let (unique_sources, source_to_input_idx) = collect_sources(timeline);
    let has_any_clips = timeline.tracks.iter().any(|t| !t.clips.is_empty());
    if !has_any_clips {
        return Err("No media clips or title cards found on the timeline.".to_string());
    }

    for src in &unique_sources {
        let is_image = matches!(
            src.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()).as_deref(),
            Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("webp") | Some("bmp")
        );
        if is_image {
            // Loop a still so it can fill the clips that use it. `-loop` and `-t` are input
            // options, so they must precede this input's `-i`. `-t` is required: an unbounded
            // `-loop 1` input never reaches EOF and the render runs forever.
            args.push("-loop".to_string());
            args.push("1".to_string());
            args.push("-t".to_string());
            args.push(format!("{:.3}", image_input_duration(timeline, src)));
        }
        args.push("-i".to_string());
        args.push(src.to_str().unwrap_or_default().to_string());
    }

    let wd = config.width as i64;
    let ht = config.height as i64;

    let mut filter_chains: Vec<String> = Vec::new();
    let mut video_out_labels: Vec<String> = Vec::new();
    let mut audio_out_labels: Vec<String> = Vec::new();
    let mut video_meta: Vec<(f64, Option<Transition>, Option<Transition>)> = Vec::new();
    let mut clip_counter = 0;

    for track in &timeline.tracks {
        if track.is_muted {
            continue;
        }
        for clip in &track.clips {
            let start_ms = (clip.timeline_start.as_secs_f64() * 1000.0).round() as i64;
            let dur = clip.duration().as_secs_f64();
            let is_video_track = track.kind == TrackKind::Video;
            let participates_video = is_video_track
                && (clip.has_video
                    || clip.background.is_some()
                    || clip.elements.iter().any(|e| e.is_visual()));

            // ---------------- Video chain ----------------
            if participates_video {
                let v_label = format!("v{}", clip_counter);
                let mut current = v_label.clone();

                // 1. Base layer: streamed media, or a solid/picture background.
                if clip.has_video {
                    let ii = *source_to_input_idx
                        .get(&clip.source_path)
                        .ok_or_else(|| "Source index lookup error".to_string())?;
                    let in_sec = clip.source_in.as_secs_f64();
                    let out_sec = clip.source_out.as_secs_f64();
                    let base = format!(
                        "[{}:v]trim=start={:.3}:end={:.3},setpts=PTS-STARTPTS,scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2,setsar=1,fps={}[{}]",
                        ii, in_sec, out_sec, wd, ht, wd, ht, config.fps, current
                    );
                    filter_chains.push(base);
                } else {
                    let bg_label = match &clip.background {
                        Some(SlideBackground::Solid(c)) => {
                            let hex = format!(
                                "0x{:02X}{:02X}{:02X}",
                                c.r(),
                                c.g(),
                                c.b()
                            );
                            format!(
                                "color=c={}:s={}x{}:d={:.3}:r={}[{}]",
                                hex, wd, ht, dur, config.fps, current
                            )
                        }
                        // A picture backdrop is handled as a full-frame overlay element below,
                        // so the base is just black behind it.
                        _ => format!("color=c=black:s={}x{}:d={:.3}:r={}[{}]", wd, ht, dur, config.fps, current),
                    };
                    filter_chains.push(bg_label);
                }

                // A Picture backdrop overlays the whole frame, underneath the other elements.
                if let Some(SlideBackground::Picture(p)) = &clip.background {
                    let ii = *source_to_input_idx.get(p).ok_or("bg input")?;
                    let fl = next_overlay_label();
                    let outl = next_overlay_label();
                    filter_chains.push(format!(
                        "[{}:v]scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2,setsar=1[{}]",
                        ii, wd, ht, wd, ht, fl
                    ));
                    filter_chains.push(format!("[{current}][{fl}]overlay=0:0[{}]", outl));
                    current = outl;
                }

                // 2. Overlay every visual element in z-order (x/y top-left normalized).
                for (ei, el) in clip.elements.iter().enumerate() {
                    match el {
                        SlideElement::Picture { path, x, y, w, h }
                        | SlideElement::Sticker { path, x, y, w, h, .. }
                        | SlideElement::Video { path, x, y, w, h } => {
                            let ii = *source_to_input_idx.get(path).ok_or("elem input")?;
                            let px = ((x.min(1.0) * wd as f32) as i64).clamp(0, wd - 1);
                            let py = ((y.min(1.0) * ht as f32) as i64).clamp(0, ht - 1);
                            let pw = ((w.min(1.0) * wd as f32) as i64).clamp(16, wd);
                            let ph = ((h.min(1.0) * ht as f32) as i64).clamp(16, ht);
                            let fl = format!("sv{}_{}", clip_counter, ei);
                            let mut src = format!(
                                "[{}:v]scale={}:{}:force_original_aspect_ratio=decrease,setpts=PTS-STARTPTS",
                                ii, pw, ph
                            );
                            if matches!(el, SlideElement::Video { .. }) {
                                src.push_str(&format!(",trim=start=0:end={:.3}", dur));
                            }
                            src.push_str(&format!("[{}]", fl));
                            filter_chains.push(src);
                            let outl = next_overlay_label();
                            filter_chains.push(format!(
                                "[{current}][{fl}]overlay=x={}:y={}[{}]",
                                px, py, outl
                            ));
                            current = outl;
                        }
                        SlideElement::Text(o) if !o.text.trim().is_empty() => {
                            let dt = build_drawtext_filter(o, config);
                            if !dt.is_empty() {
                                let outl = next_overlay_label();
                                filter_chains.push(format!("[{}]{}[{}]", current, dt, outl));
                                current = outl;
                            }
                        }
                        SlideElement::Calendar(c) => {
                            let mut text_overlay = TextOverlay::default();
                            text_overlay.text = crate::core::calendar_gen::CalendarMonth::format_multi_month_string(
                                c.year,
                                c.start_month,
                                c.month_count,
                                c.show_holidays,
                                crate::core::calendar_gen::CalendarStyle::BoxedGrid,
                                &c.holidays,
                                &c.custom_events,
                            );
                            text_overlay.x = c.x + c.w / 2.0;
                            text_overlay.y = c.y + c.h / 2.0;
                            text_overlay.font_family = crate::core::text_overlay::FontFamilyPreset::Monospace;
                            text_overlay.box_style = crate::core::text_overlay::TextBoxStyle::TranslucentBox;
                            let dt = build_drawtext_filter(&text_overlay, config);
                            if !dt.is_empty() {
                                let outl = next_overlay_label();
                                filter_chains.push(format!("[{}]{}[{}]", current, dt, outl));
                                current = outl;
                            }
                        }
                        _ => {}
                    }
                }

                video_out_labels.push(current.clone());
                video_meta.push((dur, clip.start_transition().cloned(), clip.end_transition().cloned()));
            }

            // ---------------- Audio: normal clip audio ----------------
            if clip.has_audio && !clip.source_path.as_os_str().is_empty() {
                let ii = *source_to_input_idx
                    .get(&clip.source_path)
                    .ok_or_else(|| "Source index lookup error".to_string())?;
                let a_label = format!("a{}", clip_counter);
                let mut a_filter = format!(
                    "[{}:a]atrim=start={:.3}:end={:.3},asetpts=PTS-STARTPTS",
                    ii, clip.source_in.as_secs_f64(), clip.source_out.as_secs_f64()
                );
                if let Some(vol_expr) = clip.volume_envelope.to_ffmpeg_volume_expression() {
                    a_filter.push_str(&format!(",{}", vol_expr));
                }
                if (track.volume - 1.0).abs() > 0.01 {
                    a_filter.push_str(&format!(",volume={:.3}", track.volume));
                }
                a_filter.push_str(&format!(",adelay={}|{}[{}]", start_ms, start_ms, a_label));
                filter_chains.push(a_filter);
                audio_out_labels.push(a_label);
            }

            // ---------------- Audio: slide audio elements ----------------
            for (ei, el) in clip.elements.iter().enumerate() {
                if let SlideElement::Audio { path, volume } = el {
                    let ii = *source_to_input_idx.get(path).ok_or("audio elem input")?;
                    let al = format!("sa{}_{}", clip_counter, ei);
                    let f = format!(
                        "[{}:a]atrim=start=0:end={:.3},asetpts=PTS-STARTPTS,volume={:.3},adelay={}|{}[{}]",
                        ii, dur, volume, start_ms, start_ms, al
                    );
                    filter_chains.push(f);
                    audio_out_labels.push(al);
                }
            }

            clip_counter += 1;
        }
    }

    // ---------------- Combine video (xfade) ----------------
    let mut final_video_label = "v_final".to_string();
    if video_out_labels.is_empty() {
        let total_dur = timeline.duration().as_secs_f64();
        filter_chains.push(format!(
            "color=c=black:s={}x{}:d={:.3}:r={}[v_final]",
            wd, ht, total_dur, config.fps
        ));
    } else if video_out_labels.len() == 1 {
        let mut base_lbl = video_out_labels[0].clone();
        if let Some(tr) = &video_meta[0].1 {
            let col = if tr.kind == crate::core::transition::TransitionKind::DipToWhite {
                "white"
            } else {
                "black"
            };
            let fade_lbl = "v_fade_in_0".to_string();
            filter_chains.push(format!(
                "[{}]fade=t=in:st=0:d={:.3}:color={}[{}]",
                base_lbl, tr.duration_secs, col, fade_lbl
            ));
            base_lbl = fade_lbl;
        }
        if let Some(tr) = &video_meta[0].2 {
            let col = if tr.kind == crate::core::transition::TransitionKind::DipToWhite {
                "white"
            } else {
                "black"
            };
            let fade_lbl = "v_fade_out_0".to_string();
            let start_out = (video_meta[0].0 - tr.duration_secs).max(0.0);
            filter_chains.push(format!(
                "[{}]fade=t=out:st={:.3}:d={:.3}:color={}[{}]",
                base_lbl, start_out, tr.duration_secs, col, fade_lbl
            ));
            base_lbl = fade_lbl;
        }
        final_video_label = base_lbl;
    } else {
        let mut initial_lbl = video_out_labels[0].clone();
        if let Some(tr) = &video_meta[0].1 {
            let col = if tr.kind == crate::core::transition::TransitionKind::DipToWhite {
                "white"
            } else {
                "black"
            };
            let fade_lbl = "v_fade_in_0".to_string();
            filter_chains.push(format!(
                "[{}]fade=t=in:st=0:d={:.3}:color={}[{}]",
                initial_lbl, tr.duration_secs, col, fade_lbl
            ));
            initial_lbl = fade_lbl;
        }

        let n = video_out_labels.len();
        let active_transitions: Vec<Option<Transition>> = (0..n)
            .map(|i| {
                if i == 0 {
                    None
                } else {
                    video_meta[i].1.or(video_meta[i - 1].2)
                }
            })
            .collect();
        let overlaps: Vec<f64> = (0..n)
            .map(|i| {
                if i == 0 {
                    0.0
                } else {
                    active_transitions[i].map(|t| t.duration_secs).unwrap_or(0.05)
                }
            })
            .collect();

        let mut ready: Vec<String> = Vec::with_capacity(n);
        for i in 0..n {
            let in_lbl = if i == 0 {
                initial_lbl.clone()
            } else {
                video_out_labels[i].clone()
            };
            if i + 1 < n {
                let out_lbl = format!("vx_{}", i);
                filter_chains.push(format!(
                    "[{}]tpad=stop_mode=clone:stop_duration={:.3}[{}]",
                    in_lbl, overlaps[i + 1], out_lbl
                ));
                ready.push(out_lbl);
            } else {
                ready.push(in_lbl);
            }
        }

        let mut sum_dur = 0.0f64;
        let mut sum_overlap = 0.0f64;
        let mut current = ready[0].clone();
        for i in 1..n {
            sum_dur += video_meta[i - 1].0;
            sum_overlap += overlaps[i];
            let offset = (sum_dur - sum_overlap).max(0.0);
            let kind = active_transitions[i]
                .map(|t| t.kind)
                .unwrap_or(crate::core::transition::TransitionKind::CrossFade);
            let out_lbl = format!("vxo_{}", i);
            filter_chains.push(format!(
                "[{current}][{}]xfade=transition={}:duration={:.3}:offset={:.3}[{}]",
                ready[i], kind.to_xfade(), overlaps[i], offset, out_lbl
            ));
            current = out_lbl;
        }

        if let Some(tr) = &video_meta[n - 1].2 {
            let fade_lbl = "v_fade_out_end".to_string();
            let col = if tr.kind == crate::core::transition::TransitionKind::DipToWhite {
                "white"
            } else {
                "black"
            };
            let total_dur = (sum_dur + video_meta[n - 1].0 - sum_overlap).max(0.0);
            let start_out = (total_dur - tr.duration_secs).max(0.0);
            filter_chains.push(format!(
                "[{}]fade=t=out:st={:.3}:d={:.3}:color={}[{}]",
                current, start_out, tr.duration_secs, col, fade_lbl
            ));
            current = fade_lbl;
        }
        final_video_label = current;
    }

    // ---------------- Combine audio (amix) ----------------
    let mut final_audio_label = "a_final".to_string();
    if audio_out_labels.is_empty() {
        let total_dur = timeline.duration().as_secs_f64();
        filter_chains.push(format!(
            "anullsrc=r=48000:cl=stereo:d={:.3}[a_final]",
            total_dur
        ));
    } else if audio_out_labels.len() == 1 {
        final_audio_label = audio_out_labels[0].clone();
    } else {
        let mut mix_str = String::new();
        for label in &audio_out_labels {
            mix_str.push_str(&format!("[{}]", label));
        }
        mix_str.push_str(&format!(
            "amix=inputs={}:duration=longest:dropout_transition=0,volume={}[a_final]",
            audio_out_labels.len(),
            audio_out_labels.len()
        ));
        filter_chains.push(mix_str);
    }

    let filter_complex_script = filter_chains.join(";");
    args.push("-filter_complex".to_string());
    args.push(filter_complex_script);

    args.push("-map".to_string());
    args.push(format!("[{}]", final_video_label));
    args.push("-map".to_string());
    args.push(format!("[{}]", final_audio_label));

    match config.encoder {
        EncoderType::Libx264 => {
            args.push("-c:v".to_string());
            args.push("libx264".to_string());
            args.push("-preset".to_string());
            args.push("fast".to_string());
            args.push("-crf".to_string());
            args.push("20".to_string());
            args.push("-pix_fmt".to_string());
            args.push("yuv420p".to_string());
        }
        EncoderType::VaapiH264 => {
            args.push("-c:v".to_string());
            args.push("h264_vaapi".to_string());
            args.push("-b:v".to_string());
            args.push(format!("{}k", config.video_bitrate_kbps));
        }
        EncoderType::QsvH264 => {
            args.push("-c:v".to_string());
            args.push("h264_qsv".to_string());
            args.push("-b:v".to_string());
            args.push(format!("{}k", config.video_bitrate_kbps));
        }
        EncoderType::NvencH264 => {
            args.push("-c:v".to_string());
            args.push("h264_nvenc".to_string());
            args.push("-b:v".to_string());
            args.push(format!("{}k", config.video_bitrate_kbps));
            args.push("-pix_fmt".to_string());
            args.push("yuv420p".to_string());
        }
    }

    args.push("-c:a".to_string());
    args.push("aac".to_string());
    args.push("-b:a".to_string());
    args.push(format!("{}k", config.audio_bitrate_kbps));
    args.push("-ar".to_string());
    args.push("48000".to_string());

    args.push("-y".to_string());
    args.push("-progress".to_string());
    args.push("pipe:1".to_string());
    args.push(config.output_path.to_str().unwrap_or("output.mp4").to_string());

    Ok(args)
}

/// Monotonically increasing unique label for overlay outputs.
#[allow(non_snake_case)]
fn next_overlay_label() -> String {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static C: AtomicUsize = AtomicUsize::new(0);
    format!("ol{}", C.fetch_add(1, Ordering::Relaxed) + 5000)
}

fn build_drawtext_filter(overlay: &TextOverlay, config: &ExportConfig) -> String {
    let formatted = overlay.formatted_text();
    let lines: Vec<&str> = formatted.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    let font_size = ((overlay.font_size / 720.0) * (config.height as f32))
        .max(14.0)
        .round() as u32;

    let base_font = overlay.font_family.ffmpeg_font_name();
    let font_name = match (overlay.is_bold, overlay.is_italic) {
        (true, true) => format!("{} Bold Italic", base_font),
        (true, false) => format!("{} Bold", base_font),
        (false, true) => format!("{} Italic", base_font),
        (false, false) => base_font.to_string(),
    };

    let paint = crate::core::TextPaint::from_color32(overlay.text_color);
    let hex_color = paint.to_ffmpeg_fontcolor();

    // Center-anchored, matching the preview: a click near the middle maps to where it was placed.
    let x_expr = format!("(w-text_w)/2 + ({} - 0.5)*w", overlay.x);
    let y_base = format!("(h-text_h)/2 + ({} - 0.5)*h", overlay.y);

    let box_str = match overlay.box_style {
        TextBoxStyle::None => {
            if overlay.show_shadow {
                ":shadowcolor=black@0.8:shadowx=2:shadowy=2".to_string()
            } else {
                String::new()
            }
        }
        TextBoxStyle::TranslucentBox => {
            format!(":box=1:boxcolor=black@{:.2}:boxborderw=16", overlay.box_opacity)
        }
        TextBoxStyle::SolidBanner => {
            format!(":box=1:boxcolor=black@{:.2}:boxborderw=24", overlay.box_opacity)
        }
    };

    let mut drawtext_filters = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let escaped = line
            .replace('\\', "\\\\")
            .replace('\'', "'\\\\''")
            .replace(':', "\\:")
            .replace('%', "\\%");
        let line_y_expr = if lines.len() == 1 {
            y_base.clone()
        } else {
            let offset = (i as i32 - (lines.len() as i32 / 2)) * (font_size as i32 + 10);
            if offset >= 0 {
                format!("{}+{}", y_base, offset)
            } else {
                format!("{}-{}", y_base, -offset)
            }
        };
        drawtext_filters.push(format!(
            "drawtext=text='{}':font='{}':fontsize={}:fontcolor={}{}:x={}:y={}",
            escaped, font_name, font_size, hex_color, box_str, x_expr, line_y_expr
        ));
    }
    drawtext_filters.join(",")
}
