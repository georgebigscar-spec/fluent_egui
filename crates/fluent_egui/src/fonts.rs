//! Font registration helpers.
//!
//! Segoe UI and the icon fonts used by fluent_ui are not freely
//! redistributable, so they are not bundled. Load the ones you are allowed to
//! ship and register them here. With the default `selawik` feature,
//! [`add_selawik`] registers Selawik, the open Segoe UI fallback, including a
//! semibold weight for headings.

use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily};

/// The family [`add_selawik`] registers for semibold text. Pass it to
/// [`crate::FluentTheme::with_semibold_font`].
pub const SEMIBOLD_FAMILY: &str = "FluentSemibold";

/// Makes Selawik the default text font and registers Selawik Semibold as
/// [`SEMIBOLD_FAMILY`]. Selawik covers Latin only; other scripts fall back to
/// the fonts already in the proportional family.
///
/// ```no_run
/// # fn setup(ctx: &egui::Context) {
/// let mut defs = egui::FontDefinitions::default();
/// fluent_egui::fonts::add_selawik(&mut defs);
/// ctx.set_fonts(defs);
/// fluent_egui::FluentTheme::new(fluent_egui::Brightness::Light)
///     .with_semibold_font()
///     .install(ctx);
/// # }
/// ```
#[cfg(feature = "selawik")]
pub fn add_selawik(defs: &mut FontDefinitions) {
    const REGULAR: &[u8] = include_bytes!("../assets/fonts/Selawik-Regular.ttf");
    const SEMIBOLD: &[u8] = include_bytes!("../assets/fonts/Selawik-Semibold.ttf");
    set_proportional(defs, "Selawik", REGULAR.to_vec());
    defs.font_data.insert(
        "Selawik-Semibold".to_owned(),
        Arc::new(FontData::from_static(SEMIBOLD)),
    );
    let mut family = vec!["Selawik-Semibold".to_owned()];
    family.extend(
        defs.families
            .get(&FontFamily::Proportional)
            .cloned()
            .unwrap_or_default(),
    );
    defs.families
        .insert(FontFamily::Name(SEMIBOLD_FAMILY.into()), family);
}

/// Adds `bytes` as the only font of the named family `family`.
///
/// Use it for the icon fonts ([`crate::icons::FLUENT_ICONS_FAMILY`],
/// [`crate::icons::WINDOWS_ICONS_FAMILY`]) or a semibold text font, which you
/// then pass to [`crate::Typography::with_semibold_family`].
pub fn add_family(defs: &mut FontDefinitions, family: &str, bytes: Vec<u8>) {
    defs.font_data
        .insert(family.to_owned(), Arc::new(FontData::from_owned(bytes)));
    defs.families
        .insert(FontFamily::Name(family.into()), vec![family.to_owned()]);
}

/// Puts `bytes` first in the proportional family, so it becomes the default
/// text font (Segoe UI Variable on Windows, or the open Selawik fallback).
pub fn set_proportional(defs: &mut FontDefinitions, name: &str, bytes: Vec<u8>) {
    defs.font_data
        .insert(name.to_owned(), Arc::new(FontData::from_owned(bytes)));
    defs.families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, name.to_owned());
}
