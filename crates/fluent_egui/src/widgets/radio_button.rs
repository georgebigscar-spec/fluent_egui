use egui::{Response, Role, Sense, Stroke, Ui, Vec2, Widget, WidgetInfo, WidgetText};

use super::button::accent_fill;
use super::{LABELED_MIN_HEIGHT, allocate_labeled};
use crate::FluentTheme;
use crate::utils::{ControlState, States, paint_focus_border};

/// A Fluent radio button, fluent_ui's `RadioButton`. Returns a clicked
/// response; for the usual "pick one of several values" case use
/// [`radio_value`].
pub struct RadioButton {
    checked: bool,
    text: Option<WidgetText>,
    min_height: f32,
}

impl RadioButton {
    pub fn new(checked: bool, text: impl Into<WidgetText>) -> Self {
        Self {
            checked,
            text: Some(text.into()),
            min_height: LABELED_MIN_HEIGHT,
        }
    }

    pub fn without_text(checked: bool) -> Self {
        Self {
            checked,
            text: None,
            min_height: LABELED_MIN_HEIGHT,
        }
    }

    /// Height of the row. Defaults to 24 so a stack of radio buttons gets
    /// breathing room; pass 0 for the compact fluent_ui size.
    pub fn min_height(mut self, min_height: f32) -> Self {
        self.min_height = min_height;
        self
    }
}

/// A radio button that sets `current` to `alternative` when clicked.
pub fn radio_value<T: PartialEq>(
    ui: &mut Ui,
    current: &mut T,
    alternative: T,
    text: impl Into<WidgetText>,
) -> Response {
    let mut response = ui.add(RadioButton::new(*current == alternative, text));
    if response.clicked() && *current != alternative {
        *current = alternative;
        response.mark_changed();
    }
    response
}

impl Widget for RadioButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = FluentTheme::of(ui.ctx());
        let r = &theme.resources;
        let label = self
            .text
            .as_ref()
            .map(|t| t.text().to_owned())
            .unwrap_or_default();
        let (response, circle, text) = allocate_labeled(
            ui,
            Vec2::splat(20.0),
            6.0,
            self.min_height,
            self.text,
            Sense::click(),
        );
        response.widget_info(|| {
            WidgetInfo::selected(Role::RadioButton, ui.is_enabled(), self.checked, &label)
        });

        if ui.is_rect_visible(response.rect) {
            let state = ControlState::from_response(&response);
            let painter = ui.painter();
            let center = circle.center();
            let radius = circle.width() / 2.0;
            let dur = theme.fast_animation_duration.as_secs_f32();

            // fluent_ui draws a thick border around a small dot; animate the
            // border width the same way.
            let (target_width, ring, dot) = if self.checked {
                let width = if state.disabled {
                    4.0
                } else if state.hovered && !state.pressed {
                    3.4
                } else {
                    5.0
                };
                (
                    width,
                    accent_fill(&theme).resolve(state),
                    r.text_on_accent_fill_color_primary,
                )
            } else {
                let fill = States::new(
                    r.control_alt_fill_color_secondary,
                    r.control_alt_fill_color_tertiary,
                    r.control_alt_fill_color_quarternary,
                    r.control_alt_fill_color_disabled,
                )
                .resolve(state);
                let ring = States::new(
                    r.text_fill_color_tertiary,
                    r.text_fill_color_tertiary,
                    theme.accent_color.default_brush_for(theme.brightness),
                    r.text_fill_color_disabled,
                )
                .resolve(state);
                (if state.pressed { 4.5 } else { 1.0 }, ring, fill)
            };
            let width =
                ui.ctx()
                    .animate_value_with_time(response.id.with("ring"), target_width, dur);
            // Paint the dot first and the ring over it, so translucent fills
            // don't let the ring color show through.
            painter.circle_filled(center, radius - width / 2.0, dot);
            painter.circle_stroke(center, radius - width / 2.0, Stroke::new(width, ring));
            if let Some((pos, galley)) = text {
                let color = if state.disabled {
                    r.text_fill_color_disabled
                } else {
                    r.text_fill_color_primary
                };
                painter.galley(pos, galley, color);
            }
            if state.focused {
                paint_focus_border(painter, circle, &theme);
            }
        }
        response
    }
}
