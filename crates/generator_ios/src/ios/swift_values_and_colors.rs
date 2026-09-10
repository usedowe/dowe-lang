fn swift_scale_value(value: &ResponsiveValue<ScaleValue>) -> String {
    swift_responsive_value(value, |value| format!("CGFloat({})", value.native_units()))
}

fn swift_size_value(value: &ResponsiveValue<SizeValue>) -> String {
    swift_responsive_value(value, |value| match value {
        SizeValue::Scale(value) => format!("DoweSize.fixed(CGFloat({}))", value.native_units()),
        SizeValue::Container(value) => {
            format!(
                "DoweSize.fixed(CGFloat({}))",
                value.scale_value().native_units()
            )
        }
        SizeValue::Percent(value) => {
            format!("DoweSize.percent(CGFloat({}))", f32::from(*value) / 100.0)
        }
        SizeValue::Full => "DoweSize.full".to_string(),
        SizeValue::Auto => "DoweSize.auto".to_string(),
        SizeValue::ViewportMinus(value) => {
            format!("DoweSize.viewportMinus(CGFloat({}))", value.native_units())
        }
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
        CodeTokenKind::Attribute => "DoweDesign.accent".to_string(),
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

fn swift_font_token_value(
    value: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
) -> String {
    let default = swift_font_expr(&default_family);
    value
        .map(|value| {
            format!(
                "{} ?? {default}",
                swift_responsive_value(value, swift_font_expr)
            )
        })
        .unwrap_or_else(|| default.to_string())
}

fn swift_rounded_value(value: &ResponsiveValue<RoundedSize>) -> String {
    swift_responsive_value(value, |value| {
        format!("CGFloat({})", rounded_points(*value))
    })
}

fn swift_border_value(value: &ResponsiveValue<BorderWidth>) -> String {
    swift_responsive_value(value, |value| format!("CGFloat({})", value.0))
}

fn swift_shadow_value(value: &ResponsiveValue<ShadowSize>) -> String {
    swift_responsive_value(value, |value| format!("CGFloat({})", shadow_points(*value)))
}

fn swift_shadow_offset_value(value: &ResponsiveValue<ShadowSize>) -> String {
    swift_responsive_value(value, |value| {
        format!("CGFloat({})", shadow_offset_points(*value))
    })
}

fn swift_shadow_opacity_value(value: &ResponsiveValue<ShadowSize>) -> String {
    swift_responsive_value(value, |value| format!("Double({})", shadow_opacity(*value)))
}

fn swift_justify_value(value: &ResponsiveValue<Justify>) -> String {
    swift_responsive_value(value, |value| {
        let name = match value {
            Justify::EndSafe => "endSafe",
            Justify::CenterSafe => "centerSafe",
            _ => value.as_str(),
        };
        format!("DoweJustify.{name}")
    })
}

fn swift_flex_direction_value(value: &ResponsiveValue<FlexDirection>) -> String {
    format!(
        "({} ?? DoweFlexDirection.row)",
        swift_responsive_value(value, |value| format!(
            "DoweFlexDirection.{}",
            value.as_str()
        ))
    )
}

fn swift_align_value(value: &ResponsiveValue<Align>) -> String {
    swift_responsive_value(value, |value| {
        let name = match value {
            Align::BaselineLast => "baselineLast",
            Align::EndSafe => "endSafe",
            Align::CenterSafe => "centerSafe",
            _ => value.as_str(),
        };
        format!("DoweAlign.{name}")
    })
}

fn swift_grid_alignment_value(value: &ResponsiveValue<GridAlignment>) -> String {
    swift_responsive_value(value, |value| {
        let name = match value {
            GridAlignment::EndSafe => "endSafe",
            GridAlignment::CenterSafe => "centerSafe",
            GridAlignment::BaselineLast => "baselineLast",
            _ => value.as_str(),
        };
        format!("DoweAlign.{name}")
    })
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
        .map(|value| format!("{value} ?? DoweDesign.backgroundText"))
        .unwrap_or_else(|| "DoweDesign.backgroundText".to_string())
}

fn swift_svg_paths(paths: &[SvgPath]) -> String {
    let values = paths
        .iter()
        .map(|path| {
            format!(
                "DoweSvgPathData(data: \"{}\", fill: {}, transform: {})",
                escape_swift(&path.data),
                swift_svg_fill(path.fill),
                path.transform
                    .as_ref()
                    .map(|value| format!(
                        "CGAffineTransform(a: {}, b: {}, c: {}, d: {}, tx: {}, ty: {})",
                        value.a, value.b, value.c, value.d, value.e, value.f
                    ))
                    .unwrap_or_else(|| "nil".to_string())
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

