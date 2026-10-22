//! Fluent Design widgets for [egui](https://github.com/emilk/egui), ported from
//! the Flutter [fluent_ui](https://github.com/bdlukaa/fluent_ui) package.
//!
//! Install a theme once, then use the widgets:
//!
//! ```no_run
//! # fn ui(ctx: &egui::Context, ui: &mut egui::Ui) {
//! use fluent_egui::{Brightness, FluentTheme};
//!
//! FluentTheme::new(Brightness::Dark).install(ctx);
//! ui.label(FluentTheme::of(ctx).typography.title.rich("Hello, Fluent"));
//! ui.add(fluent_egui::Divider::horizontal());
//! # }
//! ```

pub mod date;
pub mod fonts;
pub mod icons;
pub mod theme;
pub mod utils;
pub mod widgets;

pub use date::{Date, Time, Weekday};
pub use theme::{AccentColor, Brightness, FluentTheme, ResourceDictionary, Typography, colors};
pub use widgets::*;
