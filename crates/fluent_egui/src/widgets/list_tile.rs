use egui::{
    Rect, Response, Role, Sense, TextWrapMode, Ui, Vec2, Widget, WidgetInfo, WidgetText, pos2, vec2,
};

use super::checkbox::paint_box;
use crate::FluentTheme;
use crate::utils::{ControlState, States, animate_color, paint_focus_border};

/// How a [`ListTile`] shows that it is selected, fluent_ui's
/// `ListTileSelectionMode`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListTileSelectionMode {
    #[default]
    None,
    /// An accent pill on the left, for lists where one item is selected.
    Single,
    /// A checkbox on the left.
    Multiple,
}

/// A row with an optional icon, a title, a subtitle and trailing text,
/// fluent_ui's `ListTile`. `.clicked()` on the response toggles or picks it.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui, selected: &mut usize) {
/// use fluent_egui::{ListTile, ListTileSelectionMode};
///
/// for (i, name) in ["Inbox", "Archive"].into_iter().enumerate() {
///     let tile = ListTile::new(name)
///         .selection_mode(ListTileSelectionMode::Single)
///         .selected(*selected == i);
///     if ui.add(tile).clicked() {
///         *selected = i;
///     }
/// }
/// # }
/// ```
pub struct ListTile {
    title: WidgetText,
    subtitle: Option<WidgetText>,
    leading: Option<WidgetText>,
    trailing: Option<WidgetText>,
    selected: bool,
    selection_mode: ListTileSelectionMode,
}

impl ListTile {
    pub fn new(title: impl Into<WidgetText>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            leading: None,
            trailing: None,
            selected: false,
            selection_mode: ListTileSelectionMode::None,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<WidgetText>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// An icon or other short text before the title.
    pub fn leading(mut self, leading: impl Into<WidgetText>) -> Self {
        self.leading = Some(leading.into());
        self
    }

    /// Text at the end of the row, like a date or a count.
    pub fn trailing(mut self, trailing: impl Into<WidgetText>) -> Self {
        self.trailing = Some(trailing.into());
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn selection_mode(mut self, mode: ListTileSelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }
}

impl Widget for ListTile {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = FluentTheme::of(ui.ctx());
        let r = &theme.resources;
        let width = ui.available_width().max(88.0 + 8.0);
        // Left part: a 12px gap, a checkbox, or room for the pill.
        let lead_in = match self.selection_mode {
            ListTileSelectionMode::None => 12.0,
            ListTileSelectionMode::Single => 3.0 + 8.0,
            ListTileSelectionMode::Multiple => 6.0 + 20.0 + 12.0,
        };
        let layout = |t: WidgetText, size: f32, wrap: f32| {
            t.into_galley(
                ui,
                Some(TextWrapMode::Truncate),
                wrap,
                egui::FontId::new(size, theme.typography.body.font_id().family),
            )
        };
        let leading = self.leading.map(|t| layout(t, 16.0, f32::INFINITY));
        let trailing = self
            .trailing
            .map(|t| layout(t, theme.typography.body.size, f32::INFINITY));
        let text_left = lead_in + leading.as_ref().map_or(0.0, |g| g.size().x + 14.0);
        let text_width =
            (width - 8.0 - text_left - 12.0 - trailing.as_ref().map_or(0.0, |g| g.size().x + 12.0))
                .max(0.0);
        let title = layout(self.title.clone(), 16.0, text_width);
        let subtitle = self
            .subtitle
            .map(|t| layout(t, theme.typography.caption.size, text_width));
        let text_height = title.size().y + subtitle.as_ref().map_or(0.0, |g| g.size().y);
        let tile_height = (text_height + 12.0).max(40.0);

        let (rect, response) =
            ui.allocate_exact_size(vec2(width, tile_height + 4.0), Sense::click());
        let label = self.title.text().to_owned();
        response.widget_info(|| {
            let mut info = WidgetInfo::labeled(Role::ListItem, ui.is_enabled(), &label);
            if self.selection_mode != ListTileSelectionMode::None {
                info.selected = Some(self.selected);
            }
            info
        });
        if !ui.is_rect_visible(rect) {
            return response;
        }

        let tile = rect.shrink2(vec2(4.0, 2.0));
        let mut state = ControlState::from_response(&response);
        let fills = States::new(
            r.subtle_fill_color_transparent,
            r.subtle_fill_color_secondary,
            r.subtle_fill_color_tertiary,
            r.subtle_fill_color_transparent,
        );
        // A selected tile looks hovered at rest.
        let fill = if self.selected && !state.pressed && !state.disabled {
            fills.hovered
        } else {
            fills.resolve(state)
        };
        let fill = animate_color(
            ui.ctx(),
            response.id.with("fill"),
            fill,
            theme.faster_animation_duration,
        );
        let painter = ui.painter();
        painter.rect_filled(tile, 4.0, fill);

        match self.selection_mode {
            ListTileSelectionMode::None => {}
            ListTileSelectionMode::Single => {
                // The pill grows in when selected and shrinks while pressed.
                let target = if !self.selected {
                    0.0
                } else if state.pressed {
                    0.3
                } else {
                    1.0
                };
                let t = ui.ctx().animate_value_with_time(
                    response.id.with("pill"),
                    target,
                    theme.medium_animation_duration.as_secs_f32(),
                );
                let height = (tile.height() - 12.0) * 0.7 * t;
                if height > 0.5 {
                    let pill = Rect::from_center_size(
                        pos2(tile.left() + 1.5, tile.center().y),
                        vec2(3.0, height),
                    );
                    painter.rect_filled(
                        pill,
                        1.5,
                        theme.accent_color.default_brush_for(theme.brightness),
                    );
                }
            }
            ListTileSelectionMode::Multiple => {
                let boxed = Rect::from_min_size(
                    pos2(tile.left() + 6.0, tile.center().y - 10.0),
                    Vec2::splat(20.0),
                );
                state.focused = false;
                paint_box(
                    ui,
                    response.id.with("check"),
                    boxed,
                    state,
                    Some(self.selected),
                );
            }
        }

        let color = if state.disabled {
            r.text_fill_color_disabled
        } else {
            r.text_fill_color_primary
        };
        let painter = ui.painter();
        let mut x = tile.left() + lead_in;
        if let Some(leading) = leading {
            let w = leading.size().x;
            painter.galley(
                pos2(x, tile.center().y - leading.size().y / 2.0),
                leading,
                color,
            );
            x += w + 14.0;
        }
        let mut y = tile.center().y - text_height / 2.0;
        let title_height = title.size().y;
        painter.galley(pos2(x, y), title, color);
        y += title_height;
        if let Some(subtitle) = subtitle {
            painter.galley(pos2(x, y), subtitle, r.text_fill_color_secondary);
        }
        if let Some(trailing) = trailing {
            painter.galley(
                pos2(
                    tile.right() - 12.0 - trailing.size().x,
                    tile.center().y - trailing.size().y / 2.0,
                ),
                trailing,
                r.text_fill_color_secondary,
            );
        }
        if response.has_focus() {
            paint_focus_border(painter, tile, &theme);
        }
        response
    }
}
