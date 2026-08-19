use std::path::PathBuf;

/// Find ffmpeg executable path (env var, beside exe, in cwd, in bin/, bundled, or in PATH).
pub fn find_ffmpeg_executable() -> PathBuf {
    // 1. Explicit Environment Variable
    if let Ok(env_path) = std::env::var("FFMPEG_PATH") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return p;
        }
    }

    // 2. Beside current application executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let next_to_exe = parent.join(if cfg!(target_os = "windows") { "ffmpeg.exe" } else { "ffmpeg" });
            if next_to_exe.exists() {
                return next_to_exe;
            }
            let bin_sub = parent.join("bin").join(if cfg!(target_os = "windows") { "ffmpeg.exe" } else { "ffmpeg" });
            if bin_sub.exists() {
                return bin_sub;
            }
            let ffmpeg_sub = parent.join("ffmpeg").join("bin").join(if cfg!(target_os = "windows") { "ffmpeg.exe" } else { "ffmpeg" });
            if ffmpeg_sub.exists() {
                return ffmpeg_sub;
            }
        }
    }

    // 3. Common relative directories
    let local_paths = [
        PathBuf::from(if cfg!(target_os = "windows") { "ffmpeg.exe" } else { "ffmpeg" }),
        PathBuf::from("bin").join(if cfg!(target_os = "windows") { "ffmpeg.exe" } else { "ffmpeg" }),
        PathBuf::from("ffmpeg").join("bin").join(if cfg!(target_os = "windows") { "ffmpeg.exe" } else { "ffmpeg" }),
        PathBuf::from("assets").join("bin").join(if cfg!(target_os = "windows") { "ffmpeg.exe" } else { "ffmpeg" }),
        PathBuf::from("C:/ffmpeg/bin/ffmpeg.exe"),
    ];

    for p in &local_paths {
        if p.exists() {
            return p.clone();
        }
    }

    // 4. Default to system PATH resolution
    if cfg!(target_os = "windows") {
        PathBuf::from("ffmpeg.exe")
    } else {
        PathBuf::from("ffmpeg")
    }
}

/// Find ffprobe executable path (env var, beside exe, in cwd, in bin/, bundled, or in PATH).
pub fn find_ffprobe_executable() -> PathBuf {
    // 1. Explicit Environment Variable
    if let Ok(env_path) = std::env::var("FFPROBE_PATH") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return p;
        }
    }

    // 2. Beside current application executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let next_to_exe = parent.join(if cfg!(target_os = "windows") { "ffprobe.exe" } else { "ffprobe" });
            if next_to_exe.exists() {
                return next_to_exe;
            }
            let bin_sub = parent.join("bin").join(if cfg!(target_os = "windows") { "ffprobe.exe" } else { "ffprobe" });
            if bin_sub.exists() {
                return bin_sub;
            }
            let ffprobe_sub = parent.join("ffmpeg").join("bin").join(if cfg!(target_os = "windows") { "ffprobe.exe" } else { "ffprobe" });
            if ffprobe_sub.exists() {
                return ffprobe_sub;
            }
        }
    }

    // 3. Common relative directories
    let local_paths = [
        PathBuf::from(if cfg!(target_os = "windows") { "ffprobe.exe" } else { "ffprobe" }),
        PathBuf::from("bin").join(if cfg!(target_os = "windows") { "ffprobe.exe" } else { "ffprobe" }),
        PathBuf::from("ffmpeg").join("bin").join(if cfg!(target_os = "windows") { "ffprobe.exe" } else { "ffprobe" }),
        PathBuf::from("assets").join("bin").join(if cfg!(target_os = "windows") { "ffprobe.exe" } else { "ffprobe" }),
        PathBuf::from("C:/ffmpeg/bin/ffprobe.exe"),
    ];

    for p in &local_paths {
        if p.exists() {
            return p.clone();
        }
    }

    // 4. Default to system PATH resolution
    if cfg!(target_os = "windows") {
        PathBuf::from("ffprobe.exe")
    } else {
        PathBuf::from("ffprobe")
    }
}
