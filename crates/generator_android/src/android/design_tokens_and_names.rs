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
        ComponentVariant::Solid => color_ref(family_on_color(color)),
        ComponentVariant::Soft => color_ref(family_on_soft_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            color_ref(family_color(color))
        }
    }
}

fn nav_active_content(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Ghost) {
        ComponentVariant::Solid => color_ref(family_on_color(color)),
        ComponentVariant::Soft => color_ref(family_on_soft_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost
            if matches!(color, ColorFamily::Background | ColorFamily::Surface) =>
        {
            color_ref(family_on_color(color))
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
            ColorFamily::Background => color_ref(ColorToken::OnBackground),
            _ => color_ref(ColorToken::OnSurface),
        },
        ComponentVariant::Ghost
            if matches!(color, ColorFamily::Background | ColorFamily::Surface) =>
        {
            color_ref(family_on_color(color))
        }
        _ => variant_content(props),
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
        ComponentVariant::Solid => java_color(family_on_color(color)),
        ComponentVariant::Soft => java_color(family_on_soft_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            java_color(family_color(color))
        }
    }
}

fn dev_nav_active_content(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Ghost) {
        ComponentVariant::Solid => java_color(family_on_color(color)),
        ComponentVariant::Soft => java_color(family_on_soft_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost
            if matches!(color, ColorFamily::Background | ColorFamily::Surface) =>
        {
            java_color(family_on_color(color))
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
            ColorFamily::Background => java_color(ColorToken::OnBackground),
            _ => java_color(ColorToken::OnSurface),
        },
        ComponentVariant::Ghost
            if matches!(color, ColorFamily::Background | ColorFamily::Surface) =>
        {
            java_color(family_on_color(color))
        }
        _ => dev_variant_content(props),
    }
}

fn dev_card_border(props: &VariantProps) -> &'static str {
    if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        dev_variant_content(props)
    } else {
        "null"
    }
}

fn tabs_list_background(props: &TabsProps) -> &'static str {
    match props.variant {
        TabsVariant::Solid | TabsVariant::Pills => color_ref(family_soft_color(props.color)),
        TabsVariant::Outlined | TabsVariant::Line | TabsVariant::Ghost => "Color.Transparent",
    }
}

fn tabs_list_content(props: &TabsProps) -> &'static str {
    match props.variant {
        TabsVariant::Solid | TabsVariant::Pills => color_ref(family_on_soft_color(props.color)),
        TabsVariant::Outlined | TabsVariant::Line | TabsVariant::Ghost => {
            color_ref(tabs_accent_token(props.color))
        }
    }
}

fn tabs_active_background(props: &TabsProps) -> &'static str {
    if props.color == ColorFamily::Muted {
        color_ref(family_on_color(props.color))
    } else {
        color_ref(family_color(props.color))
    }
}

fn tabs_active_content(props: &TabsProps) -> &'static str {
    if props.color == ColorFamily::Muted {
        color_ref(family_color(props.color))
    } else {
        color_ref(family_on_color(props.color))
    }
}

fn tabs_accent(props: &TabsProps) -> &'static str {
    color_ref(tabs_accent_token(props.color))
}

fn tabs_border(props: &TabsProps) -> &'static str {
    match props.variant {
        TabsVariant::Outlined => color_ref(ColorToken::Muted),
        TabsVariant::Line => tabs_accent(props),
        TabsVariant::Solid | TabsVariant::Ghost | TabsVariant::Pills => "null",
    }
}

fn tabs_accent_token(value: ColorFamily) -> ColorToken {
    match value {
        ColorFamily::Muted | ColorFamily::Background | ColorFamily::Surface => {
            family_on_color(value)
        }
        _ => family_color(value),
    }
}

fn dev_tabs_list_background(props: &TabsProps) -> &'static str {
    match props.variant {
        TabsVariant::Solid | TabsVariant::Pills => java_color(family_soft_color(props.color)),
        TabsVariant::Outlined | TabsVariant::Line | TabsVariant::Ghost => "Color.TRANSPARENT",
    }
}

fn dev_tabs_list_content(props: &TabsProps) -> &'static str {
    match props.variant {
        TabsVariant::Solid | TabsVariant::Pills => java_color(family_on_soft_color(props.color)),
        TabsVariant::Outlined | TabsVariant::Line | TabsVariant::Ghost => {
            java_color(tabs_accent_token(props.color))
        }
    }
}

fn dev_tabs_active_background(props: &TabsProps) -> &'static str {
    if props.color == ColorFamily::Muted {
        java_color(family_on_color(props.color))
    } else {
        java_color(family_color(props.color))
    }
}

fn dev_tabs_active_content(props: &TabsProps) -> &'static str {
    if props.color == ColorFamily::Muted {
        java_color(family_color(props.color))
    } else {
        java_color(family_on_color(props.color))
    }
}

fn dev_tabs_accent(props: &TabsProps) -> &'static str {
    java_color(tabs_accent_token(props.color))
}

fn dev_tabs_border(props: &TabsProps) -> &'static str {
    match props.variant {
        TabsVariant::Outlined => java_color(ColorToken::Muted),
        TabsVariant::Line => dev_tabs_accent(props),
        TabsVariant::Solid | TabsVariant::Ghost | TabsVariant::Pills => "null",
    }
}

fn dev_text_color(props: &TextProps, inherited_color: Option<&str>) -> String {
    let fallback = inherited_color.unwrap_or("DOWE_ON_BACKGROUND");
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

fn dev_scale_value(value: &ResponsiveValue<ScaleValue>) -> String {
    dev_responsive_value(value, |value| value.native_units().to_string())
}

fn dev_size_value(value: &ResponsiveValue<SizeValue>) -> String {
    dev_responsive_value(value, |value| match value {
        SizeValue::Scale(value) => value.native_units().to_string(),
        SizeValue::Full => "ViewGroup.LayoutParams.MATCH_PARENT".to_string(),
    })
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
    match value {
        ColorToken::Primary => "DoweDesign.primary",
        ColorToken::OnPrimary => "DoweDesign.onPrimary",
        ColorToken::Secondary => "DoweDesign.secondary",
        ColorToken::OnSecondary => "DoweDesign.onSecondary",
        ColorToken::Tertiary => "DoweDesign.tertiary",
        ColorToken::OnTertiary => "DoweDesign.onTertiary",
        ColorToken::Muted => "DoweDesign.muted",
        ColorToken::OnMuted => "DoweDesign.onMuted",
        ColorToken::Background => "DoweDesign.background",
        ColorToken::OnBackground => "DoweDesign.onBackground",
        ColorToken::Surface => "DoweDesign.surface",
        ColorToken::OnSurface => "DoweDesign.onSurface",
        ColorToken::Success => "DoweDesign.success",
        ColorToken::OnSuccess => "DoweDesign.onSuccess",
        ColorToken::Info => "DoweDesign.info",
        ColorToken::OnInfo => "DoweDesign.onInfo",
        ColorToken::Warning => "DoweDesign.warning",
        ColorToken::OnWarning => "DoweDesign.onWarning",
        ColorToken::Danger => "DoweDesign.danger",
        ColorToken::OnDanger => "DoweDesign.onDanger",
        ColorToken::SoftPrimary => "DoweDesign.softPrimary",
        ColorToken::OnSoftPrimary => "DoweDesign.onSoftPrimary",
        ColorToken::SoftSecondary => "DoweDesign.softSecondary",
        ColorToken::OnSoftSecondary => "DoweDesign.onSoftSecondary",
        ColorToken::SoftTertiary => "DoweDesign.softTertiary",
        ColorToken::OnSoftTertiary => "DoweDesign.onSoftTertiary",
        ColorToken::SoftMuted => "DoweDesign.softMuted",
        ColorToken::OnSoftMuted => "DoweDesign.onSoftMuted",
        ColorToken::SoftSuccess => "DoweDesign.softSuccess",
        ColorToken::OnSoftSuccess => "DoweDesign.onSoftSuccess",
        ColorToken::SoftInfo => "DoweDesign.softInfo",
        ColorToken::OnSoftInfo => "DoweDesign.onSoftInfo",
        ColorToken::SoftWarning => "DoweDesign.softWarning",
        ColorToken::OnSoftWarning => "DoweDesign.onSoftWarning",
        ColorToken::SoftDanger => "DoweDesign.softDanger",
        ColorToken::OnSoftDanger => "DoweDesign.onSoftDanger",
    }
}

fn android_design_block(theme: &DesignTheme) -> String {
    let mut output = String::from("object DoweDesign {\n");
    for token in ColorToken::all() {
        output.push_str(&format!(
            "    val {} = {}\n",
            token.as_str(),
            android_color_literal(theme.color_value(*token))
        ));
    }
    output.push_str(&format!(
        "    val radius = {}.dp\n    val radiusBox = {}.dp\n    val radiusUi = {}.dp\n}}\n",
        theme.radii.radius, theme.radii.radius_box, theme.radii.radius_ui
    ));
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
    val radius: Dp,
    val radiusBox: Dp,
    val radiusUi: Dp
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
    let colors = ColorToken::all()
        .iter()
        .map(|token| {
            format!(
                "            \"{}\" to {},",
                token.as_str(),
                android_color_literal(theme.color_value(*token))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "        DoweGeneratedTheme(name = \"{}\", colors = mapOf(\n{}\n        ), radius = {}.dp, radiusBox = {}.dp, radiusUi = {}.dp),",
        escape_kotlin(&theme.name),
        colors,
        theme.radii.radius,
        theme.radii.radius_box,
        theme.radii.radius_ui
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
    if raw.len() == 6 {
        format!("Color.rgb({red}, {green}, {blue})")
    } else {
        let alpha = u8::from_str_radix(&raw[6..8], 16).expect("alpha color");
        format!("Color.argb({alpha}, {red}, {green}, {blue})")
    }
}

fn font_display_name(value: FontFamily) -> &'static str {
    value.catalog_entry().android_family_name
}

fn dev_design_constants(theme: &DesignTheme) -> String {
    let mut output = String::new();
    for token in ColorToken::all() {
        output.push_str(&format!(
            "    private static final int {} = {};\n",
            java_color(*token),
            android_java_color_literal(theme.color_value(*token))
        ));
    }
    output.push_str(&format!(
        "    private static final float DOWE_RADIUS = {}f;\n    private static final float DOWE_RADIUS_BOX = {}f;\n    private static final float DOWE_RADIUS_UI = {}f;\n",
        theme.radii.radius, theme.radii.radius_box, theme.radii.radius_ui
    ));
    output
}

fn java_color(value: ColorToken) -> &'static str {
    match value {
        ColorToken::Primary => "DOWE_PRIMARY",
        ColorToken::OnPrimary => "DOWE_ON_PRIMARY",
        ColorToken::Secondary => "DOWE_SECONDARY",
        ColorToken::OnSecondary => "DOWE_ON_SECONDARY",
        ColorToken::Tertiary => "DOWE_TERTIARY",
        ColorToken::OnTertiary => "DOWE_ON_TERTIARY",
        ColorToken::Muted => "DOWE_MUTED",
        ColorToken::OnMuted => "DOWE_ON_MUTED",
        ColorToken::Background => "DOWE_BACKGROUND",
        ColorToken::OnBackground => "DOWE_ON_BACKGROUND",
        ColorToken::Surface => "DOWE_SURFACE",
        ColorToken::OnSurface => "DOWE_ON_SURFACE",
        ColorToken::Success => "DOWE_SUCCESS",
        ColorToken::OnSuccess => "DOWE_ON_SUCCESS",
        ColorToken::Info => "DOWE_INFO",
        ColorToken::OnInfo => "DOWE_ON_INFO",
        ColorToken::Warning => "DOWE_WARNING",
        ColorToken::OnWarning => "DOWE_ON_WARNING",
        ColorToken::Danger => "DOWE_DANGER",
        ColorToken::OnDanger => "DOWE_ON_DANGER",
        ColorToken::SoftPrimary => "DOWE_SOFT_PRIMARY",
        ColorToken::OnSoftPrimary => "DOWE_ON_SOFT_PRIMARY",
        ColorToken::SoftSecondary => "DOWE_SOFT_SECONDARY",
        ColorToken::OnSoftSecondary => "DOWE_ON_SOFT_SECONDARY",
        ColorToken::SoftTertiary => "DOWE_SOFT_TERTIARY",
        ColorToken::OnSoftTertiary => "DOWE_ON_SOFT_TERTIARY",
        ColorToken::SoftMuted => "DOWE_SOFT_MUTED",
        ColorToken::OnSoftMuted => "DOWE_ON_SOFT_MUTED",
        ColorToken::SoftSuccess => "DOWE_SOFT_SUCCESS",
        ColorToken::OnSoftSuccess => "DOWE_ON_SOFT_SUCCESS",
        ColorToken::SoftInfo => "DOWE_SOFT_INFO",
        ColorToken::OnSoftInfo => "DOWE_ON_SOFT_INFO",
        ColorToken::SoftWarning => "DOWE_SOFT_WARNING",
        ColorToken::OnSoftWarning => "DOWE_ON_SOFT_WARNING",
        ColorToken::SoftDanger => "DOWE_SOFT_DANGER",
        ColorToken::OnSoftDanger => "DOWE_ON_SOFT_DANGER",
    }
}

fn family_color(value: ColorFamily) -> ColorToken {
    match value {
        ColorFamily::Primary => ColorToken::Primary,
        ColorFamily::Secondary => ColorToken::Secondary,
        ColorFamily::Tertiary => ColorToken::Tertiary,
        ColorFamily::Muted => ColorToken::Muted,
        ColorFamily::Background => ColorToken::Background,
        ColorFamily::Surface => ColorToken::Surface,
        ColorFamily::Success => ColorToken::Success,
        ColorFamily::Info => ColorToken::Info,
        ColorFamily::Warning => ColorToken::Warning,
        ColorFamily::Danger => ColorToken::Danger,
    }
}

fn family_on_color(value: ColorFamily) -> ColorToken {
    match value {
        ColorFamily::Primary => ColorToken::OnPrimary,
        ColorFamily::Secondary => ColorToken::OnSecondary,
        ColorFamily::Tertiary => ColorToken::OnTertiary,
        ColorFamily::Muted => ColorToken::OnMuted,
        ColorFamily::Background => ColorToken::OnBackground,
        ColorFamily::Surface => ColorToken::OnSurface,
        ColorFamily::Success => ColorToken::OnSuccess,
        ColorFamily::Info => ColorToken::OnInfo,
        ColorFamily::Warning => ColorToken::OnWarning,
        ColorFamily::Danger => ColorToken::OnDanger,
    }
}

fn family_soft_color(value: ColorFamily) -> ColorToken {
    match value {
        ColorFamily::Primary => ColorToken::SoftPrimary,
        ColorFamily::Secondary => ColorToken::SoftSecondary,
        ColorFamily::Tertiary => ColorToken::SoftTertiary,
        ColorFamily::Muted => ColorToken::SoftMuted,
        ColorFamily::Background => ColorToken::Background,
        ColorFamily::Surface => ColorToken::Surface,
        ColorFamily::Success => ColorToken::SoftSuccess,
        ColorFamily::Info => ColorToken::SoftInfo,
        ColorFamily::Warning => ColorToken::SoftWarning,
        ColorFamily::Danger => ColorToken::SoftDanger,
    }
}

fn family_on_soft_color(value: ColorFamily) -> ColorToken {
    match value {
        ColorFamily::Primary => ColorToken::OnSoftPrimary,
        ColorFamily::Secondary => ColorToken::OnSoftSecondary,
        ColorFamily::Tertiary => ColorToken::OnSoftTertiary,
        ColorFamily::Muted => ColorToken::OnSoftMuted,
        ColorFamily::Background => ColorToken::OnBackground,
        ColorFamily::Surface => ColorToken::OnSurface,
        ColorFamily::Success => ColorToken::OnSoftSuccess,
        ColorFamily::Info => ColorToken::OnSoftInfo,
        ColorFamily::Warning => ColorToken::OnSoftWarning,
        ColorFamily::Danger => ColorToken::OnSoftDanger,
    }
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
