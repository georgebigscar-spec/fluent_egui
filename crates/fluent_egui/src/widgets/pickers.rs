//! The parts [`super::DatePicker`] and [`super::TimePicker`] share,
//! fluent_ui's `lib/src/controls/pickers/pickers.dart`.

use egui::{
    Align, Align2, Area, Color32, Id, Key, Order, Rect, Response, Role, Sense, Shadow, Stroke,
    StrokeKind, Ui, WidgetInfo, pos2, vec2,
};

use super::button::{ButtonStyle, paint_border};
use super::calendar_view::paint_caret;
use crate::FluentTheme;
use crate::utils::{
    ControlState, States, animate_color, paint_check, paint_close, paint_focus_border, wheel_steps,
    with_alpha,
};

/// fluent_ui's `kPickerHeight`.
pub(crate) const PICKER_HEIGHT: f32 = 32.0;
/// fluent_ui's `kOneLineTileHeight`, the height of a row in the popup.
const ROW_HEIGHT: f32 = 40.0;
/// Rows above and below the highlighted one.
const HALF_ROWS: i64 = 4;
const FOOTER_HEIGHT: f32 = ROW_HEIGHT;

/// One segment of a picker button and its column in the popup.
pub(crate) struct PickerColumn {
    pub items: Vec<String>,
    pub width: f32,
    /// Whether scrolling past the end wraps around (months, days, minutes).
    pub looping: bool,
    pub align: Align,
    /// Shown on the button when there is no value.
    pub placeholder: &'static str,
}

impl PickerColumn {
    /// The item shown at (unbounded) position `index`.
    fn item(&self, index: i64) -> Option<&str> {
        let n = self.items.len() as i64;
        if n == 0 {
            return None;
        }
        let i = if self.looping {
            index.rem_euclid(n)
        } else if (0..n).contains(&index) {
            index
        } else {
            return None;
        };
        Some(&self.items[i as usize])
    }

    /// Moves `index` by `by` rows. Looping columns keep counting past the
    /// end (so the slide animation takes the short way); others clamp.
    fn advance(&self, index: i64, by: i64) -> i64 {
        if self.looping {
            index + by
        } else {
            self.normalize(index + by)
        }
    }

    /// Brings a position back into range: wrapped, or clamped.
    pub fn normalize(&self, index: i64) -> i64 {
        let n = self.items.len() as i64;
        if self.looping {
            index.rem_euclid(n.max(1))
        } else {
            index.clamp(0, (n - 1).max(0))
        }
    }
}

/// The popup's working copy of the selection, one position per column.
#[derive(Clone, Default)]
pub(crate) struct PickerPopupState {
    pub open: bool,
    pub indices: Vec<i64>,
}

impl PickerPopupState {
    pub fn load(ui: &Ui, id: Id) -> Self {
        ui.data(|d| d.get_temp::<Self>(id.with("popup")))
            .unwrap_or_default()
    }

    pub fn store(self, ui: &Ui, id: Id) {
        ui.data_mut(|d| d.insert_temp(id.with("popup"), self));
    }
}

/// The button of a date or time picker: one segment per column with
/// dividers between them. `values` holds each segment's text, or `None`
/// for placeholders.
pub(crate) fn picker_button(
    ui: &mut Ui,
    id: Id,
    columns: &[PickerColumn],
    values: Option<Vec<String>>,
    label: &str,
    open: bool,
) -> Response {
    let theme = FluentTheme::of(ui.ctx());
    let r = &theme.resources;
    let width: f32 = columns.iter().map(|c| c.width).sum();
    let (rect, _) = ui.allocate_exact_size(vec2(width, PICKER_HEIGHT), Sense::hover());
    let response = ui.interact(rect, id, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), label));
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
        let mut x = rect.left();
        for (i, column) in columns.iter().enumerate() {
            let cell = Rect::from_min_size(pos2(x, rect.top()), vec2(column.width, rect.height()));
            let (text, color) = match &values {
                Some(values) => (values[i].as_str(), style.foreground.resolve(state)),
                None => (
                    column.placeholder,
                    if state.disabled {
                        r.text_fill_color_disabled
                    } else {
                        r.text_fill_color_secondary
                    },
                ),
            };
            let (pos, align) = match column.align {
                Align::Min => (
                    pos2(cell.left() + 12.0, cell.center().y),
                    Align2::LEFT_CENTER,
                ),
                _ => (cell.center(), Align2::CENTER_CENTER),
            };
            painter.text(pos, align, text, theme.typography.body.font_id(), color);
            if i > 0 {
                painter.vline(
                    cell.left(),
                    rect.y_range().shrink(1.0),
                    Stroke::new(1.0, r.control_stroke_color_default),
                );
            }
            x += column.width;
        }
        if state.focused {
            paint_focus_border(painter, rect, &theme);
        }
    }
    response
}

/// What the user did in the popup this frame.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PopupAction {
    Accept,
    Cancel,
}

/// The wheel popup over `button`: a column per [`PickerColumn`] with the
/// selected row highlighted in line with the button, and accept and cancel
/// buttons below. Edits `indices` in place.
pub(crate) fn picker_popup(
    ui: &Ui,
    id: Id,
    button: Rect,
    columns: &[PickerColumn],
    indices: &mut [i64],
) -> Option<PopupAction> {
    let theme = FluentTheme::of(ui.ctx());
    let r = &theme.resources;
    let width = button.width().max(260.0);
    let rows_height = (2 * HALF_ROWS + 1) as f32 * ROW_HEIGHT;
    let size = vec2(width, rows_height + 1.0 + FOOTER_HEIGHT);
    // The highlighted row sits over the button.
    let mut origin = pos2(
        button.center().x - width / 2.0,
        button.center().y - rows_height / 2.0,
    );
    let screen = ui.ctx().content_rect();
    origin.y = origin
        .y
        .clamp(screen.top(), (screen.bottom() - size.y).max(screen.top()));
    origin.x = origin
        .x
        .clamp(screen.left(), (screen.right() - size.x).max(screen.left()));
    let popup = Rect::from_min_size(origin, size);

    let mut action = None;
    Area::new(id.with("picker_popup"))
        .order(Order::Foreground)
        .fixed_pos(origin)
        .constrain(false)
        .show(ui.ctx(), |ui| {
            ui.allocate_rect(popup, Sense::click());
            let shadow = Shadow {
                offset: [0, 8],
                blur: 16,
                spread: 0,
                color: with_alpha(
                    Color32::BLACK,
                    if theme.brightness.is_dark() {
                        0.26
                    } else {
                        0.14
                    },
                ),
            };
            ui.painter().add(shadow.as_shape(popup, 8));
            ui.painter().rect(
                popup,
                8.0,
                theme.menu_color,
                Stroke::new(1.0, r.surface_stroke_color_flyout),
                StrokeKind::Inside,
            );

            // The highlight spans every column, fluent_ui's `PickerHighlightTile`.
            let center_row = Rect::from_min_size(
                pos2(popup.left(), popup.top() + HALF_ROWS as f32 * ROW_HEIGHT),
                vec2(width, ROW_HEIGHT),
            );
            ui.painter().rect_filled(
                center_row.shrink2(vec2(4.0, 2.0)),
                4.0,
                theme.accent_color.default_brush_for(theme.brightness),
            );

            let total: f32 = columns.iter().map(|c| c.width).sum();
            let scale = width / total;
            let mut x = popup.left();
            for (i, (column, index)) in columns.iter().zip(indices.iter_mut()).enumerate() {
                let col_rect = Rect::from_min_size(
                    pos2(x, popup.top()),
                    vec2(column.width * scale, rows_height),
                );
                wheel_column(ui, id.with(("column", i)), col_rect, column, index);
                if i > 0 {
                    ui.painter().vline(
                        col_rect.left(),
                        col_rect.y_range(),
                        Stroke::new(1.0, theme.divider.color),
                    );
                }
                x = col_rect.right();
            }

            let footer_top = popup.top() + rows_height;
            ui.painter().hline(
                popup.x_range(),
                footer_top + 0.5,
                Stroke::new(1.0, theme.divider.color),
            );
            // fluent_ui's `YesNoPickerControl`.
            let half = width / 2.0;
            for (i, accept) in [(0.0, true), (1.0, false)] {
                let rect = Rect::from_min_size(
                    pos2(popup.left() + i * half, footer_top + 1.0),
                    vec2(half, FOOTER_HEIGHT),
                )
                .shrink(4.0);
                let response = ui.interact(rect, id.with(("footer", accept)), Sense::click());
                let state = ControlState::from_response(&response);
                let fill = States::new(
                    r.subtle_fill_color_transparent,
                    r.subtle_fill_color_secondary,
                    r.subtle_fill_color_tertiary,
                    r.subtle_fill_color_transparent,
                )
                .resolve(state);
                ui.painter().rect_filled(rect, 4.0, fill);
                let color = r.text_fill_color_primary;
                if accept {
                    paint_check(ui.painter(), rect.center(), 12.0, color);
                } else {
                    paint_close(ui.painter(), rect.center(), color);
                }
                if response.clicked() {
                    action = Some(if accept {
                        PopupAction::Accept
                    } else {
                        PopupAction::Cancel
                    });
                }
            }
        });

    let (enter, escape, outside) = ui.input(|i| {
        (
            i.key_pressed(Key::Enter),
            i.key_pressed(Key::Escape),
            i.pointer.any_pressed()
                && i.pointer
                    .interact_pos()
                    .is_some_and(|p| !popup.contains(p) && !button.contains(p)),
        )
    });
    if enter {
        action = Some(PopupAction::Accept);
    } else if escape || outside {
        action = Some(PopupAction::Cancel);
    }
    action
}

/// One scrolling column: the wheel steps it, clicking a row picks it, and
/// while hovered the top and bottom rows turn into step buttons.
fn wheel_column(ui: &mut Ui, id: Id, rect: Rect, column: &PickerColumn, index: &mut i64) {
    let theme = FluentTheme::of(ui.ctx());
    let r = &theme.resources;
    let step = wheel_steps(ui, rect);
    if step != 0 {
        *index = column.advance(*index, i64::from(step));
    }
    // Slide between positions. Looping columns animate the short way round.
    let shown = ui.ctx().animate_value_with_time(
        id.with("offset"),
        *index as f32,
        theme.fast_animation_duration.as_secs_f32(),
    );
    let offset = shown - *index as f32;
    let painter = ui.painter_at(rect);
    let hovered = ui.rect_contains_pointer(rect);
    let row_rect = |k: i64| {
        Rect::from_min_size(
            pos2(
                rect.left(),
                rect.top() + (k + HALF_ROWS) as f32 * ROW_HEIGHT,
            ),
            vec2(rect.width(), ROW_HEIGHT),
        )
    };
    for k in -HALF_ROWS - 1..=HALF_ROWS + 1 {
        let Some(text) = column.item(*index + k) else {
            continue;
        };
        let row = row_rect(k).translate(vec2(0.0, -offset * ROW_HEIGHT));
        let selected = k == 0;
        let color = if selected {
            r.text_on_accent_fill_color_primary
        } else {
            r.text_fill_color_primary
        };
        let (pos, align) = match column.align {
            Align::Min => (pos2(row.left() + 12.0, row.center().y), Align2::LEFT_CENTER),
            _ => (row.center(), Align2::CENTER_CENTER),
        };
        painter.text(pos, align, text, theme.typography.body.font_id(), color);
        if !selected && k.abs() <= HALF_ROWS {
            let response = ui.interact(row_rect(k), id.with(("row", k)), Sense::click());
            let edge = hovered && k.abs() == HALF_ROWS;
            if response.clicked() {
                let target = if edge { k.signum() } else { k };
                if column.item(*index + target).is_some() {
                    *index = column.advance(*index, target);
                }
            }
        }
    }
    if hovered {
        for dir in [-1.0_f32, 1.0] {
            let row = row_rect(dir as i64 * HALF_ROWS);
            painter.rect_filled(row, 0.0, theme.menu_color);
            paint_caret(
                &painter,
                row.center(),
                dir,
                with_alpha(r.text_fill_color_secondary, 0.8),
            );
        }
    }
}
