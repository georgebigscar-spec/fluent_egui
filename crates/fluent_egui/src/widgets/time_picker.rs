use egui::{Align, AsIdSalt, IdSalt, Response, Ui, Widget};

use super::pickers::{PickerColumn, PickerPopupState, PopupAction, picker_button, picker_popup};
use crate::date::Time;

/// A button showing hour, minute and AM/PM that opens scrolling columns to
/// pick them, fluent_ui's `TimePicker`.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui, alarm: &mut Option<fluent_egui::Time>) {
/// ui.add(fluent_egui::TimePicker::new("alarm", alarm).minute_increment(5));
/// # }
/// ```
pub struct TimePicker<'a> {
    id_salt: IdSalt,
    value: &'a mut Option<Time>,
    use_24_hour: bool,
    minute_increment: u8,
}

impl<'a> TimePicker<'a> {
    pub fn new(id_salt: impl AsIdSalt, value: &'a mut Option<Time>) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            value,
            use_24_hour: false,
            minute_increment: 1,
        }
    }

    /// Hours 0 to 23 and no AM/PM column.
    pub fn use_24_hour(mut self, on: bool) -> Self {
        self.use_24_hour = on;
        self
    }

    /// Offer only every `n`th minute, e.g. 15 for quarter hours.
    pub fn minute_increment(mut self, n: u8) -> Self {
        self.minute_increment = n.clamp(1, 30);
        self
    }
}

impl Widget for TimePicker<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let id = ui.make_persistent_id(self.id_salt);
        let mut popup = PickerPopupState::load(ui, id);
        let step = self.minute_increment;
        let hours: Vec<String> = if self.use_24_hour {
            (0..24).map(|h| format!("{h:02}")).collect()
        } else {
            (1..=12).map(|h| h.to_string()).collect()
        };
        let mut columns = vec![
            PickerColumn {
                items: hours,
                width: 80.0,
                looping: true,
                align: Align::Center,
                placeholder: "hour",
            },
            PickerColumn {
                items: (0..60)
                    .step_by(usize::from(step))
                    .map(|m| format!("{m:02}"))
                    .collect(),
                width: 80.0,
                looping: true,
                align: Align::Center,
                placeholder: "minute",
            },
        ];
        if !self.use_24_hour {
            columns.push(PickerColumn {
                items: vec!["AM".to_owned(), "PM".to_owned()],
                width: 80.0,
                looping: false,
                align: Align::Center,
                placeholder: "AM",
            });
        }

        // Time to popup positions and back.
        let to_indices = |t: Time| {
            let minute = i64::from(t.minute / step);
            if self.use_24_hour {
                vec![i64::from(t.hour), minute]
            } else {
                // Positions 0..12 are the hours 1..12.
                let hour12 = (t.hour + 11) % 12;
                vec![i64::from(hour12), minute, i64::from(t.hour >= 12)]
            }
        };
        let from_indices = |i: &[i64]| {
            let minute = (i[1].rem_euclid(columns[1].items.len() as i64) as u8) * step;
            let hour = if self.use_24_hour {
                i[0].rem_euclid(24) as u8
            } else {
                let hour12 = i[0].rem_euclid(12) as u8 + 1;
                let pm = i[2] == 1;
                hour12 % 12 + if pm { 12 } else { 0 }
            };
            Time { hour, minute }
        };

        let values = self.value.map(|t| {
            let i = to_indices(t);
            columns
                .iter()
                .zip(&i)
                .map(|(c, i)| c.items[*i as usize].clone())
                .collect()
        });
        let label = self
            .value
            .map_or_else(|| "Pick a time".to_owned(), |t| t.to_string());
        let mut response = picker_button(ui, id, &columns, values, &label, popup.open);

        if response.clicked() && !popup.open {
            let now = self.value.unwrap_or(Time {
                hour: 12,
                minute: 0,
            });
            popup.open = true;
            popup.indices = to_indices(now);
        } else if popup.open {
            let action = picker_popup(ui, id, response.rect, &columns, &mut popup.indices);
            if let Some(action) = action {
                popup.open = false;
                if action == PopupAction::Accept {
                    let picked = Some(from_indices(&popup.indices));
                    if picked != *self.value {
                        *self.value = picked;
                        response.mark_changed();
                    }
                }
            }
        }
        popup.store(ui, id);
        response
    }
}
