use egui::{Color32, CornerRadius, Frame, InnerResponse, Margin, Stroke, Ui};

use crate::FluentTheme;

/// A bordered surface that groups content, fluent_ui's `Card`.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui) {
/// fluent_egui::Card::new().show(ui, |ui| ui.label("Inside a card"));
/// # }
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct Card {
    padding: Option<Margin>,
    fill: Option<Color32>,
    border: Option<Color32>,
    corner_radius: Option<CornerRadius>,
}

impl Card {
    pub fn new() -> Self {
        Self::default()
    }

    /// 12 on every side by default.
    pub fn padding(mut self, padding: impl Into<Margin>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    pub fn fill(mut self, color: Color32) -> Self {
        self.fill = Some(color);
        self
    }

    pub fn border(mut self, color: Color32) -> Self {
        self.border = Some(color);
        self
    }

    /// 4 by default.
    pub fn corner_radius(mut self, radius: impl Into<CornerRadius>) -> Self {
        self.corner_radius = Some(radius.into());
        self
    }

    /// The egui [`Frame`] this card paints.
    pub fn frame(&self, theme: &FluentTheme) -> Frame {
        Frame::new()
            .fill(self.fill.unwrap_or(theme.card_color))
            .stroke(Stroke::new(
                1.0,
                self.border
                    .unwrap_or(theme.resources.card_stroke_color_default),
            ))
            .corner_radius(self.corner_radius.unwrap_or(CornerRadius::same(4)))
            .inner_margin(self.padding.unwrap_or(Margin::same(12)))
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = FluentTheme::of(ui.ctx());
        self.frame(&theme).show(ui, add_contents)
    }
}
