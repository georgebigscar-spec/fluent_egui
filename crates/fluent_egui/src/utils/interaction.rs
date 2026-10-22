use egui::Response;

/// The interaction state of a control, the egui counterpart of fluent_ui's
/// `HoverButton` states. Widgets pick their colors from it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ControlState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub disabled: bool,
}

impl ControlState {
    pub fn from_response(response: &Response) -> Self {
        let disabled = !response.enabled();
        Self {
            hovered: !disabled && response.hovered(),
            pressed: !disabled && response.is_pointer_button_down_on(),
            focused: !disabled && response.has_focus(),
            disabled,
        }
    }

    /// Chooses between the four values every Fluent control defines:
    /// rest, pointer over, pressed and disabled.
    pub fn pick<T>(self, rest: T, hovered: T, pressed: T, disabled: T) -> T {
        if self.disabled {
            disabled
        } else if self.pressed {
            pressed
        } else if self.hovered {
            hovered
        } else {
            rest
        }
    }
}
