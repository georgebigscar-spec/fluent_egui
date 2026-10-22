//! Fluent widgets. The layout mirrors fluent_ui's `lib/src/controls`.

mod acrylic;
mod auto_suggest_box;
mod bars;
mod breadcrumb_bar;
mod button;
mod calendar_date_picker;
mod calendar_view;
mod card;
mod checkbox;
mod color_picker;
mod content_dialog;
mod date_picker;
mod divider;
mod dynamic_overflow;
mod expander;
mod icon;
mod info_badge;
mod info_bar;
mod info_label;
mod list_tile;
mod menu;
mod navigation_view;
mod number_box;
mod page_transition;
mod pickers;
mod progress;
mod radio_button;
mod rating;
mod scaffold_page;
mod slider;
mod tab_view;
mod text_box;
mod time_picker;
mod toggle_switch;
mod tooltip;
mod tree_view;

pub use acrylic::{Acrylic, Mica};
pub use auto_suggest_box::{AutoSuggestBox, AutoSuggestBoxResponse};
pub use bars::{CommandBar, CommandBarUi, MenuBar, MenuBarUi, TeachingTip, TeachingTipResponse};
pub use breadcrumb_bar::BreadcrumbBar;
pub use button::{Button, ButtonKind, ButtonStyle, IconButtonMode};
pub use calendar_date_picker::CalendarDatePicker;
pub use calendar_view::{CalendarView, DateRange};
pub use card::Card;
pub use checkbox::Checkbox;
pub use color_picker::{ColorPicker, ColorSpectrumShape};
pub use content_dialog::{ContentDialog, DialogResult};
pub use date_picker::DatePicker;
pub use divider::Divider;
pub use dynamic_overflow::{DynamicOverflow, DynamicOverflowResponse};
pub use expander::{Expander, ExpanderResponse};
pub use icon::{icon, windows_icon};
pub use info_badge::{InfoBadge, Severity};
pub use info_bar::{InfoBar, InfoBarResponse};
pub use info_label::InfoLabel;
pub use list_tile::{ListTile, ListTileSelectionMode};
pub use menu::{
    ComboBox, DropDownButton, MenuItem, SplitButton, SplitButtonResponse, combo_value,
    context_menu, flyout_frame, menu_separator, sub_menu,
};
pub use navigation_view::{NavigationView, NavigationViewResponse, PaneDisplayMode, PaneItem};
pub use number_box::{NumberBox, SpinButtonPlacement};
pub use page_transition::{PageTransition, PageTransitionKind};
pub use progress::{ProgressBar, ProgressRing};
pub use radio_button::{RadioButton, radio_value};
pub use rating::Rating;
pub use scaffold_page::ScaffoldPage;
pub use slider::Slider;
pub use tab_view::{CloseButtonVisibility, Tab, TabView, TabViewResponse, TabWidthBehavior};
pub use text_box::{PasswordBox, PasswordRevealMode, TextBox};
pub use time_picker::TimePicker;
pub use toggle_switch::ToggleSwitch;
pub use tooltip::{FluentTooltip, tooltip_frame};
pub use tree_view::{TreeView, TreeViewItem, TreeViewResponse, TreeViewSelectionMode};

use std::sync::Arc;

use egui::{Galley, Pos2, Rect, Response, Sense, TextWrapMode, Ui, Vec2, WidgetText};

use crate::FluentTheme;

/// Default row height of checkboxes and radio buttons, so stacked ones get
/// some space between them. Between fluent_ui (20) and WinUI (32).
pub(crate) const LABELED_MIN_HEIGHT: f32 = 24.0;

/// Layout shared by checkbox-like controls: an indicator of `indicator` size,
/// then `gap`, then an optional label, in a row at least `min_height` tall. Returns the response for the whole row,
/// the indicator rect and the placed label.
pub(crate) fn allocate_labeled(
    ui: &mut Ui,
    indicator: Vec2,
    gap: f32,
    min_height: f32,
    text: Option<WidgetText>,
    sense: Sense,
) -> (Response, Rect, Option<(Pos2, Arc<Galley>)>) {
    let theme = FluentTheme::of(ui.ctx());
    let galley = text.map(|t| {
        t.into_galley(
            ui,
            Some(TextWrapMode::Extend),
            f32::INFINITY,
            theme.typography.body.font_id(),
        )
    });
    let text_size = galley.as_ref().map_or(Vec2::ZERO, |g| g.size());
    let size = Vec2::new(
        indicator.x
            + if galley.is_some() {
                gap + text_size.x
            } else {
                0.0
            },
        indicator.y.max(text_size.y).max(min_height),
    );
    let (rect, response) = ui.allocate_exact_size(size, sense);
    let indicator_rect = Rect::from_min_size(
        Pos2::new(rect.left(), rect.center().y - indicator.y / 2.0),
        indicator,
    );
    let label = galley.map(|g| {
        let pos = Pos2::new(
            indicator_rect.right() + gap,
            rect.center().y - g.size().y / 2.0,
        );
        (pos, g)
    });
    (response, indicator_rect, label)
}

/// A transparent square button (like a large `IconButton`) whose glyph is
/// painted by `paint` at the center, in the foreground color for its state.
pub(crate) fn glyph_button(
    ui: &mut Ui,
    size: f32,
    paint: impl FnOnce(&egui::Painter, Pos2, egui::Color32),
) -> Response {
    let theme = FluentTheme::of(ui.ctx());
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    if ui.is_rect_visible(rect) {
        let state = crate::utils::ControlState::from_response(&response);
        let style = button::ButtonStyle::icon(&theme, button::IconButtonMode::Large);
        let fill = crate::utils::animate_color(
            ui.ctx(),
            response.id.with("fill"),
            style.background.resolve(state),
            theme.faster_animation_duration,
        );
        ui.painter().rect_filled(rect, 4.0, fill);
        paint(ui.painter(), rect.center(), style.foreground.resolve(state));
        if state.focused {
            crate::utils::paint_focus_border(ui.painter(), rect, &theme);
        }
    }
    response
}
