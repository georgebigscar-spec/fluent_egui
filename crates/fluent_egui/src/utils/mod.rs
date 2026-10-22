//! Building blocks shared by the widgets.

mod animate;
mod color;
mod focus;
mod glyphs;
mod interaction;
mod paint;
mod states;

pub use animate::animate_color;
pub use color::{lerp, with_alpha};
pub use focus::paint_focus_border;
pub use glyphs::{paint_chevron_down, paint_chevron_up, paint_eye};
pub use interaction::ControlState;
pub use paint::{paint_arc, paint_elevation_border};
pub use states::States;
