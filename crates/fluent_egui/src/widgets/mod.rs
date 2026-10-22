//! Fluent widgets. The layout mirrors fluent_ui's `lib/src/controls`.

mod button;
mod checkbox;
mod divider;
mod icon;
mod info_badge;
mod info_label;
mod progress;
mod radio_button;
mod slider;
mod toggle_switch;

pub use button::{Button, ButtonKind, ButtonStyle, IconButtonMode};
pub use checkbox::Checkbox;
pub use divider::Divider;
pub use icon::{icon, windows_icon};
pub use info_badge::{InfoBadge, Severity};
pub use info_label::InfoLabel;
pub use progress::{ProgressBar, ProgressRing};
pub use radio_button::{RadioButton, radio_value};
pub use slider::Slider;
pub use toggle_switch::ToggleSwitch;

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
