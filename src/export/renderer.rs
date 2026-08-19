use crate::export::filter_graph::{build_ffmpeg_export_command, ExportConfig};
use crate::core::timeline::Timeline;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::watch;

#[derive(Clone, Debug, PartialEq)]
pub enum RenderProgress {
    Idle,
    Rendering {
        progress_pct: f32,
        current_time_secs: f64,
        fps: f32,
    },
    Completed {
        output_path: std::path::PathBuf,
    },
    Failed {
        error: String,
    },
}

/// Spawns the async FFmpeg render export pipeline.
pub fn render_project_async(
    timeline: Timeline,
    config: ExportConfig,
) -> watch::Receiver<RenderProgress> {
    let (tx, rx) = watch::channel(RenderProgress::Idle);

    tokio::spawn(async move {
        let total_duration_secs = timeline.duration().as_secs_f64();
        if total_duration_secs <= 0.0 {
            let _ = tx.send(RenderProgress::Failed {
                error: "Timeline is empty".to_string(),
            });
            return;
        }

        let args = match build_ffmpeg_export_command(&timeline, &config) {
            Ok(a) => a,
            Err(e) => {
                let _ = tx.send(RenderProgress::Failed { error: e });
                return;
            }
        };

        let _ = tx.send(RenderProgress::Rendering {
            progress_pct: 0.0,
            current_time_secs: 0.0,
            fps: 0.0,
        });

        let ffmpeg_bin = crate::media::frame_cache::find_ffmpeg_executable();
    let mut child = match Command::new(&ffmpeg_bin)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(RenderProgress::Failed {
                    error: format!("Failed to spawn ffmpeg: {}", e),
                });
                return;
            }
        };

        let mut cur_time_secs = 0.0f64;
        let mut cur_fps = 0.0f32;
        let _ = cur_time_secs;

        let stderr_lines = std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::VecDeque::with_capacity(50)));
        let stderr_lines_clone = stderr_lines.clone();

        let stderr_handle = if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let mut lock = stderr_lines_clone.lock().await;
                    if lock.len() >= 50 {
                        lock.pop_front();
                    }
                    lock.push_back(line);
                }
            })
        } else {
            tokio::spawn(async {})
        };

        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if line.starts_with("out_time_us=") {
                    let us_str = line.trim_start_matches("out_time_us=");
                    if let Ok(us) = us_str.parse::<f64>() {
                        cur_time_secs = us / 1_000_000.0;
                        let pct = ((cur_time_secs / total_duration_secs) * 100.0).clamp(0.0, 99.0)
                            as f32;
                        let _ = tx.send(RenderProgress::Rendering {
                            progress_pct: pct,
                            current_time_secs: cur_time_secs,
                            fps: cur_fps,
                        });
                    }
                } else if line.starts_with("fps=") {
                    let fps_str = line.trim_start_matches("fps=");
                    if let Ok(fps) = fps_str.trim().parse::<f32>() {
                        cur_fps = fps;
                    }
                }
            }
        }

        let _ = stderr_handle.await;

        match child.wait().await {
            Ok(status) if status.success() => {
                let _ = tx.send(RenderProgress::Completed {
                    output_path: config.output_path,
                });
            }
            Ok(status) => {
                let lock = stderr_lines.lock().await;
                let error_detail = lock.iter().cloned().collect::<Vec<_>>().join("\n");
                let error_msg = if error_detail.trim().is_empty() {
                    format!("FFmpeg export process exited with code: {:?}", status.code())
                } else {
                    format!("FFmpeg failed (exit code {:?}):\n{}", status.code(), error_detail)
                };
                let _ = tx.send(RenderProgress::Failed { error: error_msg });
            }
            Err(e) => {
                let _ = tx.send(RenderProgress::Failed {
                    error: format!("FFmpeg execution error: {}", e),
                });
            }
        }
    });

    rx
}
