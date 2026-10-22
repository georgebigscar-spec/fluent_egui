use egui::{Painter, Rect, Stroke, StrokeKind};

use crate::FluentTheme;

/// Paints the two-stroke Fluent focus border around `rect`, like fluent_ui's
/// `FocusBorder` with `renderOutside: true`.
pub fn paint_focus_border(painter: &Painter, rect: Rect, theme: &FluentTheme) {
    let focus = &theme.focus;
    let outer = rect.expand(focus.primary_width);
    painter.rect_stroke(
        outer,
        focus.corner_radius,
        Stroke::new(focus.primary_width, focus.primary_color),
        StrokeKind::Inside,
    );
    painter.rect_stroke(
        rect,
        (focus.corner_radius - focus.primary_width).max(0.0),
        Stroke::new(focus.secondary_width, focus.secondary_color),
        StrokeKind::Inside,
    );
}
