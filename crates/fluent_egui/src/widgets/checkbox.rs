use egui::epaint::PathShape;
use egui::{
    Pos2, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, Vec2, Widget, WidgetInfo,
    WidgetText, pos2,
};

use super::button::{accent_fill, on_accent_text};
use super::{LABELED_MIN_HEIGHT, allocate_labeled};
use crate::FluentTheme;
use crate::utils::{ControlState, States, animate_color, paint_focus_border};

enum Value<'a> {
    Bool(&'a mut bool),
    /// `None` is the third (indeterminate) state.
    Tristate(&'a mut Option<bool>),
}

/// A Fluent checkbox, fluent_ui's `Checkbox`.
pub struct Checkbox<'a> {
    value: Value<'a>,
    text: Option<WidgetText>,
    min_height: f32,
}

impl<'a> Checkbox<'a> {
    pub fn new(checked: &'a mut bool, text: impl Into<WidgetText>) -> Self {
        Self {
            value: Value::Bool(checked),
            text: Some(text.into()),
            min_height: LABELED_MIN_HEIGHT,
        }
    }

    /// A checkbox without a label.
    pub fn without_text(checked: &'a mut bool) -> Self {
        Self {
            value: Value::Bool(checked),
            text: None,
            min_height: LABELED_MIN_HEIGHT,
        }
    }

    /// A three-state checkbox. `None` shows a dash; clicking it goes to
    /// `None` again only through code, like fluent_ui (clicks flip between
    /// checked and unchecked, and the third state stays put).
    pub fn tristate(checked: &'a mut Option<bool>, text: impl Into<WidgetText>) -> Self {
        Self {
            value: Value::Tristate(checked),
            text: Some(text.into()),
            min_height: LABELED_MIN_HEIGHT,
        }
    }

    /// Height of the row. Defaults to 24 so stacked checkboxes get some
    /// space; pass 0 for the compact fluent_ui size.
    pub fn min_height(mut self, min_height: f32) -> Self {
        self.min_height = min_height;
        self
    }
}

impl Widget for Checkbox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = FluentTheme::of(ui.ctx());
        let r = &theme.resources;
        let label = self
            .text
            .as_ref()
            .map(|t| t.text().to_owned())
            .unwrap_or_default();
        let (mut response, boxed, text) = allocate_labeled(
            ui,
            Vec2::splat(20.0),
            8.0,
            self.min_height,
            self.text,
            Sense::click(),
        );

        let current = match &self.value {
            Value::Bool(b) => Some(**b),
            Value::Tristate(t) => **t,
        };
        if response.clicked() {
            match self.value {
                Value::Bool(b) => *b = !*b,
                Value::Tristate(t) => *t = t.map(|c| !c),
            }
            response.mark_changed();
        }
        response.widget_info(|| match current {
            Some(c) => WidgetInfo::selected(Role::CheckBox, ui.is_enabled(), c, &label),
            None => WidgetInfo::labeled(Role::CheckBox, ui.is_enabled(), &label),
        });

        if ui.is_rect_visible(response.rect) {
            let state = ControlState::from_response(&response);
            let painter = ui.painter();
            let on = current != Some(false);
            let (fill, border) = if on {
                let fill = accent_fill(&theme).resolve(state);
                let border = if state.disabled {
                    r.control_strong_stroke_color_disabled
                } else {
                    fill
                };
                (fill, border)
            } else {
                let fill = States::new(
                    r.control_alt_fill_color_secondary,
                    r.control_alt_fill_color_tertiary,
                    r.control_alt_fill_color_quarternary,
                    r.control_alt_fill_color_disabled,
                )
                .resolve(state);
                let border = if state.disabled || state.pressed {
                    r.control_strong_stroke_color_disabled
                } else {
                    r.control_strong_stroke_color_default
                };
                (fill, border)
            };
            let id = response.id;
            let fill = animate_color(
                ui.ctx(),
                id.with("fill"),
                fill,
                theme.faster_animation_duration,
            );
            let border = animate_color(
                ui.ctx(),
                id.with("border"),
                border,
                theme.faster_animation_duration,
            );
            painter.rect_filled(boxed, 6.0, fill);
            painter.rect_stroke(boxed, 6.0, Stroke::new(1.0, border), StrokeKind::Inside);

            let mark = on_accent_text(&theme).resolve(state);
            match current {
                Some(true) => paint_check_mark(painter, boxed, mark),
                None => {
                    let dash = Rect::from_center_size(boxed.center(), Vec2::new(8.0, 1.4));
                    painter.rect_filled(dash, 0.0, mark);
                }
                Some(false) => {}
            }
            if let Some((pos, galley)) = text {
                let color = if state.disabled {
                    r.text_fill_color_disabled
                } else {
                    r.text_fill_color_primary
                };
                painter.galley(pos, galley, color);
            }
            if state.focused {
                paint_focus_border(painter, boxed, &theme);
            }
        }
        response
    }
}

/// The `check_mark` glyph drawn as a path, so it works without icon fonts.
fn paint_check_mark(painter: &egui::Painter, rect: Rect, color: egui::Color32) {
    let c = rect.center();
    let at = |x: f32, y: f32| -> Pos2 { pos2(c.x + x, c.y + y) };
    painter.add(PathShape::line(
        vec![at(-4.5, 0.5), at(-1.5, 3.5), at(4.5, -3.0)],
        Stroke::new(1.5, color),
    ));
}
