use std::f32::consts::{PI, TAU};

use egui::epaint::{Mesh, PathShape};
use egui::{
    Color32, Key, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Widget, WidgetInfo, pos2, vec2,
};

use crate::FluentTheme;
use crate::utils::paint_focus_border;

/// A row of stars for rating, fluent_ui's `RatingBar`. Fractional ratings
/// show partly filled stars; clicking sets whole stars, and clicking the
/// current rating again clears it.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui, rating: &mut f32) {
/// ui.add(fluent_egui::Rating::new(rating));
/// # }
/// ```
pub struct Rating<'a> {
    rating: &'a mut f32,
    amount: u32,
    icon_size: f32,
    star_spacing: f32,
}

impl<'a> Rating<'a> {
    pub fn new(rating: &'a mut f32) -> Self {
        Self {
            rating,
            amount: 5,
            icon_size: 20.0,
            star_spacing: 8.0,
        }
    }

    /// Number of stars, 5 by default.
    pub fn amount(mut self, amount: u32) -> Self {
        self.amount = amount.max(1);
        self
    }

    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self
    }

    pub fn star_spacing(mut self, spacing: f32) -> Self {
        self.star_spacing = spacing;
        self
    }
}

impl Widget for Rating<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = FluentTheme::of(ui.ctx());
        let r = &theme.resources;
        let n = self.amount as f32;
        let pitch = self.icon_size + self.star_spacing;
        let size = vec2(
            n * self.icon_size + (n - 1.0) * self.star_spacing,
            self.icon_size,
        );
        let (rect, mut response) = ui.allocate_exact_size(size, Sense::click_and_drag());
        let star_at = |x: f32| ((x - rect.left()) / pitch).floor().clamp(0.0, n - 1.0) as u32 + 1;

        let hovered = response.hover_pos().map(|p| star_at(p.x));
        if let Some(p) = response.interact_pointer_pos() {
            let star = star_at(p.x) as f32;
            if response.clicked() && *self.rating == star {
                *self.rating = 0.0;
                response.mark_changed();
            } else if (response.clicked() || response.dragged()) && *self.rating != star {
                *self.rating = star;
                response.mark_changed();
            }
        }
        if response.has_focus() {
            let step = ui.input(|i| {
                i.num_presses(Key::ArrowRight) as f32 - i.num_presses(Key::ArrowLeft) as f32
            });
            if step != 0.0 {
                *self.rating = (self.rating.round() + step).clamp(0.0, n);
                response.mark_changed();
            }
        }
        let value = *self.rating;
        response.widget_info(|| WidgetInfo::slider(ui.is_enabled(), f64::from(value), "Rating"));

        if ui.is_rect_visible(rect) {
            let accent = theme.accent_color.default_brush_for(theme.brightness);
            let enabled = ui.is_enabled();
            for i in 0..self.amount {
                let star = Rect::from_min_size(
                    pos2(rect.left() + i as f32 * pitch, rect.top()),
                    egui::Vec2::splat(self.icon_size),
                );
                // fluent_ui's colors for disabled, hover preview and rest.
                let (fill, rated, unrated) = if !enabled {
                    let f = (value - i as f32).clamp(0.0, 1.0);
                    (f, r.text_fill_color_disabled, r.text_fill_color_secondary)
                } else if let Some(h) = hovered {
                    if i < h {
                        let c = if i < value as u32 {
                            accent
                        } else {
                            r.text_fill_color_primary
                        };
                        (1.0, c, c)
                    } else {
                        let c = r.control_alt_fill_color_tertiary;
                        (0.0, c, c)
                    }
                } else {
                    let f = (value - i as f32).clamp(0.0, 1.0);
                    (f, accent, r.text_fill_color_secondary)
                };
                paint_star(ui.painter(), star, fill, rated, unrated);
            }
            if response.has_focus() {
                paint_focus_border(ui.painter(), rect.expand(2.0), &theme);
            }
        }
        response
    }
}

/// A star outline in `unrated`, filled with `rated` from the left up to
/// `fill` (0 to 1) of its width.
fn paint_star(painter: &Painter, rect: Rect, fill: f32, rated: Color32, unrated: Color32) {
    let center = rect.center() + vec2(0.0, rect.height() * 0.04);
    let outer = rect.width() * 0.5;
    let inner = outer * 0.45;
    let points: Vec<Pos2> = (0..10)
        .map(|i| {
            let angle = -PI / 2.0 + i as f32 * TAU / 10.0;
            let radius = if i % 2 == 0 { outer } else { inner };
            center + radius * vec2(angle.cos(), angle.sin())
        })
        .collect();
    if fill > 0.0 {
        // The star is star-shaped around its center, so a fan fills it.
        let mut mesh = Mesh::default();
        mesh.colored_vertex(center, rated);
        for p in &points {
            mesh.colored_vertex(*p, rated);
        }
        for i in 1..=10u32 {
            mesh.add_triangle(0, i, i % 10 + 1);
        }
        let clip = Rect::from_min_max(
            rect.min,
            pos2(rect.left() + rect.width() * fill, rect.bottom()),
        );
        painter
            .with_clip_rect(clip.intersect(painter.clip_rect()))
            .add(mesh);
    }
    if fill < 1.0 {
        painter.add(PathShape::closed_line(points, Stroke::new(1.0, unrated)));
    }
}
