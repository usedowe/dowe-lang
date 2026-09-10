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
