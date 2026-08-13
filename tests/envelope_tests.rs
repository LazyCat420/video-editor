use video_editor::core::envelope::{CurveType, VolumeEnvelope, VolumeNode};
use video_editor::core::time::TimeCode;

#[test]
fn test_volume_node_db_conversion() {
    let node_unity = VolumeNode::new(1, TimeCode::ZERO, 1.0, CurveType::Linear);
    assert!((node_unity.gain_to_db() - 0.0).abs() < 0.01);

    let node_half = VolumeNode::new(2, TimeCode::ZERO, 0.5, CurveType::Linear);
    assert!((node_half.gain_to_db() - (-6.02)).abs() < 0.05);

    let node_silent = VolumeNode::new(3, TimeCode::ZERO, 0.0, CurveType::Linear);
    assert_eq!(node_silent.gain_to_db(), -60.0);

    let node_from_db = VolumeNode::from_db(4, TimeCode::ZERO, -6.0, CurveType::Linear);
    assert!((node_from_db.gain - 0.501).abs() < 0.01);
}

#[test]
fn test_envelope_linear_interpolation() {
    let mut env = VolumeEnvelope::new();
    let t0 = TimeCode::ZERO;
    let t1 = TimeCode::from_secs_f64(2.0);
    let t2 = TimeCode::from_secs_f64(4.0);

    env.add_node(t0, 1.0, CurveType::Linear);
    env.add_node(t1, 0.0, CurveType::Linear);
    env.add_node(t2, 2.0, CurveType::Linear);

    // At t0: gain = 1.0
    assert!((env.eval_gain(t0) - 1.0).abs() < 0.001);

    // At 1.0s (midpoint of [0..2]): gain = 0.5
    let t_mid1 = TimeCode::from_secs_f64(1.0);
    assert!((env.eval_gain(t_mid1) - 0.5).abs() < 0.001);

    // At 2.0s: gain = 0.0
    assert!((env.eval_gain(t1) - 0.0).abs() < 0.001);

    // At 3.0s (midpoint of [2..4]): gain = 1.0
    let t_mid2 = TimeCode::from_secs_f64(3.0);
    assert!((env.eval_gain(t_mid2) - 1.0).abs() < 0.001);

    // At 4.0s: gain = 2.0
    assert!((env.eval_gain(t2) - 2.0).abs() < 0.001);
}

#[test]
fn test_envelope_smooth_bezier_interpolation() {
    let mut env = VolumeEnvelope::new();
    let t0 = TimeCode::ZERO;
    let t1 = TimeCode::from_secs_f64(2.0);

    env.add_node(t0, 0.0, CurveType::SmoothBezier);
    env.add_node(t1, 1.0, CurveType::SmoothBezier);

    // Midpoint of smoothstep S-curve is exactly 0.5
    let t_mid = TimeCode::from_secs_f64(1.0);
    assert!((env.eval_gain(t_mid) - 0.5).abs() < 0.001);

    // At 25% time, smoothstep has slower acceleration than linear (t*t*(3-2t)) = 0.25*0.25*2.5 = 0.15625
    let t_quarter = TimeCode::from_secs_f64(0.5);
    assert!((env.eval_gain(t_quarter) - 0.15625).abs() < 0.01);
}

#[test]
fn test_envelope_ffmpeg_expression_generation() {
    let mut env = VolumeEnvelope::new();
    env.add_node(TimeCode::from_secs_f64(0.0), 1.0, CurveType::Linear);
    env.add_node(TimeCode::from_secs_f64(2.0), 0.2, CurveType::Linear);
    env.add_node(TimeCode::from_secs_f64(4.0), 1.0, CurveType::Linear);

    let expr = env.to_ffmpeg_volume_expression();
    assert!(expr.is_some());
    let expr_str = expr.unwrap();
    assert!(expr_str.starts_with("volume=eval=frame:volume='"));
    assert!(expr_str.contains("if(lte(t,0.000),1.0000"));
    assert!(expr_str.contains("between(t,0.000,2.000)"));
    assert!(expr_str.contains("between(t,2.000,4.000)"));
}
