use egui::{Color32, Stroke, Visuals};

pub struct AppTheme;

impl AppTheme {
    // Backgrounds
    pub const BG_APP: Color32 = Color32::from_rgb(18, 20, 26);
    pub const BG_PANEL: Color32 = Color32::from_rgb(26, 29, 38);
    pub const BG_CARD: Color32 = Color32::from_rgb(35, 39, 51);
    pub const BG_HOVER: Color32 = Color32::from_rgb(46, 52, 68);

    // Accents
    pub const ACCENT_BLUE: Color32 = Color32::from_rgb(59, 130, 246);
    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(6, 182, 212);
    pub const ACCENT_GREEN: Color32 = Color32::from_rgb(16, 185, 129);
    pub const ACCENT_YELLOW: Color32 = Color32::from_rgb(245, 158, 11);
    pub const ACCENT_RED: Color32 = Color32::from_rgb(239, 68, 68);

    // Timeline & Tracks
    pub const TRACK_VIDEO_BG: Color32 = Color32::from_rgb(22, 38, 70);
    pub const TRACK_VIDEO_BORDER: Color32 = Color32::from_rgb(37, 99, 235);
    pub const TRACK_AUDIO_BG: Color32 = Color32::from_rgb(18, 50, 42);
    pub const TRACK_AUDIO_BORDER: Color32 = Color32::from_rgb(16, 185, 129);

    pub const CLIP_VIDEO_BG: Color32 = Color32::from_rgb(30, 58, 138);
    pub const CLIP_VIDEO_SELECTED: Color32 = Color32::from_rgb(59, 130, 246);
    pub const CLIP_AUDIO_BG: Color32 = Color32::from_rgb(6, 78, 59);
    pub const CLIP_AUDIO_SELECTED: Color32 = Color32::from_rgb(16, 185, 129);

    // Playhead & Keyframes
    pub const PLAYHEAD_COLOR: Color32 = Color32::from_rgb(239, 68, 68);
    pub const NODE_COLOR: Color32 = Color32::from_rgb(245, 158, 11);
    pub const NODE_HOVER_COLOR: Color32 = Color32::from_rgb(251, 191, 36);
    pub const ENVELOPE_LINE_COLOR: Color32 = Color32::from_rgb(245, 158, 11);
    pub const WAVEFORM_COLOR: Color32 = Color32::from_rgba_premultiplied(52, 211, 153, 90);

    // Typography
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(243, 244, 246);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(156, 163, 175);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(107, 114, 128);

    pub fn apply(ctx: &egui::Context) {
        let mut visuals = Visuals::dark();
        visuals.panel_fill = Self::BG_PANEL;
        visuals.window_fill = Self::BG_PANEL;
        visuals.faint_bg_color = Self::BG_APP;
        visuals.extreme_bg_color = Self::BG_APP;

        visuals.widgets.noninteractive.bg_fill = Self::BG_PANEL;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);

        visuals.widgets.inactive.bg_fill = Self::BG_CARD;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);
        visuals.widgets.inactive.rounding = egui::Rounding::same(4.0);

        visuals.widgets.hovered.bg_fill = Self::BG_HOVER;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        visuals.widgets.hovered.rounding = egui::Rounding::same(4.0);

        visuals.widgets.active.bg_fill = Self::ACCENT_BLUE;
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        visuals.widgets.active.rounding = egui::Rounding::same(4.0);

        ctx.set_visuals(visuals);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
        ctx.set_style(style);
    }
}
