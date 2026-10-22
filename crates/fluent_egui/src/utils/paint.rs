use std::f32::consts::TAU;

use egui::epaint::PathShape;
use egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind};

/// The border of a Fluent control at rest: a 1px stroke in `top`, with the
/// bottom edge in `bottom` so the control looks slightly raised. Approximates
/// WinUI's `ControlElevationBorderBrush` gradient.
pub fn paint_elevation_border(
    painter: &Painter,
    rect: Rect,
    corner_radius: f32,
    top: Color32,
    bottom: Color32,
) {
    painter.rect_stroke(
        rect,
        corner_radius,
        Stroke::new(1.0, top),
        StrokeKind::Inside,
    );
    if bottom != top {
        let x = (rect.left() + corner_radius)..=(rect.right() - corner_radius);
        painter.hline(x, rect.bottom() - 0.5, Stroke::new(1.0, bottom));
    }
}

/// Paints an arc with round caps. Angles are in radians, clockwise from the
/// positive x axis (screen coordinates).
pub fn paint_arc(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    start: f32,
    sweep: f32,
    stroke: Stroke,
) {
    if sweep.abs() < 1e-3 {
        return;
    }
    let segments = ((sweep.abs() / TAU) * 64.0).ceil().max(2.0) as usize;
    let points: Vec<Pos2> = (0..=segments)
        .map(|i| {
            let angle = start + sweep * i as f32 / segments as f32;
            center + radius * egui::vec2(angle.cos(), angle.sin())
        })
        .collect();
    let (first, last) = (points[0], points[segments]);
    painter.add(PathShape::line(points, stroke));
    painter.circle_filled(first, stroke.width / 2.0, stroke.color);
    painter.circle_filled(last, stroke.width / 2.0, stroke.color);
}
