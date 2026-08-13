# Video Editor (Rust + FFmpeg)

A high-performance, lightweight non-linear video editor (NLE) built in **Rust** designed to run smoothly on low-spec hardware (older Dell PCs, dual/quad-core CPUs, integrated Intel/AMD graphics, and low RAM budgets).

## Features
- **Low-Spec Optimized:** Ultra-low RAM overhead (~35MB base), background 360p intraframe proxy generator for 60 FPS scrub playback without taxing older CPUs.
- **Multi-Track Timeline:** Drag-and-drop video and audio tracks, magnetic snapping, precision trimming, and instant playhead splitting.
- **Interactive Audio Envelope Node Line Graph:** Add, drag, and manipulate volume keyframe nodes on audio tracks with real-time gain curves and FFmpeg automation export.
- **Waveform Peak Engine:** Instant binary waveform rendering (`.peaks`) without scanning raw audio in memory.
- **Production FFmpeg Export:** Multi-track composition, volume curves, and high-quality rendering with hardware acceleration detection (`vaapi`, `qsv`, `nvenc`) and CPU `libx264` fallback.

## Prerequisites
- Rust & Cargo (`rustc >= 1.75`)
- FFmpeg (`ffmpeg` and `ffprobe` in PATH)
- On Linux / WSL: Standard X11/Wayland and ALSA dev packages (`libasound2-dev`, `libgl1-mesa-dev`).

## Building & Running
```bash
cargo run --release
```

## Running Tests
```bash
cargo test
```
