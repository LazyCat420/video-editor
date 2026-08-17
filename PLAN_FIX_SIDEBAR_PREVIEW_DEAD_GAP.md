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
