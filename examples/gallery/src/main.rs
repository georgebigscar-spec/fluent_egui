//! The fluent_egui gallery, the counterpart of fluent_ui's example app.
//!
//! Icon fonts are not bundled. Point `FLUENT_ICONS_TTF` (and optionally
//! `SEGOE_ICONS_TTF`) at the font files to see the Icons page.

use eframe::egui;
use fluent_egui::icons::{self, FLUENT_ICONS_FAMILY, WINDOWS_ICONS_FAMILY};
use fluent_egui::{
    AccentColor, Brightness, Button, Checkbox, ComboBox, Divider, DropDownButton, FluentTheme,
    IconButtonMode, InfoBadge, InfoLabel, MenuItem, NumberBox, PasswordBox, ProgressBar,
    ProgressRing, Severity, Slider, SplitButton, TextBox, ToggleSwitch, colors, combo_value, fonts,
    radio_value,
};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("fluent_egui gallery")
            .with_inner_size([1100.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "fluent_egui gallery",
        options,
        Box::new(|cc| Ok(Box::new(Gallery::new(&cc.egui_ctx)))),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Buttons,
    Inputs,
    TextInput,
    Menus,
    Progress,
    Status,
    Colors,
    Typography,
    Divider,
    Icons,
}

impl Page {
    const ALL: [(Page, &'static str); 10] = [
        (Page::Buttons, "Buttons"),
        (Page::Inputs, "Inputs"),
        (Page::TextInput, "Text fields"),
        (Page::Menus, "Drop-downs"),
        (Page::Progress, "Progress"),
        (Page::Status, "InfoBadge & InfoLabel"),
        (Page::Colors, "Colors"),
        (Page::Typography, "Typography"),
        (Page::Divider, "Divider"),
        (Page::Icons, "Icons"),
    ];
}

struct Gallery {
    page: Page,
    brightness: Brightness,
    accent: usize,
    icon_fonts: Vec<&'static str>,
    icon_filter: String,
    demo: Demo,
}

/// State for the control pages.
struct Demo {
    disabled: bool,
    clicks: u32,
    toggled: bool,
    checked: bool,
    tristate: Option<bool>,
    switch: bool,
    radio: usize,
    slider: f32,
    stepped: f32,
    progress: f32,
    name: String,
    notes: String,
    password: String,
    quantity: f64,
    price: f64,
    color: usize,
    font: Option<usize>,
    last_action: String,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            disabled: false,
            clicks: 0,
            toggled: false,
            checked: true,
            tristate: None,
            switch: true,
            radio: 0,
            slider: 40.0,
            stepped: 3.0,
            progress: 65.0,
            name: String::new(),
            notes: String::new(),
            password: String::new(),
            quantity: 3.0,
            price: 9.99,
            color: 0,
            font: None,
            last_action: "nothing yet".to_owned(),
        }
    }
}

impl Gallery {
    fn new(ctx: &egui::Context) -> Self {
        let mut defs = egui::FontDefinitions::default();
        let mut icon_fonts = Vec::new();
        for (var, family) in [
            ("FLUENT_ICONS_TTF", FLUENT_ICONS_FAMILY),
            ("SEGOE_ICONS_TTF", WINDOWS_ICONS_FAMILY),
        ] {
            if let Some(bytes) = std::env::var_os(var).and_then(|p| std::fs::read(p).ok()) {
                fonts::add_family(&mut defs, family, bytes);
                icon_fonts.push(family);
            }
        }
        ctx.set_fonts(defs);

        let gallery = Self {
            // GALLERY_PAGE picks the start page by its title, handy for screenshots.
            page: std::env::var("GALLERY_PAGE")
                .ok()
                .and_then(|name| {
                    Page::ALL
                        .iter()
                        .find(|(_, title)| title.to_lowercase().starts_with(&name.to_lowercase()))
                        .map(|(page, _)| *page)
                })
                .unwrap_or(Page::Buttons),
            brightness: if std::env::var_os("GALLERY_DARK").is_some() {
                Brightness::Dark
            } else {
                Brightness::Light
            },
            accent: 5, // blue
            icon_fonts,
            icon_filter: String::new(),
            demo: Demo::default(),
        };
        gallery.install_theme(ctx);
        gallery
    }

    fn accent(&self) -> AccentColor {
        colors::ACCENT_COLORS[self.accent].1
    }

    fn install_theme(&self, ctx: &egui::Context) {
        FluentTheme::with_accent(self.brightness, self.accent()).install(ctx);
    }
}

impl eframe::App for Gallery {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let theme = FluentTheme::of(&ctx);

        egui::Panel::left("navigation")
            .resizable(false)
            .exact_size(220.0)
            .show_separator_line(false)
            .show(ui, |ui| {
                ui.add_space(12.0);
                ui.label(theme.typography.subtitle.rich("fluent_egui"));
                ui.add_space(8.0);
                for (page, name) in Page::ALL {
                    ui.selectable_value(&mut self.page, page, name);
                }
                ui.add_space(8.0);
                ui.add(Divider::horizontal());
                ui.add_space(8.0);

                let mut changed = false;
                ui.horizontal(|ui| {
                    changed |= ui
                        .selectable_value(&mut self.brightness, Brightness::Light, "Light")
                        .changed();
                    changed |= ui
                        .selectable_value(&mut self.brightness, Brightness::Dark, "Dark")
                        .changed();
                });
                ui.label(theme.typography.caption.rich("Accent"));
                ui.horizontal_wrapped(|ui| {
                    for (i, (name, accent)) in colors::ACCENT_COLORS.iter().enumerate() {
                        let (rect, response) =
                            ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::click());
                        ui.painter().rect_filled(rect, 4.0, accent.normal);
                        if i == self.accent {
                            ui.painter().rect_stroke(
                                rect,
                                4.0,
                                egui::Stroke::new(2.0, theme.resources.text_fill_color_primary),
                                egui::StrokeKind::Outside,
                            );
                        }
                        if response.on_hover_text(*name).clicked() {
                            self.accent = i;
                            changed = true;
                        }
                    }
                });
                if changed {
                    self.install_theme(&ctx);
                }
            });

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| match self.page {
                Page::Buttons => buttons_page(ui, &theme, &mut self.demo, &self.icon_fonts),
                Page::Inputs => inputs_page(ui, &theme, &mut self.demo),
                Page::TextInput => text_page(ui, &theme, &mut self.demo),
                Page::Menus => menus_page(ui, &theme, &mut self.demo),
                Page::Progress => progress_page(ui, &theme, &mut self.demo),
                Page::Status => status_page(ui, &theme, &mut self.demo),
                Page::Colors => colors_page(ui, &theme),
                Page::Typography => typography_page(ui, &theme),
                Page::Divider => divider_page(ui, &theme),
                Page::Icons => icons_page(ui, &theme, &self.icon_fonts, &mut self.icon_filter),
            });
        });
    }
}

fn page_title(ui: &mut egui::Ui, theme: &FluentTheme, title: &str) {
    ui.add_space(8.0);
    ui.label(theme.typography.title.rich(title));
    ui.add_space(12.0);
}

/// A color chip with its name underneath, allocated as one fixed-size rect so
/// it wraps cleanly inside `horizontal_wrapped`.
fn swatch(ui: &mut egui::Ui, theme: &FluentTheme, color: egui::Color32, name: &str) {
    let width = 96.0f32.max(name.len() as f32 * 7.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 68.0), egui::Sense::hover());
    let chip = egui::Rect::from_min_size(rect.min, egui::vec2(width, 48.0));
    let painter = ui.painter();
    painter.rect_filled(chip, 4.0, color);
    painter.rect_stroke(
        chip,
        4.0,
        egui::Stroke::new(1.0, theme.resources.card_stroke_color_default),
        egui::StrokeKind::Inside,
    );
    let caption = &theme.typography.caption;
    painter.text(
        egui::pos2(rect.left(), chip.bottom() + 4.0),
        egui::Align2::LEFT_TOP,
        name,
        caption.font_id(),
        caption.color,
    );
}

fn colors_page(ui: &mut egui::Ui, theme: &FluentTheme) {
    page_title(ui, theme, "Colors");
    ui.label(theme.typography.body_strong.rich("Accent shades"));
    let a = theme.accent_color;
    ui.horizontal_wrapped(|ui| {
        for (name, color) in [
            ("darkest", a.darkest),
            ("darker", a.darker),
            ("dark", a.dark),
            ("normal", a.normal),
            ("light", a.light),
            ("lighter", a.lighter),
            ("lightest", a.lightest),
        ] {
            swatch(ui, theme, color, name);
        }
    });
    ui.add_space(12.0);
    ui.label(theme.typography.body_strong.rich("Grey"));
    ui.horizontal_wrapped(|ui| {
        for (shade, color) in colors::GREY_SHADES {
            swatch(ui, theme, color, &format!("grey {shade}"));
        }
    });
    ui.add_space(12.0);
    ui.label(theme.typography.body_strong.rich("Resources"));
    let r = &theme.resources;
    ui.horizontal_wrapped(|ui| {
        for (name, color) in [
            ("textFillColorPrimary", r.text_fill_color_primary),
            ("textFillColorSecondary", r.text_fill_color_secondary),
            ("controlFillColorDefault", r.control_fill_color_default),
            ("controlFillColorSecondary", r.control_fill_color_secondary),
            ("controlStrokeColorDefault", r.control_stroke_color_default),
            (
                "cardBackgroundFillColorDefault",
                r.card_background_fill_color_default,
            ),
            ("dividerStrokeColorDefault", r.divider_stroke_color_default),
            (
                "solidBackgroundFillColorBase",
                r.solid_background_fill_color_base,
            ),
            ("systemFillColorSuccess", r.system_fill_color_success),
            ("systemFillColorCaution", r.system_fill_color_caution),
            ("systemFillColorCritical", r.system_fill_color_critical),
            (
                "systemFillColorAttentionBackground",
                r.system_fill_color_attention_background,
            ),
        ] {
            swatch(ui, theme, color, name);
        }
    });
}

fn typography_page(ui: &mut egui::Ui, theme: &FluentTheme) {
    page_title(ui, theme, "Typography");
    for (name, style) in theme.typography.styles() {
        ui.horizontal(|ui| {
            ui.add_sized(
                [120.0, style.line_height],
                egui::Label::new(
                    theme
                        .typography
                        .caption
                        .rich(format!("{name}\n{}/{}", style.size, style.line_height)),
                ),
            );
            ui.label(style.rich("The quick brown fox"));
        });
    }
}

fn divider_page(ui: &mut egui::Ui, theme: &FluentTheme) {
    page_title(ui, theme, "Divider");
    ui.label(theme.typography.body.rich("Horizontal"));
    ui.add(Divider::horizontal().size(400.0));
    ui.add_space(16.0);
    ui.label(theme.typography.body.rich("Vertical"));
    ui.horizontal(|ui| {
        ui.label("Left");
        ui.add(Divider::vertical().size(60.0));
        ui.label("Right");
    });
}

fn icons_page(
    ui: &mut egui::Ui,
    theme: &FluentTheme,
    installed: &[&'static str],
    filter: &mut String,
) {
    page_title(ui, theme, "Icons");
    if installed.is_empty() {
        ui.label(theme.typography.body.rich(
            "Icon fonts are not bundled. Set FLUENT_ICONS_TTF and/or SEGOE_ICONS_TTF \
             to the font files and restart the gallery.",
        ));
        return;
    }
    ui.add(egui::TextEdit::singleline(filter).hint_text("Filter icons"));
    let filter = filter.to_lowercase();
    for (family, title, all) in [
        (FLUENT_ICONS_FAMILY, "Fluent icons", icons::fluent::ALL),
        (WINDOWS_ICONS_FAMILY, "Windows icons", icons::windows::ALL),
    ] {
        if !installed.contains(&family) {
            continue;
        }
        ui.add_space(12.0);
        ui.label(theme.typography.body_strong.rich(title));
        let font = egui::FontId::new(24.0, egui::FontFamily::Name(family.into()));
        ui.horizontal_wrapped(|ui| {
            for (name, ch) in all.iter().filter(|(n, _)| n.contains(&filter)).take(400) {
                ui.label(egui::RichText::new(ch.to_string()).font(font.clone()))
                    .on_hover_text(*name);
            }
        });
    }
}

fn section(ui: &mut egui::Ui, theme: &FluentTheme, title: &str) {
    ui.add_space(16.0);
    ui.label(theme.typography.body_strong.rich(title));
    ui.add_space(6.0);
}

fn disabled_switch(ui: &mut egui::Ui, demo: &mut Demo) {
    ui.add(ToggleSwitch::new(&mut demo.disabled, "Disable controls"));
}

fn buttons_page(ui: &mut egui::Ui, theme: &FluentTheme, demo: &mut Demo, icon_fonts: &[&str]) {
    page_title(ui, theme, "Buttons");
    disabled_switch(ui, demo);
    let enabled = !demo.disabled;

    section(
        ui,
        theme,
        "Button, FilledButton, OutlinedButton, HyperlinkButton",
    );
    ui.horizontal(|ui| {
        if ui.add_enabled(enabled, Button::new("Standard")).clicked() {
            demo.clicks += 1;
        }
        if ui.add_enabled(enabled, Button::filled("Filled")).clicked() {
            demo.clicks += 1;
        }
        if ui
            .add_enabled(enabled, Button::outlined("Outlined"))
            .clicked()
        {
            demo.clicks += 1;
        }
        if ui
            .add_enabled(enabled, Button::hyperlink("Hyperlink"))
            .clicked()
        {
            demo.clicks += 1;
        }
    });
    ui.label(
        theme
            .typography
            .caption
            .rich(format!("Clicked {} times", demo.clicks)),
    );

    section(ui, theme, "ToggleButton");
    ui.add_enabled(enabled, Button::toggle(&mut demo.toggled, "Toggle me"));

    section(ui, theme, "IconButton");
    ui.horizontal(|ui| {
        let has_font = icon_fonts.contains(&FLUENT_ICONS_FAMILY);
        for mode in [
            IconButtonMode::Tiny,
            IconButtonMode::Small,
            IconButtonMode::Large,
        ] {
            let button = if has_font {
                Button::icon(fluent_egui::icon(icons::fluent::ADD))
            } else {
                Button::icon("+")
            };
            ui.add_enabled(enabled, button.icon_mode(mode))
                .on_hover_text(format!("{mode:?}"));
        }
    });
}

fn inputs_page(ui: &mut egui::Ui, theme: &FluentTheme, demo: &mut Demo) {
    page_title(ui, theme, "Inputs");
    disabled_switch(ui, demo);
    let enabled = !demo.disabled;

    section(ui, theme, "Checkbox");
    ui.add_enabled(enabled, Checkbox::new(&mut demo.checked, "Two-state"));
    ui.horizontal(|ui| {
        ui.add_enabled(
            enabled,
            Checkbox::tristate(&mut demo.tristate, "Three-state"),
        );
        if ui
            .add_enabled(enabled, Button::hyperlink("Reset to third state"))
            .clicked()
        {
            demo.tristate = None;
        }
    });

    section(ui, theme, "RadioButton");
    ui.add_enabled_ui(enabled, |ui| {
        for (i, name) in ["Option 1", "Option 2", "Option 3"].into_iter().enumerate() {
            radio_value(ui, &mut demo.radio, i, name);
        }
    });

    section(ui, theme, "ToggleSwitch");
    let label = if demo.switch { "On" } else { "Off" };
    ui.add_enabled(enabled, ToggleSwitch::new(&mut demo.switch, label));

    section(ui, theme, "Slider");
    ui.horizontal(|ui| {
        ui.add_enabled(enabled, Slider::new(&mut demo.slider, 0.0..=100.0));
        ui.label(format!("{:.0}", demo.slider));
    });
    ui.horizontal(|ui| {
        ui.add_enabled(
            enabled,
            Slider::new(&mut demo.stepped, 0.0..=10.0).step(1.0),
        );
        ui.label(format!("{:.0} (step 1)", demo.stepped));
    });
}

fn progress_page(ui: &mut egui::Ui, theme: &FluentTheme, demo: &mut Demo) {
    page_title(ui, theme, "Progress");
    section(ui, theme, "Determinate");
    ui.add(Slider::new(&mut demo.progress, 0.0..=100.0));
    ui.add_space(8.0);
    ui.add(ProgressBar::new(demo.progress).width(300.0));
    ui.add_space(8.0);
    ui.add(ProgressRing::new(demo.progress));

    section(ui, theme, "Indeterminate");
    ui.add(ProgressBar::indeterminate().width(300.0));
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add(ProgressRing::indeterminate());
        ui.add(ProgressRing::indeterminate().backwards(true));
    });
}

fn status_page(ui: &mut egui::Ui, theme: &FluentTheme, demo: &mut Demo) {
    page_title(ui, theme, "InfoBadge & InfoLabel");
    section(ui, theme, "InfoBadge");
    for severity in [
        Severity::Info,
        Severity::Warning,
        Severity::Success,
        Severity::Error,
    ] {
        ui.horizontal(|ui| {
            ui.add(InfoBadge::dot().severity(severity));
            ui.add(InfoBadge::new("8").severity(severity));
            ui.add(InfoBadge::new("99+").severity(severity));
            ui.label(format!("{severity:?}"));
        });
    }

    section(ui, theme, "InfoLabel");
    InfoLabel::new("Header label").show(ui, |ui| {
        ui.add(egui::TextEdit::singleline(&mut demo.name).hint_text("Your name"));
    });
    ui.add_space(8.0);
    InfoLabel::new("Label on the side")
        .header(false)
        .show(ui, |ui| {
            ui.add(ToggleSwitch::without_text(&mut demo.switch));
        });
}

fn text_page(ui: &mut egui::Ui, theme: &FluentTheme, demo: &mut Demo) {
    page_title(ui, theme, "Text fields");
    disabled_switch(ui, demo);
    let enabled = !demo.disabled;

    section(ui, theme, "TextBox");
    ui.add_enabled(enabled, TextBox::new(&mut demo.name).placeholder("Name"));
    ui.add_space(8.0);
    ui.add_enabled(
        enabled,
        TextBox::multiline(&mut demo.notes, 3).placeholder("Notes"),
    );

    section(ui, theme, "PasswordBox");
    ui.add_enabled(
        enabled,
        PasswordBox::new(&mut demo.password).placeholder("Password"),
    );
    ui.label(
        theme
            .typography
            .caption
            .rich("Hold the eye button to peek."),
    );

    section(ui, theme, "NumberBox");
    ui.horizontal(|ui| {
        ui.add_enabled(
            enabled,
            NumberBox::new(&mut demo.quantity)
                .range(0.0..=99.0)
                .precision(0)
                .width(140.0),
        );
        ui.label(format!("quantity = {}", demo.quantity));
    });
    ui.horizontal(|ui| {
        ui.add_enabled(
            enabled,
            NumberBox::new(&mut demo.price)
                .range(0.0..=1000.0)
                .small_change(0.5)
                .precision(2)
                .width(140.0),
        );
        ui.label(format!("price = {:.2}", demo.price));
    });
}

fn menus_page(ui: &mut egui::Ui, theme: &FluentTheme, demo: &mut Demo) {
    page_title(ui, theme, "Drop-downs");
    disabled_switch(ui, demo);
    let enabled = !demo.disabled;

    ui.add_enabled_ui(enabled, |ui| {
        section(ui, theme, "DropDownButton");
        DropDownButton::new("Email").show(ui, |ui| {
            for action in ["Send", "Reply", "Reply all"] {
                if ui.add(MenuItem::new(action)).clicked() {
                    demo.last_action = action.to_owned();
                }
            }
        });

        section(ui, theme, "SplitButton");
        let split = SplitButton::new("Choose color").show(ui, |ui| {
            for name in ["Red", "Green", "Blue"] {
                if ui.add(MenuItem::new(name)).clicked() {
                    demo.last_action = format!("color {name}");
                }
            }
        });
        if split.main.clicked() {
            demo.last_action = "split button main action".to_owned();
        }
        ui.label(
            theme
                .typography
                .caption
                .rich(format!("Last action: {}", demo.last_action)),
        );

        section(ui, theme, "ComboBox");
        let colors = ["Blue", "Green", "Red", "Yellow"];
        ComboBox::new("color", colors[demo.color])
            .width(200.0)
            .show_ui(ui, |ui| {
                for (i, name) in colors.iter().enumerate() {
                    combo_value(ui, &mut demo.color, i, *name);
                }
            });
        ui.add_space(8.0);
        let fonts = ["Segoe UI", "Arial", "Consolas"];
        let combo = match demo.font {
            Some(i) => ComboBox::new("font", fonts[i]),
            None => ComboBox::placeholder("font", "Pick a font"),
        };
        combo.width(200.0).show_ui(ui, |ui| {
            for (i, name) in fonts.iter().enumerate() {
                combo_value(ui, &mut demo.font, Some(i), *name);
            }
        });
    });
}
