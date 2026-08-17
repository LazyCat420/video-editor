# PLAN: Play walks the whole deck from the playhead, not just the selected slide

## Symptom (user-reported)

With multiple slides on the timeline, pressing Play does not play from the
beginning through to the end. It plays only the slide that is currently
selected, and the preview only shows what is playing if the user clicks the
slide that is playing.

## Root cause — VERIFIED by a failing test

`VideoEditorApp::active_slide()` (`src/app/slide_ops.rs:12`) resolves the slide
to render in three tiers, **selection first**:

```rust
// tier 1: any clip with is_selected
// tier 2: the clip under the playhead
// tier 3: the first clip on the track
```

Every preview render path is driven by `active_slide()`:

| Call site | File |
| --- | --- |
| playing branch of the frame loop | `src/app/mod.rs:390` |
| paused branch of the frame loop | `src/app/mod.rs:447` |
| `refresh_preview_frame()` | `src/app/playback.rs:354` |
| `slide_visuals()` (elements/videos) | `src/app/playback.rs:202` |

So while `is_playing` is true and the playhead advances across slide
boundaries, tier 1 keeps returning the **selected** clip. The playhead really
does move and the timeline really does end at the right time — only the
*picture* is pinned. That exactly matches "it only plays the slide you select,
and you have to click the playing slide to see it".

### Reproduction (already written and run)

`tests/playback_sequence_tests.rs`, three 5s blank slides:

```
test playhead_picks_the_slide_it_is_over_when_nothing_is_selected ... ok
test selection_still_drives_the_preview_while_paused ... ok
test timeline_duration_spans_every_slide ... ok
test playback_follows_the_playhead_not_the_selection ... FAILED
    left: Some(5)    // slide 3 — the selected one
    right: Some(3)   // slide 1 — the one under the playhead at t=0
```

The three passing tests are the controls: they prove the harness builds a real
timeline, that playhead resolution works when nothing is selected, and that
click-to-preview while paused is genuinely a separate behaviour we must keep.
Only the playing case fails. This is a positive control set, not a green suite.

## Secondary defects found while tracing (both contribute to the symptom)

**B. Slide video decoders are never restarted after a rewind.**
`src/app/playback.rs:277` decides whether to start a decoder with
`is_first = !self.slide_video_players.contains_key(&path)`. `pause_playback()`
and `seek_to()` call `player.stop()` on those entries but **never remove them**
(`src/app/playback.rs:28`, `:49`). After the first play the key is present
forever, so `is_first` is false, `start()` is never called again, and a rewind
leaves a stopped decoder that yields no frames. The same bug means a video used
on two different slides never re-seeks for the second slide.

**C. `slide_elapsed` is computed from the wrong slide.**
`slide_visuals()` (`src/app/playback.rs:206`) derives
`slide_elapsed = playhead - active.timeline_start` from whatever `active_slide()`
returned. When tier 1 hands back the selected slide while the playhead is
elsewhere, this value is meaningless — often negative and clamped to 0, which
freezes the element video on its first frame. Fixing A largely fixes C, but the
clamp should be made explicit against the *rendered* slide.

## The fix

### 1. Make slide resolution state-aware (fixes A)

Split the single `active_slide()` into two intents, because the app genuinely
has two:

- **`slide_for_playback()`** — strictly playhead-first: clip under the
  playhead, then nothing. Used by every *render* path.
- **`active_slide()`** — keeps today's selection-first behaviour. Used by every
  *editing* path (drop targets, element add/delete, the slide deck highlight,
  `resolve_target_slide_id`).

Then have the render paths choose by transport state:

```rust
pub fn slide_to_render(&self) -> Option<&Clip> {
    if self.project.timeline.is_playing {
        self.slide_for_playback()
    } else {
        self.active_slide()
    }
}
```

Swap `active_slide()` → `slide_to_render()` at the four render call sites listed
above. Editing call sites (`slide_ops.rs`, `canvas_ops.rs`, `calendar_ops.rs`)
are left on `active_slide()` and are unaffected.

Why not just delete the selection tier? Because clicking a slide while paused
*should* preview it — that is the fourth test, and it currently passes. A blanket
change would trade this bug for a regression.

### 2. Keep the deck selection following the playhead during playback

While playing, move `is_selected` onto the slide under the playhead as it
crosses each boundary. This makes the slide deck highlight track playback so the
user sees which slide is on screen without clicking, and it leaves the selection
on the last-played slide when playback stops. Guard it so it only fires on an
actual boundary crossing, not every frame (compare against
`current_playing_clip_id`).

### 3. Restart decoders on rewind / seek / slide change (fixes B)

Replace the `stop()`-in-place loops with removal so the next play re-seeks:

- `pause_playback()` and `seek_to()`: drop the entries
  (`self.slide_video_players.clear()`), not just `stop()` them.
- In `slide_visuals()`, restart a decoder whenever the requested
  `slide_elapsed` has jumped backwards or the player is no longer running,
  rather than only on first insertion.

### 4. Make `start_playback` explicit about starting where the playhead is

`start_playback()` already reads the playhead, so a Rewind-then-Play works at
the clip level. Add the selection sync from step 2 so the very first frame after
Rewind shows slide 1 rather than the stale selection.

## Test plan

Extend `tests/playback_sequence_tests.rs`:

1. `playback_follows_the_playhead_not_the_selection` — **must go green** (it is
   red today; that is the proof the fix does something).
2. `selection_still_drives_the_preview_while_paused` — **must stay green** (the
   anti-regression guard for click-to-preview).
3. New: walk the playhead in 0.5s steps from 0 to 15s and assert the sequence of
   rendered slide ids is exactly `[1,1,…,2,2,…,3,3,…]` with no gaps and no
   backtracking — catches an off-by-one at the boundaries.
4. New: assert selection follows the playhead during playback and does **not**
   move while paused.
5. New: after `stop_playback()` (rewind to zero) then `start_playback()`, the
   rendered slide is slide 1 even though slide 3 was selected.
6. New: `slide_video_players` is empty after `pause_playback()` and after
   `seek_to()`, so the next play re-seeks (the B guard).

Run: `cargo test --test playback_sequence_tests` plus the full
`cargo test` suite to confirm no regression in the existing timeline/dnd tests.

## Risk / blast radius

Low and contained. The change adds a resolver and swaps four call sites; no
existing function's behaviour changes for editing. The main thing to watch is
that transitions (`composite_transition`) are fed the rendered slide's
`track_id`/`id`, which they already are — they take whatever the caller resolved.

## What I have NOT verified

- The fix itself is not implemented yet — only the diagnosis is test-backed.
- Nothing here is confirmed on screen. Per the repo's history the Windows `.exe`
  will not render under Xvfb, so visual confirmation needs a build the user runs,
  or the linux target. The tests above are logic-level, not pixel-level.

---

# SHIPPED — implementation + proof (2026-08-16)

All of the above is now implemented, plus four more defects found while
auditing slide creation and the slideshow UI at the user's request.

## What changed

| Fix | Where |
| --- | --- |
| `slide_for_playback()` / `slide_to_render()` split; 4 render sites swapped | `slide_ops.rs`, `playback.rs`, `mod.rs` |
| Selection follows the playhead during playback (`sync_selection_to_playhead`) | `playback.rs`, called from the playing branch of `update()` |
| `pause_playback`/`seek_to` DROP slide decoders (`clear()`, Drop stops ffmpeg) instead of stopping them in place | `playback.rs` |
| Decoder restart predicate `needs_restart_for` — restarts on rewind/file-switch/never-started, never on EOF or forward stalls (no ffmpeg spawn storm) | `stream_player.rs` |
| Slide inserts go through `insert_slide_after_active` + reflow — no more overlapping slides when the playhead isn't at the deck end (e.g. right after Rewind) | `slide_ops.rs` (blank + 3 templates + `resolve_target_slide_id`) |
| Bottom slideshow panel now calls `render_horizontal_filmstrip` (it existed but had ZERO callers — budgeted, never drawn) below the transport row | `slide_ops.rs::render_bottom_slideshow_bar` |
| Deck thumbnails are real miniatures (`paint_slide_mini`: background + elements at true positions; textures prefetched by `ensure_slide_thumb_textures`) | `slide_deck.rs`, `playback.rs` |
| Prev/Next Slide now select AND seek (they previously only moved `is_selected`, so the next Play started elsewhere; also removed a `reorder_slide(0,0)` no-op hack) | `slide_ops.rs` |
| Preview canvas left-anchored (`vertical` not `vertical_centered`) — the 16:9 letterbox slack no longer opens a black bar against the sidebar | `preview_player.rs` |

## Proof

- `tests/playback_sequence_tests.rs` grew from 4 to 10 tests. The originally
  RED test (`playback_follows_the_playhead_not_the_selection`, failed with
  `left: Some(5), right: Some(3)` on the old code) is GREEN through the real
  render entry (`slide_to_render`). The metric can fail: it did, on HEAD~.
- New: playhead walk 0→15s visits slides exactly `[1,2,3]` with no
  backtracking under adversarial selection; selection sync fires only while
  playing; Rewind→Play renders slide 1; three inserts at playhead ZERO pack to
  `[0,5,10]`; mid-deck insert lands after the playhead slide; pause AND seek
  leave `slide_video_players` empty.
- Full suite: **97 tests, 0 failures** — including the 44 pre-existing
  slide-DnD tests. Two of those initially failed against a playhead-position
  insert; the after-active-slide (PowerPoint) rule satisfies both old and new
  expectations and was adopted deliberately.

## Open items

- NOT verified on screen: the Windows .exe does not render under Xvfb on this
  box, so filmstrip layout, thumbnail quality, and the sidebar gap need one
  visual pass by the user (test list below).
- `active_slide()`'s tier-3 "first clip" fallback is unchanged; editing paths
  still use selection-first resolution on purpose.
- Slide-element AUDIO during playback is still not mixed (pre-existing).

## What YOU should test (player test list) — UNVERIFIED on screen

1. **Rewind → Play with 2+ slides**: click slide 2 first, hit ⏮ Rewind, then
   PLAY. Right: it plays slide 1 then slide 2 to the end, and the highlighted
   card in the strip follows along. Bug: picture stays on slide 2 or the
   highlight doesn't move.
2. **Slide videos after rewind**: put a video on slide 1, play past it, rewind,
   play again. Right: the video plays again from its start. Bug: frozen frame.
3. **Bottom strip**: in Slideshow mode the area under the transport buttons
   shows one card per slide with a real miniature (photos where the photos
   are, text where the text is), plus an ➕ Add Slide card. Bug: black void or
   icon-only cards after a few seconds (textures may take a moment on first
   load — that part is expected).
4. **Add New Slide right after Rewind**: total duration must GROW by 5s (check
   the 00:xx/00:xx readout). Bug: total unchanged → slides overlapped.
5. **Prev/Next Slide buttons**: clicking them moves both the preview AND the
   playhead (the time readout jumps to that slide's start).
6. **Sidebar gap**: the preview canvas should sit flush against the sidebar
   with slack only on the far right. Bug: black column between sidebar and
   canvas.
