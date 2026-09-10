fn swift_svg_fill(fill: SvgPathFill) -> String {
    match fill {
        SvgPathFill::None => "DoweSvgFill.none".to_string(),
        SvgPathFill::CurrentColor => "DoweSvgFill.currentColor".to_string(),
        SvgPathFill::Color(token) => format!("DoweSvgFill.color({})", color_ref(token)),
        SvgPathFill::RawFill {
            color,
            opacity,
            even_odd,
        } => format!(
            "DoweSvgFill.fill(.some({}), {}, {})",
            swift_hex_color(color),
            opacity as f32 / 255.0,
            even_odd
        ),
        SvgPathFill::Fill {
            color,
            opacity,
            even_odd,
        } => format!(
            "DoweSvgFill.fill({}, {}, {})",
            color
                .map(color_ref)
                .map(|value| format!(".some({value})"))
                .unwrap_or_else(|| ".none".to_string()),
            opacity as f32 / 255.0,
            even_odd
        ),
        SvgPathFill::RawStroke {
            color,
            opacity,
            width,
            line_cap,
            line_join,
        } => format!(
            "DoweSvgFill.stroke(.some({}), {}, {}, \"{}\", \"{}\")",
            swift_hex_color(color),
            opacity as f32 / 255.0,
            width as f32 / 100.0,
            match line_cap {
                SvgLineCap::Butt => "butt",
                SvgLineCap::Round => "round",
                SvgLineCap::Square => "square",
            },
            match line_join {
                SvgLineJoin::Miter => "miter",
                SvgLineJoin::Round => "round",
                SvgLineJoin::Bevel => "bevel",
            }
        ),
        SvgPathFill::LiteralFill {
            red,
            green,
            blue,
            opacity,
            even_odd,
        } => format!(
            "DoweSvgFill.fill(.some({}), {}, {})",
            swift_rgb_color(red, green, blue),
            opacity as f32 / 255.0,
            even_odd
        ),
        SvgPathFill::LiteralStroke {
            red,
            green,
            blue,
            opacity,
            width,
            line_cap,
            line_join,
        } => format!(
            "DoweSvgFill.stroke(.some({}), {}, {}, \"{}\", \"{}\")",
            swift_rgb_color(red, green, blue),
            opacity as f32 / 255.0,
            width as f32 / 100.0,
            match line_cap {
                SvgLineCap::Butt => "butt",
                SvgLineCap::Round => "round",
                SvgLineCap::Square => "square",
            },
            match line_join {
                SvgLineJoin::Miter => "miter",
                SvgLineJoin::Round => "round",
                SvgLineJoin::Bevel => "bevel",
            }
        ),
        SvgPathFill::Stroke {
            color,
            opacity,
            width,
            line_cap,
            line_join,
        } => format!(
            "DoweSvgFill.stroke({}, {}, {}, \"{}\", \"{}\")",
            color
                .map(color_ref)
                .map(|value| format!(".some({value})"))
                .unwrap_or_else(|| ".none".to_string()),
            opacity as f32 / 255.0,
            width as f32 / 100.0,
            match line_cap {
                SvgLineCap::Butt => "butt",
                SvgLineCap::Round => "round",
                SvgLineCap::Square => "square",
            },
            match line_join {
                SvgLineJoin::Miter => "miter",
                SvgLineJoin::Round => "round",
                SvgLineJoin::Bevel => "bevel",
            }
        ),
    }
}

fn swift_rgb_color(red: u8, green: u8, blue: u8) -> String {
    format!(
        "Color(red: {:.3}, green: {:.3}, blue: {:.3})",
        red as f32 / 255.0,
        green as f32 / 255.0,
        blue as f32 / 255.0
    )
}

fn swift_hex_color(value: &str) -> String {
    let raw = value.trim_start_matches('#');
    let red = u8::from_str_radix(&raw[0..2], 16).expect("red color");
    let green = u8::from_str_radix(&raw[2..4], 16).expect("green color");
    let blue = u8::from_str_radix(&raw[4..6], 16).expect("blue color");
    swift_rgb_color(red, green, blue)
}

fn swift_rgba_color(value: &str) -> String {
    if let Some((red, green, blue, alpha)) = parse_rgba(value) {
        format!(
            "Color(red: {:.3}, green: {:.3}, blue: {:.3}).opacity({})",
            red as f32 / 255.0,
            green as f32 / 255.0,
            blue as f32 / 255.0,
            alpha
        )
    } else {
        "Color.black.opacity(0.4)".to_string()
    }
}

fn swift_responsive_value<T, F>(value: &ResponsiveValue<T>, map: F) -> String
where
    F: Fn(&T) -> String,
{
    let entries = value
        .entries
        .iter()
        .map(|entry| {
            format!(
                "{}: {}",
                swift_breakpoint_arg(entry.breakpoint),
                map(&entry.value)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("doweResponsive(viewportWidth, {entries})")
}

fn swift_breakpoint_arg(value: Breakpoint) -> &'static str {
    value.as_str()
}

fn rounded_points(value: RoundedSize) -> u16 {
    match value {
        RoundedSize::Xs => 4,
        RoundedSize::Sm => 6,
        RoundedSize::Md => 8,
        RoundedSize::Lg => 12,
        RoundedSize::Xl => 18,
        RoundedSize::Full => 999,
    }
}

fn shadow_points(value: ShadowSize) -> u16 {
    match value {
        ShadowSize::Xs => 2,
        ShadowSize::Sm => 12,
        ShadowSize::Md => 24,
        ShadowSize::Lg => 44,
        ShadowSize::Xl => 70,
    }
}

fn shadow_offset_points(value: ShadowSize) -> u16 {
    match value {
        ShadowSize::Xs => 1,
        ShadowSize::Sm => 4,
        ShadowSize::Md => 10,
        ShadowSize::Lg => 18,
        ShadowSize::Xl => 28,
    }
}

fn shadow_opacity(value: ShadowSize) -> &'static str {
    match value {
        ShadowSize::Xs => "0.12",
        ShadowSize::Sm => "0.14",
        ShadowSize::Md => "0.16",
        ShadowSize::Lg => "0.18",
        ShadowSize::Xl => "0.22",
    }
}

