fn swift_scale_value(value: &ResponsiveValue<ScaleValue>) -> String {
    swift_responsive_value(value, |value| format!("CGFloat({})", value.native_units()))
}

fn swift_size_value(value: &ResponsiveValue<SizeValue>) -> String {
    swift_responsive_value(value, |value| match value {
        SizeValue::Scale(value) => format!("DoweSize.fixed(CGFloat({}))", value.native_units()),
        SizeValue::Full => "DoweSize.full".to_string(),
    })
}

fn swift_color_value(value: &ResponsiveValue<ColorToken>) -> String {
    swift_responsive_value(value, |value| color_ref(*value).to_string())
}

fn swift_bool_value(value: &ResponsiveValue<bool>) -> String {
    swift_responsive_value(value, |value| value.to_string())
}

fn swift_code_tokens(tokens: &[CodeToken], plain: &str) -> String {
    let values = tokens
        .iter()
        .map(|token| {
            format!(
                "DoweCodeToken(text: \"{}\", color: {})",
                escape_swift(&token.text),
                swift_code_token_color(token.kind, plain)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn swift_code_token_color(kind: CodeTokenKind, plain: &str) -> String {
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

fn swift_font_value(
    value: Option<&ResponsiveValue<FontFamily>>,
    size: &str,
    default_family: FontFamily,
) -> String {
    let default = swift_font_expr(&default_family);
    value
        .map(|value| {
            format!(
                "doweFont({} ?? {default}, size: {size})",
                swift_responsive_value(value, swift_font_expr)
            )
        })
        .unwrap_or_else(|| format!("doweFont({default}, size: {size})"))
}

fn swift_rounded_value(value: &ResponsiveValue<RoundedSize>) -> String {
    swift_responsive_value(value, |value| {
        format!("CGFloat({})", rounded_points(*value))
    })
}

fn swift_border_value(value: &ResponsiveValue<BorderWidth>) -> String {
    swift_responsive_value(value, |value| format!("CGFloat({})", value.0))
}

fn swift_justify_value(value: &ResponsiveValue<Justify>) -> String {
    swift_responsive_value(value, |value| format!("DoweJustify.{}", value.as_str()))
}

fn swift_align_value(value: &ResponsiveValue<Align>) -> String {
    swift_responsive_value(value, |value| format!("DoweAlign.{}", value.as_str()))
}

fn swift_grid_alignment_value(value: &ResponsiveValue<GridAlignment>) -> String {
    swift_responsive_value(value, |value| format!("DoweAlign.{}", value.as_str()))
}

fn swift_font_expr(value: &FontFamily) -> String {
    format!(".{}", value.as_str())
}

fn swift_font_cases(font_families: &BTreeSet<FontFamily>) -> String {
    font_families
        .iter()
        .map(|font| format!("    case {}", font.as_str()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn swift_font_switch(font_families: &BTreeSet<FontFamily>) -> String {
    font_families
        .iter()
        .map(|font| {
            format!(
                "    case .{}:\n        return {}",
                font.as_str(),
                swift_font_return(*font)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn swift_font_return(value: FontFamily) -> String {
    if value == FontFamily::System {
        return ".system(size: size)".to_string();
    }

    format!(
        ".custom(\"{}\", size: size)",
        value.catalog_entry().ios_family_name
    )
}

fn swift_gap_value(value: &GapValue) -> String {
    match value {
        GapValue::Single(value) => swift_gap_size(value),
        GapValue::Pair(row, _) => swift_gap_size(row),
    }
}

fn swift_gap_size(value: &GapSize) -> String {
    match value {
        GapSize::Scale(value) => format!("CGFloat({})", value.native_units()),
        GapSize::Px(value) => format!("CGFloat({value})"),
    }
}

fn swift_cover_value(value: &ResponsiveValue<CoverSource>) -> String {
    swift_responsive_value(value, |value| format!("\"{}\"", escape_swift(&value.0)))
}

fn swift_section_background_value(value: &ResponsiveValue<SectionBackground>) -> String {
    swift_responsive_value(value, swift_section_background_expr)
}

fn swift_section_background_expr(value: &SectionBackground) -> String {
    match value {
        SectionBackground::Soft => "DoweSectionBackground.soft".to_string(),
        SectionBackground::Aurora => "DoweSectionBackground.aurora".to_string(),
        SectionBackground::Sunrise => "DoweSectionBackground.sunrise".to_string(),
        SectionBackground::Ocean => "DoweSectionBackground.ocean".to_string(),
        SectionBackground::Meadow => "DoweSectionBackground.meadow".to_string(),
        SectionBackground::Slate => "DoweSectionBackground.slate".to_string(),
    }
}

fn swift_overlay_value(value: &ResponsiveValue<OverlayPaint>) -> String {
    swift_responsive_value(value, swift_overlay_expr)
}

fn swift_overlay_expr(value: &OverlayPaint) -> String {
    match value {
        OverlayPaint::BlackOpacity(value) => {
            format!("DoweOverlay.color(Color.black.opacity({value}))")
        }
        OverlayPaint::Color(value) => format!("DoweOverlay.color({})", color_ref(*value)),
        OverlayPaint::Rgba(value) => format!("DoweOverlay.color({})", swift_rgba_color(value)),
        OverlayPaint::LinearGradient(value) => {
            let (start, end) = gradient_colors(value);
            format!(
                "DoweOverlay.gradient({}, {})",
                swift_rgba_color(start),
                swift_rgba_color(end)
            )
        }
    }
}

fn swift_svg_view_box(value: &SvgViewBox) -> String {
    format!(
        "DoweSvgViewBox(minX: CGFloat({}), minY: CGFloat({}), width: CGFloat({}), height: CGFloat({}))",
        value.min_x, value.min_y, value.width, value.height
    )
}

fn swift_svg_color(props: &StyleProps) -> String {
    props
        .text
        .as_ref()
        .map(swift_color_value)
        .map(|value| format!("{value} ?? DoweDesign.onBackground"))
        .unwrap_or_else(|| "DoweDesign.onBackground".to_string())
}

fn swift_svg_paths(paths: &[SvgPath]) -> String {
    let values = paths
        .iter()
        .map(|path| {
            format!(
                "DoweSvgPathData(data: \"{}\", fill: {})",
                escape_swift(&path.data),
                swift_svg_fill(path.fill)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn swift_svg_fill(fill: SvgPathFill) -> String {
    match fill {
        SvgPathFill::None => "DoweSvgFill.none".to_string(),
        SvgPathFill::CurrentColor => "DoweSvgFill.currentColor".to_string(),
        SvgPathFill::Color(token) => format!("DoweSvgFill.color({})", color_ref(token)),
    }
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

fn swift_modifiers_for_text(
    title: bool,
    props: &TextProps,
    font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
) -> Vec<String> {
    let size = text_size(title, props);
    let mut modifiers = vec![
        format!(".font({})", swift_font_value(font, &size, default_family)),
        format!(".fontWeight({})", text_weight(title, props)),
        format!(
            ".lineSpacing(doweTextLineSpacing(fontSize: {size}, lineHeight: {}))",
            text_line_height(title, props)
        ),
    ];

    if title || props.letter_spacing.is_some() {
        modifiers.push(format!(
            ".tracking(doweTextTracking(fontSize: {size}, em: {}))",
            text_spacing(title, props)
        ));
    }

    if let Some(color) = text_color(props) {
        modifiers.push(format!(".foregroundStyle({color})"));
    }
    modifiers.extend(swift_modifiers_for_style(&props.style));
    modifiers
}

fn text_color(props: &TextProps) -> Option<String> {
    props
        .style
        .text
        .as_ref()
        .map(swift_color_value)
        .map(|value| format!("{value} ?? DoweDesign.onBackground"))
}

fn text_size(title: bool, props: &TextProps) -> String {
    let fallback = swift_text_size_expr(title, TextSize::Md);
    props
        .size
        .as_ref()
        .map(|value| swift_responsive_value(value, |value| swift_text_size_expr(title, *value)))
        .map(|value| format!("{value} ?? {fallback}"))
        .unwrap_or(fallback)
}

fn text_line_height(title: bool, props: &TextProps) -> String {
    let fallback = format!(
        "CGFloat({})",
        text_typography(title, TextSize::Md).line_height
    );
    props
        .size
        .as_ref()
        .map(|value| {
            swift_responsive_value(value, |value| {
                format!("CGFloat({})", text_typography(title, *value).line_height)
            })
        })
        .map(|value| format!("{value} ?? {fallback}"))
        .unwrap_or(fallback)
}

fn text_weight(title: bool, props: &TextProps) -> String {
    if let Some(value) = props.weight.as_ref() {
        let fallback = swift_text_weight(TextWeight::Regular);
        return format!(
            "{} ?? {fallback}",
            swift_responsive_value(value, |value| swift_text_weight(*value).to_string())
        );
    }

    if title {
        let fallback = swift_text_weight(text_typography(true, TextSize::Md).weight);
        props
            .size
            .as_ref()
            .map(|value| {
                swift_responsive_value(value, |value| {
                    swift_text_weight(text_typography(true, *value).weight).to_string()
                })
            })
            .map(|value| format!("{value} ?? {fallback}"))
            .unwrap_or_else(|| fallback.to_string())
    } else {
        swift_text_weight(TextWeight::Regular).to_string()
    }
}

fn text_spacing(title: bool, props: &TextProps) -> String {
    if let Some(value) = props.letter_spacing.as_ref() {
        let fallback = "CGFloat(0)";
        return format!(
            "{} ?? {fallback}",
            swift_responsive_value(value, |value| {
                format!("CGFloat({})", text_spacing_em(*value))
            })
        );
    }

    if title {
        let fallback = format!(
            "CGFloat({})",
            text_typography(true, TextSize::Md).letter_spacing_em
        );
        props
            .size
            .as_ref()
            .map(|value| {
                swift_responsive_value(value, |value| {
                    format!(
                        "CGFloat({})",
                        text_typography(true, *value).letter_spacing_em
                    )
                })
            })
            .map(|value| format!("{value} ?? {fallback}"))
            .unwrap_or(fallback)
    } else {
        "CGFloat(0)".to_string()
    }
}

fn swift_text_size_expr(title: bool, value: TextSize) -> String {
    let size = text_typography(title, value).font_size;
    format!(
        "doweTextSize(viewportWidth, min: CGFloat({}), preferredBase: CGFloat({}), preferredViewport: CGFloat({}), max: CGFloat({}))",
        size.min, size.preferred_base, size.preferred_viewport, size.max
    )
}

fn swift_text_weight(value: TextWeight) -> &'static str {
    match value {
        TextWeight::Thin => "Font.Weight.ultraLight",
        TextWeight::Extralight => "Font.Weight.thin",
        TextWeight::Light => "Font.Weight.light",
        TextWeight::Regular => "Font.Weight.regular",
        TextWeight::Medium => "Font.Weight.medium",
        TextWeight::Semibold => "Font.Weight.semibold",
        TextWeight::Bold => "Font.Weight.bold",
        TextWeight::Extrabold => "Font.Weight.heavy",
        TextWeight::Black => "Font.Weight.black",
    }
}
