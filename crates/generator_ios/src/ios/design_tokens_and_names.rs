include!("design_tokens_variant_palette.rs");

fn color_ref(value: ColorToken) -> &'static str {
    match value.as_str() {
        "white" => "Color.white",
        "black" => "Color.black",
        "transparent" => "Color.clear",
        _ => intern_generated_color_name(format!("DoweDesign.{}", swift_color_member(value))),
    }
}

fn swift_color_member(value: ColorToken) -> &'static str {
    value.as_str()
}

fn intern_generated_color_name(value: String) -> &'static str {
    static NAMES: OnceLock<Mutex<BTreeSet<&'static str>>> = OnceLock::new();
    let names = NAMES.get_or_init(|| Mutex::new(BTreeSet::new()));
    let mut names = names.lock().expect("generated color name registry");
    if let Some(existing) = names.get(value.as_str()) {
        return existing;
    }
    let value = Box::leak(value.into_boxed_str());
    names.insert(value);
    value
}

fn swift_design_block(design: &DesignConfig) -> String {
    let theme = design.default_theme();
    let mut output = String::from(
        "@MainActor\nfinal class DoweDesign: ObservableObject {\n    static let shared = DoweDesign()\n    @Published private(set) var name: String\n    @Published private var colors: [String: Color]\n    @Published private var currentRadius: CGFloat\n",
    );
    output.push_str(&format!(
        "    private init() {{\n        let stored = UserDefaults.standard.string(forKey: \"theme-preference\") ?? \"{}\"\n        let theme = DoweThemeModule.themes.first(where: {{ $0.name == stored }}) ?? DoweThemeModule.themes.first(where: {{ $0.name == DoweThemeModule.defaultTheme }})!\n        name = theme.name\n        colors = theme.colors\n        currentRadius = theme.radius\n    }}\n    static func applyTheme(_ name: String) {{\n        guard let theme = DoweThemeModule.themes.first(where: {{ $0.name == name }}) else {{ return }}\n        UserDefaults.standard.set(theme.name, forKey: \"theme-preference\")\n        shared.name = theme.name\n        shared.colors = theme.colors\n        shared.currentRadius = theme.radius\n    }}\n",
        escape_swift(&design.default_theme)
    ));
    for token in theme.ordered_color_tokens() {
        output.push_str(&format!(
            "    static var {}: Color {{ shared.colors[\"{}\"] ?? {} }}\n",
            token.as_str(),
            token.as_str(),
            swift_color_literal(theme.color_value(token))
        ));
    }
    output.push_str("    static var radius: CGFloat { shared.currentRadius }\n}\n");
    output
}

fn swift_theme_module(design_config: &DesignConfig) -> String {
    let names = design_config
        .themes
        .iter()
        .map(|theme| format!("        \"{}\",", escape_swift(&theme.name)))
        .collect::<Vec<_>>()
        .join("\n");
    let themes = design_config
        .themes
        .iter()
        .map(swift_theme_record)
        .collect::<Vec<_>>()
        .join("\n");
    let select_options = design_config
        .themes
        .iter()
        .map(|theme| {
            format!(
                "        DoweSelectOption(value: {}, label: {}, description: nil),",
                swift_string_literal(&theme.name),
                swift_string_literal(&theme_display_label(&theme.name))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"import SwiftUI

struct DoweGeneratedTheme {{
    let name: String
    let colors: [String: Color]
    let radius: CGFloat
}}

enum DoweThemeModule {{
    static let generated = true
    static let defaultTheme = "{}"
    static let names = [
{}
    ]
    static let selectOptions: [DoweSelectOption] = [
{}
    ]
    static let themes = [
{}
    ]
}}
"#,
        escape_swift(&design_config.default_theme),
        names,
        select_options,
        themes
    )
}

fn theme_display_label(value: &str) -> String {
    value
        .split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn swift_theme_record(theme: &DesignTheme) -> String {
    let colors = theme
        .ordered_color_tokens()
        .into_iter()
        .map(|token| {
            format!(
                "            \"{}\": {},",
                token.as_str(),
                swift_color_literal(theme.color_value(token))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "        DoweGeneratedTheme(name: \"{}\", colors: [\n{}\n        ], radius: CGFloat({})),",
        escape_swift(&theme.name),
        colors,
        theme.radius
    )
}

fn swift_color_literal(value: &str) -> String {
    let (red, green, blue, alpha) = hex_components(value);
    let color = format!(
        "Color(red: {:.3}, green: {:.3}, blue: {:.3})",
        red as f32 / 255.0,
        green as f32 / 255.0,
        blue as f32 / 255.0
    );
    if alpha == 255 {
        color
    } else {
        format!("{color}.opacity({:.3})", alpha as f32 / 255.0)
    }
}

fn hex_components(value: &str) -> (u8, u8, u8, u8) {
    let raw = value.trim_start_matches('#');
    let red = u8::from_str_radix(&raw[0..2], 16).expect("red");
    let green = u8::from_str_radix(&raw[2..4], 16).expect("green");
    let blue = u8::from_str_radix(&raw[4..6], 16).expect("blue");
    let alpha = if raw.len() == 8 {
        u8::from_str_radix(&raw[6..8], 16).expect("alpha")
    } else {
        255
    };
    (red, green, blue, alpha)
}

fn family_color(value: ColorFamily) -> ColorToken {
    value.color_token()
}

fn family_text_color(value: ColorFamily) -> ColorToken {
    value.text_token()
}

fn family_title_color(value: ColorFamily) -> ColorToken {
    value.title_token()
}

fn swift_view_name(route: &str) -> String {
    format!("{}View", pascal_route(route))
}

fn pascal_route(route: &str) -> String {
    let mut name = String::new();

    for segment in route.split(|value: char| !value.is_ascii_alphanumeric()) {
        if segment.is_empty() {
            continue;
        }

        let mut chars = segment.chars();
        if let Some(first) = chars.next() {
            name.push(first.to_ascii_uppercase());
            for value in chars {
                name.push(value.to_ascii_lowercase());
            }
        }
    }

    if name.is_empty() {
        "Index".to_string()
    } else {
        name
    }
}

fn parse_rgba(value: &str) -> Option<(u16, u16, u16, String)> {
    let inner = value.strip_prefix("rgba(")?.strip_suffix(')')?;
    let parts = inner.split(',').map(str::trim).collect::<Vec<_>>();
    if parts.len() != 4 {
        return None;
    }
    let red = parts[0].parse::<u16>().ok()?;
    let green = parts[1].parse::<u16>().ok()?;
    let blue = parts[2].parse::<u16>().ok()?;
    if red > 255 || green > 255 || blue > 255 {
        return None;
    }
    Some((red, green, blue, parts[3].to_string()))
}

fn gradient_colors(value: &str) -> (&str, &str) {
    let colors = value
        .split("rgba(")
        .skip(1)
        .filter_map(|part| part.split_once(')').map(|(color, _)| color))
        .map(|color| format!("rgba({color})"))
        .collect::<Vec<_>>();
    if colors.len() >= 2 {
        let first = Box::leak(colors[0].clone().into_boxed_str());
        let second = Box::leak(colors[1].clone().into_boxed_str());
        (first, second)
    } else {
        ("rgba(0,0,0,0.2)", "rgba(0,0,0,0.6)")
    }
}

fn escape_swift(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
