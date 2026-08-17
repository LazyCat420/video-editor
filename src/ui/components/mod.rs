pub mod card;
pub use card::{ActionRowCard, SidebarTabs};

/// Run `add_contents` inside a child Ui hard-capped to `width`.
///
/// The parent advances by EXACTLY `width` no matter what the child allocates.
/// This is the only unconditionally exact cap in egui 0.29: `new_child`
/// allocates nothing in the parent (ui.rs:242-246) and
/// `advance_cursor_after_rect` allocates exactly the rect it is handed —
/// unlike `set_max_width` (advisory: an oversized allocation is unioned back
/// into min_rect AND max_rect, layout.rs:49-52) or `allocate_ui_with_layout`
/// (re-allocates the grown child rect, ui.rs:1400-1413). Used by the sidebar
/// so no over-wide child can ever push the CentralPanel right again (the
/// sidebar↔preview dead-gap bug, twice now). Overflowing children are clipped
/// invisible — content must still be sized to fit to stay usable.
pub fn show_width_capped<R>(
    ui: &mut egui::Ui,
    width: f32,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let mut body = ui.max_rect();
    body.max.x = body.min.x + width;
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(body)
            .layout(*ui.layout()),
    );
    child.set_clip_rect(child.clip_rect().intersect(body));
    let result = add_contents(&mut child);
    let used = egui::Rect::from_min_size(
        body.min,
        egui::vec2(width, child.min_rect().height().max(body.height())),
    );
    ui.advance_cursor_after_rect(used);
    result
}
