use egui::{FontFamily, RichText};

use crate::icons::{FLUENT_ICONS_FAMILY, WINDOWS_ICONS_FAMILY};

/// A [`crate::icons::fluent`] icon as text, for labels and buttons.
///
/// The `FluentIcons` font family must be registered (see [`crate::fonts`]),
/// otherwise egui panics when laying it out.
pub fn icon(ch: char) -> RichText {
    RichText::new(ch).family(FontFamily::Name(FLUENT_ICONS_FAMILY.into()))
}

/// A [`crate::icons::windows`] icon as text. Needs the `SegoeIcons` family.
pub fn windows_icon(ch: char) -> RichText {
    RichText::new(ch).family(FontFamily::Name(WINDOWS_ICONS_FAMILY.into()))
}
