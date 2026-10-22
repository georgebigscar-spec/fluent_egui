use egui::{Color32, FontId, Response, Sense, TextWrapMode, Ui, Vec2, Widget, WidgetText};

use crate::FluentTheme;

/// Severity of a status element, fluent_ui's `InfoBarSeverity`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Severity {
    #[default]
    Info,
    Warning,
    Success,
    Error,
}

impl Severity {
    /// The badge color fluent_ui's `InfoBadge` uses for this severity.
    pub fn badge_color(self, theme: &FluentTheme) -> Color32 {
        let r = &theme.resources;
        match self {
            Self::Info => theme.accent_color.default_brush_for(theme.brightness),
            Self::Warning => r.system_fill_color_solid_attention_background,
            Self::Success => r.system_fill_color_success,
            Self::Error => r.system_fill_color_critical,
        }
    }
}

/// A small status badge, fluent_ui's `InfoBadge`: a 6px dot, or a pill with
/// a number, short text or icon.
pub struct InfoBadge {
    text: Option<WidgetText>,
    severity: Severity,
    color: Option<Color32>,
    text_color: Option<Color32>,
}

impl InfoBadge {
    pub fn dot() -> Self {
        Self {
            text: None,
            severity: Severity::Info,
            color: None,
            text_color: None,
        }
    }

    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: Some(text.into()),
            ..Self::dot()
        }
    }

    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn text_color(mut self, color: Color32) -> Self {
        self.text_color = Some(color);
        self
    }
}

impl Widget for InfoBadge {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = FluentTheme::of(ui.ctx());
        let color = self
            .color
            .unwrap_or_else(|| self.severity.badge_color(&theme));
        let Some(text) = self.text else {
            let (rect, response) = ui.allocate_exact_size(Vec2::splat(6.0), Sense::hover());
            ui.painter().circle_filled(rect.center(), 3.0, color);
            return response;
        };
        let galley = text.into_galley(
            ui,
            Some(TextWrapMode::Extend),
            f32::INFINITY,
            FontId::proportional(11.0),
        );
        // Padding: 4 left and right, 1 at the bottom; at least 16x16.
        let size = Vec2::new((galley.size().x + 8.0).max(16.0), 16.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            painter.rect_filled(rect, 8.0, color);
            let pos = rect.center() - galley.size() / 2.0 - Vec2::new(0.0, 0.5);
            let text_color = self
                .text_color
                .unwrap_or(theme.resources.text_on_accent_fill_color_primary);
            painter.galley(pos, galley, text_color);
        }
        response
    }
}
