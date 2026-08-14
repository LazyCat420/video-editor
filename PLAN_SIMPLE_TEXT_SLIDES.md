# Implementation Plan: Click-to-Place Text + PowerPoint-style Slide Editor

**Repo:** `video-editor` (Rust + egui, eframe 0.29)  
**Status:** Plan for review — **not to be implemented until approved.**

## Goal & Problem Analysis

Today there is no way to lay content out the way a slide needs:

1. **Text is a fixed-anchor enum** (`TextPosition: Center/TopHeader/BottomBanner/LowerThird`) — you cannot click the frame to place it or drag it. Preview and export both render at 4 preset spots.
2. **1 clip : 1 overlay** — `Clip.text_overlay: Option<TextOverlay>`. A clip cannot hold several text blocks, and there is **no model for a slide holding pictures + video + audio + text at once**.
3. **The Text panel is overbuilt** (`text_bin.rs`, 688 lines): two overlapping tabs, a Title-Card background-mode toggle, color swatches, opacity, shadow, duration slider — none of it what the user asked for.
4. **Font style only changes the export, not the preview.** `draw_text_overlay` maps every preset to egui's built-in Proportional family except Monospace, so Sans/Serif/Impact/Handwritten all look identical in preview and only differ once ffmpeg's `drawtext font=` runs. With 10 presets this must be fixed by bundling real fonts.
5. There is a **blank-slide / title-card primitive** (`Clip::new_title_card`, `is_title_card`, `title_card_bg`) but it can only hold one text overlay — not a collage.

**Desired behaviour (user, confirmed):**
- Add text **on top of a video/image clip** (normal clip case).
- Add a **blank slide** and put **text / audio / video / pictures all in the same slide simultaneously**, like PowerPoint.
- Text: type it, pick style / size / **bold / italic**, **click the frame to place it**, drag to move it.
- **Solid background optional** behind text, with **both** a **full-width banner** and a **tight box** as choices.
- **10 font styles** (keep the existing 5, add 5).

---

## Locked Design Decisions (from user)

| Topic | Decision taken |
|---|---|
| Text-on-clip | Text attaches to the clip under the playhead and renders **on top of the video/image**. |
| Blank slide | A blank slide is a clip with its own background (solid color or picture) that can hold **several elements at once** — text, picture, video, audio. |
| Placement | Free normalized `x/y` (0..1). **Click on the preview frame to place**; drag to move. Works for text and for picture/video elements. |
| Fonts | Keep the 5 presets; **add 5 more = 10 total**. |
| Solid background | Text background is optional with **two styles**: full-width banner **and** tight rounded box. |
| End state | PowerPoint-like slide: one container, many placed elements, composited in preview and export. |

---

## Part 1 — Core model: every clip is a base layer + a list of elements

The cleanest way to get both "text on a clip" and "blank slides with everything" is one unified model: **every clip has a base layer and an ordered list of placed elements.**

### 1.1 `ClipBase` — replace the title-card booleans (`src/core/clip.rs`)

```rust
pub enum ClipBase {
    Video(PathBuf),   // normal video clip (has_video == true)
    Image(PathBuf),   // still image clip
    Solid(Color32),   // blank slide / title-card background
}
```

Replace the three fields `has_video`, `is_title_card`, `title_card_bg`, `source_path`-as-string with `base: ClipBase`. `new_title_card` becomes `new_blank_slide(id, track, duration, bg) -> Clip` using `ClipBase::Solid` (or an image chosen later as the slide's own background).

### 1.2 `SlideElement` — one element placed in a slide (`src/core/slide.rs`, new)

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SlideElement {
    Text    { overlay: TextOverlay },
    Picture { path: PathBuf, x: f32, y: f32, w: f32, h: f32 }, // normalized 0..1, w/h = frac of frame
    Video   { path: PathBuf, x: f32, y: f32, w: f32, h: f32 },
    Audio   { path: PathBuf, volume: f32 },
}
```

- Layering is **Vec order** (index 0 = bottom, last = top); the UI exposes up/down. No separate `z` field.
- `Text`'s `overlay` already carries its own `x/y/size` (Part 2). `Picture`/`Video` carry their own `x/y/w/h` because they are boxes, not glyph anchors.

### 1.3 Replace `text_overlay: Option<TextOverlay>` with `elements: Vec<SlideElement>`

`Clip.elements: Vec<SlideElement>` consolidates text-on-clip and blank-slide content into one list on **every** clip. A video clip typically has text elements on top of `ClipBase::Video`; a blank slide has any mix over `ClipBase::Solid`.

**Back-compat:** on `.vproj` load, map the old `text_overlay` (if any) into `elements = [Text{overlay}]` and old `is_title_card`/`title_card_bg` into `ClipBase`. Do this once in the project deserializer (`src/core/project.rs`); no alias lives past load.

---

## Part 2 — Free placement for text and boxes

### 2.1 Text: `x/y` replaces `TextPosition` (`src/core/text_overlay.rs`)

- Remove `position: TextPosition`; add
  ```rust
  #[serde(default = "default_center")] pub x: f32, // 0.5, anchor = center of the text box
  #[serde(default = "default_center")] pub y: f32, // 0.5
  ```
- Keep `font_family, font_size, is_bold, is_italic, is_all_caps, alignment, text_color, box_style, box_opacity, show_shadow`. The new background choice maps to `box_style` (Part 4).

### 2.2 Preview: render & interact at `x/y` (`src/ui/preview_player.rs`)

- `draw_text_overlay` (currently `preview_player.rs:180-303`) anchors at `rect.min + (overlay.x*rect.w, overlay.y*rect.h)` instead of the `TextPosition` match.
- Replace `Sense::click()` (`preview_player.rs:42`) with `Sense::click_and_drag()`; return new `PlayerAction`s:
  - `PlaceElement { index }` — when a placement tool is armed and the preview is clicked, place at the normalized click point.
  - `MoveElement { index, x, y }` — hit-test the element box and drag to update `x/y`.
- Draw picture/video elements (Part 3) between the base frame and the text elements in Vec order.

### 2.3 Export: `drawtext` at `x/y` (`src/export/filter_graph.rs`)

In `build_drawtext_filter` (`filter_graph.rs:430-490`) replace the `TextPosition` match with center-anchored expressions:
```rust
x = format!("(w-text_w)/2 + ({} - 0.5)*w", overlay.x)
y = format!("(h-text_h)/2 + ({} - 0.5)*h", overlay.y)
```
Multi-line y-offset logic already exists and stays.

---

## Part 3 — The Slide Builder (blank slide with everything)

Blank slide = `ClipBase::Solid` (or a chosen picture) + `elements: Vec<SlideElement>`. Same model powers "text on a video clip"; the Slider Builder just exposes the fuller palette.

### 3.1 Builder UI (`src/ui/slide_bin.rs`, new — replaces `text_bin.rs`'s two tabs)

A single panel with:
1. **Add Text** — text box, font style/size/bold/italic, color, background style (Part 4). Armed → click the preview to place at that point (goes into `elements` as `Text`).
2. **Add Picture** — pick a file; place by clicking the preview (goes in as `Picture`), then drag to position and drag a corner to size.
3. **Add Video** — pick a file; same box workflow as Picture.
4. **Add Audio** — pick a file; becomes an `Audio` element on the slide (mixes over the slide duration).
5. **Background** — the slide's own `ClipBase`: Solid color or a chosen picture (reuses the current title-card background picker — kept, but as slide background, not a separate card concept).
6. **Element list** — current placeholder thumbnails/labels with `↑/↓` (reorder), `Delete`; clicking an element selects it for drag/resize.

The old TitleCardBuilder/TextOnClip tab split is deleted. `TextBinAction` becomes `SlideBinAction` with `AddElement(SlideElement)`, `UpdateElement{idx, SlideElement}`, `RemoveElement{idx}`, `ReorderElement{idx, dir}`, `SetBackground(ClipBase)`.

### 3.2 App wiring (`src/app.rs`)

- Replace the `TextBinAction` match (`app.rs:820-878`) with `SlideBinAction`. The same mutations edit `clip.elements` whether the clip is a video clip or a blank slide — one code path.
- Add a `ResolveTargetClip()` helper: the clip under the playhead, or a new blank slide inserted there. **Text on a plain video/image clip → attaches to that clip (`ClipBase::Video/Image`, elements get `Text`)**, as the user requested. If nothing is under the playhead, insert a blank slide and target it.
- `get_active_text_overlay` becomes `active_elements() -> Vec<&SlideElement>`; add `update_active_element(idx, new)`.
- Place/drag `PlayerAction`s route into these helpers.

### 3.3 Composition preview

For the clip under the playhead, draw in order: base frame → for each element (Picture texture, Video frame texture at `x/y/w/h`) → for each Text overlay (scaled box/glyphs). Reuses `draw_text_overlay` and the existing frame-texture path. Multiple textures = one `ui.painter_at(rect)` pass, so z-order is free.

### 3.4 Composition export (`src/export/filter_graph.rs`)

For a slide clip emit, in order:
- base → the input video/image, else `color=c=…` / `movie=…` for the picture background;
- each `Picture`/`Video` → a chained `overlay=x=…:y=…:w=…:h=…` (scaled via `scale`) — the current single-drawtext path generalizes to N `overlay` filters;
- each `Text` → the existing `drawtext` (now at `x/y`);
- all `Audio` elements → an `amix` over the slide's duration, mixed with the clip's own audio track (`filter_graph.rs` already builds multi-input `amix`).

---

## Part 4 — Text formatting details

### 4.1 10 font presets (`src/core/text_overlay.rs`)

Keep the 5; add 5. Each preset maps to **both** an egui preview font (an embedded TTF) and an ffmpeg `drawtext font=` name. Target is Windows, so export names use standard Windows fonts already installed so videos render on any PC:

| Preset | Preview (embedded TTF) | Export (ffmpeg font) |
|---|---|---|
| SansSerif | ~Noto Sans | Arial |
| Serif | ~Roboto Serif | Times New Roman |
| Monospace | ~JetBrains Mono | Courier New |
| Impact | ~Anton | Impact |
| Handwritten | ~Caveat | Comic Sans MS |
| **Condensed** (new) | ~Oswald | Arial Narrow / Oswald |
| **Display** (new) | ~Bebas Neue | Cooper Black |
| **VintageSerif** (new) | ~Alegreya | Georgia |
| **Script** (new) | ~Great Vibes | Brush Script MT |
| **Futuristic** (new) | ~Rajdhani | Century Gothic |

- `FontFamilyPreset::all()` → 10.
- **Preview fonts are the real fix:** bundle ~the 10 TTFs (silent-OFL fonts) and load them into egui's `FontDefinitions` at startup (`src/ui/theme.rs`), then `draw_text_overlay`'s `FontId` selects the per-preset family (not just Proportional/Monospace). Now Font Style visibly changes in the preview.
- `ffmpeg_font_name()` returns the export font. If a name is missing on the export machine, ffmpeg falls back to a default (acceptable; export runs on the same Windows box as the preview, so names are present).

### 4.2 Background: banner AND tight box (`src/core/text_overlay.rs`, `src/ui/slide_bin.rs`)

Keep `TextBoxStyle { None, TranslucentBox, SolidBanner }` and expose it as a 3-way choice in the Text panel:
- **None** — no background.
- **Tight Box** (`TranslucentBox`) — rounded box just behind the letters (preview `preview_player.rs:box_rect`, export `box=1:boxcolor=black@…:boxborderw=16`).
- **Full Banner** (`SolidBanner`) — full-width strip (preview `SolidBanner` rect, export `box=1:…:boxborderw=24`).
A single on/off toggle becomes these three options. `box_opacity`/`show_shadow` stay in the model, hidden in the UI (defaults: 0.65 / true).

### 4.3 The Text panel is now one flat form (`src/ui/slide_bin.rs`)

Text box → font combo (10) → size slider → Bold / Italic → color swatches → background (None / Tight Box / Full Banner) → **Add to Slide** (arms click-to-place). The ALL-CAPS toggle, opacity slider, and shadow toggle are removed from the visible UI (fields remain in the model, unused by default).

---

## Build order (all in scope; sequence is a dependency order, not a deferral)

1. **Model** — `ClipBase`, `SlideElement`, `elements: Vec<SlideElement>` replacing `text_overlay`; `TextPosition`→`x/y`; 10 presets; `box_style` 3-way. Back-compat loader in `project.rs`. (`core/`)
2. **Preview/export at x/y** — `draw_text_overlay`, `build_drawtext_filter`; box rendering for both banner and tight box. (`preview_player.rs`, `filter_graph.rs`)
3. **Text panel simplify** — flat form, still emitting a single text element. (`ui/slide_bin.rs`)
4. **Embed preview fonts** — load 10 TTFs into egui; fix per-preset `FontId`. (`ui/theme.rs`, `text_overlay.rs`)
5. **Slide builder** — Add Text/Picture/Video/Audio, background picker, element list (reorder/delete), click-to-place + drag/resize. (`ui/slide_bin.rs`, `app.rs`)
6. **Composition preview + export** — draw/mix all elements (`overlay` chain + `drawtext` + `amix`). (`preview_player.rs`, `filter_graph.rs`)

## Open Questions (only what blocks implementation)

1. **Blank-slide default duration** — when a blank slide is inserted with no media, how long? (Proposed: 4s, user-adjustable in the panel.)
2. **Picture on a plain video clip** — allowed (timescale supports it) or only on blank slides? (Proposed: allowed anywhere — one unified model.)
3. **Slide audio in export** — should slide `Audio` elements also obey a simple volume slider per element? (Proposed: add a per-element volume slider; cheap.)
4. **Embedded font licensing/size** — OK to ship ~10 OFL-licensed TTFs (~1–2 MB) inside the app so preview styles are real?

## Verification Plan

1. `cargo build` in `video-editor`.
2. **Text on clip:** add text over a video clip → renders on top; click the frame to place; drag moves it live.
3. **Blank slide collage:** blank slide + add a picture, a video, text, and an audio element → all visible/audible at once; drag/resize/reorder each; z-order respects list order.
4. **Backgrounds:** None / Tight Box / Full Banner each render correctly in preview and export.
5. **Fonts:** all 10 presets look distinct in preview and match export (sample export frame).
6. **Export parity:** exported slide composes base + pictures + videos + text + audio matching the preview layout.
7. **Back-compat:** a pre-change `.vproj` (old `TextPosition`, single `text_overlay`, title-card bools) loads and appears centered as expected.
8. `cargo test` — update overlay-position fixtures in `filter_graph.rs`; add tests for `SlideElement` reorder and clip-base mapping.
