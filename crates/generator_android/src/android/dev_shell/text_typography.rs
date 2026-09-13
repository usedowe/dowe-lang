fn dev_activity_text_typography_helpers() -> String {
    let body_cases = dev_text_metric_cases(false, "size");
    let title_cases = dev_text_metric_cases(true, "size");
    let body_line_height_cases = dev_text_metric_cases(false, "line_height");
    let title_line_height_cases = dev_text_metric_cases(true, "line_height");
    let title_weight_cases = dev_text_metric_cases(true, "weight");
    let title_spacing_cases = dev_text_metric_cases(true, "spacing");
    format!(
        r#"
    private float doweDynamicTextSize(String value, boolean title) {{
        if (title) {{
            switch (value) {{
{title_cases}
                default: return {title_default_size};
            }}
        }}
        switch (value) {{
{body_cases}
            default: return {body_default_size};
        }}
    }}

    private float doweDynamicTextLineHeight(String value, boolean title) {{
        if (title) {{
            switch (value) {{
{title_line_height_cases}
                default: return {title_default_line_height};
            }}
        }}
        switch (value) {{
{body_line_height_cases}
            default: return {body_default_line_height};
        }}
    }}

    private int doweDynamicTextWeight(String value) {{
        switch (value) {{
            case "thin": return 100;
            case "extralight": return 200;
            case "light": return 300;
            case "regular": return 400;
            case "medium": return 500;
            case "semibold": return 600;
            case "bold": return 700;
            case "extrabold": return 800;
            case "black": return 900;
            default: return 400;
        }}
    }}

    private int doweDynamicTextWeightForSize(String value) {{
        switch (value) {{
{title_weight_cases}
            default: return {title_default_weight};
        }}
    }}

    private float doweDynamicTextSpacing(String value) {{
        switch (value) {{
            case "tightest": return -0.06f;
            case "tighter": return -0.04f;
            case "tight": return -0.02f;
            case "normal": return 0f;
            case "wide": return 0.02f;
            case "wider": return 0.04f;
            case "widest": return 0.06f;
            default:
                try {{ return Float.parseFloat(value); }} catch (NumberFormatException ignored) {{ return 0f; }}
        }}
    }}

    private float doweDynamicTextSpacingForSize(String value) {{
        switch (value) {{
{title_spacing_cases}
            default: return {title_default_spacing};
        }}
    }}
"#,
        title_cases = title_cases,
        title_default_size = dev_text_size_expr(true, TextSize::Md),
        body_cases = body_cases,
        body_default_size = dev_text_size_expr(false, TextSize::Md),
        title_line_height_cases = title_line_height_cases,
        title_default_line_height = format!("{}f", text_typography(true, TextSize::Md).line_height),
        body_line_height_cases = body_line_height_cases,
        body_default_line_height = format!("{}f", text_typography(false, TextSize::Md).line_height),
        title_weight_cases = title_weight_cases,
        title_default_weight = dowe_components::text_weight_number(text_typography(true, TextSize::Md).weight),
        title_spacing_cases = title_spacing_cases,
        title_default_spacing = format!("{}f", text_typography(true, TextSize::Md).letter_spacing_em),
    )
}

fn dev_text_metric_cases(title: bool, metric: &str) -> String {
    dowe_components::TextSize::all()
        .iter()
        .map(|size| {
            let typography = text_typography(title, *size);
            let value = match metric {
                "size" => dev_text_size_expr(title, *size),
                "line_height" => format!("{}f", typography.line_height),
                "weight" => dowe_components::text_weight_number(typography.weight).to_string(),
                "spacing" => format!("{}f", typography.letter_spacing_em),
                _ => unreachable!(),
            };
            format!("            case \"{}\": return {value};", size.as_str())
        })
        .collect::<Vec<_>>()
        .join("\n")
}
