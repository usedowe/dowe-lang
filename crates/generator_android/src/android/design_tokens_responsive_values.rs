fn dev_text_color(title: bool, props: &TextProps, inherited_color: Option<&str>) -> String {
    let fallback = if title {
        dev_inherited_title_color(inherited_color).unwrap_or("DOWE_BACKGROUND_TITLE")
    } else {
        dev_inherited_text_color(inherited_color).unwrap_or("DOWE_BACKGROUND_TEXT")
    };
    props
        .style
        .text
        .as_ref()
        .map(dev_color_value)
        .map(|value| format!("doweColor({value}, {fallback})"))
        .unwrap_or_else(|| fallback.to_string())
}

fn dev_text_size(title: bool, props: &TextProps) -> String {
    let fallback = dev_text_size_expr(title, TextSize::Md);
    props
        .size
        .as_ref()
        .map(|value| dev_responsive_float_value(value, |value| dev_text_size_expr(title, *value)))
        .map(|value| format!("doweTextSize({value}, {fallback})"))
        .unwrap_or(fallback)
}

fn dev_text_line_height(title: bool, props: &TextProps) -> String {
    let fallback = format!("{}f", text_typography(title, TextSize::Md).line_height);
    props
        .size
        .as_ref()
        .map(|value| {
            dev_responsive_float_value(value, |value| {
                format!("{}f", text_typography(title, *value).line_height)
            })
        })
        .map(|value| format!("doweTextSize({value}, {fallback})"))
        .unwrap_or(fallback)
}

fn dev_text_weight(title: bool, props: &TextProps) -> String {
    if let Some(value) = props.weight.as_ref() {
        let fallback = dev_text_weight_value(TextWeight::Regular);
        return format!(
            "doweTextWeight({}, {fallback})",
            dev_responsive_value(value, |value| dev_text_weight_value(*value).to_string())
        );
    }

    if title {
        let fallback = dev_text_weight_value(text_typography(true, TextSize::Md).weight);
        props
            .size
            .as_ref()
            .map(|value| {
                dev_responsive_value(value, |value| {
                    dev_text_weight_value(text_typography(true, *value).weight).to_string()
                })
            })
            .map(|value| format!("doweTextWeight({value}, {fallback})"))
            .unwrap_or_else(|| fallback.to_string())
    } else {
        dev_text_weight_value(TextWeight::Regular).to_string()
    }
}

fn dev_text_spacing(title: bool, props: &TextProps) -> String {
    if let Some(value) = props.letter_spacing.as_ref() {
        let fallback = "0f";
        return format!(
            "doweTextSize({}, {fallback})",
            dev_responsive_float_value(value, |value| format!("{}f", text_spacing_em(*value)))
        );
    }

    if title {
        let fallback = format!("{}f", text_typography(true, TextSize::Md).letter_spacing_em);
        props
            .size
            .as_ref()
            .map(|value| {
                dev_responsive_float_value(value, |value| {
                    format!("{}f", text_typography(true, *value).letter_spacing_em)
                })
            })
            .map(|value| format!("doweTextSize({value}, {fallback})"))
            .unwrap_or(fallback)
    } else {
        "0f".to_string()
    }
}

fn dev_optional_size(value: Option<&ResponsiveValue<SizeValue>>) -> String {
    value
        .map(dev_size_value)
        .unwrap_or_else(|| "null".to_string())
}

fn dev_drawer_radius(props: &StyleProps) -> String {
    props
        .rounded
        .as_ref()
        .map(|value| {
            format!(
                "doweFloat({}, 0f)",
                dev_responsive_float_value(value, |value| format!("{}f", rounded_dp(*value)))
            )
        })
        .unwrap_or_else(|| "0f".to_string())
}

fn dev_style_radius(props: &StyleProps) -> String {
    props
        .rounded
        .as_ref()
        .map(|value| {
            format!(
                "doweFloat({}, DOWE_RADIUS)",
                dev_responsive_float_value(value, |value| format!("{}f", rounded_dp(*value)))
            )
        })
        .unwrap_or_else(|| "DOWE_RADIUS".to_string())
}

fn dev_rounded_value(value: &ResponsiveValue<RoundedSize>) -> String {
    dev_responsive_float_value(value, |value| format!("{}f", rounded_dp(*value)))
}

fn dev_border_value(value: &ResponsiveValue<BorderWidth>) -> String {
    dev_responsive_value(value, |value| value.0.to_string())
}

fn dev_scale_value(value: &ResponsiveValue<ScaleValue>) -> String {
    dev_responsive_value(value, |value| value.native_units().to_string())
}

fn dev_size_value(value: &ResponsiveValue<SizeValue>) -> String {
    dev_responsive_value(value, |value| match value {
        SizeValue::Scale(value) => value.native_units().to_string(),
        SizeValue::Container(value) => value.scale_value().native_units().to_string(),
        SizeValue::Percent(value) => format!("dowePercentSize({value})"),
        SizeValue::Full => "ViewGroup.LayoutParams.MATCH_PARENT".to_string(),
        SizeValue::Auto => "ViewGroup.LayoutParams.WRAP_CONTENT".to_string(),
        SizeValue::ViewportMinus(value) => {
            format!("runtime.doweViewportHeight({})", value.native_units())
        }
    })
}

fn dev_section_exact_height(value: &ResponsiveValue<SizeValue>) -> Option<String> {
    if value
        .entries
        .iter()
        .all(|entry| matches!(entry.value, SizeValue::ViewportMinus(_)))
    {
        Some(format!("runtime.doweDp({})", dev_size_value(value)))
    } else {
        None
    }
}

fn android_section_bounded_size(
    value: &ResponsiveValue<SizeValue>,
    spacing: &dowe_components::SpacingProps,
) -> ResponsiveValue<SizeValue> {
    let effective_spacing = dowe_components::section_content_spacing(spacing);
    ResponsiveValue::ordered(
        value
            .entries
            .iter()
            .map(|entry| {
                let top = android_section_spacing_edge(&effective_spacing, entry.breakpoint, true);
                let bottom =
                    android_section_spacing_edge(&effective_spacing, entry.breakpoint, false);
                let vertical_inset = top.saturating_add(bottom);
                let value = match entry.value {
                    SizeValue::ViewportMinus(inset) => SizeValue::ViewportMinus(
                        ScaleValue::from_half_steps(inset.0.saturating_add(vertical_inset)),
                    ),
                    value => value,
                };
                dowe_components::ResponsiveEntry {
                    breakpoint: entry.breakpoint,
                    value,
                }
            })
            .collect(),
    )
}

fn android_section_spacing_edge(
    spacing: &dowe_components::SpacingProps,
    breakpoint: Breakpoint,
    top: bool,
) -> u16 {
    let value = if let Some(all) = spacing.p.as_ref() {
        android_responsive_scale_at(all, breakpoint)
    } else if top {
        spacing
            .pt
            .as_ref()
            .or(spacing.py.as_ref())
            .and_then(|value| android_responsive_scale_at(value, breakpoint))
    } else {
        spacing
            .pb
            .as_ref()
            .or(spacing.py.as_ref())
            .and_then(|value| android_responsive_scale_at(value, breakpoint))
    };
    value.map(|value| value.0).unwrap_or_default()
}

fn android_responsive_scale_at(
    value: &ResponsiveValue<ScaleValue>,
    breakpoint: Breakpoint,
) -> Option<ScaleValue> {
    value
        .entries
        .iter()
        .rev()
        .find(|entry| entry.breakpoint.min_width() <= breakpoint.min_width())
        .map(|entry| entry.value)
}

fn dev_color_value(value: &ResponsiveValue<ColorToken>) -> String {
    dev_responsive_value(value, |value| java_color(*value).to_string())
}

fn dev_section_background_value(value: &ResponsiveValue<SectionBackground>) -> String {
    dev_responsive_string_value(value, |value| format!("\"{}\"", value.as_str()))
}

fn dev_font_value(value: Option<&ResponsiveValue<FontFamily>>) -> String {
    value
        .map(|value| {
            format!(
                "doweFontName({})",
                dev_responsive_string_value(value, |value| {
                    format!("\"{}\"", font_display_name(*value))
                })
            )
        })
        .unwrap_or_else(|| "doweFontName(null)".to_string())
}

fn dev_bool_value(value: &ResponsiveValue<bool>) -> String {
    format!(
        "doweResponsiveBool(viewportWidth, {})",
        dev_responsive_args(value, |value| value.to_string())
    )
}

fn dev_text_size_expr(title: bool, value: TextSize) -> String {
    let size = text_typography(title, value).font_size;
    format!(
        "doweFluidTextSize({}f, {}f, {}f, {}f)",
        size.min, size.preferred_base, size.preferred_viewport, size.max
    )
}

fn dev_responsive_value<T, F>(value: &ResponsiveValue<T>, map: F) -> String
where
    F: Fn(&T) -> String,
{
    format!(
        "doweResponsiveInt(viewportWidth, {})",
        dev_responsive_args(value, map)
    )
}

fn dev_responsive_float_value<T, F>(value: &ResponsiveValue<T>, map: F) -> String
where
    F: Fn(&T) -> String,
{
    format!(
        "doweResponsiveFloat(viewportWidth, {})",
        dev_responsive_args(value, map)
    )
}

fn dev_responsive_string_value<T, F>(value: &ResponsiveValue<T>, map: F) -> String
where
    F: Fn(&T) -> String,
{
    format!(
        "doweResponsiveString(viewportWidth, {})",
        dev_responsive_args(value, map)
    )
}

fn dev_responsive_args<T, F>(value: &ResponsiveValue<T>, map: F) -> String
where
    F: Fn(&T) -> String,
{
    [
        Breakpoint::Xs,
        Breakpoint::Sm,
        Breakpoint::Md,
        Breakpoint::Lg,
        Breakpoint::Xl,
    ]
    .into_iter()
    .map(|breakpoint| {
        value
            .entries
            .iter()
            .find(|entry| entry.breakpoint == breakpoint)
            .map(|entry| map(&entry.value))
            .unwrap_or_else(|| "null".to_string())
    })
    .collect::<Vec<_>>()
    .join(", ")
}

fn color_ref(value: ColorToken) -> &'static str {
    match value.as_str() {
        "white" => "Color.White",
        "black" => "Color.Black",
        "transparent" => "Color.Transparent",
        _ => intern_generated_color_name(format!("DoweDesign.{}", value.as_str())),
    }
}

