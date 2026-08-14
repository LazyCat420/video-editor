use crate::core::clip::Clip;
use crate::core::timeline::Timeline;
use crate::core::transition::{Transition, TransitionKind};
use crate::ui::theme::AppTheme;
use egui::{Button, Color32, Frame, RichText, Rounding, ScrollArea, Stroke, Ui};

pub struct TransitionBinView;

pub enum TransitionBinAction {
    None,
    SetTransition {
        clip_id: u64,
        transition: Option<Transition>,
    },
}

impl TransitionBinView {
    pub fn render(ui: &mut Ui, timeline: &mut Timeline) -> TransitionBinAction {
        let mut action = TransitionBinAction::None;

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

                        ui.add_space(4.0);

                        if let Some(tr) = clip.transition.as_ref() {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("✦ Active: {}", tr.kind.label()))
                                        .size(13.0)
                                        .color(AppTheme::accent_yellow())
                                        .strong(),
                                );
                            });

                            ui.add_space(4.0);

                            // Duration slider
                            let mut dur = tr.duration_secs;
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Length:")
                                        .size(12.0)
                                        .color(AppTheme::text_secondary()),
                                );
                                if crate::ui::small_slider(ui, 18.0, |ui| {
                                    ui.add_sized(
                                        [110.0, 18.0],
                                        egui::Slider::new(&mut dur, 0.2..=2.0)
                                            .custom_formatter(|v, _| format!("{:.1}s", v))
                                            .step_by(0.1),
                                    )
                                })
                                .changed()
                                {
                                    action = TransitionBinAction::SetTransition {
                                        clip_id: clip.id,
                                        transition: Some(Transition {
                                            kind: tr.kind,
                                            duration_secs: dur,
                                        }),
                                    };
                                }
                            });

                            ui.add_space(4.0);

                            // Remove transition button
                            let remove_btn = Button::new(
                                RichText::new("❌ Remove (Hard Cut)")
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
                                    transition: None,
                                };
                            }
                        } else {
                            ui.label(
                                RichText::new("Click any transition below to apply it to this clip:")
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

            // Groupings for the 17 Transition Styles
            let categories: &[(&str, &[(TransitionKind, &str, &str)])] = &[
                (
                    "✨ Dissolves & Fades",
                    &[
                        (
                            TransitionKind::CrossFade,
                            "Cross Fade",
                            "Classic smooth blend between scenes",
                        ),
                        (
                            TransitionKind::DipToBlack,
                            "Dip to Black",
                            "Fades to black then fades next scene in",
                        ),
                        (
                            TransitionKind::DipToWhite,
                            "Dip to White",
                            "Bright flash fade between scenes",
                        ),
                    ],
                ),
                (
                    "📐 Wipes",
                    &[
                        (
                            TransitionKind::WipeLeft,
                            "Wipe Left",
                            "Wipes across screen to the left",
                        ),
                        (
                            TransitionKind::WipeRight,
                            "Wipe Right",
                            "Wipes across screen to the right",
                        ),
                        (
                            TransitionKind::WipeUp,
                            "Wipe Up",
                            "Wipes vertically from bottom to top",
                        ),
                        (
                            TransitionKind::WipeDown,
                            "Wipe Down",
                            "Wipes vertically from top to bottom",
                        ),
                    ],
                ),
                (
                    "🚀 Slides & Push",
                    &[
                        (
                            TransitionKind::SlideLeft,
                            "Slide Left",
                            "Pushes incoming clip in from the right",
                        ),
                        (
                            TransitionKind::SlideRight,
                            "Slide Right",
                            "Pushes incoming clip in from the left",
                        ),
                        (
                            TransitionKind::SlideUp,
                            "Slide Up",
                            "Pushes incoming clip in from the bottom",
                        ),
                        (
                            TransitionKind::SlideDown,
                            "Slide Down",
                            "Pushes incoming clip in from the top",
                        ),
                        (
                            TransitionKind::SmoothLeft,
                            "Smooth Slide",
                            "Soft continuous cinematic slide",
                        ),
                    ],
                ),
                (
                    "🎨 Shapes, Zoom & Effects",
                    &[
                        (
                            TransitionKind::CircleOpen,
                            "Circle / Iris Open",
                            "Circular expanding iris reveal",
                        ),
                        (
                            TransitionKind::CircleClose,
                            "Circle Close",
                            "Circular closing iris transition",
                        ),
                        (
                            TransitionKind::Radial,
                            "Radial Clock",
                            "Clockwise sweep wipe",
                        ),
                        (
                            TransitionKind::ZoomIn,
                            "Zoom In",
                            "Smoothly zooms into the incoming scene",
                        ),
                        (
                            TransitionKind::SqueezeHorizontal,
                            "Squeeze Horizontal",
                            "Squeezes outgoing clip horizontally",
                        ),
                        (
                            TransitionKind::Pixelate,
                            "Pixelate",
                            "Mosaic pixel dissolution blend",
                        ),
                    ],
                ),
            ];

            ScrollArea::vertical().show(ui, |ui| {
                for (cat_name, items) in categories {
                    ui.label(
                        RichText::new(*cat_name)
                            .strong()
                            .size(14.0)
                            .color(AppTheme::accent_blue()),
                    );
                    ui.add_space(4.0);

                    for (kind, title, desc) in *items {
                        let is_active = selected_clip
                            .as_ref()
                            .and_then(|c| c.transition.as_ref())
                            .map(|t| t.kind == *kind)
                            .unwrap_or(false);

                        let card_fill = if is_active {
                            Color32::from_rgb(35, 60, 95)
                        } else {
                            AppTheme::bg_card()
                        };

                        let border_stroke = if is_active {
                            Stroke::new(1.5, AppTheme::accent_yellow())
                        } else {
                            Stroke::new(1.0, AppTheme::bg_hover())
                        };

                        let resp = Frame::none()
                            .fill(card_fill)
                            .stroke(border_stroke)
                            .rounding(Rounding::same(6.0))
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(*title)
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
                                                    let current_dur = clip
                                                        .transition
                                                        .as_ref()
                                                        .map(|t| t.duration_secs)
                                                        .unwrap_or(0.5);
                                                    action = TransitionBinAction::SetTransition {
                                                        clip_id: clip.id,
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
                                let current_dur = clip
                                    .transition
                                    .as_ref()
                                    .map(|t| t.duration_secs)
                                    .unwrap_or(0.5);
                                action = TransitionBinAction::SetTransition {
                                    clip_id: clip.id,
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
