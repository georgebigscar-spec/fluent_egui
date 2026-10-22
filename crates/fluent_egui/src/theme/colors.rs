//! The Fluent color palette, fluent_ui's `Colors`.

use egui::Color32;

use super::AccentColor;

const fn rgb(hex: u32) -> Color32 {
    let [_, r, g, b] = hex.to_be_bytes();
    Color32::from_rgb(r, g, b)
}

const fn accent(shades: [u32; 7]) -> AccentColor {
    AccentColor {
        darkest: rgb(shades[0]),
        darker: rgb(shades[1]),
        dark: rgb(shades[2]),
        normal: rgb(shades[3]),
        light: rgb(shades[4]),
        lighter: rgb(shades[5]),
        lightest: rgb(shades[6]),
    }
}

pub const TRANSPARENT: Color32 = Color32::TRANSPARENT;
pub const BLACK: Color32 = Color32::BLACK;
pub const WHITE: Color32 = Color32::WHITE;

/// Grey shades from 10 (lightest) to 220 (darkest), see [`grey`].
pub const GREY_SHADES: [(u16, Color32); 22] = [
    (220, rgb(0x11100F)),
    (210, rgb(0x161514)),
    (200, rgb(0x1B1A19)),
    (190, rgb(0x201F1E)),
    (180, rgb(0x252423)),
    (170, rgb(0x292827)),
    (160, rgb(0x323130)),
    (150, rgb(0x3B3A39)),
    (140, rgb(0x484644)),
    (130, rgb(0x605E5C)),
    (120, rgb(0x797775)),
    (110, rgb(0x8A8886)),
    (100, rgb(0x979593)),
    (90, rgb(0xA19F9D)),
    (80, rgb(0xB3B0AD)),
    (70, rgb(0xBEBBB8)),
    (60, rgb(0xC8C6C4)),
    (50, rgb(0xD2D0CE)),
    (40, rgb(0xE1DFDD)),
    (30, rgb(0xEDEBE9)),
    (20, rgb(0xF3F2F1)),
    (10, rgb(0xFAF9F8)),
];

/// A grey shade, like `Colors.grey[shade]`. Panics on a shade that is not a
/// multiple of 10 between 10 and 220.
pub fn grey(shade: u16) -> Color32 {
    GREY_SHADES
        .iter()
        .find(|(s, _)| *s == shade)
        .map(|(_, c)| *c)
        .unwrap_or_else(|| panic!("no grey shade {shade}"))
}

pub const YELLOW: AccentColor = accent([
    0xf9a825, 0xfbc02d, 0xfdd835, 0xffeb3b, 0xffee58, 0xfff176, 0xfff59d,
]);
pub const ORANGE: AccentColor = accent([
    0x993d07, 0xac4508, 0xd1540a, 0xf7630c, 0xf87a30, 0xf99154, 0xfa9e68,
]);
pub const RED: AccentColor = accent([
    0x8f0a15, 0xa20b18, 0xb90d1c, 0xe81123, 0xec404f, 0xee5865, 0xf06b76,
]);
pub const MAGENTA: AccentColor = accent([
    0x6f0061, 0x7e006e, 0x90007e, 0xb4009e, 0xc333b1, 0xca4cbb, 0xd060c2,
]);
pub const PURPLE: AccentColor = accent([
    0x472f68, 0x513576, 0x644293, 0x744da9, 0x8664b4, 0x9d82c2, 0xa890c9,
]);
pub const BLUE: AccentColor = accent([
    0x004a83, 0x005494, 0x0066b4, 0x0078d4, 0x268cda, 0x4ca0e0, 0x60abe4,
]);
pub const TEAL: AccentColor = accent([
    0x006e5b, 0x007c67, 0x00977d, 0x00b294, 0x26bda4, 0x4cc9b4, 0x60cfbc,
]);
pub const GREEN: AccentColor = accent([
    0x094c09, 0x0c5d0c, 0x0e6f0e, 0x107c10, 0x278927, 0x4b9c4b, 0x6aad6a,
]);

/// Every accent color above, in fluent_ui's order.
pub const ACCENT_COLORS: [(&str, AccentColor); 8] = [
    ("yellow", YELLOW),
    ("orange", ORANGE),
    ("red", RED),
    ("magenta", MAGENTA),
    ("purple", PURPLE),
    ("blue", BLUE),
    ("teal", TEAL),
    ("green", GREEN),
];

pub const WARNING_PRIMARY: Color32 = rgb(0xd83b01);
/// Warning background: `.0` for dark mode, `.1` for light mode.
pub const WARNING_SECONDARY: (Color32, Color32) = (rgb(0x433519), rgb(0xfff4ce));
pub const ERROR_PRIMARY: Color32 = rgb(0xa80000);
/// Error background: `.0` for dark mode, `.1` for light mode.
pub const ERROR_SECONDARY: (Color32, Color32) = (rgb(0x442726), rgb(0xfde7e9));
pub const SUCCESS_PRIMARY: Color32 = rgb(0x107c10);
/// Success background: `.0` for dark mode, `.1` for light mode.
pub const SUCCESS_SECONDARY: (Color32, Color32) = (rgb(0x393d1b), rgb(0xdff6dd));
