use egui::epaint::{Mesh, QuadraticBezierShape};
use egui::{
    AsIdSalt, Color32, CornerRadius, Frame, Id, IdSalt, Painter, Pos2, Rect, Response, Role,
    ScrollArea, Sense, Stroke, TextWrapMode, Ui, Vec2, WidgetInfo, WidgetText, pos2, vec2,
};

use super::FluentTooltip as _;
use crate::FluentTheme;
use crate::Localizations;
use crate::utils::{ControlState, animate_color, paint_focus_border};

const MIN_TAB_WIDTH: f32 = 80.0;
const MAX_TAB_WIDTH: f32 = 240.0;
const TAB_HEIGHT: f32 = 34.0;
const BUTTON_WIDTH: f32 = 32.0;
/// Radius of the tab corners and of the "ears" joining the selected tab to
/// the content below.
const RADIUS: f32 = 6.0;

/// How wide tabs are, fluent_ui's `TabWidthBehavior`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabWidthBehavior {
    /// All tabs share the width, between 80 and 240px.
    #[default]
    Equal,
    /// Each tab is as wide as its content.
    SizeToContent,
    /// Unselected tabs show only their icon.
    Compact,
}

/// When tabs show their close button, fluent_ui's `CloseButtonVisibilityMode`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CloseButtonVisibility {
    #[default]
    Always,
    OnHover,
    Never,
}

/// One tab of a [`TabView`].
pub struct Tab {
    title: WidgetText,
    icon: Option<WidgetText>,
    closable: bool,
    id: Option<IdSalt>,
}

impl Tab {
    pub fn new(title: impl Into<WidgetText>) -> Self {
        Self {
            title: title.into(),
            icon: None,
            closable: true,
            id: None,
        }
    }

    /// A 16px icon before the title.
    pub fn icon(mut self, icon: impl Into<WidgetText>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Identifies the tab across frames. Needed for dragging tabs to
    /// reorder them, since their positions change; defaults to the position.
    pub fn id_salt(mut self, salt: impl AsIdSalt) -> Self {
        self.id = Some(IdSalt::new(salt));
        self
    }
}

/// What [`TabView::show`] returns. Apply `closed`, `added` and `moved` to
/// your list of tabs; the current index is already updated.
pub struct TabViewResponse<R> {
    /// What the body closure returned.
    pub inner: R,
    /// The tab whose close button (or middle click) asked to close it.
    pub closed: Option<usize>,
    /// The add button was clicked.
    pub added: bool,
    /// A tab was dragged from the first index to the second.
    pub moved: Option<(usize, usize)>,
    /// The selection changed this frame.
    pub changed: bool,
}

/// A strip of tabs over a content area, fluent_ui's `TabView`.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui, docs: &mut Vec<String>, current: &mut usize) {
/// use fluent_egui::{Tab, TabView};
///
/// let tabs = docs.iter().map(|d| Tab::new(d.as_str()));
/// let response = TabView::new("docs").show(ui, current, tabs, |ui, i| {
///     ui.label(format!("Document {i}"));
/// });
/// if let Some(i) = response.closed {
///     docs.remove(i);
/// }
/// if response.added {
///     docs.push("New document".to_owned());
/// }
/// # }
/// ```
pub struct TabView {
    id_salt: IdSalt,
    width_behavior: TabWidthBehavior,
    close_button: CloseButtonVisibility,
    add_button: bool,
    reorder: bool,
    min_body_height: f32,
}

impl TabView {
    pub fn new(id_salt: impl AsIdSalt) -> Self {
        Self {
            id_salt: IdSalt::new(id_salt),
            width_behavior: TabWidthBehavior::default(),
            close_button: CloseButtonVisibility::default(),
            add_button: true,
            reorder: true,
            min_body_height: 0.0,
        }
    }

    pub fn tab_width(mut self, behavior: TabWidthBehavior) -> Self {
        self.width_behavior = behavior;
        self
    }

    pub fn close_button(mut self, visibility: CloseButtonVisibility) -> Self {
        self.close_button = visibility;
        self
    }

    /// Shows the "+" button after the tabs. On by default.
    pub fn add_button(mut self, show: bool) -> Self {
        self.add_button = show;
        self
    }

    /// Lets tabs be dragged to reorder them. On by default.
    pub fn reorderable(mut self, reorder: bool) -> Self {
        self.reorder = reorder;
        self
    }

    pub fn min_body_height(mut self, height: f32) -> Self {
        self.min_body_height = height;
        self
    }

    pub fn show<R>(
        self,
        ui: &mut Ui,
        current: &mut usize,
        tabs: impl IntoIterator<Item = Tab>,
        add_body: impl FnOnce(&mut Ui, usize) -> R,
    ) -> TabViewResponse<R> {
        let theme = FluentTheme::of(ui.ctx());
        let r = &theme.resources;
        let id = ui.make_persistent_id(self.id_salt);
        let tabs: Vec<Tab> = tabs.into_iter().collect();
        let before = *current;
        if !tabs.is_empty() {
            *current = (*current).min(tabs.len() - 1);
        }

        let mut closed = None;
        let mut added = false;
        let mut moved = None;
        let saved_spacing = ui.spacing().item_spacing;
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        ui.add_space(4.5);

        let width = ui.available_width();
        let strip_width = width
            - 8.0
            - if self.add_button {
                BUTTON_WIDTH + 4.0
            } else {
                0.0
            };
        let equal_width =
            (strip_width / tabs.len().max(1) as f32 - 1.0).clamp(MIN_TAB_WIDTH, MAX_TAB_WIDTH);

        let mut selected_span = None;
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
            ui.set_height(TAB_HEIGHT);
            ui.add_space(8.0);
            let scroll = ScrollArea::horizontal()
                .id_salt(id.with("strip"))
                .max_width(strip_width)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        let mut rects = Vec::with_capacity(tabs.len());
                        let mut dragged = None;
                        for (i, tab) in tabs.iter().enumerate() {
                            let selected = i == *current;
                            let tab_id = match tab.id {
                                Some(salt) => id.with(salt),
                                None => id.with(i),
                            };
                            let out = self.tab(ui, tab_id, tab, selected, equal_width);
                            if out.response.clicked() || out.response.drag_started() {
                                *current = i;
                            }
                            if out.close || out.response.middle_clicked() {
                                closed = Some(i);
                            }
                            if self.reorder && out.response.dragged() {
                                dragged = Some(i);
                            }
                            rects.push(out.response.rect);
                            if selected {
                                selected_span = Some(
                                    out.response
                                        .rect
                                        .x_range()
                                        .intersection(ui.clip_rect().x_range()),
                                );
                            }
                            // The divider is hidden next to the selected tab.
                            let (div, _) =
                                ui.allocate_exact_size(vec2(1.0, TAB_HEIGHT), Sense::hover());
                            if i + 1 < tabs.len() && !selected && i + 1 != *current {
                                ui.painter().vline(
                                    div.center().x,
                                    div.y_range().shrink(8.0),
                                    Stroke::new(1.0, theme.divider.color),
                                );
                            }
                        }
                        // Dragging a tab over a neighbor swaps them.
                        if let Some(from) = dragged
                            && let Some(pointer) = ui.ctx().pointer_interact_pos()
                            && let Some(to) =
                                rects.iter().position(|r| r.x_range().contains(pointer.x))
                            && to != from
                        {
                            moved = Some((from, to));
                            *current = to;
                        }
                    });
                });
            if self.add_button {
                ui.add_space(4.0);
                let add = add_button(ui).fluent_tooltip(Localizations::of(ui.ctx()).new_tab_label);
                if add.clicked() {
                    added = true;
                }
            }
            scroll
        });

        // The content area, the same color as the selected tab.
        let body = Frame::new()
            .fill(r.solid_background_fill_color_tertiary)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = saved_spacing;
                ui.set_width(width);
                ui.set_min_height(self.min_body_height);
                add_body(ui, *current)
            });
        // Its outline, open where the selected tab joins it.
        let rect = body.response.rect.shrink(0.5);
        let stroke = Stroke::new(1.0, r.card_stroke_color_default);
        let painter = ui.painter();
        painter.vline(rect.left(), rect.y_range(), stroke);
        painter.vline(rect.right(), rect.y_range(), stroke);
        painter.hline(rect.x_range(), rect.bottom(), stroke);
        match selected_span {
            Some(span) if span.span() > 0.0 => {
                let left = (span.min - RADIUS).max(rect.left());
                let right = (span.max + RADIUS).min(rect.right());
                painter.hline(rect.left()..=left, rect.top(), stroke);
                painter.hline(right..=rect.right(), rect.top(), stroke);
            }
            _ => {
                painter.hline(rect.x_range(), rect.top(), stroke);
            }
        }
        let inner = body.inner;
        ui.spacing_mut().item_spacing = saved_spacing;

        TabViewResponse {
            inner,
            closed,
            added,
            moved,
            changed: *current != before,
        }
    }

    fn tab(&self, ui: &mut Ui, id: Id, tab: &Tab, selected: bool, equal_width: f32) -> TabOutput {
        let theme = FluentTheme::of(ui.ctx());
        let r = &theme.resources;
        let font = egui::FontId::new(12.0, theme.typography.body.font_id().family);
        let icon = tab.icon.clone().map(|i| {
            i.into_galley(
                ui,
                Some(TextWrapMode::Extend),
                f32::INFINITY,
                egui::FontId::proportional(16.0),
            )
        });
        let show_title = self.width_behavior != TabWidthBehavior::Compact || selected;
        let hovered_before = ui.ctx().read_response(id).is_some_and(|r| r.hovered());
        let show_close = tab.closable
            && match self.close_button {
                CloseButtonVisibility::Always => true,
                CloseButtonVisibility::OnHover => hovered_before || selected,
                CloseButtonVisibility::Never => false,
            };
        // Padding 8/4 (selected 9/5), icon then 10, title then 4, close 4+32.
        let icon_width = icon.as_ref().map_or(0.0, |_| 16.0 + 10.0);
        let close_width = if show_close { 4.0 + BUTTON_WIDTH } else { 0.0 };
        let chrome = 8.0 + 4.0 + icon_width + close_width;
        let title_galley = |ui: &Ui, max: f32| {
            tab.title
                .clone()
                .into_galley(ui, Some(TextWrapMode::Truncate), max, font.clone())
        };
        let width = match self.width_behavior {
            TabWidthBehavior::Equal => equal_width,
            _ if show_title => {
                let natural = title_galley(ui, f32::INFINITY).size().x + 4.0;
                (chrome + natural).min(MAX_TAB_WIDTH)
            }
            _ => chrome,
        };
        let (rect, _) = ui.allocate_exact_size(vec2(width, TAB_HEIGHT), Sense::hover());
        let sense = if self.reorder {
            Sense::click_and_drag()
        } else {
            Sense::click()
        };
        let response = ui.interact(rect, id, sense);
        let label = tab.title.text().to_owned();
        response.widget_info(|| {
            let mut info = WidgetInfo::labeled(Role::Tab, true, &label);
            info.selected = Some(selected);
            info
        });

        let state = ControlState::from_response(&response);
        let close_rect = Rect::from_min_size(
            pos2(
                rect.right() - 4.0 - BUTTON_WIDTH + if selected { -1.0 } else { 0.0 },
                rect.center().y - 12.0,
            ),
            vec2(BUTTON_WIDTH, 24.0),
        );
        let close = show_close.then(|| {
            let close = ui
                .interact(close_rect, id.with("close"), Sense::click())
                .fluent_tooltip(Localizations::of(ui.ctx()).close_tab_label_suffix);
            let cs = ControlState::from_response(&close);
            let fill = animate_color(
                ui.ctx(),
                close.id.with("fill"),
                cs.pick(
                    r.subtle_fill_color_transparent,
                    r.subtle_fill_color_secondary,
                    r.subtle_fill_color_tertiary,
                    r.subtle_fill_color_transparent,
                ),
                theme.faster_animation_duration,
            );
            (close.clicked(), fill)
        });

        if ui.is_rect_visible(rect) {
            let foreground = if selected {
                r.text_fill_color_primary
            } else {
                state.pick(
                    r.text_fill_color_secondary,
                    r.text_fill_color_primary,
                    r.text_fill_color_secondary,
                    r.text_fill_color_disabled,
                )
            };
            let painter = ui.painter();
            if selected {
                paint_selected_tab(
                    painter,
                    rect,
                    r.solid_background_fill_color_tertiary,
                    Stroke::new(1.0, r.card_stroke_color_default),
                );
            } else {
                let fill = state.pick(
                    r.layer_on_mica_base_alt_fill_color_transparent,
                    r.layer_on_mica_base_alt_fill_color_secondary,
                    r.layer_on_mica_base_alt_fill_color_default,
                    r.layer_on_mica_base_alt_fill_color_transparent,
                );
                let fill = animate_color(
                    ui.ctx(),
                    id.with("fill"),
                    fill,
                    theme.faster_animation_duration,
                );
                painter.rect_filled(
                    rect,
                    CornerRadius {
                        nw: RADIUS as u8,
                        ne: RADIUS as u8,
                        ..Default::default()
                    },
                    fill,
                );
            }
            let content = rect;
            let mut x = content.left() + if selected { 9.0 } else { 8.0 };
            if let Some(icon) = icon {
                painter.galley(
                    pos2(
                        x + 8.0 - icon.size().x / 2.0,
                        content.center().y - icon.size().y / 2.0,
                    ),
                    icon,
                    foreground,
                );
                x += icon_width;
            }
            if show_title {
                let max = (content.right() - x - 4.0 - close_width).max(0.0);
                let galley = title_galley(ui, max);
                painter.galley(
                    pos2(x, content.center().y - galley.size().y / 2.0),
                    galley,
                    foreground,
                );
            }
            if let Some((_, fill)) = close {
                painter.rect_filled(close_rect, 4.0, fill);
                paint_close_glyph(painter, close_rect.center(), foreground);
            }
            if state.focused {
                paint_focus_border(painter, rect, &theme);
            }
        }
        TabOutput {
            response,
            close: close.is_some_and(|(clicked, _)| clicked),
        }
    }
}

struct TabOutput {
    response: Response,
    close: bool,
}

/// A smaller `chrome_close` for tabs.
fn paint_close_glyph(painter: &Painter, center: Pos2, color: Color32) {
    let stroke = Stroke::new(1.0, color);
    let d = 3.5;
    painter.line_segment([center + vec2(-d, -d), center + vec2(d, d)], stroke);
    painter.line_segment([center + vec2(-d, d), center + vec2(d, -d)], stroke);
}

/// The selected tab: rounded on top, with concave "ears" at the bottom
/// corners flowing into the content area, fluent_ui's `_TabPainter`.
fn paint_selected_tab(painter: &Painter, rect: Rect, color: Color32, stroke: Stroke) {
    painter.rect_filled(
        rect,
        CornerRadius {
            nw: RADIUS as u8,
            ne: RADIUS as u8,
            ..Default::default()
        },
        color,
    );
    let curve = |points: [Pos2; 3]| {
        QuadraticBezierShape::from_points_stroke(points, false, Color32::TRANSPARENT, Stroke::NONE)
            .flatten(Some(0.1))
    };
    let (l, r, t, b) = (rect.left(), rect.right(), rect.top(), rect.bottom());
    let left_ear = curve([pos2(l - RADIUS, b), pos2(l, b), pos2(l, b - RADIUS)]);
    let right_ear = curve([pos2(r, b - RADIUS), pos2(r, b), pos2(r + RADIUS, b)]);
    for (ear, corner) in [
        (&left_ear, rect.left_bottom()),
        (&right_ear, rect.right_bottom()),
    ] {
        // Fan out from the corner, which sees the whole curve.
        let mut mesh = Mesh::default();
        mesh.colored_vertex(corner, color);
        for p in ear {
            mesh.colored_vertex(*p, color);
        }
        for i in 1..ear.len() as u32 {
            mesh.add_triangle(0, i, i + 1);
        }
        painter.add(mesh);
    }

    // The outline: left ear, up, across the rounded top, down, right ear.
    let half = stroke.width / 2.0;
    let mut outline = left_ear.clone();
    outline.extend(curve([
        pos2(l, t + RADIUS),
        pos2(l, t),
        pos2(l + RADIUS, t),
    ]));
    outline.extend(curve([
        pos2(r - RADIUS, t),
        pos2(r, t),
        pos2(r, t + RADIUS),
    ]));
    outline.extend(right_ear);
    for p in &mut outline {
        // Keep the 1px line on whole pixels, inside the tab.
        p.x += if p.x < rect.center().x { half } else { -half };
        p.y += half;
    }
    painter.add(egui::epaint::PathShape::line(outline, stroke));
}

/// The "+" button after the tabs.
fn add_button(ui: &mut Ui) -> Response {
    let theme = FluentTheme::of(ui.ctx());
    let r = &theme.resources;
    let (slot, _) = ui.allocate_exact_size(vec2(BUTTON_WIDTH, TAB_HEIGHT), Sense::hover());
    let rect = Rect::from_center_size(slot.center(), vec2(BUTTON_WIDTH, 28.0));
    let response = ui.interact(rect, slot_id(ui, slot), Sense::click());
    response.widget_info(|| {
        WidgetInfo::labeled(
            Role::Button,
            true,
            Localizations::of(ui.ctx()).new_tab_label,
        )
    });
    let state = ControlState::from_response(&response);
    let fill = animate_color(
        ui.ctx(),
        response.id.with("fill"),
        state.pick(
            r.subtle_fill_color_transparent,
            r.subtle_fill_color_secondary,
            r.subtle_fill_color_tertiary,
            r.subtle_fill_color_transparent,
        ),
        theme.faster_animation_duration,
    );
    let painter = ui.painter();
    painter.rect_filled(rect, 4.0, fill);
    let c = rect.center();
    let stroke = Stroke::new(1.2, theme.inactive_color);
    painter.line_segment([c + vec2(-5.0, 0.0), c + vec2(5.0, 0.0)], stroke);
    painter.line_segment([c + vec2(0.0, -5.0), c + vec2(0.0, 5.0)], stroke);
    if state.focused {
        paint_focus_border(painter, rect, &theme);
    }
    response
}

fn slot_id(ui: &Ui, slot: Rect) -> Id {
    ui.scope_id()
        .with(("fluent_tab_add", slot.left().to_bits()))
}
