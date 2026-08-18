use crate::core::clip::Clip;
use crate::core::effects::SlideEffectKind;
use crate::core::stickers::{StickerCatalog, StickerCategory, StickerItem};
use crate::core::timeline::Timeline;
use crate::core::transition::{Transition, TransitionKind};
use crate::ui::theme::AppTheme;
use egui::{Button, Color32, Frame, Id, RichText, Rounding, ScrollArea, Ui, Vec2};
use std::path::PathBuf;

pub struct EffectsAndTransitionsBinView;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionSlot {
    In,  // Beginning of clip (left edge)
    Out, // End of clip (right edge)
}

pub enum EffectsAndTransitionsAction {
    None,
    SetTransition {
        clip_id: u64,
        slot: TransitionSlot,
        transition: Option<Transition>,
    },
    ToggleEffect {
        clip_id: u64,
        kind: SlideEffectKind,
    },
    ClearEffects {
        clip_id: u64,
    },
    AddSticker {
        path: PathBuf,
        name: String,
        category: StickerCategory,
    },
}

// Backwards compatibility alias
pub type TransitionBinAction = EffectsAndTransitionsAction;
pub type TransitionBinView = EffectsAndTransitionsBinView;

static STICKER_IMAGE_CACHE: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<String, Option<egui::ColorImage>>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

impl EffectsAndTransitionsBinView {
    fn get_or_load_thumbnail(
        ctx: &egui::Context,
        path: &std::path::Path,
        id: &str,
    ) -> Option<egui::TextureHandle> {
        let tex_id = egui::Id::new(format!("stk_tex_{}", id));
        let cached = ctx.data(|d| d.get_temp::<egui::TextureHandle>(tex_id));
        if let Some(t) = cached {
            return Some(t);
        }

        // Check persistent in-memory decoded ColorImage cache to avoid repeated disk reads
        let color_img_opt = {
            let mut cache = STICKER_IMAGE_CACHE.lock().unwrap();
            if let Some(img) = cache.get(id) {
                img.clone()
            } else {
                let loaded = if path.exists() {
                    if let Ok(dyn_img) = image::open(path) {
                        let rgba = dyn_img.to_rgba8();
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        let pixels = rgba.into_raw();
                        Some(egui::ColorImage::from_rgba_unmultiplied(size, &pixels))
                    } else {
                        None
                    }
                } else {
                    None
                };
                cache.insert(id.to_string(), loaded.clone());
                loaded
            }
        };

        if let Some(color_img) = color_img_opt {
            let label = format!("stk_tex_lbl_{}", id);
            let t = ctx.load_texture(label, color_img, egui::TextureOptions::LINEAR);
            ctx.data_mut(|d| d.insert_temp(tex_id, t.clone()));
            return Some(t);
        }

        None
    }

    pub fn render(ui: &mut Ui, timeline: &mut Timeline) -> EffectsAndTransitionsAction {
        let mut action = EffectsAndTransitionsAction::None;

        // Persistent slot selection in egui memory
        let slot_id = Id::new("effects_and_trans_selected_slot");
        let mut selected_slot: TransitionSlot = ui
            .data_mut(|d| d.get_temp(slot_id))
            .unwrap_or(TransitionSlot::In);

        // Persistent sticker category filter (default to Halloween for compact initial load)
        let cat_filter_id = Id::new("effects_and_trans_sticker_cat");
        let mut selected_cat: StickerCategory = ui
            .data_mut(|d| d.get_temp(cat_filter_id))
            .unwrap_or(StickerCategory::Halloween);

        // Persistent collapsible header states (both DEFAULT UNCOLLAPSED / OPEN = true)
        let effects_open_id = Id::new("effects_section_uncollapsed_v1");
        let mut effects_open: bool = ui.data_mut(|d| d.get_temp(effects_open_id)).unwrap_or(true);

        let stickers_open_id = Id::new("stickers_section_uncollapsed_v1");
        let mut stickers_open: bool = ui.data_mut(|d| d.get_temp(stickers_open_id)).unwrap_or(true);

        let trans_open_id = Id::new("trans_section_uncollapsed_v1");
        let mut trans_open: bool = ui.data_mut(|d| d.get_temp(trans_open_id)).unwrap_or(true);

        // Find the currently selected clip (if any)
        let selected_clip: Option<Clip> = timeline.get_selected_clip().cloned();

        ui.vertical(|ui| {
            ui.add_space(4.0);

            // Context / Selection Status Banner
            Frame::none()
                .fill(AppTheme::bg_card())
                .rounding(Rounding::same(8.0))
                .inner_margin(8.0)
                .show(ui, |ui| {
                    if let Some(clip) = &selected_clip {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("🎬 Active Slide:")
                                    .strong()
                                    .size(12.5)
                                    .color(AppTheme::accent_blue()),
                            );
                            ui.label(
                                RichText::new(&clip.name)
                                    .strong()
                                    .size(12.5)
                                    .color(Color32::WHITE),
                            );
                        });
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("ℹ No Slide Selected")
                                    .strong()
                                    .size(12.5)
                                    .color(AppTheme::text_secondary()),
                            );
                            ui.add_space(1.0);
                            ui.label(
                                RichText::new("Click a slide to customize celebration effects & transitions.")
                                    .size(11.0)
                                    .color(AppTheme::text_muted()),
                            );
                        });
                    }
                });

            ui.add_space(6.0);

            let row_w = ui.available_width();

            ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                // =========================================================
                // 1. CELEBRATION & SCREEN EFFECTS (DEFAULT UNCOLLAPSED)
                // =========================================================
                ui.horizontal(|ui| {
                    let arrow = if effects_open { "▼" } else { "▶" };
                    let header_text = format!("{} ✨ Celebration & Screen Effects", arrow);
                    let label = ui.add(
                        egui::Label::new(
                            RichText::new(header_text)
                                .strong()
                                .size(13.0)
                                .color(AppTheme::accent_yellow()),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if label.clicked() {
                        effects_open = !effects_open;
                        ui.data_mut(|d| d.insert_temp(effects_open_id, effects_open));
                    }
                });

                if effects_open {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("PowerPoint celebration animation overlays:")
                            .size(10.5)
                            .color(AppTheme::text_muted()),
                    );
                    ui.add_space(4.0);

                    for kind in SlideEffectKind::all() {
                        let is_active = selected_clip
                            .as_ref()
                            .map(|c| c.has_effect(*kind))
                            .unwrap_or(false);

                        if crate::ui::components::ActionRowCard::render(
                            ui,
                            kind.icon(),
                            kind.label(),
                            kind.description(),
                            is_active,
                            row_w,
                        ) {
                            if let Some(clip) = &selected_clip {
                                action = EffectsAndTransitionsAction::ToggleEffect {
                                    clip_id: clip.id,
                                    kind: *kind,
                                };
                            }
                        }
                        ui.add_space(3.0);
                    }

                    if let Some(clip) = &selected_clip {
                        if !clip.effects.is_empty() {
                            ui.add_space(2.0);
                            let clear_btn = Button::new(
                                RichText::new("🗑 Clear All Effects on Slide")
                                    .size(11.0)
                                    .color(Color32::from_rgb(255, 120, 120)),
                            )
                            .fill(AppTheme::bg_card())
                            .min_size(Vec2::new(row_w, 24.0));

                            if ui.add(clear_btn).clicked() {
                                action = EffectsAndTransitionsAction::ClearEffects { clip_id: clip.id };
                            }
                        }
                    }
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // =========================================================
                // 2. HOLIDAY & THEME STICKERS (DEFAULT UNCOLLAPSED)
                // =========================================================
                ui.horizontal(|ui| {
                    let arrow = if stickers_open { "▼" } else { "▶" };
                    let header_text = format!("{} 🎀 Holiday & Theme Stickers", arrow);
                    let label = ui.add(
                        egui::Label::new(
                            RichText::new(header_text)
                                .strong()
                                .size(13.0)
                                .color(AppTheme::accent_cyan()),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if label.clicked() {
                        stickers_open = !stickers_open;
                        ui.data_mut(|d| d.insert_temp(stickers_open_id, stickers_open));
                    }
                });

                if stickers_open {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Collage stickers for all holidays (Click to add to slide):")
                            .size(10.5)
                            .color(AppTheme::text_muted()),
                    );
                    ui.add_space(4.0);

                    // Category Filter Pills (2 rows of compact pills)
                    ui.horizontal_wrapped(|ui| {
                        for cat in StickerCategory::all_filter_categories() {
                            let is_sel = selected_cat == *cat;
                            let btn = Button::new(
                                RichText::new(cat.short_label())
                                    .size(10.0)
                                    .strong()
                                    .color(if is_sel { Color32::WHITE } else { AppTheme::text_secondary() }),
                            )
                            .fill(if is_sel { AppTheme::accent_blue() } else { AppTheme::bg_card() })
                            .min_size(Vec2::new(0.0, 20.0));

                            if ui.add(btn).clicked() {
                                selected_cat = *cat;
                                ui.data_mut(|d| d.insert_temp(cat_filter_id, selected_cat));
                            }
                        }
                    });

                    ui.add_space(6.0);

                    // Filtered stickers grid
                    let all_stickers = StickerCatalog::all_stickers();
                    let filtered: Vec<&StickerItem> = all_stickers
                        .iter()
                        .filter(|s| selected_cat == StickerCategory::All || s.category == selected_cat)
                        .collect();

                    let assets_dir = std::path::Path::new("assets");

                    for item in filtered {
                        let path = StickerCatalog::sticker_asset_path(assets_dir, &item.id);
                        let desc = format!("Category: {}", item.category.short_label());

                        // Fetch or load cached thumbnail texture
                        let tex = Self::get_or_load_thumbnail(ui.ctx(), &path, &item.id);

                        if crate::ui::components::ActionRowCard::render_with_image(
                            ui,
                            tex.as_ref(),
                            item.emoji,
                            &item.name,
                            &desc,
                            false,
                            row_w,
                        ) {
                            action = EffectsAndTransitionsAction::AddSticker {
                                path,
                                name: item.name.clone(),
                                category: item.category,
                            };
                        }
                        ui.add_space(3.0);
                    }
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // =========================================================
                // 3. VIDEO TRANSITIONS (DEFAULT UNCOLLAPSED)
                // =========================================================
                ui.horizontal(|ui| {
                    let arrow = if trans_open { "▼" } else { "▶" };
                    let header_text = format!("{} 🔄 Video Transitions", arrow);
                    let label = ui.add(
                        egui::Label::new(
                            RichText::new(header_text)
                                .strong()
                                .size(13.0)
                                .color(AppTheme::accent_green()),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if label.clicked() {
                        trans_open = !trans_open;
                        ui.data_mut(|d| d.insert_temp(trans_open_id, trans_open));
                    }
                });

                if trans_open {
                    ui.add_space(4.0);

                    // Placement toggle: Beginning (In) vs End (Out)
                    ui.label(
                        RichText::new("Apply Transition To:")
                            .size(11.5)
                            .color(AppTheme::text_secondary())
                            .strong(),
                    );
                    ui.columns(2, |cols| {
                        let in_active = selected_slot == TransitionSlot::In;
                        let out_active = selected_slot == TransitionSlot::Out;

                        let in_btn = Button::new(
                            RichText::new("⇤ Beginning (In)")
                                .size(11.5)
                                .strong()
                                .color(if in_active { Color32::WHITE } else { AppTheme::text_secondary() }),
                        )
                        .fill(if in_active { AppTheme::accent_blue() } else { AppTheme::bg_panel() })
                        .min_size(egui::vec2(cols[0].available_width(), 26.0));

                        if cols[0].add(in_btn).on_hover_text("Apply transition when the clip starts").clicked() {
                            selected_slot = TransitionSlot::In;
                            cols[0].data_mut(|d| d.insert_temp(slot_id, selected_slot));
                        }

                        let out_btn = Button::new(
                            RichText::new("End (Out) ⇥")
                                .size(11.5)
                                .strong()
                                .color(if out_active { Color32::WHITE } else { AppTheme::text_secondary() }),
                        )
                        .fill(if out_active { AppTheme::accent_blue() } else { AppTheme::bg_panel() })
                        .min_size(egui::vec2(cols[1].available_width(), 26.0));

                        if cols[1].add(out_btn).on_hover_text("Apply transition when the clip finishes").clicked() {
                            selected_slot = TransitionSlot::Out;
                            cols[1].data_mut(|d| d.insert_temp(slot_id, selected_slot));
                        }
                    });

                    ui.add_space(6.0);

                    let categories = [
                        (
                            "✨ Dissolves & Fades",
                            vec![
                                (TransitionKind::CrossFade, "Standard smooth blend between two clips"),
                                (TransitionKind::DipToBlack, "Fade out to pure black, then fade into next"),
                                (TransitionKind::DipToWhite, "Bright flash transition for dramatic cuts"),
                            ],
                        ),
                        (
                            "↔ Wipes",
                            vec![
                                (TransitionKind::WipeLeft, "Reveals by wiping in from right edge"),
                                (TransitionKind::WipeRight, "Reveals by wiping in from left edge"),
                                (TransitionKind::WipeUp, "Reveals by wiping up from bottom"),
                                (TransitionKind::WipeDown, "Reveals by wiping down from top"),
                            ],
                        ),
                        (
                            "🎬 Slides & Motion",
                            vec![
                                (TransitionKind::SlideLeft, "Pushes outgoing clip to the left"),
                                (TransitionKind::SlideRight, "Pushes outgoing clip to the right"),
                                (TransitionKind::SlideUp, "Pushes outgoing clip upward"),
                                (TransitionKind::SlideDown, "Pushes outgoing clip downward"),
                                (TransitionKind::SmoothLeft, "Soft feathered directional slide"),
                            ],
                        ),
                        (
                            "🔷 Shapes & Stylized",
                            vec![
                                (TransitionKind::CircleOpen, "Circular opening iris reveal from center"),
                                (TransitionKind::CircleClose, "Circular closing iris transition"),
                                (TransitionKind::Radial, "Clockwise clock sweep radial transition"),
                                (TransitionKind::ZoomIn, "Dramatic zoom into the new shot"),
                                (TransitionKind::SqueezeHorizontal, "Squeezes outgoing picture horizontally"),
                                (TransitionKind::Pixelate, "Retro pixelation mosaic dissolve"),
                            ],
                        ),
                    ];

                    for (cat_name, items) in &categories {
                        ui.label(
                            RichText::new(*cat_name)
                                .strong()
                                .size(12.0)
                                .color(AppTheme::accent_yellow()),
                        );
                        ui.add_space(2.0);

                        for (kind, desc) in items {
                            let is_active = selected_clip
                                .as_ref()
                                .and_then(|c| match selected_slot {
                                    TransitionSlot::In => c.start_transition(),
                                    TransitionSlot::Out => c.end_transition(),
                                })
                                .map(|t| t.kind == *kind)
                                .unwrap_or(false);

                            if crate::ui::components::ActionRowCard::render(
                                ui,
                                kind.icon(),
                                kind.label(),
                                desc,
                                is_active,
                                row_w,
                            ) {
                                if let Some(clip) = &selected_clip {
                                    let current_dur = match selected_slot {
                                        TransitionSlot::In => clip.start_transition().map(|t| t.duration_secs).unwrap_or(0.5),
                                        TransitionSlot::Out => clip.end_transition().map(|t| t.duration_secs).unwrap_or(0.5),
                                    };
                                    action = EffectsAndTransitionsAction::SetTransition {
                                        clip_id: clip.id,
                                        slot: selected_slot,
                                        transition: Some(Transition {
                                            kind: *kind,
                                            duration_secs: current_dur,
                                        }),
                                    };
                                }
                            }
                            ui.add_space(3.0);
                        }
                        ui.add_space(4.0);
                    }
                }
            });
        });

        action
    }
}
