//! The fluent_egui gallery, the counterpart of fluent_ui's example app.
//!
//! Icon fonts are not bundled. Point `FLUENT_ICONS_TTF` (and optionally
//! `SEGOE_ICONS_TTF`) at the font files to see the Icons page.

use eframe::egui;
use fluent_egui::icons::{self, FLUENT_ICONS_FAMILY, WINDOWS_ICONS_FAMILY};
use fluent_egui::{
    AccentColor, Acrylic, AutoSuggestBox, BreadcrumbBar, Brightness, Button, CalendarDatePicker,
    CalendarView, Card, Checkbox, ColorPicker, ColorSpectrumShape, ComboBox, CommandBar,
    ContentDialog, DatePicker, DialogResult, Divider, DropDownButton, Expander, FluentTheme,
    FluentTooltip as _, IconButtonMode, InfoBadge, InfoBar, InfoLabel, ListTile,
    ListTileSelectionMode, Localizations, MenuBar, MenuItem, Mica, NavigationView, NumberBox,
    PageTransition, PageTransitionKind, PaneDisplayMode, PaneItem, PasswordBox, ProgressBar,
    ProgressRing, Rating, ScaffoldPage, Severity, Slider, SplitButton, Tab, TabView, TeachingTip,
    TextBox, TimePicker, ToggleSwitch, TreeView, TreeViewItem, TreeViewSelectionMode, colors,
    combo_value, context_menu, fonts, l10n, menu_separator, radio_value, sub_menu,
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
    Surfaces,
    Flyouts,
    Navigation,
    Pickers,
    Progress,
    Status,
    Colors,
    Typography,
    Divider,
    Icons,
    Settings,
}

/// A pane entry: a section header, or a page with its icon and title.
enum Entry {
    Header(&'static str),
    Page(Page, &'static str, &'static str),
}

impl Page {
    const PANE: [Entry; 18] = [
        Entry::Header("Controls"),
        Entry::Page(Page::Buttons, "🔘", "Buttons"),
        Entry::Page(Page::Inputs, "☑", "Inputs"),
        Entry::Page(Page::TextInput, "✏", "Text fields"),
        Entry::Page(Page::Menus, "⏷", "Drop-downs"),
        Entry::Header("Layout"),
        Entry::Page(Page::Surfaces, "🗖", "Surfaces"),
        Entry::Page(Page::Flyouts, "💬", "Flyouts & menus"),
        Entry::Page(Page::Navigation, "🗺", "Navigation"),
        Entry::Page(Page::Pickers, "📅", "Pickers"),
        Entry::Header("Status"),
        Entry::Page(Page::Progress, "⏳", "Progress"),
        Entry::Page(Page::Status, "🔔", "InfoBadge & InfoLabel"),
        Entry::Header("Theming"),
        Entry::Page(Page::Colors, "🎨", "Colors"),
        Entry::Page(Page::Typography, "🔤", "Typography"),
        Entry::Page(Page::Divider, "➖", "Divider"),
        Entry::Page(Page::Icons, "⭐", "Icons"),
    ];

    fn all() -> impl Iterator<Item = (Page, &'static str)> {
        Self::PANE
            .iter()
            .filter_map(|e| match e {
                Entry::Page(page, _, title) => Some((*page, *title)),
                Entry::Header(_) => None,
            })
            .chain([(Page::Settings, "Settings")])
    }

    fn title(self) -> &'static str {
        Self::all().find(|(p, _)| *p == self).map_or("", |(_, t)| t)
    }
}

struct Gallery {
    page: Page,
    brightness: Brightness,
    accent: usize,
    icon_fonts: Vec<&'static str>,
    icon_filter: String,
    nav_mode: PaneDisplayMode,
    transition: usize,
    locale: usize,
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
    info_visible: bool,
    dialog_open: bool,
    tip_open: bool,
    tabs: Vec<(u32, String)>,
    next_tab: u32,
    current_tab: usize,
    path: Vec<&'static str>,
    tree_single: Vec<&'static str>,
    tree_multiple: Vec<&'static str>,
    date: Option<fluent_egui::Date>,
    due: Option<fluent_egui::Date>,
    birthday: Option<fluent_egui::Date>,
    alarm: Option<fluent_egui::Time>,
    meeting: Option<fluent_egui::Time>,
    picked_color: egui::Color32,
    box_spectrum: bool,
    rating: f32,
    query: String,
    list_selected: usize,
    list_checked: [bool; 3],
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
            info_visible: true,
            dialog_open: false,
            tip_open: false,
            tabs: (1..=3).map(|i| (i, format!("Document {i}"))).collect(),
            next_tab: 4,
            current_tab: 0,
            path: BREADCRUMB.to_vec(),
            tree_single: vec!["vacation.jpg"],
            tree_multiple: vec!["cv.pdf"],
            date: None,
            due: None,
            birthday: fluent_egui::Date::new(1990, 6, 15),
            alarm: None,
            meeting: fluent_egui::Time::new(14, 30),
            picked_color: egui::Color32::from_rgb(0x00, 0x78, 0xd4),
            box_spectrum: false,
            rating: 3.5,
            query: String::new(),
            list_selected: 0,
            list_checked: [true, false, false],
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
                    Page::all()
                        .find(|(_, title)| title.to_lowercase().starts_with(&name.to_lowercase()))
                        .map(|(page, _)| page)
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
            // GALLERY_NAV picks the pane display mode by name.
            nav_mode: std::env::var("GALLERY_NAV")
                .ok()
                .and_then(|name| {
                    NAV_MODES
                        .iter()
                        .find(|(_, n)| n.eq_ignore_ascii_case(&name))
                        .map(|(m, _)| *m)
                })
                .unwrap_or_default(),
            transition: 0,
            // GALLERY_LOCALE picks the language, such as `ru`.
            locale: std::env::var("GALLERY_LOCALE")
                .ok()
                .map(|tag| Localizations::for_locale(&tag).locale)
                .and_then(|tag| l10n::ALL.iter().position(|l| l.locale == tag))
                .unwrap_or(0),
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
        l10n::ALL[self.locale].install(ctx);
    }
}

impl eframe::App for Gallery {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let theme = FluentTheme::of(&ctx);
        let items = Page::PANE.iter().map(|e| match e {
            Entry::Header(text) => PaneItem::header(*text),
            Entry::Page(page, icon, title) => PaneItem::new(*page, *title).icon(*icon),
        });
        let current = self.page;
        let mut page = current;
        let mut nav = NavigationView::new("gallery_nav");
        if let Some(transition) = TRANSITIONS[self.transition].0 {
            nav = nav.transition(transition);
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(theme.mica_background_color))
            .show(ui, |ui| {
                nav.title("fluent_egui")
                    .display_mode(self.nav_mode)
                    .items(items)
                    .footer_items([PaneItem::new(Page::Settings, "Settings").icon("⚙")])
                    .show(ui, &mut page, |ui| {
                        ScaffoldPage::new()
                            .header(current.title())
                            .show(ui, |ui| match current {
                                Page::Buttons => {
                                    buttons_page(ui, &theme, &mut self.demo, &self.icon_fonts)
                                }
                                Page::Inputs => inputs_page(ui, &theme, &mut self.demo),
                                Page::TextInput => text_page(ui, &theme, &mut self.demo),
                                Page::Menus => menus_page(ui, &theme, &mut self.demo),
                                Page::Surfaces => surfaces_page(ui, &theme, &mut self.demo),
                                Page::Flyouts => flyouts_page(ui, &theme, &mut self.demo),
                                Page::Navigation => navigation_page(ui, &theme, &mut self.demo),
                                Page::Pickers => pickers_page(ui, &theme, &mut self.demo),
                                Page::Progress => progress_page(ui, &theme, &mut self.demo),
                                Page::Status => status_page(ui, &theme, &mut self.demo),
                                Page::Colors => colors_page(ui, &theme),
                                Page::Typography => typography_page(ui, &theme),
                                Page::Divider => divider_page(ui, &theme),
                                Page::Icons => {
                                    icons_page(ui, &theme, &self.icon_fonts, &mut self.icon_filter)
                                }
                                Page::Settings => {
                                    if settings_page(ui, &theme, self) {
                                        self.install_theme(&ctx);
                                    }
                                }
                            });
                    });
            });
        self.page = page;
    }
}

fn pickers_page(ui: &mut egui::Ui, theme: &FluentTheme, demo: &mut Demo) {
    let caption = |ui: &mut egui::Ui, text: String| {
        ui.label(theme.typography.caption.rich(text));
    };
    section(ui, theme, "DatePicker");
    ui.add(DatePicker::new("birthday", &mut demo.birthday));
    caption(
        ui,
        format!("Value: {:?}", demo.birthday.map(|d| d.to_string())),
    );

    section(
        ui,
        theme,
        "TimePicker (12-hour, and 24-hour in 15 minute steps)",
    );
    ui.horizontal(|ui| {
        ui.add(TimePicker::new("alarm", &mut demo.alarm));
        ui.add_space(12.0);
        ui.add(
            TimePicker::new("meeting", &mut demo.meeting)
                .use_24_hour(true)
                .minute_increment(15),
        );
    });
    caption(
        ui,
        format!(
            "Values: {:?}, {:?}",
            demo.alarm.map(|t| t.to_string()),
            demo.meeting.map(|t| t.to_string())
        ),
    );

    section(ui, theme, "CalendarDatePicker");
    ui.add(CalendarDatePicker::new("due", &mut demo.due));

    section(ui, theme, "CalendarView");
    ui.add(CalendarView::new("calendar", &mut demo.date));
    caption(
        ui,
        match demo.date {
            Some(date) => format!("Picked {date}"),
            None => "Nothing picked yet".to_owned(),
        },
    );

    section(ui, theme, "ColorPicker");
    ui.add(ToggleSwitch::new(&mut demo.box_spectrum, "Box spectrum"));
    ui.add_space(8.0);
    let shape = if demo.box_spectrum {
        ColorSpectrumShape::Box
    } else {
        ColorSpectrumShape::Ring
    };
    ui.add(ColorPicker::new("color_picker", &mut demo.picked_color).spectrum_shape(shape));
}

const FRUITS: [&str; 12] = [
    "Apple",
    "Apricot",
    "Banana",
    "Blackberry",
    "Blueberry",
    "Cherry",
    "Grape",
    "Lemon",
    "Mango",
    "Orange",
    "Peach",
    "Pear",
];

const NAV_MODES: [(PaneDisplayMode, &str); 5] = [
    (PaneDisplayMode::Auto, "Auto"),
    (PaneDisplayMode::Open, "Open"),
    (PaneDisplayMode::Compact, "Compact"),
    (PaneDisplayMode::Minimal, "Minimal"),
    (PaneDisplayMode::Top, "Top"),
];

/// Returns whether the theme changed.
fn settings_page(ui: &mut egui::Ui, theme: &FluentTheme, gallery: &mut Gallery) -> bool {
    let mut changed = false;
    section(ui, theme, "Theme mode");
    changed |= radio_value(ui, &mut gallery.brightness, Brightness::Light, "Light").changed();
    changed |= radio_value(ui, &mut gallery.brightness, Brightness::Dark, "Dark").changed();

    section(ui, theme, "Accent color");
    ui.horizontal_wrapped(|ui| {
        for (i, (name, accent)) in colors::ACCENT_COLORS.iter().enumerate() {
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());
            ui.painter().rect_filled(rect, 4.0, accent.normal);
            if i == gallery.accent {
                ui.painter().rect_stroke(
                    rect,
                    4.0,
                    egui::Stroke::new(2.0, theme.resources.text_fill_color_primary),
                    egui::StrokeKind::Outside,
                );
            }
            if response.fluent_tooltip(*name).clicked() {
                gallery.accent = i;
                changed = true;
            }
        }
    });

    section(ui, theme, "Navigation pane");
    let current = NAV_MODES
        .iter()
        .find(|(m, _)| *m == gallery.nav_mode)
        .map_or("", |(_, n)| n);
    ComboBox::new("nav_mode", current).show_ui(ui, |ui| {
        for (mode, name) in NAV_MODES {
            combo_value(ui, &mut gallery.nav_mode, mode, name);
        }
    });

    section(ui, theme, "Page transition");
    ComboBox::new("transition", TRANSITIONS[gallery.transition].1).show_ui(ui, |ui| {
        for (i, (_, name)) in TRANSITIONS.iter().enumerate() {
            combo_value(ui, &mut gallery.transition, i, *name);
        }
    });

    section(ui, theme, "Language of built-in strings");
    let before = gallery.locale;
    ComboBox::new("locale", l10n::ALL[gallery.locale].locale)
        .width(120.0)
        .show_ui(ui, |ui| {
            for (i, l) in l10n::ALL.iter().enumerate() {
                combo_value(ui, &mut gallery.locale, i, l.locale);
            }
        });
    changed |= gallery.locale != before;
    let l10n = &l10n::ALL[gallery.locale];
    ui.label(theme.typography.caption.rich(format!(
        "{} · {} · {} · {}",
        l10n.close_button_label, l10n.pick_a_date, l10n.no_results_found_label, l10n.new_tab_label
    )));
    changed
}

const TRANSITIONS: [(Option<PageTransition>, &str); 5] = [
    (None, "Default (entrance, slide in top mode)"),
    (
        Some(PageTransition {
            kind: PageTransitionKind::Entrance {
                vertical: true,
                reverse: false,
                start_from: 0.25,
            },
            duration: None,
        }),
        "Entrance",
    ),
    (
        Some(PageTransition {
            kind: PageTransitionKind::DrillIn,
            duration: None,
        }),
        "Drill in",
    ),
    (
        Some(PageTransition {
            kind: PageTransitionKind::HorizontalSlide { from_left: false },
            duration: None,
        }),
        "Horizontal slide",
    ),
    (
        Some(PageTransition {
            kind: PageTransitionKind::Suppress,
            duration: None,
        }),
        "Suppress",
    ),
];

fn navigation_page(ui: &mut egui::Ui, theme: &FluentTheme, demo: &mut Demo) {
    section(ui, theme, "NavigationView");
    ui.label("This gallery is a NavigationView. Change its display mode under Settings.");

    section(
        ui,
        theme,
        "TabView (drag tabs to reorder, middle-click to close)",
    );
    let tabs = demo
        .tabs
        .iter()
        .map(|(id, title)| Tab::new(title.as_str()).icon("📄").id_salt(*id));
    let response = TabView::new("demo_tabs").min_body_height(120.0).show(
        ui,
        &mut demo.current_tab,
        tabs,
        |ui, i| {
            egui::Frame::NONE
                .inner_margin(16)
                .show(ui, |ui| match demo.tabs.get(i) {
                    Some((_, title)) => ui.label(format!("Content of {title}")),
                    None => ui.label("No tabs open. Press + to add one."),
                });
        },
    );
    if let Some(i) = response.closed {
        demo.tabs.remove(i);
        if demo.current_tab > i || demo.current_tab >= demo.tabs.len() {
            demo.current_tab = demo.current_tab.saturating_sub(1);
        }
    }
    if let Some((from, to)) = response.moved {
        let tab = demo.tabs.remove(from);
        demo.tabs.insert(to, tab);
    }
    if response.added {
        demo.tabs
            .push((demo.next_tab, format!("Document {}", demo.next_tab)));
        demo.next_tab += 1;
        demo.current_tab = demo.tabs.len() - 1;
    }

    section(ui, theme, "BreadcrumbBar (click a crumb to go back)");
    if let Some(i) = BreadcrumbBar::new(demo.path.iter().copied()).show(ui) {
        demo.path.truncate(i + 1);
    }
    if demo.path.len() < BREADCRUMB.len() && ui.add(Button::new("Reset path")).clicked() {
        demo.path = BREADCRUMB.to_vec();
    }
    ui.add_space(8.0);
    egui::Frame::NONE.show(ui, |ui| {
        ui.set_max_width(260.0);
        BreadcrumbBar::new(BREADCRUMB).show(ui);
    });
    ui.label(
        theme
            .typography
            .caption
            .rich("The same path in 260px collapses into a menu."),
    );

    section(ui, theme, "TreeView");
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(280.0);
            ui.label(theme.typography.caption.rich("Single selection"));
            TreeView::new("tree_single").show(ui, &file_tree(), &mut demo.tree_single);
        });
        ui.vertical(|ui| {
            ui.set_width(280.0);
            ui.label(theme.typography.caption.rich("Multiple selection"));
            TreeView::new("tree_multiple")
                .selection_mode(TreeViewSelectionMode::Multiple)
                .show(ui, &file_tree(), &mut demo.tree_multiple);
        });
    });
}

const BREADCRUMB: [&str; 5] = ["Home", "Documents", "Design", "Northwind", "Specs"];

fn file_tree() -> Vec<TreeViewItem<&'static str>> {
    vec![
        TreeViewItem::new("work", "Work Documents")
            .leading("📁")
            .expanded(true)
            .children([
                TreeViewItem::new("plan", "XYZ Functional Spec").leading("📄"),
                TreeViewItem::new("feature", "Feature Schedule").leading("📄"),
                TreeViewItem::new("overview", "Overview").leading("📄"),
            ]),
        TreeViewItem::new("personal", "Personal Documents")
            .leading("📁")
            .expanded(true)
            .children([
                TreeViewItem::new("home", "Home Remodel")
                    .leading("📁")
                    .children([
                        TreeViewItem::new("contractor", "Contractor Contact Info").leading("📄"),
                        TreeViewItem::new("paint", "Paint Color Scheme").leading("📄"),
                    ]),
                TreeViewItem::new("cv.pdf", "CV.pdf").leading("📄"),
                TreeViewItem::new("vacation.jpg", "vacation.jpg").leading("🖼"),
            ]),
    ]
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

    section(ui, theme, "Rating");
    ui.horizontal(|ui| {
        ui.add_enabled(enabled, Rating::new(&mut demo.rating));
        ui.label(format!("{:.1}", demo.rating));
    });
}

fn progress_page(ui: &mut egui::Ui, theme: &FluentTheme, demo: &mut Demo) {
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

    section(ui, theme, "AutoSuggestBox");
    let out = ui
        .add_enabled_ui(enabled, |ui| {
            AutoSuggestBox::new("fruit", &mut demo.query, &FRUITS)
                .placeholder("Search fruits")
                .width(260.0)
                .show(ui)
        })
        .inner;
    if let Some(i) = out.chosen {
        demo.last_action = format!("suggestion {}", FRUITS[i]);
    }
}

fn menus_page(ui: &mut egui::Ui, theme: &FluentTheme, demo: &mut Demo) {
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

/// Acrylic cards over a colourful backdrop, beside a Mica panel.
fn materials(ui: &mut egui::Ui, theme: &FluentTheme, width: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 170.0), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        8.0,
        theme.resources.card_background_fill_color_default,
    );
    for (i, color) in [
        colors::ORANGE.normal,
        colors::BLUE.normal,
        colors::MAGENTA.normal,
        colors::GREEN.normal,
    ]
    .into_iter()
    .enumerate()
    {
        let x = rect.left() + 60.0 + i as f32 * (width - 120.0) / 3.0;
        let y = rect.center().y + if i % 2 == 0 { -30.0 } else { 30.0 };
        painter.circle_filled(egui::pos2(x, y), 46.0, color);
    }
    painter.text(
        rect.left_bottom() + egui::vec2(12.0, -10.0),
        egui::Align2::LEFT_BOTTOM,
        "backdrop",
        theme.typography.caption.font_id(),
        theme.resources.text_fill_color_secondary,
    );

    let card = |ui: &mut egui::Ui, x: f32, acrylic: Acrylic, title: &str, body: &str| {
        let area = egui::Rect::from_min_size(
            egui::pos2(rect.left() + x, rect.top() + 24.0),
            egui::vec2(width / 2.0 - 36.0, 0.0),
        );
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(area));
        acrylic.show(&mut child, |ui| {
            ui.set_width(ui.available_width());
            ui.label(theme.typography.body_strong.rich(title));
            ui.label(body);
        });
    };
    card(
        ui,
        24.0,
        Acrylic::new().corner_radius(8).elevation(8.0),
        "Acrylic",
        "Tint and luminosity layers with grain.",
    );
    card(
        ui,
        width / 2.0 + 12.0,
        Acrylic::new()
            .corner_radius(8)
            .elevation(8.0)
            .disabled(true),
        "Acrylic, disabled",
        "The opaque fallback colour.",
    );

    ui.add_space(8.0);
    Mica::new()
        .corner_radius(8)
        .inner_margin(12)
        .show(ui, |ui| {
            ui.set_width(width - 24.0);
            ui.label(theme.typography.body_strong.rich("Mica"));
            ui.label("The window backdrop colour; this whole gallery sits on it.");
        });
}

fn surfaces_page(ui: &mut egui::Ui, theme: &FluentTheme, demo: &mut Demo) {
    let width = ui.available_width().min(560.0);
    ui.allocate_ui(egui::vec2(width, 0.0), |ui| {
        section(ui, theme, "Card");
        Card::new().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(theme.typography.body_strong.rich("Card title"));
            ui.label("Cards group related content on a surface with a subtle border.");
        });

        section(ui, theme, "Acrylic and Mica");
        materials(ui, theme, width);

        section(ui, theme, "Expander");
        Expander::new("expander", "Advanced settings").show(ui, |ui| {
            ui.add(Checkbox::new(&mut demo.checked, "Enable telemetry"));
            ui.add(ToggleSwitch::new(&mut demo.switch, "Auto update"));
        });
        ui.add_space(8.0);
        Expander::new("expander2", "Open by default").initially_open(true).show(ui, |ui| {
            ui.label("This expander starts open.");
        });

        section(ui, theme, "InfoBar");
        if demo.info_visible {
            let bar = InfoBar::new("Update available")
                .message("Restart the app to apply it.")
                .action("Restart")
                .closable(true)
                .show(ui);
            if bar.closed {
                demo.info_visible = false;
            }
            if bar.action_clicked {
                demo.last_action = "restart from InfoBar".to_owned();
            }
        } else if ui.add(Button::hyperlink("Show the closable InfoBar again")).clicked() {
            demo.info_visible = true;
        }
        for (severity, title, message) in [
            (Severity::Warning, "Low disk space", "Free some space soon."),
            (Severity::Success, "Saved", "All changes are on disk."),
            (Severity::Error, "Sync failed", "Check your connection."),
        ] {
            ui.add_space(8.0);
            InfoBar::new(title).message(message).severity(severity).show(ui);
        }
        ui.add_space(8.0);
        InfoBar::new("Long InfoBar")
            .message("Title, message and action stack vertically when the bar is long, which suits longer explanations like this one.")
            .long(true)
            .action("Learn more")
            .show(ui);
    });

    section(ui, theme, "ListTile");
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(300.0);
            ui.label(theme.typography.caption.rich("Single selection"));
            for (i, (icon, title, subtitle)) in [
                ("📥", "Inbox", "12 unread"),
                ("📤", "Sent", "Last sent yesterday"),
                ("🗑", "Deleted", "Empty"),
            ]
            .into_iter()
            .enumerate()
            {
                let tile = ListTile::new(title)
                    .subtitle(subtitle)
                    .leading(icon)
                    .selection_mode(ListTileSelectionMode::Single)
                    .selected(demo.list_selected == i);
                if ui.add(tile).clicked() {
                    demo.list_selected = i;
                }
            }
        });
        ui.vertical(|ui| {
            ui.set_width(300.0);
            ui.label(theme.typography.caption.rich("Multiple selection"));
            for (i, title) in ["Wi-Fi", "Bluetooth", "Airplane mode"]
                .into_iter()
                .enumerate()
            {
                let tile = ListTile::new(title)
                    .trailing(if demo.list_checked[i] { "On" } else { "Off" })
                    .selection_mode(ListTileSelectionMode::Multiple)
                    .selected(demo.list_checked[i]);
                if ui.add(tile).clicked() {
                    demo.list_checked[i] = !demo.list_checked[i];
                }
            }
        });
    });
}

fn flyouts_page(ui: &mut egui::Ui, theme: &FluentTheme, demo: &mut Demo) {
    section(ui, theme, "MenuBar");
    MenuBar::new().show(ui, |bar| {
        bar.menu("File", |ui| {
            for (item, shortcut) in [("New", "Ctrl+N"), ("Open", "Ctrl+O"), ("Save", "Ctrl+S")] {
                if ui.add(MenuItem::new(item).shortcut(shortcut)).clicked() {
                    demo.last_action = format!("File > {item}");
                }
            }
            menu_separator(ui);
            if ui.add(MenuItem::new("Exit")).clicked() {
                demo.last_action = "File > Exit".to_owned();
            }
        });
        bar.menu("Edit", |ui| {
            for item in ["Undo", "Cut", "Copy", "Paste"] {
                if ui.add(MenuItem::new(item)).clicked() {
                    demo.last_action = format!("Edit > {item}");
                }
            }
            menu_separator(ui);
            sub_menu(ui, "Transform", |ui| {
                for item in ["Uppercase", "Lowercase", "Title case"] {
                    if ui.add(MenuItem::new(item)).clicked() {
                        demo.last_action = format!("Edit > Transform > {item}");
                    }
                }
            });
        });
        bar.menu("Help", |ui| {
            if ui.add(MenuItem::new("About")).clicked() {
                demo.last_action = "Help > About".to_owned();
            }
        });
    });

    section(ui, theme, "CommandBar");
    // Both closures report through a Cell, since they can't both borrow demo.
    let picked = std::cell::Cell::new(None::<String>);
    CommandBar::new().show_with_overflow(
        ui,
        |bar| {
            for (icon, label) in [("➕", "New"), ("✏", "Edit"), ("🗑", "Delete")] {
                if bar.button(icon, label).clicked() {
                    picked.set(Some(format!("command {label}")));
                }
            }
            bar.separator();
            if bar.icon_button("🔗").fluent_tooltip("Share").clicked() {
                picked.set(Some("command Share".to_owned()));
            }
        },
        |ui| {
            for item in ["Settings", "Help"] {
                if ui.add(MenuItem::new(item)).clicked() {
                    picked.set(Some(format!("more > {item}")));
                }
            }
        },
    );
    if let Some(action) = picked.take() {
        demo.last_action = action;
    }

    section(ui, theme, "Context menu (right-click the card)");
    let card = Card::new().show(ui, |ui| {
        ui.set_width(300.0);
        ui.label("Right-click here");
    });
    let card = ui.interact(
        card.response.rect,
        ui.scope_id().with("ctx_card"),
        egui::Sense::click(),
    );
    context_menu(&card, |ui| {
        for (item, shortcut) in [("Cut", "Ctrl+X"), ("Copy", "Ctrl+C"), ("Paste", "Ctrl+V")] {
            if ui.add(MenuItem::new(item).shortcut(shortcut)).clicked() {
                demo.last_action = format!("context > {item}");
            }
        }
        menu_separator(ui);
        sub_menu(ui, "Sort by", |ui| {
            for item in ["Name", "Date modified", "Size"] {
                if ui.add(MenuItem::new(item)).clicked() {
                    demo.last_action = format!("context > Sort by > {item}");
                }
            }
        });
    });

    section(ui, theme, "Tooltip, ContentDialog, TeachingTip");
    ui.horizontal(|ui| {
        ui.add(Button::new("Hover me"))
            .fluent_tooltip("A Fluent tooltip");
        if ui.add(Button::filled("Open dialog")).clicked() {
            demo.dialog_open = true;
        }
        let tip_target = ui.add(Button::new("Show teaching tip"));
        if tip_target.clicked() {
            demo.tip_open = true;
        }
        let tip = TeachingTip::new("Teaching tip")
            .subtitle("Point people at features they may have missed.")
            .action("Got it")
            .show(&tip_target, &mut demo.tip_open);
        if tip.action_clicked {
            demo.last_action = "teaching tip action".to_owned();
        }
    });
    ui.add_space(8.0);
    ui.label(
        theme
            .typography
            .caption
            .rich(format!("Last action: {}", demo.last_action)),
    );

    if demo.dialog_open {
        let result = ContentDialog::new("demo_dialog", "Delete file permanently?")
            .body("If you delete this file, you won't be able to recover it. Do you want to delete it?")
            .primary_button("Delete")
            .secondary_button("Move to bin")
            .close_button("Cancel")
            .show(ui.ctx());
        if let Some(result) = result {
            demo.dialog_open = false;
            demo.last_action = match result {
                DialogResult::Primary => "dialog: delete",
                DialogResult::Secondary => "dialog: move to bin",
                DialogResult::Close => "dialog: cancel",
            }
            .to_owned();
        }
    }
}
