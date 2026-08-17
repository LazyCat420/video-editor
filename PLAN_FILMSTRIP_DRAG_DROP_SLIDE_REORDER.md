# Implementation Plan: Drag-and-Drop Slide Reordering in the Bottom Filmstrip

**Repository:** `video-editor` (`LazyCat420/video-editor`)
**Status:** **IMPLEMENTED, CODE UNCOMMITTED** — the code sits in the working tree of the
primary checkout, not on a branch. This document is committed on its own. See §6 for why,
and §7 for what is still unverified.

---

## 1. Problem Statement

The bottom filmstrip (`render_horizontal_filmstrip`, the strip of `#1 … #12` cards under the
slideshow preview) could only reorder slides **one position at a time** via the `◀` / `▶`
buttons in each card's header. Moving slide 1 to the end of a 12-slide deck took eleven
clicks.

The request: keep the one-step arrows, and *additionally* allow dragging a slide card to any
position in the deck in a single gesture.

---

## 2. What Already Existed

Reordering was never the missing piece. Three things were already in place:

| Piece | Location | Note |
|---|---|---|
| `reorder_slide(from_idx, to_idx, ctx)` | `src/app/slide_ops.rs:325` | Already an **arbitrary-index** move: `remove(from)` + `insert(to)`. Already snapshots for undo and calls `reflow_slide_timeline_positions()`. |
| `MoveSlideUp` / `MoveSlideDown` | `src/ui/slide_deck.rs` | The arrow buttons, dispatching to `reorder_slide(idx, idx±1)`. |
| An in-repo DnD idiom | `src/ui/timeline_view.rs:305-392` | `TrackReorderDrag(u64)` — `dnd_set_drag_payload` on a handle, `dnd_hover_payload` to highlight, `dnd_release_payload` to commit. |

So the work was a UI affordance on top of an existing, already-undoable primitive — not new
timeline logic.

---

## 3. The Design Decision That Mattered: Targets vs. Gaps

The existing `TrackReorderDrag` pattern drops **onto a target row** and reorders to that
row's index. For a vertical list of a handful of tracks that is adequate.

**It does not generalise to a horizontal filmstrip.** Dropping on card #3 is ambiguous — it
can mean "before #3" or "after #3" — and a target-based scheme has no way to express "past
the last card", which is exactly how a slide reaches the end of the deck.

So the filmstrip resolves an insertion **gap**, not a target:

- A gap is a slot *between* cards. Gap `g` in a deck of `len` means "land between the slides
  currently at `g-1` and `g`". Gaps run `0..=len`, so there are `len+1` of them.
- The gap is computed from pointer-x against each card's **centre**:
  `card_rects.iter().position(|r| pos.x < r.center().x).unwrap_or(len)`.
  Release over a card's left half → before it. Right half → after it. Past the last card →
  `unwrap_or(len)` → append.

This is the PowerPoint behaviour, and the `unwrap_or(len)` fallthrough is the only reason
"drag to the very end" is expressible at all.

---

## 4. The Off-By-One This Encodes

Reordering is `remove(from)` **then** `insert(target)`. Removing first shifts every later
element down by one, so a gap to the *right* of the source overshoots by exactly one.

`gap_to_target_index` (`src/ui/slide_deck.rs`) is the single place this is decided:

```rust
pub fn gap_to_target_index(from_idx: usize, to_gap: usize, len: usize) -> Option<usize> {
    if from_idx >= len { return None; }
    if to_gap == from_idx || to_gap == from_idx + 1 { return None; }  // no-op
    let target = if to_gap > from_idx { to_gap - 1 } else { to_gap };
    Some(target.min(len - 1))
}
```

Three behaviours are deliberate:

1. **Rightward drags decrement, leftward drags do not.** Nothing before the source has moved,
   so a leftward gap *is* the destination index.
2. **Both gaps flanking the dragged card return `None`.** Picking a slide up and putting it
   back is a no-op, and must not push an undo snapshot — otherwise a stray click-drag costs
   the user a Ctrl+Z step that appears to do nothing.
3. **Out-of-range source returns `None`** rather than panicking.

### Why it is a function and not inline arithmetic

`tests/slide_dnd_tests.rs:246` (`test_media_bin_reorder_forward_and_backward`, pre-existing)
re-derives this same shift correction **inside the test body** and asserts on its own local
result. That test cannot see the production code drift — it would stay green if the shipping
conversion changed underneath it. The new tests call `gap_to_target_index` directly so the
assertions bind to the code that actually runs.

---

## 5. Implementation

| File | Change |
|---|---|
| `src/ui/mod.rs` | New payload `SlideReorderDrag(pub usize)` — carries the source index only; the destination is resolved at drop time from pointer position. |
| `src/ui/slide_deck.rs` | New action `ReorderSlideToGap { from_idx, to_gap }`; `gap_to_target_index`; thumbnail becomes the drag handle (`Sense::click_and_drag()`); card rects collected during layout; gap resolution + drop-indicator painting; carried card dimmed. `render_horizontal_slide_card` now returns `(SlideDeckAction, egui::Rect)`. |
| `src/app/slide_ops.rs` | New `slide_count()` helper (video-track clip count — the same list the filmstrip indexes). Dispatch for `ReorderSlideToGap`. |
| `src/app/mod.rs` | Same dispatch arm — the deck action enum is matched in **two** places (`app/mod.rs:635` and `app/slide_ops.rs:604`); both had to be updated or the build breaks on a non-exhaustive match. |

### Interaction details

- **Click still selects.** egui's `clicked()` is already false for the release that terminates
  a drag, so a reorder gesture does not double as a selection.
- **The header buttons are untouched.** `#`, `×`, `◀`, `▶` keep their own click senses; only
  the thumbnail is drag-sensitive, so the arrows still work as before.
- **Payload is taken on release** via `DragAndDrop::take_payload` guarded by
  `pointer.any_released()`, so the drop resolves exactly once.
- **Feedback:** a 3px yellow vertical line painted in the live target gap (only when
  `gap_to_target_index` says the move is real), plus the carried card dimmed to
  `rgb(16,22,30)` with a cyan border.

---

## 6. Why the Code Is Uncommitted

The primary checkout carries **another session's uncommitted work** — a stickers/effects
feature (`SlideElement::Sticker`, `src/core/stickers.rs`, `src/core/effects.rs`,
`src/ui/effects_and_transitions_bin.rs`, ~87 sticker PNGs), all untracked or unstaged.

The two files this feature must edit (`slide_deck.rs`, `app/mod.rs`) already contain that
work. Staging them produced a diff carrying `SlideElement::Sticker` match arms and
`SidebarTab::EffectsAndTransitions` wiring that **depend on files git does not track**, so
the branch would not have compiled standalone and the commit would have bundled someone
else's unfinished feature under this message.

Work was done in a worktree (`slide-filmstrip-dnd`) as the workflow requires, verified there,
then copied into the primary tree so the feature is runnable. That worktree and branch were
removed. Pre-change backups of the five touched files: `scratchpad/pbak/`.

**Open item:** the code still needs committing once the stickers/effects work lands, and the
commit must separate the two features.

**Secondary hazard worth recording:** the bulk of the primary tree's dirty diff is CRLF/LF
line-ending churn — `git diff --stat` reported ~5900 changed lines across 19 files, while
`git diff --ignore-all-space` reported 72 real ones. A reviewer trusting the raw stat would
badly misjudge the scope of what is in flight there.

---

## 7. Verification

### Automated — `cargo test --test slide_dnd_tests`

Five new tests, all passing:

| Test | Pins |
|---|---|
| `test_gap_to_target_index_shift_correction` | The rightward decrement, the append case, leftward identity, both no-op gaps, out-of-range rejection. |
| `test_drag_slide_to_arbitrary_position_reorders_deck` | First→last, last→first, and a middle move, through the real `reorder_slide`. |
| `test_dragged_slide_keeps_its_content_and_timeline_reflows` | Background + elements travel with the slide; timeline starts stay contiguous with mixed durations (0/10/15s). |
| `test_arrow_reorder_still_moves_one_position` | The `◀`/`▶` path is unregressed; a move past the end is a no-op, not a panic. |
| `test_slide_reorder_is_undoable` | Ctrl+Z restores the prior order. |

### Sabotage check — the tests are not vacuous

Replacing `let target = if to_gap > from_idx { to_gap - 1 } else { to_gap };` with
`let target = to_gap;` (deleting the shift correction) turned
`test_gap_to_target_index_shift_correction` and
`test_drag_slide_to_arbitrary_position_reorders_deck` **red**. Restored, both green again.
The tests can fail on the defect they claim to guard.

### Build

`cargo build --bin video-editor` clean, no warnings. Note this project's `.cargo/config.toml`
sets `build.target = "x86_64-pc-windows-gnullvm"`, so the default build produces
`target/x86_64-pc-windows-gnullvm/debug/video-editor.exe` — **not** a Linux binary, and
nothing appears at `target/debug/video-editor`. A plain `cargo build` reporting "Finished"
with no binary at the expected path is this config, not a failure.

### NOT verified: the interaction itself

**No part of the drag gesture has been exercised on screen.** The tests cover
`gap_to_target_index` and `reorder_slide`; they do **not** cover the egui wiring —
`drag_started`, payload set/take, the pointer-x→gap mapping against real laid-out rects, or
the indicator line. A green suite here is consistent with a drag handle that never arms.

Attempted and failed on this box:

- `xvfb-run` + `ffmpeg -f x11grab` under `LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe`:
  the process stays up (45s, no panic, empty log) but **never maps a window** — the captured
  root is 2 colours after 50s. Software GL cannot present this app here.
- The real target is the Windows `.exe`, which this session cannot drive.

So the following are **UNVERIFIED** and need a human at the app:

1. Dragging a thumbnail actually arms a drag (vs. only ever registering as a click).
2. The drop lands where the yellow indicator showed.
3. Drag to the very end and to the very front.
4. Click-without-move still selects and does not reorder.
5. `◀` / `▶` still move one position.
6. Ctrl+Z restores order after a drag reorder.
7. A drop-in-place consumes no undo step.

---

## 8. Pre-Existing Failure (not from this work)

`test_calendar_box_resizing_and_properties` fails with `left: 0.46, right: 0.9`
(`tests/slide_dnd_tests.rs:1024`). It fails **identically on the untouched primary checkout**,
from the in-flight calendar work. Not caused by, and not addressed by, this change.
