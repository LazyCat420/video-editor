use std::path::PathBuf;
use video_editor::core::clip::Clip;
use video_editor::core::envelope::CurveType;
use video_editor::core::time::TimeCode;
use video_editor::core::timeline::Timeline;
use video_editor::export::filter_graph::{build_ffmpeg_export_command, EncoderType, ExportConfig};

#[test]
fn test_ffmpeg_export_command_generation() {
    let mut timeline = Timeline::new(30.0);
    let v_track_id = timeline.tracks[0].id;

    // Add video clip
    let mut v_clip = Clip::new(
        1,
        v_track_id,
        "Video 1".to_string(),
        PathBuf::from("/media/video1.mp4"),
        TimeCode::from_secs_f64(10.0),
        true,
        true,
    );
    v_clip.source_in = TimeCode::from_secs_f64(1.0);
    v_clip.source_out = TimeCode::from_secs_f64(6.0);
    v_clip.timeline_start = TimeCode::ZERO;

    // Add volume envelope keyframes to audio
    v_clip.volume_envelope.add_node(TimeCode::ZERO, 1.0, CurveType::Linear);
    v_clip.volume_envelope.add_node(TimeCode::from_secs_f64(2.0), 0.3, CurveType::Linear); // Duck volume
    v_clip.volume_envelope.add_node(TimeCode::from_secs_f64(5.0), 1.0, CurveType::Linear); // Fade back in

    if let Some(t) = timeline.get_track_mut(v_track_id) {
        t.add_clip(v_clip);
    }

    let config = ExportConfig {
        output_path: PathBuf::from("/media/final_export.mp4"),
        width: 1920,
        height: 1080,
        fps: 30.0,
        video_bitrate_kbps: 6000,
        audio_bitrate_kbps: 192,
        encoder: EncoderType::Libx264,
    };

    let args_res = build_ffmpeg_export_command(&timeline, &config);
    assert!(args_res.is_ok());

    let args = args_res.unwrap();
    let full_cmd = args.join(" ");

    assert!(full_cmd.contains("-i /media/video1.mp4"));
    assert!(full_cmd.contains("-filter_complex"));
    assert!(full_cmd.contains("trim=start=1.000:end=6.000"));
    assert!(full_cmd.contains("scale=1920:1080"));
    assert!(full_cmd.contains("volume=eval=frame:volume="));
    assert!(full_cmd.contains("-c:v libx264"));
    assert!(full_cmd.contains("-c:a aac"));
    assert!(full_cmd.contains("/media/final_export.mp4"));
}
