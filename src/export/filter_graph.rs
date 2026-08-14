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

/// Builds the complete FFmpeg CLI command arguments for rendering the timeline project.
pub fn build_ffmpeg_export_command(
    timeline: &Timeline,
    config: &ExportConfig,
) -> Result<Vec<String>, String> {
    if timeline.duration().micros == 0 {
        return Err("Timeline is empty. Add media clips before exporting.".to_string());
    }

    let mut args = Vec::new();
    args.push("-y".to_string()); // Overwrite output file

    // 1. Collect unique media source paths (excluding synthetic title cards)
    let mut unique_sources: Vec<PathBuf> = Vec::new();
    let mut source_to_input_idx: HashMap<PathBuf, usize> = HashMap::new();

    for track in &timeline.tracks {
        for clip in &track.clips {
            if !clip.is_title_card && !source_to_input_idx.contains_key(&clip.source_path) {
                let idx = unique_sources.len();
                source_to_input_idx.insert(clip.source_path.clone(), idx);
                unique_sources.push(clip.source_path.clone());
            }
        }
    }

    let has_any_clips = timeline.tracks.iter().any(|t| !t.clips.is_empty());
    if !has_any_clips {
        return Err("No media clips or title cards found on the timeline.".to_string());
    }

    // Add inputs to FFmpeg args
    for src in &unique_sources {
        args.push("-i".to_string());
        args.push(src.to_str().unwrap_or_default().to_string());
    }

    // 2. Build Filter Complex Graph
    let mut filter_chains = Vec::new();
    let mut video_out_labels = Vec::new();
    let mut audio_out_labels = Vec::new();
    let mut video_meta: Vec<(f64, Option<Transition>, Option<Transition>)> = Vec::new();

    let mut clip_counter = 0;

    for track in &timeline.tracks {
        if track.is_muted {
            continue;
        }

        for clip in &track.clips {
            let in_sec = clip.source_in.as_secs_f64();
            let out_sec = clip.source_out.as_secs_f64();
            let start_ms = (clip.timeline_start.as_secs_f64() * 1000.0).round() as i64;

            // Video processing
            if clip.has_video && track.kind == TrackKind::Video {
                let v_label = format!("v{}", clip_counter);
                let mut v_filter = if clip.is_title_card {
                    let theme = clip.title_card_theme.unwrap_or_default();
                    let (c1, _) = theme.colors();
                    let hex = format!("0x{:02X}{:02X}{:02X}", c1.r(), c1.g(), c1.b());
                    format!(
                        "color=c={}:s={}x{}:d={:.3}:r={}",
                        hex,
                        config.width,
                        config.height,
                        clip.duration().as_secs_f64(),
                        config.fps
                    )
                } else {
                    let input_idx = *source_to_input_idx
                        .get(&clip.source_path)
                        .ok_or_else(|| "Source index lookup error".to_string())?;
                    format!(
                        "[{}:v]trim=start={:.3}:end={:.3},setpts=PTS-STARTPTS,scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2,setsar=1,fps={}",
                        input_idx,
                        in_sec,
                        out_sec,
                        config.width,
                        config.height,
                        config.width,
                        config.height,
                        config.fps,
                    )
                };

                // Append on-screen text overlay drawtext filter if present
                if let Some(ref overlay) = clip.text_overlay {
                    if !overlay.text.trim().is_empty() {
                        v_filter.push_str(&build_drawtext_filter(overlay, config));
                    }
                }

                v_filter.push_str(&format!("[{}]", v_label));
                filter_chains.push(v_filter);
                video_out_labels.push(v_label.clone());
                video_meta.push((
                    clip.duration().as_secs_f64(),
                    clip.start_transition().cloned(),
                    clip.end_transition().cloned(),
                ));
            }

            // Audio processing
            if clip.has_audio && !clip.is_title_card {
                let input_idx = *source_to_input_idx
                    .get(&clip.source_path)
                    .ok_or_else(|| "Source index lookup error".to_string())?;
                let a_label = format!("a{}", clip_counter);
                let mut a_filter = format!(
                    "[{}:a]atrim=start={:.3}:end={:.3},asetpts=PTS-STARTPTS",
                    input_idx, in_sec, out_sec
                );

                // Apply dynamic Volume Envelope
                if let Some(vol_expr) = clip.volume_envelope.to_ffmpeg_volume_expression() {
                    a_filter.push_str(&format!(",{}", vol_expr));
                }

                // Apply track volume
                if (track.volume - 1.0).abs() > 0.01 {
                    a_filter.push_str(&format!(",volume={:.3}", track.volume));
                }

                // Apply timeline delay
                a_filter.push_str(&format!(",adelay={}|{}[{}]", start_ms, start_ms, a_label));

                filter_chains.push(a_filter);
                audio_out_labels.push(a_label);
            }

            clip_counter += 1;
        }
    }

    // Combine Video Tracks
    let mut final_video_label = "v_final".to_string();
    if video_out_labels.is_empty() {
        // Generate a black background video if no video tracks exist
        let total_dur = timeline.duration().as_secs_f64();
        filter_chains.push(format!(
            "color=c=black:s={}x{}:d={:.3}:r={}[v_final]",
            config.width, config.height, total_dur, config.fps
        ));
    } else if video_out_labels.len() == 1 {
        let mut base_lbl = video_out_labels[0].clone();
        if let Some(tr) = &video_meta[0].1 {
            let fade_lbl = "v_fade_in_0".to_string();
            let col = if tr.kind == crate::core::transition::TransitionKind::DipToWhite {
                "white"
            } else {
                "black"
            };
            filter_chains.push(format!(
                "[{}]fade=t=in:st=0:d={:.3}:color={}[{}]",
                base_lbl, tr.duration_secs, col, fade_lbl
            ));
            base_lbl = fade_lbl;
        }
        if let Some(tr) = &video_meta[0].2 {
            let fade_lbl = "v_fade_out_0".to_string();
            let col = if tr.kind == crate::core::transition::TransitionKind::DipToWhite {
                "white"
            } else {
                "black"
            };
            let start_out = (video_meta[0].0 - tr.duration_secs).max(0.0);
            filter_chains.push(format!(
                "[{}]fade=t=out:st={:.3}:d={:.3}:color={}[{}]",
                base_lbl, start_out, tr.duration_secs, col, fade_lbl
            ));
            base_lbl = fade_lbl;
        }
        final_video_label = base_lbl;
    } else {
        // First, if clip 0 has a leading fade-in, apply it
        let mut initial_lbl = video_out_labels[0].clone();
        if let Some(tr) = &video_meta[0].1 {
            let fade_lbl = "v_fade_in_0".to_string();
            let col = if tr.kind == crate::core::transition::TransitionKind::DipToWhite {
                "white"
            } else {
                "black"
            };
            filter_chains.push(format!(
                "[{}]fade=t=in:st=0:d={:.3}:color={}[{}]",
                initial_lbl, tr.duration_secs, col, fade_lbl
            ));
            initial_lbl = fade_lbl;
        }

        // Chain the clips through ffmpeg xfade so the selected transition blends each clip
        // into the next one. Boundaries with no transition use a near-instant crossfade (a
        // quick cut).
        let n = video_out_labels.len();
        // Overlap feeding INTO clip i (i>=1): the transition duration attached to clip i (or out of i-1).
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
                    active_transitions[i]
                        .map(|t| t.duration_secs)
                        .unwrap_or(0.05)
                }
            })
            .collect();

        // Every clip except the last gets a cloned tail as long as the next transition, so
        // the previous picture is still showing during the blend.
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
                    in_lbl,
                    overlaps[i + 1],
                    out_lbl
                ));
                ready.push(out_lbl);
            } else {
                ready.push(in_lbl);
            }
        }

        // xfade offset bookkeeping: offset_i = (sum durations before i) - (sum overlaps up to i).
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
                "[{}][{}]xfade=transition={}:duration={:.3}:offset={:.3}[{}]",
                current,
                ready[i],
                kind.to_xfade(),
                overlaps[i],
                offset,
                out_lbl
            ));
            current = out_lbl;
        }

        // Check if final clip has an End fade out
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

    // Combine Audio Tracks
    let mut final_audio_label = "a_final".to_string();
    if audio_out_labels.is_empty() {
        // Generate a silent audio stream if no audio tracks exist
        let total_dur = timeline.duration().as_secs_f64();
        filter_chains.push(format!(
            "anullsrc=r=48000:cl=stereo:d={:.3}[a_final]",
            total_dur
        ));
    } else if audio_out_labels.len() == 1 {
        final_audio_label = audio_out_labels[0].clone();
    } else {
        // Mix all audio streams using amix
        let mut mix_str = String::new();
        for label in &audio_out_labels {
            mix_str.push_str(&format!("[{}]", label));
        }
        mix_str.push_str(&format!(
            "amix=inputs={}:duration=longest:dropout_transition=0,volume={}[a_final]",
            audio_out_labels.len(),
            audio_out_labels.len() // Compensation factor
        ));
        filter_chains.push(mix_str);
    }

    let filter_complex_script = filter_chains.join(";");
    args.push("-filter_complex".to_string());
    args.push(filter_complex_script);

    // Map final streams
    args.push("-map".to_string());
    args.push(format!("[{}]", final_video_label));
    args.push("-map".to_string());
    args.push(format!("[{}]", final_audio_label));

    // Video encoder selection
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

    // Audio encoder
    args.push("-c:a".to_string());
    args.push("aac".to_string());
    args.push("-b:a".to_string());
    args.push(format!("{}k", config.audio_bitrate_kbps));
    args.push("-ar".to_string());
    args.push("48000".to_string());

    // Progress reporting
    args.push("-progress".to_string());
    args.push("pipe:1".to_string());

    // Output destination
    args.push(config.output_path.to_str().unwrap_or("output.mp4").to_string());

    Ok(args)
}

fn build_drawtext_filter(
    overlay: &crate::core::text_overlay::TextOverlay,
    config: &ExportConfig,
) -> String {
    use crate::core::text_overlay::TextPosition;
    let mut out = String::new();

    let escaped_text = overlay
        .text
        .replace('\\', "\\\\")
        .replace('\'', "'\\''")
        .replace(':', "\\:")
        .replace('%', "\\%");

    let font_size = ((overlay.font_size / 720.0) * (config.height as f32))
        .max(16.0)
        .round() as u32;

    let (x_expr, y_expr) = match overlay.position {
        TextPosition::CenterTitle => {
            if overlay.subtitle.is_some() {
                ("(w-text_w)/2", "(h-text_h)/2-30")
            } else {
                ("(w-text_w)/2", "(h-text_h)/2")
            }
        }
        TextPosition::BottomBanner => ("(w-text_w)/2", "h-text_h-50"),
        TextPosition::TopHeader => ("(w-text_w)/2", "50"),
        TextPosition::LowerThird => ("60", "h-text_h-50"),
    };

    let col_str = match overlay.style {
        crate::core::text_overlay::TextStylePreset::GoldElegance => "gold",
        crate::core::text_overlay::TextStylePreset::SunsetGlow => "coral",
        _ => "white",
    };

    let box_str = if overlay.show_box {
        ":box=1:boxcolor=black@0.65:boxborderw=14"
    } else {
        ":shadowcolor=black@0.8:shadowx=2:shadowy=2"
    };

    out.push_str(&format!(
        ",drawtext=text='{}':fontsize={}:fontcolor={}{}:x={}:y={}",
        escaped_text, font_size, col_str, box_str, x_expr, y_expr
    ));

    if let Some(sub) = &overlay.subtitle {
        let escaped_sub = sub
            .replace('\\', "\\\\")
            .replace('\'', "'\\''")
            .replace(':', "\\:")
            .replace('%', "\\%");
        let sub_font_size = ((font_size as f32) * 0.65).max(12.0).round() as u32;
        let sub_y = match overlay.position {
            TextPosition::CenterTitle => "(h-text_h)/2+40",
            TextPosition::BottomBanner => "h-text_h-20",
            TextPosition::TopHeader => "110",
            TextPosition::LowerThird => "h-text_h-20",
        };
        out.push_str(&format!(
            ",drawtext=text='{}':fontsize={}:fontcolor=lightgray{}:x={}:y={}",
            escaped_sub, sub_font_size, box_str, x_expr, sub_y
        ));
    }

    out
}
