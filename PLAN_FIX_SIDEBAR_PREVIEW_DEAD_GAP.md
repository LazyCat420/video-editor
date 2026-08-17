# Implementation Plan: Remove the Dead Gap Between the Sidebar and the Preview

**Repository:** `video-editor` (`LazyCat420/video-editor`)
**Status:** **SHIPPED** — merged to `main` as `a8c0f66`, pushed to `origin/main`.

---

## 1. Problem Statement & Root Cause

### Symptom
A black vertical strip roughly 60px wide ran the full height of the window between the
left sidebar and the video preview. It was present on both the Formatting and Transitions
tabs, and it moved the preview, the transport bar and the bottom slideshow bar to the right.

### Root Cause

The sidebar is declared `.exact_width(280.0)`, and the panel *was* 280px wide. The gap was
not a width setting — it came from how egui closes out a `SidePanel`. From
`egui-0.29.1/src/containers/panel.rs`:

```rust
let rect = inner_response.response.rect;  // the frame's CONTENT rect
// ...
cursor.min.x = rect.max.x;                // where the NEXT panel begins
```

The next panel starts wherever the *content* ended, not at the declared width. So:

1. A child widget wider than the panel makes the frame report a larger content rect.
2. `cursor.min.x` moves right by the overflow, pushing the central panel away.
3. The panel's own clip rect still cuts at 280px, so the surplus is **not** drawn as visible
   overflow — it paints as empty panel background. A gap, not a clipped widget.

**The overflow also compounds.** Widgets size themselves from `ui.available_width()`, so one
over-wide row raises the value its siblings read, and the next row grows a little more again.
Measured on the Transitions preset list, the running content width climbed
`272 → 277 → 284` across the catalogue. Any single row looked nearly fine; only the total
drifted.

### Three measured sources of overflow

| Source | File | Evidence |
|---|---|---|
| Long holiday name + colour swatch on a non-wrapping `ui.horizontal` | `src/ui/slide_bin.rs` | "Columbus / Indigenous Peoples Day" reached **334px** against 290 available; "Independence Day (4th of July)" 302; "Martin Luther King Jr. Day" 270 |
| Self-sizing emoji icon badge | `src/ui/components/card.rs` | Commented "fixed 28x24 px" but nothing enforced it. `◀️` (U+25C0 + FE0F) and `🔍` are wider than the badge budget and pushed their cards past the panel |
| Column widths read mid-row, and `item_spacing.x` omitted from the budget | `src/ui/components/card.rs` | `text_w` was derived from `ui.available_width()` *during* the row; the theme's 10px `item_spacing.x` between the three columns was never subtracted |

---

## 2. What Does NOT Fix This

Each of these was tried and **measured ineffective** — worth recording so they are not
re-attempted:

- `.exact_width(w)` — sets the layout width, is not a ceiling.
- `.width_range(w..=w)` — clamps `panel_rect`, but the cursor is advanced from the content
  rect, so it changes nothing here.
- `ui.set_max_width(w)` on the panel body — children still report their own desired width.
- `ui.set_clip_rect(..)` — limits *painting*, not size reporting.
- Hiding the scrollbar / `auto_shrink` / `ScrollArea::max_width` — the scroll style measured
  `floating=true, allocated_width=0.0`, so the scrollbar was never the cause.

**There is no post-hoc correction.** egui's `min_rect` can only grow: `set_min_width` and
`expand_to_include_rect` expand it, and `shrink_width_to_current` only lowers `max_width`.
Content that has already overflowed cannot be reeled back in. It must not overflow.

---

## 3. Changes Made

### `src/ui/components/card.rs`
- `ActionRowCard::render` takes a new `card_w: f32`. **The caller owns the width**; the card
  never measures it, so no row can widen its siblings.
- The icon glyph is centred in a hard-sized `ICON_GLYPH_W × ICON_GLYPH_H` cell with its own
  clip rect, so emoji width cannot grow the badge.
- The text column derives from `card_w`, subtracts `item_spacing.x` explicitly for both
  column gaps, and leaves 2px for sub-pixel rounding. Title and description use
  `Label::truncate()`.

### `src/ui/slide_bin.rs`
- New `SlideBinView::holiday_row` helper, shared by the US and Chinese lists (previously two
  copies of the same block). Lays out right-to-left so the swatch is placed first and the
  label is confined to what is left. The row height is allocated explicitly — a bare
  right-to-left layout claims all remaining panel height and renders one holiday per screen.

### `src/ui/transition_bin.rs`
- `row_w` is captured *before* the `ScrollArea`, while `available_width()` still reflects the
  sidebar's real content width, and passed to each card.

### `src/app/mod.rs`
- `SIDEBAR_INNER_MARGIN_X` constant so the frame margin and the content clamp cannot drift.
- The panel body is clamped to `sidebar_width - 2 * margin` so `available_width()` is constant
  for every child.

---

## 4. Verification

Measured by instrumenting `ctx.available_rect()` around each panel and
`ui.min_rect().max.x` around each section, then stripping the instrumentation.

| | Before | After |
|---|---|---|
| Sidebar content extent | 334 (Formatting) / 284 (Transitions) | **272** |
| Central panel starts at | 342 / 292 | **280** |
| Dead gap | 62px / 12px | **0px** |

All three tabs land at 280. Confirmed on screen with before/after screenshots from the same
build (`--target x86_64-unknown-linux-gnu` under Xvfb, captured with `ffmpeg -f x11grab`),
not from the numbers alone.

`cargo test`: **86 passed, 0 failed.** `cargo build` clean.

### Screenshot note for future layout work
The default build target is a Windows `.exe` (`x86_64-pc-windows-gnullvm`, set in
`.cargo/config.toml`) and will **not** render under Xvfb — it produces a black frame. To
verify a layout change visually on Linux, build `--target x86_64-unknown-linux-gnu` and
capture that binary instead.

---

## 5. Open Items

- **Slides tab not visually confirmed.** Slideshow mode force-switches away from
  `SidebarTab::Slides` (`src/app/mod.rs`), so every measurement landed on Formatting. The
  fix is container-level and the tab shares the same clamped body, but it was not observed
  in Timeline mode.
- **The ratchet is latent elsewhere.** Any future sidebar widget that sizes itself from
  `ui.available_width()` mid-row can reopen this gap. The durable rule is the one applied to
  `ActionRowCard`: the container decides the width and passes it in.
- Pre-existing unrelated warning: unused `RichText` import in `examples/slider_shot.rs`.

---

# RECURRENCE + FINAL FIX (2026-08-16): the clamp never worked — the cap does

The gap came back (~145px, user screenshot, Formatting tab with a picture
element selected). The original fix was built on a wrong premise; this chapter
records the verified mechanism so it does not recur a third time.

## Why the a8c0f66 clamp could not work — egui 0.29.1 source, verified

- `exact_width(280)` clamps only the panel's INPUT width (`panel.rs:236-246`).
- The panel's REPORTED rect is `content min_rect + margins` (`panel.rs:286`,
  `frame.rs:313`) with **no post-clamp**; `cursor.min.x = rect.max.x`
  (`panel.rs:293`) and `allocate_left_panel` (`panel.rs:391`) hand that grown
  rect to the CentralPanel. gap = widest child + 16 − 280.
- `ui.set_max_width()` is advisory: an oversized allocation is unioned back
  into min_rect AND max_rect (`ui.rs:1268` → `layout.rs:49-52`). The clip only
  made overflow invisible.
- Amplifier: `ScrollArea::vertical().auto_shrink([false,false])` hits the
  `(false,false) => inner_size.x.max(content_size.x)` arm
  (`scroll_area.rs:902-913`) — it propagates the widest row out to the panel,
  even rows scrolled offscreen.

Measured with real font metrics (headless `egui::Context::run`): element list
with a long filename = **634px**, picture inspector = **546px**, vs the 264px
budget. The overflow was filename-driven (`ui.button(format!("… {}", file))`
in a non-wrapping horizontal row), which is why the gap size varied between
sessions.

## The fix — two layers

1. **Structural cap** — `show_width_capped` in `src/ui/components/mod.rs`:
   `new_child(UiBuilder::max_rect)` (allocates nothing in the parent,
   `ui.rs:242-246`) + `advance_cursor_after_rect(exact rect)`. The parent
   advances by exactly the cap no matter what children allocate. Wired at the
   sidebar root in `src/app/mod.rs`. NOT `allocate_ui_with_layout` — that
   re-allocates the grown child rect (`ui.rs:1400-1413`).
2. **Rows resized to actually fit** (a capped-but-overflowing row would be
   clipped into dead, unclickable buttons): filename/text snippets truncated
   (`file_label` caps at 22 chars), media inspector headers moved the filename
   to its own wrapping line, the ~410px picture action row split in two,
   background-style/move-delete/size-preset/months/year-start rows shortened
   or tightened, ComboBox/TextEdit widths corrected for their own padding.
   All in `src/ui/slide_bin.rs`.

## Proof

`tests/sidebar_width_tests.rs` (new):
- Red-first: both budget tests FAILED on the old rows (634px / 546px).
- `formatting_tab_element_list_fits_the_budget` + 
  `every_element_inspector_fits_the_budget` (all 5 element types, worst-case
  74-char filename): green after the row fixes.
- `width_cap_survives_a_pathological_child`: sabotage control — a deliberate
  400px button (positive control asserts it really measures ≥399px) must not
  grow the capped parent past 265. Green. This is the guard that keeps any
  future over-wide row from reopening the gap.

Full suite: **100 tests, 0 failures**, build clean.

## Open items

- On-screen confirmation is the user's (Win exe / Xvfb limit): sidebar flush
  against the preview; picture-inspector buttons all visible AND clickable;
  a long-named import opens no gap.
- The row-level budget test only covers the Formatting tab; Transitions/Slides
  tabs are protected by the structural cap but their rows are not measured.
