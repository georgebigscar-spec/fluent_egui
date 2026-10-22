use egui::epaint::{Mesh, Shadow};
use egui::{
    Color32, ColorImage, Context, CornerRadius, Id, InnerResponse, Margin, Rect, Shape, Stroke,
    StrokeKind, TextureHandle, TextureOptions, TextureWrapMode, Ui, UiBuilder, pos2,
};

use crate::FluentTheme;
use crate::utils::{hsv_to_rgb, rgb_to_hsv, with_alpha};

/// A translucent surface, fluent_ui's `Acrylic`.
///
/// egui has no backdrop blur, so the backdrop shows through the luminosity
/// and tint layers unblurred. With the default opacities only a faint hint of
/// it remains, which reads close to the real material on most content.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui) {
/// fluent_egui::Acrylic::new().corner_radius(8).show(ui, |ui| {
///     ui.label("On acrylic");
/// });
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Acrylic {
    tint: Option<Color32>,
    tint_alpha: Option<f32>,
    luminosity_alpha: Option<f32>,
    corner_radius: CornerRadius,
    elevation: f32,
    inner_margin: Margin,
    disabled: bool,
}

impl Default for Acrylic {
    fn default() -> Self {
        Self::new()
    }
}

impl Acrylic {
    pub fn new() -> Self {
        Self {
            tint: None,
            tint_alpha: None,
            luminosity_alpha: None,
            corner_radius: CornerRadius::ZERO,
            elevation: 0.0,
            inner_margin: Margin::same(12),
            disabled: false,
        }
    }

    /// Defaults to the theme's acrylic background colour.
    pub fn tint(mut self, tint: Color32) -> Self {
        self.tint = Some(tint);
        self
    }

    /// Defaults to the theme's acrylic tint opacity.
    pub fn tint_alpha(mut self, alpha: f32) -> Self {
        self.tint_alpha = Some(alpha);
        self
    }

    /// Defaults to the theme's acrylic luminosity opacity. fluent_ui derives
    /// a lighter one from the tint, which only works over a blurred backdrop.
    pub fn luminosity_alpha(mut self, alpha: f32) -> Self {
        self.luminosity_alpha = Some(alpha);
        self
    }

    pub fn corner_radius(mut self, radius: impl Into<CornerRadius>) -> Self {
        self.corner_radius = radius.into();
        self
    }

    /// Shadow depth, as in fluent_ui.
    pub fn elevation(mut self, elevation: f32) -> Self {
        self.elevation = elevation;
        self
    }

    pub fn inner_margin(mut self, margin: impl Into<Margin>) -> Self {
        self.inner_margin = margin.into();
        self
    }

    /// Paints the opaque fallback colour instead, fluent_ui's
    /// `DisableAcrylic`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The tint and luminosity layer colours, `AcrylicHelper` in fluent_ui.
    fn layers(&self, theme: &FluentTheme) -> (Color32, Color32) {
        let tint = self.tint.unwrap_or(theme.acrylic_background_color);
        let tint_alpha = self
            .tint_alpha
            .unwrap_or(theme.acrylic_tint_opacity)
            .clamp(0.0, 1.0);
        let alpha = f32::from(tint.a()) / 255.0;
        let modifier = if self.luminosity_alpha.is_some() {
            1.0
        } else {
            tint_opacity_modifier(tint)
        };
        let tint_color = with_alpha(tint, alpha * tint_alpha * modifier);
        (
            tint_color,
            luminosity_color(
                tint,
                alpha * tint_alpha,
                Some(
                    self.luminosity_alpha
                        .unwrap_or(theme.acrylic_luminosity_opacity),
                ),
            ),
        )
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = FluentTheme::of(ui.ctx());
        let background = ui.painter().add(Shape::Noop);
        let outer = ui.available_rect_before_wrap();
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(outer - self.inner_margin)
                .layout(*ui.layout()),
        );
        let inner = add_contents(&mut child);
        let rect = child.min_rect() + self.inner_margin;
        let response = ui.allocate_rect(rect, egui::Sense::hover());

        let radius = self.corner_radius;
        let mut shapes = Vec::new();
        if self.elevation > 0.0 {
            // fluent_ui stacks two soft shadows.
            for (alpha, blur) in [(0.13, 0.9), (0.11, 0.225)] {
                shapes.push(
                    Shadow {
                        offset: [0, (self.elevation * blur * 0.5) as i8],
                        blur: (self.elevation * blur).min(255.0) as u8,
                        spread: 0,
                        color: with_alpha(theme.shadow_color, alpha),
                    }
                    .as_shape(rect, radius)
                    .into(),
                );
            }
        }
        if self.disabled {
            shapes.push(Shape::rect_filled(
                rect,
                radius,
                theme.acrylic_fallback_color,
            ));
        } else {
            let (tint, luminosity) = self.layers(&theme);
            shapes.push(Shape::rect_filled(rect, radius, luminosity));
            shapes.push(Shape::rect_filled(rect, radius, tint));
            shapes.push(noise_shape(ui.ctx(), rect.shrink(radius.average() * 0.3)));
        }
        if self.elevation > 0.0 {
            shapes.push(Shape::rect_stroke(
                rect,
                radius,
                Stroke::new(1.0, theme.resources.surface_stroke_color_flyout),
                StrokeKind::Inside,
            ));
        }
        ui.painter().set(background, Shape::Vec(shapes));
        InnerResponse::new(inner, response)
    }
}

/// A plain backdrop for the whole window, fluent_ui's `Mica`. Real Mica
/// samples the desktop wallpaper, which egui cannot reach, so this is the
/// theme's solid base colour, the same fallback Windows uses.
#[derive(Clone, Copy, Debug)]
pub struct Mica {
    background: Option<Color32>,
    corner_radius: CornerRadius,
    elevation: f32,
    inner_margin: Margin,
}

impl Default for Mica {
    fn default() -> Self {
        Self::new()
    }
}

impl Mica {
    pub fn new() -> Self {
        Self {
            background: None,
            corner_radius: CornerRadius::ZERO,
            elevation: 0.0,
            inner_margin: Margin::ZERO,
        }
    }

    /// Defaults to the theme's mica background colour.
    pub fn background_color(mut self, color: Color32) -> Self {
        self.background = Some(color);
        self
    }

    pub fn corner_radius(mut self, radius: impl Into<CornerRadius>) -> Self {
        self.corner_radius = radius.into();
        self
    }

    pub fn elevation(mut self, elevation: f32) -> Self {
        self.elevation = elevation;
        self
    }

    pub fn inner_margin(mut self, margin: impl Into<Margin>) -> Self {
        self.inner_margin = margin.into();
        self
    }

    /// An [`egui::Frame`] for panels and windows.
    pub fn frame(&self, ctx: &Context) -> egui::Frame {
        let theme = FluentTheme::of(ctx);
        let mut frame = egui::Frame::new()
            .fill(self.background.unwrap_or(theme.mica_background_color))
            .corner_radius(self.corner_radius)
            .inner_margin(self.inner_margin);
        if self.elevation > 0.0 {
            frame = frame.shadow(Shadow {
                offset: [0, (self.elevation * 0.5) as i8],
                blur: self.elevation.min(255.0) as u8,
                spread: 0,
                color: with_alpha(theme.shadow_color, 0.2),
            });
        }
        frame
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        self.frame(ui.ctx()).show(ui, add_contents)
    }
}

/// `AcrylicHelper.getTintOpacityModifier`: light and dark tints need less
/// opacity than mid tones to look the same.
fn tint_opacity_modifier(color: Color32) -> f32 {
    const MID: f32 = 0.5;
    const WHITE_MAX: f32 = 0.45;
    const MID_MAX: f32 = 0.9;
    const BLACK_MAX: f32 = 0.85;
    let [_, s, v] = rgb_to_hsv(rgb(color));
    if v == MID {
        return MID_MAX;
    }
    let (lowest, max_deviation) = if v > MID {
        (WHITE_MAX, 1.0 - MID)
    } else {
        (BLACK_MAX, MID)
    };
    let mut suppression = MID_MAX - lowest;
    let deviation = (v - MID).abs() / max_deviation;
    if s > 0.0 {
        suppression *= (1.0 - s * 2.0).max(0.0);
    }
    MID_MAX - suppression * deviation
}

/// `AcrylicHelper.getLuminosityColor`.
fn luminosity_color(tint: Color32, tint_alpha: f32, luminosity_alpha: Option<f32>) -> Color32 {
    if let Some(alpha) = luminosity_alpha {
        return with_alpha(tint, alpha.clamp(0.0, 1.0));
    }
    let [h, s, v] = rgb_to_hsv(rgb(tint));
    let [r, g, b] = hsv_to_rgb([h, s, v.clamp(0.125, 0.965)]).map(|c| (c * 255.0).round() as u8);
    let alpha = (tint_alpha * (1.03 - 0.15) + 0.15).min(1.0);
    with_alpha(Color32::from_rgb(r, g, b), alpha)
}

fn rgb(color: Color32) -> [f32; 3] {
    let [r, g, b, _] = color.to_srgba_unmultiplied();
    [r, g, b].map(|c| f32::from(c) / 255.0)
}

/// The acrylic grain: a tiled noise texture at 2% opacity.
fn noise_shape(ctx: &Context, rect: Rect) -> Shape {
    const SIZE: usize = 64;
    let id = Id::NULL.with("fluent_acrylic_noise");
    let texture = ctx
        .data_mut(|d| d.get_temp::<TextureHandle>(id))
        .unwrap_or_else(|| {
            // A small xorshift keeps the grain the same every run.
            let mut seed = 0x2545_f491_u32;
            let pixels = (0..SIZE * SIZE)
                .map(|_| {
                    seed ^= seed << 13;
                    seed ^= seed >> 17;
                    seed ^= seed << 5;
                    Color32::from_gray((seed >> 24) as u8)
                })
                .collect();
            let image = ColorImage::new([SIZE, SIZE], pixels);
            let options = TextureOptions {
                wrap_mode: TextureWrapMode::Repeat,
                ..TextureOptions::NEAREST
            };
            let handle = ctx.load_texture("fluent_acrylic_noise", image, options);
            ctx.data_mut(|d| d.insert_temp(id, handle.clone()));
            handle
        });
    let uv = Rect::from_min_max(
        pos2(0.0, 0.0),
        pos2(rect.width() / SIZE as f32, rect.height() / SIZE as f32),
    );
    let mut mesh = Mesh::with_texture(texture.id());
    mesh.add_rect_with_uv(rect, uv, with_alpha(Color32::WHITE, 0.02));
    Shape::mesh(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifier_matches_fluent_ui() {
        // Pure white is fully suppressed to the white maximum.
        assert!((tint_opacity_modifier(Color32::WHITE) - 0.45).abs() < 1e-4);
        assert!((tint_opacity_modifier(Color32::BLACK) - 0.85).abs() < 1e-4);
        // Saturated colours are not suppressed.
        assert!((tint_opacity_modifier(Color32::from_rgb(255, 0, 0)) - 0.9).abs() < 1e-4);
    }
}
