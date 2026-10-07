---
part: Plans
status: in-progress
updated: 2026-10-07
review-by: 2026-11-06
---
# What is actually open

Everything [the plan store](01-the-plan-store-ruled.md) claimed was pending had
shipped. These five are the reverse: nothing advertises them as open, and they
are. Found by ruling the plans against the code on 2026-10-07, at `9773faf`.

The repo is dormant (last commit 2026-08-19). Nothing here is urgent; it is here
so that whoever picks the repo up does not have to re-derive it.

## 1. PPTX export is built, tested, and unreachable from the UI

`src/export/pptx_exporter.rs` is a complete 341-line PowerPoint writer:
`export_to_pptx(timeline, path)`, re-exported as `video_editor::export::export_to_pptx`
(`src/export/mod.rs:8`), and asserted by **four** separate suites —
`tests/pptx_position_test.rs`, `tests/slide_dnd_tests.rs:1527`,
`tests/effects_and_stickers_tests.rs:124`, `tests/text_overlay_tests.rs:203`
(the last inspects the generated slide XML for `b="1"`/`i="1"`).

**No UI path calls it.** `ExportFormatTab` has exactly two variants —
`VideoMp4` and `PresentationPdf` (`src/ui/export_dialog.rs:8-11`) — and a grep for
`pptx` across all of `src/` outside `src/export/` returns nothing.

It was reachable and then was not: `cef248e` (2026-08-19, *"…and simplify export
formats"*) cut 80 lines from `export_dialog.rs` on the last working day in the
repo. **This may well have been deliberate** — the whole August arc is
senior-friendly simplification, and dropping a third export tab fits it exactly.
What is missing is the record. The plans promise PowerPoint output in their titles
(`PLAN_DRAG_DROP_POWERPOINT_SLIDES`, `PLAN_ADD_BLANK_PAGE_TOOLBAR_POWERPOINT`) and
nothing says the output format was dropped.

The tests are why nobody noticed: they call the exporter directly, so they stay
green while the feature is unreachable. **A feature covered only by tests that
bypass the UI is indistinguishable from a shipped one.**

**Decide one way or the other:** restore a PPTX tab, or delete the exporter and
its four test blocks in the same change and say here that PDF replaced it. Leaving
it is the only bad option — 341 lines of maintained dead code that reads as a
feature.

## 2. The master plan's Phase 7 shipped as an always-on default, not a toggle

`PLAN_RUST_VIDEO_EDITOR.md` Phase 7 asks for *"a low-memory mode toggle in settings
(reduces frame buffer to 30 frames, enforces 240p/360p proxy)"*. What exists:

- The 360p proxy is **unconditional**, not a mode — `proxy_generator.rs:15`
  generates a 360p intraframe proxy for every import; `Clip.proxy_path` is built
  for all clips (`src/core/clip.rs:16`).
- The frame cache is a **fixed 120 frames** — `frame_cache.rs:34`, commented
  *"120 frames @ 360p ~= 100MB RAM"*. Not 30, and not switchable.
- There is **no 240p path and no toggle**; a grep for `low_memory|lowmem|240p`
  finds nothing.

So the decision was made by making the low-spec behaviour the only behaviour,
which is a reasonable answer for an app whose whole premise is low-end hardware —
but the plan still reads as though a settings toggle is owed. Either record this as
the answer (and strike the toggle), or add the toggle. Note the numbers do not
quite line up: the plan's own budget is `< 150 MB` resident and the fixed cache
alone is sized at ~100 MB.

## 3. Phase 7's measurements were never taken

The same phase asks for three numbers, and no commit, test or document in the repo
reports any of them:

- CPU utilisation during an active 60 fps timeline scrub.
- Zero memory leaks over a 30-minute editing session.
- Frame-fetch latency `< 16 ms` and resident memory `< 150 MB` while multi-track
  editing (§7.3).

Everything in the playback arc — `-re` pacing, backpressure, the dual deck, texture
dirty-checking, the "60MB RAM ceiling" in `b58a6d5` — was tuned against *observed
stutter*, not against these targets. That is not nothing, but the plan's acceptance
criteria remain unmeasured, so "runs well on low-end hardware" is currently a
claim, not a measurement.

## 4. The Windows player test list for the music/import work is unverified

`PLAN_GRANDMA_MUSIC_AND_IMPORT_FIX.md` is honest about this and it is still true:
*"automated tests green; on-Windows player test list below is UNVERIFIED."* That
work is the last four commits in the repo (`20e6aaa`, `7803d96`, `15f59be`,
`d75917f`, then `c85da08` async import and `9773faf` the zero-slides music row),
i.e. the newest and least exercised code, and it is exactly the part that only a
human on Windows can confirm — real audio out of a real preview.

Its test list is the acceptance criteria. Run it on Windows before trusting the
music feature, and record the result here.

## 5. Two fully-merged branches are still lying around

`windows-distribution` (local) and `origin/feat/low-spec-nle` are both **0 commits
ahead of `main`** — verified, nothing stranded on either. They are leftovers, and a
branch list that contains merged branches makes the next reader check each one
before trusting `main`. `windows-distribution` was deleted locally in this pass;
`origin/feat/low-spec-nle` is a remote branch and was left alone.

## Not open, recorded so it is not re-investigated

- **`PLAN_FILMSTRIP_DRAG_DROP_SLIDE_REORDER.md`'s "CODE UNCOMMITTED" is stale, not a
  loss.** The code landed as `2bc5be1`, hours after the plan was committed. Working
  tree clean, no stashes, no branch ahead of `main`. See
  [the plan store](01-the-plan-store-ruled.md).
- **The master plan's three missing test names are not missing tests.** 218 test
  functions exist and cover those areas under different names.
