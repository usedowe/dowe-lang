fn text_color(title: bool, props: &TextProps) -> String {
    props
        .style
        .text
        .as_ref()
        .map(compose_color_value)
        .map(|value| {
            let fallback = if title {
                "LocalDoweTitleColor.current"
            } else {
                "LocalContentColor.current"
            };
            format!("{value} ?: {fallback}")
        })
        .unwrap_or_else(|| {
            if title {
                "LocalDoweTitleColor.current".to_string()
            } else {
                "Color.Unspecified".to_string()
            }
        })
}

fn text_size(title: bool, props: &TextProps, context: &ComposeReactiveContext) -> String {
    if let Some(binding) = props.size_binding.as_ref() {
        let value = compose_text_binding(binding, context);
        return format!(
            "doweDynamicTextSize({value}, viewportWidth, title = {title})"
        );
    }
    let fallback = compose_text_size_expr(title, TextSize::Md);
    props
        .size
        .as_ref()
        .map(|value| compose_responsive_value(value, |value| compose_text_size_expr(title, *value)))
        .map(|value| format!("{value} ?: {fallback}"))
        .unwrap_or(fallback)
}

fn text_line_height(
    title: bool,
    props: &TextProps,
    size: &str,
    context: &ComposeReactiveContext,
) -> String {
    if let Some(binding) = props.size_binding.as_ref() {
        let value = compose_text_binding(binding, context);
        return format!(
            "doweDynamicTextLineHeight({value}, title = {title}, fontSize = {size})"
        );
    }
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

fn text_weight(title: bool, props: &TextProps, context: &ComposeReactiveContext) -> String {
    if let Some(binding) = props.weight_binding.as_ref() {
        let value = compose_text_binding(binding, context);
        return format!("doweDynamicTextWeight({value})");
    }
    if let Some(value) = props.weight.as_ref() {
        let fallback = compose_text_weight(TextWeight::Regular);
        return format!(
            "{} ?: {fallback}",
            compose_responsive_value(value, |value| compose_text_weight(*value).to_string())
        );
    }

    if title {
        if let Some(binding) = props.size_binding.as_ref() {
            let value = compose_text_binding(binding, context);
            return format!(
                "doweDynamicTextWeightForSize({value}, title = true)"
            );
        }
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

fn text_spacing(title: bool, props: &TextProps, context: &ComposeReactiveContext) -> String {
    if let Some(binding) = props.letter_spacing_binding.as_ref() {
        let value = compose_text_binding(binding, context);
        return format!("doweDynamicTextSpacing({value})");
    }
    if let Some(value) = props.letter_spacing.as_ref() {
        let fallback = "0f.em";
        return format!(
            "{} ?: {fallback}",
            compose_responsive_value(value, |value| compose_text_spacing(*value).to_string())
        );
    }

    if title {
        if let Some(binding) = props.size_binding.as_ref() {
            let value = compose_text_binding(binding, context);
            return format!(
                "doweDynamicTextSpacingForSize({value}, title = true)"
            );
        }
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

fn compose_text_binding(
    binding: &dowe_components::PropBinding,
    context: &ComposeReactiveContext,
) -> String {
    if let Some(item) = context.item_value(&binding.path) {
        let path = context
            .item_path(&binding.path)
            .unwrap_or_else(|| binding.path.clone());
        format!("state.text(\"{}\", {item})", escape_kotlin(&path))
    } else {
        format!(
            "state.text(\"{}\")",
            escape_kotlin(&context.signal_path(&binding.path))
        )
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
