use std::f32::consts::TAU;

use egui::epaint::Mesh;
use egui::{
    AsIdSalt, Color32, Id, IdSalt, Pos2, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, Vec2,
    Widget, WidgetInfo, pos2, vec2,
};

use super::{ComboBox, NumberBox, TextBox, combo_value};
use crate::FluentTheme;
use crate::utils::paint_focus_border;

/// fluent_ui's `_ColorPickerSizes` and `_ColorPickerSpacing`.
const SPECTRUM: f32 = 256.0;
const PREVIEW: f32 = 44.0;
const SLIDER: f32 = 12.0;
const SMALL: f32 = 12.0;
const LARGE: f32 = 24.0;
const WIDTH: f32 = SPECTRUM + SMALL + PREVIEW;

/// The shape of the hue and saturation area, fluent_ui's `ColorSpectrumShape`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorSpectrumShape {
    /// Hue around the circle, saturation from the center out.
    #[default]
    Ring,
    /// Hue left to right, saturation bottom to top.
    Box,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ChannelMode {
    #[default]
    Rgb,
    Hsv,
}

/// The picker's own copy of the color. Hue and saturation survive grays and
/// black, which a `Color32` would lose.
#[derive(Clone, Debug, Default)]
struct PickerState {
    hsva: [f32; 4],
    /// The color `hsva` was last synced with.
    color: Color32,
    mode: ChannelMode,
    hex: String,
    hex_editing: bool,
}

/// Picks a color from a spectrum with value and alpha sliders, a preview
/// and hex and channel inputs. fluent_ui's `ColorPicker` in its vertical
/// layout.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui, color: &mut egui::Color32) {
/// ui.add(fluent_egui::ColorPicker::new("accent", color));
/// # }
/// ```
pub struct ColorPicker<'a> {
    id_salt: IdSalt,
    color: &'a mut Color32,
    shape: ColorSpectrumShape,
    alpha: bool,
    inputs: bool,
}

impl<'a> ColorPicker<'a> {
    pub fn new(id_salt: impl AsIdSalt, color: &'a mut Color32) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            color,
            shape: ColorSpectrumShape::Ring,
            alpha: true,
            inputs: true,
        }
    }

    pub fn spectrum_shape(mut self, shape: ColorSpectrumShape) -> Self {
        self.shape = shape;
        self
    }

    /// Shows the alpha slider and input. On by default.
    pub fn alpha(mut self, enabled: bool) -> Self {
        self.alpha = enabled;
        self
    }

    /// Shows the hex and channel inputs. On by default.
    pub fn inputs(mut self, visible: bool) -> Self {
        self.inputs = visible;
        self
    }
}

impl Widget for ColorPicker<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let id = ui.make_persistent_id(self.id_salt);
        let mut state = ui
            .data(|d| d.get_temp::<PickerState>(id))
            .unwrap_or_default();
        if state.color != *self.color || state.hex.is_empty() {
            let [r, g, b, a] = self.color.to_srgba_unmultiplied();
            let [h, s, v] = rgb_to_hsv([r, g, b].map(|c| f32::from(c) / 255.0));
            // Keep the hue (and saturation) when the color has none.
            let keep_hue = s == 0.0 || v == 0.0;
            let keep_sat = v == 0.0;
            state.hsva = [
                if keep_hue { state.hsva[0] } else { h },
                if keep_sat { state.hsva[1] } else { s },
                v,
                f32::from(a) / 255.0,
            ];
            state.color = *self.color;
            if !state.hex_editing {
                state.hex = to_hex(*self.color, self.alpha);
            }
        }
        let before = state.hsva;

        let response = ui
            .vertical(|ui| {
                ui.spacing_mut().item_spacing = vec2(SMALL, LARGE);
                ui.horizontal(|ui| {
                    spectrum(ui, id.with("spectrum"), self.shape, &mut state.hsva);
                    preview(ui, hsva_to_color(state.hsva));
                });
                let [h, s, _, _] = state.hsva;
                let full = hsva_to_color([h, s, 1.0, 1.0]);
                gradient_slider(
                    ui,
                    id.with("value"),
                    "Value",
                    &mut state.hsva[2],
                    [Color32::BLACK, full],
                    false,
                );
                if self.alpha {
                    let opaque = hsva_to_color([h, s, state.hsva[2], 1.0]);
                    let clear =
                        Color32::from_rgba_unmultiplied(opaque.r(), opaque.g(), opaque.b(), 0);
                    gradient_slider(
                        ui,
                        id.with("alpha"),
                        "Alpha",
                        &mut state.hsva[3],
                        [clear, opaque],
                        true,
                    );
                } else {
                    state.hsva[3] = 1.0;
                }
                if self.inputs {
                    ui.spacing_mut().item_spacing.y = SMALL;
                    self.inputs_ui(ui, id, &mut state);
                }
            })
            .response;

        if state.hsva != before {
            state.hex = to_hex(hsva_to_color(state.hsva), self.alpha);
        }
        let new_color = hsva_to_color(state.hsva);
        let mut response = response;
        if new_color != *self.color {
            *self.color = new_color;
            response.mark_changed();
        }
        state.color = *self.color;
        ui.data_mut(|d| d.insert_temp(id, state));
        response
    }
}

impl ColorPicker<'_> {
    /// Hex input, a mode switch and one number box per channel.
    fn inputs_ui(&self, ui: &mut Ui, id: Id, state: &mut PickerState) {
        ui.horizontal(|ui| {
            let names = ["RGB", "HSV"];
            let current = names[state.mode as usize];
            ComboBox::new(id.with("mode"), current).show_ui(ui, |ui| {
                combo_value(ui, &mut state.mode, ChannelMode::Rgb, "RGB");
                combo_value(ui, &mut state.mode, ChannelMode::Hsv, "HSV");
            });
            let hex = TextBox::new(&mut state.hex)
                .id_salt(id.with("hex"))
                .width(120.0)
                .show(ui)
                .response
                .response;
            state.hex_editing = hex.has_focus();
            if hex.lost_focus()
                && let Some(color) = parse_hex(&state.hex)
            {
                let [r, g, b, a] = color.to_srgba_unmultiplied();
                let [h, s, v] = rgb_to_hsv([r, g, b].map(|c| f32::from(c) / 255.0));
                state.hsva = [
                    h,
                    s,
                    v,
                    if self.alpha {
                        f32::from(a) / 255.0
                    } else {
                        1.0
                    },
                ];
            }
        });

        let channel = |ui: &mut Ui, name: &str, value: &mut f64, max: f64, salt: &str| {
            ui.horizontal(|ui| {
                let changed = ui
                    .add(
                        NumberBox::new(value)
                            .range(0.0..=max)
                            .width(120.0)
                            .id_salt(id.with(salt)),
                    )
                    .changed();
                ui.label(name);
                changed
            })
            .inner
        };
        match state.mode {
            ChannelMode::Rgb => {
                let color = hsva_to_color(state.hsva);
                let [r, g, b, _] = color.to_srgba_unmultiplied();
                let mut rgb = [r, g, b].map(f64::from);
                let mut changed = false;
                for (i, name) in ["Red", "Green", "Blue"].iter().enumerate() {
                    changed |= channel(ui, name, &mut rgb[i], 255.0, name);
                }
                if changed {
                    let [h, s, v] = rgb_to_hsv(rgb.map(|c| c as f32 / 255.0));
                    state.hsva = [
                        if s == 0.0 { state.hsva[0] } else { h },
                        s,
                        v,
                        state.hsva[3],
                    ];
                }
            }
            ChannelMode::Hsv => {
                let mut hsv = [
                    f64::from(state.hsva[0] * 360.0),
                    f64::from(state.hsva[1] * 100.0),
                    f64::from(state.hsva[2] * 100.0),
                ];
                let mut changed = false;
                for (i, (name, max)) in [("Hue", 359.0), ("Saturation", 100.0), ("Value", 100.0)]
                    .iter()
                    .enumerate()
                {
                    changed |= channel(ui, name, &mut hsv[i], *max, name);
                }
                if changed {
                    state.hsva = [
                        hsv[0] as f32 / 360.0,
                        hsv[1] as f32 / 100.0,
                        hsv[2] as f32 / 100.0,
                        state.hsva[3],
                    ];
                }
            }
        }
        if self.alpha {
            let mut alpha = f64::from((state.hsva[3] * 100.0).round());
            if channel(ui, "Opacity", &mut alpha, 100.0, "Opacity") {
                state.hsva[3] = alpha as f32 / 100.0;
            }
        }
    }
}

/// The hue and saturation area, at full value so it stays readable.
fn spectrum(ui: &mut Ui, id: Id, shape: ColorSpectrumShape, hsva: &mut [f32; 4]) {
    let theme = FluentTheme::of(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(SPECTRUM), Sense::hover());
    let response = ui.interact(rect, id, Sense::click_and_drag());
    response.widget_info(|| WidgetInfo::labeled(Role::ColorWell, true, "Color spectrum"));
    let center = rect.center();
    let radius = SPECTRUM / 2.0;

    if let Some(p) = response.interact_pointer_pos()
        && (response.dragged() || response.clicked() || response.is_pointer_button_down_on())
    {
        match shape {
            ColorSpectrumShape::Ring => {
                let d = p - center;
                hsva[0] = (d.y.atan2(d.x) / TAU).rem_euclid(1.0);
                hsva[1] = (d.length() / radius).min(1.0);
            }
            ColorSpectrumShape::Box => {
                hsva[0] = ((p.x - rect.left()) / rect.width()).clamp(0.0, 0.9999);
                hsva[1] = (1.0 - (p.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
            }
        }
    }

    let painter = ui.painter();
    let mut mesh = Mesh::default();
    const STEPS: u32 = 120;
    match shape {
        ColorSpectrumShape::Ring => {
            // A fan from the white center to fully saturated hues: vertex
            // colors interpolate linearly, which is exactly HSV saturation.
            mesh.colored_vertex(center, Color32::WHITE);
            for i in 0..=STEPS {
                let hue = i as f32 / STEPS as f32;
                let angle = hue * TAU;
                mesh.colored_vertex(
                    center + radius * vec2(angle.cos(), angle.sin()),
                    hsva_to_color([hue, 1.0, 1.0, 1.0]),
                );
            }
            for i in 1..=STEPS {
                mesh.add_triangle(0, i, i + 1);
            }
        }
        ColorSpectrumShape::Box => {
            for i in 0..=STEPS {
                let hue = i as f32 / STEPS as f32;
                let x = rect.left() + hue * rect.width();
                mesh.colored_vertex(pos2(x, rect.top()), hsva_to_color([hue, 1.0, 1.0, 1.0]));
                mesh.colored_vertex(pos2(x, rect.bottom()), Color32::WHITE);
            }
            for i in 0..STEPS {
                let a = 2 * i;
                mesh.add_triangle(a, a + 1, a + 2);
                mesh.add_triangle(a + 1, a + 2, a + 3);
            }
        }
    }
    painter.add(mesh);
    if shape == ColorSpectrumShape::Box {
        painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(1.0, theme.resources.control_stroke_color_default),
            StrokeKind::Inside,
        );
    }

    let marker = match shape {
        ColorSpectrumShape::Ring => {
            let angle = hsva[0] * TAU;
            center + hsva[1] * radius * vec2(angle.cos(), angle.sin())
        }
        ColorSpectrumShape::Box => pos2(
            rect.left() + hsva[0] * rect.width(),
            rect.top() + (1.0 - hsva[1]) * rect.height(),
        ),
    };
    let under = hsva_to_color([hsva[0], hsva[1], 1.0, 1.0]);
    painter.circle_stroke(marker, 7.5, Stroke::new(2.0, contrast(under)));
    if response.has_focus() {
        paint_focus_border(painter, rect, &theme);
    }
}

/// Black or white, whichever reads better on `color`, as fluent_ui decides.
fn contrast(color: Color32) -> Color32 {
    let lin = |c: u8| {
        let c = f32::from(c) / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let l = 0.2126 * lin(color.r()) + 0.7152 * lin(color.g()) + 0.0722 * lin(color.b());
    if (l + 0.05) * (l + 0.05) > 0.30 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

/// The tall swatch beside the spectrum, over a checkerboard for alpha.
fn preview(ui: &mut Ui, color: Color32) {
    let theme = FluentTheme::of(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(vec2(PREVIEW, SPECTRUM), Sense::hover());
    paint_checkerboard(ui, rect, 4.0);
    ui.painter().rect_filled(rect, 4.0, color);
    ui.painter().rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0, theme.resources.control_stroke_color_default),
        StrokeKind::Inside,
    );
}

/// A 12px track filled with a gradient from `colors[0]` to `colors[1]`, and
/// a ring thumb at `value` (0 to 1).
fn gradient_slider(
    ui: &mut Ui,
    id: Id,
    label: &str,
    value: &mut f32,
    colors: [Color32; 2],
    checkered: bool,
) {
    let theme = FluentTheme::of(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(vec2(WIDTH, SLIDER), Sense::hover());
    let thumb_radius = 10.0;
    let hit = rect.expand2(vec2(0.0, thumb_radius - SLIDER / 2.0));
    let response = ui.interact(hit, id, Sense::click_and_drag());
    response.widget_info(|| WidgetInfo::slider(true, f64::from(*value), label));
    let track = rect.shrink2(vec2(SLIDER / 2.0, 0.0));
    if let Some(p) = response.interact_pointer_pos()
        && (response.dragged() || response.clicked() || response.is_pointer_button_down_on())
    {
        *value = ((p.x - track.left()) / track.width()).clamp(0.0, 1.0);
    }
    if response.has_focus() {
        let step = ui.input(|i| {
            i.num_presses(egui::Key::ArrowRight) as f32 - i.num_presses(egui::Key::ArrowLeft) as f32
        });
        *value = (*value + step * 0.01).clamp(0.0, 1.0);
    }

    let painter = ui.painter();
    if checkered {
        paint_checkerboard(ui, rect, SLIDER / 2.0);
    }
    paint_gradient(painter, rect, colors);
    painter.rect_stroke(
        rect,
        SLIDER / 2.0,
        Stroke::new(1.0, theme.resources.control_stroke_color_default),
        StrokeKind::Inside,
    );
    let thumb = pos2(egui::lerp(track.x_range(), *value), rect.center().y);
    let fill = crate::utils::lerp(colors[0], colors[1], *value);
    painter.circle_filled(
        thumb,
        thumb_radius,
        theme.resources.control_solid_fill_color_default,
    );
    painter.circle_filled(thumb, thumb_radius - 3.0, fill.to_opaque());
    painter.circle_stroke(
        thumb,
        thumb_radius,
        Stroke::new(1.0, theme.resources.control_stroke_color_default),
    );
    if response.has_focus() {
        paint_focus_border(painter, rect, &theme);
    }
}

/// A horizontal gradient with rounded ends (a pill of radius height/2).
fn paint_gradient(painter: &egui::Painter, rect: Rect, colors: [Color32; 2]) {
    let r = rect.height() / 2.0;
    let mut mesh = Mesh::default();
    // Caps as fans, the middle as one quad.
    let cap = |mesh: &mut Mesh, center: Pos2, from: f32, color: Color32| {
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(center, color);
        const N: u32 = 12;
        for i in 0..=N {
            let a = from + std::f32::consts::PI * i as f32 / N as f32;
            mesh.colored_vertex(center + r * vec2(a.cos(), a.sin()), color);
        }
        for i in 1..=N {
            mesh.add_triangle(base, base + i, base + i + 1);
        }
    };
    cap(
        &mut mesh,
        pos2(rect.left() + r, rect.center().y),
        TAU / 4.0,
        colors[0],
    );
    cap(
        &mut mesh,
        pos2(rect.right() - r, rect.center().y),
        -TAU / 4.0,
        colors[1],
    );
    let base = mesh.vertices.len() as u32;
    mesh.colored_vertex(pos2(rect.left() + r, rect.top()), colors[0]);
    mesh.colored_vertex(pos2(rect.right() - r, rect.top()), colors[1]);
    mesh.colored_vertex(pos2(rect.right() - r, rect.bottom()), colors[1]);
    mesh.colored_vertex(pos2(rect.left() + r, rect.bottom()), colors[0]);
    mesh.add_triangle(base, base + 1, base + 2);
    mesh.add_triangle(base, base + 2, base + 3);
    painter.add(mesh);
}

/// Light and dark squares behind translucent colors, clipped to a rounded
/// rect by drawing only the squares inside it.
fn paint_checkerboard(ui: &Ui, rect: Rect, radius: f32) {
    let painter = ui.painter_at(rect);
    let size = 4.0;
    painter.rect_filled(rect, radius, Color32::from_gray(255));
    let cols = (rect.width() / size).ceil() as i32;
    let rows = (rect.height() / size).ceil() as i32;
    for row in 0..rows {
        for col in 0..cols {
            if (row + col) % 2 == 0 {
                continue;
            }
            let cell = Rect::from_min_size(
                rect.min + vec2(col as f32 * size, row as f32 * size),
                Vec2::splat(size),
            );
            // Skip squares in the rounded corners.
            let inner = rect.shrink(radius);
            let nearest = cell.center().clamp(inner.min, inner.max);
            if radius > 0.0 && cell.center().distance(nearest) > radius {
                continue;
            }
            painter.rect_filled(cell, 0.0, Color32::from_gray(204));
        }
    }
}

/// sRGB components (0 to 1) to hue, saturation and value (0 to 1).
fn rgb_to_hsv([r, g, b]: [f32; 3]) -> [f32; 3] {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d == 0.0 {
        0.0
    } else if max == r {
        ((g - b) / d).rem_euclid(6.0)
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    } / 6.0;
    let s = if max == 0.0 { 0.0 } else { d / max };
    [h, s, max]
}

fn hsv_to_rgb([h, s, v]: [f32; 3]) -> [f32; 3] {
    let h = h.rem_euclid(1.0) * 6.0;
    let c = v * s;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    [r + m, g + m, b + m]
}

fn hsva_to_color([h, s, v, a]: [f32; 4]) -> Color32 {
    let [r, g, b] = hsv_to_rgb([h, s, v]).map(|c| (c * 255.0).round() as u8);
    Color32::from_rgba_unmultiplied(r, g, b, (a * 255.0).round() as u8)
}

fn to_hex(color: Color32, alpha: bool) -> String {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    if alpha {
        format!("#{a:02X}{r:02X}{g:02X}{b:02X}")
    } else {
        format!("#{r:02X}{g:02X}{b:02X}")
    }
}

/// `#RRGGBB` or `#AARRGGBB`, with or without the `#`.
fn parse_hex(text: &str) -> Option<Color32> {
    let hex = text.trim().trim_start_matches('#');
    let value = u32::from_str_radix(hex, 16).ok()?;
    match hex.len() {
        6 => {
            let [_, r, g, b] = value.to_be_bytes();
            Some(Color32::from_rgb(r, g, b))
        }
        8 => {
            let [a, r, g, b] = value.to_be_bytes();
            Some(Color32::from_rgba_unmultiplied(r, g, b, a))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_round_trip() {
        for c in [
            [255, 0, 0],
            [12, 200, 99],
            [0, 0, 0],
            [128, 128, 128],
            [1, 2, 250],
        ] {
            let rgb = c.map(|v: u8| f32::from(v) / 255.0);
            let back = hsv_to_rgb(rgb_to_hsv(rgb)).map(|v| (v * 255.0).round() as u8);
            assert_eq!(back, c);
        }
    }

    #[test]
    fn hex() {
        let c = Color32::from_rgba_unmultiplied(0x12, 0x34, 0x56, 0x80);
        assert_eq!(to_hex(c, true), "#80123456");
        assert_eq!(parse_hex("#80123456"), Some(c));
        assert_eq!(
            parse_hex("123456"),
            Some(Color32::from_rgb(0x12, 0x34, 0x56))
        );
        assert_eq!(parse_hex("#12345"), None);
    }
}
