use egui::epaint::PathShape;
use egui::{
    AsIdSalt, Color32, Id, IdSalt, Pos2, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, Vec2,
    Widget, WidgetInfo, pos2, vec2,
};

use super::button::accent_fill;
use crate::FluentTheme;
use crate::date::{Date, MONTH_NAMES, Weekday};
use crate::utils::{ControlState, States, animate_color, paint_focus_border, wheel_steps};

const WIDTH: f32 = 300.0;
const PADDING: f32 = 8.0;
const HEADER_HEIGHT: f32 = 32.0;
/// fluent_ui's `_rowHeight`, one week.
const ROW_HEIGHT: f32 = 40.0;
const WEEKDAY_ROW: f32 = 24.0;
const GRID_HEIGHT: f32 = WEEKDAY_ROW + 6.0 * ROW_HEIGHT;
const HEIGHT: f32 = PADDING + HEADER_HEIGHT + 4.0 + 1.0 + 4.0 + GRID_HEIGHT + PADDING;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Month,
    Year,
    Decade,
}

#[derive(Clone, Copy, Debug)]
struct CalendarState {
    mode: Mode,
    /// The month (or year, or decade) being shown.
    shown: Date,
}

/// A month grid for picking a date, with year and decade views a click on
/// the header away. fluent_ui's `CalendarView` in single selection mode.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui, date: &mut Option<fluent_egui::Date>) {
/// if ui.add(fluent_egui::CalendarView::new("calendar", date)).changed() {
///     println!("picked {date:?}");
/// }
/// # }
/// ```
pub struct CalendarView<'a> {
    id_salt: IdSalt,
    selected: &'a mut Option<Date>,
    min_date: Option<Date>,
    max_date: Option<Date>,
    first_day_of_week: Weekday,
    today: Option<Date>,
}

impl<'a> CalendarView<'a> {
    pub fn new(id_salt: impl AsIdSalt, selected: &'a mut Option<Date>) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            selected,
            min_date: None,
            max_date: None,
            first_day_of_week: Weekday::Sunday,
            today: None,
        }
    }

    /// Days before this one can't be picked and are struck through.
    pub fn min_date(mut self, date: Date) -> Self {
        self.min_date = Some(date);
        self
    }

    /// Days after this one can't be picked and are struck through.
    pub fn max_date(mut self, date: Date) -> Self {
        self.max_date = Some(date);
        self
    }

    pub fn first_day_of_week(mut self, day: Weekday) -> Self {
        self.first_day_of_week = day;
        self
    }

    /// The highlighted "today". Defaults to [`Date::today`], which is UTC.
    pub fn today(mut self, date: Date) -> Self {
        self.today = Some(date);
        self
    }

    fn in_range(&self, date: Date) -> bool {
        self.min_date.is_none_or(|min| date >= min) && self.max_date.is_none_or(|max| date <= max)
    }
}

impl Widget for CalendarView<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = FluentTheme::of(ui.ctx());
        let r = &theme.resources;
        let id = ui.make_persistent_id(self.id_salt);
        let today = self.today.unwrap_or_else(Date::today);
        let mut state = ui
            .data(|d| d.get_temp::<CalendarState>(id))
            .unwrap_or(CalendarState {
                mode: Mode::Month,
                shown: self.selected.unwrap_or(today),
            });

        let (rect, mut response) = ui.allocate_exact_size(vec2(WIDTH, HEIGHT), Sense::hover());
        response.widget_info(|| WidgetInfo::labeled(Role::Grid, true, "Calendar"));
        let painter = ui.painter().clone();
        painter.rect(
            rect,
            4.0,
            r.control_fill_color_input_active,
            Stroke::new(1.0, r.control_stroke_color_default),
            StrokeKind::Inside,
        );

        // Header: the title (a button that zooms out) and the arrows.
        let top = rect.top() + PADDING;
        let arrow = |i: f32| {
            Rect::from_min_size(
                pos2(
                    rect.right() - PADDING - (i + 1.0) * HEADER_HEIGHT - i * 4.0,
                    top,
                ),
                Vec2::splat(HEADER_HEIGHT),
            )
        };
        let (down_rect, up_rect) = (arrow(0.0), arrow(1.0));
        let title_rect = Rect::from_min_max(
            pos2(rect.left() + PADDING, top),
            pos2(up_rect.left() - 4.0, top + HEADER_HEIGHT),
        );
        let title = match state.mode {
            Mode::Month => format!("{} {}", state.shown.month_name(), state.shown.year),
            Mode::Year => state.shown.year.to_string(),
            Mode::Decade => {
                let start = state.shown.year.div_euclid(10) * 10;
                format!("{start} - {}", start + 9)
            }
        };
        let zoomable = state.mode != Mode::Decade;
        let title_response = subtle_button(
            ui,
            title_rect,
            id.with("title"),
            zoomable,
            |p, rect, color| {
                let galley = p.layout_no_wrap(
                    title.clone(),
                    egui::FontId::new(14.0, theme.typography.subtitle.font_id().family),
                    color,
                );
                p.galley(
                    pos2(rect.left() + 8.0, rect.center().y - galley.size().y / 2.0),
                    galley,
                    color,
                );
            },
        );
        if title_response.clicked() {
            state.mode = match state.mode {
                Mode::Month => Mode::Year,
                _ => Mode::Decade,
            };
        }
        let mut step = 0;
        if subtle_button(ui, up_rect, id.with("up"), true, |p, rect, color| {
            paint_caret(p, rect.center(), -1.0, color)
        })
        .clicked()
        {
            step = -1;
        }
        if subtle_button(ui, down_rect, id.with("down"), true, |p, rect, color| {
            paint_caret(p, rect.center(), 1.0, color)
        })
        .clicked()
        {
            step = 1;
        }

        let divider_y = top + HEADER_HEIGHT + 4.0 + 0.5;
        painter.hline(
            rect.x_range().shrink(1.0),
            divider_y,
            Stroke::new(1.0, theme.divider.color),
        );
        let grid = Rect::from_min_size(
            pos2(rect.left() + PADDING, divider_y + 4.5),
            vec2(WIDTH - 2.0 * PADDING, GRID_HEIGHT),
        );

        // The mouse wheel pages like the arrows.
        let wheel = wheel_steps(ui, grid);
        if wheel != 0 {
            step = wheel;
        }
        if step != 0 {
            state.shown = match state.mode {
                Mode::Month => state.shown.add_months(step),
                Mode::Year => state.shown.add_months(12 * step),
                Mode::Decade => state.shown.add_months(120 * step),
            };
        }

        match state.mode {
            Mode::Month => {
                for (i, day) in (0..7).map(|i| (i, (self.first_day_of_week.index() + i) % 7)) {
                    let name = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"][day as usize];
                    let x = grid.left() + (f32::from(i) + 0.5) * grid.width() / 7.0;
                    painter.text(
                        pos2(x, grid.top() + WEEKDAY_ROW / 2.0),
                        egui::Align2::CENTER_CENTER,
                        name,
                        egui::FontId::new(12.0, theme.typography.body.font_id().family),
                        r.text_fill_color_primary,
                    );
                }
                let first = Date::new(state.shown.year, state.shown.month, 1).unwrap_or(today);
                let offset = (i64::from(first.weekday().index())
                    - i64::from(self.first_day_of_week.index()))
                .rem_euclid(7);
                let start = first.add_days(-offset);
                let days = Rect::from_min_max(grid.min + vec2(0.0, WEEKDAY_ROW), grid.max);
                for i in 0..42 {
                    let day = start.add_days(i);
                    let cell = cell_rect(days, 7, 6, i as usize);
                    let selected = *self.selected == Some(day);
                    let cell_state = CellState {
                        filled: day == today,
                        selected,
                        out_of_scope: day.month != state.shown.month,
                        blackout: !self.in_range(day),
                    };
                    let clicked = calendar_cell(
                        ui,
                        id.with(("day", day.to_days())),
                        Rect::from_center_size(cell.center(), Vec2::splat(40.0)),
                        &day.day.to_string(),
                        cell_state,
                    );
                    if clicked && !selected {
                        *self.selected = Some(day);
                        response.mark_changed();
                        if day.month != state.shown.month {
                            state.shown = day;
                        }
                    }
                }
            }
            Mode::Year => {
                // This year's months, then the first four of the next.
                let jan = Date::new(state.shown.year, 1, 1).unwrap_or(today);
                for i in 0..16 {
                    let month = jan.add_months(i as i32);
                    let cell_state = CellState {
                        filled: month.year == today.year && month.month == today.month,
                        selected: false,
                        out_of_scope: month.year != state.shown.year,
                        blackout: false,
                    };
                    let clicked = calendar_cell(
                        ui,
                        id.with(("month", month.year, month.month)),
                        Rect::from_center_size(
                            cell_rect(grid, 4, 4, i).center(),
                            Vec2::splat(56.0),
                        ),
                        &MONTH_NAMES[usize::from(month.month - 1)][..3],
                        cell_state,
                    );
                    if clicked {
                        state.shown = month;
                        state.mode = Mode::Month;
                    }
                }
            }
            Mode::Decade => {
                let start = state.shown.year.div_euclid(10) * 10;
                for i in 0..16 {
                    let year = start + i as i32;
                    let cell_state = CellState {
                        filled: year == today.year,
                        selected: false,
                        out_of_scope: year >= start + 10,
                        blackout: false,
                    };
                    let clicked = calendar_cell(
                        ui,
                        id.with(("year", year)),
                        Rect::from_center_size(
                            cell_rect(grid, 4, 4, i).center(),
                            Vec2::splat(56.0),
                        ),
                        &year.to_string(),
                        cell_state,
                    );
                    if clicked {
                        state.shown = Date::new(year, state.shown.month, 1).unwrap_or(today);
                        state.mode = Mode::Year;
                    }
                }
            }
        }

        ui.data_mut(|d| d.insert_temp(id, state));
        response
    }
}

/// The `index`th cell of a `cols` x `rows` grid filling `area`.
fn cell_rect(area: Rect, cols: usize, rows: usize, index: usize) -> Rect {
    let size = vec2(area.width() / cols as f32, area.height() / rows as f32);
    let min = area.min
        + vec2(
            (index % cols) as f32 * size.x,
            (index / cols) as f32 * size.y,
        );
    Rect::from_min_size(min, size)
}

#[derive(Clone, Copy)]
struct CellState {
    /// Today (or this month, or this year): an accent disc.
    filled: bool,
    /// The selected day: an accent ring.
    selected: bool,
    /// Outside the month (or year, or decade) being shown.
    out_of_scope: bool,
    /// Outside the allowed range: struck through and not clickable.
    blackout: bool,
}

/// A round day, month or year button, fluent_ui's `_CalendarDayItem` and
/// `_CalendarItem`.
fn calendar_cell(ui: &mut Ui, id: Id, rect: Rect, text: &str, cell: CellState) -> bool {
    let theme = FluentTheme::of(ui.ctx());
    let r = &theme.resources;
    let sense = if cell.blackout {
        Sense::hover()
    } else {
        Sense::click()
    };
    let response = ui.interact(rect, id, sense);
    response.widget_info(|| {
        let mut info = WidgetInfo::labeled(Role::Cell, !cell.blackout, text);
        info.selected = Some(cell.selected);
        info
    });
    if !ui.is_rect_visible(rect) {
        return false;
    }
    let state = ControlState::from_response(&response);
    let accent = theme.accent_color.default_brush_for(theme.brightness);
    let fill = if cell.filled {
        accent_fill(&theme).resolve(state)
    } else {
        States::new(
            r.subtle_fill_color_transparent,
            r.subtle_fill_color_secondary,
            r.subtle_fill_color_tertiary,
            r.subtle_fill_color_transparent,
        )
        .resolve(state)
    };
    let fill = animate_color(
        ui.ctx(),
        id.with("fill"),
        fill,
        theme.faster_animation_duration,
    );
    let foreground = if cell.filled {
        r.text_on_accent_fill_color_primary
    } else if cell.blackout {
        r.text_fill_color_primary
    } else if cell.out_of_scope {
        r.text_fill_color_secondary
    } else if cell.selected {
        accent
    } else {
        r.text_fill_color_primary
    };
    let painter = ui.painter();
    let radius = rect.width() / 2.0;
    if cell.selected {
        // A 1px ring with the fill inset inside it.
        painter.circle_stroke(rect.center(), radius - 0.5, Stroke::new(1.0, accent));
        painter.circle_filled(rect.center(), radius - 2.0, fill);
    } else {
        painter.circle_filled(rect.center(), radius, fill);
    }
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        theme.typography.body.font_id(),
        foreground,
    );
    if cell.blackout {
        let pad = rect.width() * 0.3;
        let color = if cell.filled {
            r.text_on_accent_fill_color_primary
        } else {
            r.control_strong_stroke_color_default
        };
        painter.line_segment(
            [
                pos2(rect.right() - pad, rect.top() + pad),
                pos2(rect.left() + pad, rect.bottom() - pad),
            ],
            Stroke::new(1.15, color),
        );
    }
    if state.focused {
        paint_focus_border(painter, rect, &theme);
    }
    response.clicked()
}

/// A transparent button over `rect` with a subtle fill on hover, whose
/// content is painted by `paint` in the foreground color.
fn subtle_button(
    ui: &mut Ui,
    rect: Rect,
    id: Id,
    enabled: bool,
    paint: impl FnOnce(&egui::Painter, Rect, Color32),
) -> Response {
    let theme = FluentTheme::of(ui.ctx());
    let r = &theme.resources;
    let response = ui.interact(
        rect,
        id,
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let state = if enabled {
        ControlState::from_response(&response)
    } else {
        ControlState::default()
    };
    let fill = States::new(
        r.subtle_fill_color_transparent,
        r.subtle_fill_color_secondary,
        r.subtle_fill_color_tertiary,
        r.subtle_fill_color_transparent,
    )
    .resolve(state);
    let fill = animate_color(
        ui.ctx(),
        id.with("fill"),
        fill,
        theme.faster_animation_duration,
    );
    let painter = ui.painter();
    painter.rect_filled(rect, 4.0, fill);
    let color = state.pick(
        r.text_fill_color_primary,
        r.text_fill_color_primary,
        r.text_fill_color_secondary,
        r.text_fill_color_disabled,
    );
    paint(painter, rect, color);
    if state.focused {
        paint_focus_border(painter, rect, &theme);
    }
    response
}

/// `caret_up_solid8` (`dir` -1) or `caret_down_solid8` (`dir` 1).
pub(crate) fn paint_caret(painter: &egui::Painter, center: Pos2, dir: f32, color: Color32) {
    let points = vec![
        center + vec2(-4.0, -2.0 * dir),
        center + vec2(4.0, -2.0 * dir),
        center + vec2(0.0, 2.5 * dir),
    ];
    painter.add(PathShape::convex_polygon(points, color, Stroke::NONE));
}
