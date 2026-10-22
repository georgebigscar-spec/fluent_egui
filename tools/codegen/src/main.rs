//! Generates Rust sources from a checkout of the Flutter `fluent_ui` package.
//!
//! Usage: `cargo run -p codegen -- <path to fluent_ui checkout> [output dir]`
//!
//! The output dir defaults to `crates/fluent_egui/src`. Generated files:
//!   * `theme/resources.rs`, from `lib/src/styles/color_resources.dart`
//!   * `icons.rs`, from `lib/src/icons.dart` and `lib/src/windows_icons.dart`

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some(source) = args.first().map(PathBuf::from) else {
        eprintln!("usage: codegen <fluent_ui checkout> [output dir]");
        process::exit(2);
    };
    let out = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("crates/fluent_egui/src"));

    let resources = read(&source.join("lib/src/styles/color_resources.dart"));
    write(&out.join("theme/resources.rs"), &gen_resources(&resources));

    let fluent = read(&source.join("lib/src/icons.dart"));
    let windows = read(&source.join("lib/src/windows_icons.dart"));
    write(&out.join("icons.rs"), &gen_icons(&fluent, &windows));
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("cannot read {}: {e}", path.display());
        process::exit(1);
    })
}

fn write(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap_or_else(|e| {
        eprintln!("cannot write {}: {e}", path.display());
        process::exit(1);
    });
    println!("wrote {}", path.display());
}

const HEADER: &str = "// GENERATED FILE, DO NOT EDIT. Regenerate with `cargo run -p codegen`.\n";

fn gen_resources(dart: &str) -> String {
    let fields: Vec<&str> = dart
        .lines()
        .filter_map(|l| l.trim().strip_prefix("final Color "))
        .filter_map(|l| l.strip_suffix(';'))
        .collect();
    let light = parse_colors(constructor_body(dart, "const ResourceDictionary.light({"));
    let dark = parse_colors(constructor_body(dart, "const ResourceDictionary.dark({"));

    let mut s = String::from(HEADER);
    s.push_str(
        "// Source: fluent_ui lib/src/styles/color_resources.dart, which mirrors\n\
         // https://github.com/microsoft/microsoft-ui-xaml/blob/main/dev/CommonStyles/Common_themeresources_any.xaml\n\n\
         use egui::Color32;\n\n\
         /// The colors used by all the components, for light or dark mode.\n\
         #[derive(Clone, Copy, Debug, PartialEq)]\n\
         pub struct ResourceDictionary {\n",
    );
    for f in &fields {
        writeln!(s, "    pub {}: Color32,", snake_case(f)).unwrap();
    }
    s.push_str(
        "}\n\n\
         const fn c(r: u8, g: u8, b: u8, a: u8) -> Color32 {\n    \
         Color32::from_rgba_unmultiplied_const(r, g, b, a)\n}\n\n\
         #[rustfmt::skip]\nimpl ResourceDictionary {\n",
    );
    for (name, colors) in [("LIGHT", &light), ("DARK", &dark)] {
        writeln!(s, "    pub const {name}: Self = Self {{").unwrap();
        for f in &fields {
            let argb = colors
                .iter()
                .find(|(n, _)| n == f)
                .unwrap_or_else(|| panic!("{name} has no value for {f}"))
                .1;
            writeln!(s, "        {}: {},", snake_case(f), color32(argb)).unwrap();
        }
        s.push_str("    };\n\n");
    }
    s.truncate(s.len() - 1);
    s.push_str("}\n");
    s
}

fn constructor_body<'a>(dart: &'a str, start: &str) -> &'a str {
    let from = dart.find(start).expect("constructor not found") + start.len();
    let len = dart[from..].find("});").expect("unterminated constructor");
    &dart[from..from + len]
}

fn parse_colors(body: &str) -> Vec<(String, u32)> {
    body.lines()
        .filter_map(|l| {
            let l = l.trim().strip_prefix("this.")?;
            let (name, rest) = l.split_once(" = const Color(0x")?;
            let hex = rest.split(')').next()?;
            Some((name.to_owned(), u32::from_str_radix(hex, 16).ok()?))
        })
        .collect()
}

/// Dart colors are `0xAARRGGBB`.
fn color32(argb: u32) -> String {
    let [a, r, g, b] = argb.to_be_bytes();
    format!("c(0x{r:02x}, 0x{g:02x}, 0x{b:02x}, 0x{a:02x})")
}

fn gen_icons(fluent: &str, windows: &str) -> String {
    let mut s = String::from(HEADER);
    s.push_str(
        "// Source: fluent_ui lib/src/icons.dart and lib/src/windows_icons.dart\n\n\
         //! Icon codepoints. Render them with a font registered under\n\
         //! [`FLUENT_ICONS_FAMILY`] or [`WINDOWS_ICONS_FAMILY`], see [`crate::fonts`].\n\n\
         /// Font family name for [`fluent`] icons.\n\
         pub const FLUENT_ICONS_FAMILY: &str = \"FluentIcons\";\n\
         /// Font family name for [`windows`] icons.\n\
         pub const WINDOWS_ICONS_FAMILY: &str = \"SegoeIcons\";\n",
    );
    for (module, doc, dart) in [
        (
            "fluent",
            "Fluent (Fabric MDL2) icons, `FluentIcons` in fluent_ui.",
            fluent,
        ),
        (
            "windows",
            "Segoe Fluent icons, `WindowsIcons` in fluent_ui.",
            windows,
        ),
    ] {
        let icons = parse_icons(dart);
        writeln!(s, "\n/// {doc}\n#[rustfmt::skip]\npub mod {module} {{").unwrap();
        for (name, cp) in &icons {
            writeln!(
                s,
                "    pub const {}: char = '\\u{{{cp:x}}}';",
                name.to_uppercase()
            )
            .unwrap();
        }
        s.push_str("\n    /// Every icon with its name, for search and galleries.\n");
        s.push_str("    pub const ALL: &[(&str, char)] = &[\n");
        for (name, _) in &icons {
            writeln!(s, "        (\"{name}\", {}),", name.to_uppercase()).unwrap();
        }
        s.push_str("    ];\n}\n");
    }
    s
}

fn parse_icons(dart: &str) -> Vec<(String, u32)> {
    let marker = "static const IconData ";
    dart.match_indices(marker)
        .map(|(i, _)| {
            let rest = &dart[i + marker.len()..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            let hex = &rest[rest.find("0x").expect("icon without codepoint") + 2..];
            let hex: String = hex.chars().take_while(char::is_ascii_hexdigit).collect();
            (name, u32::from_str_radix(&hex, 16).unwrap())
        })
        .collect()
}

fn snake_case(camel: &str) -> String {
    let mut s = String::with_capacity(camel.len() + 8);
    for c in camel.chars() {
        if c.is_ascii_uppercase() {
            s.push('_');
            s.push(c.to_ascii_lowercase());
        } else {
            s.push(c);
        }
    }
    s
}
