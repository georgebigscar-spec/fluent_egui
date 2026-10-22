use egui::{InnerResponse, Ui, WidgetText};

/// Labels a control, fluent_ui's `InfoLabel`: the label above the content
/// (header, the default) or after it.
///
/// ```no_run
/// # fn ui(ui: &mut egui::Ui, name: &mut String) {
/// fluent_egui::InfoLabel::new("Name").show(ui, |ui| ui.text_edit_singleline(name));
/// # }
/// ```
pub struct InfoLabel {
    label: WidgetText,
    header: bool,
}

impl InfoLabel {
    pub fn new(label: impl Into<WidgetText>) -> Self {
        Self {
            label: label.into(),
            header: true,
        }
    }

    /// `false` puts the label to the right of the content.
    pub fn header(mut self, header: bool) -> Self {
        self.header = header;
        self
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        // egui's Body text style is the Fluent body style once the theme
        // is installed, so a plain label already matches fluent_ui.
        let label = self.label;
        let styled = |ui: &mut Ui| ui.label(label);
        if self.header {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                styled(ui);
                add_contents(ui)
            })
        } else {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let inner = add_contents(ui);
                styled(ui);
                inner
            })
        }
    }
}
