use egui::text_edit::TextEditOutput;
use egui::{
    AsIdSalt, CornerRadius, Frame, IdSalt, Margin, Rect, Response, RichText, Sense, Shape, Stroke,
    StrokeKind, TextEdit, Ui, Vec2, Widget, pos2,
};

use crate::FluentTheme;
use crate::utils::{ControlState, paint_eye};

/// fluent_ui's `kTextBoxPadding`.
const PADDING: Margin = Margin {
    left: 10,
    right: 6,
    top: 5,
    bottom: 6,
};

/// A Fluent text field, fluent_ui's `TextBox`, built on [`TextEdit`].
pub struct TextBox<'a> {
    text: &'a mut String,
    placeholder: Option<String>,
    password: bool,
    rows: Option<usize>,
    width: Option<f32>,
    id_salt: Option<IdSalt>,
    /// Extra room on the right for buttons drawn over the field.
    trailing: f32,
}

impl<'a> TextBox<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            placeholder: None,
            password: false,
            rows: None,
            width: None,
            id_salt: None,
            trailing: 0.0,
        }
    }

    /// A multi-line text box showing `rows` lines.
    pub fn multiline(text: &'a mut String, rows: usize) -> Self {
        Self {
            rows: Some(rows.max(1)),
            ..Self::new(text)
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Hides the text. See also [`PasswordBox`].
    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    /// Defaults to the available width, at most 300.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn id_salt(mut self, id_salt: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id_salt));
        self
    }

    pub(crate) fn trailing(mut self, trailing: f32) -> Self {
        self.trailing = trailing;
        self
    }

    /// Shows the text box and returns the full [`TextEditOutput`].
    pub fn show(self, ui: &mut Ui) -> TextEditOutput {
        let theme = FluentTheme::of(ui.ctx());
        let r = &theme.resources;
        let enabled = ui.is_enabled();
        let width = self
            .width
            .unwrap_or_else(|| ui.available_width().min(300.0));
        // Reserve the background slot first so it ends up below the text.
        let background = ui.painter().add(Shape::Noop);

        // Same id TextEdit will use, to style the placeholder for focus.
        // (TextEdit wraps the salt in another IdSalt, so mirror that.)
        let edit_id = self.id_salt.map_or_else(
            || ui.next_auto_id(),
            |s| ui.make_persistent_id(IdSalt::new(s)),
        );
        let focused_before = ui.memory(|m| m.has_focus(edit_id));
        let placeholder_color = if focused_before {
            r.text_fill_color_tertiary
        } else {
            r.text_fill_color_secondary
        };
        let mut edit = match self.rows {
            Some(rows) => TextEdit::multiline(self.text).desired_rows(rows),
            None => TextEdit::singleline(self.text),
        };
        if let Some(id) = self.id_salt {
            edit = edit.id_salt(id);
        }
        if let Some(placeholder) = self.placeholder {
            let color = if enabled {
                placeholder_color
            } else {
                r.text_fill_color_disabled
            };
            edit = edit.hint_text(RichText::new(placeholder).color(color));
        }
        let font = theme.typography.body.font_id();
        let mut margin = PADDING;
        margin.right += self.trailing.round() as i8;
        if self.rows.is_none() {
            // Center a single line in the 32px field; fluent_ui gets this from
            // the 20px line height, egui rows are shorter.
            let row = ui.ctx().fonts_mut(|f| f.row_height(&font));
            let top = ((32.0 - row) / 2.0).floor().max(0.0) as i8;
            margin.top = top;
            margin.bottom = top;
        }
        let output = edit
            // A custom frame makes TextEdit ignore `.margin()`, so the padding
            // goes on the frame.
            .frame(Frame::NONE.inner_margin(margin))
            .password(self.password)
            .font(font)
            .text_color(if enabled {
                r.text_fill_color_primary
            } else {
                r.text_fill_color_disabled
            })
            .desired_width(width - f32::from(margin.left) - f32::from(margin.right))
            .min_size(Vec2::new(width, 32.0))
            .show(ui);

        let response: &Response = &output.response;
        let state = ControlState::from_response(response);
        let rect = response.rect;
        let radius = CornerRadius::same(4);
        let fill = if !enabled {
            r.control_fill_color_disabled
        } else if state.focused || state.pressed {
            r.control_fill_color_input_active
        } else if state.hovered {
            r.control_fill_color_secondary
        } else {
            r.control_fill_color_default
        };
        let mut shapes = vec![
            Shape::rect_filled(rect, radius, fill),
            Shape::rect_stroke(
                rect,
                radius,
                Stroke::new(1.0, r.control_stroke_color_default),
                StrokeKind::Inside,
            ),
        ];
        // The bottom highlight: 2px accent while focused, a thin strong
        // stroke otherwise.
        let underline = if state.focused {
            Some((2.0, theme.accent_color.default_brush_for(theme.brightness)))
        } else if enabled {
            Some((1.25, r.control_strong_stroke_color_default))
        } else {
            None
        };
        if let Some((height, color)) = underline {
            let line = Rect::from_min_max(pos2(rect.left(), rect.bottom() - height), rect.max);
            let bottom = CornerRadius {
                nw: 0,
                ne: 0,
                sw: 4,
                se: 4,
            };
            shapes.push(Shape::rect_filled(
                line.shrink2(Vec2::new(0.5, 0.0)),
                bottom,
                color,
            ));
        }
        ui.painter().set(background, Shape::Vec(shapes));
        output
    }
}

impl Widget for TextBox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response.response
    }
}

/// When a [`PasswordBox`] shows its text, fluent_ui's `PasswordRevealMode`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PasswordRevealMode {
    /// A reveal button shows the text while it is held down.
    #[default]
    Peek,
    /// Always hidden, no reveal button.
    Hidden,
    /// Always shown.
    Visible,
}

/// A password field with a press-to-peek button, fluent_ui's `PasswordBox`.
pub struct PasswordBox<'a> {
    text: &'a mut String,
    placeholder: Option<String>,
    reveal_mode: PasswordRevealMode,
    width: Option<f32>,
    id_salt: Option<IdSalt>,
}

impl<'a> PasswordBox<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            placeholder: None,
            reveal_mode: PasswordRevealMode::default(),
            width: None,
            id_salt: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn reveal_mode(mut self, mode: PasswordRevealMode) -> Self {
        self.reveal_mode = mode;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn id_salt(mut self, id_salt: impl AsIdSalt) -> Self {
        self.id_salt = Some(IdSalt::new(id_salt));
        self
    }
}

impl Widget for PasswordBox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = FluentTheme::of(ui.ctx());
        let peek_id = ui.next_auto_id().with("peek");
        let peeking = ui.data(|d| d.get_temp::<bool>(peek_id)).unwrap_or(false);
        let show_button = self.reveal_mode == PasswordRevealMode::Peek && !self.text.is_empty();
        let hidden = match self.reveal_mode {
            PasswordRevealMode::Visible => false,
            PasswordRevealMode::Hidden => true,
            PasswordRevealMode::Peek => !peeking,
        };

        let mut text_box = TextBox::new(self.text)
            .password(hidden)
            .trailing(if show_button { 28.0 } else { 0.0 });
        if let Some(p) = self.placeholder {
            text_box = text_box.placeholder(p);
        }
        if let Some(w) = self.width {
            text_box = text_box.width(w);
        }
        if let Some(id) = self.id_salt {
            text_box = text_box.id_salt(id);
        }
        let output = text_box.show(ui);
        let response = output.response.response;

        let mut now_peeking = false;
        if show_button && ui.is_enabled() {
            let rect = response.rect;
            let button = Rect::from_center_size(
                pos2(rect.right() - 18.0, rect.center().y),
                Vec2::new(28.0, 24.0),
            );
            let b = ui.interact(button, peek_id.with("button"), Sense::click_and_drag());
            now_peeking = b.is_pointer_button_down_on();
            if b.hovered() || now_peeking {
                let r = &theme.resources;
                let fill = if now_peeking {
                    r.subtle_fill_color_tertiary
                } else {
                    r.subtle_fill_color_secondary
                };
                ui.painter().rect_filled(button, 4.0, fill);
            }
            if now_peeking && !response.has_focus() {
                response.request_focus();
            }
            paint_eye(
                ui.painter(),
                button.center(),
                theme.resources.text_fill_color_secondary,
            );
        }
        if now_peeking != peeking {
            ui.data_mut(|d| d.insert_temp(peek_id, now_peeking));
            ui.ctx().request_repaint();
        }
        response
    }
}
