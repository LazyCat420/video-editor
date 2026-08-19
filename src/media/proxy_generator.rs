use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::watch;

#[derive(Clone, Debug, PartialEq)]
pub enum ProxyStatus {
    Pending,
    Generating { progress_pct: f32 },
    Ready { proxy_path: PathBuf },
    Failed { error: String },
}

/// Spawns a background task to generate a fast 360p intraframe proxy video for smooth playback on low-end CPUs.
pub fn generate_proxy_async<P: AsRef<Path>>(
    source_path: P,
    total_duration_secs: f64,
) -> watch::Receiver<ProxyStatus> {
    let (tx, rx) = watch::channel(ProxyStatus::Pending);
    let src = source_path.as_ref().to_path_buf();

    tokio::spawn(async move {
        let _ = tx.send(ProxyStatus::Generating { progress_pct: 0.0 });

        let proxy_dir = std::env::temp_dir().join("video_editor_proxies");
        let _ = tokio::fs::create_dir_all(&proxy_dir).await;

        let file_stem = src
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("media");
        let proxy_path = proxy_dir.join(format!("{}_proxy_360p.mp4", file_stem));

        // Check if cached proxy already exists and is non-empty
        if let Ok(metadata) = tokio::fs::metadata(&proxy_path).await {
            if metadata.len() > 1000 {
                let _ = tx.send(ProxyStatus::Ready {
                    proxy_path: proxy_path.clone(),
                });
                return;
            }
        }

        let ffmpeg_bin = crate::media::find_ffmpeg_executable();
        let mut child = match Command::new(&ffmpeg_bin)
            .args([
                "-y",
                "-i",
                src.to_str().unwrap_or_default(),
                "-vf",
                "scale=-2:360",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-tune",
                "fastdecode",
                "-g",
                "1", // All keyframes (intra-only) for instant seeking
                "-an",
                "-progress",
                "pipe:1",
                proxy_path.to_str().unwrap_or_default(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(ProxyStatus::Failed {
                    error: format!("Failed to spawn ffmpeg for proxy: {}", e),
                });
                return;
            }
        };

        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if line.starts_with("out_time_us=") {
                    let us_str = line.trim_start_matches("out_time_us=");
                    if let Ok(us) = us_str.parse::<f64>() {
                        let cur_secs = us / 1_000_000.0;
                        if total_duration_secs > 0.0 {
                            let pct = ((cur_secs / total_duration_secs) * 100.0).clamp(0.0, 99.0)
                                as f32;
                            let _ = tx.send(ProxyStatus::Generating { progress_pct: pct });
                        }
                    }
                }
            }
        }

        match child.wait().await {
            Ok(status) if status.success() => {
                let _ = tx.send(ProxyStatus::Ready { proxy_path });
            }
            Ok(status) => {
                let _ = tx.send(ProxyStatus::Failed {
                    error: format!("ffmpeg exited with code {}", status),
                });
            }
            Err(e) => {
                let _ = tx.send(ProxyStatus::Failed {
                    error: format!("ffmpeg process error: {}", e),
                });
            }
        }
    });

    rx
}
