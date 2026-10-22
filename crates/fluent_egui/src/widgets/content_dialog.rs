use egui::{
    Align, AsIdSalt, Context, CornerRadius, Frame, Id, IdSalt, Layout, Margin, Modal, Shadow, Ui,
    Vec2, WidgetText,
};

use super::Button;
use crate::FluentTheme;
use crate::utils::with_alpha;

/// Which [`ContentDialog`] button closed it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogResult {
    Primary,
    Secondary,
    /// The close button, Escape, or a click outside the dialog.
    Close,
}

/// A modal dialog, fluent_ui's `ContentDialog`: a title, content, and up to
/// three buttons on a footer, the primary one filled.
///
/// Show it every frame while it should be open:
///
/// ```no_run
/// # fn ui(ctx: &egui::Context, open: &mut bool) {
/// use fluent_egui::{ContentDialog, DialogResult};
///
/// if *open {
///     let result = ContentDialog::new("delete", "Delete file?")
///         .body("This can't be undone.")
///         .primary_button("Delete")
///         .close_button("Cancel")
///         .show(ctx);
///     if let Some(result) = result {
///         *open = false;
///         if result == DialogResult::Primary { /* delete */ }
///     }
/// }
/// # }
/// ```
pub struct ContentDialog {
    id: IdSalt,
    title: WidgetText,
    body: Option<WidgetText>,
    primary: Option<WidgetText>,
    secondary: Option<WidgetText>,
    close: Option<WidgetText>,
    width: f32,
}

impl ContentDialog {
    pub fn new(id_salt: impl AsIdSalt, title: impl Into<WidgetText>) -> Self {
        Self {
            id: IdSalt::new(id_salt),
            title: title.into(),
            body: None,
            primary: None,
            secondary: None,
            close: None,
            width: 368.0,
        }
    }

    pub fn body(mut self, text: impl Into<WidgetText>) -> Self {
        self.body = Some(text.into());
        self
    }

    pub fn primary_button(mut self, text: impl Into<WidgetText>) -> Self {
        self.primary = Some(text.into());
        self
    }

    pub fn secondary_button(mut self, text: impl Into<WidgetText>) -> Self {
        self.secondary = Some(text.into());
        self
    }

    pub fn close_button(mut self, text: impl Into<WidgetText>) -> Self {
        self.close = Some(text.into());
        self
    }

    /// Content width, clamped to WinUI's 320..=548. 368 by default.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.clamp(320.0, 548.0);
        self
    }

    /// Shows the dialog with the text body. Returns the result once closed.
    pub fn show(self, ctx: &Context) -> Option<DialogResult> {
        self.show_with(ctx, |_| ()).0
    }

    /// Shows the dialog with custom content below the body text.
    pub fn show_with<R>(
        mut self,
        ctx: &Context,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> (Option<DialogResult>, R) {
        let theme = FluentTheme::of(ctx);
        let r = &theme.resources;
        let frame = Frame::new()
            .fill(theme.menu_color)
            .corner_radius(12)
            .stroke(egui::Stroke::new(1.0, r.surface_stroke_color_default))
            .shadow(Shadow {
                offset: [0, 16],
                blur: 32,
                spread: 0,
                color: with_alpha(egui::Color32::BLACK, 0.3),
            });
        let mut result = None;
        let width = self.width;
        let modal = Modal::new(Id::unique(("fluent_egui::ContentDialog", self.id)))
            .frame(frame)
            .backdrop_color(r.smoke_fill_color_default)
            .show(ctx, |ui| {
                ui.set_width(width + 48.0);
                ui.spacing_mut().item_spacing.y = 0.0;
                let inner = Frame::new().inner_margin(Margin::same(24)).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 12.0;
                    ui.label(theme.typography.subtitle.rich(self.title.text()));
                    if let Some(body) = self.body.take() {
                        ui.add(egui::Label::new(body).wrap());
                    }
                    add_contents(ui)
                });
                let buttons: Vec<(WidgetText, DialogResult)> = [
                    (self.primary.take(), DialogResult::Primary),
                    (self.secondary.take(), DialogResult::Secondary),
                    (self.close.take(), DialogResult::Close),
                ]
                .into_iter()
                .filter_map(|(text, result)| text.map(|t| (t, result)))
                .collect();
                if !buttons.is_empty() {
                    Frame::new()
                        .fill(theme.mica_background_color)
                        .corner_radius(CornerRadius {
                            nw: 0,
                            ne: 0,
                            sw: 12,
                            se: 12,
                        })
                        .inner_margin(Margin::same(24))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            let count = buttons.len() as f32;
                            let gap = 8.0;
                            let button_width = (ui.available_width() - gap * (count - 1.0)) / count;
                            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                ui.spacing_mut().item_spacing.x = gap;
                                for (text, kind) in buttons {
                                    let button = if kind == DialogResult::Primary {
                                        Button::filled(text)
                                    } else {
                                        Button::new(text)
                                    };
                                    if ui
                                        .add(button.min_size(Vec2::new(button_width, 32.0)))
                                        .clicked()
                                    {
                                        result = Some(kind);
                                    }
                                }
                            });
                        });
                }
                inner.inner
            });
        if result.is_none() && modal.should_close() {
            result = Some(DialogResult::Close);
        }
        (result, modal.inner)
    }
}
