fn compose_font_value(
    value: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
) -> String {
    let default = format!("DoweFont.{}", font_name(default_family));
    value
        .map(|value| {
            format!(
                "doweFontFamily({} ?: {default})",
                compose_responsive_value(value, compose_font_expr),
            )
        })
        .unwrap_or_else(|| format!("doweFontFamily({default})"))
}

fn compose_cover_value(value: &ResponsiveValue<CoverSource>) -> String {
    compose_responsive_value(value, |value| format!("\"{}\"", escape_kotlin(&value.0)))
}

fn compose_section_background_value(value: &ResponsiveValue<SectionBackground>) -> String {
    compose_responsive_value(value, compose_section_background_expr)
}

fn compose_section_background_expr(value: &SectionBackground) -> String {
    match value {
        SectionBackground::Aurora => "DoweSectionBackground.Aurora".to_string(),
        SectionBackground::Sunrise => "DoweSectionBackground.Sunrise".to_string(),
        SectionBackground::Ocean => "DoweSectionBackground.Ocean".to_string(),
        SectionBackground::Meadow => "DoweSectionBackground.Meadow".to_string(),
        SectionBackground::Slate => "DoweSectionBackground.Slate".to_string(),
    }
}

fn compose_optional_overlay(value: Option<&ResponsiveValue<OverlayPaint>>) -> String {
    value
        .map(compose_overlay_value)
        .unwrap_or_else(|| "null".to_string())
}

fn compose_overlay_value(value: &ResponsiveValue<OverlayPaint>) -> String {
    compose_responsive_value(value, compose_overlay_expr)
}

fn compose_overlay_expr(value: &OverlayPaint) -> String {
    match value {
        OverlayPaint::BlackOpacity(value) => {
            format!("DoweOverlay.Solid(Color.Black.copy(alpha = {value}f))")
        }
        OverlayPaint::Color(value) => format!("DoweOverlay.Solid({})", color_ref(*value)),
        OverlayPaint::Rgba(value) => format!("DoweOverlay.Solid({})", compose_rgba_color(value)),
        OverlayPaint::LinearGradient(value) => {
            let (start, end) = gradient_colors(value);
            format!(
                "DoweOverlay.Gradient({}, {})",
                compose_rgba_color(&start),
                compose_rgba_color(&end)
            )
        }
    }
}

fn compose_svg_view_box(value: &SvgViewBox) -> String {
    format!(
        "DoweSvgViewBox({}f, {}f, {}f, {}f)",
        value.min_x, value.min_y, value.width, value.height
    )
}

fn compose_svg_color(props: &StyleProps) -> String {
    props
        .text
        .as_ref()
        .map(compose_color_value)
        .map(|value| format!("{value} ?: LocalContentColor.current"))
        .unwrap_or_else(|| "LocalContentColor.current".to_string())
}

fn compose_svg_paths(paths: &[SvgPath]) -> String {
    let values = paths
        .iter()
        .map(|path| {
            format!(
                "DoweSvgPath(\"{}\", {}, {})",
                escape_kotlin(&path.data),
                compose_svg_fill(path.fill),
                path.transform
                    .as_ref()
                    .map(|value| format!(
                        "DoweSvgTransform({}f, {}f, {}f, {}f, {}f, {}f)",
                        value.a, value.b, value.c, value.d, value.e, value.f
                    ))
                    .unwrap_or_else(|| "null".to_string())
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
}

fn compose_svg_fill(fill: SvgPathFill) -> String {
    match fill {
        SvgPathFill::None => "DoweSvgFill.None".to_string(),
        SvgPathFill::CurrentColor => "DoweSvgFill.CurrentColor".to_string(),
        SvgPathFill::Color(token) => format!("DoweSvgFill.Solid({})", color_ref(token)),
        SvgPathFill::RawFill {
            color,
            opacity,
            even_odd,
        } => format!(
            "DoweSvgFill.Fill({}, {}f, {})",
            android_color_literal(color),
            opacity as f32 / 255.0,
            even_odd
        ),
        SvgPathFill::Fill {
            color,
            opacity,
            even_odd,
        } => format!(
            "DoweSvgFill.Fill({}, {}f, {})",
            color
                .map(color_ref)
                .map(|value| format!("{value}"))
                .unwrap_or_else(|| "null".to_string()),
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
            "DoweSvgFill.Stroke({}, {}f, {}f, \"{}\", \"{}\")",
            android_color_literal(color),
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
            "DoweSvgFill.Fill(Color(0xFF{red:02X}{green:02X}{blue:02X}), {}f, {})",
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
            "DoweSvgFill.Stroke(Color(0xFF{red:02X}{green:02X}{blue:02X}), {}f, {}f, \"{}\", \"{}\")",
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
            "DoweSvgFill.Stroke({}, {}f, {}f, \"{}\", \"{}\")",
            color
                .map(color_ref)
                .map(|value| format!("{value}"))
                .unwrap_or_else(|| "null".to_string()),
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

fn compose_code_tokens(tokens: &[CodeToken], plain: &str) -> String {
    let values = tokens
        .iter()
        .map(|token| {
            format!(
                "DoweCodeToken(text = \"{}\", color = {})",
                escape_kotlin(&token.text),
                compose_code_token_color(token.kind, plain)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("listOf({values})")
}

fn compose_code_token_color(kind: CodeTokenKind, plain: &str) -> String {
    match kind {
        CodeTokenKind::Plain => plain.to_string(),
        CodeTokenKind::Keyword => "DoweDesign.primary".to_string(),
        CodeTokenKind::Type => "DoweDesign.info".to_string(),
        CodeTokenKind::String => "DoweDesign.success".to_string(),
        CodeTokenKind::Number => "DoweDesign.warning".to_string(),
        CodeTokenKind::Attribute => "DoweDesign.accent".to_string(),
        CodeTokenKind::Comment => "DoweDesign.muted".to_string(),
        CodeTokenKind::Punctuation => "DoweDesign.danger".to_string(),
    }
}

fn compose_rgba_color(value: &str) -> String {
    if let Some((red, green, blue, alpha)) = parse_rgba(value) {
        format!("Color({red}, {green}, {blue}, ({alpha}f * 255).toInt())")
    } else {
        "Color.Black.copy(alpha = 0.4f)".to_string()
    }
}

fn compose_rounded_value(value: &ResponsiveValue<RoundedSize>) -> String {
    compose_responsive_value(value, |value| format!("{}.dp", rounded_dp(*value)))
}

fn compose_border_value(value: &ResponsiveValue<BorderWidth>) -> String {
    compose_responsive_value(value, |value| format!("{}.dp", value.0))
}

fn compose_shadow_value(value: &ResponsiveValue<ShadowSize>) -> String {
    compose_responsive_value(value, |value| format!("{}.dp", shadow_dp(*value)))
}

fn compose_justify_value(value: &ResponsiveValue<Justify>) -> String {
    compose_responsive_value(value, |value| {
        format!("DoweJustify.{}", compose_justify_name(*value))
    })
}

fn compose_flex_direction_value(value: &ResponsiveValue<FlexDirection>) -> String {
    format!(
        "({} ?: DoweFlexDirection.Row)",
        compose_responsive_value(value, |value| match value {
            FlexDirection::Row => "DoweFlexDirection.Row".to_string(),
            FlexDirection::Column => "DoweFlexDirection.Column".to_string(),
        })
    )
}

fn compose_align_value(value: &ResponsiveValue<Align>) -> String {
    compose_responsive_value(value, |value| {
        format!("DoweAlign.{}", compose_align_name(*value))
    })
}

fn compose_grid_alignment_value(value: &ResponsiveValue<GridAlignment>) -> String {
    compose_responsive_value(value, |value| {
        format!("DoweAlign.{}", compose_grid_alignment_name(*value))
    })
}

fn compose_font_expr(value: &FontFamily) -> String {
    format!("DoweFont.{}", font_name(*value))
}

fn compose_gap_expr(value: &GapValue) -> String {
    match value {
        GapValue::Single(value) => compose_gap_size(value),
        GapValue::Pair(row, _) => compose_gap_size(row),
    }
}

fn compose_gap_size(value: &GapSize) -> String {
    match value {
        GapSize::Scale(value) => format!("{}.dp", value.native_units()),
        GapSize::Px(value) => format!("{value}.dp"),
    }
}

fn compose_responsive_value<T, F>(value: &ResponsiveValue<T>, map: F) -> String
where
    F: Fn(&T) -> String,
{
    let entries = value
        .entries
        .iter()
        .map(|entry| format!("{} = {}", entry.breakpoint.as_str(), map(&entry.value)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("doweResponsive(viewportWidth, {entries})")
}

fn compose_text_align(value: &ResponsiveValue<TextAlign>) -> String {
    format!(
        "{} ?: TextAlign.Start",
        compose_responsive_value(value, |value| format!(
            "TextAlign.{}",
            compose_text_align_name(*value)
        ))
    )
}

fn compose_text_align_name(value: TextAlign) -> &'static str {
    match value {
        TextAlign::Start => "Start",
        TextAlign::Center => "Center",
        TextAlign::End => "End",
        TextAlign::Justify => "Justify",
    }
}

fn compose_justify_name(value: Justify) -> &'static str {
    match value {
        Justify::Start => "Start",
        Justify::Center => "Center",
        Justify::End => "End",
        Justify::Between => "Between",
        Justify::Around => "Around",
        Justify::Evenly => "Evenly",
        Justify::Stretch => "Stretch",
        Justify::Normal => "Normal",
        Justify::EndSafe => "EndSafe",
        Justify::CenterSafe => "CenterSafe",
    }
}

fn compose_align_name(value: Align) -> &'static str {
    match value {
        Align::Start => "Start",
        Align::Center => "Center",
        Align::End => "End",
        Align::Stretch => "Stretch",
        Align::Baseline => "Baseline",
        Align::BaselineLast => "BaselineLast",
        Align::EndSafe => "EndSafe",
        Align::CenterSafe => "CenterSafe",
    }
}

fn compose_grid_alignment_name(value: GridAlignment) -> &'static str {
    match value {
        GridAlignment::Start => "Start",
        GridAlignment::End => "End",
        GridAlignment::EndSafe => "EndSafe",
        GridAlignment::Center => "Center",
        GridAlignment::CenterSafe => "CenterSafe",
        GridAlignment::Between => "Between",
        GridAlignment::Around => "Around",
        GridAlignment::Evenly => "Evenly",
        GridAlignment::Stretch => "Stretch",
        GridAlignment::Baseline => "Baseline",
        GridAlignment::BaselineLast => "BaselineLast",
        GridAlignment::Normal => "Normal",
    }
}

fn rounded_dp(value: RoundedSize) -> u16 {
    match value {
        RoundedSize::Xs => 4,
        RoundedSize::Sm => 6,
        RoundedSize::Md => 8,
        RoundedSize::Lg => 12,
        RoundedSize::Xl => 18,
        RoundedSize::Full => 999,
    }
}

fn shadow_dp(value: ShadowSize) -> u16 {
    match value {
        ShadowSize::Xs => 2,
        ShadowSize::Sm => 12,
        ShadowSize::Md => 24,
        ShadowSize::Lg => 44,
        ShadowSize::Xl => 70,
    }
}

fn font_name(value: FontFamily) -> &'static str {
    match value {
        FontFamily::System => "System",
        FontFamily::Inter => "Inter",
        FontFamily::Roboto => "Roboto",
        FontFamily::Montserrat => "Montserrat",
        FontFamily::Lato => "Lato",
        FontFamily::Poppins => "Poppins",
        FontFamily::Manrope => "Manrope",
        FontFamily::Quicksand => "Quicksand",
        FontFamily::Lora => "Lora",
        FontFamily::Syne => "Syne",
        FontFamily::Jost => "Jost",
        FontFamily::Puritan => "Puritan",
    }
}

fn compose_font_family_ref(value: FontFamily) -> String {
    if value == FontFamily::System {
        "FontFamily.Default".to_string()
    } else {
        format!("DoweFonts.{}", value.as_str())
    }
}

fn android_font_resource_name(asset_stem: &str) -> String {
    asset_stem.replace('-', "_")
}

