fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    chars
        .next()
        .map(|first| first.to_ascii_uppercase().to_string() + chars.as_str())
        .unwrap_or_default()
}

fn compose_horizontal_arrangement(
    justify: Option<&ResponsiveValue<Justify>>,
    gap: Option<&ResponsiveValue<GapValue>>,
) -> String {
    format!(
        "doweHorizontalArrangement({}, {})",
        compose_optional_justify(justify),
        compose_optional_gap(gap)
    )
}

fn compose_vertical_arrangement(
    justify: Option<&ResponsiveValue<Justify>>,
    gap: Option<&ResponsiveValue<GapValue>>,
) -> String {
    format!(
        "doweVerticalArrangement({}, {})",
        compose_optional_justify(justify),
        compose_optional_gap(gap)
    )
}

fn compose_horizontal_alignment(value: Option<&ResponsiveValue<Align>>) -> String {
    format!("doweHorizontalAlignment({})", compose_optional_align(value))
}

fn compose_grid_horizontal_alignment(value: Option<&ResponsiveValue<GridAlignment>>) -> String {
    format!(
        "doweGridHorizontalAlignment({})",
        compose_optional_grid_alignment(value)
    )
}

fn compose_grid_vertical_alignment(value: Option<&ResponsiveValue<GridAlignment>>) -> String {
    format!(
        "doweGridVerticalAlignment({})",
        compose_optional_grid_alignment(value)
    )
}

fn compose_grid_horizontal_stretch(value: Option<&ResponsiveValue<GridAlignment>>) -> String {
    format!(
        "doweGridHorizontalStretch({})",
        compose_optional_grid_alignment(value)
    )
}

fn compose_vertical_alignment(value: Option<&ResponsiveValue<Align>>) -> String {
    format!("doweVerticalAlignment({})", compose_optional_align(value))
}

fn compose_card_radius(props: &StyleProps) -> String {
    props
        .rounded
        .as_ref()
        .map(compose_rounded_value)
        .map(|value| format!("{value} ?: DoweDesign.radius"))
        .unwrap_or_else(|| "DoweDesign.radius".to_string())
}

fn compose_drawer_radius(props: &StyleProps) -> String {
    props
        .rounded
        .as_ref()
        .map(compose_rounded_value)
        .map(|value| format!("{value} ?: 0.dp"))
        .unwrap_or_else(|| "0.dp".to_string())
}

fn compose_control_radius(props: &StyleProps) -> String {
    props
        .rounded
        .as_ref()
        .map(compose_rounded_value)
        .map(|value| format!("{value} ?: DoweDesign.radius"))
        .unwrap_or_else(|| "DoweDesign.radius".to_string())
}

fn compose_button_border(props: &VariantProps) -> String {
    if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        format!("BorderStroke(1.dp, {})", variant_content(props))
    } else {
        "null".to_string()
    }
}

fn compose_card_border(props: &VariantProps) -> String {
    if props.style.border.is_some() {
        "null".to_string()
    } else if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        format!("BorderStroke(1.dp, {})", variant_content(props))
    } else {
        "null".to_string()
    }
}

fn compose_animation_preset(value: ViewAnimation) -> &'static str {
    match value {
        ViewAnimation::None => "DoweAnimationPreset.None",
        ViewAnimation::FadeIn => "DoweAnimationPreset.FadeIn",
        ViewAnimation::SlideUp => "DoweAnimationPreset.SlideUp",
        ViewAnimation::SlideDown => "DoweAnimationPreset.SlideDown",
        ViewAnimation::SlideLeft => "DoweAnimationPreset.SlideLeft",
        ViewAnimation::SlideRight => "DoweAnimationPreset.SlideRight",
        ViewAnimation::ScaleIn => "DoweAnimationPreset.ScaleIn",
    }
}

