fn dev_text_weight_value(value: TextWeight) -> &'static str {
    match value {
        TextWeight::Thin => "100",
        TextWeight::Extralight => "200",
        TextWeight::Light => "300",
        TextWeight::Regular => "400",
        TextWeight::Medium => "500",
        TextWeight::Semibold => "600",
        TextWeight::Bold => "700",
        TextWeight::Extrabold => "800",
        TextWeight::Black => "900",
    }
}

fn variant_container(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => color_ref(family_color(color)),
        ComponentVariant::Soft => color_ref(family_soft_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            "Color.Transparent"
        }
    }
}

fn variant_content(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => color_ref(family_text_color(color)),
        ComponentVariant::Soft => color_ref(family_soft_text_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            color_ref(family_color(color))
        }
    }
}

fn variant_title(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => color_ref(family_title_color(color)),
        ComponentVariant::Soft => color_ref(family_soft_title_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            color_ref(family_color(color))
        }
    }
}

fn scheme_title(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Soft => color_ref(family_soft_title_color(color)),
        _ => color_ref(family_title_color(color)),
    }
}

fn side_nav_header_content(props: &VariantProps) -> &'static str {
    color_ref(family_color(props.color.unwrap_or(ColorFamily::Primary)))
}

fn nav_active_content(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Ghost) {
        ComponentVariant::Solid => color_ref(family_text_color(color)),
        ComponentVariant::Soft => color_ref(family_soft_text_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost
            if matches!(color, ColorFamily::Background | ColorFamily::Surface) =>
        {
            color_ref(family_text_color(color))
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
            ColorFamily::Background => color_ref(ColorToken::BackgroundText),
            _ => color_ref(ColorToken::SurfaceText),
        },
        ComponentVariant::Ghost
            if matches!(color, ColorFamily::Background | ColorFamily::Surface) =>
        {
            color_ref(family_text_color(color))
        }
        _ => variant_content(props),
    }
}

fn card_variant_title(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Outlined => match color {
            ColorFamily::Background => color_ref(ColorToken::BackgroundTitle),
            _ => color_ref(ColorToken::SurfaceTitle),
        },
        ComponentVariant::Ghost
            if matches!(color, ColorFamily::Background | ColorFamily::Surface) =>
        {
            color_ref(family_text_color(color))
        }
        ComponentVariant::Solid | ComponentVariant::Soft => variant_title(props),
        ComponentVariant::Line | ComponentVariant::Ghost => variant_content(props),
    }
}

fn table_variant_container(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Surface);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => color_ref(family_color(color)),
        ComponentVariant::Soft => color_ref(family_soft_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            "Color.Transparent"
        }
    }
}

fn table_variant_content(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Surface);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => color_ref(family_text_color(color)),
        ComponentVariant::Soft => color_ref(family_soft_text_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost
            if matches!(color, ColorFamily::Background | ColorFamily::Surface) =>
        {
            color_ref(family_text_color(color))
        }
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            color_ref(family_color(color))
        }
    }
}

fn dev_variant_container(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => java_color(family_color(color)),
        ComponentVariant::Soft => java_color(family_soft_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            "Color.TRANSPARENT"
        }
    }
}

fn dev_variant_content(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => java_color(family_text_color(color)),
        ComponentVariant::Soft => java_color(family_soft_text_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            java_color(family_color(color))
        }
    }
}

fn dev_variant_title(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => java_color(family_title_color(color)),
        ComponentVariant::Soft => java_color(family_soft_title_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            java_color(family_color(color))
        }
    }
}

fn dev_scheme_title(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Soft => java_color(family_soft_title_color(color)),
        _ => java_color(family_title_color(color)),
    }
}

fn dev_side_nav_header_content(props: &VariantProps) -> &'static str {
    java_color(family_color(props.color.unwrap_or(ColorFamily::Primary)))
}

fn dev_nav_active_content(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Ghost) {
        ComponentVariant::Solid => java_color(family_text_color(color)),
        ComponentVariant::Soft => java_color(family_soft_text_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost
            if matches!(color, ColorFamily::Background | ColorFamily::Surface) =>
        {
            java_color(family_text_color(color))
        }
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            java_color(family_color(color))
        }
    }
}

fn dev_card_variant_container(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Outlined => match color {
            ColorFamily::Background => java_color(ColorToken::Background),
            _ => java_color(ColorToken::Surface),
        },
        _ => dev_variant_container(props),
    }
}

fn dev_card_variant_content(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Outlined => match color {
            ColorFamily::Background => java_color(ColorToken::BackgroundText),
            _ => java_color(ColorToken::SurfaceText),
        },
        ComponentVariant::Ghost
            if matches!(color, ColorFamily::Background | ColorFamily::Surface) =>
        {
            java_color(family_text_color(color))
        }
        _ => dev_variant_content(props),
    }
}

fn dev_card_variant_title(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Outlined => match color {
            ColorFamily::Background => java_color(ColorToken::BackgroundTitle),
            _ => java_color(ColorToken::SurfaceTitle),
        },
        ComponentVariant::Ghost
            if matches!(color, ColorFamily::Background | ColorFamily::Surface) =>
        {
            java_color(family_text_color(color))
        }
        ComponentVariant::Solid | ComponentVariant::Soft => dev_variant_title(props),
        ComponentVariant::Line | ComponentVariant::Ghost => dev_variant_content(props),
    }
}

fn dev_table_variant_container(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Surface);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => java_color(family_color(color)),
        ComponentVariant::Soft => java_color(family_soft_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            "Color.TRANSPARENT"
        }
    }
}

fn dev_table_variant_content(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Surface);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => java_color(family_text_color(color)),
        ComponentVariant::Soft => java_color(family_soft_text_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost
            if matches!(color, ColorFamily::Background | ColorFamily::Surface) =>
        {
            java_color(family_text_color(color))
        }
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            java_color(family_color(color))
        }
    }
}

fn dev_table_border(props: &VariantProps) -> &'static str {
    if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        dev_table_variant_content(props)
    } else {
        "null"
    }
}

fn dev_card_border(props: &VariantProps) -> &'static str {
    if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        dev_variant_content(props)
    } else {
        "null"
    }
}

fn dev_button_border(props: &VariantProps) -> &'static str {
    if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        dev_variant_content(props)
    } else {
        "null"
    }
}

fn tabs_list_background(props: &TabsProps) -> &'static str {
    match props.variant {
        TabsVariant::Solid | TabsVariant::Pills => color_ref(family_soft_color(props.color)),
        TabsVariant::Outlined | TabsVariant::Line | TabsVariant::Ghost | TabsVariant::Stepper => {
            "Color.Transparent"
        }
    }
}

fn tabs_list_content(props: &TabsProps) -> &'static str {
    match props.variant {
        TabsVariant::Solid | TabsVariant::Pills => color_ref(family_soft_text_color(props.color)),
        TabsVariant::Outlined | TabsVariant::Line | TabsVariant::Ghost | TabsVariant::Stepper => {
            color_ref(tabs_accent_token(props.color))
        }
    }
}

fn tabs_active_background(props: &TabsProps) -> &'static str {
    if props.color == ColorFamily::Muted {
        color_ref(family_text_color(props.color))
    } else {
        color_ref(family_color(props.color))
    }
}

fn tabs_active_content(props: &TabsProps) -> &'static str {
    if props.color == ColorFamily::Muted {
        color_ref(family_color(props.color))
    } else {
        color_ref(family_text_color(props.color))
    }
}

fn tabs_accent(props: &TabsProps) -> &'static str {
    color_ref(tabs_accent_token(props.color))
}

fn tabs_border(props: &TabsProps) -> &'static str {
    match props.variant {
        TabsVariant::Outlined => color_ref(ColorToken::Muted),
        TabsVariant::Line => tabs_accent(props),
        TabsVariant::Solid | TabsVariant::Ghost | TabsVariant::Pills | TabsVariant::Stepper => {
            "null"
        }
    }
}

fn tabs_accent_token(value: ColorFamily) -> ColorToken {
    match value {
        ColorFamily::Muted | ColorFamily::Background | ColorFamily::Surface => {
            family_text_color(value)
        }
        _ => family_color(value),
    }
}

fn dev_tabs_list_background(props: &TabsProps) -> &'static str {
    match props.variant {
        TabsVariant::Solid | TabsVariant::Pills => java_color(family_soft_color(props.color)),
        TabsVariant::Outlined | TabsVariant::Line | TabsVariant::Ghost | TabsVariant::Stepper => {
            "Color.TRANSPARENT"
        }
    }
}

fn dev_tabs_list_content(props: &TabsProps) -> &'static str {
    match props.variant {
        TabsVariant::Solid | TabsVariant::Pills => java_color(family_soft_text_color(props.color)),
        TabsVariant::Outlined | TabsVariant::Line | TabsVariant::Ghost | TabsVariant::Stepper => {
            java_color(tabs_accent_token(props.color))
        }
    }
}

fn dev_tabs_active_background(props: &TabsProps) -> &'static str {
    if props.color == ColorFamily::Muted {
        java_color(family_text_color(props.color))
    } else {
        java_color(family_color(props.color))
    }
}

fn dev_tabs_active_content(props: &TabsProps) -> &'static str {
    if props.color == ColorFamily::Muted {
        java_color(family_color(props.color))
    } else {
        java_color(family_text_color(props.color))
    }
}

fn dev_tabs_accent(props: &TabsProps) -> &'static str {
    java_color(tabs_accent_token(props.color))
}

fn dev_tabs_border(props: &TabsProps) -> &'static str {
    match props.variant {
        TabsVariant::Outlined => java_color(ColorToken::Muted),
        TabsVariant::Line => dev_tabs_accent(props),
        TabsVariant::Solid | TabsVariant::Ghost | TabsVariant::Pills | TabsVariant::Stepper => {
            "null"
        }
    }
}

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
    intern_generated_color_name(format!("DoweDesign.{}", value.as_str()))
}

fn android_design_block(design: &DesignConfig) -> String {
    let theme = design.default_theme();
    let mut output = String::from("object DoweDesign {\n");
    output.push_str(&format!(
        "    var name by mutableStateOf(\"{}\")\n        private set\n",
        escape_kotlin(&design.default_theme)
    ));
    for token in theme.ordered_color_tokens() {
        output.push_str(&format!(
            "    var {} by mutableStateOf({})\n        private set\n",
            token.as_str(),
            android_color_literal(theme.color_value(token))
        ));
    }
    output.push_str(&format!(
        "    var radius by mutableStateOf({}.dp)\n        private set\n",
        theme.radius
    ));
    output.push_str("    fun applyTheme(name: String) {\n        val theme = DoweThemeModule.themes.firstOrNull { it.name == name } ?: DoweThemeModule.themes.first { it.name == DoweThemeModule.defaultTheme }\n        this.name = theme.name\n");
    for token in theme.ordered_color_tokens() {
        output.push_str(&format!(
            "        {} = theme.colors[\"{}\"] ?: {}\n",
            token.as_str(),
            token.as_str(),
            android_color_literal(theme.color_value(token))
        ));
    }
    output.push_str("        radius = theme.radius\n    }\n}\n");
    output
}

fn android_theme_module(design_config: &DesignConfig) -> String {
    let names = design_config
        .themes
        .iter()
        .map(|theme| format!("        \"{}\",", escape_kotlin(&theme.name)))
        .collect::<Vec<_>>()
        .join("\n");
    let themes = design_config
        .themes
        .iter()
        .map(android_theme_record)
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"package dev.dowe.generated

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

data class DoweGeneratedTheme(
    val name: String,
    val colors: Map<String, Color>,
    val radius: Dp
)

object DoweThemeModule {{
    const val generated = true
    const val defaultTheme = "{}"
    val names = listOf(
{}
    )
    val themes = listOf(
{}
    )
}}
"#,
        escape_kotlin(&design_config.default_theme),
        names,
        themes
    )
}

fn android_theme_record(theme: &DesignTheme) -> String {
    let colors = theme
        .ordered_color_tokens()
        .into_iter()
        .map(|token| {
            format!(
                "            \"{}\" to {},",
                token.as_str(),
                android_color_literal(theme.color_value(token))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "        DoweGeneratedTheme(name = \"{}\", colors = mapOf(\n{}\n        ), radius = {}.dp),",
        escape_kotlin(&theme.name),
        colors,
        theme.radius
    )
}

fn android_color_literal(value: &str) -> String {
    let raw = value.trim_start_matches('#');
    let value = if raw.len() == 6 {
        format!("FF{}", raw.to_ascii_uppercase())
    } else {
        format!(
            "{}{}",
            raw[6..8].to_ascii_uppercase(),
            raw[0..6].to_ascii_uppercase()
        )
    };
    format!("Color(0x{value})")
}

fn android_java_color_literal(value: &str) -> String {
    let raw = value.trim_start_matches('#');
    let red = u8::from_str_radix(&raw[0..2], 16).expect("red color");
    let green = u8::from_str_radix(&raw[2..4], 16).expect("green color");
    let blue = u8::from_str_radix(&raw[4..6], 16).expect("blue color");
    let alpha = if raw.len() == 6 {
        255
    } else {
        u8::from_str_radix(&raw[6..8], 16).expect("alpha color")
    };
    format!("0x{alpha:02X}{red:02X}{green:02X}{blue:02X}")
}

fn font_display_name(value: FontFamily) -> &'static str {
    value.catalog_entry().android_family_name
}

fn dev_design_constants(design: &DesignConfig) -> String {
    let theme = design.default_theme();
    let mut output = String::new();
    output.push_str(&format!(
        "    private static final String DOWE_DEFAULT_THEME = \"{}\";\n",
        escape_java(&design.default_theme)
    ));
    for token in theme.ordered_color_tokens() {
        output.push_str(&format!(
            "    private static int {};\n",
            java_color(token)
        ));
    }
    output.push_str("    private static float DOWE_RADIUS;\n");
    output.push_str("\n    private void doweApplyTheme(String name) {\n");
    for token in theme.ordered_color_tokens() {
        output.push_str(&format!(
            "        {} = {};\n",
            java_color(token),
            android_java_color_literal(theme.color_value(token))
        ));
    }
    output.push_str(&format!("        DOWE_RADIUS = {}f;\n", theme.radius));
    for (index, theme) in design.themes.iter().enumerate() {
        output.push_str(&format!(
            "        {} (\"{}\".equals(name)) {{\n",
            if index == 0 { "if" } else { "else if" },
            escape_java(&theme.name)
        ));
        for token in design.default_theme().ordered_color_tokens() {
            output.push_str(&format!(
                "            {} = {};\n",
                java_color(token),
                android_java_color_literal(theme.color_value(token))
            ));
        }
        output.push_str(&format!(
            "            DOWE_RADIUS = {}f;\n        }}\n",
            theme.radius
        ));
    }
    output.push_str("    }\n");
    output
}

fn java_color(value: ColorToken) -> &'static str {
    let mut output = String::from("DOWE_");
    for character in value.as_str().chars() {
        if character.is_ascii_uppercase() {
            output.push('_');
        }
        output.push(character.to_ascii_uppercase());
    }
    intern_generated_color_name(output)
}

fn intern_generated_color_name(value: String) -> &'static str {
    static NAMES: OnceLock<Mutex<BTreeSet<&'static str>>> = OnceLock::new();
    let names = NAMES.get_or_init(|| Mutex::new(BTreeSet::new()));
    let mut names = names.lock().expect("generated color name registry");
    if let Some(existing) = names.get(value.as_str()) {
        return existing;
    }
    let value = Box::leak(value.into_boxed_str());
    names.insert(value);
    value
}

fn family_color(value: ColorFamily) -> ColorToken {
    value.color_token()
}

fn family_text_color(value: ColorFamily) -> ColorToken {
    value.text_token()
}

fn family_title_color(value: ColorFamily) -> ColorToken {
    value.title_token()
}

fn family_soft_color(value: ColorFamily) -> ColorToken {
    value.color_token()
}

fn family_soft_text_color(value: ColorFamily) -> ColorToken {
    value.text_token()
}

fn family_soft_title_color(value: ColorFamily) -> ColorToken {
    value.title_token()
}

fn compose_screen_name(route: &str) -> String {
    format!("{}Screen", pascal_route(route))
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
