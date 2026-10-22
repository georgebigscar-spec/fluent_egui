use egui::{Rect, Response, Role, Sense, TextWrapMode, Ui, WidgetInfo, WidgetText, pos2, vec2};

use super::{MenuItem, menu::show_menu};
use crate::FluentTheme;
use crate::theme::TextStyle;
use crate::utils::{ControlState, paint_chevron_right, paint_focus_border};

/// Width taken by a chevron between items: 8px with 6px on each side.
const CHEVRON_WIDTH: f32 = 20.0;
/// Width of the "more" button shown when leading items overflow.
const MORE_WIDTH: f32 = 28.0;

/// A path of items, the last one being the current location, fluent_ui's
/// `BreadcrumbBar`. Items that don't fit collapse, from the start, into a
/// "more" menu.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui, path: &mut Vec<String>) {
/// if let Some(i) = fluent_egui::BreadcrumbBar::new(path.iter().map(String::as_str)).show(ui) {
///     path.truncate(i + 1);
/// }
/// # }
/// ```
pub struct BreadcrumbBar {
    items: Vec<WidgetText>,
    text_style: Option<TextStyle>,
}

impl BreadcrumbBar {
    pub fn new(items: impl IntoIterator<Item = impl Into<WidgetText>>) -> Self {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            text_style: None,
        }
    }

    /// The text style of the items, `body` by default. Use e.g. `title` for
    /// a page header.
    pub fn text_style(mut self, style: TextStyle) -> Self {
        self.text_style = Some(style);
        self
    }

    /// Returns the index of the clicked item. The last item can't be clicked.
    pub fn show(self, ui: &mut Ui) -> Option<usize> {
        let theme = FluentTheme::of(ui.ctx());
        let font = self
            .text_style
            .as_ref()
            .unwrap_or(&theme.typography.body)
            .font_id();
        let galleys: Vec<_> = self
            .items
            .iter()
            .map(|t| {
                t.clone()
                    .into_galley(ui, Some(TextWrapMode::Extend), f32::INFINITY, font.clone())
            })
            .collect();
        let n = galleys.len();
        if n == 0 {
            return None;
        }

        // Collapse leading items until the rest fits, but keep the last one.
        let available = ui.available_width();
        let width_from = |first: usize| -> f32 {
            galleys[first..].iter().map(|g| g.size().x).sum::<f32>()
                + CHEVRON_WIDTH * (n - first - 1) as f32
        };
        let first = (0..n)
            .find(|&k| {
                let more = if k > 0 {
                    MORE_WIDTH + CHEVRON_WIDTH
                } else {
                    0.0
                };
                more + width_from(k) <= available
            })
            .unwrap_or(n - 1);

        let mut clicked = None;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            if first > 0 {
                let more = more_button(ui, font.size);
                chevron(ui, font.size);
                show_menu(&theme, &more, None, |ui| {
                    for (i, item) in self.items[..first].iter().enumerate() {
                        if ui.add(MenuItem::new(item.clone())).clicked() {
                            clicked = Some(i);
                        }
                    }
                });
            }
            for (i, galley) in galleys.into_iter().enumerate().skip(first) {
                let last = i + 1 == n;
                let label = self.items[i].text().to_owned();
                let (rect, response) = ui.allocate_exact_size(
                    galley.size(),
                    if last { Sense::hover() } else { Sense::click() },
                );
                response.widget_info(|| WidgetInfo::labeled(Role::Link, true, &label));
                let state = ControlState::from_response(&response);
                // fluent_ui's `buttonForegroundColor`.
                let r = &theme.resources;
                let color = state.pick(
                    r.text_fill_color_primary,
                    r.text_fill_color_primary,
                    r.text_fill_color_secondary,
                    r.text_fill_color_disabled,
                );
                ui.painter().galley(rect.min, galley, color);
                if state.focused {
                    paint_focus_border(ui.painter(), rect.expand(2.0), &theme);
                }
                if response.clicked() {
                    clicked = Some(i);
                }
                if !last {
                    chevron(ui, font.size);
                }
            }
        });
        clicked
    }
}

fn chevron(ui: &mut Ui, font_size: f32) {
    let theme = FluentTheme::of(ui.ctx());
    let height = ui.fonts_mut(|f| f.row_height(&egui::FontId::proportional(font_size)));
    let (rect, _) = ui.allocate_exact_size(vec2(CHEVRON_WIDTH, height), Sense::hover());
    paint_chevron_right(
        ui.painter(),
        rect.center(),
        theme.resources.text_fill_color_primary,
    );
}

/// The `more` (three dots) button that opens the overflow menu.
fn more_button(ui: &mut Ui, font_size: f32) -> Response {
    let theme = FluentTheme::of(ui.ctx());
    let r = &theme.resources;
    let height = ui
        .fonts_mut(|f| f.row_height(&egui::FontId::proportional(font_size)))
        .max(19.0);
    let (slot, response) = ui.allocate_exact_size(vec2(MORE_WIDTH, height), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, "More"));
    let rect = Rect::from_center_size(slot.center(), vec2(MORE_WIDTH - 8.0, height));
    let state = ControlState::from_response(&response);
    let color = state.pick(
        r.text_fill_color_primary,
        r.text_fill_color_primary,
        r.text_fill_color_secondary,
        r.text_fill_color_disabled,
    );
    let fill = state.pick(
        r.subtle_fill_color_transparent,
        r.subtle_fill_color_secondary,
        r.subtle_fill_color_tertiary,
        r.subtle_fill_color_transparent,
    );
    ui.painter().rect_filled(rect, 4.0, fill);
    for dx in [-5.0, 0.0, 5.0] {
        ui.painter()
            .circle_filled(pos2(rect.center().x + dx, rect.center().y), 1.0, color);
    }
    if state.focused {
        paint_focus_border(ui.painter(), rect, &theme);
    }
    response
}
