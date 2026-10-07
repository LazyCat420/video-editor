---
part: Plans
status: reference
updated: 2026-10-07
---
# The 34 root plans, ruled against the code

This repository kept its entire written record as `PLAN_*.md` files at the root
and had no `documentation/` at all, so nothing ever aged them. On 2026-10-07 the
daily documentation audit found **22 of the 34** still reading as unbuilt work —
*"Plan for review — not to be implemented until approved"*, *"awaiting user
approval before implementation"*, *"Ready for user brainstorming review"* —
over features that are in `src/`, covered by tests, and shipped weeks ago.

The repository has been dormant since **2026-08-19** (0 commits in the 30 days
before this ruling), so these rulings are stable: the code they were checked
against is not moving.

## Where this renders

Nowhere yet. There is no `build_docs.py` in this repo and no served page — these
chapters are read in git. What they *do* buy is that `docs_audit.py` reads
`<repo>/documentation/chapters/*.md`, so from today this repo's record is inside
the thing that checks whether documentation is still true, instead of outside it.

## How each plan was ruled

Not by its own status line, and not by its file date. Both are actively
misleading here (see [the two traps](#the-two-traps-this-store-set) below). Each
plan was ruled by two independent checks:

1. **A commit whose subject restates the plan's own title** — this log is unusually
   descriptive, and the correspondence is near 1:1.
2. **The nouns in the current tree**: the module, type or test the plan said it
   would create, read out of `src/` and `tests/` at `9773faf`.

A plan is ruled `shipped` only where both agree. **No build or test run was
performed this pass** — this is a documentation ruling from source, so "shipped"
here means the code is present and reachable, not that the suite is green today.

## The two traps this store set

**1. The plan files are younger than the work they plan.** 21 of the 34 were added
to git in a single commit — `8110aeb` (2026-08-16), whose message says *"carry
over accumulated plan documents"*. The playback work they plan shipped on
2026-08-13. So every one of those files has a git date **three days after** its
own implementation, and a staleness clock keyed on file age ranks them as the
freshest documents in the repo.

**Worse, four plans were added by the very commit that implemented them:**

| Plan | Added by | Which is |
|---|---|---|
| `PLAN_SIMPLE_TEXT_SLIDES.md` | `19fda9a` | `feat(slides): unified slide editor - click-to-place text/media…` |
| `PLAN_FOLDERS_TRACK_DELETE_REORDER.md` | `23fcfa4` | `Add folder bins, drag-to-timeline, track delete & reorder` |
| `PLAN_VIDEO_PLAYBACK_MULTI_CUT_LAG_FIX.md` | `f8fb85f` | `Fix media thumbnails, floating reorder, folder import behavior` |
| `PLAN_FIX_VIDEO_EXPORT_IMAGE_INPUTS.md` | `3d1ec22` | `fix(export): pass still images as bounded looped inputs` |

`PLAN_SIMPLE_TEXT_SLIDES.md` says **"not to be implemented until approved"** in the
same commit that implements it. No value of a staleness threshold reaches that
file: it is exactly as old as the code that refutes it. The only instrument that
works is the one that ignores dates — does the code contain the plan's nouns.

**2. A plan's own test names are not the tests that shipped.** The master plan
(`PLAN_RUST_VIDEO_EDITOR.md` §7.1) names four tests. One exists
(`test_envelope_linear_interpolation`, `tests/envelope_tests.rs`); the other three
— `test_envelope_multi_node_sorting`, `test_timeline_clip_split`,
`test_filter_graph_volume_curve_syntax` — **do not exist under those names**, while
the repo carries **218** test functions and suites covering exactly those areas
(`tests/envelope_tests.rs`, `tests/timeline_tests.rs`, `tests/filter_graph_tests.rs`).
Greping a plan's own test names is a check that fails on a healthy tree. Verify the
behaviour, never the identifier the plan guessed.

## Ruling: shipped (22)

Each row is the plan, the commit whose subject restates it, and the code that
proves it present at `9773faf`.

| Plan | Shipped by | Code |
|---|---|---|
| `ADD_BLANK_PAGE_TOOLBAR_POWERPOINT` | `0656c5c` add ➕ Add Blank Page button on timeline toolbar; canvas by `d1f2fbc`/`19fda9a` | `src/ui/timeline_view.rs`, `src/app/slide_ops.rs` |
| `AUTO_FIT_SLIDE_DURATION_TO_MEDIA` | `87a4b0e` auto-fit blank slide duration to longest media clip with timeline ripple shift | `tests/slide_dnd_tests.rs` (ripple/longest cases) |
| `CUT_CLIPS_PLAYBACK_FIX` | `26afbf2` removing -re stall and passing exact segment duration bounds | `src/media/stream_player.rs` |
| `DND_CARD_REORDER_ANIMATION` | `2e60a30` push-down drop slots, dimmed placeholders, cursor ghost previews | `src/ui/components/card.rs`, `src/ui/media_bin.rs` |
| `DRAG_DROP_POWERPOINT_SLIDES` | `d1f2fbc` drag-and-drop media to canvas, click-to-add text, context-aware inspector | `src/app/canvas_ops.rs`, `src/ui/slide_deck.rs` |
| `FIX_MULTI_VIDEO_SLIDE_PLAYBACK` | `783404c` continues until longest video finishes with independent end clamping | `src/app/playback.rs` |
| `FIX_SLIDE_BACKGROUND_GLITCH` | `aec62a5` …and guarantee solid canvas frame | `src/app/canvas_ops.rs` |
| `FIX_VIDEO_MIRRORING_ON_BLANK_SLIDE` | `aec62a5` eliminate video mirroring in slide background | `src/app/canvas_ops.rs` |
| `FOLDERS_TRACK_DELETE_REORDER` | `23fcfa4` folder bins, drag-to-timeline, track delete & reorder | `probe.rs:66 scan_folder_for_media`, `media_bin.rs:14 ImportFolder`, collapsible folder groups `media_bin.rs:118-375` |
| `IMPORT_MEDIA_BIN_NO_AUTO_TIMELINE` | `c3db773` prevent auto-adding imported files/folders to timeline | `src/ui/media_bin.rs` |
| `PLAYBACK_PREVIEW_DECODER_FIX` | `2dfff71` continuous FFmpeg rawvideo playback engine; `35dcf95`, `c45d1c0` | `src/media/stream_player.rs`, `src/ui/preview_player.rs` |
| `PTS_CLOCK_SYNC_FIX` | `2c86e52` PTS timestamp presentation clock sync to eliminate 2x drain and freeze | `src/media/stream_player.rs` |
| `SIMPLE_TEXT_SLIDES` | `19fda9a` unified slide editor; `a36660a` title cards, on-clip overlays, font picker | `src/core/text_overlay.rs` (435 ln), `src/ui/text_renderer.rs`, `tests/text_overlay_tests.rs` |
| `SIMPLIFIED_SENIOR_MENU` | `3a806fd` ultra-simplify context menu to 5 core actions | `src/ui/menu_bar.rs` |
| `SLIDE_VIDEO_PLAYBACK_TIMELINE_AND_DND_FIX` | `92dcd93` real-time slide video playback, timeline layer badges, media bin dnd reorder fix | `src/app/playback.rs`, `src/ui/timeline_view.rs` |
| `SMOOTH_PLAYBACK_AND_MEMORY_MANAGEMENT` | `b58a6d5` stderr pipe deadlock + texture dirty-checking for 60MB RAM ceiling | `src/media/frame_cache.rs`, `src/ui/preview_player.rs` |
| `STREAM_FREEZE_FIX` | `b58a6d5` (pipe deadlock) + `6a8e084` (rate pacing) — the plan's own subtitle names both | `src/media/stream_player.rs` |
| `STREAM_RATE_BACKPRESSURE_AND_CLIP_TRANSITIONS` | `6a8e084` -re rate pacing, producer backpressure, cross-clip transition detection | `src/media/stream_player.rs` |
| `TIMELINE_DRAG_AND_FORMAT_EXPANSION` | `2ae96ae` backwards clip dragging, exclude self from snapping, grabbing-hand cursor | drag in `src/ui/timeline_view.rs`; formats in `src/media/probe.rs:20-27` — 10 video + 10 audio extensions, both cases |
| `VIDEO_AUDIO_PLAYBACK_ENGINE_AUDIT` | `2dfff71` + the audio engine commits `d75917f`/`20e6aaa` | `src/audio/` (mixer, music_engine, player, envelope_eval) |
| `VIDEO_PREVIEW_AND_SENIOR_UI_FIX` | `c45d1c0` frame extraction, auto-zero playhead, simplify UI for senior accessibility | `src/ui/preview_player.rs`, `src/ui/theme.rs` |
| `ZERO_LATENCY_CUT_TRANSITION` | `1d2fe95` DualDeckPlayer lookahead pre-buffering for 0ms cut transitions; `c63cfd5` | `DualDeckPlayer` in `src/media/stream_player.rs`, used from `src/app/mod.rs` |

## Ruling: already ruled where a reader meets them (5)

These carry a correct verdict in their own first lines. **That is the pattern to
copy** — a ruling in a chapter does not fix the file a reader actually opens.

| Plan | Says | Checked |
|---|---|---|
| `FIX_SIDEBAR_PREVIEW_DEAD_GAP` | SHIPPED, merged as `a8c0f66` | True; `c93a93a` then fixed it structurally (cap the panel, fit the rows), `tests/sidebar_width_tests.rs` |
| `FIX_VIDEO_EXPORT_IMAGE_INPUTS` | Implemented and verified, branch `fix/export-loop-arg-order` | True, landed as `3d1ec22`; branch gone, work on `main` |
| `GRANDMA_MUSIC_AND_IMPORT_FIX` | Implemented on `feat/grandma-music`; on-Windows player test list **UNVERIFIED** | True and still the honest state — see [What is actually open](02-what-is-actually-open.md) |
| `PLAY_ALL_SLIDES_FROM_START` | diagnosed with a failing test (`70ff78d`) | Fixed by `d98c04e` *play walks the whole deck; filmstrip, thumbnails, overlap, gap* |
| `VIDEO_PLAYBACK_MULTI_CUT_LAG_FIX` | "Confirmed root cause (all 4 subagents converge)" | Fixed by `fcb3253` drop -t cap on continuous streams + skip redundant frame re-clone |

### One of them was wrong in a way worth keeping

`PLAN_FILMSTRIP_DRAG_DROP_SLIDE_REORDER.md` still reads **"IMPLEMENTED, CODE
UNCOMMITTED — the code sits in the working tree of the primary checkout, not on a
branch."** That was true for a few hours on 2026-08-17: the plan was committed
first (`692a81a`), then the code (`2bc5be1` *drag a slide anywhere in the filmstrip
to reorder*). It has been false for 51 days, and it is the one claim in this store
that would have sent a reader hunting for lost work. Checked today: the working
tree is clean, there are no stashes, and `windows-distribution` and
`origin/feat/low-spec-nle` are both **0 commits ahead of `main`** — nothing is
stranded. `tests/slide_dnd_tests.rs` covers the behaviour.

## Ruling: superseded in place (2)

Both describe a lag/CPU problem that a later, differently-shaped fix closed. Kept,
not deleted — their root-cause analysis is why the current design looks as it does.

- `PLAN_CUTS_LAG_FIX` and `PLAN_CONTINUOUS_STREAM_CPU_FIX` → the continuous-stream
  approach landed as `118e034` *preserving continuous active stream across cuts*
  and `fcb3253` *drop -t cap on continuous streams*, then was superseded again by
  the dual-deck engine (`1d2fe95`). `PLAN_DUAL_DECK_SEAMLESS_PLAYBACK` is the
  design that won; read it first.

## The master plan's four open questions were answered by what shipped

`PLAN_RUST_VIDEO_EDITOR.md` §8 closes by asking the owner four questions and
calling itself *"Ready for user brainstorming review"*. Three were decided in code
and one by the repo's own existence. Nobody closed them, so they read as
outstanding forever:

| # | Question | Answered by the code |
|---|---|---|
| 1 | Repo inside `sun/` or standalone? | Inside — this checkout is `sun/video-editor` |
| 2 | Volume keyframes per clip or per track? | **Per clip.** `Clip.volume_envelope` (`src/core/clip.rs:30`), and `split_at` carries the envelope into the second clip (`clip.rs:246-262`). `Track` keeps only a scalar `volume: f32` |
| 3 | Export formats and defaults? | MP4 H.264 is the default tab, PDF the second; `cef248e` *simplify export formats* settled it. See [What is actually open](02-what-is-actually-open.md) — PPTX did not survive this |
| 4 | Crossfades only, or hooks for text/filters? | Hooks, fully: 18 transitions (`5d2cf1b`), `src/core/effects.rs` (1,699 ln, 6 animations), `text_overlay.rs`, `stickers.rs` |

**Nothing in this store was deleted.** A dropped plan that cannot say why it was
dropped makes "why not" the expensive question all over again.
