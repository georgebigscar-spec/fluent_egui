use std::ops::Range;

use egui::{AsIdSalt, IdSalt, Layout, Rect, Response, Sense, Ui, UiBuilder, vec2};

/// What [`DynamicOverflow::show`] returns.
pub struct DynamicOverflowResponse {
    pub response: Response,
    /// How many items fit; the rest went to the overflow widget.
    pub visible: usize,
}

/// A row that shows as many items as fit and hands the rest to an overflow
/// widget at the end, such as a "more" menu. fluent_ui's `DynamicOverflow`.
///
/// Item widths are measured once and remembered, so items should keep their
/// size from frame to frame.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui) {
/// use fluent_egui::{Button, DynamicOverflow, DropDownButton, MenuItem};
///
/// let names = ["Cut", "Copy", "Paste", "Rename", "Share", "Delete"];
/// DynamicOverflow::new("actions").show(
///     ui,
///     names.len(),
///     |ui, i| {
///         ui.add(Button::new(names[i]));
///     },
///     |ui, hidden| {
///         DropDownButton::new("More").show(ui, |ui| {
///             for i in hidden {
///                 ui.add(MenuItem::new(names[i]));
///             }
///         });
///     },
/// );
/// # }
/// ```
pub struct DynamicOverflow {
    id_salt: IdSalt,
    spacing: f32,
    always_show_overflow: bool,
}

#[derive(Clone, Default)]
struct OverflowState {
    widths: Vec<Option<f32>>,
    overflow_width: Option<f32>,
}

impl DynamicOverflow {
    pub fn new(id_salt: impl AsIdSalt) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            spacing: 4.0,
            always_show_overflow: false,
        }
    }

    /// Gap between items, 4 by default.
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Shows the overflow widget even when every item fits, fluent_ui's
    /// `alwaysDisplayOverflowWidget`.
    pub fn always_show_overflow(mut self, always: bool) -> Self {
        self.always_show_overflow = always;
        self
    }

    /// Lays out `count` items with `item`, then calls `overflow` with the
    /// range of items that did not fit (empty when all fit and
    /// [`DynamicOverflow::always_show_overflow`] is set).
    pub fn show(
        self,
        ui: &mut Ui,
        count: usize,
        mut item: impl FnMut(&mut Ui, usize),
        overflow: impl FnOnce(&mut Ui, Range<usize>),
    ) -> DynamicOverflowResponse {
        let id = ui.make_persistent_id(self.id_salt);
        let mut state = ui
            .data(|d| d.get_temp::<OverflowState>(id))
            .unwrap_or_default();
        state.widths.resize(count, None);

        // Measure items we have not seen yet in an invisible pass.
        let origin = ui.cursor().min;
        for i in 0..count {
            if state.widths[i].is_none() {
                let mut probe = ui.new_child(
                    UiBuilder::new()
                        .id_salt(id.with(("probe", i)))
                        .max_rect(Rect::from_min_size(origin, vec2(f32::INFINITY, 1000.0)))
                        .layout(Layout::left_to_right(egui::Align::Center))
                        .invisible(),
                );
                item(&mut probe, i);
                state.widths[i] = Some(probe.min_rect().width());
            }
        }

        let available = ui.available_width();
        let overflow_width = state.overflow_width.unwrap_or(40.0);
        let widths: Vec<f32> = state.widths.iter().map(|w| w.unwrap_or(0.0)).collect();
        let total: f32 = widths.iter().map(|w| w + self.spacing).sum();
        let all_fit = total - self.spacing
            + if self.always_show_overflow {
                overflow_width + self.spacing
            } else {
                0.0
            }
            <= available;
        let visible = if all_fit {
            count
        } else {
            let room = available - overflow_width - self.spacing;
            let mut used = 0.0;
            widths
                .iter()
                .take_while(|w| {
                    used += *w + self.spacing;
                    used - self.spacing <= room
                })
                .count()
        };

        let inner = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = self.spacing;
            for i in 0..visible {
                let before = ui.cursor().min.x;
                item(ui, i);
                // Keep the width current in case the item changed size.
                state.widths[i] = Some(ui.min_rect().right() - before);
            }
            if visible < count || self.always_show_overflow {
                let start = ui.cursor().min.x;
                overflow(ui, visible..count);
                state.overflow_width = Some(ui.min_rect().right() - start);
            }
        });
        ui.data_mut(|d| d.insert_temp(id, state));
        DynamicOverflowResponse {
            response: inner.response.interact(Sense::hover()),
            visible,
        }
    }
}
