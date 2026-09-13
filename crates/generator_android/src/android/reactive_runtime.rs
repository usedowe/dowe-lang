fn compose_reactive_runtime() -> String {
    [
        include!("runtime_kotlin_foundations.rs"),
        include!("runtime_kotlin_svg_import.rs"),
        include!("runtime_kotlin_state_core.rs"),
        include!("runtime_kotlin_actions.rs"),
        include!("runtime_kotlin_stdlib_and_helpers.rs"),
    ]
    .concat()
    .replace("__DOWE_VARIANTS__", &runtime_kotlin_values(dowe_components::BuiltinComponent::Button, "variant"))
    .replace("__DOWE_SCHEMES__", &runtime_kotlin_values(dowe_components::BuiltinComponent::Button, "scheme"))
    .replace("__DOWE_SIZES__", &runtime_kotlin_values(dowe_components::BuiltinComponent::Button, "size"))
    .replace("__DOWE_ROUNDED__", &runtime_kotlin_values(dowe_components::BuiltinComponent::Button, "rounded"))
    .replace("__DOWE_COLORS__", &runtime_kotlin_values(dowe_components::BuiltinComponent::Button, "scheme"))
    .replace("__DOWE_ICON_NAMES__", &runtime_kotlin_icons())
    .replace("__DOWE_DYNAMIC_TEXT_METRICS__", &runtime_kotlin_text_metrics())
}

fn runtime_kotlin_icons() -> String {
    dowe_components::all_icon_names()
        .iter()
        .map(|value| format!("\"{}\"", value.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(", ")
}

fn runtime_kotlin_values(component: dowe_components::BuiltinComponent, name: &str) -> String {
    dowe_components::prop_allowed_values(component, name)
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

fn runtime_kotlin_text_metrics() -> String {
    let body_cases = runtime_kotlin_text_metric_cases(false);
    let title_cases = runtime_kotlin_text_metric_cases(true);
    format!(
        r#"private data class DoweTextMetrics(
    val min: Float,
    val preferredBase: Float,
    val preferredViewport: Float,
    val max: Float,
    val lineHeight: Float,
    val weight: Int,
    val letterSpacing: Float
)
private fun doweDynamicTextMetrics(value: String, title: Boolean): DoweTextMetrics = if (title) {{
    when (value) {{
{title_cases}
            else -> {title_default}
    }}
}} else {{
    when (value) {{
{body_cases}
            else -> {body_default}
    }}
}}
private fun doweDynamicTextSize(value: String, viewportWidth: Dp, title: Boolean): TextUnit {{
    val metrics = doweDynamicTextMetrics(value, title)
    return doweTextSize(viewportWidth, metrics.min, metrics.preferredBase, metrics.preferredViewport, metrics.max)
}}
private fun doweDynamicTextLineHeight(value: String, title: Boolean, fontSize: TextUnit): TextUnit =
    doweTextLineHeight(fontSize, doweDynamicTextMetrics(value, title).lineHeight)
private fun doweDynamicTextWeightForSize(value: String, title: Boolean): FontWeight =
    when (doweDynamicTextMetrics(value, title).weight) {{
        100 -> FontWeight.Thin
        200 -> FontWeight.ExtraLight
        300 -> FontWeight.Light
        400 -> FontWeight.Normal
        500 -> FontWeight.Medium
        600 -> FontWeight.SemiBold
        700 -> FontWeight.Bold
        800 -> FontWeight.ExtraBold
        900 -> FontWeight.Black
        else -> FontWeight.Normal
    }}
private fun doweDynamicTextSpacingForSize(value: String, title: Boolean): TextUnit =
    doweDynamicTextMetrics(value, title).letterSpacing.em
"#,
        title_cases = title_cases,
        title_default = runtime_kotlin_text_metric_value(true, dowe_components::TextSize::Md),
        body_cases = body_cases,
        body_default = runtime_kotlin_text_metric_value(false, dowe_components::TextSize::Md),
    )
}

fn runtime_kotlin_text_metric_cases(title: bool) -> String {
    dowe_components::TextSize::all()
        .iter()
        .map(|size| {
            format!(
                "            \"{}\" -> {}",
                size.as_str(),
                runtime_kotlin_text_metric_value(title, *size)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn runtime_kotlin_text_metric_value(title: bool, size: dowe_components::TextSize) -> String {
    let typography = dowe_components::text_typography(title, size);
    let font_size = typography.font_size;
    format!(
        "DoweTextMetrics(min = {}f, preferredBase = {}f, preferredViewport = {}f, max = {}f, lineHeight = {}f, weight = {}, letterSpacing = {}f)",
        font_size.min,
        font_size.preferred_base,
        font_size.preferred_viewport,
        font_size.max,
        typography.line_height,
        dowe_components::text_weight_number(typography.weight),
        typography.letter_spacing_em,
    )
}
