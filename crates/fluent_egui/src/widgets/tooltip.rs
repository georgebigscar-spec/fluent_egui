use egui::{Color32, Frame, Margin, Response, Shadow, Tooltip, WidgetText};

use crate::FluentTheme;
use crate::utils::with_alpha;

/// Fluent tooltips for any [`Response`], fluent_ui's `Tooltip`.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui) {
/// use fluent_egui::FluentTooltip as _;
///
/// ui.button("Save").fluent_tooltip("Save the document (Ctrl+S)");
/// # }
/// ```
pub trait FluentTooltip {
    /// Shows `text` in a Fluent tooltip while the widget is hovered.
    fn fluent_tooltip(self, text: impl Into<WidgetText>) -> Self;
}

impl FluentTooltip for Response {
    fn fluent_tooltip(self, text: impl Into<WidgetText>) -> Self {
        let theme = FluentTheme::of(&self.ctx);
        let caption = &theme.typography.caption;
        let text = text.into();
        let mut tooltip = Tooltip::for_enabled(&self);
        tooltip.popup = tooltip.popup.frame(tooltip_frame(&theme));
        tooltip.show(|ui| {
            let text = match text {
                WidgetText::Text(s) => caption
                    .rich(s)
                    .color(theme.resources.text_fill_color_primary)
                    .into(),
                other => other,
            };
            ui.label(text);
        });
        self
    }
}

/// fluent_ui's `TooltipThemeData.standard` decoration.
pub fn tooltip_frame(theme: &FluentTheme) -> Frame {
    Frame::new()
        .fill(theme.resources.system_fill_color_solid_neutral_background)
        .corner_radius(4)
        .inner_margin(Margin {
            left: 8,
            right: 8,
            top: 5,
            bottom: 7,
        })
        .shadow(Shadow {
            offset: [1, 1],
            blur: 10,
            spread: 0,
            color: with_alpha(Color32::BLACK, 0.2),
        })
}
