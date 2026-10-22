use egui::{Frame, Margin, ScrollArea, Ui, WidgetText};

use crate::FluentTheme;

/// A page with a title header and padded content, fluent_ui's
/// `ScaffoldPage` with a `PageHeader`.
///
/// The padding is 24px (12px horizontally below 640px wide), and the content
/// scrolls under a fixed header unless [`ScaffoldPage::scrollable`] is off.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui) {
/// fluent_egui::ScaffoldPage::new()
///     .header("Settings")
///     .show(ui, |ui| {
///         ui.label("Content");
///     });
/// # }
/// ```
pub struct ScaffoldPage {
    header: Option<WidgetText>,
    padding: Option<Margin>,
    scrollable: bool,
}

impl Default for ScaffoldPage {
    fn default() -> Self {
        Self::new()
    }
}

impl ScaffoldPage {
    pub fn new() -> Self {
        Self {
            header: None,
            padding: None,
            scrollable: true,
        }
    }

    /// The page title, in the `title` text style.
    pub fn header(mut self, title: impl Into<WidgetText>) -> Self {
        self.header = Some(title.into());
        self
    }

    /// Replaces the default page padding.
    pub fn padding(mut self, padding: impl Into<Margin>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        let theme = FluentTheme::of(ui.ctx());
        let padding = self.padding.unwrap_or_else(|| {
            let horizontal = if ui.available_width() <= 640.0 {
                12
            } else {
                24
            };
            Margin::symmetric(horizontal, 24)
        });
        let side = |top, bottom| Margin {
            top,
            bottom,
            ..padding
        };
        if let Some(title) = self.header {
            Frame::NONE
                .inner_margin(side(padding.top, 18))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    let title = match title {
                        WidgetText::Text(s) => theme.typography.title.rich(s).into(),
                        other => other,
                    };
                    ui.label(title);
                });
        }
        let top = if ui.min_rect().height() > 0.0 {
            0
        } else {
            padding.top
        };
        let content = |ui: &mut Ui| {
            Frame::NONE
                .inner_margin(side(top, padding.bottom))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    add_contents(ui)
                })
                .inner
        };
        if self.scrollable {
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, content)
                .inner
        } else {
            content(ui)
        }
    }
}
