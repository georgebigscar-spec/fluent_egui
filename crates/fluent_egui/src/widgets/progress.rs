use std::f32::consts::{FRAC_PI_2, PI, TAU};

use egui::{Color32, Id, Rect, Response, Role, Sense, Stroke, Ui, Vec2, Widget, WidgetInfo, pos2};

use crate::FluentTheme;
use crate::utils::paint_arc;

/// A horizontal progress bar, fluent_ui's `ProgressBar`.
///
/// `value` is in `0.0..=100.0`; `None` shows the indeterminate animation.
pub struct ProgressBar {
    value: Option<f32>,
    stroke_width: f32,
    width: Option<f32>,
    active_color: Option<Color32>,
    background_color: Option<Color32>,
}

impl ProgressBar {
    pub fn new(value: f32) -> Self {
        Self {
            value: Some(value),
            stroke_width: 4.5,
            width: None,
            active_color: None,
            background_color: None,
        }
    }

    pub fn indeterminate() -> Self {
        Self {
            value: None,
            stroke_width: 4.5,
            width: None,
            active_color: None,
            background_color: None,
        }
    }

    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    /// Defaults to the available width, at least 130.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// The filled part, fluent_ui's `activeColor`. Defaults to the accent
    /// color, pass a status color for meters such as password strength.
    pub fn active_color(mut self, color: Color32) -> Self {
        self.active_color = Some(color);
        self
    }

    /// The track behind it, fluent_ui's `backgroundColor`.
    pub fn background_color(mut self, color: Color32) -> Self {
        self.background_color = Some(color);
        self
    }
}

/// State of the indeterminate bar animation, which moves two ends of a line
/// at different speeds. The math comes from fluent_ui (credited there to
/// raitonoberu).
#[derive(Clone, Copy)]
struct BarAnimation {
    p1: f32,
    p2: f32,
    cycle: f32,
    last: f32,
}

impl BarAnimation {
    const STEP1: f32 = 2.7;
    const STEP2: f32 = 4.5;
    const VELOCITY_SCALE: f32 = 0.8;
    const SHORT: f32 = 0.4;
    const LONG: f32 = 80.0 / 130.0;

    fn advance(&mut self, value: f32) {
        let mut delta = value - self.last;
        self.last = value;
        if delta < 0.0 {
            delta += 1.0;
        }
        let velocity = |p: f32| (1.0 + (PI * p - FRAC_PI_2).cos() * Self::VELOCITY_SCALE) * delta;
        let (v1, v2) = (velocity(self.p1), velocity(self.p2));
        let (step, length) = if self.cycle > 0.0 {
            (Self::STEP1, Self::SHORT)
        } else {
            (Self::STEP2, Self::LONG)
        };
        self.p2 = (self.p2 + step * v2).min(1.0);
        if self.p2 - self.p1 >= length || self.p2 == 1.0 {
            self.p1 = (self.p1 + step * v1).min(1.0);
        }
        if self.p1 == 1.0 {
            self.cycle = -self.cycle;
            self.p1 = 0.0;
            self.p2 = 0.0;
        }
    }
}

impl Widget for ProgressBar {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = FluentTheme::of(ui.ctx());
        let width = self
            .width
            .unwrap_or_else(|| ui.available_width())
            .max(130.0);
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(width, self.stroke_width), Sense::hover());
        response.widget_info(|| {
            let mut info = WidgetInfo::new(Role::ProgressIndicator);
            info.value = self.value.map(f64::from);
            info
        });

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let radius = self.stroke_width / 2.0;
            let active = self
                .active_color
                .unwrap_or_else(|| theme.accent_color.default_brush_for(theme.brightness));
            let background = self
                .background_color
                .unwrap_or(theme.inactive_background_color);
            painter.rect_filled(rect, radius, background);
            let segment = |from: f32, to: f32| {
                let x0 = rect.left() + from.clamp(0.0, 1.0) * rect.width();
                let x1 = rect.left() + to.clamp(0.0, 1.0) * rect.width();
                if x1 - x0 > 0.0 {
                    let r = Rect::from_min_max(
                        pos2(x0, rect.top()),
                        pos2(x1.max(x0 + self.stroke_width), rect.bottom()),
                    );
                    painter.rect_filled(r.intersect(rect), radius, active);
                }
            };
            match self.value {
                Some(v) => segment(0.0, v / 100.0),
                None => {
                    let id: Id = response.id.with("indeterminate");
                    let time = ui.input(|i| i.time) as f32;
                    let mut anim = ui.data(|d| d.get_temp(id)).unwrap_or(BarAnimation {
                        p1: 0.0,
                        p2: 0.0,
                        cycle: 1.0,
                        last: (time / 3.0).fract(),
                    });
                    anim.advance((time / 3.0).fract());
                    ui.data_mut(|d| d.insert_temp(id, anim));
                    segment(anim.p1, anim.p2);
                    ui.ctx().request_repaint();
                }
            }
        }
        response
    }
}

/// A circular progress indicator, fluent_ui's `ProgressRing`.
///
/// `value` is in `0.0..=100.0`; `None` spins indefinitely.
pub struct ProgressRing {
    value: Option<f32>,
    stroke_width: f32,
    size: f32,
    backwards: bool,
}

impl ProgressRing {
    pub fn new(value: f32) -> Self {
        Self {
            value: Some(value),
            stroke_width: 4.5,
            size: 36.0,
            backwards: false,
        }
    }

    pub fn indeterminate() -> Self {
        Self {
            value: None,
            stroke_width: 4.5,
            size: 36.0,
            backwards: false,
        }
    }

    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    /// Diameter, 36 by default.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Spins counter-clockwise.
    pub fn backwards(mut self, backwards: bool) -> Self {
        self.backwards = backwards;
        self
    }
}

impl Widget for ProgressRing {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = FluentTheme::of(ui.ctx());
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(self.size), Sense::hover());
        response.widget_info(|| {
            let mut info = WidgetInfo::new(Role::ProgressIndicator);
            info.value = self.value.map(f64::from);
            info
        });

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center = rect.center();
            let radius = (self.size - self.stroke_width) / 2.0;
            painter.circle_stroke(
                center,
                radius,
                Stroke::new(self.stroke_width, theme.inactive_background_color),
            );
            let stroke = Stroke::new(
                self.stroke_width,
                theme.accent_color.default_brush_for(theme.brightness),
            );
            let top = -FRAC_PI_2;
            match self.value {
                Some(v) => paint_arc(
                    painter,
                    center,
                    radius,
                    top,
                    (v / 100.0).clamp(0.0, 1.0) * TAU,
                    stroke,
                ),
                None => {
                    // Two-step tween sequences from fluent_ui, in degrees.
                    let t = ((ui.input(|i| i.time) / 2.0).fract()) as f32;
                    let (start, sweep) = if t < 0.5 {
                        (450.0 * t * 2.0, 180.0 * t * 2.0)
                    } else {
                        let u = (t - 0.5) * 2.0;
                        (450.0 + 630.0 * u, 180.0 * (1.0 - u))
                    };
                    let start = if self.backwards { -start } else { start };
                    paint_arc(
                        painter,
                        center,
                        radius,
                        (start - 90.0f32).to_radians(),
                        sweep.to_radians(),
                        stroke,
                    );
                    ui.ctx().request_repaint();
                }
            }
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indeterminate_bar_stays_in_bounds_and_cycles() {
        let mut anim = BarAnimation {
            p1: 0.0,
            p2: 0.0,
            cycle: 1.0,
            last: 0.0,
        };
        let mut flips = 0;
        for frame in 1..=600 {
            let before = anim.cycle;
            anim.advance((frame as f32 / 180.0).fract());
            assert!((0.0..=1.0).contains(&anim.p1) && (0.0..=1.0).contains(&anim.p2));
            assert!(anim.p1 <= anim.p2);
            if anim.cycle != before {
                flips += 1;
            }
        }
        assert!(flips >= 2, "short and long lines should alternate");
    }
}
