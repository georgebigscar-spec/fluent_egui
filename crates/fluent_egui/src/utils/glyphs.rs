//! Small glyphs drawn with paths, so controls render without icon fonts.

use egui::epaint::PathShape;
use egui::{Color32, Painter, Pos2, Stroke, vec2};

/// `chevron_down`, about 10px wide, centered on `center`.
pub fn paint_chevron_down(painter: &Painter, center: Pos2, color: Color32) {
    chevron(painter, center, 1.0, color);
}

/// `chevron_up`, about 10px wide, centered on `center`.
pub fn paint_chevron_up(painter: &Painter, center: Pos2, color: Color32) {
    chevron(painter, center, -1.0, color);
}

fn chevron(painter: &Painter, center: Pos2, dir: f32, color: Color32) {
    let points = vec![
        center + vec2(-4.5, -2.25 * dir),
        center + vec2(0.0, 2.25 * dir),
        center + vec2(4.5, -2.25 * dir),
    ];
    painter.add(PathShape::line(points, Stroke::new(1.25, color)));
}

/// The `red_eye` reveal glyph of a password box.
pub fn paint_eye(painter: &Painter, center: Pos2, color: Color32) {
    let stroke = Stroke::new(1.2, color);
    let lid = |dir: f32| -> Vec<Pos2> {
        (0..=12)
            .map(|i| {
                let t = i as f32 / 12.0 * std::f32::consts::PI;
                center + vec2(-6.0 * t.cos(), -dir * 3.5 * t.sin())
            })
            .collect()
    };
    painter.add(PathShape::line(lid(1.0), stroke));
    painter.add(PathShape::line(lid(-1.0), stroke));
    painter.circle_stroke(center, 1.8, stroke);
}
