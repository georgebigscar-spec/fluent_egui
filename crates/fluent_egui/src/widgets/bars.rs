use egui::{
    Align, Id, Layout, Margin, Popup, PopupCloseBehavior, Response, Sense, Stroke, TextWrapMode,
    Ui, Vec2, WidgetText, pos2,
};

use super::{Button, flyout_frame, glyph_button};
use crate::FluentTheme;
use crate::Localizations;
use crate::utils::{ControlState, States, animate_color, paint_close, paint_focus_border};

/// A transparent text button for bars: subtle fill on hover and press.
fn bar_button(
    ui: &mut Ui,
    icon: Option<WidgetText>,
    text: Option<WidgetText>,
    open: bool,
) -> Response {
    let theme = FluentTheme::of(ui.ctx());
    let r = &theme.resources;
    let font = theme.typography.body.font_id();
    let layout = |t: WidgetText, size: f32| {
        t.into_galley(
            ui,
            Some(TextWrapMode::Extend),
            f32::INFINITY,
            egui::FontId::new(size, font.family.clone()),
        )
    };
    let icon = icon.map(|i| layout(i, 16.0));
    let text = text.map(|t| layout(t, font.size));
    let gap = if icon.is_some() && text.is_some() {
        8.0
    } else {
        0.0
    };
    let content = Vec2::new(
        icon.as_ref().map_or(0.0, |g| g.size().x) + gap + text.as_ref().map_or(0.0, |g| g.size().x),
        icon.as_ref()
            .map_or(0.0, |g| g.size().y)
            .max(text.as_ref().map_or(0.0, |g| g.size().y)),
    );
    let padding = Margin {
        left: 10,
        right: 10,
        top: 6,
        bottom: 6,
    };
    let size = (content + padding.sum()).max(Vec2::new(32.0, 32.0));
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if ui.is_rect_visible(rect) {
        let mut state = ControlState::from_response(&response);
        state.pressed |= open;
        let fill = States::new(
            r.subtle_fill_color_transparent,
            r.subtle_fill_color_secondary,
            r.subtle_fill_color_tertiary,
            r.subtle_fill_color_transparent,
        )
        .resolve(state);
        let fill = animate_color(
            ui.ctx(),
            response.id.with("fill"),
            fill,
            theme.faster_animation_duration,
        );
        let painter = ui.painter();
        painter.rect_filled(rect, 4.0, fill);
        let color = if state.disabled {
            r.text_fill_color_disabled
        } else {
            r.text_fill_color_primary
        };
        let mut x = rect.center().x - content.x / 2.0;
        if let Some(icon) = icon {
            let w = icon.size().x;
            painter.galley(pos2(x, rect.center().y - icon.size().y / 2.0), icon, color);
            x += w + gap;
        }
        if let Some(text) = text {
            painter.galley(pos2(x, rect.center().y - text.size().y / 2.0), text, color);
        }
        if state.focused {
            paint_focus_border(painter, rect, &theme);
        }
    }
    response
}

/// A row of drop-down menus, fluent_ui's `MenuBar`.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui) {
/// use fluent_egui::{MenuBar, MenuItem};
///
/// MenuBar::new().show(ui, |bar| {
///     bar.menu("File", |ui| {
///         if ui.add(MenuItem::new("Open").shortcut("Ctrl+O")).clicked() { /* ... */ }
///     });
///     bar.menu("Edit", |ui| { ui.add(MenuItem::new("Undo")); });
/// });
/// # }
/// ```
#[derive(Default)]
pub struct MenuBar;

/// Adds menus to a [`MenuBar`].
pub struct MenuBarUi<'a> {
    ui: &'a mut Ui,
    bar_id: Id,
}

impl MenuBar {
    pub fn new() -> Self {
        Self
    }

    pub fn show<R>(self, ui: &mut Ui, add_menus: impl FnOnce(&mut MenuBarUi<'_>) -> R) -> R {
        let bar_id = ui.next_auto_id().with("fluent_menu_bar");
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            add_menus(&mut MenuBarUi { ui, bar_id })
        })
        .inner
    }
}

impl MenuBarUi<'_> {
    /// A top-level menu titled `title`. Returns what `add_contents` returned
    /// while the menu was open.
    pub fn menu<R>(
        &mut self,
        title: impl Into<WidgetText>,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<R> {
        let ctx = self.ui.ctx().clone();
        let theme = FluentTheme::of(&ctx);
        let title_id = self.ui.next_auto_id();
        // Same as `Popup::default_response_id` for the title button below.
        let popup_id = title_id.with("popup");
        let open_now = Popup::is_id_open(&ctx, popup_id);
        let response = bar_button(self.ui, None, Some(title.into()), open_now);

        // Like a desktop menu bar: while one menu is open, hovering another
        // title switches to it.
        let open_in_bar: Option<Id> = ctx.data(|d| d.get_temp(self.bar_id));
        if let Some(other) = open_in_bar
            && other != popup_id
            && Popup::is_id_open(&ctx, other)
            && response.hovered()
        {
            Popup::open_id(&ctx, popup_id);
        }
        let inner = Popup::menu(&response)
            .frame(flyout_frame(&theme))
            .width(200.0)
            .gap(2.0)
            .show(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                add_contents(ui)
            });
        if inner.is_some() {
            ctx.data_mut(|d| d.insert_temp(self.bar_id, popup_id));
        }
        inner.map(|r| r.inner)
    }
}

/// A toolbar of commands, fluent_ui's `CommandBar`, with an optional "more"
/// menu for secondary commands.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui) {
/// use fluent_egui::{CommandBar, MenuItem};
///
/// CommandBar::new().show_with_overflow(
///     ui,
///     |bar| {
///         if bar.button("+", "New").clicked() { /* ... */ }
///         bar.separator();
///         bar.button("✏", "Edit");
///     },
///     |ui| { ui.add(MenuItem::new("Settings")); },
/// );
/// # }
/// ```
pub struct CommandBar {}

/// Adds commands to a [`CommandBar`].
pub struct CommandBarUi<'a> {
    ui: &'a mut Ui,
}

impl Default for CommandBar {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandBar {
    pub fn new() -> Self {
        Self {}
    }

    /// A bar with only primary commands.
    pub fn show<R>(self, ui: &mut Ui, add_primary: impl FnOnce(&mut CommandBarUi<'_>) -> R) -> R {
        self.show_impl(ui, add_primary, None::<fn(&mut Ui)>)
    }

    /// Primary commands plus a "…" button opening a menu of secondary ones.
    pub fn show_with_overflow<R>(
        self,
        ui: &mut Ui,
        add_primary: impl FnOnce(&mut CommandBarUi<'_>) -> R,
        add_secondary: impl FnOnce(&mut Ui),
    ) -> R {
        self.show_impl(ui, add_primary, Some(add_secondary))
    }

    fn show_impl<R>(
        self,
        ui: &mut Ui,
        add_primary: impl FnOnce(&mut CommandBarUi<'_>) -> R,
        add_secondary: Option<impl FnOnce(&mut Ui)>,
    ) -> R {
        let theme = FluentTheme::of(ui.ctx());
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            let r = add_primary(&mut CommandBarUi { ui });
            if let Some(add_secondary) = add_secondary {
                let more = bar_button(ui, Some("…".into()), None, false)
                    .on_hover_text(Localizations::of(ui.ctx()).see_more);
                Popup::menu(&more)
                    .frame(flyout_frame(&theme))
                    .width(200.0)
                    .gap(2.0)
                    .show(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        add_secondary(ui);
                    });
            }
            r
        })
        .inner
    }
}

impl CommandBarUi<'_> {
    /// A command with an icon and a label (`CommandBarButton`).
    pub fn button(
        &mut self,
        icon: impl Into<WidgetText>,
        label: impl Into<WidgetText>,
    ) -> Response {
        bar_button(self.ui, Some(icon.into()), Some(label.into()), false)
    }

    /// An icon-only command; give it a tooltip.
    pub fn icon_button(&mut self, icon: impl Into<WidgetText>) -> Response {
        bar_button(self.ui, Some(icon.into()), None, false)
    }

    /// A vertical divider between groups (`CommandBarSeparator`).
    pub fn separator(&mut self) {
        let theme = FluentTheme::of(self.ui.ctx());
        let (rect, _) = self
            .ui
            .allocate_exact_size(Vec2::new(9.0, 32.0), Sense::hover());
        self.ui.painter().vline(
            rect.center().x,
            (rect.top() + 6.0)..=(rect.bottom() - 6.0),
            Stroke::new(1.0, theme.resources.divider_stroke_color_default),
        );
    }

    /// The underlying [`Ui`], for adding other widgets.
    pub fn ui(&mut self) -> &mut Ui {
        self.ui
    }
}

/// What [`TeachingTip::show`] returns.
#[derive(Clone, Copy, Debug, Default)]
pub struct TeachingTipResponse {
    pub action_clicked: bool,
}

/// A rich tip pointing at a widget, fluent_ui's `TeachingTip`. Stays open
/// until closed with its X button, the action, or by setting `open = false`.
pub struct TeachingTip {
    title: WidgetText,
    subtitle: Option<WidgetText>,
    action: Option<WidgetText>,
    width: f32,
}

impl TeachingTip {
    pub fn new(title: impl Into<WidgetText>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            action: None,
            width: 320.0,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<WidgetText>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// A filled button at the bottom; clicking it also closes the tip.
    pub fn action(mut self, text: impl Into<WidgetText>) -> Self {
        self.action = Some(text.into());
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn show(self, target: &Response, open: &mut bool) -> TeachingTipResponse {
        let mut out = TeachingTipResponse::default();
        if !*open {
            return out;
        }
        let theme = FluentTheme::of(&target.ctx);
        let mut close = false;
        let popup = Popup::from_response(target)
            .id(target.id.with("fluent_teaching_tip"))
            .open(true)
            .close_behavior(PopupCloseBehavior::IgnoreClicks)
            .frame(flyout_frame(&theme).inner_margin(12))
            .width(self.width)
            .gap(10.0)
            .show(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(self.width - 40.0);
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.add(
                            egui::Label::new(theme.typography.body_strong.rich(self.title.text()))
                                .wrap(),
                        );
                        if let Some(subtitle) = &self.subtitle {
                            ui.add(egui::Label::new(subtitle.clone()).wrap());
                        }
                    });
                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        close |= glyph_button(ui, 28.0, paint_close)
                            .on_hover_text(Localizations::of(ui.ctx()).close_button_label)
                            .clicked();
                    });
                });
                if let Some(action) = &self.action {
                    ui.add_space(12.0);
                    let width = ui.available_width();
                    if ui
                        .add(Button::filled(action.clone()).min_size(Vec2::new(width, 32.0)))
                        .clicked()
                    {
                        out.action_clicked = true;
                        close = true;
                    }
                }
            });
        if let Some(popup) = popup {
            paint_tail(
                &target.ctx,
                popup.response.layer_id,
                popup.response.rect,
                target.rect,
                &theme,
            );
        }
        if close {
            *open = false;
        }
        out
    }
}

/// The tail of a [`TeachingTip`], a triangle in the gap pointing at the
/// target. Only drawn when the tip sits right above or below it.
fn paint_tail(
    ctx: &egui::Context,
    layer: egui::LayerId,
    tip: egui::Rect,
    target: egui::Rect,
    theme: &FluentTheme,
) {
    const HALF_WIDTH: f32 = 12.0;
    const HEIGHT: f32 = 10.0;
    let (base_y, apex_y) = if tip.top() >= target.bottom() {
        (tip.top() + 0.5, tip.top() - HEIGHT)
    } else if tip.bottom() <= target.top() {
        (tip.bottom() - 0.5, tip.bottom() + HEIGHT)
    } else {
        return;
    };
    let margin = 8.0 + HALF_WIDTH;
    if tip.width() < 2.0 * margin {
        return;
    }
    let x = target
        .center()
        .x
        .clamp(tip.left() + margin, tip.right() - margin);
    let points = vec![
        pos2(x - HALF_WIDTH, base_y),
        pos2(x, apex_y),
        pos2(x + HALF_WIDTH, base_y),
    ];
    let painter = ctx.layer_painter(layer);
    let stroke = Stroke::new(1.0, theme.resources.surface_stroke_color_default);
    painter.add(egui::Shape::convex_polygon(
        points.clone(),
        theme.menu_color,
        Stroke::NONE,
    ));
    painter.add(egui::Shape::line(points, stroke));
}
