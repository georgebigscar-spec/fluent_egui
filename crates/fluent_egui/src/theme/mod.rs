//! Theme data, fluent_ui's `lib/src/styles`.

mod accent;
pub mod colors;
mod resources;
mod typography;

use std::sync::Arc;
use std::time::Duration;

use egui::{Color32, Context, CornerRadius, Id, Stroke, TextStyle as EguiTextStyle, Visuals};

pub use accent::AccentColor;
pub use resources::ResourceDictionary;
pub use typography::{TextStyle, Typography};

use crate::utils::with_alpha;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Brightness {
    #[default]
    Light,
    Dark,
}

impl Brightness {
    pub fn is_dark(self) -> bool {
        self == Self::Dark
    }
}

/// The two-stroke focus border, fluent_ui's `FocusThemeData.standard`.
#[derive(Clone, Debug, PartialEq)]
pub struct FocusTheme {
    pub corner_radius: f32,
    pub primary_width: f32,
    pub primary_color: Color32,
    pub secondary_width: f32,
    pub secondary_color: Color32,
}

/// fluent_ui's `DividerThemeData.standard`.
#[derive(Clone, Debug, PartialEq)]
pub struct DividerTheme {
    pub thickness: f32,
    /// Space left at each end of the line.
    pub margin: f32,
    pub color: Color32,
}

/// The Fluent theme, fluent_ui's `FluentThemeData`.
///
/// Per-control themes (buttons, checkboxes, ...) are added as those controls
/// are ported.
#[derive(Clone, Debug)]
pub struct FluentTheme {
    pub brightness: Brightness,
    pub accent_color: AccentColor,
    pub resources: ResourceDictionary,
    pub typography: Typography,

    pub active_color: Color32,
    pub inactive_color: Color32,
    pub inactive_background_color: Color32,
    pub shadow_color: Color32,
    pub scaffold_background_color: Color32,
    pub acrylic_background_color: Color32,
    pub acrylic_tint_opacity: f32,
    pub acrylic_luminosity_opacity: f32,
    pub acrylic_fallback_color: Color32,
    pub mica_background_color: Color32,
    pub menu_color: Color32,
    pub card_color: Color32,
    pub selection_color: Color32,

    pub faster_animation_duration: Duration,
    pub fast_animation_duration: Duration,
    pub medium_animation_duration: Duration,
    pub slow_animation_duration: Duration,
    /// Easing applied to animations, `Curves.easeInOut` in fluent_ui.
    pub animation_curve: fn(f32) -> f32,

    pub focus: FocusTheme,
    pub divider: DividerTheme,
}

impl FluentTheme {
    /// The default theme for `brightness` with the blue accent.
    pub fn new(brightness: Brightness) -> Self {
        Self::with_accent(brightness, colors::BLUE)
    }

    pub fn light() -> Self {
        Self::new(Brightness::Light)
    }

    pub fn dark() -> Self {
        Self::new(Brightness::Dark)
    }

    pub fn with_accent(brightness: Brightness, accent_color: AccentColor) -> Self {
        let light = !brightness.is_dark();
        let resources = if light {
            ResourceDictionary::LIGHT
        } else {
            ResourceDictionary::DARK
        };
        let pick = |l: u32, d: u32| {
            let [_, r, g, b] = if light { l } else { d }.to_be_bytes();
            Color32::from_rgb(r, g, b)
        };
        Self {
            brightness,
            accent_color,
            typography: Typography::with_color(resources.text_fill_color_primary),
            active_color: Color32::WHITE,
            inactive_color: if light {
                Color32::BLACK
            } else {
                Color32::WHITE
            },
            inactive_background_color: pick(0xd6d6d6, 0x292929),
            shadow_color: if light {
                Color32::BLACK
            } else {
                colors::grey(130)
            },
            scaffold_background_color: resources.layer_on_acrylic_fill_color_default,
            acrylic_background_color: pick(0xfcfcfc, 0x2c2c2c),
            acrylic_tint_opacity: if light { 0.0 } else { 0.15 },
            acrylic_luminosity_opacity: if light { 0.85 } else { 0.96 },
            acrylic_fallback_color: pick(0xf9f9f9, 0x2c2c2c),
            mica_background_color: resources.solid_background_fill_color_base,
            menu_color: pick(0xf9f9f9, 0x2c2c2c),
            card_color: resources.card_background_fill_color_default,
            selection_color: accent_color.normal,
            faster_animation_duration: Duration::from_millis(83),
            fast_animation_duration: Duration::from_millis(167),
            medium_animation_duration: Duration::from_millis(250),
            slow_animation_duration: Duration::from_millis(358),
            animation_curve: ease_in_out,
            focus: FocusTheme {
                corner_radius: 6.0,
                primary_width: 2.0,
                primary_color: resources.focus_stroke_color_outer,
                secondary_width: 1.0,
                secondary_color: resources.focus_stroke_color_inner,
            },
            divider: DividerTheme {
                thickness: 1.0,
                margin: 10.0,
                color: resources.divider_stroke_color_default,
            },
            resources,
        }
    }

    /// The theme installed in `ctx`, or the default light theme.
    pub fn of(ctx: &Context) -> Arc<Self> {
        ctx.data(|d| d.get_temp::<Arc<Self>>(Self::id()))
            .unwrap_or_else(|| Arc::new(Self::light()))
    }

    /// Stores the theme in `ctx` for the widgets and restyles egui's own
    /// widgets to match.
    pub fn install(self, ctx: &Context) {
        let theme = Arc::new(self);
        ctx.data_mut(|d| d.insert_temp(Self::id(), theme.clone()));
        let egui_theme = if theme.brightness.is_dark() {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        };
        ctx.set_theme(egui_theme);
        ctx.style_mut_of(egui_theme, |style| {
            style.visuals = theme.visuals();
            style.animation_time = theme.fast_animation_duration.as_secs_f32();
            for (name, text) in theme.typography.styles() {
                style
                    .text_styles
                    .insert(EguiTextStyle::Name(name.into()), text.font_id());
            }
            let t = &theme.typography;
            style
                .text_styles
                .insert(EguiTextStyle::Heading, t.subtitle.font_id());
            style
                .text_styles
                .insert(EguiTextStyle::Body, t.body.font_id());
            style
                .text_styles
                .insert(EguiTextStyle::Button, t.body.font_id());
            style
                .text_styles
                .insert(EguiTextStyle::Small, t.caption.font_id());
        });
    }

    /// egui [`Visuals`] approximating this theme, so stock egui widgets blend
    /// in with the Fluent ones.
    pub fn visuals(&self) -> Visuals {
        let r = &self.resources;
        let dark = self.brightness.is_dark();
        let accent = self.accent_color.default_brush_for(self.brightness);
        let mut v = if dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        // Left unset on purpose: an override would also recolor the text
        // Fluent widgets paint themselves (e.g. white on accent).
        v.override_text_color = None;
        v.hyperlink_color = accent;
        v.panel_fill = r.solid_background_fill_color_base;
        v.window_fill = self.menu_color;
        v.window_stroke = Stroke::new(1.0, r.surface_stroke_color_flyout);
        v.window_corner_radius = CornerRadius::same(8);
        v.menu_corner_radius = CornerRadius::same(8);
        v.extreme_bg_color = r.control_fill_color_input_active;
        v.faint_bg_color = r.subtle_fill_color_secondary;
        v.code_bg_color = r.control_fill_color_secondary;
        v.selection.bg_fill = with_alpha(accent, if dark { 0.6 } else { 0.4 });
        v.selection.stroke = Stroke::new(1.0, r.text_fill_color_primary);

        let w = &mut v.widgets;
        let radius = CornerRadius::same(4);
        let fg = |c| Stroke::new(1.0, c);
        w.noninteractive.bg_fill = r.card_background_fill_color_default;
        w.noninteractive.weak_bg_fill = r.card_background_fill_color_default;
        w.noninteractive.bg_stroke = fg(r.divider_stroke_color_default);
        w.noninteractive.fg_stroke = fg(r.text_fill_color_primary);
        for (state, fill) in [
            (&mut w.inactive, r.control_fill_color_default),
            (&mut w.hovered, r.control_fill_color_secondary),
            (&mut w.active, r.control_fill_color_tertiary),
            (&mut w.open, r.control_fill_color_tertiary),
        ] {
            state.bg_fill = fill;
            state.weak_bg_fill = fill;
            state.bg_stroke = fg(r.control_stroke_color_default);
            state.fg_stroke = fg(r.text_fill_color_primary);
            state.corner_radius = radius;
            state.expansion = 0.0;
        }
        w.active.fg_stroke = fg(r.text_fill_color_secondary);
        v
    }

    fn id() -> Id {
        Id::unique("fluent_egui::FluentTheme")
    }
}

impl Default for FluentTheme {
    fn default() -> Self {
        Self::light()
    }
}

/// Flutter's `Curves.easeInOut`, cubic-bezier(0.42, 0, 0.58, 1).
pub fn ease_in_out(t: f32) -> f32 {
    cubic_bezier(0.42, 0.0, 0.58, 1.0, t)
}

/// Evaluates a CSS-style cubic bezier easing curve at `t`.
pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let curve = |a: f32, b: f32, s: f32| {
        3.0 * a * s * (1.0 - s).powi(2) + 3.0 * b * s.powi(2) * (1.0 - s) + s.powi(3)
    };
    // Find s with x(s) == t by bisection, which is plenty for UI easing.
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..24 {
        let mid = (lo + hi) / 2.0;
        if curve(x1, x2, mid) < t {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    curve(y1, y2, (lo + hi) / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ease_in_out_is_symmetric() {
        assert!(ease_in_out(0.0).abs() < 1e-4);
        assert!((ease_in_out(1.0) - 1.0).abs() < 1e-4);
        assert!((ease_in_out(0.5) - 0.5).abs() < 1e-3);
        assert!((ease_in_out(0.25) + ease_in_out(0.75) - 1.0).abs() < 1e-3);
    }

    #[test]
    fn accent_brushes_follow_brightness() {
        let blue = colors::BLUE;
        assert_eq!(blue.default_brush_for(Brightness::Light), blue.dark);
        assert_eq!(blue.default_brush_for(Brightness::Dark), blue.lighter);
        assert_eq!(blue.secondary_brush_for(Brightness::Light).a(), 230);
    }

    #[test]
    fn resources_match_winui() {
        // Spot checks against Common_themeresources_any.xaml.
        assert_eq!(
            ResourceDictionary::LIGHT.text_fill_color_primary,
            Color32::from_rgba_unmultiplied(0, 0, 0, 0xe4)
        );
        assert_eq!(
            ResourceDictionary::DARK.text_fill_color_primary,
            Color32::WHITE
        );
        assert_eq!(
            ResourceDictionary::DARK.solid_background_fill_color_base,
            Color32::from_rgb(0x20, 0x20, 0x20)
        );
    }

    #[test]
    fn accent_from_color_keeps_normal() {
        let accent = AccentColor::from_color(colors::BLUE.normal);
        assert_eq!(accent.normal, colors::BLUE.normal);
        assert_eq!(accent.darkest.a(), 255);
    }
}
