---
part: Start Here
status: verified
type: spec
updated: 2026-10-23
review-by: 2026-11-23
---

# How this documentation is organised

Video Editor is a Rust and FFmpeg non-linear editor for low-spec PCs, built around a slide-deck style timeline and a deliberately simple interface for senior users. The documents are grouped by what we built, not by date. Every page declares one area as `part:` in its front matter; the Doc-Client Web Hub groups by it. The repo root holds only `README.md`.

## The areas

| Area | What lives here |
|---|---|
| Playback engine | the continuous FFmpeg stream player, dual-deck cut playback, audio engine, frame cache and memory bounds: [docs/playback/](../../docs/playback/PLAN_DUAL_DECK_SEAMLESS_PLAYBACK.md) |
| Timeline and slides | the PowerPoint-style slide canvas, text slides, drag and drop on the timeline: [docs/timeline-slides/](../../docs/timeline-slides/PLAN_SIMPLE_TEXT_SLIDES.md) |
| Media import and export | media bin, folders, music import, export: [docs/media/](../../docs/media/PLAN_FOLDERS_TRACK_DELETE_REORDER.md) |
| Senior-friendly UI | the five-action context menu, undo and redo, simplified controls: [docs/senior-ui/](../../docs/senior-ui/PLAN_SIMPLIFIED_SENIOR_MENU.md) |
| Project plan and ops | the master plan, the gates, dependency security, and what is still open: [docs/project/](../../docs/project/PLAN_RUST_VIDEO_EDITOR.md), [GATES](../../docs/ops/GATES.md), chapters 01 to 03 |

Finished one-off fix plans and superseded plans are in [docs/archive/](../../docs/archive/PLAN_CUTS_LAG_FIX.md), each with a banner naming the commit that shipped it or the plan that replaced it. Nothing was deleted.

## Page types (`type:`)

- **spec**: how a system works or was designed to work; the page to read first for that area.
- **decision**: a short dated note of what was decided and why.
- **plan**: work not yet done (or not yet verified).
- **record**: evidence of a finished batch or ruling.

Status words: `in-progress` (open work), `shipped`, `verified`, `reference`, `superseded`.

## Rules that stop the pile

1. One current page per topic. A fix to an existing system updates that area's page; it does not add a new `PLAN_*.md`.
2. No new markdown in the repo root. File under `docs/<area>/` with the front matter above.
3. When a plan ships, record the commit in its banner and move it to `docs/archive/`. Search `git grep` for the file name first; if code or a test cites it, leave it.
4. Do not trust a plan's own status line. [Chapter 01](01-the-plan-store-ruled.md) shows that 22 of 34 plans still read as unbuilt long after they shipped. Rule a plan against the code and the commit log.

## Still open

- [What is actually open](02-what-is-actually-open.md) lists five items (PPTX export unreachable from the UI, Phase 7 low-memory mode, and others) that need an owner decision.
- The four engine design pages in `docs/playback/` and the slide, media and UI design pages are marked `shipped` on the strength of chapter 01's ruling at commit `9773faf`; they have no `covers:` line and no automatic staleness check.
