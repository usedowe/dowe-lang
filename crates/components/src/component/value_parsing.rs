fn parse_scale_prop(name: &str, value: &PropValue) -> ComponentResult<ResponsiveValue<ScaleValue>> {
    parse_responsive(
        name,
        value,
        "Dowe scale value from 0 to 96",
        |scalar| match scalar {
            PropScalar::Number(value) => scale_value(value),
            PropScalar::String(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_size_prop(name: &str, value: &PropValue) -> ComponentResult<ResponsiveValue<SizeValue>> {
    parse_responsive(
        name,
        value,
        "Dowe scale value from 0 to 96 or full",
        |scalar| match scalar {
            PropScalar::Number(value) => scale_value(value).map(SizeValue::Scale),
            PropScalar::String(value) if value == "full" => Some(SizeValue::Full),
            PropScalar::String(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_rounded_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<RoundedSize>> {
    parse_responsive(
        name,
        value,
        "xs, sm, md, lg, xl or full",
        |scalar| match scalar {
            PropScalar::String(value) => RoundedSize::from_name(value),
            PropScalar::Number(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_border_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<BorderWidth>> {
    parse_responsive(name, value, "integer from 1 to 4", |scalar| match scalar {
        PropScalar::Number(value) => value
            .parse::<u8>()
            .ok()
            .filter(|value| (1..=4).contains(value))
            .map(BorderWidth),
        PropScalar::String(_) | PropScalar::Boolean(_) => None,
    })
}

fn parse_justify_prop(name: &str, value: &PropValue) -> ComponentResult<ResponsiveValue<Justify>> {
    parse_responsive(
        name,
        value,
        "start, center, end, between, around or evenly",
        |scalar| match scalar {
            PropScalar::String(value) => Justify::from_name(value),
            PropScalar::Number(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_align_prop(name: &str, value: &PropValue) -> ComponentResult<ResponsiveValue<Align>> {
    parse_responsive(
        name,
        value,
        "start, center, end, stretch or baseline",
        |scalar| match scalar {
            PropScalar::String(value) => Align::from_name(value),
            PropScalar::Number(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_grid_alignment_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<GridAlignment>> {
    parse_responsive(
        name,
        value,
        "start, center, end or stretch",
        |scalar| match scalar {
            PropScalar::String(value) => GridAlignment::from_name(value),
            PropScalar::Number(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_gap_prop(
    name: &str,
    value: &PropValue,
    pair_allowed: bool,
) -> ComponentResult<ResponsiveValue<GapValue>> {
    parse_responsive(
        name,
        value,
        "Dowe scale value or px value",
        |scalar| match scalar {
            PropScalar::Number(value) => {
                scale_value(value).map(|value| GapValue::Single(GapSize::Scale(value)))
            }
            PropScalar::String(value) => parse_gap_value(value, pair_allowed),
            PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_grid_tracks_prop(
    name: &str,
    value: &PropValue,
    auto_allowed: bool,
) -> ComponentResult<ResponsiveValue<GridTracks>> {
    parse_responsive(
        name,
        value,
        "positive integer, auto or portable grid template",
        |scalar| match scalar {
            PropScalar::Number(value) => value
                .parse::<u16>()
                .ok()
                .filter(|value| *value > 0)
                .map(GridTracks::Count),
            PropScalar::String(value) if auto_allowed && value == "auto" => Some(GridTracks::Auto),
            PropScalar::String(value) => parse_grid_template(value),
            PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_span_prop(name: &str, value: &PropValue) -> ComponentResult<ResponsiveValue<GridSpan>> {
    parse_responsive(name, value, "positive integer", |scalar| match scalar {
        PropScalar::Number(value) => value
            .parse::<u16>()
            .ok()
            .filter(|value| *value > 0)
            .map(GridSpan),
        PropScalar::String(_) | PropScalar::Boolean(_) => None,
    })
}

fn parse_animation_prop(name: &str, value: &PropValue) -> ComponentResult<ViewAnimation> {
    match value {
        PropValue::String(value) => ViewAnimation::from_name(value).ok_or_else(|| {
            ComponentError::invalid_prop(
                name,
                "none, fadeIn, slideUp, slideDown, slideLeft, slideRight or scaleIn",
            )
        }),
        PropValue::Number(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(
                name,
                "none, fadeIn, slideUp, slideDown, slideLeft, slideRight or scaleIn",
            ))
        }
    }
}

fn parse_text_size_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<TextSize>> {
    parse_responsive(
        name,
        value,
        "text size from xs to 9xl",
        |scalar| match scalar {
            PropScalar::String(value) => TextSize::from_name(value),
            PropScalar::Number(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_text_weight_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<TextWeight>> {
    parse_responsive(
        name,
        value,
        "thin, extralight, light, regular, medium, semibold, bold, extrabold or black",
        |scalar| match scalar {
            PropScalar::String(value) => TextWeight::from_name(value),
            PropScalar::Number(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_text_spacing_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<TextSpacing>> {
    parse_responsive(
        name,
        value,
        "tightest, tighter, tight, normal, wide, wider or widest",
        |scalar| match scalar {
            PropScalar::String(value) => TextSpacing::from_name(value),
            PropScalar::Number(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_variant_prop(name: &str, value: &PropValue) -> ComponentResult<ComponentVariant> {
    match value {
        PropValue::String(value) => ComponentVariant::from_name(value)
            .ok_or_else(|| ComponentError::invalid_prop(name, "solid, soft, outlined or ghost")),
        PropValue::Number(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => Err(
            ComponentError::invalid_prop(name, "solid, soft, outlined or ghost"),
        ),
    }
}

fn parse_tabs_variant_prop(name: &str, value: &PropValue) -> ComponentResult<TabsVariant> {
    match value {
        PropValue::String(value) => TabsVariant::from_name(value)
            .ok_or_else(|| ComponentError::invalid_prop(name, "solid, outlined, line, ghost or pills")),
        PropValue::Number(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => Err(
            ComponentError::invalid_prop(name, "solid, outlined, line, ghost or pills"),
        ),
    }
}

fn parse_tabs_position_prop(name: &str, value: &PropValue) -> ComponentResult<TabsPosition> {
    match value {
        PropValue::String(value) => TabsPosition::from_name(value)
            .ok_or_else(|| ComponentError::invalid_prop(name, "top, bottom, start or end")),
        PropValue::Number(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => Err(
            ComponentError::invalid_prop(name, "top, bottom, start or end"),
        ),
    }
}

fn parse_button_size_prop(name: &str, value: &PropValue) -> ComponentResult<ButtonSize> {
    match value {
        PropValue::String(value) => ButtonSize::from_name(value)
            .ok_or_else(|| ComponentError::invalid_prop(name, "xs, sm, md, lg or xl")),
        PropValue::Number(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(name, "xs, sm, md, lg or xl"))
        }
    }
}

fn parse_side_nav_size_prop(name: &str, value: &PropValue) -> ComponentResult<SideNavSize> {
    match value {
        PropValue::String(value) => SideNavSize::from_name(value)
            .ok_or_else(|| ComponentError::invalid_prop(name, "sm, md or lg")),
        PropValue::Number(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(name, "sm, md or lg"))
        }
    }
}

fn parse_table_size_prop(name: &str, value: &PropValue) -> ComponentResult<TableSize> {
    match value {
        PropValue::String(value) => TableSize::from_name(value)
            .ok_or_else(|| ComponentError::invalid_prop(name, "sm, md or lg")),
        PropValue::Number(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(name, "sm, md or lg"))
        }
    }
}

fn parse_table_column_align_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<TableColumnAlign> {
    match value {
        PropValue::String(value) => TableColumnAlign::from_name(value)
            .ok_or_else(|| ComponentError::invalid_prop(name, "start, center or end")),
        PropValue::Number(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(name, "start, center or end"))
        }
    }
}

fn parse_static_string(name: &str, value: &PropValue) -> ComponentResult<String> {
    match value {
        PropValue::String(value) => Ok(value.clone()),
        PropValue::Number(value) => Ok(value.clone()),
        PropValue::Boolean(value) => Ok(value.to_string()),
        PropValue::Responsive(_) => Err(ComponentError::invalid_prop(name, "static scalar")),
    }
}

fn parse_static_bool(name: &str, value: &PropValue) -> ComponentResult<bool> {
    match value {
        PropValue::Boolean(value) => Ok(*value),
        PropValue::String(_) | PropValue::Number(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(name, "boolean"))
        }
    }
}

fn parse_family_prop(
    component: BuiltinComponent,
    name: &str,
    value: &PropValue,
) -> ComponentResult<ColorFamily> {
    let accepts_structural = matches!(
        component,
        BuiltinComponent::Card
            | BuiltinComponent::Code
            | BuiltinComponent::Video
            | BuiltinComponent::Candlestick
            | BuiltinComponent::Table
            | BuiltinComponent::Divider
            | BuiltinComponent::AppBar
            | BuiltinComponent::Footer
            | BuiltinComponent::BottomBar
            | BuiltinComponent::NavMenu
            | BuiltinComponent::SideNav
            | BuiltinComponent::Sidebar
            | BuiltinComponent::Drawer
            | BuiltinComponent::Avatar
            | BuiltinComponent::Badge
            | BuiltinComponent::Chip
            | BuiltinComponent::Modal
            | BuiltinComponent::AlertDialog
            | BuiltinComponent::Tooltip
            | BuiltinComponent::Toast
            | BuiltinComponent::Dropdown
            | BuiltinComponent::Command
            | BuiltinComponent::Accordion
            | BuiltinComponent::Collapsible
            | BuiltinComponent::Countdown
            | BuiltinComponent::Dropzone
            | BuiltinComponent::Tabs
    );
    let expected = if accepts_structural {
        "primary, secondary, tertiary, muted, background, surface, success, info, warning or danger"
    } else {
        "primary, secondary, tertiary, muted, success, info, warning or danger"
    };
    match value {
        PropValue::String(value) => {
            let family = ColorFamily::from_name(value)
                .ok_or_else(|| ComponentError::invalid_prop(name, expected))?;
            if !accepts_structural
                && matches!(family, ColorFamily::Background | ColorFamily::Surface)
            {
                return Err(ComponentError::invalid_prop(name, expected));
            }
            Ok(family)
        }
        PropValue::Number(_) | PropValue::Boolean(_) | PropValue::Responsive(_) => {
            Err(ComponentError::invalid_prop(name, expected))
        }
    }
}

fn parse_show_prop(name: &str, value: &PropValue) -> ComponentResult<VisibilityCondition> {
    match value {
        PropValue::String(value) => {
            if is_reference_path(value) {
                Ok(VisibilityCondition::Signal(value.clone()))
            } else {
                Err(ComponentError::invalid_prop(
                    name,
                    "boolean, responsive boolean or signal bool path",
                ))
            }
        }
        PropValue::Boolean(_) | PropValue::Responsive(_) => parse_responsive(
            name,
            value,
            "boolean",
            |scalar| match scalar {
                PropScalar::Boolean(value) => Some(*value),
                PropScalar::String(_) | PropScalar::Number(_) => None,
            },
        )
        .map(VisibilityCondition::Static),
        PropValue::Number(_) => Err(ComponentError::invalid_prop(
            name,
            "boolean, responsive boolean or signal bool path",
        )),
    }
}

fn is_reference_path(value: &str) -> bool {
    if value.is_empty()
        || value.starts_with('.')
        || value.ends_with('.')
        || value.split('.').any(|part| part.is_empty())
    {
        return false;
    }
    value.split('.').all(is_reference_part)
}

fn is_reference_part(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|value| value.is_ascii_alphanumeric() || value == '_')
}

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

fn parse_grid_template(value: &str) -> Option<GridTracks> {
    if value.trim().is_empty() {
        return None;
    }
    for track in value.split_whitespace() {
        if !is_valid_grid_track(track) {
            return None;
        }
    }
    Some(GridTracks::Template(value.to_string()))
}

fn is_valid_grid_track(value: &str) -> bool {
    if value == "auto" {
        return true;
    }
    if let Some(number) = value
        .strip_suffix("px")
        .or_else(|| value.strip_suffix("fr"))
    {
        return number
            .parse::<u16>()
            .ok()
            .filter(|value| *value > 0)
            .is_some();
    }
    if let Some(number) = value.strip_suffix('%') {
        return number
            .parse::<u16>()
            .ok()
            .filter(|value| (1..=100).contains(value))
            .is_some();
    }
    false
}

fn is_valid_rgba(value: &str) -> bool {
    let Some(inner) = value
        .strip_prefix("rgba(")
        .and_then(|value| value.strip_suffix(')'))
    else {
        return false;
    };
    let parts = inner.split(',').map(str::trim).collect::<Vec<_>>();
    if parts.len() != 4 {
        return false;
    }
    let rgb_valid = parts[..3].iter().all(|part| {
        part.parse::<u16>()
            .ok()
            .filter(|value| *value <= 255)
            .is_some()
    });
    let alpha_valid = parts[3]
        .parse::<f32>()
        .ok()
        .filter(|value| (0.0..=1.0).contains(value))
        .is_some();
    rgb_valid && alpha_valid
}

fn is_valid_linear_gradient(value: &str) -> bool {
    let Some(inner) = value
        .strip_prefix("linear-gradient(")
        .and_then(|value| value.strip_suffix(')'))
    else {
        return false;
    };
    if inner.contains(';') || inner.contains('{') || inner.contains('}') {
        return false;
    }
    inner.contains("rgba(")
        || ColorToken::all()
            .iter()
            .any(|token| inner.contains(token.as_str()))
}
