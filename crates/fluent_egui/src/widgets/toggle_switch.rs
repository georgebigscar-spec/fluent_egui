use egui::{
    Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, Vec2, Widget, WidgetInfo, WidgetText, pos2,
};

use super::allocate_labeled;
use super::button::accent_fill;
use crate::FluentTheme;
use crate::utils::{ControlState, States, animate_color, paint_focus_border};

/// A Fluent toggle switch, fluent_ui's `ToggleSwitch`. Click or drag the
/// knob to change it.
pub struct ToggleSwitch<'a> {
    checked: &'a mut bool,
    text: Option<WidgetText>,
}

impl<'a> ToggleSwitch<'a> {
    pub fn new(checked: &'a mut bool, text: impl Into<WidgetText>) -> Self {
        Self {
            checked,
            text: Some(text.into()),
        }
    }

    pub fn without_text(checked: &'a mut bool) -> Self {
        Self {
            checked,
            text: None,
        }
    }
}

impl Widget for ToggleSwitch<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = FluentTheme::of(ui.ctx());
        let r = &theme.resources;
        let label = self
            .text
            .as_ref()
            .map(|t| t.text().to_owned())
            .unwrap_or_default();
        let (mut response, track, text) = allocate_labeled(
            ui,
            Vec2::new(40.0, 20.0),
            10.0,
            self.text,
            Sense::click_and_drag(),
        );

        // While dragging, the knob follows the pointer; on release the side
        // it ends up on decides the value, like fluent_ui.
        let drag_id = response.id.with("drag");
        let mut drag_pos: Option<f32> = ui.data(|d| d.get_temp(drag_id));
        if response.dragged()
            && let Some(p) = response.interact_pointer_pos()
        {
            let t = ((p.x - track.left()) / track.width()).clamp(0.0, 1.0);
            drag_pos = Some(t);
            ui.data_mut(|d| d.insert_temp(drag_id, t));
        }
        if response.drag_stopped() {
            if let Some(t) = drag_pos.take() {
                let value = t >= 0.5;
                if value != *self.checked {
                    *self.checked = value;
                    response.mark_changed();
                }
            }
            ui.data_mut(|d| d.remove::<f32>(drag_id));
        } else if response.clicked() {
            *self.checked = !*self.checked;
            response.mark_changed();
        }
        let checked = *self.checked;
        response
            .widget_info(|| WidgetInfo::selected(Role::Switch, ui.is_enabled(), checked, &label));

        if ui.is_rect_visible(response.rect) {
            let mut state = ControlState::from_response(&response);
            state.pressed |= drag_pos.is_some();
            let painter = ui.painter();
            let dur = theme.faster_animation_duration;
            let (fill, border) = if checked {
                let c = accent_fill(&theme).resolve(state);
                (c, c)
            } else {
                let fill = States::new(
                    r.control_alt_fill_color_secondary,
                    r.control_alt_fill_color_tertiary,
                    r.control_alt_fill_color_quarternary,
                    r.control_alt_fill_color_disabled,
                )
                .resolve(state);
                let border = if state.disabled {
                    r.control_strong_fill_color_disabled
                } else {
                    r.control_strong_fill_color_default
                };
                (fill, border)
            };
            let id = response.id;
            let fill = animate_color(ui.ctx(), id.with("fill"), fill, dur);
            let border = animate_color(ui.ctx(), id.with("border"), border, dur);
            painter.rect_filled(track, 10.0, fill);
            painter.rect_stroke(track, 10.0, Stroke::new(1.0, border), StrokeKind::Inside);

            // DefaultToggleSwitchKnob: 12x12 at rest, 14x14 on hover and
            // 5px wider while pressed, centered vertically in the track.
            let grow = if state.hovered || state.pressed {
                2.0
            } else {
                0.0
            };
            let height = 12.0 + grow;
            let width = height + if state.pressed { 5.0 } else { 0.0 };
            let margin = (track.height() - height) / 2.0;
            let width =
                ui.ctx()
                    .animate_value_with_time(id.with("knob_w"), width, dur.as_secs_f32());
            let travel = track.width() - 2.0 * margin - width;
            let target = drag_pos.unwrap_or(if checked { 1.0 } else { 0.0 });
            let t = if drag_pos.is_some() {
                target
            } else {
                ui.ctx()
                    .animate_value_with_time(id.with("knob"), target, dur.as_secs_f32())
            };
            let left = track.left() + margin + travel * t;
            let knob = Rect::from_min_size(
                pos2(left, track.center().y - height / 2.0),
                Vec2::new(width, height),
            );
            let knob_color = match (checked, state.disabled) {
                (true, false) => r.text_on_accent_fill_color_primary,
                (true, true) => r.text_on_accent_fill_color_disabled,
                (false, false) => r.text_fill_color_secondary,
                (false, true) => r.text_fill_color_disabled,
            };
            painter.rect_filled(knob, height / 2.0, knob_color);

            if let Some((pos, galley)) = text {
                let color = if state.disabled {
                    r.text_fill_color_disabled
                } else {
                    r.text_fill_color_primary
                };
                painter.galley(pos, galley, color);
            }
            if state.focused {
                paint_focus_border(painter, track, &theme);
            }
        }
        response
    }
}
