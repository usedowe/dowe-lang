fn variant_container(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => color_ref(family_color(color)),
        ComponentVariant::Soft => color_ref(family_soft_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            "Color.clear"
        }
    }
}

fn variant_content(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => color_ref(family_on_color(color)),
        ComponentVariant::Soft => color_ref(family_on_soft_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            color_ref(family_color(color))
        }
    }
}

fn nav_active_content(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Ghost) {
        ComponentVariant::Solid => color_ref(family_on_color(color)),
        ComponentVariant::Soft => color_ref(family_on_soft_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost
            if matches!(color, ColorFamily::Background | ColorFamily::Surface) =>
        {
            color_ref(family_on_color(color))
        }
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            color_ref(family_color(color))
        }
    }
}

fn card_variant_container(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Outlined => match color {
            ColorFamily::Background => color_ref(ColorToken::Background),
            _ => color_ref(ColorToken::Surface),
        },
        _ => variant_container(props),
    }
}

fn card_variant_content(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Outlined => match color {
            ColorFamily::Background => color_ref(ColorToken::OnBackground),
            _ => color_ref(ColorToken::OnSurface),
        },
        ComponentVariant::Ghost if matches!(color, ColorFamily::Background | ColorFamily::Surface) => {
            color_ref(family_on_color(color))
        }
        _ => variant_content(props),
    }
}

fn tabs_list_background(props: &TabsProps) -> &'static str {
    match props.variant {
        TabsVariant::Solid | TabsVariant::Pills => color_ref(family_soft_color(props.color)),
        TabsVariant::Outlined | TabsVariant::Line | TabsVariant::Ghost => "Color.clear",
    }
}

fn tabs_list_content(props: &TabsProps) -> &'static str {
    match props.variant {
        TabsVariant::Solid | TabsVariant::Pills => color_ref(family_on_soft_color(props.color)),
        TabsVariant::Outlined | TabsVariant::Line | TabsVariant::Ghost => {
            color_ref(tabs_accent_token(props.color))
        }
    }
}

fn tabs_active_background(props: &TabsProps) -> &'static str {
    if props.color == ColorFamily::Muted {
        color_ref(family_on_color(props.color))
    } else {
        color_ref(family_color(props.color))
    }
}

fn tabs_active_content(props: &TabsProps) -> &'static str {
    if props.color == ColorFamily::Muted {
        color_ref(family_color(props.color))
    } else {
        color_ref(family_on_color(props.color))
    }
}

fn tabs_accent(props: &TabsProps) -> &'static str {
    color_ref(tabs_accent_token(props.color))
}

fn tabs_border(props: &TabsProps) -> String {
    match props.variant {
        TabsVariant::Outlined => format!("Optional({})", color_ref(ColorToken::Muted)),
        TabsVariant::Line => format!("Optional({})", tabs_accent(props)),
        TabsVariant::Solid | TabsVariant::Ghost | TabsVariant::Pills => "nil".to_string(),
    }
}

fn tabs_accent_token(value: ColorFamily) -> ColorToken {
    match value {
        ColorFamily::Muted | ColorFamily::Background | ColorFamily::Surface => {
            family_on_color(value)
        }
        _ => family_color(value),
    }
}

fn color_ref(value: ColorToken) -> &'static str {
    match value {
        ColorToken::Primary => "DoweDesign.primary",
        ColorToken::OnPrimary => "DoweDesign.onPrimary",
        ColorToken::Secondary => "DoweDesign.secondary",
        ColorToken::OnSecondary => "DoweDesign.onSecondary",
        ColorToken::Tertiary => "DoweDesign.tertiary",
        ColorToken::OnTertiary => "DoweDesign.onTertiary",
        ColorToken::Muted => "DoweDesign.muted",
        ColorToken::OnMuted => "DoweDesign.onMuted",
        ColorToken::Background => "DoweDesign.background",
        ColorToken::OnBackground => "DoweDesign.onBackground",
        ColorToken::Surface => "DoweDesign.surface",
        ColorToken::OnSurface => "DoweDesign.onSurface",
        ColorToken::Success => "DoweDesign.success",
        ColorToken::OnSuccess => "DoweDesign.onSuccess",
        ColorToken::Info => "DoweDesign.info",
        ColorToken::OnInfo => "DoweDesign.onInfo",
        ColorToken::Warning => "DoweDesign.warning",
        ColorToken::OnWarning => "DoweDesign.onWarning",
        ColorToken::Danger => "DoweDesign.danger",
        ColorToken::OnDanger => "DoweDesign.onDanger",
        ColorToken::SoftPrimary => "DoweDesign.softPrimary",
        ColorToken::OnSoftPrimary => "DoweDesign.onSoftPrimary",
        ColorToken::SoftSecondary => "DoweDesign.softSecondary",
        ColorToken::OnSoftSecondary => "DoweDesign.onSoftSecondary",
        ColorToken::SoftTertiary => "DoweDesign.softTertiary",
        ColorToken::OnSoftTertiary => "DoweDesign.onSoftTertiary",
        ColorToken::SoftMuted => "DoweDesign.softMuted",
        ColorToken::OnSoftMuted => "DoweDesign.onSoftMuted",
        ColorToken::SoftSuccess => "DoweDesign.softSuccess",
        ColorToken::OnSoftSuccess => "DoweDesign.onSoftSuccess",
        ColorToken::SoftInfo => "DoweDesign.softInfo",
        ColorToken::OnSoftInfo => "DoweDesign.onSoftInfo",
        ColorToken::SoftWarning => "DoweDesign.softWarning",
        ColorToken::OnSoftWarning => "DoweDesign.onSoftWarning",
        ColorToken::SoftDanger => "DoweDesign.softDanger",
        ColorToken::OnSoftDanger => "DoweDesign.onSoftDanger",
    }
}

fn swift_design_block(theme: &DesignTheme) -> String {
    let mut output = String::from("enum DoweDesign {\n");
    for token in ColorToken::all() {
        output.push_str(&format!(
            "    static let {} = {}\n",
            token.as_str(),
            swift_color_literal(theme.color_value(*token))
        ));
    }
    output.push_str(&format!(
        "    static let radius: CGFloat = {}\n    static let radiusBox: CGFloat = {}\n    static let radiusUi: CGFloat = {}\n}}\n",
        theme.radii.radius, theme.radii.radius_box, theme.radii.radius_ui
    ));
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
    format!(
        r#"import SwiftUI

struct DoweGeneratedTheme {{
    let name: String
    let colors: [String: Color]
    let radius: CGFloat
    let radiusBox: CGFloat
    let radiusUi: CGFloat
}}

enum DoweThemeModule {{
    static let generated = true
    static let defaultTheme = "{}"
    static let names = [
{}
    ]
    static let themes = [
{}
    ]
}}
"#,
        escape_swift(&design_config.default_theme),
        names,
        themes
    )
}

fn swift_theme_record(theme: &DesignTheme) -> String {
    let colors = ColorToken::all()
        .iter()
        .map(|token| {
            format!(
                "            \"{}\": {},",
                token.as_str(),
                swift_color_literal(theme.color_value(*token))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "        DoweGeneratedTheme(name: \"{}\", colors: [\n{}\n        ], radius: CGFloat({}), radiusBox: CGFloat({}), radiusUi: CGFloat({})),",
        escape_swift(&theme.name),
        colors,
        theme.radii.radius,
        theme.radii.radius_box,
        theme.radii.radius_ui
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
    match value {
        ColorFamily::Primary => ColorToken::Primary,
        ColorFamily::Secondary => ColorToken::Secondary,
        ColorFamily::Tertiary => ColorToken::Tertiary,
        ColorFamily::Muted => ColorToken::Muted,
        ColorFamily::Background => ColorToken::Background,
        ColorFamily::Surface => ColorToken::Surface,
        ColorFamily::Success => ColorToken::Success,
        ColorFamily::Info => ColorToken::Info,
        ColorFamily::Warning => ColorToken::Warning,
        ColorFamily::Danger => ColorToken::Danger,
    }
}

fn family_on_color(value: ColorFamily) -> ColorToken {
    match value {
        ColorFamily::Primary => ColorToken::OnPrimary,
        ColorFamily::Secondary => ColorToken::OnSecondary,
        ColorFamily::Tertiary => ColorToken::OnTertiary,
        ColorFamily::Muted => ColorToken::OnMuted,
        ColorFamily::Background => ColorToken::OnBackground,
        ColorFamily::Surface => ColorToken::OnSurface,
        ColorFamily::Success => ColorToken::OnSuccess,
        ColorFamily::Info => ColorToken::OnInfo,
        ColorFamily::Warning => ColorToken::OnWarning,
        ColorFamily::Danger => ColorToken::OnDanger,
    }
}

fn family_soft_color(value: ColorFamily) -> ColorToken {
    match value {
        ColorFamily::Primary => ColorToken::SoftPrimary,
        ColorFamily::Secondary => ColorToken::SoftSecondary,
        ColorFamily::Tertiary => ColorToken::SoftTertiary,
        ColorFamily::Muted => ColorToken::SoftMuted,
        ColorFamily::Background => ColorToken::Background,
        ColorFamily::Surface => ColorToken::Surface,
        ColorFamily::Success => ColorToken::SoftSuccess,
        ColorFamily::Info => ColorToken::SoftInfo,
        ColorFamily::Warning => ColorToken::SoftWarning,
        ColorFamily::Danger => ColorToken::SoftDanger,
    }
}

fn family_on_soft_color(value: ColorFamily) -> ColorToken {
    match value {
        ColorFamily::Primary => ColorToken::OnSoftPrimary,
        ColorFamily::Secondary => ColorToken::OnSoftSecondary,
        ColorFamily::Tertiary => ColorToken::OnSoftTertiary,
        ColorFamily::Muted => ColorToken::OnSoftMuted,
        ColorFamily::Background => ColorToken::OnBackground,
        ColorFamily::Surface => ColorToken::OnSurface,
        ColorFamily::Success => ColorToken::OnSoftSuccess,
        ColorFamily::Info => ColorToken::OnSoftInfo,
        ColorFamily::Warning => ColorToken::OnSoftWarning,
        ColorFamily::Danger => ColorToken::OnSoftDanger,
    }
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
