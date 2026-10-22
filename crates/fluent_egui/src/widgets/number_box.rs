use std::ops::RangeInclusive;

use egui::{AsIdSalt, IdSalt, Key, Rect, Response, Sense, Ui, Vec2, Widget, pos2};

use super::text_box::TextBox;
use crate::FluentTheme;
use crate::utils::{ControlState, States, paint_chevron_down, paint_chevron_up};

/// Where a [`NumberBox`] shows its up/down buttons, fluent_ui's
/// `SpinButtonPlacementMode`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpinButtonPlacement {
    /// Two buttons inside the field, on the right.
    #[default]
    Inline,
    /// No buttons; arrow and page keys still work.
    None,
}

/// A numeric field, fluent_ui's `NumberBox`.
///
/// Type a number and press Enter (or leave the field) to commit it.
/// Up/Down change it by `small_change`, PageUp/PageDown by `large_change`.
/// Invalid input reverts to the last value.
pub struct NumberBox<'a> {
    value: &'a mut f64,
    range: RangeInclusive<f64>,
    small_change: f64,
    large_change: f64,
    precision: Option<usize>,
    spin_buttons: SpinButtonPlacement,
    width: Option<f32>,
    placeholder: Option<String>,
    id_salt: Option<IdSalt>,
}

impl<'a> NumberBox<'a> {
    pub fn new(value: &'a mut f64) -> Self {
        Self {
            value,
            range: f64::NEG_INFINITY..=f64::INFINITY,
            small_change: 1.0,
            large_change: 10.0,
            precision: None,
            spin_buttons: SpinButtonPlacement::default(),
            width: None,
            placeholder: None,
            id_salt: None,
        }
    }

    pub fn range(mut self, range: RangeInclusive<f64>) -> Self {
        self.range = range;
        self
    }

    pub fn small_change(mut self, step: f64) -> Self {
        self.small_change = step;
        self
    }

    pub fn large_change(mut self, step: f64) -> Self {
        self.large_change = step;
        self
    }

    /// Number of decimals to show and round to. `None` shows the value as is.
    pub fn precision(mut self, decimals: usize) -> Self {
        self.precision = Some(decimals);
        self
    }

    pub fn spin_buttons(mut self, placement: SpinButtonPlacement) -> Self {
        self.spin_buttons = placement;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn id_salt(mut self, id_salt: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id_salt));
        self
    }

    fn clamp(&self, v: f64) -> f64 {
        let v = v.clamp(*self.range.start(), *self.range.end());
        match self.precision {
            Some(p) => {
                let scale = 10f64.powi(p as i32);
                (v * scale).round() / scale
            }
            None => v,
        }
    }

    fn format(&self, v: f64) -> String {
        match self.precision {
            Some(p) => format!("{v:.p$}"),
            None => format!("{v}"),
        }
    }
}

/// Parses user input, accepting a comma as the decimal separator.
fn parse(text: &str) -> Option<f64> {
    text.trim()
        .replace(',', ".")
        .parse()
        .ok()
        .filter(|v: &f64| v.is_finite())
}

impl Widget for NumberBox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = FluentTheme::of(ui.ctx());
        let salt = self
            .id_salt
            .unwrap_or_else(|| IdSalt::new(ui.next_auto_id()));
        let buffer_id = ui.make_persistent_id(salt).with("buffer");
        let spin = self.spin_buttons == SpinButtonPlacement::Inline;

        // While editing, the text lives in a buffer; otherwise show the value.
        let mut text: String = ui
            .data(|d| d.get_temp(buffer_id))
            .unwrap_or_else(|| self.format(*self.value));
        let mut text_box =
            TextBox::new(&mut text)
                .id_salt(salt)
                .trailing(if spin { 64.0 } else { 0.0 });
        if let Some(w) = self.width {
            text_box = text_box.width(w);
        }
        if let Some(p) = &self.placeholder {
            text_box = text_box.placeholder(p.clone());
        }
        let output = text_box.show(ui);
        let mut response = output.response.response;

        let old = *self.value;
        let mut new = old;
        let mut commit_text = false;
        if response.has_focus() {
            ui.input(|i| {
                let current = parse(&text).unwrap_or(old);
                if i.key_pressed(Key::ArrowUp) {
                    new = current + self.small_change;
                } else if i.key_pressed(Key::ArrowDown) {
                    new = current - self.small_change;
                } else if i.key_pressed(Key::PageUp) {
                    new = current + self.large_change;
                } else if i.key_pressed(Key::PageDown) {
                    new = current - self.large_change;
                }
            });
        }
        if response.lost_focus() {
            commit_text = true;
        }

        if spin {
            let rect = response.rect;
            let r = &theme.resources;
            for (i, delta) in [(1, self.small_change), (0, -self.small_change)] {
                let button = Rect::from_min_size(
                    pos2(
                        rect.right() - 4.0 - 30.0 * (i as f32 + 1.0),
                        rect.top() + 4.0,
                    ),
                    Vec2::new(28.0, rect.height() - 8.0),
                );
                let b = ui.interact(button, buffer_id.with(("spin", i)), Sense::click());
                let state = ControlState::from_response(&b);
                let fill = States::new(
                    r.subtle_fill_color_transparent,
                    r.subtle_fill_color_secondary,
                    r.subtle_fill_color_tertiary,
                    r.subtle_fill_color_transparent,
                )
                .resolve(state);
                ui.painter().rect_filled(button, 4.0, fill);
                let color = if state.disabled {
                    r.text_fill_color_disabled
                } else {
                    r.text_fill_color_secondary
                };
                if delta > 0.0 {
                    paint_chevron_up(ui.painter(), button.center(), color);
                } else {
                    paint_chevron_down(ui.painter(), button.center(), color);
                }
                if b.clicked() {
                    new = parse(&text).unwrap_or(old) + delta;
                }
            }
        }

        if new != old || commit_text {
            if commit_text && new == old {
                new = parse(&text).unwrap_or(old);
            }
            let new = self.clamp(new);
            if new != old {
                *self.value = new;
                response.mark_changed();
            }
            text = self.format(new);
        }
        let editing = response.has_focus() && !commit_text;
        ui.data_mut(|d| {
            if editing {
                d.insert_temp(buffer_id, text);
            } else {
                d.remove::<String>(buffer_id);
            }
        });
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_comma_and_rejects_garbage() {
        assert_eq!(parse(" 1,5 "), Some(1.5));
        assert_eq!(parse("42"), Some(42.0));
        assert_eq!(parse("abc"), None);
        assert_eq!(parse("inf"), None);
    }

    #[test]
    fn clamps_and_rounds() {
        let mut v = 0.0;
        let nb = NumberBox::new(&mut v).range(0.0..=10.0).precision(1);
        assert_eq!(nb.clamp(4.26), 4.3);
        assert_eq!(nb.clamp(-2.0), 0.0);
        assert_eq!(nb.clamp(99.0), 10.0);
        assert_eq!(nb.format(2.0), "2.0");
    }
}
