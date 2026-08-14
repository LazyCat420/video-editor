use egui::{Color32, FontFamily, FontId, Stroke, TextStyle, Visuals};
use std::cell::RefCell;

/// Which color scheme the whole program uses.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThemeKind {
    Dark,
    Light,
}

impl ThemeKind {
    pub fn label(&self) -> &'static str {
        match self {
            ThemeKind::Dark => "Dark (Easy on Eyes)",
            ThemeKind::Light => "Light (Bright)",
        }
    }

    pub fn all() -> [ThemeKind; 2] {
        [ThemeKind::Dark, ThemeKind::Light]
    }
}

/// The set of colors used across the app. Kept in one place so swapping theme
/// changes every surface (cards, tracks, text, accents) consistently.
#[derive(Clone, Copy)]
struct Palette {
    bg_app: Color32,
    bg_panel: Color32,
    bg_card: Color32,
    bg_hover: Color32,
    accent_blue: Color32,
    accent_cyan: Color32,
    accent_green: Color32,
    accent_yellow: Color32,
    accent_red: Color32,
    track_video_bg: Color32,
    track_video_border: Color32,
    track_audio_bg: Color32,
    track_audio_border: Color32,
    clip_video_bg: Color32,
    clip_video_sel: Color32,
    clip_audio_bg: Color32,
    clip_audio_sel: Color32,
    playhead: Color32,
    node: Color32,
    node_hover: Color32,
    env_line: Color32,
    waveform: Color32,
    text_primary: Color32,
    text_secondary: Color32,
    text_muted: Color32,
}

impl Palette {
    fn dark() -> Self {
        Self {
            bg_app: Color32::from_rgb(18, 20, 26),
            bg_panel: Color32::from_rgb(26, 29, 38),
            bg_card: Color32::from_rgb(35, 39, 51),
            bg_hover: Color32::from_rgb(46, 52, 68),
            accent_blue: Color32::from_rgb(37, 99, 235),
            accent_cyan: Color32::from_rgb(6, 182, 212),
            accent_green: Color32::from_rgb(16, 185, 129),
            accent_yellow: Color32::from_rgb(245, 158, 11),
            accent_red: Color32::from_rgb(239, 68, 68),
            track_video_bg: Color32::from_rgb(24, 43, 78),
            track_video_border: Color32::from_rgb(59, 130, 246),
            track_audio_bg: Color32::from_rgb(20, 56, 46),
            track_audio_border: Color32::from_rgb(16, 185, 129),
            clip_video_bg: Color32::from_rgb(37, 99, 235),
            clip_video_sel: Color32::from_rgb(96, 165, 250),
            clip_audio_bg: Color32::from_rgb(5, 150, 105),
            clip_audio_sel: Color32::from_rgb(52, 211, 153),
            playhead: Color32::from_rgb(255, 68, 68),
            node: Color32::from_rgb(251, 191, 36),
            node_hover: Color32::from_rgb(253, 224, 71),
            env_line: Color32::from_rgb(251, 191, 36),
            waveform: Color32::from_rgba_premultiplied(167, 243, 208, 140),
            text_primary: Color32::from_rgb(249, 250, 251),
            text_secondary: Color32::from_rgb(209, 213, 219),
            text_muted: Color32::from_rgb(156, 163, 175),
        }
    }

    fn light() -> Self {
        Self {
            bg_app: Color32::from_rgb(232, 236, 243),
            bg_panel: Color32::from_rgb(246, 248, 251),
            bg_card: Color32::from_rgb(255, 255, 255),
            bg_hover: Color32::from_rgb(219, 226, 235),
            accent_blue: Color32::from_rgb(37, 99, 235),
            accent_cyan: Color32::from_rgb(8, 145, 178),
            accent_green: Color32::from_rgb(5, 150, 105),
            accent_yellow: Color32::from_rgb(202, 138, 4),
            accent_red: Color32::from_rgb(220, 38, 38),
            track_video_bg: Color32::from_rgb(210, 228, 252),
            track_video_border: Color32::from_rgb(59, 130, 246),
            track_audio_bg: Color32::from_rgb(197, 244, 222),
            track_audio_border: Color32::from_rgb(16, 185, 129),
            clip_video_bg: Color32::from_rgb(147, 197, 253),
            clip_video_sel: Color32::from_rgb(96, 165, 250),
            clip_audio_bg: Color32::from_rgb(60, 202, 154),
            clip_audio_sel: Color32::from_rgb(16, 162, 118),
            playhead: Color32::from_rgb(220, 38, 38),
            node: Color32::from_rgb(202, 138, 4),
            node_hover: Color32::from_rgb(217, 119, 6),
            env_line: Color32::from_rgb(202, 138, 4),
            waveform: Color32::from_rgba_premultiplied(6, 95, 70, 130),
            text_primary: Color32::from_rgb(17, 24, 39),
            text_secondary: Color32::from_rgb(51, 65, 85),
            text_muted: Color32::from_rgb(100, 116, 139),
        }
    }
}

struct ThemeState {
    kind: ThemeKind,
    palette: Palette,
    font_scale: f32,
}

impl Default for ThemeState {
    fn default() -> Self {
        Self {
            kind: ThemeKind::Dark,
            palette: Palette::dark(),
            font_scale: 1.0,
        }
    }
}

thread_local! {
    static STATE: RefCell<ThemeState> = RefCell::new(ThemeState::default());
}

fn with_palette<T>(f: impl FnOnce(&Palette) -> T) -> T {
    STATE.with(|s| f(&s.borrow().palette))
}

pub struct AppTheme;

impl AppTheme {
    /// The font size the user picked (as a zoom multiplier on the whole UI).
    pub fn font_scale_now() -> f32 {
        STATE.with(|s| s.borrow().font_scale)
    }

    /// Theme currently active.
    pub fn theme_now() -> ThemeKind {
        STATE.with(|s| s.borrow().kind)
    }

    /// Swap the palette + font zoom and rebuild every egui style that depends on them.
    pub fn configure(ctx: &egui::Context, kind: ThemeKind, font_scale: f32) {
        let palette = match kind {
            ThemeKind::Dark => Palette::dark(),
            ThemeKind::Light => Palette::light(),
        };
        STATE.with(|s| {
            *s.borrow_mut() = ThemeState {
                kind,
                palette,
                font_scale,
            };
        });
        Self::apply(ctx);
    }

    fn apply(ctx: &egui::Context) {
        with_palette(|pal| {
            let light = STATE.with(|s| s.borrow().kind == ThemeKind::Light);
            let font_scale = STATE.with(|s| s.borrow().font_scale);

            let mut visuals = if light {
                Visuals::light()
            } else {
                Visuals::dark()
            };
            visuals.panel_fill = pal.bg_panel;
            visuals.window_fill = pal.bg_panel;
            visuals.faint_bg_color = pal.bg_app;
            visuals.extreme_bg_color = pal.bg_app;

            visuals.widgets.noninteractive.bg_fill = pal.bg_panel;
            visuals.widgets.noninteractive.fg_stroke =
                Stroke::new(1.0, pal.text_primary);

            visuals.widgets.inactive.bg_fill = pal.bg_card;
            visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, pal.text_primary);
            visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);

            visuals.widgets.hovered.bg_fill = pal.bg_hover;
            visuals.widgets.hovered.fg_stroke = Stroke::new(1.2, pal.text_primary);
            visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);

            visuals.widgets.active.bg_fill = pal.accent_blue;
            visuals.widgets.active.fg_stroke = Stroke::new(1.2, Color32::WHITE);
            visuals.widgets.active.rounding = egui::Rounding::same(6.0);

            ctx.set_visuals(visuals);

            let mut style = (*ctx.style()).clone();
            style.spacing.item_spacing = egui::vec2(10.0, 8.0);
            style.spacing.button_padding = egui::vec2(14.0, 8.0);
            // Sliders a little smaller throughout the program.
            style.spacing.slider_width = 90.0;
            // Rail thinner than default (8 -> 4).
            style.spacing.slider_rail_height = 4.0;
            style.spacing.interact_size = egui::vec2(24.0, 18.0);

            let mut text_styles = std::collections::BTreeMap::new();
            text_styles.insert(TextStyle::Heading, FontId::new(20.0, FontFamily::Proportional));
            text_styles.insert(TextStyle::Button, FontId::new(15.0, FontFamily::Proportional));
            text_styles.insert(TextStyle::Body, FontId::new(15.0, FontFamily::Proportional));
            text_styles.insert(TextStyle::Monospace, FontId::new(15.0, FontFamily::Monospace));
            text_styles.insert(TextStyle::Small, FontId::new(13.0, FontFamily::Proportional));
            style.text_styles = text_styles;
            ctx.set_style(style);

            // Zoom the whole UI (text + controls) by the user's chosen text size.
            ctx.set_pixels_per_point(font_scale.max(0.6).min(1.8));
        });
    }

    // ---- Palette accessors (kept as methods so every call site works unchanged) ----
    pub fn bg_app() -> Color32 { with_palette(|p| p.bg_app) }
    pub fn bg_panel() -> Color32 { with_palette(|p| p.bg_panel) }
    pub fn bg_card() -> Color32 { with_palette(|p| p.bg_card) }
    pub fn bg_hover() -> Color32 { with_palette(|p| p.bg_hover) }
    pub fn accent_blue() -> Color32 { with_palette(|p| p.accent_blue) }
    pub fn accent_cyan() -> Color32 { with_palette(|p| p.accent_cyan) }
    pub fn accent_green() -> Color32 { with_palette(|p| p.accent_green) }
    pub fn accent_yellow() -> Color32 { with_palette(|p| p.accent_yellow) }
    pub fn accent_red() -> Color32 { with_palette(|p| p.accent_red) }
    pub fn track_video_bg() -> Color32 { with_palette(|p| p.track_video_bg) }
    pub fn track_video_border() -> Color32 { with_palette(|p| p.track_video_border) }
    pub fn track_audio_bg() -> Color32 { with_palette(|p| p.track_audio_bg) }
    pub fn track_audio_border() -> Color32 { with_palette(|p| p.track_audio_border) }
    pub fn clip_video_bg() -> Color32 { with_palette(|p| p.clip_video_bg) }
    pub fn clip_video_selected() -> Color32 { with_palette(|p| p.clip_video_sel) }
    pub fn clip_audio_bg() -> Color32 { with_palette(|p| p.clip_audio_bg) }
    pub fn clip_audio_selected() -> Color32 { with_palette(|p| p.clip_audio_sel) }
    pub fn playhead_color() -> Color32 { with_palette(|p| p.playhead) }
    pub fn node_color() -> Color32 { with_palette(|p| p.node) }
    pub fn node_hover_color() -> Color32 { with_palette(|p| p.node_hover) }
    pub fn envelope_line_color() -> Color32 { with_palette(|p| p.env_line) }
    pub fn waveform_color() -> Color32 { with_palette(|p| p.waveform) }
    pub fn text_primary() -> Color32 { with_palette(|p| p.text_primary) }
    pub fn text_secondary() -> Color32 { with_palette(|p| p.text_secondary) }
    pub fn text_muted() -> Color32 { with_palette(|p| p.text_muted) }
}
