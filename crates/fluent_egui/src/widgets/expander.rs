use egui::collapsing_header::CollapsingState;
use egui::{
    AsIdSalt, CornerRadius, Frame, IdSalt, Margin, Pos2, Rect, Response, Role, Sense, Stroke,
    StrokeKind, TextWrapMode, Ui, Vec2, WidgetInfo, WidgetText, pos2, vec2,
};

use crate::FluentTheme;
use crate::utils::{ControlState, States, paint_focus_border};

/// What [`Expander::show`] returns.
pub struct ExpanderResponse<R> {
    /// The header row; `.clicked()` when it was toggled.
    pub header: Response,
    /// What the body closure returned, if the body was shown.
    pub body: Option<R>,
    pub open: bool,
}

/// A header that expands to reveal content, fluent_ui's `Expander`.
pub struct Expander {
    id_salt: IdSalt,
    header: WidgetText,
    initially_open: bool,
}

impl Expander {
    pub fn new(id_salt: impl AsIdSalt, header: impl Into<WidgetText>) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            header: header.into(),
            initially_open: false,
        }
    }

    pub fn initially_open(mut self, open: bool) -> Self {
        self.initially_open = open;
        self
    }

    pub fn show<R>(self, ui: &mut Ui, add_body: impl FnOnce(&mut Ui) -> R) -> ExpanderResponse<R> {
        let theme = FluentTheme::of(ui.ctx());
        let r = &theme.resources;
        let id = ui.make_persistent_id(self.id_salt);
        let mut state = CollapsingState::load_with_default_open(ui.ctx(), id, self.initially_open);
        let openness = state.openness(ui.ctx());

        // Header and body touch; the body restores normal spacing inside.
        let saved_spacing = ui.spacing().item_spacing.y;
        ui.spacing_mut().item_spacing.y = 0.0;
        let width = ui.available_width();
        let label = self.header.text().to_owned();
        let galley = self.header.into_galley(
            ui,
            Some(TextWrapMode::Wrap),
            width - 16.0 - 64.0,
            theme.typography.body.font_id(),
        );
        let height = (galley.size().y + 16.0).max(42.0);
        let (rect, header) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
        if header.clicked() {
            state.toggle(ui);
        }
        let open = state.is_open();
        header.widget_info(|| {
            let mut info = WidgetInfo::labeled(Role::Button, ui.is_enabled(), &label);
            info.selected = Some(open);
            info
        });

        if ui.is_rect_visible(rect) {
            let cs = ControlState::from_response(&header);
            let painter = ui.painter();
            // Square bottom corners while the body is attached below.
            let bottom = if openness > 0.0 { 0 } else { 6 };
            let radius = CornerRadius {
                nw: 6,
                ne: 6,
                sw: bottom,
                se: bottom,
            };
            painter.rect_filled(rect, radius, r.card_background_fill_color_default);
            painter.rect_stroke(
                rect,
                radius,
                Stroke::new(1.0, r.card_stroke_color_default),
                StrokeKind::Inside,
            );
            let text_color = if cs.disabled {
                r.text_fill_color_disabled
            } else {
                r.text_fill_color_primary
            };
            painter.galley(
                pos2(rect.left() + 16.0, rect.center().y - galley.size().y / 2.0),
                galley,
                text_color,
            );

            // The chevron sits in a 28px rounded square that reacts like a
            // subtle button (`uncheckedInputColor` with transparent rest).
            let chevron_box = Rect::from_center_size(
                pos2(rect.right() - 8.0 - 14.0, rect.center().y),
                Vec2::splat(28.0),
            );
            let fill = States::new(
                r.subtle_fill_color_transparent,
                r.subtle_fill_color_secondary,
                r.subtle_fill_color_tertiary,
                r.subtle_fill_color_transparent,
            )
            .resolve(cs);
            painter.rect_filled(chevron_box, 6.0, fill);
            let nudge = if cs.pressed { 1.0 } else { 0.0 };
            paint_flipping_chevron(
                painter,
                chevron_box.center() + vec2(0.0, nudge),
                openness,
                text_color,
            );
            if cs.focused {
                paint_focus_border(painter, rect, &theme);
            }
        }

        let body = state.show_body_unindented(ui, |ui| {
            Frame::new()
                .fill(r.card_background_fill_color_secondary)
                .stroke(Stroke::new(1.0, r.card_stroke_color_default))
                .corner_radius(CornerRadius {
                    nw: 0,
                    ne: 0,
                    sw: 6,
                    se: 6,
                })
                .inner_margin(Margin::same(16))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing.y = saved_spacing;
                    add_body(ui)
                })
                .inner
        });
        ExpanderResponse {
            header,
            body: body.map(|b| b.inner),
            open,
        }
    }
}

/// A chevron that points down when closed and up when open, flattening
/// through the middle of the animation.
fn paint_flipping_chevron(
    painter: &egui::Painter,
    center: Pos2,
    openness: f32,
    color: egui::Color32,
) {
    let dir = 1.0 - 2.0 * openness;
    let points = vec![
        center + vec2(-4.0, -2.0 * dir),
        center + vec2(0.0, 2.0 * dir),
        center + vec2(4.0, -2.0 * dir),
    ];
    painter.add(egui::epaint::PathShape::line(
        points,
        Stroke::new(1.2, color),
    ));
}
