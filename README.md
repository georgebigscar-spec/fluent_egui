# fluent_egui

Fluent Design widgets for [egui](https://github.com/emilk/egui), ported from the
Flutter package [fluent_ui](https://github.com/bdlukaa/fluent_ui).

Status: early. Done so far: the theme (WinUI color resources, accent colors,
Windows 11 type ramp), icon codepoints, and the basic controls: `Button`
(standard, filled, outlined, hyperlink, icon, toggle), `Checkbox`,
`RadioButton`, `ToggleSwitch`, `Slider`, `ProgressBar`, `ProgressRing`,
`InfoBadge`, `InfoLabel` and `Divider`, and the input controls: `TextBox`,
`PasswordBox`, `NumberBox`, `DropDownButton`, `SplitButton` and `ComboBox`,
and the surfaces and flyouts: `Card`, `Expander`, `InfoBar`, tooltips,
`MenuFlyout`/context menus, `ContentDialog`, `MenuBar`, `CommandBar` and
`TeachingTip`, and navigation: `NavigationView` (open, compact, minimal and
top panes), `ScaffoldPage`, `TabView`, `BreadcrumbBar` and `TreeView`, and the
pickers: `CalendarView`, `CalendarDatePicker`, `DatePicker`, `TimePicker` and
`ColorPicker`, plus `Rating`, `AutoSuggestBox`, `ListTile`, menu submenus,
`Acrylic`/`Mica`, page transitions and localized built-in strings in 41
languages. Beyond fluent_ui there is a `DataTable` in the spirit of TanStack
Table and shadcn/ui's data table. Dates and times use the crate's own small `Date` and `Time`
types, so no date library is pulled in.

```rust
use fluent_egui::{Brightness, FluentTheme};

// Once, e.g. in your eframe app's constructor:
FluentTheme::new(Brightness::Dark).install(ctx);

// In your UI:
let theme = FluentTheme::of(ui.ctx());
ui.label(theme.typography.title.rich("Hello, Fluent"));
if ui.add(fluent_egui::Button::filled("Save")).clicked() { /* ... */ }
ui.add(fluent_egui::ToggleSwitch::new(&mut dark_mode, "Dark mode"));
```

`install` also restyles egui's built-in widgets so they blend in with the
Fluent ones.

## Layout

| Path | What |
| --- | --- |
| `crates/fluent_egui` | The library. `theme/` mirrors fluent_ui's `lib/src/styles`, `widgets/` mirrors `lib/src/controls`. |
| `examples/gallery` | Demo app, `cargo run -p gallery`. |
| `tools/codegen` | Regenerates `theme/resources.rs`, `icons.rs` and `l10n/strings.rs` from a fluent_ui checkout. |

Regenerating:

```sh
git clone --depth 1 https://github.com/bdlukaa/fluent_ui /tmp/fluent_ui
cargo run -p codegen -- /tmp/fluent_ui
```

## Fonts

Segoe UI Variable and the icon fonts used by fluent_ui are not freely
redistributable, so register the ones you may ship with `fluent_egui::fonts`.
The default `selawik` feature bundles Selawik Regular and Semibold (SIL OFL
1.1, see `crates/fluent_egui/assets/fonts`), Microsoft's open Segoe UI
fallback, so headings get their semibold weight:

```rust
let mut defs = egui::FontDefinitions::default();
fluent_egui::fonts::add_selawik(&mut defs);
ctx.set_fonts(defs);
FluentTheme::new(Brightness::Light).with_semibold_font().install(ctx);
```

Selawik covers Latin only, so Cyrillic and other scripts fall back to egui's
default font in regular weight. The gallery picks up icon fonts from the `FLUENT_ICONS_TTF`
and `SEGOE_ICONS_TTF` environment variables.

## Porting roadmap

1. ✅ Foundation: theme, palette, typography, icons, focus border, Divider, gallery.
2. ✅ Basic controls: Button, FilledButton, OutlinedButton, IconButton, HyperlinkButton,
   ToggleButton, Checkbox, RadioButton, ToggleSwitch, Slider, ProgressBar/Ring,
   InfoBadge, InfoLabel.
3. ✅ Input: TextBox, PasswordBox, NumberBox, DropDownButton, SplitButton, ComboBox,
   Rating, AutoSuggestBox.
4. ✅ Surfaces and flyouts: Card, Expander, ListTile, InfoBar, Tooltip, Flyout,
   MenuFlyout (with submenus), ContentDialog, CommandBar, MenuBar, TeachingTip.
5. ✅ Navigation: ScaffoldPage, NavigationView, TabView, BreadcrumbBar, TreeView.
   Nested pane items, a search box in the pane and the TabView keyboard
   shortcuts (Ctrl+T, Ctrl+W, Ctrl+Tab, Ctrl+1 to 9) are in.
6. ✅ Pickers: DatePicker, TimePicker, CalendarView (single, multiple and
   range selection), ColorPicker.
7. ✅ Extras: Acrylic/Mica, page transitions (Entrance, DrillIn,
   HorizontalSlide, Suppress, used by NavigationView), localization
   (`Localizations`, generated from fluent_ui's ARB files). egui has no
   backdrop blur, so Acrylic shows the backdrop unblurred under its luminosity
   layer, and Mica is the solid base colour. DynamicOverflow, localized month
   and weekday names (29 languages) and the first day of the week per locale
   are in.
8. ✅ Polish: clear button in text fields, Slider value tooltip and vertical
   slider, TeachingTip tail, bundled Selawik Semibold for headings.
9. ✅ DataTable (not in fluent_ui): column definitions with formatting and
   custom cells, multi-column sorting, search, text and faceted filters,
   pagination with a page size box, row selection with checkboxes, row action
   menus, column visibility, column resizing, horizontal and vertical scrolling,
   and drag and drop to reorder rows and columns.

## Localization

```rust
fluent_egui::Localizations::for_locale("ru-RU").install(ctx);
```

Widgets then use the Russian strings for tooltips, placeholders and labels.
Missing translations fall back to English.

## License

BSD-3-Clause, like fluent_ui. See [LICENSE](LICENSE).
