fn parse_animation_prop(name: &str, value: &PropValue) -> ComponentResult<ViewAnimation> {
    match value {
        PropValue::String(value) => ViewAnimation::from_name(value).ok_or_else(|| {
            ComponentError::invalid_prop(
                name,
                "none, fadeIn, slideUp, slideDown, slideLeft, slideRight or scaleIn",
            )
        }),
        PropValue::Number(_)
        | PropValue::Boolean(_)
        | PropValue::Responsive(_)
        | PropValue::Binding(_) => Err(ComponentError::invalid_prop(
            name,
            "none, fadeIn, slideUp, slideDown, slideLeft, slideRight or scaleIn",
        )),
    }
}

fn parse_rotation_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<ViewRotation>> {
    parse_responsive(
        name,
        value,
        "whole degrees from -180 to 180",
        |scalar| match scalar {
            PropScalar::Number(value) => value
                .parse::<i16>()
                .ok()
                .filter(|value| (-180..=180).contains(value))
                .map(ViewRotation),
            PropScalar::String(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_view_scale_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<ViewScale>> {
    parse_responsive(
        name,
        value,
        "decimal factor from 0.5 to 2",
        |scalar| match scalar {
            PropScalar::Number(value) => parse_decimal_hundredths(value)
                .filter(|value| (50..=200).contains(value))
                .map(ViewScale),
            PropScalar::String(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_translation_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<ResponsiveValue<ViewTranslation>> {
    parse_responsive(
        name,
        value,
        "signed Dowe scale value from -96 to 96 in half steps",
        |scalar| match scalar {
            PropScalar::Number(value) => signed_half_steps(value)
                .filter(|value| (-192..=192).contains(value))
                .map(ViewTranslation),
            PropScalar::String(_) | PropScalar::Boolean(_) => None,
        },
    )
}

fn parse_transition_prop(name: &str, value: &PropValue) -> ComponentResult<ViewTransition> {
    match value {
        PropValue::String(value) => ViewTransition::from_name(value)
            .ok_or_else(|| ComponentError::invalid_prop(name, "none, quick, smooth or spring")),
        PropValue::Number(_)
        | PropValue::Boolean(_)
        | PropValue::Responsive(_)
        | PropValue::Binding(_) => Err(ComponentError::invalid_prop(
            name,
            "none, quick, smooth or spring",
        )),
    }
}

fn parse_gesture_prop(name: &str, value: &PropValue) -> ComponentResult<ViewGesture> {
    match value {
        PropValue::String(value) => ViewGesture::from_name(value)
            .ok_or_else(|| ComponentError::invalid_prop(name, "none, lift, press, grow or tilt")),
        PropValue::Number(_)
        | PropValue::Boolean(_)
        | PropValue::Responsive(_)
        | PropValue::Binding(_) => Err(ComponentError::invalid_prop(
            name,
            "none, lift, press, grow or tilt",
        )),
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
        PropValue::String(value) => ComponentVariant::from_name(value).ok_or_else(|| {
            ComponentError::invalid_prop(name, "solid, outline, outlined, line or ghost")
        }),
        PropValue::Number(_)
        | PropValue::Boolean(_)
        | PropValue::Responsive(_)
        | PropValue::Binding(_) => Err(ComponentError::invalid_prop(
            name,
            "solid, outline, outlined, line or ghost",
        )),
    }
}

fn parse_tabs_variant_prop(name: &str, value: &PropValue) -> ComponentResult<TabsVariant> {
    match value {
        PropValue::String(value) => TabsVariant::from_name(value).ok_or_else(|| {
            ComponentError::invalid_prop(name, "solid, outlined, line, ghost or pills")
        }),
        PropValue::Number(_)
        | PropValue::Boolean(_)
        | PropValue::Responsive(_)
        | PropValue::Binding(_) => Err(ComponentError::invalid_prop(
            name,
            "solid, outlined, line, ghost or pills",
        )),
    }
}

fn parse_tabs_position_prop(name: &str, value: &PropValue) -> ComponentResult<TabsPosition> {
    match value {
        PropValue::String(value) => TabsPosition::from_name(value)
            .ok_or_else(|| ComponentError::invalid_prop(name, "top, bottom, start or end")),
        PropValue::Number(_)
        | PropValue::Boolean(_)
        | PropValue::Responsive(_)
        | PropValue::Binding(_) => Err(ComponentError::invalid_prop(
            name,
            "top, bottom, start or end",
        )),
    }
}

fn parse_button_size_prop(name: &str, value: &PropValue) -> ComponentResult<ButtonSize> {
    match value {
        PropValue::String(value) => ButtonSize::from_name(value)
            .ok_or_else(|| ComponentError::invalid_prop(name, "xs, sm, md, lg or xl")),
        PropValue::Number(_)
        | PropValue::Boolean(_)
        | PropValue::Responsive(_)
        | PropValue::Binding(_) => Err(ComponentError::invalid_prop(name, "xs, sm, md, lg or xl")),
    }
}

fn parse_side_nav_size_prop(name: &str, value: &PropValue) -> ComponentResult<SideNavSize> {
    match value {
        PropValue::String(value) => SideNavSize::from_name(value)
            .ok_or_else(|| ComponentError::invalid_prop(name, "sm, md or lg")),
        PropValue::Number(_)
        | PropValue::Boolean(_)
        | PropValue::Responsive(_)
        | PropValue::Binding(_) => Err(ComponentError::invalid_prop(name, "sm, md or lg")),
    }
}

fn parse_table_size_prop(name: &str, value: &PropValue) -> ComponentResult<TableSize> {
    match value {
        PropValue::String(value) => TableSize::from_name(value)
            .ok_or_else(|| ComponentError::invalid_prop(name, "sm, md or lg")),
        PropValue::Number(_)
        | PropValue::Boolean(_)
        | PropValue::Responsive(_)
        | PropValue::Binding(_) => Err(ComponentError::invalid_prop(name, "sm, md or lg")),
    }
}

fn parse_table_column_align_prop(
    name: &str,
    value: &PropValue,
) -> ComponentResult<TableColumnAlign> {
    match value {
        PropValue::String(value) => TableColumnAlign::from_name(value)
            .ok_or_else(|| ComponentError::invalid_prop(name, "start, center or end")),
        PropValue::Number(_)
        | PropValue::Boolean(_)
        | PropValue::Responsive(_)
        | PropValue::Binding(_) => Err(ComponentError::invalid_prop(name, "start, center or end")),
    }
}

fn parse_static_string(name: &str, value: &PropValue) -> ComponentResult<String> {
    match value {
        PropValue::String(value) => Ok(value.clone()),
        PropValue::Number(value) => Ok(value.clone()),
        PropValue::Boolean(value) => Ok(value.to_string()),
        PropValue::Responsive(_) | PropValue::Binding(_) => {
            Err(ComponentError::invalid_prop(name, "static scalar"))
        }
    }
}

fn parse_static_bool(name: &str, value: &PropValue) -> ComponentResult<bool> {
    match value {
        PropValue::Boolean(value) => Ok(*value),
        PropValue::String(_)
        | PropValue::Number(_)
        | PropValue::Responsive(_)
        | PropValue::Binding(_) => Err(ComponentError::invalid_prop(name, "boolean")),
    }
}


