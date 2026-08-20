# Grandma-Friendly Music: Open Files Fix, Music Row, Real Preview Playback

**Repository:** `video-editor` (`LazyCat420/video-editor`)
**Status:** Implemented on `feat/grandma-music`; automated tests green; on-Windows player test list below is UNVERIFIED.

---

## 1. The Reported Problem

An elderly user clicked **📂 Open Files** (top-left) to add music to her slideshow and
"nothing happened."

### Root cause — the import succeeded, invisibly

`📂 Open Files` *is* the right button (Open Project is a separate item under 📁 Project, and
mp3 is fully supported by the picker's filters). The handler imported the file into
`project.media_assets` — whose only viewer, `MediaBinView` (`src/ui/media_bin.rs`), is **never
rendered anywhere in the app**. No call site exists. So every import landed in an invisible
bin: no slide, no music, no error. Probe failures were likewise swallowed
(`probe_media_file(p).ok()?`), so even a broken file produced zero feedback.

The only path that worked was dragging a file from Windows Explorer onto the window, which
attached audio to the *current slide* as a `SlideElement::Audio` — a second, conflicting music
model, and one the user can't discover.

A further wrinkle: the app had **no audio engine at all** (no rodio/cpal anywhere). Music was
only ever audible in the exported file — so even a "successful" music add was silent in preview.

## 2. What Changed

Decisions made with the project owner: photos/videos from Open Files land on the *current
slide* (matching the existing drag-drop layout logic); music lands directly on the music track
**appended after the last song so songs can never overlap** (no bin, no manual positioning,
mirroring what she expects from other apps but without the overlap failure mode); the Timeline
Editor stays behind its toggle with Slideshow Studio as default; preview gets real playback.

### a. One import router (`src/app/music_ops.rs`)
`import_files` partitions by extension (`is_audio_path`, `src/media/probe.rs` — extension-based
on purpose: ffprobe reports `codec_type=video` for still images, so the probe cannot be the
routing authority). Audio → `Timeline::append_music_clip`; photos/videos → the existing
`drop_files_on_canvas` slide placement. The same partition sits inside `drop_files_on_canvas`
itself, so OS drops and filmstrip-card drops route identically. `SlideElement::Audio` is no
longer *created* anywhere, but existing projects containing one still render and export.

### b. Music model (`src/core/timeline.rs`)
`append_music_clip` starts the new clip at `track.duration()` (end of last song);
`remove_music_clip` repacks remaining songs contiguously from t=0 (`reflow_music_positions`,
the audio-track mirror of `reflow_slide_timeline_positions`). Overlap is structurally
impossible from the UI.

### c. Visible errors (`status_toast`)
A high-contrast bottom-center toast (~6 s). Probe failures name the file; opening a non-project
file via Open Project now explains itself instead of silently doing nothing.

### d. Music row UI (`src/ui/music_row.rs`)
Under the slide filmstrip in Slideshow Studio: **🎵 Add Music** (audio-only picker), one chip
per song (name, m:ss, 🗑 remove-with-reflow), a whole-track volume slider (no undo snapshots —
live mixing control), and a "Music 4:10 · Slides 2:30 (music is longer)" readout. Export
already honored `Track::volume`/`is_muted`, so no export changes were needed.

### e. Real preview playback (`src/audio/music_engine.rs`)
rodio 0.20 (`symphonia` mp3/wav/flac/vorbis/aac/isomp4). The wall-clock playhead is the master
clock; `MusicEngine::sync` runs each playing frame: starts the song under the playhead at
offset `(playhead − clip_start) + source_in` (via `skip_duration` + `take_duration`), switches
songs at clip boundaries (boundary rule: `contains_timeline_time` is end-inclusive, so the
engine additionally requires `playhead < timeline_end()` to prefer the *next* song), corrects
drift > 300 ms, applies volume/mute live. Seek/undo/redo/project-switch invalidate the sink.
Device open is lazy with a one-shot failure latch, so device-less machines get one toast and
otherwise run normally. Feature-gated as `audio-playback` (default on) with an inert same-API
stub — and the cross-link concern proved unfounded: cpal/WASAPI links clean on
`x86_64-pc-windows-gnullvm`.

**Out of scope, stated plainly:** video-embedded audio is still silent in preview (needs an
ffmpeg decode path); `.wma/.opus/.aiff` chips toast "will still be in the exported video."

## 3. What the Tests Pin

`tests/music_track_tests.rs` (new): second song starts exactly at the first song's end; append
recreates a deleted music track; removing the middle of three songs repacks from t=0; removing
a nonexistent id is a no-op; routing is extension-based (`Song.Mp3` yes, `.mp4/.png/no-ext`
no); `music_source_offset` math incl. clamp-before-clip-start.

`tests/slide_dnd_tests.rs`: the old test asserting mp3→slide-element pinned the *bug's* model
and now asserts the new one — a real generated mp3 dropped on the canvas lands on the music
track and NOT on the slide, and a missing mp3 adds nothing but sets a visible error toast.

Not machine-verifiable here: actual sound. WSL has no audio device and `cargo test` runs the
Windows exes headless, so everything sink-side is covered only by the player test list below.

### f. Import no longer freezes the UI (follow-up, same branch family)
Reported: "when I open files it kind of lags." Cause: every picked/dropped file was probed by
a synchronous ffprobe process spawn on the UI thread. Now `queue_unprobed_files` sends unknown
files to a worker thread, a progress bar ("Adding your files… X of N") renders while the batch
probes, and `pump_import_queue` replays the original drop with a hot `probe_cache` when done —
batch layout math unchanged, zero ffprobe on the UI thread. Missing files fail instantly and
synchronously (no worker), the `has_video` fallback reads the cache instead of re-probing a
known-bad file, and the cache is cleared per consumed batch. Pinned by
`add_media_to_bin_uses_probe_cache_without_ffprobe` (a nonexistent path succeeds only if the
cache was honored) and `import_queue_probes_in_background_then_applies_batch` (queues, applies
after pumping, drains the cache).

## 4. Open Items

- **Preview/export font mismatch** (pre-existing): exported `drawtext` uses system font family
  names while the UI uses bundled TTFs, so exported text can differ from preview on machines
  missing those fonts.
- **`.alac` canvas-drop** routes as audio now (list unified), but rodio's alac feature was not
  enabled; it toasts as preview-unsupported.
- The media bin (`MediaBinView`) remains dead code; the peak-extraction thread it fed still
  runs on import and still discards its peaks (`let _ = peaks;`). Candidate for deletion.
- Nothing persists across launches (no `eframe::App::save`); a future "simple mode" flag would
  need that or a `Project` field.

## 5. What YOU should test (player test list) — UNVERIFIED until run on Windows

1. **📂 Open Files → one .mp3** → a chip appears in the music row instantly; Play → the song is
   audible from the start; the volume slider changes loudness live. Bug: nothing visible
   happens (the old behavior), or the chip appears but Play is silent.
2. **Second song** → chips sit in order; playback crosses the boundary with no overlap, gap, or
   double audio.
3. **Open Files with .jpg + .mp4 + .mp3 together** → photo and video land on the current slide,
   mp3 in the music row.
4. **Drag an .mp3 from Explorer** onto the preview or a filmstrip card → music row, never the
   slide.
5. **Pause/resume** stays in place; **Stop → Play** restarts song 1 at 0:00; **seeking** into
   the middle of song 2 plays from the right spot within a beat.
6. **🗑 the first of three songs** → the rest shift up (no silence hole); Ctrl+Z brings it back
   in position.
7. **A .txt renamed .mp3** → red message naming the file; app keeps running; nothing added.
8. **A .wma** → chip appears; Play toasts "will still be in the exported video"; the exported
   mp4 contains its audio.
9. **Old project with slide-attached audio** still loads, shows its 🎵 badge, and exports with
   sound.
10. **Timeline Editor toggle** → music clips visible on "🎵 Music & Sound" at their packed
    positions; mute there silences the preview music.
11. **Big batch import**: Open Files with 10+ photos → the window stays responsive, a
    progress bar counts up, then everything appears at once. Bug: the window greys out /
    "Not Responding".
12. **Music longer than slides** → the row reads "(music is longer)"; export runs to the music's
    end (pre-existing `Timeline::duration()` behavior).
