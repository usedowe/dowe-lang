fn parse_responsive<T, F>(
    name: &str,
    value: &PropValue,
    expected: &str,
    parse: F,
) -> ComponentResult<ResponsiveValue<T>>
where
    T: Clone,
    F: Fn(&PropScalar) -> Option<T>,
{
    match value {
        PropValue::String(value) => parse(&PropScalar::String(value.clone()))
            .map(ResponsiveValue::scalar)
            .ok_or_else(|| ComponentError::invalid_prop(name, expected)),
        PropValue::Number(value) => parse(&PropScalar::Number(value.clone()))
            .map(ResponsiveValue::scalar)
            .ok_or_else(|| ComponentError::invalid_prop(name, expected)),
        PropValue::Boolean(value) => parse(&PropScalar::Boolean(*value))
            .map(ResponsiveValue::scalar)
            .ok_or_else(|| ComponentError::invalid_prop(name, expected)),
        PropValue::Binding(binding) => {
            let fallback = binding.fallback.as_deref().ok_or_else(|| {
                ComponentError::invalid_prop(name, "reactive value with a valid fallback")
            })?;
            let scalar = match fallback {
                PropValue::String(value) => PropScalar::String(value.clone()),
                PropValue::Number(value) => PropScalar::Number(value.clone()),
                PropValue::Boolean(value) => PropScalar::Boolean(*value),
                PropValue::Responsive(_) | PropValue::Binding(_) => {
                    return Err(ComponentError::invalid_prop(name, expected));
                }
            };
            parse(&scalar)
                .map(ResponsiveValue::scalar)
                .ok_or_else(|| ComponentError::invalid_prop(name, expected))
        }
        PropValue::Responsive(entries) => {
            let mut parsed = Vec::new();
            for entry in entries {
                let breakpoint = Breakpoint::from_name(&entry.breakpoint)
                    .ok_or_else(|| ComponentError::invalid_prop(name, "valid breakpoint"))?;
                let value = parse(&entry.value)
                    .ok_or_else(|| ComponentError::invalid_prop(name, expected))?;
                parsed.push(ResponsiveEntry { breakpoint, value });
            }
            Ok(ResponsiveValue::ordered(parsed))
        }
    }
}

fn scale_value(value: &str) -> Option<ScaleValue> {
    let half_steps = scale_half_steps(value)?;
    TAILWIND_SCALE
        .iter()
        .copied()
        .find(|scale| scale.0 == half_steps)
}

fn scale_half_steps(value: &str) -> Option<u16> {
    if let Some(integer) = value.strip_suffix(".0") {
        return integer.parse::<u16>().ok().map(|value| value * 2);
    }
    if let Some(integer) = value.strip_suffix(".5") {
        return integer.parse::<u16>().ok().map(|value| value * 2 + 1);
    }
    value.parse::<u16>().ok().map(|value| value * 2)
}

fn signed_half_steps(value: &str) -> Option<i16> {
    let negative = value.starts_with('-');
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let half_steps = scale_half_steps(unsigned)?;
    i16::try_from(half_steps)
        .ok()
        .map(|value| if negative { -value } else { value })
}

fn parse_decimal_hundredths(value: &str) -> Option<u16> {
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    if fraction.len() > 2 || whole.is_empty() || value.starts_with('-') {
        return None;
    }
    let whole = whole.parse::<u16>().ok()?;
    let fraction = match fraction.len() {
        0 => 0,
        1 => fraction.parse::<u16>().ok()? * 10,
        2 => fraction.parse::<u16>().ok()?,
        _ => return None,
    };
    whole.checked_mul(100)?.checked_add(fraction)
}

fn parse_gap_value(value: &str, pair_allowed: bool) -> Option<GapValue> {
    let parts = value.split_whitespace().collect::<Vec<_>>();
    match parts.as_slice() {
        [single] => parse_gap_size(single).map(GapValue::Single),
        [row, column] if pair_allowed => Some(GapValue::Pair(
            parse_gap_size(row)?,
            parse_gap_size(column)?,
        )),
        _ => None,
    }
}

fn parse_gap_size(value: &str) -> Option<GapSize> {
    if let Some(px) = value.strip_suffix("px") {
        return px.parse::<u16>().ok().map(GapSize::Px);
    }
    scale_value(value).map(GapSize::Scale)
}

