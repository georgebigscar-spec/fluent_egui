use super::ControlState;

/// One value per interaction state, the egui counterpart of Flutter's
/// `WidgetStateProperty` as fluent_ui uses it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct States<T> {
    pub rest: T,
    pub hovered: T,
    pub pressed: T,
    pub disabled: T,
}

impl<T: Copy> States<T> {
    pub const fn new(rest: T, hovered: T, pressed: T, disabled: T) -> Self {
        Self {
            rest,
            hovered,
            pressed,
            disabled,
        }
    }

    /// The same value in every state.
    pub const fn all(value: T) -> Self {
        Self::new(value, value, value, value)
    }

    pub fn resolve(&self, state: ControlState) -> T {
        state.pick(self.rest, self.hovered, self.pressed, self.disabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_wins_over_pressed_over_hovered() {
        let s = States::new(0, 1, 2, 3);
        let mut state = ControlState {
            hovered: true,
            ..Default::default()
        };
        assert_eq!(s.resolve(state), 1);
        state.pressed = true;
        assert_eq!(s.resolve(state), 2);
        state.disabled = true;
        assert_eq!(s.resolve(state), 3);
        assert_eq!(s.resolve(ControlState::default()), 0);
    }
}
