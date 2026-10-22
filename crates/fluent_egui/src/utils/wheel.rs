use egui::{Event, Rect, Ui, Vec2};

/// Turns mouse wheel input over `rect` into discrete steps: -1 for up, 1 for
/// down, 0 for none. The scroll is consumed, so an enclosing scroll area
/// stays put while the pointer is over a stepping control.
pub fn wheel_steps(ui: &Ui, rect: Rect) -> i32 {
    if !ui.rect_contains_pointer(rect) {
        return 0;
    }
    ui.input_mut(|i| {
        let dy: f32 = i
            .events
            .iter()
            .filter_map(|e| match e {
                Event::MouseWheel { delta, .. } => Some(delta.y),
                _ => None,
            })
            .sum();
        i.smooth_scroll_delta = Vec2::ZERO;
        if dy > 0.0 {
            -1
        } else if dy < 0.0 {
            1
        } else {
            0
        }
    })
}
