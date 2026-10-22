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

/// The `chrome_close` X, about 10px across.
pub fn paint_close(painter: &Painter, center: Pos2, color: Color32) {
    let stroke = Stroke::new(1.2, color);
    let d = 4.5;
    painter.line_segment([center + vec2(-d, -d), center + vec2(d, d)], stroke);
    painter.line_segment([center + vec2(-d, d), center + vec2(d, -d)], stroke);
}

/// A check mark, about 9px wide.
pub fn paint_check(painter: &Painter, center: Pos2, size: f32, color: Color32) {
    let s = size / 9.0;
    let points = vec![
        center + vec2(-4.5, 0.5) * s,
        center + vec2(-1.5, 3.5) * s,
        center + vec2(4.5, -3.0) * s,
    ];
    painter.add(PathShape::line(
        points,
        Stroke::new(1.5 * s.max(1.0), color),
    ));
}

/// What [`paint_status_icon`] draws inside its circle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusGlyph {
    Info,
    Warning,
    Success,
    Error,
}

/// A filled 16px status circle with a glyph, like the InfoBar icons
/// (`info_solid`, `critical_error_solid`, `completed_solid`,
/// `status_error_full`).
pub fn paint_status_icon(
    painter: &Painter,
    center: Pos2,
    glyph: StatusGlyph,
    fill: Color32,
    ink: Color32,
) {
    painter.circle_filled(center, 8.0, fill);
    let stroke = Stroke::new(1.5, ink);
    match glyph {
        StatusGlyph::Info => {
            painter.circle_filled(center + vec2(0.0, -3.5), 1.0, ink);
            painter.line_segment([center + vec2(0.0, -1.0), center + vec2(0.0, 4.0)], stroke);
        }
        StatusGlyph::Warning => {
            painter.line_segment([center + vec2(0.0, -4.0), center + vec2(0.0, 1.0)], stroke);
            painter.circle_filled(center + vec2(0.0, 3.5), 1.0, ink);
        }
        StatusGlyph::Success => paint_check(painter, center, 8.0, ink),
        StatusGlyph::Error => {
            let d = 3.0;
            painter.line_segment([center + vec2(-d, -d), center + vec2(d, d)], stroke);
            painter.line_segment([center + vec2(-d, d), center + vec2(d, -d)], stroke);
        }
    }
}
