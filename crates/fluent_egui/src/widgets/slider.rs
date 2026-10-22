use std::ops::RangeInclusive;

use egui::{Key, Rect, Response, Sense, Ui, Vec2, Widget, WidgetInfo, pos2};

use super::button::accent_fill;
use crate::FluentTheme;
use crate::utils::{ControlState, paint_focus_border};

const THUMB_RADIUS: f32 = 10.0;
const TRACK_HEIGHT: f32 = 3.75;

/// A horizontal Fluent slider, fluent_ui's `Slider`.
///
/// Drag the thumb, click the track, or use the arrow keys while focused.
pub struct Slider<'a> {
    value: &'a mut f32,
    range: RangeInclusive<f32>,
    step: Option<f32>,
    width: Option<f32>,
}

impl<'a> Slider<'a> {
    pub fn new(value: &'a mut f32, range: RangeInclusive<f32>) -> Self {
        Self {
            value,
            range,
            step: None,
            width: None,
        }
    }

    /// Snaps the value to multiples of `step` from the start of the range,
    /// like fluent_ui's `divisions`.
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step).filter(|s| *s > 0.0);
        self
    }

    /// Total width. Defaults to 200 or the available width, whichever is
    /// smaller.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    fn snap(&self, v: f32) -> f32 {
        let (min, max) = (*self.range.start(), *self.range.end());
        let v = match self.step {
            Some(step) => min + ((v - min) / step).round() * step,
            None => v,
        };
        v.clamp(min.min(max), max.max(min))
    }
}

impl Widget for Slider<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = FluentTheme::of(ui.ctx());
        let r = &theme.resources;
        let width = self
            .width
            .unwrap_or_else(|| ui.available_width().min(200.0));
        let (rect, mut response) =
            ui.allocate_exact_size(Vec2::new(width, 32.0), Sense::click_and_drag());
        let (min, max) = (*self.range.start(), *self.range.end());
        let span = max - min;
        let track_x = (rect.left() + THUMB_RADIUS)..=(rect.right() - THUMB_RADIUS);
        let track_width = track_x.end() - track_x.start();

        let old = *self.value;
        let mut new = old;
        if let Some(p) = response.interact_pointer_pos()
            && (response.dragged() || response.clicked())
            && track_width > 0.0
        {
            let t = ((p.x - track_x.start()) / track_width).clamp(0.0, 1.0);
            new = min + t * span;
        }
        if response.has_focus() {
            let step = self.step.unwrap_or(span / 100.0);
            ui.input(|i| {
                if i.key_pressed(Key::ArrowRight) || i.key_pressed(Key::ArrowUp) {
                    new += step;
                }
                if i.key_pressed(Key::ArrowLeft) || i.key_pressed(Key::ArrowDown) {
                    new -= step;
                }
                if i.key_pressed(Key::Home) {
                    new = min;
                }
                if i.key_pressed(Key::End) {
                    new = max;
                }
            });
        }
        let new = self.snap(new);
        if new != old {
            *self.value = new;
            response.mark_changed();
        }
        let value = *self.value;
        response.widget_info(|| WidgetInfo::slider(ui.is_enabled(), value as f64, ""));

        if ui.is_rect_visible(rect) {
            let state = ControlState::from_response(&response);
            let sliding = response.dragged();
            let painter = ui.painter();
            let t = if span == 0.0 {
                0.0
            } else {
                ((value - min) / span).clamp(0.0, 1.0)
            };
            let thumb_x = track_x.start() + t * track_width;
            let cy = rect.center().y;

            let active = accent_fill(&theme).resolve(state);
            let inactive = if state.disabled {
                r.control_strong_fill_color_disabled
            } else {
                r.control_strong_fill_color_default
            };
            let track = |from: f32, to: f32| {
                Rect::from_min_max(
                    pos2(from, cy - TRACK_HEIGHT / 2.0),
                    pos2(to, cy + TRACK_HEIGHT / 2.0),
                )
            };
            painter.rect_filled(
                track(*track_x.start(), *track_x.end()),
                TRACK_HEIGHT / 2.0,
                inactive,
            );
            painter.rect_filled(track(*track_x.start(), thumb_x), TRACK_HEIGHT / 2.0, active);

            // Thumb ball: a solid disc with an accent dot that grows on hover
            // and shrinks while pressed (`thumbBallInnerFactor`).
            let factor = if state.pressed || sliding {
                0.45
            } else if state.hovered {
                0.66
            } else {
                0.5
            };
            let factor = ui.ctx().animate_value_with_time(
                response.id.with("thumb"),
                factor,
                theme.fast_animation_duration.as_secs_f32(),
            );
            let center = pos2(thumb_x, cy);
            painter.circle_filled(center, THUMB_RADIUS, r.control_solid_fill_color_default);
            painter.circle_stroke(
                center,
                THUMB_RADIUS - 0.5,
                egui::Stroke::new(1.0, r.control_stroke_color_default),
            );
            painter.circle_filled(center, THUMB_RADIUS * factor, active);
            if state.focused {
                let thumb = Rect::from_center_size(center, Vec2::splat(THUMB_RADIUS * 2.0));
                paint_focus_border(painter, thumb, &theme);
            }
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snaps_to_step_and_range() {
        let mut v = 0.0;
        let slider = Slider::new(&mut v, 0.0..=10.0).step(2.5);
        assert_eq!(slider.snap(3.6), 2.5);
        assert_eq!(slider.snap(3.8), 5.0);
        assert_eq!(slider.snap(42.0), 10.0);
        assert_eq!(slider.snap(-1.0), 0.0);
    }
}
