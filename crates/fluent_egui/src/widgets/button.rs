use egui::{
    Color32, CornerRadius, Margin, Rect, Response, Role, Sense, TextWrapMode, Ui, Vec2, Widget,
    WidgetInfo, WidgetText,
};

use crate::utils::{
    ControlState, States, animate_color, paint_elevation_border, paint_focus_border, with_alpha,
};
use crate::{Brightness, FluentTheme};

/// Which fluent_ui button a [`Button`] looks like.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonKind {
    /// `Button`: neutral fill with an elevation border.
    Standard,
    /// `FilledButton`: accent fill, for the primary action.
    Filled,
    /// `OutlinedButton`: transparent with a solid outline.
    Outlined,
    /// `HyperlinkButton`: accent text without a border.
    Hyperlink,
    /// `IconButton`: transparent until hovered.
    Icon(IconButtonMode),
}

/// Size of an icon button, fluent_ui's `IconButtonMode`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconButtonMode {
    /// 11px icon with the standard button padding.
    Tiny,
    /// 14px icon with the standard button padding.
    Small,
    /// 14px icon with 8px padding all around.
    #[default]
    Large,
}

/// How the border of a button is drawn in one state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonBorder {
    None,
    Solid(Color32),
    /// Raised look: `top` around, `bottom` on the bottom edge.
    Elevation {
        top: Color32,
        bottom: Color32,
    },
}

/// The resolved look of a button, fluent_ui's `ButtonStyle` with its
/// `WidgetStateProperty`s spelled out per state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonStyle {
    pub background: States<Color32>,
    pub foreground: States<Color32>,
    pub border: States<ButtonBorder>,
    pub padding: Margin,
    pub corner_radius: CornerRadius,
    /// Font size of the label. `None` uses the body style.
    pub text_size: Option<f32>,
}

/// fluent_ui's `kDefaultButtonPadding`.
pub const DEFAULT_BUTTON_PADDING: Margin = Margin {
    left: 11,
    right: 11,
    top: 5,
    bottom: 6,
};

impl ButtonStyle {
    /// `Button.defaultStyleOf`.
    pub fn standard(theme: &FluentTheme) -> Self {
        let r = &theme.resources;
        let raised = ButtonBorder::Elevation {
            top: r.control_stroke_color_default,
            bottom: r.control_stroke_color_secondary,
        };
        let flat = ButtonBorder::Solid(r.control_stroke_color_default);
        Self {
            background: States::new(
                r.control_fill_color_default,
                r.control_fill_color_secondary,
                r.control_fill_color_tertiary,
                r.control_fill_color_disabled,
            ),
            foreground: States::new(
                r.text_fill_color_primary,
                r.text_fill_color_primary,
                r.text_fill_color_secondary,
                r.text_fill_color_disabled,
            ),
            border: States::new(raised, raised, flat, flat),
            padding: DEFAULT_BUTTON_PADDING,
            corner_radius: CornerRadius::same(4),
            text_size: None,
        }
    }

    /// `FilledButton.defaultStyleOf`.
    pub fn filled(theme: &FluentTheme) -> Self {
        let r = &theme.resources;
        let raised = ButtonBorder::Elevation {
            top: r.control_stroke_color_on_accent_default,
            bottom: r.control_stroke_color_on_accent_secondary,
        };
        let flat = ButtonBorder::Solid(r.control_fill_color_transparent);
        Self {
            background: accent_fill(theme),
            foreground: on_accent_text(theme),
            border: States::new(raised, raised, flat, flat),
            ..Self::standard(theme)
        }
    }

    /// `OutlinedButton.defaultStyleOf`.
    pub fn outlined(theme: &FluentTheme) -> Self {
        let inactive = theme.inactive_color;
        Self {
            background: States::new(
                Color32::TRANSPARENT,
                with_alpha(inactive, 0.10),
                with_alpha(inactive, 0.25),
                with_alpha(theme.resources.control_fill_color_disabled, 0.30),
            ),
            foreground: States::all(inactive),
            border: States::all(ButtonBorder::Solid(inactive)),
            padding: DEFAULT_BUTTON_PADDING,
            corner_radius: CornerRadius::same(2),
            text_size: Some(13.0),
        }
    }

    /// `HyperlinkButton.defaultStyleOf`.
    pub fn hyperlink(theme: &FluentTheme) -> Self {
        let r = &theme.resources;
        let (a, b) = (theme.accent_color, theme.brightness);
        Self {
            background: States::new(
                r.subtle_fill_color_transparent,
                r.subtle_fill_color_secondary,
                r.subtle_fill_color_tertiary,
                r.subtle_fill_color_disabled,
            ),
            foreground: States::new(
                a.default_brush_for(b),
                a.secondary_brush_for(b),
                a.tertiary_brush_for(b),
                r.control_fill_color_disabled,
            ),
            border: States::all(ButtonBorder::None),
            padding: DEFAULT_BUTTON_PADDING,
            corner_radius: CornerRadius::same(4),
            text_size: None,
        }
    }

    /// `IconButton.defaultStyleOf`.
    pub fn icon(theme: &FluentTheme, mode: IconButtonMode) -> Self {
        let r = &theme.resources;
        Self {
            background: States::new(
                r.subtle_fill_color_transparent,
                r.subtle_fill_color_secondary,
                r.subtle_fill_color_tertiary,
                r.control_fill_color_disabled,
            ),
            foreground: States::new(
                r.text_fill_color_primary,
                r.text_fill_color_primary,
                r.text_fill_color_primary,
                r.text_fill_color_disabled,
            ),
            border: States::all(ButtonBorder::None),
            padding: match mode {
                IconButtonMode::Large => Margin::same(8),
                _ => DEFAULT_BUTTON_PADDING,
            },
            corner_radius: CornerRadius::same(4),
            text_size: Some(if mode == IconButtonMode::Tiny {
                11.0
            } else {
                14.0
            }),
        }
    }

    /// The checked look of a `ToggleButton`: filled colors on the standard
    /// button, like `ToggleButtonThemeData.standard`.
    pub fn toggle_checked(theme: &FluentTheme) -> Self {
        Self::filled(theme)
    }

    pub fn for_kind(theme: &FluentTheme, kind: ButtonKind) -> Self {
        match kind {
            ButtonKind::Standard => Self::standard(theme),
            ButtonKind::Filled => Self::filled(theme),
            ButtonKind::Outlined => Self::outlined(theme),
            ButtonKind::Hyperlink => Self::hyperlink(theme),
            ButtonKind::Icon(mode) => Self::icon(theme, mode),
        }
    }
}

/// `FilledButton.backgroundColor`, also used for checked inputs.
pub(crate) fn accent_fill(theme: &FluentTheme) -> States<Color32> {
    let (a, b): (_, Brightness) = (theme.accent_color, theme.brightness);
    States::new(
        a.default_brush_for(b),
        a.secondary_brush_for(b),
        a.tertiary_brush_for(b),
        theme.resources.accent_fill_color_disabled,
    )
}

/// `FilledButton.foregroundColor`.
pub(crate) fn on_accent_text(theme: &FluentTheme) -> States<Color32> {
    let r = &theme.resources;
    States::new(
        r.text_on_accent_fill_color_primary,
        r.text_on_accent_fill_color_primary,
        r.text_on_accent_fill_color_secondary,
        r.text_on_accent_fill_color_disabled,
    )
}

/// A Fluent button: `Button`, `FilledButton`, `OutlinedButton`,
/// `HyperlinkButton`, `IconButton` and `ToggleButton` from fluent_ui.
///
/// Disable it with [`Ui::add_enabled`].
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui, dark_mode: &mut bool) {
/// use fluent_egui::Button;
///
/// if ui.add(Button::filled("Save")).clicked() { /* ... */ }
/// ui.add(Button::toggle(dark_mode, "Dark mode"));
/// # }
/// ```
pub struct Button<'a> {
    text: WidgetText,
    kind: ButtonKind,
    toggle: Option<&'a mut bool>,
    style: Option<ButtonStyle>,
    min_size: Vec2,
}

impl<'a> Button<'a> {
    fn with_kind(text: impl Into<WidgetText>, kind: ButtonKind) -> Self {
        Self {
            text: text.into(),
            kind,
            toggle: None,
            style: None,
            min_size: Vec2::ZERO,
        }
    }

    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self::with_kind(text, ButtonKind::Standard)
    }

    pub fn filled(text: impl Into<WidgetText>) -> Self {
        Self::with_kind(text, ButtonKind::Filled)
    }

    pub fn outlined(text: impl Into<WidgetText>) -> Self {
        Self::with_kind(text, ButtonKind::Outlined)
    }

    pub fn hyperlink(text: impl Into<WidgetText>) -> Self {
        Self::with_kind(text, ButtonKind::Hyperlink)
    }

    /// An icon button. Pass [`crate::icon`] for a font icon, or any text.
    pub fn icon(icon: impl Into<WidgetText>) -> Self {
        Self::with_kind(icon, ButtonKind::Icon(IconButtonMode::default()))
    }

    /// A `ToggleButton`: a standard button that flips `checked` when clicked
    /// and shows the accent fill while checked.
    pub fn toggle(checked: &'a mut bool, text: impl Into<WidgetText>) -> Self {
        let mut button = Self::new(text);
        button.toggle = Some(checked);
        button
    }

    /// Changes the icon button size. No effect on other kinds.
    pub fn icon_mode(mut self, mode: IconButtonMode) -> Self {
        if let ButtonKind::Icon(_) = self.kind {
            self.kind = ButtonKind::Icon(mode);
        }
        self
    }

    /// Replaces the default look. Start from e.g. [`ButtonStyle::standard`].
    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = Some(style);
        self
    }

    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }
}

impl Widget for Button<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = FluentTheme::of(ui.ctx());
        let checked = self.toggle.as_deref().copied();
        let style = self.style.unwrap_or_else(|| match checked {
            Some(true) => ButtonStyle::toggle_checked(&theme),
            _ => ButtonStyle::for_kind(&theme, self.kind),
        });

        let mut font = theme.typography.body.font_id();
        if let Some(size) = style.text_size {
            font.size = size;
        }
        let label = self.text.text().to_owned();
        let galley = self
            .text
            .into_galley(ui, Some(TextWrapMode::Extend), f32::INFINITY, font);
        let padding = style.padding;
        let size = (galley.size() + padding.sum()).max(self.min_size);
        let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());

        if let Some(value) = self.toggle
            && response.clicked()
        {
            *value = !*value;
            response.mark_changed();
        }
        response.widget_info(|| match checked {
            Some(c) => WidgetInfo::selected(Role::Button, ui.is_enabled(), c, &label),
            None => WidgetInfo::labeled(Role::Button, ui.is_enabled(), &label),
        });

        if ui.is_rect_visible(rect) {
            let state = ControlState::from_response(&response);
            let painter = ui.painter();
            let background = animate_color(
                ui.ctx(),
                response.id.with("background"),
                style.background.resolve(state),
                theme.faster_animation_duration,
            );
            let foreground = animate_color(
                ui.ctx(),
                response.id.with("foreground"),
                style.foreground.resolve(state),
                theme.fast_animation_duration,
            );
            painter.rect_filled(rect, style.corner_radius, background);
            paint_border(
                painter,
                rect,
                style.corner_radius,
                style.border.resolve(state),
            );
            let content = Rect::from_min_max(
                rect.min + padding.left_top(),
                rect.max - padding.right_bottom(),
            );
            let text_pos = content.center() - galley.size() / 2.0;
            painter.galley(text_pos, galley, foreground);
            if state.focused {
                paint_focus_border(painter, rect, &theme);
            }
        }
        response
    }
}

pub(crate) fn paint_border(
    painter: &egui::Painter,
    rect: Rect,
    radius: CornerRadius,
    border: ButtonBorder,
) {
    match border {
        ButtonBorder::None => {}
        ButtonBorder::Solid(color) => paint_elevation_border(painter, rect, radius, color, color),
        ButtonBorder::Elevation { top, bottom } => {
            paint_elevation_border(painter, rect, radius, top, bottom);
        }
    }
}
