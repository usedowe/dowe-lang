fn parse_video_src(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if value.starts_with("https://") && parse_cover_source(&value).is_some() {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(name, "https URL"))
    }
}

fn parse_video_poster(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    parse_cover_source(&value)
        .map(|source| source.0)
        .ok_or_else(|| ComponentError::invalid_prop(name, "asset path or https URL"))
}

fn parse_reference_path(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if is_reference_path(&value) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(name, "signal array path"))
    }
}

fn parse_candlestick_stream(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if valid_candlestick_stream(&value) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(
            name,
            "absolute path or https URL",
        ))
    }
}

fn valid_candlestick_stream(value: &str) -> bool {
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return false;
    }
    if value.starts_with("https://") {
        return parse_cover_source(value).is_some();
    }
    value.starts_with('/') && !value.starts_with("//") && !value.contains("://")
}

fn parse_candlestick_color(name: &str, value: &PropValue) -> ComponentResult<ColorToken> {
    let value = parse_required_string(name, value)?;
    ColorToken::from_name(&value).ok_or_else(|| ComponentError::invalid_prop(name, "color token"))
}

fn parse_table_field(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if is_reference_path(&value) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(name, "relative field path"))
    }
}

fn parse_table_column_width(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if valid_table_width(&value) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(name, "portable table width"))
    }
}

fn valid_table_width(value: &str) -> bool {
    matches!(value, "auto" | "min-content" | "max-content")
        || valid_positive_width_suffix(value, "px")
        || valid_positive_width_suffix(value, "rem")
        || valid_positive_width_suffix(value, "%")
        || valid_positive_width_suffix(value, "fr")
}

fn valid_positive_width_suffix(value: &str, suffix: &str) -> bool {
    let Some(number) = value.strip_suffix(suffix) else {
        return false;
    };
    !number.is_empty()
        && number
            .parse::<f32>()
            .ok()
            .filter(|value| *value > 0.0)
            .is_some()
}

fn parse_positive_u16(name: &str, value: &PropValue) -> ComponentResult<u16> {
    match value {
        PropValue::Number(value) => value
            .parse::<u16>()
            .ok()
            .filter(|value| *value > 0)
            .ok_or_else(|| ComponentError::invalid_prop(name, "positive integer")),
        PropValue::String(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(name, "positive integer"))
        }
    }
}

fn parse_avatar_src(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    parse_cover_source(&value)
        .map(|source| source.0)
        .ok_or_else(|| ComponentError::invalid_prop(name, "asset path or https URL"))
}

fn parse_media_source(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    parse_cover_source(&value)
        .map(|source| source.0)
        .ok_or_else(|| ComponentError::invalid_prop(name, "asset path or https URL"))
}

fn parse_static_string_or_number(name: &str, value: &PropValue) -> ComponentResult<String> {
    match value {
        PropValue::String(value) | PropValue::Number(value) => Ok(value.clone()),
        PropValue::Boolean(_) | PropValue::Responsive(_) => Err(ComponentError::invalid_prop(
            name,
            "static string or number",
        )),
    }
}

fn parse_image_aspect(name: &str, value: &PropValue) -> ComponentResult<ImageAspect> {
    let value = parse_required_string(name, value)?;
    ImageAspect::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "horizontal, vertical, square or auto"))
}

fn parse_image_object_fit(name: &str, value: &PropValue) -> ComponentResult<ImageObjectFit> {
    let value = parse_required_string(name, value)?;
    ImageObjectFit::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "cover, contain, fill or none"))
}

fn parse_image_loading(name: &str, value: &PropValue) -> ComponentResult<ImageLoading> {
    let value = parse_required_string(name, value)?;
    ImageLoading::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "lazy or eager"))
}

fn parse_carousel_orientation(
    name: &str,
    value: &PropValue,
) -> ComponentResult<CarouselOrientation> {
    let value = parse_required_string(name, value)?;
    CarouselOrientation::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "horizontal or vertical"))
}

fn parse_carousel_indicator(
    name: &str,
    value: &PropValue,
) -> ComponentResult<CarouselIndicatorType> {
    let value = parse_required_string(name, value)?;
    CarouselIndicatorType::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "bar or dot"))
}
