use egui::{Align, Frame, Layout, Margin, Response, Stroke, TextWrapMode, Ui, Vec2, WidgetText};

use super::{Button, Severity, glyph_button};
use crate::FluentTheme;
use crate::Localizations;
use crate::utils::{StatusGlyph, paint_close, paint_status_icon};

/// What [`InfoBar::show`] returns.
pub struct InfoBarResponse {
    pub response: Response,
    /// The close button was clicked; hide the bar.
    pub closed: bool,
    /// The action button was clicked.
    pub action_clicked: bool,
}

/// An inline status message, fluent_ui's `InfoBar`.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui, visible: &mut bool) {
/// use fluent_egui::{InfoBar, Severity};
///
/// if *visible {
///     let bar = InfoBar::new("Saved").message("All changes are on disk.")
///         .severity(Severity::Success)
///         .closable(true)
///         .show(ui);
///     if bar.closed { *visible = false; }
/// }
/// # }
/// ```
pub struct InfoBar {
    title: WidgetText,
    message: Option<WidgetText>,
    severity: Severity,
    long: bool,
    closable: bool,
    action: Option<WidgetText>,
}

impl InfoBar {
    pub fn new(title: impl Into<WidgetText>) -> Self {
        Self {
            title: title.into(),
            message: None,
            severity: Severity::Info,
            long: false,
            closable: false,
            action: None,
        }
    }

    pub fn message(mut self, message: impl Into<WidgetText>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Stacks title, message and action vertically (`isLong`).
    pub fn long(mut self, long: bool) -> Self {
        self.long = long;
        self
    }

    /// Shows a close button; see [`InfoBarResponse::closed`].
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Adds a button; see [`InfoBarResponse::action_clicked`].
    pub fn action(mut self, text: impl Into<WidgetText>) -> Self {
        self.action = Some(text.into());
        self
    }

    pub fn show(self, ui: &mut Ui) -> InfoBarResponse {
        let theme = FluentTheme::of(ui.ctx());
        let r = &theme.resources;
        let (background, icon_color, glyph) = match self.severity {
            Severity::Info => (
                r.system_fill_color_attention_background,
                theme.accent_color.default_brush_for(theme.brightness),
                StatusGlyph::Info,
            ),
            Severity::Warning => (
                r.system_fill_color_caution_background,
                r.system_fill_color_caution,
                StatusGlyph::Warning,
            ),
            Severity::Success => (
                r.system_fill_color_success_background,
                r.system_fill_color_success,
                StatusGlyph::Success,
            ),
            Severity::Error => (
                r.system_fill_color_critical_background,
                r.system_fill_color_critical,
                StatusGlyph::Error,
            ),
        };
        let mut closed = false;
        let mut action_clicked = false;

        let frame = Frame::new()
            .fill(background)
            .stroke(Stroke::new(1.0, r.card_stroke_color_default))
            .corner_radius(4)
            .inner_margin(Margin {
                left: 14,
                right: 8,
                top: 14,
                bottom: 14,
            });
        let inner = frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height(48.0 - 28.0);
            let align = if self.long { Align::Min } else { Align::Center };
            // A row like `ui.horizontal`: `with_layout` would center the
            // content in, and so take, all the height left in `ui`.
            let row = Vec2::new(ui.available_width(), 20.0);
            ui.allocate_ui_with_layout(row, Layout::left_to_right(align), |ui| {
                // Icon box matches the title's line height so they line up.
                let (icon_rect, _) =
                    ui.allocate_exact_size(Vec2::new(16.0, 20.0), egui::Sense::hover());
                paint_status_icon(
                    ui.painter(),
                    icon_rect.center(),
                    glyph,
                    icon_color,
                    r.text_fill_color_inverse,
                );
                ui.add_space(14.0 - ui.spacing().item_spacing.x);

                let close_width = if self.closable { 40.0 } else { 0.0 };
                let content_width = (ui.available_width() - close_width).max(0.0);
                let title = theme.typography.body_strong.rich(self.title.text());
                ui.allocate_ui_with_layout(
                    Vec2::new(content_width, 20.0),
                    Layout::left_to_right(align),
                    |ui| {
                        if self.long {
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 6.0;
                                ui.add(egui::Label::new(title).wrap_mode(TextWrapMode::Wrap));
                                if let Some(message) = &self.message {
                                    ui.add(
                                        egui::Label::new(message.clone())
                                            .wrap_mode(TextWrapMode::Wrap),
                                    );
                                }
                                if let Some(action) = &self.action {
                                    ui.add_space(6.0);
                                    action_clicked = ui.add(Button::new(action.clone())).clicked();
                                }
                            });
                        } else {
                            ui.spacing_mut().item_spacing.x = 6.0;
                            ui.label(title);
                            if let Some(message) = &self.message {
                                ui.add(egui::Label::new(message.clone()).truncate());
                            }
                            if let Some(action) = &self.action {
                                ui.add_space(10.0);
                                action_clicked = ui.add(Button::new(action.clone())).clicked();
                            }
                        }
                    },
                );
                if self.closable {
                    ui.with_layout(Layout::right_to_left(align), |ui| {
                        closed = glyph_button(ui, 30.0, paint_close)
                            .on_hover_text(Localizations::of(ui.ctx()).close_button_label)
                            .clicked();
                    });
                }
            });
        });
        InfoBarResponse {
            response: inner.response,
            closed,
            action_clicked,
        }
    }
}
