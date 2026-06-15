fn parse_chat_box_mode(name: &str, value: &PropValue) -> ComponentResult<ChatBoxMode> {
    let value = parse_required_string(name, value)?;
    ChatBoxMode::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "conversation or prompt"))
}

fn parse_empty_kind(name: &str, value: &PropValue) -> ComponentResult<EmptyKind> {
    let value = parse_required_string(name, value)?;
    EmptyKind::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "playlist, result, data or template"))
}

fn parse_marquee_speed(name: &str, value: &PropValue) -> ComponentResult<MarqueeSpeed> {
    let value = parse_required_string(name, value)?;
    MarqueeSpeed::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "slow, normal or fast"))
}

fn parse_marquee_orientation(name: &str, value: &PropValue) -> ComponentResult<MarqueeOrientation> {
    let value = parse_required_string(name, value)?;
    MarqueeOrientation::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "horizontal or vertical"))
}

fn parse_rich_text_mark_style(name: &str, value: &PropValue) -> ComponentResult<RichTextMarkStyle> {
    let value = parse_required_string(name, value)?;
    RichTextMarkStyle::from_name(&value).ok_or_else(|| {
        ComponentError::invalid_prop(
            name,
            "mark, grad, pill, slant, glow, under, strike, box, wave, neon, pop or tag",
        )
    })
}

fn parse_countdown_size(name: &str, value: &PropValue) -> ComponentResult<CountdownSize> {
    let value = parse_required_string(name, value)?;
    CountdownSize::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "sm, md, lg or xl"))
}

fn parse_map_marker_icon(name: &str, value: &PropValue) -> ComponentResult<MapMarkerIcon> {
    let value = parse_required_string(name, value)?;
    MapMarkerIcon::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "default, start, end or waypoint"))
}

fn parse_control_size_prop(name: &str, value: &PropValue) -> ComponentResult<ButtonSize> {
    let size = parse_button_size_prop(name, value)?;
    match size {
        ButtonSize::Sm | ButtonSize::Md | ButtonSize::Lg => Ok(size),
        ButtonSize::Xs | ButtonSize::Xl => Err(ComponentError::invalid_prop(name, "sm, md or lg")),
    }
}

fn parse_view_icon(name: &str, value: &PropValue) -> ComponentResult<ViewIcon> {
    let value = parse_required_string(name, value)?;
    ViewIcon::from_name(&value).ok_or_else(|| {
        ComponentError::invalid_prop(
            name,
            "plus, link, edit, trash, search, settings, upload, file or dismiss",
        )
    })
}

fn parse_static_scale(name: &str, value: &PropValue) -> ComponentResult<ScaleValue> {
    match value {
        PropValue::Number(value) => scale_value(value)
            .ok_or_else(|| ComponentError::invalid_prop(name, "Dowe scale value from 0 to 96")),
        PropValue::String(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => Err(
            ComponentError::invalid_prop(name, "Dowe scale value from 0 to 96"),
        ),
    }
}

fn parse_number_literal(name: &str, value: &PropValue) -> ComponentResult<String> {
    match value {
        PropValue::Number(value) if value.parse::<f64>().is_ok() => Ok(value.clone()),
        PropValue::String(_)
        | PropValue::Number(_)
        | PropValue::Boolean(_)
        | PropValue::Responsive(_) => Err(ComponentError::invalid_prop(name, "number")),
    }
}

fn parse_positive_number_literal(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_number_literal(name, value)?;
    if value.parse::<f64>().ok().is_some_and(|value| value > 0.0) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(name, "positive number"))
    }
}

fn parse_positive_u64(name: &str, value: &PropValue) -> ComponentResult<u64> {
    match value {
        PropValue::Number(value) => value
            .parse::<u64>()
            .ok()
            .filter(|value| *value > 0)
            .ok_or_else(|| ComponentError::invalid_prop(name, "positive integer")),
        PropValue::String(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(name, "positive integer"))
        }
    }
}

fn parse_non_negative_u64(name: &str, value: &PropValue) -> ComponentResult<u64> {
    match value {
        PropValue::Number(value) => value
            .parse::<u64>()
            .ok()
            .ok_or_else(|| ComponentError::invalid_prop(name, "non-negative integer")),
        PropValue::String(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(name, "non-negative integer"))
        }
    }
}

fn parse_single_color_token(name: &str, value: &PropValue) -> ComponentResult<ColorToken> {
    let value = parse_required_string(name, value)?;
    ColorToken::from_name(&value).ok_or_else(|| ComponentError::invalid_prop(name, "color token"))
}

fn validate_slider_range(min: &str, max: &str, value: &str) -> ComponentResult<()> {
    let min_value = min
        .parse::<f64>()
        .map_err(|_| ComponentError::invalid_prop("min", "number"))?;
    let max_value = max
        .parse::<f64>()
        .map_err(|_| ComponentError::invalid_prop("max", "number"))?;
    let current_value = value
        .parse::<f64>()
        .map_err(|_| ComponentError::invalid_prop("value", "number"))?;
    if max_value <= min_value {
        return Err(ComponentError::invalid_prop_combination(
            "`max` must be greater than `min`",
        ));
    }
    if current_value < min_value || current_value > max_value {
        return Err(ComponentError::invalid_prop(
            "value",
            "number between min and max",
        ));
    }
    Ok(())
}

fn apply_icon_button_size_defaults(style: &mut StyleProps, size: ButtonSize) {
    let value = SizeValue::Scale(size.min_height());
    if style.sizing.w.is_none() {
        style.sizing.w = Some(ResponsiveValue::scalar(value.clone()));
    }
    if style.sizing.h.is_none() {
        style.sizing.h = Some(ResponsiveValue::scalar(value));
    }
    if style.rounded.is_none() {
        style.rounded = Some(ResponsiveValue::scalar(RoundedSize::Full));
    }
}

fn parse_non_negative_u16(name: &str, value: &PropValue) -> ComponentResult<u16> {
    match value {
        PropValue::Number(value) => value
            .parse::<u16>()
            .ok()
            .ok_or_else(|| ComponentError::invalid_prop(name, "non-negative integer")),
        PropValue::String(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(name, "non-negative integer"))
        }
    }
}

fn parse_hex_color_prop(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if valid_hex_color(&value) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(name, "hex color like #3b82f6"))
    }
}

fn valid_hex_color(value: &str) -> bool {
    let Some(hex) = value.strip_prefix('#') else {
        return false;
    };
    matches!(hex.len(), 3 | 6) && hex.chars().all(|value| value.is_ascii_hexdigit())
}

fn parse_date_literal(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if valid_date_literal(&value) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(name, "YYYY-MM-DD"))
    }
}

fn valid_date_literal(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    if !bytes
        .iter()
        .enumerate()
        .all(|(index, value)| index == 4 || index == 7 || value.is_ascii_digit())
    {
        return false;
    }
    let year = value[0..4].parse::<u16>().ok();
    let month = value[5..7].parse::<u8>().ok();
    let day = value[8..10].parse::<u8>().ok();
    let (Some(year), Some(month), Some(day)) = (year, month, day) else {
        return false;
    };
    if year == 0 || !(1..=12).contains(&month) {
        return false;
    }
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => return false,
    };
    (1..=max_day).contains(&day)
}

fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn validate_date_bounds(min: Option<&str>, max: Option<&str>) -> ComponentResult<()> {
    if let (Some(min), Some(max)) = (min, max)
        && min > max
    {
        return Err(ComponentError::invalid_prop_combination(
            "`min` cannot be greater than `max`",
        ));
    }
    Ok(())
}
