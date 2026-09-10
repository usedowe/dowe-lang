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
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            "Color.Transparent"
        }
    }
}

fn variant_content(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => color_ref(family_text_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            color_ref(family_color(color))
        }
    }
}

fn variant_title(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => color_ref(family_title_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            color_ref(family_color(color))
        }
    }
}

fn scheme_title(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => color_ref(family_title_color(color)),
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
        ComponentVariant::Ghost if matches!(color, ColorFamily::Background | ColorFamily::Surface) => color_ref(family_text_color(color)),
        _ => variant_content(props),
    }
}

fn card_surface_container(props: &VariantProps) -> &'static str { variant_container(props) }
fn card_surface_content(props: &VariantProps) -> &'static str { variant_content(props) }
fn card_surface_title(props: &VariantProps) -> &'static str { variant_title(props) }

fn table_variant_container(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Surface);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => color_ref(family_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            "Color.Transparent"
        }
    }
}

fn table_variant_content(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Surface);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => color_ref(family_text_color(color)),
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
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            "Color.TRANSPARENT"
        }
    }
}

fn dev_variant_content(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => java_color(family_text_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            java_color(family_color(color))
        }
    }
}

fn dev_variant_title(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => java_color(family_title_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            java_color(family_color(color))
        }
    }
}

fn dev_scheme_title(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Primary);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => java_color(family_title_color(color)),
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
    dev_variant_container(props)
}

fn dev_card_variant_content(props: &VariantProps) -> &'static str {
    dev_variant_content(props)
}

fn dev_card_variant_title(props: &VariantProps) -> &'static str {
    dev_variant_title(props)
}

fn dev_table_variant_container(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Surface);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => java_color(family_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            "Color.TRANSPARENT"
        }
    }
}

fn dev_table_variant_content(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Surface);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => java_color(family_text_color(color)),
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
    if props.style.border.is_some() {
        "null"
    } else if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
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
        TabsVariant::Solid | TabsVariant::Pills => color_ref(family_color(props.color)),
        TabsVariant::Outlined | TabsVariant::Line | TabsVariant::Ghost | TabsVariant::Stepper => {
            "Color.Transparent"
        }
    }
}

fn tabs_list_content(props: &TabsProps) -> &'static str {
    match props.variant {
        TabsVariant::Solid | TabsVariant::Pills => color_ref(family_text_color(props.color)),
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
        TabsVariant::Solid | TabsVariant::Pills => java_color(family_color(props.color)),
        TabsVariant::Outlined | TabsVariant::Line | TabsVariant::Ghost | TabsVariant::Stepper => {
            "Color.TRANSPARENT"
        }
    }
}

fn dev_tabs_list_content(props: &TabsProps) -> &'static str {
    match props.variant {
        TabsVariant::Solid | TabsVariant::Pills => java_color(family_text_color(props.color)),
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

