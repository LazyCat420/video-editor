use crate::core::envelope::{CurveType, VolumeEnvelope};
use crate::core::time::TimeCode;
use crate::media::peak_extractor::WaveformPeaks;
use crate::ui::theme::AppTheme;
use egui::{Color32, Pos2, Rect, Stroke, Vec2};

pub fn render_audio_envelope_graph(
    ui: &mut egui::Ui,
    rect: Rect,
    envelope: &mut VolumeEnvelope,
    peaks: Option<&WaveformPeaks>,
    clip_duration: TimeCode,
    zoom_pps: f32,
    clip_id: u64,
) {
    let painter = ui.painter_at(rect);

    // 1. Draw Waveform Peaks in background if available
    if let Some(peaks_data) = peaks {
        let points = (rect.width() / 2.0).max(10.0) as usize;
        let dt = clip_duration.as_secs_f64() / points as f64;
        let mid_y = rect.center().y;
        let half_h = (rect.height() * 0.45).max(4.0);

        for i in 0..points {
            let t = i as f64 * dt;
            let peak = peaks_data.get_peak_at_sec(t);
            let x = rect.min.x + (i as f32 * 2.0);

            let top_y = mid_y - (peak[1] * half_h);
            let bot_y = mid_y - (peak[0] * half_h);

            painter.line_segment(
                [Pos2::new(x, top_y), Pos2::new(x, bot_y)],
                Stroke::new(1.5, AppTheme::waveform_color()),
            );
        }
    }

    if !envelope.enabled {
        return;
    }

    // 2. Draw 0dB Unity Reference Line (at 50% height, since gain range is 0.0 to 2.0)
    let unity_y = rect.max.y - 0.5 * rect.height();
    painter.line_segment(
        [
            Pos2::new(rect.min.x, unity_y),
            Pos2::new(rect.max.x, unity_y),
        ],
        Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 40)),
    );

    // Ensure we have at least start and end boundary nodes
    if envelope.nodes.is_empty() {
        envelope.add_node(TimeCode::ZERO, 1.0, CurveType::Linear);
        envelope.add_node(clip_duration, 1.0, CurveType::Linear);
    }

    // Coordinate mapping helper closures
    let time_to_x = |time: TimeCode| -> f32 {
        rect.min.x + (time.as_secs_f64() as f32 * zoom_pps)
    };

    let gain_to_y = |gain: f32| -> f32 {
        let normalized = (gain / 2.0).clamp(0.0, 1.0); // 0.0 -> bottom, 1.0 -> middle, 2.0 -> top
        rect.max.y - (normalized * rect.height())
    };

    let x_to_time = |x: f32| -> TimeCode {
        let delta_px = (x - rect.min.x).max(0.0);
        let secs = (delta_px / zoom_pps) as f64;
        TimeCode::from_secs_f64(secs).min(clip_duration)
    };

    let y_to_gain = |y: f32| -> f32 {
        let normalized = ((rect.max.y - y) / rect.height()).clamp(0.0, 1.0);
        let mut gain = normalized * 2.0;
        // Snap to 1.0 (0dB) if close
        if (gain - 1.0).abs() < 0.06 {
            gain = 1.0;
        }
        gain
    };

    // 3. Draw Continuous Envelope Curve Line
    let mut curve_points = Vec::new();
    let num_segments = (rect.width() / 4.0).max(20.0) as usize;
    let seg_dt = clip_duration.as_secs_f64() / num_segments as f64;

    for i in 0..=num_segments {
        let t = TimeCode::from_secs_f64(i as f64 * seg_dt).min(clip_duration);
        let gain = envelope.eval_gain(t);
        let px = time_to_x(t).clamp(rect.min.x, rect.max.x);
        let py = gain_to_y(gain);
        curve_points.push(Pos2::new(px, py));
    }

    for window in curve_points.windows(2) {
        painter.line_segment(
            [window[0], window[1]],
            Stroke::new(2.0, AppTheme::envelope_line_color()),
        );
    }

    // 4. Interactive Node Handle Rendering & Dragging
    let mouse_pos = ui.input(|i| i.pointer.hover_pos());
    let mouse_down = ui.input(|i| i.pointer.primary_down());
    let mouse_clicked = ui.input(|i| i.pointer.primary_clicked());
    let secondary_clicked = ui.input(|i| i.pointer.secondary_clicked());

    let mut node_to_delete = None;
    let mut node_to_update = None;

    for node in &envelope.nodes {
        let node_pos = Pos2::new(time_to_x(node.time_offset), gain_to_y(node.gain));
        let node_rect = Rect::from_center_size(node_pos, Vec2::splat(16.0));
        let is_hovered = mouse_pos.map_or(false, |p| node_rect.contains(p));

        // Draw node circle
        let radius = if is_hovered { 6.5 } else { 4.5 };
        let fill = if is_hovered {
            AppTheme::node_hover_color()
        } else {
            AppTheme::node_color()
        };
        painter.circle(node_pos, radius, fill, Stroke::new(1.5, Color32::WHITE));

        if is_hovered {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grab);
            let db_str = format!("{:.1} dB ({:.0}%)", node.gain_to_db(), node.gain * 100.0);
            egui::show_tooltip(ui.ctx(), ui.layer_id(), egui::Id::new(node.id), |ui| {
                ui.label(format!("Node #{}: {}\nTime: {}", node.id, db_str, node.time_offset));
            });

            // Right-click to delete node (if more than 2 nodes remain)
            if secondary_clicked && envelope.nodes.len() > 2 {
                node_to_delete = Some(node.id);
            }
        }
    }

    // Handle Dragging
    let drag_id = ui.make_persistent_id(format!("node_drag_{}", clip_id));
    if let Some(pos) = mouse_pos {
        if rect.contains(pos) {
            // Check if user started dragging a node
            if mouse_down {
                for node in &envelope.nodes {
                    let node_pos = Pos2::new(time_to_x(node.time_offset), gain_to_y(node.gain));
                    let node_rect = Rect::from_center_size(node_pos, Vec2::splat(16.0));
                    if node_rect.contains(pos) {
                        ui.memory_mut(|m| m.data.insert_temp(drag_id, node.id));
                        break;
                    }
                }
            }

            // Double-click or Ctrl+Click on line to add a new keyframe node
            let ctrl_held = ui.input(|i| i.modifiers.command || i.modifiers.ctrl);
            let double_clicked = ui.input(|i| i.pointer.button_double_clicked(egui::PointerButton::Primary));

            if double_clicked || (mouse_clicked && ctrl_held) {
                let new_time = x_to_time(pos.x);
                let new_gain = y_to_gain(pos.y);
                envelope.add_node(new_time, new_gain, CurveType::Linear);
            }
        }
    }

    // Process active dragging
    if mouse_down {
        if let Some(active_node_id) = ui.memory(|m| m.data.get_temp::<u64>(drag_id)) {
            if let Some(pos) = mouse_pos {
                let new_time = x_to_time(pos.x);
                let new_gain = y_to_gain(pos.y);
                node_to_update = Some((active_node_id, new_time, new_gain));
            }
        }
    } else {
        ui.memory_mut(|m| m.data.remove::<u64>(drag_id));
    }

    // Apply updates
    if let Some((id, t, g)) = node_to_update {
        envelope.update_node(id, t, g);
    }
    if let Some(id) = node_to_delete {
        envelope.remove_node(id);
    }
}
