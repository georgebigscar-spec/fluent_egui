use std::ops::RangeInclusive;

use egui::{
    Area, Key, Order, Pos2, Rect, Response, Sense, Ui, Vec2, Widget, WidgetInfo, pos2, vec2,
};

use super::button::accent_fill;
use super::tooltip::tooltip_frame;
use crate::FluentTheme;
use crate::utils::{ControlState, paint_focus_border};

const THUMB_RADIUS: f32 = 10.0;
const TRACK_HEIGHT: f32 = 3.75;

/// A Fluent slider, fluent_ui's `Slider`. Horizontal by default,
/// [`Slider::vertical`] for a vertical one with the minimum at the bottom.
///
/// Drag the thumb, click the track, or use the arrow keys while focused.
pub struct Slider<'a> {
    value: &'a mut f32,
    range: RangeInclusive<f32>,
    step: Option<f32>,
    length: Option<f32>,
    vertical: bool,
    label: Option<Box<dyn Fn(f32) -> String + 'a>>,
}

impl<'a> Slider<'a> {
    pub fn new(value: &'a mut f32, range: RangeInclusive<f32>) -> Self {
        Self {
            value,
            range,
            step: None,
            length: None,
            vertical: false,
            label: None,
        }
    }

    /// Snaps the value to multiples of `step` from the start of the range,
    /// like fluent_ui's `divisions`.
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step).filter(|s| *s > 0.0);
        self
    }

    /// Total width (height when vertical). Defaults to 200 or the available
    /// space, whichever is smaller.
    pub fn width(mut self, width: f32) -> Self {
        self.length = Some(width);
        self
    }

    /// Lays the slider out bottom to top, fluent_ui's `Axis.vertical`.
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    /// Shows the value in a tooltip by the thumb while it is hovered or
    /// dragged, fluent_ui's `label`. Whole numbers show without decimals,
    /// others with one.
    pub fn value_tooltip(self) -> Self {
        self.value_label(|v| {
            if v.fract() == 0.0 {
                format!("{v:.0}")
            } else {
                format!("{v:.1}")
            }
        })
    }

    /// Like [`Slider::value_tooltip`] with custom text.
    pub fn value_label(mut self, label: impl Fn(f32) -> String + 'a) -> Self {
        self.label = Some(Box::new(label));
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
        let vertical = self.vertical;
        let available = if vertical {
            ui.available_height()
        } else {
            ui.available_width()
        };
        let length = self.length.unwrap_or_else(|| available.min(200.0));
        let size = if vertical {
            vec2(32.0, length)
        } else {
            vec2(length, 32.0)
        };
        let (rect, mut response) = ui.allocate_exact_size(size, Sense::click_and_drag());
        let (min, max) = (*self.range.start(), *self.range.end());
        let span = max - min;
        let track_length = length - 2.0 * THUMB_RADIUS;
        // The point on the track at fraction `t`, and back.
        let point = |t: f32| -> Pos2 {
            if vertical {
                pos2(
                    rect.center().x,
                    rect.bottom() - THUMB_RADIUS - t * track_length,
                )
            } else {
                pos2(
                    rect.left() + THUMB_RADIUS + t * track_length,
                    rect.center().y,
                )
            }
        };
        let fraction = |p: Pos2| -> f32 {
            let along = if vertical {
                rect.bottom() - THUMB_RADIUS - p.y
            } else {
                p.x - rect.left() - THUMB_RADIUS
            };
            (along / track_length).clamp(0.0, 1.0)
        };

        let old = *self.value;
        let mut new = old;
        if let Some(p) = response.interact_pointer_pos()
            && (response.dragged() || response.clicked())
            && track_length > 0.0
        {
            new = min + fraction(p) * span;
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

        let t = if span == 0.0 {
            0.0
        } else {
            ((value - min) / span).clamp(0.0, 1.0)
        };
        let center = point(t);
        if ui.is_rect_visible(rect) {
            let state = ControlState::from_response(&response);
            let sliding = response.dragged();
            let painter = ui.painter();

            let active = accent_fill(&theme).resolve(state);
            let inactive = if state.disabled {
                r.control_strong_fill_color_disabled
            } else {
                r.control_strong_fill_color_default
            };
            let track =
                |from: Pos2, to: Pos2| Rect::from_two_pos(from, to).expand(TRACK_HEIGHT / 2.0);
            painter.rect_filled(track(point(0.0), point(1.0)), TRACK_HEIGHT / 2.0, inactive);
            painter.rect_filled(track(point(0.0), center), TRACK_HEIGHT / 2.0, active);

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

        if let Some(label) = &self.label
            && (response.dragged() || response.hovered() || response.has_focus())
            && ui.is_enabled()
        {
            let text = label(value);
            let caption = theme.typography.caption.font_id();
            let galley = ui.fonts_mut(|f| {
                f.layout_no_wrap(text, caption, theme.resources.text_fill_color_primary)
            });
            let frame = tooltip_frame(&theme);
            let size = galley.size() + frame.total_margin().sum();
            // Above the thumb, or to its left when vertical.
            let pos = if vertical {
                center - vec2(THUMB_RADIUS + 8.0 + size.x, size.y / 2.0)
            } else {
                center - vec2(size.x / 2.0, THUMB_RADIUS + 8.0 + size.y)
            };
            Area::new(response.id.with("fluent_slider_label"))
                .order(Order::Tooltip)
                .fixed_pos(pos)
                .interactable(false)
                .show(ui.ctx(), |ui| {
                    frame.show(ui, |ui| {
                        ui.painter().galley(
                            ui.cursor().min,
                            galley.clone(),
                            theme.resources.text_fill_color_primary,
                        );
                        ui.allocate_space(galley.size());
                    });
                });
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
