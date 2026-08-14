use crate::core::text_overlay::{SlideBackground, SlideElement};
use crate::core::timeline::Timeline;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MediaAsset {
    pub id: u64,
    pub name: String,
    pub path: PathBuf,
    pub duration_secs: f64,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub has_video: bool,
    pub has_audio: bool,
    pub proxy_path: Option<PathBuf>,
    pub peak_path: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub timeline: Timeline,
    pub media_assets: Vec<MediaAsset>,
    pub export_width: u32,
    pub export_height: u32,
    pub export_fps: f64,
    pub export_bitrate_kbps: u32,
    pub export_audio_bitrate_kbps: u32,
    pub proxy_enabled: bool,
    pub next_asset_id: u64,
}

impl Default for Project {
    fn default() -> Self {
        Self::new("Untitled Project".to_string())
    }
}

impl Project {
    pub fn new(name: String) -> Self {
        Self {
            name,
            timeline: Timeline::new(30.0),
            media_assets: Vec::new(),
            export_width: 1920,
            export_height: 1080,
            export_fps: 30.0,
            export_bitrate_kbps: 8000,
            export_audio_bitrate_kbps: 192,
            proxy_enabled: true,
            next_asset_id: 1,
        }
    }

    pub fn next_asset_id(&mut self) -> u64 {
        let id = self.next_asset_id;
        self.next_asset_id += 1;
        id
    }

    pub fn add_asset(&mut self, asset: MediaAsset) -> u64 {
        let id = asset.id;
        self.media_assets.push(asset);
        id
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, json)
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut project: Project = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        migrate_legacy_clips(&mut project.timeline);
        Ok(project)
    }
}

/// Fold pre-slide clip fields into the new slide model so old project files load cleanly.
/// Legacy `text_overlay` becomes a Text element and the title-card bools become the slide
/// `background`. Runs once on load, so a re-save is idempotent and clean.
fn migrate_legacy_clips(timeline: &mut Timeline) {
    for track in &mut timeline.tracks {
        for clip in &mut track.clips {
            // Title card -> slide background (solid or picture), and it is no longer a
            // streamed video clip.
            if clip.is_title_card || clip.title_card_bg.is_some() {
                let bg = clip
                    .title_card_bg
                    .clone()
                    .map(|b| match b {
                        crate::core::text_overlay::TitleCardBackground::SolidColor(c) => {
                            SlideBackground::Solid(c)
                        }
                        crate::core::text_overlay::TitleCardBackground::Picture(p) => {
                            SlideBackground::Picture(p)
                        }
                    })
                    .unwrap_or_else(|| SlideBackground::Solid(egui::Color32::from_rgb(18, 18, 24)));
                clip.background = Some(bg);
                clip.source_path = PathBuf::new();
                clip.has_video = false;
                clip.is_title_card = false;
                clip.title_card_bg = None;
            }
            // Old single text overlay -> a placed Text element.
            if let Some(overlay) = clip.text_overlay.take() {
                if !overlay.text.trim().is_empty() {
                    clip.elements.push(SlideElement::Text(overlay));
                }
            }
        }
    }
}
