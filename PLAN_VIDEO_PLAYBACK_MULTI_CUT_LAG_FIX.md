# Audit & Implementation Plan — Video Choppy as Cut Count Grows

**Mode:** audit (read-only) + plan. Four read-only scout subagents audited the
playback stack in parallel (`MediaDecode`, `PlaybackOrch`, `Render`, `Audio`).
No code changed, no build, no deploy.

**Symptom:** split a clip into many sub-clips on the timeline, press play → the
more cuts you make, the choppier it plays. Freezes / stutters at each cut.

---

## 1. Confirmed root cause (all 4 subagents converge)

**The preserved "continuous" stream is still hard-capped by the FIRST clip's
`-t <duration>`, and that cap is never renewed across later adjacent cuts.**

The playback picks one clip and starts an `ffmpeg` rawvideo stream with
`-t <rem_dur>` (the *first* clip's remaining duration) — `stream_player.rs:130-135`,
driven by `rem_dur = (timeline_end - time).max(0.1)` in `app.rs:262`.

When the playhead crosses an **adjacent same-source cut**, the fast path
`switch_to_clip` → `is_continuous_with` (`stream_player.rs:311-330, 49-61`)
preserves the running stream (same path + `last_pts` within 1 s). It does **not**
restart or extend the process, **and it does not renew the `-t` budget.**

So the single shared `ffmpeg` hits **EOF at the first clip's source end**. Its
reader thread stops producing and flips `is_running = false`, but the deck was
already switched to it as "continuous" — so `get_frame_for_time` sits on the
last buffered frame (the latch) and **freezes** until the *next* boundary or
lookahead detects non-continuity and force-cold-restarts (new spawn, zero
pre-roll stall).

Scale: 1 cut → frozen 2nd clip; **N cuts → N−1 frozen boundaries + N−1 cold
restarts.** That is exactly "lags more the more cuts I make."

### Why the guard doesn't save us
`is_continuous_with` checks only `is_running && same path && |source_time −
last_pts| < 1.0` — it never accounts for the `-t` end-of-stream budget, so at
the instant of the cut the dying stream still *looks* continuous and is kept.

---

## 2. Secondary findings (same direction)

| # | Finding | Evidence | Severity |
|---|---|---|---|
| 2 | **Open-loop wall-clock presentation.** `AudioPlayer` is a pure software
  clock (`Instant::now`), the ONLY playback clock, and it advances every UI
  frame **decoupled from whether a frame was actually delivered**. Any deck-
  switch stall marches the clock, so the pulled frame goes stale then jumps to
  catch up. Repeats at every cut. | `audio/player.rs`; `app.rs:350` | High |
| 3 | **Gap / removed segment → both decks stopped + cold restart with zero
  pre-roll** → a freeze spike per removed section. | `switch_to_clip` fallback | Med |
| 4 | **Overlapping ffmpeg on cold cuts**: old child is killed on a background
  thread (not dead before the new spawn), and the reader's shared `is_running`/`buffer`
  Arcs can briefly run the old reader alongside the new one → double CPU/IO. | `stream_player.rs:96-158, 69-72` | Med |
| 5 | **~691 KB `ColorImage` cloned on the UI thread every repaint** via
  `get_frame_for_time`'s latch (~40 MB/s churn), plus a per-frame allocation in
  the reader. | `stream_player.rs:216-234` | Med |
| 6 | **Per-tick O(n) scans** grow linearly with clip count: `timeline.duration()`
  + `get_active_video_clip_info` ×2 each tick, each doing an O(clips)
  `clips.iter().find`. | `timeline.rs`, `track.rs`, `app.rs:369-375` | Low |
| 7 | Timeline/node views re-render **every** clip + waveform + envelope each
  repaint (immediate mode) — O(cuts)/frame UI cost. | `timeline_view.rs`, `node_graph_view.rs` | Low |

Findings 2–7 *amplify* the #1 bug but are not its source. Fix #1 first.

---

## 3. Web-research guidance (Rust/FFmpeg/egui playback)

- **Pull-model preview = bounded buffer + backpressure is correct** (already
  present: 30-frame `VecDeque`, 15 ms backpressure sleep). Keep it.
- **Don't let the OS/cli re-limit a stream you intend to preserve.** The `-t`
  cap is the defect: a long-lived decode pipe should be duration-bounded **in
  the consumer**, not by the producer's `-t`. (ffmpeg CLI pipe pattern.)
- **One texture handle, reuse, dirty-check** (egui best practice, already
  present in `preview_player.rs`) — keep, only trim the redundant clone.
- **Master clock slaved to delivered frames** (A/V-sync principle): present at
  the rate frames arrive, not the rate a wall-clock ticks. This is the fix for
  finding 2.

---

## 4. Fix plan (dependency-ordered)

### Step 1 — Eliminate the `-t` EOF on the continuous-preserve path (primary)

Two options; **1A is recommended.**

- **1A. Drop `-t` for same-file continuous playback.** When a deck streams a
  source file, do not pass `-t`; decode to end-of-file (bounded by the 30-frame
  backpressure). Gate in the consumer: `get_frame_for_time` already pops by
  `pts <= source_time`, so adjacent cuts just continue — **zero restarts, zero
  freezes, no process spawn per cut.** Stop the deck explicitly when the
  playhead leaves the last continuous clip (gap) or the source changes. Pre-
  decoding ahead of a 360p file is cheap and bounded.

  Caveat to handle: a later, non-adjacent use of the same source (a removed
  middle) must re-seek — the **prewarm / dual-deck swap already covers that
  (cold/jump path)**; this change only affects the preserved branch.

- **1B (fallback).** Keep `-t` but, on hitting a continuous cut, re-seek the
  deck to the new `start + rem_dur`. Preserves one-process-at-a-time but costs a
  spawn per cut (re-introduces latency). Inferior to 1A for the common case.

**Acceptance:** slice a clip into ≥15 adjacent pieces; play start-to-finish;
observe **zero freezes** at boundaries; `child_process` spawned exactly once per
continuous run.

### Step 2 — Slave the presentation clock to delivered frames

Make `get_frame_for_time` report whether a **new** frame was popped (return an
`Option<(pts, Arc<ColorImage>)>` or a flag), and only advance the playhead when
a new frame was delivered; otherwise hold. This is true frame-lock, removes the
catch-up jump at every cut, and is correct for a non-audio preview. If audio is
later added, slave to the audio clock instead.

**Acceptance:** pausing the deck (or a decode hiccup) stops the playhead too —
no open-loop drift, no jump.

### Step 3 — Serialize ffmpeg lifecycle (no overlapping children)

Before `Command::spawn` in `start()`, ensure the prior child of the SAME deck
is reaped (or track with a generation counter so a stale reader can't feed the
new buffer). Keep termination off the UI thread, but don't let two children
decode concurrently on a cold cut.

### Step 4 — Kill the redundant UI-thread frame clone

Store the latched frame as `Arc<ColorImage>` in the deck; `get_frame_for_time`
returns the `Arc` (cheap `clone` of the pointer) and only `texture.set(...)`
(copies) when it actually advanced / is dirty — which the existing
dirty-check already gates. Removes ~40 MB/s of UI-thread `Vec` copy.

### Step 5 — De-linearize per-tick work (low priority, easy)

- Cache `timeline.duration()` (recompute only on edit), and replace
  `get_clip_at`'s O(n) scan with a binary search over the sorted `clips` (clips
  are kept sorted) or a cached active-clip index keyed by playhead.
- In `update()`, avoid calling `get_active_video_clip_info` twice; reuse one
  result.

---

## 5. Verification

Per workflow: **reproduce first.** Build release
(`cargo build --release`), split a clip into N pieces (try 5 / 15 / 30), play,
confirm freeze count scales with N. Then apply Step 1 (or 1A), rebuild,
reproduce the same scenario, confirm N freezes → 0.

Automated (add a `tests/` integration that is *dependent on* `StreamVideoPlayer`,
not standalone — per repo practice):
1. **Continuous-multi-cut test:** synthesize a short source; start a deck at
   t=0 with a sub-clip `-t`; simulate crossing subsequent adjacent sub-clips via
   `switch_to_clip` + `get_frame_for_time`; assert frames keep advancing with no
   EOF freeze and `child_process` not re-spawned.
2. **No-t behavior test:** with 1A, assert the deck produces frames past the
   first sub-clip's source end without restart.
3. **Frame-lock test:** with Step 2, assert the playhead does not advance when
   no new frame is delivered.

Run `cargo test`, then Windows `cargo build --release`, then manual play test.

---

## 6. Decisions requested

1. **1A vs 1B** — recommend **1A** (drop `-t` on continuous streams).
2. Scope: **Steps 1–2 are the fix** (kill the freeze + the jump). Steps 3–5 are
   polish worth doing, but not required to stop the lag. Cut line at Step 2 if
   you want minimal blast radius.

`AUDIT COMPLETE — FINDINGS REQUIRE REVIEW`
