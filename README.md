# fluent_egui

Fluent Design widgets for [egui](https://github.com/emilk/egui), ported from the
Flutter package [fluent_ui](https://github.com/bdlukaa/fluent_ui).

Status: early. Done so far: the theme (WinUI color resources, accent colors,
Windows 11 type ramp), icon codepoints, and the basic controls: `Button`
(standard, filled, outlined, hyperlink, icon, toggle), `Checkbox`,
`RadioButton`, `ToggleSwitch`, `Slider`, `ProgressBar`, `ProgressRing`,
`InfoBadge`, `InfoLabel` and `Divider`, and the input controls: `TextBox`,
`PasswordBox`, `NumberBox`, `DropDownButton`, `SplitButton` and `ComboBox`.

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
| `tools/codegen` | Regenerates `theme/resources.rs` and `icons.rs` from a fluent_ui checkout. |

Regenerating:

```sh
git clone --depth 1 https://github.com/bdlukaa/fluent_ui /tmp/fluent_ui
cargo run -p codegen -- /tmp/fluent_ui
```

## Fonts

No fonts are bundled. Segoe UI Variable and the icon fonts used by fluent_ui are
not freely redistributable, so register the ones you may ship with
`fluent_egui::fonts`. The gallery picks up icon fonts from the `FLUENT_ICONS_TTF`
and `SEGOE_ICONS_TTF` environment variables.

## Porting roadmap

1. ✅ Foundation: theme, palette, typography, icons, focus border, Divider, gallery.
2. ✅ Basic controls: Button, FilledButton, OutlinedButton, IconButton, HyperlinkButton,
   ToggleButton, Checkbox, RadioButton, ToggleSwitch, Slider, ProgressBar/Ring,
   InfoBadge, InfoLabel.
3. ✅ Input: TextBox, PasswordBox, NumberBox, DropDownButton, SplitButton, ComboBox,
   Rating, AutoSuggestBox. (Rating and AutoSuggestBox are still to do.)
4. Surfaces and flyouts: Card, Expander, ListTile, InfoBar, Tooltip, Flyout,
   MenuFlyout, ContentDialog, CommandBar, MenuBar, TeachingTip.
5. Navigation: ScaffoldPage, NavigationView, TabView, BreadcrumbBar, TreeView.
6. Pickers: DatePicker, TimePicker, CalendarView, ColorPicker.
7. Extras: Acrylic/Mica, page transitions, DynamicOverflow, localization.

## License

BSD-3-Clause, like fluent_ui. See [LICENSE](LICENSE).
