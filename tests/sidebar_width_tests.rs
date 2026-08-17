//! Sidebar width-budget tests.
//!
//! The left sidebar is an `exact_width(280)` SidePanel with 8px margins → the
//! content budget is 264px. egui's `exact_width` clamps only the panel's INPUT
//! width; the REPORTED rect is `content min_rect + margins` with no post-clamp
//! (egui-0.29.1 panel.rs:286-293), so any child wider than the budget pushes
//! the CentralPanel right and opens a dead black gap between sidebar and
//! preview. These tests measure the real laid-out width of the Formatting tab
//! (fonts and all) for the worst-case inspector states, so no row can regress
//! past the budget unnoticed.

use video_editor::core::text_overlay::SlideElement;
use video_editor::ui::theme::{AppTheme, ThemeKind};
use video_editor::ui::SlideBinView;
use video_editor::VideoEditorApp;

const CONTENT_BUDGET: f32 = 264.0;
/// Small allowance for rounding in galley layout.
const TOLERANCE: f32 = 1.5;

/// Lay the Formatting tab out into a child Ui capped at the content budget and
/// return the width its `min_rect` actually grew to.
fn measure_formatting_tab(app: &mut VideoEditorApp) -> f32 {
    let ctx = egui::Context::default();
    AppTheme::configure(&ctx, ThemeKind::Dark, 1.0);

    let mut measured = 0.0_f32;
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800.0, 1400.0),
        )),
        ..Default::default()
    };
    let _ = ctx.run(input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let body = egui::Rect::from_min_size(
                ui.max_rect().min,
                egui::vec2(CONTENT_BUDGET, ui.max_rect().height()),
            );
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(body)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            let _ = SlideBinView::render(&mut child, app);
            measured = child.min_rect().width();
        });
    });
    measured
}

/// App with one selected slide carrying every element type, using a long
/// filename (the unbounded, user-data-driven overflow case).
fn worst_case_app() -> VideoEditorApp {
    let mut app = VideoEditorApp::default();
    app.insert_blank_slide_at_playhead(5.0, None);

    let long = std::path::PathBuf::from(
        "/media/family_reunion_lake_powell_summer_2024_camera_roll_0042_final_edit.mp4",
    );
    let id = app.active_slide().map(|c| c.id).unwrap();
    if let Some(clip) = app.project.timeline.get_clip_mut(id) {
        clip.elements.push(SlideElement::Picture {
            path: long.clone(),
            x: 0.1,
            y: 0.1,
            w: 0.5,
            h: 0.5,
        });
        clip.elements.push(SlideElement::Video {
            path: long.clone(),
            x: 0.1,
            y: 0.1,
            w: 0.5,
            h: 0.5,
        });
        clip.elements.push(SlideElement::Audio { path: long, volume: 1.0 });
        let mut overlay = video_editor::core::text_overlay::TextOverlay::new(
            "A fairly long first line of slide text that must not widen the sidebar",
        );
        overlay.font_size = 18.0;
        clip.elements.push(SlideElement::Text(overlay));
        clip.elements.push(SlideElement::Calendar(Default::default()));
    }
    app
}

/// Sabotage / positive control for the structural cap: a deliberately 400px
/// child must NOT grow the parent past the cap. This is the guarantee that no
/// future over-wide row can reopen the gap even if the row-level fixes are
/// missed — and it proves the cap actually engages (the sabotaged child
/// really is wider than the cap).
#[test]
fn width_cap_survives_a_pathological_child() {
    let ctx = egui::Context::default();
    AppTheme::configure(&ctx, ThemeKind::Dark, 1.0);

    let mut parent_w = 0.0_f32;
    let mut child_w = 0.0_f32;
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800.0, 600.0),
        )),
        ..Default::default()
    };
    let _ = ctx.run(input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Measure through a scope: the CentralPanel ui is pre-expanded to
            // the whole screen, so it cannot see a leak. The scope's reported
            // rect is exactly what the capped body allocated in its parent.
            let scope = ui.scope(|ui| {
                video_editor::ui::components::show_width_capped(ui, CONTENT_BUDGET, |ui| {
                    let r = ui.add(egui::Button::new("sabotage").min_size(egui::vec2(400.0, 20.0)));
                    child_w = r.rect.width();
                });
            });
            parent_w = scope.response.rect.width();
        });
    });

    assert!(child_w >= 399.0, "positive control: the sabotage child must really be ~400px, got {child_w:.1}");
    assert!(
        parent_w <= CONTENT_BUDGET + TOLERANCE,
        "the cap leaked: a 400px child grew the parent to {parent_w:.1}px"
    );
}

#[test]
fn formatting_tab_element_list_fits_the_budget() {
    let mut app = worst_case_app();
    app.selected_slide_element = None; // element list view (long filenames)
    let w = measure_formatting_tab(&mut app);
    assert!(
        w <= CONTENT_BUDGET + TOLERANCE,
        "element list grew the sidebar to {w:.1}px (budget {CONTENT_BUDGET}px) — this is the dead-gap bug"
    );
}

#[test]
fn every_element_inspector_fits_the_budget() {
    // Element order in worst_case_app: 0 Picture, 1 Video, 2 Audio, 3 Text, 4 Calendar.
    for idx in 0..5 {
        let mut app = worst_case_app();
        app.selected_slide_element = Some(idx);
        let w = measure_formatting_tab(&mut app);
        assert!(
            w <= CONTENT_BUDGET + TOLERANCE,
            "inspector for element {idx} grew the sidebar to {w:.1}px (budget {CONTENT_BUDGET}px) — this is the dead-gap bug"
        );
    }
}
