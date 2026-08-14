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

    // 1. Collect unique media source paths
    let mut unique_sources: Vec<PathBuf> = Vec::new();
    let mut source_to_input_idx: HashMap<PathBuf, usize> = HashMap::new();

    for track in &timeline.tracks {
        for clip in &track.clips {
            if !source_to_input_idx.contains_key(&clip.source_path) {
                let idx = unique_sources.len();
                source_to_input_idx.insert(clip.source_path.clone(), idx);
                unique_sources.push(clip.source_path.clone());
            }
        }
    }

    if unique_sources.is_empty() {
        return Err("No media clips found on the timeline.".to_string());
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
    let mut video_meta: Vec<(f64, Option<Transition>)> = Vec::new();

    let mut clip_counter = 0;

    for track in &timeline.tracks {
        if track.is_muted {
            continue;
        }

        for clip in &track.clips {
            let input_idx = *source_to_input_idx
                .get(&clip.source_path)
                .ok_or_else(|| "Source index lookup error".to_string())?;

            let in_sec = clip.source_in.as_secs_f64();
            let out_sec = clip.source_out.as_secs_f64();
            let start_ms = (clip.timeline_start.as_secs_f64() * 1000.0).round() as i64;

            // Video processing
            if clip.has_video && track.kind == TrackKind::Video {
                let v_label = format!("v{}", clip_counter);
                let v_trim = format!(
                    "[{}:v]trim=start={:.3}:end={:.3},setpts=PTS-STARTPTS,scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2,setsar=1,fps={}[{}]",
                    input_idx,
                    in_sec,
                    out_sec,
                    config.width,
                    config.height,
                    config.width,
                    config.height,
                    config.fps,
                    v_label
                );
                filter_chains.push(v_trim);
                video_out_labels.push(v_label.clone());
                video_meta.push((clip.duration().as_secs_f64(), clip.transition));
            }

            // Audio processing
            if clip.has_audio {
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
        final_video_label = video_out_labels[0].clone();
    } else {
        // Chain the clips through ffmpeg xfade so the selected transition blends each clip
        // into the next one. Boundaries with no transition use a near-instant crossfade (a
        // quick cut).
        let n = video_out_labels.len();
        // Overlap feeding INTO clip i (i>=1): the transition duration attached to clip i.
        let overlaps: Vec<f64> = (0..n)
            .map(|i| {
                if i == 0 {
                    0.0
                } else {
                    video_meta[i].1.map(|t| t.duration_secs).unwrap_or(0.05)
                }
            })
            .collect();

        // Every clip except the last gets a cloned tail as long as the next transition, so
        // the previous picture is still showing during the blend.
        let mut ready: Vec<String> = Vec::with_capacity(n);
        for i in 0..n {
            if i + 1 < n {
                let out_lbl = format!("vx_{}", i);
                filter_chains.push(format!(
                    "[{}]tpad=stop_mode=clone:stop_duration={:.3}[{}]",
                    video_out_labels[i],
                    overlaps[i + 1],
                    out_lbl
                ));
                ready.push(out_lbl);
            } else {
                ready.push(video_out_labels[i].clone());
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
            let kind = video_meta[i]
                .1
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
