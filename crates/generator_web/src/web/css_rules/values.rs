fn scale_rem(value: dowe_components::ScaleValue) -> String {
    if value.0 == 0 {
        return "0rem".to_string();
    }

    let rem = value.0 as f32 / 8.0;
    let mut output = format!("{rem:.3}");
    while output.contains('.') && output.ends_with('0') {
        output.pop();
    }
    if output.ends_with('.') {
        output.pop();
    }
    format!("{output}rem")
}

fn rounded_value(value: &str) -> &'static str {
    match value {
        "xs" => "calc(var(--dowe-radius) * 0.5)",
        "sm" => "calc(var(--dowe-radius) * 0.75)",
        "md" => "var(--dowe-radius)",
        "lg" => "calc(var(--dowe-radius) * 1.5)",
        "xl" => "calc(var(--dowe-radius) * 2.25)",
        "full" => "9999px",
        _ => "var(--dowe-radius)",
    }
}

fn animation_css(value: &str) -> Option<String> {
    let name = match value {
        "fade-in" => "dowe-fade-in",
        "slide-up" => "dowe-slide-up",
        "slide-down" => "dowe-slide-down",
        "slide-left" => "dowe-slide-left",
        "slide-right" => "dowe-slide-right",
        "scale-in" => "dowe-scale-in",
        "none" => return Some("animation:none;".to_string()),
        _ => return None,
    };
    Some(format!("animation:{name} 220ms ease-out both;"))
}

fn justify_css(value: Justify) -> &'static str {
    match value {
        Justify::Start => "flex-start",
        Justify::Center => "center",
        Justify::End => "flex-end",
        Justify::Between => "space-between",
        Justify::Around => "space-around",
        Justify::Evenly => "space-evenly",
    }
}

fn align_css(value: Align) -> &'static str {
    match value {
        Align::Start => "flex-start",
        Align::Center => "center",
        Align::End => "flex-end",
        Align::Stretch => "stretch",
        Align::Baseline => "baseline",
    }
}

fn text_align_css(value: TextAlign) -> &'static str {
    match value {
        TextAlign::Start => "start",
        TextAlign::Center => "center",
        TextAlign::End => "end",
        TextAlign::Justify => "justify",
    }
}

fn grid_alignment_css(value: GridAlignment) -> &'static str {
    match value {
        GridAlignment::Start => "start",
        GridAlignment::Center => "center",
        GridAlignment::End => "end",
        GridAlignment::Stretch => "stretch",
    }
}

fn gap_size_css(value: &GapSize) -> String {
    match value {
        GapSize::Scale(value) => scale_rem(*value),
        GapSize::Px(value) => format!("{value}px"),
    }
}

fn cover_suffix(value: &CoverSource) -> String {
    short_id("cover", &value.0)
}

fn overlay_suffix(value: &OverlayPaint) -> String {
    short_id("overlay", &overlay_key(value))
}

fn overlay_key(value: &OverlayPaint) -> String {
    match value {
        OverlayPaint::BlackOpacity(value) => format!("black-{value}"),
        OverlayPaint::Color(value) => format!("color-{}", value.as_str()),
        OverlayPaint::Rgba(value) => format!("rgba-{value}"),
        OverlayPaint::LinearGradient(value) => format!("gradient-{value}"),
    }
}

fn section_background_css(value: SectionBackground) -> String {
    match value {
        SectionBackground::Aurora => "background-image:linear-gradient(135deg,var(--dowe-primary),var(--dowe-secondary),var(--dowe-accent));".to_string(),
        SectionBackground::Sunrise => "background-image:linear-gradient(135deg,var(--dowe-warning),var(--dowe-danger),var(--dowe-surface));".to_string(),
        SectionBackground::Ocean => "background-image:linear-gradient(135deg,var(--dowe-info),var(--dowe-primary),var(--dowe-accent));".to_string(),
        SectionBackground::Meadow => "background-image:linear-gradient(135deg,var(--dowe-success),var(--dowe-accent),var(--dowe-surface));".to_string(),
        SectionBackground::Slate => "background-image:linear-gradient(135deg,var(--dowe-muted),var(--dowe-surface),var(--dowe-background));".to_string(),
    }
}

fn overlay_css(value: &OverlayPaint) -> String {
    match value {
        OverlayPaint::BlackOpacity(value) => format!("rgba(0,0,0,{value})"),
        OverlayPaint::Color(value) => format!("var(--dowe-{})", value.as_str()),
        OverlayPaint::Rgba(value) | OverlayPaint::LinearGradient(value) => value.clone(),
    }
}

fn escape_css_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn text_size_css(value: TextSize) -> String {
    fluid_text_size_css(text_typography(false, value).font_size)
}

fn fluid_text_size_css(value: dowe_components::FluidTextSize) -> String {
    format!(
        "clamp({}rem, {}rem + {}vw, {}rem)",
        points_to_rem(value.min),
        points_to_rem(value.preferred_base),
        value.preferred_viewport,
        points_to_rem(value.max)
    )
}

fn points_to_rem(value: &str) -> String {
    let rem = value.parse::<f64>().expect("text metric") / 16.0;
    let mut output = format!("{rem:.4}");
    while output.contains('.') && output.ends_with('0') {
        output.pop();
    }
    if output.ends_with('.') {
        output.pop();
    }
    output
}
