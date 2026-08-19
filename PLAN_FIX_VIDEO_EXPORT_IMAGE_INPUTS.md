# Fix: Video Export Fails / Hangs Whenever the Timeline Contains an Image

**Repository:** `video-editor` (`LazyCat420/video-editor`)
**Status:** Implemented and verified — branch `fix/export-loop-arg-order`.

---

## 1. Problem Statement

Exporting to MP4 ("Export Slideshow & Video" → Start Video Render) failed for any project
containing a still image:

```
❌ Render Failed: FFmpeg failed (exit code Some(-2)):
[in#0 @ ...] Error opening input: No such file or directory
Error opening input file -loop.
Error opening input files: No such file or directory
```

Video-only timelines exported fine, so the failure tracked exactly with "is there a picture on
this slide" — i.e. the whole slideshow use case.

## 2. Root Cause — two defects stacked behind one symptom

### Defect A: `-loop` emitted *after* `-i` (the visible failure)

`src/export/filter_graph.rs`, image-input branch, pushed args in this order:

```rust
args.push("-i".to_string());
if is_image {
    args.push("-loop".to_string());
    args.push("1".to_string());
}
args.push(src...);
```

which produces `-i -loop 1 photo.png`. `-loop` is an **input option** and must precede its
`-i`. As written, FFmpeg consumed the literal string `-loop` as the input *filename* → ENOENT
→ exit `-2`. Evidence, the command actually built (captured by the new test on the unfixed
code):

```
["-y", "-i", "-loop", "1", "C:\\...\\photo.png", "-filter_complex", ...]
```

### Defect B: an unbounded looped input never terminates (found only by running it)

Fixing the order alone was **not** enough. `-loop 1` with no `-t` is an infinite input. The
initial fix was committed on the reasoning that "overlay ends with its main input, so the
still needs no bound." That reasoning is wrong, and running the render disproved it: the
export produced a **70 MB `out.mp4` and was still growing after 38 minutes** for what should
be a 2-second 320x240 clip. It stalled the test suite until killed.

Direct FFmpeg probes (FFmpeg 8.0, the version in the failure report) on the exact filtergraph
the exporter generates:

| Variant | Result |
|---|---|
| `-loop 1 -i img` (no bound) | **never terminates** — killed at 45 s, still encoding |
| `-loop 1 -i img` + `-shortest` | **never terminates** — killed at 48 s |
| `-loop 1 -t 2 -i img` | finishes instantly, 2.000 s, 50 frames |
| plain `-i img` (no loop) | finishes instantly, 2.000 s, 50 frames |

## 3. Why `-loop` could not simply be dropped

The no-loop variant looked equally good on duration, so the two were separated by a frame
count instead. An image dropped straight onto a video track becomes a clip with
`has_video = true` (`src/media/probe.rs:142-163` sets it from ffprobe's `codec_type`, and
stills report `codec_type=video`), so it is the **base layer** and is cut with
`trim=start=..:end=source_out`. Rendering that path both ways:

| Base-layer variant | Container duration | **Actual frames** |
|---|---|---|
| `-loop 1 -t 2 -i img` | 2.000 s | **50** |
| plain `-i img` (no loop) | 2.000 s | **1** |

The no-loop render reports the full duration and contains **one frame**. Duration is a metric
that cannot fail on this defect; only a frame count catches it. Dropping `-loop` would have
silently collapsed every image clip to a single frame while every duration assertion stayed
green.

## 4. The Fix

`src/export/filter_graph.rs`:

1. Emit input options before the input: `-loop 1 -t <dur> -i <path>`.
2. New helper `image_input_duration(timeline, src)` computes `<dur>` as the timeline duration
   floored against the largest `source_out` among the clips referencing that source. The
   `source_out` term matters because inputs are de-duplicated by path (`collect_sources`), and
   a base-layer clip trimmed to e.g. `start=3:end=5` needs input frames out to 5 s even
   though the clip itself is only 2 s long.

## 5. What This Proved / Regression Tests

`tests/timeline_tests.rs`, two new tests plus three helpers:

- `test_image_input_loop_precedes_i_and_terminates` — slide with a picture background *and* a
  picture element. Asserts the exact `-loop 1 -t <positive> -i <path>` arg shape, asserts no
  `-i` is ever followed by `-loop`, then renders end to end and requires > 25 frames.
- `test_image_base_layer_clip_renders_every_frame` — an image dropped on a video track
  (`has_video = true`). Requires > 50 frames for a 3 s clip at 25 fps; this is the test that
  would catch the single-frame collapse described in §3.
- `run_export_with_deadline` — spawns FFmpeg and polls `try_wait` against a 120 s deadline,
  killing it and failing with a diagnostic. Without this a regression to Defect B hangs the
  suite indefinitely rather than failing it (which is exactly what happened during this work).
- `rendered_frame_count` — ffprobe `-count_frames`, used instead of the container duration
  for the reason in §3.

**Sabotage check.** Removing the `-t` push from the fixed code makes
`test_image_input_loop_precedes_i_and_terminates` fail on the arg-shape assertion. Note that
`test_image_base_layer_clip_renders_every_frame` **passes** under that same sabotage — a
base-layer graph is bounded by its own `trim=end`, so only the overlay path can run away. The
hang guard belongs to the overlay test; the base-layer test guards the frame collapse. Neither
test covers both defects, and the pair is not interchangeable.

**Pre-fix check.** Both tests were run against the unfixed code first and failed there, so
they are pinned to the real defect rather than to the new implementation.

## 6. Open Items (not fixed here)

- **The UI's "Video Bitrate (kbps)" field is ignored for libx264.** The encoder block emits
  `-crf 20` and only applies `-b:v` on the VAAPI/QSV/NVENC paths, so changing the bitrate box
  in the export dialog has no effect on the default encoder. Either honour it or remove the
  control.
- **A duplicate `-y`** is emitted (once at the head of the args, once before `-progress`).
  Harmless, but it indicates the arg builder has two places that think they own output flags.
- **`drop_files_on_canvas` (`src/app/canvas_ops.rs:186-193`)** turns a real dropped `.png`
  into `SlideElement::Video` rather than `Picture`, because it branches on the same
  `has_video` probe result. The existing tests in `tests/slide_dnd_tests.rs` do not catch this
  — their fake paths make ffprobe fail, so `has_video` falls back to `false` and the tests
  never exercise the real branch.

## 7. Manual Verification

1. `cargo test` — full suite.
2. `cargo run --release`, build a slide with a picture, export to MP4, confirm the render
   completes and the image is visible for the whole slide.
