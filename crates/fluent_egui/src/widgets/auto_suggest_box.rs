use egui::{
    AsIdSalt, IdSalt, Key, Popup, PopupCloseBehavior, Rect, Response, Stroke, Ui, pos2, vec2,
};

use super::{MenuItem, TextBox, flyout_frame};
use crate::FluentTheme;
use crate::Localizations;

/// What [`AutoSuggestBox::show`] returns.
pub struct AutoSuggestBoxResponse {
    /// The text box.
    pub response: Response,
    /// The index (into the items) of the suggestion chosen this frame, by
    /// click or Enter. Its text is already in the box.
    pub chosen: Option<usize>,
}

/// A text box that suggests matching items as you type, fluent_ui's
/// `AutoSuggestBox`. Matching is a case-insensitive "contains".
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui, query: &mut String) {
/// let fruits = ["Apple", "Banana", "Cherry"];
/// let out = fluent_egui::AutoSuggestBox::new("fruit", query, &fruits).show(ui);
/// if let Some(i) = out.chosen {
///     println!("picked {}", fruits[i]);
/// }
/// # }
/// ```
pub struct AutoSuggestBox<'a, S: AsRef<str>> {
    id_salt: IdSalt,
    text: &'a mut String,
    items: &'a [S],
    placeholder: Option<String>,
    width: Option<f32>,
}

#[derive(Clone, Copy, Default)]
struct SuggestState {
    /// The keyboard-highlighted row among the matches.
    highlighted: Option<usize>,
    /// Where the list was last frame, so a click on it keeps it open while
    /// the text box loses focus.
    list: Option<Rect>,
    /// The text box's widget id, learned on the first frame.
    edit_id: Option<egui::Id>,
}

impl<'a, S: AsRef<str>> AutoSuggestBox<'a, S> {
    pub fn new(id_salt: impl AsIdSalt, text: &'a mut String, items: &'a [S]) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            text,
            items,
            placeholder: None,
            width: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn show(self, ui: &mut Ui) -> AutoSuggestBoxResponse {
        let theme = FluentTheme::of(ui.ctx());
        let id = ui.make_persistent_id(self.id_salt);
        let edit_salt = id.with("edit");
        let mut state = ui
            .data(|d| d.get_temp::<SuggestState>(id))
            .unwrap_or_default();

        let query = self.text.to_lowercase();
        let matches: Vec<usize> = (0..self.items.len())
            .filter(|&i| self.items[i].as_ref().to_lowercase().contains(&query))
            .collect();

        // Arrow keys move through the list instead of the text cursor.
        let focused = state
            .edit_id
            .is_some_and(|edit_id| ui.memory(|m| m.has_focus(edit_id)));
        let mut enter = false;
        if focused {
            ui.input_mut(|i| {
                let n = matches.len();
                if n > 0 && i.consume_key(egui::Modifiers::NONE, Key::ArrowDown) {
                    state.highlighted = Some(state.highlighted.map_or(0, |h| (h + 1) % n));
                }
                if n > 0 && i.consume_key(egui::Modifiers::NONE, Key::ArrowUp) {
                    state.highlighted = Some(state.highlighted.map_or(n - 1, |h| (h + n - 1) % n));
                }
                enter = i.key_pressed(Key::Enter);
            });
        }

        let mut text_box = TextBox::new(self.text).id_salt(edit_salt).trailing(28.0);
        if let Some(p) = &self.placeholder {
            text_box = text_box.placeholder(p.clone());
        }
        if let Some(w) = self.width {
            text_box = text_box.width(w);
        }
        let output = text_box.show(ui);
        let mut response = output.response.response;
        state.edit_id = Some(response.id);
        paint_search_glyph(
            ui.painter(),
            pos2(response.rect.right() - 18.0, response.rect.center().y),
            theme.resources.text_fill_color_secondary,
        );
        if response.changed() {
            state.highlighted = None;
        }

        let mut chosen = None;
        if enter && let Some(h) = state.highlighted.and_then(|h| matches.get(h)) {
            chosen = Some(*h);
        }

        let pointer_on_list = state
            .list
            .zip(ui.ctx().pointer_latest_pos())
            .is_some_and(|(list, p)| list.contains(p));
        let open =
            (response.has_focus() || pointer_on_list) && !self.text.is_empty() && chosen.is_none();
        state.list = None;
        let popup = Popup::from_response(&response)
            .id(id.with("popup"))
            .open(open)
            .close_behavior(PopupCloseBehavior::IgnoreClicks)
            .frame(flyout_frame(&theme))
            .gap(2.0)
            .width(response.rect.width())
            .show(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                if matches.is_empty() {
                    ui.add_enabled(
                        false,
                        MenuItem::new(Localizations::of(ui.ctx()).no_results_found_label),
                    );
                }
                for (row, &i) in matches.iter().enumerate().take(12) {
                    let item = MenuItem::new(self.items[i].as_ref())
                        .selected(state.highlighted == Some(row));
                    if ui.add(item).clicked() {
                        chosen = Some(i);
                    }
                }
            });
        if let Some(popup) = popup {
            state.list = Some(popup.response.rect);
        }

        if let Some(i) = chosen {
            *self.text = self.items[i].as_ref().to_owned();
            state.highlighted = None;
            state.list = None;
            response.mark_changed();
        }
        ui.data_mut(|d| d.insert_temp(id, state));
        AutoSuggestBoxResponse { response, chosen }
    }
}

/// The `search` magnifier glyph, 12px.
fn paint_search_glyph(painter: &egui::Painter, center: egui::Pos2, color: egui::Color32) {
    let stroke = Stroke::new(1.2, color);
    let lens = center + vec2(1.0, -1.0);
    painter.circle_stroke(lens, 4.5, stroke);
    painter.line_segment([lens + vec2(-3.2, 3.2), lens + vec2(-6.5, 6.5)], stroke);
}
