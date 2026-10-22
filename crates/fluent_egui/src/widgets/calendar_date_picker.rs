use egui::{
    Align2, AsIdSalt, IdSalt, Popup, PopupCloseBehavior, Rect, Response, Role, Sense, Stroke, Ui,
    Widget, WidgetInfo, pos2, vec2,
};

use super::button::{ButtonStyle, paint_border};
use super::{CalendarView, flyout_frame};
use crate::FluentTheme;
use crate::date::Date;
use crate::utils::{ControlState, animate_color, paint_focus_border};

/// A button showing a date that opens a [`CalendarView`] flyout,
/// fluent_ui's `CalendarDatePicker`.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui, due: &mut Option<fluent_egui::Date>) {
/// ui.add(fluent_egui::CalendarDatePicker::new("due", due));
/// # }
/// ```
pub struct CalendarDatePicker<'a> {
    id_salt: IdSalt,
    value: &'a mut Option<Date>,
    placeholder: String,
    min_date: Option<Date>,
    max_date: Option<Date>,
}

impl<'a> CalendarDatePicker<'a> {
    pub fn new(id_salt: impl AsIdSalt, value: &'a mut Option<Date>) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            value,
            placeholder: "Pick a date".to_owned(),
            min_date: None,
            max_date: None,
        }
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn min_date(mut self, date: Date) -> Self {
        self.min_date = Some(date);
        self
    }

    pub fn max_date(mut self, date: Date) -> Self {
        self.max_date = Some(date);
        self
    }
}

impl Widget for CalendarDatePicker<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = FluentTheme::of(ui.ctx());
        let r = &theme.resources;
        let id = ui.make_persistent_id(self.id_salt);
        let text = self
            .value
            .map(|d| format!("{}/{}/{}", d.month, d.day, d.year));
        let (rect, _) = ui.allocate_exact_size(vec2(160.0, 32.0), Sense::hover());
        let mut response = ui.interact(rect, id, Sense::click());
        let label = text.clone().unwrap_or_else(|| self.placeholder.clone());
        response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), &label));
        let popup_id = Popup::default_response_id(&response);
        let open = Popup::is_id_open(ui.ctx(), popup_id);

        if ui.is_rect_visible(rect) {
            let mut state = ControlState::from_response(&response);
            state.pressed |= open;
            let style = ButtonStyle::standard(&theme);
            let fill = animate_color(
                ui.ctx(),
                id.with("fill"),
                style.background.resolve(state),
                theme.faster_animation_duration,
            );
            let painter = ui.painter();
            painter.rect_filled(rect, style.corner_radius, fill);
            paint_border(
                painter,
                rect,
                style.corner_radius,
                style.border.resolve(state),
            );
            let color = if text.is_some() {
                style.foreground.resolve(state)
            } else {
                r.text_fill_color_secondary
            };
            painter.text(
                pos2(rect.left() + 11.0, rect.center().y),
                Align2::LEFT_CENTER,
                &label,
                theme.typography.body.font_id(),
                color,
            );
            paint_calendar_glyph(
                painter,
                Rect::from_center_size(
                    pos2(rect.right() - 18.0, rect.center().y),
                    vec2(14.0, 13.0),
                ),
                style.foreground.resolve(state),
            );
            if state.focused {
                paint_focus_border(painter, rect, &theme);
            }
        }

        let mut changed = false;
        Popup::menu(&response)
            .frame(flyout_frame(&theme).inner_margin(0))
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .gap(4.0)
            .show(|ui| {
                let mut calendar = CalendarView::new(id.with("calendar"), self.value);
                if let Some(min) = self.min_date {
                    calendar = calendar.min_date(min);
                }
                if let Some(max) = self.max_date {
                    calendar = calendar.max_date(max);
                }
                changed = ui.add(calendar).changed();
            });
        if changed {
            Popup::close_id(ui.ctx(), popup_id);
            response.mark_changed();
        }
        response
    }
}

/// The `calendar` glyph: a box with a header bar and two rings.
fn paint_calendar_glyph(painter: &egui::Painter, rect: Rect, color: egui::Color32) {
    let stroke = Stroke::new(1.0, color);
    painter.rect_stroke(rect, 2.0, stroke, egui::StrokeKind::Inside);
    painter.hline(rect.x_range().shrink(1.0), rect.top() + 4.0, stroke);
    for x in [rect.left() + 4.0, rect.right() - 4.0] {
        painter.vline(x, (rect.top() - 2.0)..=(rect.top() + 1.5), stroke);
    }
}
