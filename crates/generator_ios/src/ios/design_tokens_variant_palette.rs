fn variant_container(props: &VariantProps) -> &'static str {
    let roles = dowe_components::variant_visual_roles(
        props.variant.unwrap_or(ComponentVariant::Solid),
        props.color.unwrap_or(ColorFamily::Primary),
    );
    color_ref(roles.background)
}

fn variant_content(props: &VariantProps) -> &'static str {
    let roles = dowe_components::variant_visual_roles(
        props.variant.unwrap_or(ComponentVariant::Solid),
        props.color.unwrap_or(ColorFamily::Primary),
    );
    color_ref(roles.content)
}

fn variant_title(props: &VariantProps) -> &'static str {
    let roles = dowe_components::variant_visual_roles(
        props.variant.unwrap_or(ComponentVariant::Solid),
        props.color.unwrap_or(ColorFamily::Primary),
    );
    color_ref(roles.title)
}

fn variant_border(props: &VariantProps) -> &'static str {
    if let Some(family) = props.style.border_color {
        return color_ref(family_color(family));
    }
    let roles = dowe_components::variant_visual_roles(
        props.variant.unwrap_or(ComponentVariant::Solid),
        props.color.unwrap_or(ColorFamily::Primary),
    );
    color_ref(roles.border.unwrap_or(ColorToken::Transparent))
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
    color_ref(
        dowe_components::card_variant_visual_roles(
            props.variant.unwrap_or(ComponentVariant::Solid),
            props.color.unwrap_or(ColorFamily::Primary),
        )
        .background,
    )
}

fn card_variant_content(props: &VariantProps) -> &'static str {
    color_ref(
        dowe_components::card_variant_visual_roles(
            props.variant.unwrap_or(ComponentVariant::Solid),
            props.color.unwrap_or(ColorFamily::Primary),
        )
        .content,
    )
}

fn card_variant_title(props: &VariantProps) -> &'static str {
    color_ref(
        dowe_components::card_variant_visual_roles(
            props.variant.unwrap_or(ComponentVariant::Solid),
            props.color.unwrap_or(ColorFamily::Primary),
        )
        .title,
    )
}

fn card_variant_border(props: &VariantProps) -> &'static str {
    if let Some(family) = props.style.border_color {
        return color_ref(family_color(family));
    }
    let roles = dowe_components::card_variant_visual_roles(
        props.variant.unwrap_or(ComponentVariant::Solid),
        props.color.unwrap_or(ColorFamily::Primary),
    );
    color_ref(roles.border.unwrap_or_else(|| {
        props.color.unwrap_or(ColorFamily::Primary).color_token()
    }))
}

fn table_variant_container(props: &VariantProps) -> &'static str {
    let color = props.color.unwrap_or(ColorFamily::Surface);
    match props.variant.unwrap_or(ComponentVariant::Solid) {
        ComponentVariant::Solid => color_ref(family_color(color)),
        ComponentVariant::Outlined | ComponentVariant::Line | ComponentVariant::Ghost => {
            "Color.clear"
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

fn tabs_list_background(props: &TabsProps) -> &'static str {
    match props.variant {
        TabsVariant::Solid | TabsVariant::Pills => color_ref(family_color(props.color)),
        TabsVariant::Outlined | TabsVariant::Line | TabsVariant::Ghost | TabsVariant::Stepper => {
            "Color.clear"
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

fn tabs_border(props: &TabsProps) -> String {
    match props.variant {
        TabsVariant::Outlined => format!("Optional({})", color_ref(ColorToken::Muted)),
        TabsVariant::Line => format!("Optional({})", tabs_accent(props)),
        TabsVariant::Solid | TabsVariant::Ghost | TabsVariant::Pills | TabsVariant::Stepper => {
            "nil".to_string()
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
