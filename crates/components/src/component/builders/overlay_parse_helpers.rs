fn parse_avatar_size(name: &str, value: &PropValue) -> ComponentResult<AvatarSize> {
    let value = parse_required_string(name, value)?;
    AvatarSize::from_name(&value).ok_or_else(|| {
        ComponentError::invalid_prop(name, "xs, sm, md, lg, xl, 2xl, 3xl, 4xl, 5xl, 6xl or 7xl")
    })
}

fn parse_avatar_status(name: &str, value: &PropValue) -> ComponentResult<AvatarStatus> {
    let value = parse_required_string(name, value)?;
    AvatarStatus::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "online, offline, busy or away"))
}

fn parse_overlay_corner_position(
    name: &str,
    value: &PropValue,
) -> ComponentResult<OverlayCornerPosition> {
    let value = parse_required_string(name, value)?;
    OverlayCornerPosition::from_name(&value).ok_or_else(|| {
        ComponentError::invalid_prop(name, "top-left, top-right, bottom-left or bottom-right")
    })
}

fn parse_overlay_position(name: &str, value: &PropValue) -> ComponentResult<OverlayPosition> {
    let value = parse_required_string(name, value)?;
    OverlayPosition::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "top, bottom, start or end"))
}

fn parse_skeleton_variant(name: &str, value: &PropValue) -> ComponentResult<SkeletonVariant> {
    let value = parse_required_string(name, value)?;
    SkeletonVariant::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "text, circular, rectangular or rounded"))
}

fn parse_skeleton_animation(name: &str, value: &PropValue) -> ComponentResult<SkeletonAnimation> {
    let value = parse_required_string(name, value)?;
    SkeletonAnimation::from_name(&value)
        .ok_or_else(|| ComponentError::invalid_prop(name, "pulse, wave or none"))
}

fn parse_toast_kind(name: &str, value: &PropValue) -> ComponentResult<ToastKind> {
    let value = parse_required_string(name, value)?;
    ToastKind::from_name(&value).ok_or_else(|| {
        ComponentError::invalid_prop(
            name,
            "primary, secondary, muted, success, info, warning, danger or error",
        )
    })
}

fn parse_signal_path(name: &str, value: &PropValue, expected: &str) -> ComponentResult<String> {
    let value = match value {
        PropValue::String(value) => value.clone(),
        PropValue::Binding(binding) => binding.path.clone(),
        _ => return Err(ComponentError::invalid_prop(name, expected)),
    };
    if is_reference_path(&value) {
        Ok(value)
    } else {
        Err(ComponentError::invalid_prop(name, expected))
    }
}

fn parse_command_shortcut(name: &str, value: &PropValue) -> ComponentResult<String> {
    let value = parse_required_string(name, value)?;
    if value.chars().count() == 1 && value.chars().all(|value| value.is_ascii_alphanumeric()) {
        Ok(value.to_ascii_lowercase())
    } else {
        Err(ComponentError::invalid_prop(
            name,
            "single ASCII letter or digit",
        ))
    }
}

fn require_solid_variant(
    component: BuiltinComponent,
    variant: Option<ComponentVariant>,
) -> ComponentResult<()> {
    if matches!(
        variant,
        Some(ComponentVariant::Outlined | ComponentVariant::Ghost)
    ) {
        return Err(ComponentError::invalid_prop("variant", "solid"));
    }
    if component == BuiltinComponent::Avatar {
        return Ok(());
    }
    Ok(())
}

fn scheme_prop_error(component: BuiltinComponent) -> ComponentError {
    ComponentError::new(format!(
        "unknown prop `color` on `{}`; use `scheme` for visual family",
        component.as_str()
    ))
}
