//! Building blocks shared by the widgets.

mod color;
mod focus;
mod interaction;

pub use color::{lerp, with_alpha};
pub use focus::paint_focus_border;
pub use interaction::ControlState;
