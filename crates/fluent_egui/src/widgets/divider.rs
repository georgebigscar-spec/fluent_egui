use egui::{Response, Sense, Ui, Vec2, Widget};

use crate::FluentTheme;

/// A thin line that groups content, like fluent_ui's `Divider`.
#[derive(Clone, Copy, Debug)]
pub struct Divider {
    vertical: bool,
    size: Option<f32>,
}

impl Divider {
    pub fn horizontal() -> Self {
        Self {
            vertical: false,
            size: None,
        }
    }

    pub fn vertical() -> Self {
        Self {
            vertical: true,
            size: None,
        }
    }

    /// Length of the line. Defaults to the available width (or height).
    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }
}

impl Widget for Divider {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = FluentTheme::of(ui.ctx());
        let style = &theme.divider;
        let along = self.size.unwrap_or(if self.vertical {
            ui.available_height()
        } else {
            ui.available_width()
        });
        let desired = if self.vertical {
            Vec2::new(style.thickness, along)
        } else {
            Vec2::new(along, style.thickness)
        };
        let (rect, response) = ui.allocate_exact_size(desired, Sense::hover());
        if ui.is_rect_visible(rect) {
            // fluent_ui applies the margin along the line, not across it.
            let line = if self.vertical {
                rect.shrink2(Vec2::new(0.0, style.margin))
            } else {
                rect.shrink2(Vec2::new(style.margin, 0.0))
            };
            ui.painter().rect_filled(line, 0.0, style.color);
        }
        response
    }
}
