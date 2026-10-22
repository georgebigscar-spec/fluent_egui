//! Localized UI strings, fluent_ui's `FluentLocalizations`.
//!
//! Widgets read their built-in labels and tooltips ("Close", "Pick a date",
//! "No results found", …) from the installed [`Localizations`]:
//!
//! ```no_run
//! # fn setup(ctx: &egui::Context) {
//! fluent_egui::Localizations::for_locale("ru-RU").install(ctx);
//! # }
//! ```

mod strings;

pub use strings::{ALL, Localizations};

impl Default for Localizations {
    fn default() -> Self {
        Self::EN
    }
}

impl Localizations {
    /// The bundled strings that best match `tag`, such as `"ru"`, `"pt-BR"` or
    /// `"zh-Hant-TW"`, falling back to the language alone and then to English.
    pub fn for_locale(tag: &str) -> Self {
        let tag = tag.replace('-', "_");
        let mut parts: Vec<&str> = tag.split('_').collect();
        while !parts.is_empty() {
            let candidate = parts.join("_");
            if let Some(found) = ALL
                .iter()
                .find(|l| l.locale.eq_ignore_ascii_case(&candidate))
            {
                return *found;
            }
            parts.pop();
        }
        Self::EN
    }

    /// The strings installed in `ctx`, English if none were.
    pub fn of(ctx: &egui::Context) -> Self {
        ctx.data(|d| d.get_temp::<Self>(egui::Id::NULL))
            .unwrap_or_default()
    }

    /// Makes these the strings every widget in `ctx` uses.
    pub fn install(self, ctx: &egui::Context) {
        ctx.data_mut(|d| d.insert_temp(egui::Id::NULL, self));
    }

    /// The colour slider tooltip, "Value 50 (Red)" in English.
    pub fn format_value_slider_tooltip(&self, value: i32, color_name: &str) -> String {
        self.value_slider_tooltip
            .replace("{value}", &value.to_string())
            .replace("{colorName}", color_name)
    }

    /// The opacity slider tooltip, "50% opacity" in English.
    pub fn format_alpha_slider_tooltip(&self, value: i32) -> String {
        self.alpha_slider_tooltip
            .replace("{value}", &value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_matching() {
        assert_eq!(Localizations::for_locale("ru-RU").locale, "ru");
        assert_eq!(Localizations::for_locale("zh-Hant-TW").locale, "zh_Hant");
        assert_eq!(Localizations::for_locale("zh").locale, "zh");
        assert_eq!(Localizations::for_locale("xx").locale, "en");
        assert_eq!(Localizations::RU.close_button_label, "Закрыть");
    }

    #[test]
    fn untranslated_falls_back_to_english() {
        // Arabic has no "Pick a date" yet.
        assert_eq!(Localizations::AR.pick_a_date, Localizations::EN.pick_a_date);
    }

    #[test]
    fn placeholders() {
        assert_eq!(
            Localizations::EN.format_alpha_slider_tooltip(40),
            "40% opacity"
        );
        assert_eq!(
            Localizations::EN.format_value_slider_tooltip(7, "Red"),
            "Value 7 (Red)"
        );
    }
}
