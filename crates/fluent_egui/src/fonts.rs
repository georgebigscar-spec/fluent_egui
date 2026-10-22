//! Font registration helpers.
//!
//! fluent_egui does not bundle fonts: Segoe UI and the icon fonts used by
//! fluent_ui are not freely redistributable. Load the ones you are allowed to
//! ship and register them here.

use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily};

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
