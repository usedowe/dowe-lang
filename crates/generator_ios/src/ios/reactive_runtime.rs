fn swift_reactive_runtime() -> String {
    [
        include!("runtime_foundations.rs"),
        include!("runtime_svg_import.rs"),
        include!("runtime_state_core.rs"),
        include!("runtime_state_queries.rs"),
        include!("runtime_state_actions.rs"),
    ]
    .concat()
    .replace("__DOWE_VARIANTS__", &runtime_swift_values(dowe_components::BuiltinComponent::Button, "variant"))
    .replace("__DOWE_SCHEMES__", &runtime_swift_values(dowe_components::BuiltinComponent::Button, "scheme"))
    .replace("__DOWE_SIZES__", &runtime_swift_values(dowe_components::BuiltinComponent::Button, "size"))
    .replace("__DOWE_ROUNDED__", &runtime_swift_values(dowe_components::BuiltinComponent::Button, "rounded"))
    .replace("__DOWE_COLORS__", &runtime_swift_values(dowe_components::BuiltinComponent::Button, "scheme"))
    .replace("__DOWE_ICON_NAMES__", &runtime_swift_icons())
    .replace("__DOWE_DYNAMIC_TEXT_METRICS__", &runtime_swift_text_metrics())
}

fn runtime_swift_icons() -> String {
    dowe_components::all_icon_names()
        .iter()
        .map(|value| format!("\"{}\"", value.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(", ")
}

fn runtime_swift_values(component: dowe_components::BuiltinComponent, name: &str) -> String {
    dowe_components::prop_allowed_values(component, name)
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

fn runtime_swift_text_metrics() -> String {
    let body_cases = runtime_swift_text_metric_cases(false);
    let title_cases = runtime_swift_text_metric_cases(true);
    format!(
        r#"private struct DoweTextMetrics {{
    let min: CGFloat
    let preferredBase: CGFloat
    let preferredViewport: CGFloat
    let max: CGFloat
    let lineHeight: CGFloat
    let weight: Int
    let letterSpacing: CGFloat
}}
private func doweDynamicTextMetrics(_ value: String, title: Bool) -> DoweTextMetrics {{
    if title {{
        switch value {{
{title_cases}
        default: return {title_default}
        }}
    }}
    switch value {{
{body_cases}
    default: return {body_default}
    }}
}}
private func doweDynamicTextSize(_ value: String, viewportWidth: CGFloat, title: Bool) -> CGFloat {{
    let metrics = doweDynamicTextMetrics(value, title: title)
    return doweTextSize(viewportWidth, min: metrics.min, preferredBase: metrics.preferredBase, preferredViewport: metrics.preferredViewport, max: metrics.max)
}}
private func doweDynamicTextLineHeight(_ value: String, title: Bool) -> CGFloat {{
    doweDynamicTextMetrics(value, title: title).lineHeight
}}
private func doweDynamicTextWeightForSize(_ value: String, title: Bool) -> Font.Weight {{
    switch doweDynamicTextMetrics(value, title: title).weight {{
    case 100: return Font.Weight.ultraLight
    case 200: return Font.Weight.thin
    case 300: return Font.Weight.light
    case 400: return Font.Weight.regular
    case 500: return Font.Weight.medium
    case 600: return Font.Weight.semibold
    case 700: return Font.Weight.bold
    case 800: return Font.Weight.heavy
    case 900: return Font.Weight.black
    default: return Font.Weight.regular
    }}
}}
private func doweDynamicTextSpacingForSize(_ value: String, title: Bool) -> CGFloat {{
    doweDynamicTextMetrics(value, title: title).letterSpacing
}}
"#,
        title_cases = title_cases,
        title_default = runtime_swift_text_metric_value(true, dowe_components::TextSize::Md),
        body_cases = body_cases,
        body_default = runtime_swift_text_metric_value(false, dowe_components::TextSize::Md),
    )
}

fn runtime_swift_text_metric_cases(title: bool) -> String {
    dowe_components::TextSize::all()
        .iter()
        .map(|size| {
            format!(
                "        case \"{}\": return {}",
                size.as_str(),
                runtime_swift_text_metric_value(title, *size)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn runtime_swift_text_metric_value(title: bool, size: dowe_components::TextSize) -> String {
    let typography = dowe_components::text_typography(title, size);
    let font_size = typography.font_size;
    format!(
        "DoweTextMetrics(min: CGFloat({}), preferredBase: CGFloat({}), preferredViewport: CGFloat({}), max: CGFloat({}), lineHeight: CGFloat({}), weight: {}, letterSpacing: CGFloat({}))",
        font_size.min,
        font_size.preferred_base,
        font_size.preferred_viewport,
        font_size.max,
        typography.line_height,
        dowe_components::text_weight_number(typography.weight),
        typography.letter_spacing_em,
    )
}
