fn parse_color_prop(name: &str, value: &PropValue) -> ComponentResult<ResponsiveValue<ColorToken>> {
    parse_responsive(name, value, "color token", |scalar| match scalar {
        PropScalar::String(value) => ColorToken::from_name(value),
        PropScalar::Number(_) | PropScalar::Boolean(_) => None,
    })
}

fn parse_font_prop(name: &str, value: &PropValue) -> ComponentResult<ResponsiveValue<FontFamily>> {
    parse_responsive(
        name,
        value,
        "system, inter, roboto, montserrat, lato, poppins, manrope, quicksand, lora, syne, jost or puritan",
        |scalar| match scalar {
            PropScalar::String(value) => FontFamily::from_name(value),
            PropScalar::Number(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_cover_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<CoverSource>> {
    parse_responsive(
        name,
        value,
        "asset path or https URL",
        |scalar| match scalar {
            PropScalar::String(value) => parse_cover_source(value),
            PropScalar::Number(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_overlay_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<OverlayPaint>> {
    parse_responsive(
        name,
        value,
        "boolean, opacity from 0 to 1, color token, rgba or linear-gradient",
        |scalar| match scalar {
            PropScalar::Boolean(true) => Some(OverlayPaint::BlackOpacity("0.4".to_string())),
            PropScalar::Boolean(false) => None,
            PropScalar::Number(value) => parse_overlay_opacity(value),
            PropScalar::String(value) => parse_overlay_string(value),
        },
    )
}

fn parse_background_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<SectionBackground>> {
    parse_responsive(
        name,
        value,
        "aurora, sunrise, ocean, meadow or slate",
        |scalar| match scalar {
            PropScalar::String(value) => SectionBackground::from_name(value),
            PropScalar::Number(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_cover_source(value: &str) -> Option<CoverSource> {
    if value.starts_with("https://") {
        let host = value
            .strip_prefix("https://")?
            .split(['/', '#', '?'])
            .next()
            .filter(|host| !host.is_empty())?;
        if host.chars().any(|value| value.is_control() || value == ' ') {
            return None;
        }
        return Some(CoverSource(value.to_string()));
    }

    if value.starts_with("//")
        || value.starts_with("javascript:")
        || value.starts_with("data:")
        || value.starts_with("file:")
        || value.contains("://")
        || value.is_empty()
    {
        return None;
    }

    Some(CoverSource(value.to_string()))
}

fn parse_overlay_opacity(value: &str) -> Option<OverlayPaint> {
    let parsed = value.parse::<f32>().ok()?;
    if !(0.0..=1.0).contains(&parsed) {
        return None;
    }
    Some(OverlayPaint::BlackOpacity(normalize_decimal(value)))
}

fn parse_overlay_string(value: &str) -> Option<OverlayPaint> {
    if let Some(token) = ColorToken::from_name(value) {
        return Some(OverlayPaint::Color(token));
    }
    if is_valid_rgba(value) {
        return Some(OverlayPaint::Rgba(value.to_string()));
    }
    if is_valid_linear_gradient(value) {
        return Some(OverlayPaint::LinearGradient(value.to_string()));
    }
    None
}

fn normalize_decimal(value: &str) -> String {
    let mut output = value.trim().trim_end_matches('0').to_string();
    if output.is_empty() {
        return "0".to_string();
    }
    if output.ends_with('.') {
        output.push('0');
    }
    if output == "0." {
        "0".to_string()
    } else {
        output
    }
}
