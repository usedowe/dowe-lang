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
        SectionBackground::Soft => "DoweSectionBackground.Soft".to_string(),
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
                "DoweSvgPath(\"{}\", {})",
                escape_kotlin(&path.data),
                compose_svg_fill(path.fill)
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
        CodeTokenKind::Attribute => "DoweDesign.tertiary".to_string(),
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

fn compose_justify_value(value: &ResponsiveValue<Justify>) -> String {
    compose_responsive_value(value, |value| {
        format!("DoweJustify.{}", compose_justify_name(*value))
    })
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

fn compose_justify_name(value: Justify) -> &'static str {
    match value {
        Justify::Start => "Start",
        Justify::Center => "Center",
        Justify::End => "End",
        Justify::Between => "Between",
        Justify::Around => "Around",
        Justify::Evenly => "Evenly",
    }
}

fn compose_align_name(value: Align) -> &'static str {
    match value {
        Align::Start => "Start",
        Align::Center => "Center",
        Align::End => "End",
        Align::Stretch => "Stretch",
        Align::Baseline => "Baseline",
    }
}

fn compose_grid_alignment_name(value: GridAlignment) -> &'static str {
    match value {
        GridAlignment::Start => "Start",
        GridAlignment::Center => "Center",
        GridAlignment::End => "End",
        GridAlignment::Stretch => "Stretch",
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

fn text_color(props: &TextProps) -> String {
    props
        .style
        .text
        .as_ref()
        .map(compose_color_value)
        .map(|value| format!("{value} ?: DoweDesign.onBackground"))
        .unwrap_or_else(|| "Color.Unspecified".to_string())
}

fn text_size(title: bool, props: &TextProps) -> String {
    let fallback = compose_text_size_expr(title, TextSize::Md);
    props
        .size
        .as_ref()
        .map(|value| compose_responsive_value(value, |value| compose_text_size_expr(title, *value)))
        .map(|value| format!("{value} ?: {fallback}"))
        .unwrap_or(fallback)
}

fn text_line_height(title: bool, props: &TextProps, size: &str) -> String {
    let fallback = format!("{}f", text_typography(title, TextSize::Md).line_height);
    let line_height = props
        .size
        .as_ref()
        .map(|value| {
            compose_responsive_value(value, |value| {
                format!("{}f", text_typography(title, *value).line_height)
            })
        })
        .map(|value| format!("{value} ?: {fallback}"))
        .unwrap_or(fallback);
    format!("doweTextLineHeight({size}, {line_height})")
}

fn text_weight(title: bool, props: &TextProps) -> String {
    if let Some(value) = props.weight.as_ref() {
        let fallback = compose_text_weight(TextWeight::Regular);
        return format!(
            "{} ?: {fallback}",
            compose_responsive_value(value, |value| compose_text_weight(*value).to_string())
        );
    }

    if title {
        let fallback = compose_text_weight(text_typography(true, TextSize::Md).weight);
        props
            .size
            .as_ref()
            .map(|value| {
                compose_responsive_value(value, |value| {
                    compose_text_weight(text_typography(true, *value).weight).to_string()
                })
            })
            .map(|value| format!("{value} ?: {fallback}"))
            .unwrap_or_else(|| fallback.to_string())
    } else {
        compose_text_weight(TextWeight::Regular).to_string()
    }
}

fn text_spacing(title: bool, props: &TextProps) -> String {
    if let Some(value) = props.letter_spacing.as_ref() {
        let fallback = "0f.em";
        return format!(
            "{} ?: {fallback}",
            compose_responsive_value(value, |value| compose_text_spacing(*value).to_string())
        );
    }

    if title {
        let fallback = compose_default_text_spacing(TextSize::Md);
        props
            .size
            .as_ref()
            .map(|value| {
                compose_responsive_value(value, |value| compose_default_text_spacing(*value))
            })
            .map(|value| format!("{value} ?: {fallback}"))
            .unwrap_or(fallback)
    } else {
        "0f.em".to_string()
    }
}

fn compose_text_size_expr(title: bool, value: TextSize) -> String {
    let size = text_typography(title, value).font_size;
    format!(
        "doweTextSize(viewportWidth, min = {}f, preferredBase = {}f, preferredViewport = {}f, max = {}f)",
        size.min, size.preferred_base, size.preferred_viewport, size.max
    )
}

fn compose_text_weight(value: TextWeight) -> &'static str {
    match value {
        TextWeight::Thin => "FontWeight.Thin",
        TextWeight::Extralight => "FontWeight.ExtraLight",
        TextWeight::Light => "FontWeight.Light",
        TextWeight::Regular => "FontWeight.Normal",
        TextWeight::Medium => "FontWeight.Medium",
        TextWeight::Semibold => "FontWeight.SemiBold",
        TextWeight::Bold => "FontWeight.Bold",
        TextWeight::Extrabold => "FontWeight.ExtraBold",
        TextWeight::Black => "FontWeight.Black",
    }
}

fn compose_text_spacing(value: TextSpacing) -> String {
    compose_em(text_spacing_em(value))
}

fn compose_default_text_spacing(value: TextSize) -> String {
    compose_em(text_typography(true, value).letter_spacing_em)
}

fn compose_em(value: &str) -> String {
    if value.starts_with('-') {
        format!("({value}f).em")
    } else {
        format!("{value}f.em")
    }
}
