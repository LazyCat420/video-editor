use crate::core::clip::Clip;
use crate::core::timeline::Timeline;
use crate::core::transition::{Transition, TransitionKind};
use crate::ui::theme::AppTheme;
use egui::{Button, Color32, Frame, Id, RichText, Rounding, ScrollArea, Stroke, Ui};

pub struct TransitionBinView;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionSlot {
    In,  // Beginning of clip (left edge)
    Out, // End of clip (right edge)
}

pub enum TransitionBinAction {
    None,
    SetTransition {
        clip_id: u64,
        slot: TransitionSlot,
        transition: Option<Transition>,
    },
}

impl TransitionBinView {
    pub fn render(ui: &mut Ui, timeline: &mut Timeline) -> TransitionBinAction {
        let mut action = TransitionBinAction::None;

        // Persistent slot selection in egui memory
        let slot_id = Id::new("transition_bin_selected_slot");
        let mut selected_slot: TransitionSlot = ui
            .data_mut(|d| d.get_temp(slot_id))
            .unwrap_or(TransitionSlot::In);

        // Find the currently selected clip (if any)
        let selected_clip: Option<Clip> = timeline.get_selected_clip().cloned();

        ui.vertical(|ui| {
            ui.add_space(4.0);

            // Context / Selection Status Banner
            Frame::none()
                .fill(AppTheme::bg_card())
                .rounding(Rounding::same(8.0))
                .inner_margin(10.0)
                .show(ui, |ui| {
                    if let Some(clip) = &selected_clip {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("🎬 Selected:")
                                    .strong()
                                    .size(13.0)
                                    .color(AppTheme::accent_blue()),
                            );
                            ui.label(
                                RichText::new(&clip.name)
                                    .strong()
                                    .size(13.0)
                                    .color(Color32::WHITE),
                            );
                        });

                        ui.add_space(6.0);

                        // Placement toggle: Beginning (In) vs End (Out)
                        ui.label(
                            RichText::new("Apply Transition To:")
                                .size(12.0)
                                .color(AppTheme::text_secondary())
                                .strong(),
                        );
                        ui.horizontal(|ui| {
                            let in_active = selected_slot == TransitionSlot::In;
                            let out_active = selected_slot == TransitionSlot::Out;

                            let in_btn = Button::new(
                                RichText::new("⇤ Beginning (In)")
                                    .size(12.0)
                                    .strong()
                                    .color(if in_active { Color32::WHITE } else { AppTheme::text_secondary() }),
                            )
                            .fill(if in_active { AppTheme::accent_blue() } else { AppTheme::bg_panel() })
                            .min_size(egui::vec2(110.0, 26.0));

                            if ui.add(in_btn).on_hover_text("Apply transition when the clip starts").clicked() {
                                selected_slot = TransitionSlot::In;
                                ui.data_mut(|d| d.insert_temp(slot_id, selected_slot));
                            }

                            let out_btn = Button::new(
                                RichText::new("End (Out) ⇥")
                                    .size(12.0)
                                    .strong()
                                    .color(if out_active { Color32::WHITE } else { AppTheme::text_secondary() }),
                            )
                            .fill(if out_active { AppTheme::accent_blue() } else { AppTheme::bg_panel() })
                            .min_size(egui::vec2(110.0, 26.0));

                            if ui.add(out_btn).on_hover_text("Apply transition when the clip finishes").clicked() {
                                selected_slot = TransitionSlot::Out;
                                ui.data_mut(|d| d.insert_temp(slot_id, selected_slot));
                            }
                        });

                        ui.add_space(6.0);

                        // Display active state for Beginning (In)
                        let active_in = clip.start_transition();
                        let active_out = clip.end_transition();

                        let current_slot_active = match selected_slot {
                            TransitionSlot::In => active_in,
                            TransitionSlot::Out => active_out,
                        };

                        if let Some(tr) = current_slot_active {
                            let slot_label = match selected_slot {
                                TransitionSlot::In => "⇤ Beginning Active:",
                                TransitionSlot::Out => "End Active ⇥:",
                            };
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("{} {}", slot_label, tr.kind.label()))
                                        .size(12.5)
                                        .color(AppTheme::accent_yellow())
                                        .strong(),
                                );
                            });

                            ui.add_space(3.0);

                            // Duration slider
                            let mut dur = tr.duration_secs;
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Length:")
                                        .size(12.0)
                                        .color(AppTheme::text_secondary()),
                                );
                                if crate::ui::small_slider(ui, 12.0, |ui| {
                                    ui.add_sized(
                                        [110.0, 12.0],
                                        egui::Slider::new(&mut dur, 0.2..=2.0)
                                            .custom_formatter(|v, _| format!("{:.1}s", v))
                                            .step_by(0.1),
                                    )
                                })
                                .changed()
                                {
                                    action = TransitionBinAction::SetTransition {
                                        clip_id: clip.id,
                                        slot: selected_slot,
                                        transition: Some(Transition {
                                            kind: tr.kind,
                                            duration_secs: dur,
                                        }),
                                    };
                                }
                            });

                            ui.add_space(4.0);

                            // Remove transition button
                            let remove_text = match selected_slot {
                                TransitionSlot::In => "❌ Remove Beginning Transition",
                                TransitionSlot::Out => "❌ Remove End Transition",
                            };
                            let remove_btn = Button::new(
                                RichText::new(remove_text)
                                    .size(12.0)
                                    .color(Color32::from_rgb(255, 130, 130)),
                            )
                            .min_size(egui::vec2(ui.available_width(), 26.0))
                            .fill(Color32::from_rgb(55, 25, 25));

                            if ui
                                .add(remove_btn)
                                .on_hover_text("Remove transition and revert to hard cut")
                                .clicked()
                            {
                                action = TransitionBinAction::SetTransition {
                                    clip_id: clip.id,
                                    slot: selected_slot,
                                    transition: None,
                                };
                            }
                        } else {
                            let slot_hint = match selected_slot {
                                TransitionSlot::In => "Click any style below to apply to the BEGINNING:",
                                TransitionSlot::Out => "Click any style below to apply to the END:",
                            };
                            ui.label(
                                RichText::new(slot_hint)
                                    .size(12.0)
                                    .color(AppTheme::text_secondary()),
                            );
                        }
                    } else {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("ℹ No Clip Selected")
                                    .strong()
                                    .size(13.0)
                                    .color(AppTheme::text_secondary()),
                            );
                            ui.add_space(2.0);
                            ui.label(
                                RichText::new("Click a video clip on the timeline, then pick a transition style below.")
                                    .size(12.0)
                                    .color(AppTheme::text_muted()),
                            );
                        });
                    }
                });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // Catalog of 18 Transition Presets grouped by category
            ScrollArea::vertical().show(ui, |ui| {
                let categories = [
                    (
                        "✨ Dissolves & Fades",
                        vec![
                            (
                                TransitionKind::CrossFade,
                                "Standard smooth blend between two clips",
                            ),
                            (
                                TransitionKind::DipToBlack,
                                "Fade out to pure black, then fade into the next clip",
                            ),
                            (
                                TransitionKind::DipToWhite,
                                "Bright flash transition often used for dramatic cuts",
                            ),
                        ],
                    ),
                    (
                        "↔ Wipes",
                        vec![
                            (
                                TransitionKind::WipeLeft,
                                "New clip reveals by wiping in from the right edge",
                            ),
                            (
                                TransitionKind::WipeRight,
                                "New clip reveals by wiping in from the left edge",
                            ),
                            (
                                TransitionKind::WipeUp,
                                "New clip reveals by wiping up from the bottom",
                            ),
                            (
                                TransitionKind::WipeDown,
                                "New clip reveals by wiping down from the top",
                            ),
                        ],
                    ),
                    (
                        "🎬 Slides & Motion",
                        vec![
                            (
                                TransitionKind::SlideLeft,
                                "Clip pushes the outgoing clip to the left",
                            ),
                            (
                                TransitionKind::SlideRight,
                                "Clip pushes the outgoing clip to the right",
                            ),
                            (
                                TransitionKind::SlideUp,
                                "Clip pushes the outgoing clip upward",
                            ),
                            (
                                TransitionKind::SlideDown,
                                "Clip pushes the outgoing clip downward",
                            ),
                            (
                                TransitionKind::SmoothLeft,
                                "Soft feathered directional slide",
                            ),
                        ],
                    ),
                    (
                        "🔷 Shapes & Stylized",
                        vec![
                            (
                                TransitionKind::CircleOpen,
                                "Circular opening iris reveal from center",
                            ),
                            (
                                TransitionKind::CircleClose,
                                "Circular closing iris transition",
                            ),
                            (
                                TransitionKind::Radial,
                                "Clockwise clock sweep radial transition",
                            ),
                            (
                                TransitionKind::ZoomIn,
                                "Dramatic zoom in transition into the new shot",
                            ),
                            (
                                TransitionKind::SqueezeHorizontal,
                                "Squeezes outgoing picture horizontally",
                            ),
                            (
                                TransitionKind::Pixelate,
                                "Retro pixelation mosaic dissolve",
                            ),
                        ],
                    ),
                ];

                for (cat_name, items) in &categories {
                    ui.label(
                        RichText::new(*cat_name)
                            .strong()
                            .size(13.0)
                            .color(AppTheme::accent_yellow()),
                    );
                    ui.add_space(4.0);

                    for (kind, desc) in items {
                        let is_active = selected_clip
                            .as_ref()
                            .and_then(|c| match selected_slot {
                                TransitionSlot::In => c.start_transition(),
                                TransitionSlot::Out => c.end_transition(),
                            })
                            .map(|t| t.kind == *kind)
                            .unwrap_or(false);

                        let card_bg = if is_active {
                            Color32::from_rgb(35, 45, 60)
                        } else {
                            AppTheme::bg_card()
                        };

                        let border_stroke = if is_active {
                            Stroke::new(1.5, AppTheme::accent_yellow())
                        } else {
                            Stroke::new(1.0, Color32::from_rgb(45, 45, 55))
                        };

                        let resp = Frame::none()
                            .fill(card_bg)
                            .rounding(Rounding::same(6.0))
                            .stroke(border_stroke)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(kind.label())
                                                    .strong()
                                                    .size(13.0)
                                                    .color(if is_active {
                                                        AppTheme::accent_yellow()
                                                    } else {
                                                        Color32::WHITE
                                                    }),
                                            );
                                            if is_active {
                                                ui.label(
                                                    RichText::new("✓ APPLIED")
                                                        .size(10.0)
                                                        .strong()
                                                        .color(AppTheme::accent_yellow()),
                                                );
                                            }
                                        });
                                        ui.label(
                                            RichText::new(*desc)
                                                .size(11.0)
                                                .color(AppTheme::text_muted()),
                                        );
                                    });

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            let apply_label = if is_active { "Active" } else { "Apply" };
                                            let btn = Button::new(
                                                RichText::new(apply_label)
                                                    .size(12.0)
                                                    .color(Color32::WHITE)
                                                    .strong(),
                                            )
                                            .fill(if is_active {
                                                AppTheme::accent_green()
                                            } else {
                                                AppTheme::accent_blue()
                                            })
                                            .min_size(egui::vec2(54.0, 26.0));

                                            if ui.add(btn).clicked() {
                                                if let Some(clip) = &selected_clip {
                                                    let current_dur = match selected_slot {
                                                        TransitionSlot::In => clip.start_transition().map(|t| t.duration_secs).unwrap_or(0.5),
                                                        TransitionSlot::Out => clip.end_transition().map(|t| t.duration_secs).unwrap_or(0.5),
                                                    };
                                                    action = TransitionBinAction::SetTransition {
                                                        clip_id: clip.id,
                                                        slot: selected_slot,
                                                        transition: Some(Transition {
                                                            kind: *kind,
                                                            duration_secs: current_dur,
                                                        }),
                                                    };
                                                }
                                            }
                                        },
                                    );
                                });
                            });

                        // Clicking anywhere on the card applies the transition if a clip is selected
                        if resp.response.interact(egui::Sense::click()).clicked() {
                            if let Some(clip) = &selected_clip {
                                let current_dur = match selected_slot {
                                    TransitionSlot::In => clip.start_transition().map(|t| t.duration_secs).unwrap_or(0.5),
                                    TransitionSlot::Out => clip.end_transition().map(|t| t.duration_secs).unwrap_or(0.5),
                                };
                                action = TransitionBinAction::SetTransition {
                                    clip_id: clip.id,
                                    slot: selected_slot,
                                    transition: Some(Transition {
                                        kind: *kind,
                                        duration_secs: current_dur,
                                    }),
                                };
                            }
                        }

                        ui.add_space(4.0);
                    }

                    ui.add_space(6.0);
                }
            });
        });

        action
    }
}
