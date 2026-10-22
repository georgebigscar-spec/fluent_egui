use egui::{Color32, FontFamily, FontId, RichText};

/// One step of the Windows 11 type ramp.
#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    pub size: f32,
    pub line_height: f32,
    pub family: FontFamily,
    pub color: Color32,
    /// The weight fluent_ui uses. egui has no synthetic bold, so this only
    /// takes effect through [`Typography::with_semibold_family`].
    pub semibold: bool,
}

impl TextStyle {
    pub fn font_id(&self) -> FontId {
        FontId::new(self.size, self.family.clone())
    }

    /// `text` styled with this step of the ramp.
    pub fn rich(&self, text: impl Into<String>) -> RichText {
        RichText::new(text)
            .font(self.font_id())
            .color(self.color)
            .line_height(Some(self.line_height))
    }
}

/// The Windows 11 type ramp, fluent_ui's `Typography`.
#[derive(Clone, Debug, PartialEq)]
pub struct Typography {
    pub display: TextStyle,
    pub title_large: TextStyle,
    pub title: TextStyle,
    pub subtitle: TextStyle,
    pub body_large: TextStyle,
    pub body_strong: TextStyle,
    pub body: TextStyle,
    pub caption: TextStyle,
}

impl Typography {
    /// The default ramp in `color`, with sizes and line heights from
    /// <https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/typography#type-ramp>.
    pub fn with_color(color: Color32) -> Self {
        let step = |size, line_height, semibold| TextStyle {
            size,
            line_height,
            family: FontFamily::Proportional,
            color,
            semibold,
        };
        Self {
            display: step(68.0, 92.0, true),
            title_large: step(40.0, 52.0, true),
            title: step(28.0, 36.0, true),
            subtitle: step(20.0, 28.0, true),
            body_large: step(18.0, 24.0, false),
            body_strong: step(14.0, 20.0, true),
            body: step(14.0, 20.0, false),
            caption: step(12.0, 16.0, false),
        }
    }

    /// Uses `family` for the semibold steps. Register the font first, for
    /// example with [`crate::fonts::add_family`].
    pub fn with_semibold_family(mut self, family: FontFamily) -> Self {
        for style in self.styles_mut() {
            if style.semibold {
                style.family = family.clone();
            }
        }
        self
    }

    /// Every step with its name, from largest to smallest.
    pub fn styles(&self) -> [(&'static str, &TextStyle); 8] {
        [
            ("display", &self.display),
            ("title_large", &self.title_large),
            ("title", &self.title),
            ("subtitle", &self.subtitle),
            ("body_large", &self.body_large),
            ("body_strong", &self.body_strong),
            ("body", &self.body),
            ("caption", &self.caption),
        ]
    }

    fn styles_mut(&mut self) -> [&mut TextStyle; 8] {
        [
            &mut self.display,
            &mut self.title_large,
            &mut self.title,
            &mut self.subtitle,
            &mut self.body_large,
            &mut self.body_strong,
            &mut self.body,
            &mut self.caption,
        ]
    }
}
