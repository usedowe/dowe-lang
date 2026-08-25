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

fn swift_modifiers_for_text(
    title: bool,
    props: &TextProps,
    font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
) -> Vec<String> {
    let size = text_size(title, props);
    let mut modifiers = vec![];
    if let Some(binding) = props.size_binding.as_ref() {
        modifiers.push(format!(".font(doweDynamicFontSize(state.text(\"{}\")))", escape_swift(&binding.path)));
    }
    if let Some(binding) = props.weight_binding.as_ref() {
        modifiers.push(format!(".fontWeight(doweDynamicFontWeight(state.text(\"{}\")))", escape_swift(&binding.path)));
    }
    modifiers.extend(vec![
        format!(".font({})", swift_font_value(font, &size, default_family)),
        format!(".fontWeight({})", text_weight(title, props)),
        format!(
            ".lineSpacing(doweTextLineSpacing(fontSize: {size}, lineHeight: {}))",
            text_line_height(title, props)
        ),
    ]);

    if title || props.letter_spacing.is_some() {
        modifiers.push(format!(
            ".tracking(doweTextTracking(fontSize: {size}, em: {}))",
            text_spacing(title, props)
        ));
    }

    if let Some(value) = props.align.as_ref() {
        if props.style.sizing.w.is_none() {
            modifiers.push(format!(
                ".frame(maxWidth: .infinity, alignment: {})",
                swift_text_frame_alignment(value)
            ));
        }
    }

    if title {
        let color = props
            .style
            .text
            .as_ref()
            .map(swift_color_value)
            .unwrap_or_else(|| "nil".to_string());
        modifiers.push(format!(
            ".modifier(DoweTitleColorModifier(explicitColor: {color}))"
        ));
    } else if let Some(color) = text_color(props) {
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
        .map(|value| format!("{value} ?? DoweDesign.backgroundText"))
}

fn swift_dowe_text_alignment(value: &ResponsiveValue<TextAlign>) -> String {
    format!(
        "{} ?? DoweTextAlignment.start",
        swift_responsive_value(value, |value| match value {
            TextAlign::Start => "DoweTextAlignment.start".to_string(),
            TextAlign::Center => "DoweTextAlignment.center".to_string(),
            TextAlign::End => "DoweTextAlignment.end".to_string(),
            TextAlign::Justify => "DoweTextAlignment.justify".to_string(),
        })
    )
}

fn swift_text_frame_alignment(value: &ResponsiveValue<TextAlign>) -> String {
    format!(
        "{} ?? Alignment.leading",
        swift_responsive_value(value, |value| match value {
            TextAlign::Start => "Alignment.leading".to_string(),
            TextAlign::Center => "Alignment.center".to_string(),
            TextAlign::End => "Alignment.trailing".to_string(),
            TextAlign::Justify => "Alignment.leading".to_string(),
        })
    )
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
