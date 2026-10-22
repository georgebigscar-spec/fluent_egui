use egui::Color32;

use super::Brightness;
use crate::utils::{lerp, with_alpha};

/// A color with seven named shades, like fluent_ui's `AccentColor`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccentColor {
    pub darkest: Color32,
    pub darker: Color32,
    /// Usually used for the pressed state.
    pub dark: Color32,
    pub normal: Color32,
    /// Usually used for the hovered state.
    pub light: Color32,
    pub lighter: Color32,
    pub lightest: Color32,
}

impl AccentColor {
    /// Derives the shades by mixing `color` with black and white, with the
    /// same factors as fluent_ui's `Color.toAccentColor()`.
    pub fn from_color(color: Color32) -> Self {
        Self {
            darkest: lerp(color, Color32::BLACK, 0.38),
            darker: lerp(color, Color32::BLACK, 0.30),
            dark: lerp(color, Color32::BLACK, 0.15),
            normal: color,
            light: lerp(color, Color32::WHITE, 0.15),
            lighter: lerp(color, Color32::WHITE, 0.30),
            lightest: lerp(color, Color32::WHITE, 0.38),
        }
    }

    /// The accent fill of a control at rest.
    pub fn default_brush_for(&self, brightness: Brightness) -> Color32 {
        match brightness {
            Brightness::Light => self.dark,
            Brightness::Dark => self.lighter,
        }
    }

    /// The accent fill of a hovered control.
    pub fn secondary_brush_for(&self, brightness: Brightness) -> Color32 {
        with_alpha(self.default_brush_for(brightness), 0.9)
    }

    /// The accent fill of a pressed control.
    pub fn tertiary_brush_for(&self, brightness: Brightness) -> Color32 {
        with_alpha(self.default_brush_for(brightness), 0.8)
    }
}
