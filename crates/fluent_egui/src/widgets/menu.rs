use egui::{
    AsIdSalt, Color32, CornerRadius, Frame, IdSalt, InnerResponse, Margin, Popup, Rect, Response,
    Role, Sense, Shadow, Stroke, TextWrapMode, Ui, Vec2, Widget, WidgetInfo, WidgetText, pos2,
};

use super::button::{Button, ButtonStyle};
use crate::FluentTheme;
use crate::utils::{
    ControlState, States, animate_color, paint_chevron_down, paint_elevation_border,
    paint_focus_border, with_alpha,
};

/// The frame of a flyout or menu: menu color, flyout stroke, 8px corners
/// and a soft shadow.
pub fn flyout_frame(theme: &FluentTheme) -> Frame {
    let shadow_alpha = if theme.brightness.is_dark() {
        0.26
    } else {
        0.14
    };
    Frame::new()
        .fill(theme.menu_color)
        .stroke(Stroke::new(
            1.0,
            theme.resources.surface_stroke_color_flyout,
        ))
        .corner_radius(8)
        .inner_margin(4)
        .shadow(Shadow {
            offset: [0, 8],
            blur: 16,
            spread: 0,
            color: with_alpha(Color32::BLACK, shadow_alpha),
        })
}

/// One row of a menu or combo box list, like fluent_ui's `MenuFlyoutItem`.
pub struct MenuItem {
    text: WidgetText,
    selected: bool,
}

impl MenuItem {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            selected: false,
        }
    }

    /// Marks the row as the current choice: subtle fill and an accent pill
    /// on the left, like a selected `ComboBoxItem`.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

impl Widget for MenuItem {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = FluentTheme::of(ui.ctx());
        let r = &theme.resources;
        let label = self.text.text().to_owned();
        let galley = self.text.into_galley(
            ui,
            Some(TextWrapMode::Extend),
            f32::INFINITY,
            theme.typography.body.font_id(),
        );
        let padding = Margin {
            left: 12,
            right: 12,
            top: 6,
            bottom: 6,
        };
        let width = ui.available_width().max(galley.size().x + padding.sum().x);
        let size = Vec2::new(width, (galley.size().y + padding.sum().y).max(32.0));
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        response.widget_info(|| {
            WidgetInfo::selected(Role::MenuItem, ui.is_enabled(), self.selected, &label)
        });

        if ui.is_rect_visible(rect) {
            let state = ControlState::from_response(&response);
            let painter = ui.painter();
            let rest = if self.selected {
                r.subtle_fill_color_secondary
            } else {
                r.subtle_fill_color_transparent
            };
            let fill = States::new(
                rest,
                r.subtle_fill_color_secondary,
                r.subtle_fill_color_tertiary,
                r.subtle_fill_color_transparent,
            )
            .resolve(state);
            let fill = animate_color(
                ui.ctx(),
                response.id.with("fill"),
                fill,
                theme.faster_animation_duration,
            );
            painter.rect_filled(rect, 4.0, fill);
            if self.selected {
                let pill = Rect::from_center_size(
                    pos2(rect.left() + 1.5, rect.center().y),
                    Vec2::new(3.0, 16.0),
                );
                painter.rect_filled(
                    pill,
                    1.5,
                    theme.accent_color.default_brush_for(theme.brightness),
                );
            }
            let color = if state.disabled {
                r.text_fill_color_disabled
            } else {
                r.text_fill_color_primary
            };
            let pos = pos2(
                rect.left() + f32::from(padding.left),
                rect.center().y - galley.size().y / 2.0,
            );
            painter.galley(pos, galley, color);
            if state.focused {
                paint_focus_border(painter, rect, &theme);
            }
        }
        response
    }
}

/// A button that opens a menu, fluent_ui's `DropDownButton`.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui) {
/// use fluent_egui::{DropDownButton, MenuItem};
///
/// DropDownButton::new("Email").show(ui, |ui| {
///     if ui.add(MenuItem::new("Send")).clicked() { /* ... */ }
///     if ui.add(MenuItem::new("Reply")).clicked() { /* ... */ }
/// });
/// # }
/// ```
pub struct DropDownButton {
    text: WidgetText,
}

impl DropDownButton {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self { text: text.into() }
    }

    /// Shows the button, and the menu while it is open.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<Option<R>> {
        let theme = FluentTheme::of(ui.ctx());
        let mut style = ButtonStyle::standard(&theme);
        style.padding.right += 22;
        let response = ui.add(Button::new(self.text).style(style));
        let state = ControlState::from_response(&response);
        let chevron = pos2(response.rect.right() - 18.0, response.rect.center().y);
        paint_chevron_down(ui.painter(), chevron, style.foreground.resolve(state));
        let inner = show_menu(&theme, &response, None, add_contents);
        InnerResponse::new(inner, response)
    }
}

/// What a [`SplitButton`] returns.
pub struct SplitButtonResponse<R> {
    /// The main part; check `.clicked()` for the primary action.
    pub main: Response,
    /// The chevron part that opens the menu.
    pub menu: Response,
    /// What the menu closure returned, if it was open.
    pub inner: Option<R>,
}

/// A button with a primary action and a chevron that opens a menu,
/// fluent_ui's `SplitButton`.
pub struct SplitButton {
    text: WidgetText,
}

impl SplitButton {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self { text: text.into() }
    }

    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> SplitButtonResponse<R> {
        let theme = FluentTheme::of(ui.ctx());
        let InnerResponse {
            inner: (main, menu),
            ..
        } = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 1.0;
            let mut left = ButtonStyle::standard(&theme);
            left.corner_radius = CornerRadius {
                nw: 4,
                sw: 4,
                ne: 0,
                se: 0,
            };
            let main = ui.add(Button::new(self.text).style(left));
            let mut right = ButtonStyle::standard(&theme);
            right.corner_radius = CornerRadius {
                nw: 0,
                sw: 0,
                ne: 4,
                se: 4,
            };
            right.padding.left = 8;
            right.padding.right = 8;
            // A blank label as wide as the chevron, which is painted on top.
            let menu = ui.add(
                Button::new(" ")
                    .style(right)
                    .min_size(Vec2::new(32.0, main.rect.height())),
            );
            let state = ControlState::from_response(&menu);
            paint_chevron_down(
                ui.painter(),
                menu.rect.center(),
                right.foreground.resolve(state),
            );
            (main, menu)
        });
        let inner = show_menu(&theme, &menu, None, add_contents);
        SplitButtonResponse { main, menu, inner }
    }
}

fn show_menu<R>(
    theme: &FluentTheme,
    anchor: &Response,
    width: Option<f32>,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    // Menu rows fill the available width, so give the popup a definite one:
    // the requested width, or at least the anchor's (and 160px).
    let width = width.unwrap_or_else(|| anchor.rect.width().max(160.0));
    let popup = Popup::menu(anchor)
        .frame(flyout_frame(theme))
        .gap(4.0)
        .width(width);
    popup
        .show(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            add_contents(ui)
        })
        .map(|r| r.inner)
}

/// A drop-down list for picking one value, fluent_ui's `ComboBox`.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui, color: &mut usize) {
/// use fluent_egui::{ComboBox, combo_value};
///
/// let names = ["Blue", "Green", "Red"];
/// ComboBox::new("color", names[*color]).show_ui(ui, |ui| {
///     for (i, name) in names.iter().enumerate() {
///         combo_value(ui, color, i, *name);
///     }
/// });
/// # }
/// ```
pub struct ComboBox {
    id_salt: IdSalt,
    selected_text: WidgetText,
    placeholder: bool,
    width: f32,
}

impl ComboBox {
    /// `selected_text` is what the closed box shows.
    pub fn new(id_salt: impl AsIdSalt, selected_text: impl Into<WidgetText>) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            selected_text: selected_text.into(),
            placeholder: false,
            width: 120.0,
        }
    }

    /// A box with nothing selected, showing `text` in the placeholder color.
    pub fn placeholder(id_salt: impl AsIdSalt, text: impl Into<WidgetText>) -> Self {
        Self {
            placeholder: true,
            ..Self::new(id_salt, text)
        }
    }

    /// Minimum width, 120 by default.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn show_ui<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<Option<R>> {
        let theme = FluentTheme::of(ui.ctx());
        let style = ButtonStyle::standard(&theme);
        let galley = self.selected_text.into_galley(
            ui,
            Some(TextWrapMode::Truncate),
            self.width.max(ui.available_width()) - 44.0,
            theme.typography.body.font_id(),
        );
        let size = Vec2::new((galley.size().x + 44.0).max(self.width), 32.0);
        let id = ui.make_persistent_id(self.id_salt);
        let rect = ui.allocate_space(size).1;
        let response = ui.interact(rect, id, Sense::click());
        let open = Popup::is_id_open(ui.ctx(), Popup::default_response_id(&response));
        response
            .widget_info(|| WidgetInfo::labeled(Role::ComboBox, ui.is_enabled(), galley.text()));

        if ui.is_rect_visible(rect) {
            let mut state = ControlState::from_response(&response);
            state.pressed |= open;
            let painter = ui.painter();
            let fill = animate_color(
                ui.ctx(),
                id.with("fill"),
                style.background.resolve(state),
                theme.faster_animation_duration,
            );
            painter.rect_filled(rect, 4.0, fill);
            let r = &theme.resources;
            paint_elevation_border(
                painter,
                rect,
                4,
                r.control_stroke_color_default,
                r.control_stroke_color_secondary,
            );
            let color = if state.disabled {
                r.text_fill_color_disabled
            } else if self.placeholder {
                r.text_fill_color_secondary
            } else {
                r.text_fill_color_primary
            };
            painter.galley(
                pos2(rect.left() + 11.0, rect.center().y - galley.size().y / 2.0),
                galley,
                color,
            );
            let chevron_color = if state.disabled {
                r.text_fill_color_disabled
            } else {
                r.text_fill_color_secondary
            };
            paint_chevron_down(
                painter,
                pos2(rect.right() - 18.0, rect.center().y),
                chevron_color,
            );
            if state.focused {
                paint_focus_border(painter, rect, &theme);
            }
        }
        let inner = show_menu(&theme, &response, Some(rect.width()), add_contents);
        InnerResponse::new(inner, response)
    }
}

/// A [`ComboBox`] row that sets `current` to `alternative` when clicked.
pub fn combo_value<T: PartialEq>(
    ui: &mut Ui,
    current: &mut T,
    alternative: T,
    text: impl Into<WidgetText>,
) -> Response {
    let mut response = ui.add(MenuItem::new(text).selected(*current == alternative));
    if response.clicked() && *current != alternative {
        *current = alternative;
        response.mark_changed();
    }
    response
}
