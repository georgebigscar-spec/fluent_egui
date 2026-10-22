use egui::{Align, AsIdSalt, IdSalt, Response, Ui, Widget};

use super::pickers::{PickerColumn, PickerPopupState, PopupAction, picker_button, picker_popup};
use crate::Localizations;
use crate::date::{Date, days_in_month};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Field {
    Month,
    Day,
    Year,
}

/// A button showing month, day and year that opens scrolling columns to
/// pick them, fluent_ui's `DatePicker`.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui, birthday: &mut Option<fluent_egui::Date>) {
/// ui.add(fluent_egui::DatePicker::new("birthday", birthday));
/// # }
/// ```
pub struct DatePicker<'a> {
    id_salt: IdSalt,
    value: &'a mut Option<Date>,
    fields: Vec<Field>,
    start_year: Option<i32>,
    end_year: Option<i32>,
}

impl<'a> DatePicker<'a> {
    pub fn new(id_salt: impl AsIdSalt, value: &'a mut Option<Date>) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            value,
            fields: vec![Field::Month, Field::Day, Field::Year],
            start_year: None,
            end_year: None,
        }
    }

    fn toggle(mut self, field: Field, show: bool) -> Self {
        self.fields.retain(|f| *f != field);
        if show {
            self.fields.push(field);
            self.fields.sort_by_key(|f| *f as u8);
        }
        self
    }

    pub fn show_month(self, show: bool) -> Self {
        self.toggle(Field::Month, show)
    }

    pub fn show_day(self, show: bool) -> Self {
        self.toggle(Field::Day, show)
    }

    pub fn show_year(self, show: bool) -> Self {
        self.toggle(Field::Year, show)
    }

    /// The first year offered, 100 years before today by default.
    pub fn start_year(mut self, year: i32) -> Self {
        self.start_year = Some(year);
        self
    }

    /// The last year offered, 25 years after today by default.
    pub fn end_year(mut self, year: i32) -> Self {
        self.end_year = Some(year);
        self
    }
}

impl Widget for DatePicker<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let id = ui.make_persistent_id(self.id_salt);
        let today = Date::today();
        let start = self.start_year.unwrap_or(today.year - 100);
        let end = self.end_year.unwrap_or(today.year + 25).max(start);
        let mut popup = PickerPopupState::load(ui, id);

        // The month and year the day column is counted for: the popup's
        // working values while open.
        let (mut month, mut year) = match *self.value {
            Some(d) => (d.month, d.year),
            None => (today.month, today.year),
        };
        if popup.open && popup.indices.len() == 3 {
            month = popup.indices[0].rem_euclid(12) as u8 + 1;
            year = start + popup.indices[2].clamp(0, i64::from(end - start)) as i32;
        }
        let l10n = Localizations::of(ui.ctx());
        let names = l10n.date_names();
        let column = |field: Field| match field {
            Field::Month => PickerColumn {
                items: names.months.iter().map(|m| (*m).to_owned()).collect(),
                width: 132.0,
                looping: true,
                align: Align::Min,
                placeholder: l10n.month,
            },
            Field::Day => PickerColumn {
                items: (1..=days_in_month(year, month))
                    .map(|d| d.to_string())
                    .collect(),
                width: 78.0,
                looping: true,
                align: Align::Center,
                placeholder: l10n.day,
            },
            Field::Year => PickerColumn {
                items: (start..=end).map(|y| y.to_string()).collect(),
                width: 78.0,
                looping: false,
                align: Align::Center,
                placeholder: l10n.year,
            },
        };
        let columns: Vec<PickerColumn> = self.fields.iter().map(|f| column(*f)).collect();
        let values = self.value.map(|d| {
            self.fields
                .iter()
                .map(|f| match f {
                    Field::Month => names.month(d.month).to_owned(),
                    Field::Day => d.day.to_string(),
                    Field::Year => d.year.to_string(),
                })
                .collect()
        });
        let label = self
            .value
            .map_or_else(|| l10n.pick_a_date.to_owned(), |d| d.to_string());
        let mut response = picker_button(ui, id, &columns, values, &label, popup.open);

        if response.clicked() && !popup.open {
            let d = self.value.unwrap_or(today);
            popup.open = true;
            popup.indices = vec![
                i64::from(d.month) - 1,
                i64::from(d.day) - 1,
                i64::from(d.year.clamp(start, end) - start),
            ];
        } else if popup.open {
            let slot = |f: &Field| *f as usize;
            let mut indices: Vec<i64> =
                self.fields.iter().map(|f| popup.indices[slot(f)]).collect();
            let action = picker_popup(ui, id, response.rect, &columns, &mut indices);
            for (f, i) in self.fields.iter().zip(indices) {
                popup.indices[slot(f)] = i;
            }
            if let Some(action) = action {
                popup.open = false;
                if action == PopupAction::Accept {
                    let month = popup.indices[0].rem_euclid(12) as u8 + 1;
                    let year = start + popup.indices[2].clamp(0, i64::from(end - start)) as i32;
                    let days = i64::from(days_in_month(year, month));
                    let day = popup.indices[1].rem_euclid(days) as u8 + 1;
                    let picked = Date::new(year, month, day);
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
