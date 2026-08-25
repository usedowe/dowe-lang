fn parse_style_props(
    component: BuiltinComponent,
    props: &[ComponentProp],
    mode: StylePropMode,
) -> ComponentResult<StyleProps> {
    let mut style = StyleProps::default();
    let mut scheme = None;

    for prop in props {
        let value = prop.value.binding_fallback().unwrap_or_else(|| prop.value.clone());
        let binding = prop.value.binding().cloned();
        match prop.name.as_str() {
            "id" => style.element.id = Some(parse_id_prop(&prop.name, &value)?),
            "show" => style.element.show = Some(parse_show_prop(&prop.name, &value)?),
            "font" => {
                let font = parse_font_prop(&prop.name, &value)?;
                style.element.font = Some(font.clone());
                style.font = Some(font);
            }
            "bind"
                if matches!(
                    component,
                    BuiltinComponent::Input
                        | BuiltinComponent::Select
                        | BuiltinComponent::ComboBox
                        | BuiltinComponent::Editor
                        | BuiltinComponent::ImageCropper
                        | BuiltinComponent::Password
                        | BuiltinComponent::Phone
                        | BuiltinComponent::Pin
                        | BuiltinComponent::Textarea
                        | BuiltinComponent::Slider
                        | BuiltinComponent::Checkbox
                        | BuiltinComponent::Color
                        | BuiltinComponent::Date
                        | BuiltinComponent::RadioGroup
                        | BuiltinComponent::Toggle
                        | BuiltinComponent::Swap
                ) =>
            {
                style.element.bind = Some(parse_required_string(&prop.name, &value)?)
            }
            "onClick"
                if matches!(
                    component,
                    BuiltinComponent::Button
                        | BuiltinComponent::IconButton
                        | BuiltinComponent::Avatar
                        | BuiltinComponent::Fab
                        | BuiltinComponent::Empty
                        | BuiltinComponent::Box
                        | BuiltinComponent::Card
                        | BuiltinComponent::Chip
                ) =>
            {
                style.element.on_click = Some(parse_required_string(&prop.name, &value)?)
            }
            "scheme" if matches!(mode, StylePropMode::Box | StylePropMode::Section) => {
                scheme = Some(parse_family_prop(component, &prop.name, &value)?);
            }
            "bg" if style_accepts_colors(mode) => {
                style.bg = Some(parse_color_prop(&prop.name, &value)?);
                style.bg_binding = binding;
            }
            "color" if style_accepts_colors(mode) && !matches!(mode, StylePropMode::Card) => {
                style.text = Some(parse_color_prop(&prop.name, &value)?);
                style.text_binding = binding;
            }
            "cover" if style_accepts_cover(mode) => {
                style.cover = Some(parse_cover_prop(&prop.name, &value)?)
            }
            "overlay" if style_accepts_cover(mode) => {
                style.overlay = Some(parse_overlay_prop(&prop.name, &value)?)
            }
            "background" if style_accepts_background(mode) => {
                style.background = Some(parse_background_prop(&prop.name, &value)?)
            }
            "centerX" if matches!(mode, StylePropMode::Box | StylePropMode::Section) => {
                style.center_x = Some(parse_responsive_bool_prop(&prop.name, &value)?)
            }
            "centerY" if matches!(mode, StylePropMode::Box | StylePropMode::Section) => {
                style.center_y = Some(parse_responsive_bool_prop(&prop.name, &value)?)
            }
            "gap" if matches!(mode, StylePropMode::Section) => {
                style.gap = Some(parse_gap_prop(&prop.name, &value, false)?)
            }
            "flex"
                if matches!(
                    component,
                    BuiltinComponent::Section
                        | BuiltinComponent::Box
                        | BuiltinComponent::Grid
                        | BuiltinComponent::Flex
                        | BuiltinComponent::Card
                ) =>
            {
                style.flex = Some(parse_flex_item_prop(&prop.name, &value)?)
            }
            "boxed" if matches!(mode, StylePropMode::Section) => {
                style.boxed = parse_static_bool(&prop.name, &value)?
            }
            "animation" if style_accepts_animation(mode) => {
                style.set_animation(Some(parse_animation_prop(&prop.name, &value)?))
            }
            "rotate" => {
                style.motion_mut().rotate = Some(parse_rotation_prop(&prop.name, &value)?)
            }
            "scale" => {
                style.motion_mut().scale = Some(parse_view_scale_prop(&prop.name, &value)?)
            }
            "translateX" => {
                style.motion_mut().translate_x =
                    Some(parse_translation_prop(&prop.name, &value)?)
            }
            "translateY" => {
                style.motion_mut().translate_y =
                    Some(parse_translation_prop(&prop.name, &value)?)
            }
            "transition" => {
                style.motion_mut().transition =
                    Some(parse_transition_prop(&prop.name, &value)?)
            }
            "gesture" => {
                style.motion_mut().gesture = Some(parse_gesture_prop(&prop.name, &value)?)
            }
            "colSpan" if style_accepts_grid_item(mode) => {
                style.grid_item_mut().col_span = Some(parse_span_prop(&prop.name, &value)?)
            }
            "rowSpan" if style_accepts_grid_item(mode) => {
                style.grid_item_mut().row_span = Some(parse_span_prop(&prop.name, &value)?)
            }
            "position" if matches!(mode, StylePropMode::Box) => {
                let value = parse_required_string(&prop.name, &value)?;
                style.position_mut().mode = BoxPosition::from_name(&value).ok_or_else(|| {
                    ComponentError::invalid_prop("position", "static, relative, absolute or fixed")
                })?;
            }
            "top" if matches!(mode, StylePropMode::Box) => {
                style.position_mut().top = Some(parse_scale_prop(&prop.name, &value)?)
            }
            "right" if matches!(mode, StylePropMode::Box) => {
                style.position_mut().right = Some(parse_scale_prop(&prop.name, &value)?)
            }
            "bottom" if matches!(mode, StylePropMode::Box) => {
                style.position_mut().bottom = Some(parse_scale_prop(&prop.name, &value)?)
            }
            "left" if matches!(mode, StylePropMode::Box) => {
                style.position_mut().left = Some(parse_scale_prop(&prop.name, &value)?)
            }
            "p" => {
                style.spacing.p = Some(parse_scale_prop(&prop.name, &value)?);
                style.spacing.p_binding = binding;
            }
            "px" => { style.spacing.px = Some(parse_scale_prop(&prop.name, &value)?); style.spacing.px_binding = binding; },
            "py" => { style.spacing.py = Some(parse_scale_prop(&prop.name, &value)?); style.spacing.py_binding = binding; },
            "pl" => { style.spacing.pl = Some(parse_scale_prop(&prop.name, &value)?); style.spacing.pl_binding = binding; },
            "pr" => { style.spacing.pr = Some(parse_scale_prop(&prop.name, &value)?); style.spacing.pr_binding = binding; },
            "pt" => { style.spacing.pt = Some(parse_scale_prop(&prop.name, &value)?); style.spacing.pt_binding = binding; },
            "pb" => { style.spacing.pb = Some(parse_scale_prop(&prop.name, &value)?); style.spacing.pb_binding = binding; },
            "w" => {
                style.sizing.w = Some(parse_size_prop(&prop.name, &value)?);
                style.sizing.w_binding = binding;
            }
            "h" => {
                style.sizing.h = Some(parse_size_prop(&prop.name, &value)?);
                style.sizing.h_binding = binding;
            }
            "minW" => { style.sizing.min_w = Some(parse_size_prop(&prop.name, &value)?); style.sizing.min_w_binding = binding; },
            "minH" => { style.sizing.min_h = Some(parse_size_prop(&prop.name, &value)?); style.sizing.min_h_binding = binding; },
            "maxW" => { style.sizing.max_w = Some(parse_size_prop(&prop.name, &value)?); style.sizing.max_w_binding = binding; },
            "maxH" => { style.sizing.max_h = Some(parse_size_prop(&prop.name, &value)?); style.sizing.max_h_binding = binding; },
            "rounded" => {
                style.rounded = Some(parse_rounded_prop(&prop.name, &value)?);
                style.rounded_binding = binding;
            }
            "border" => {
                style.border = Some(parse_border_prop(&prop.name, &value)?);
                style.border_binding = binding;
            }
            "borderColor" => {
                style.border_color = Some(parse_family_prop(component, &prop.name, &value)?)
            }
            "shadow" => style.shadow = Some(parse_shadow_prop(&prop.name, &value)?),
            "shadowColor" => {
                style.shadow_color = Some(parse_family_prop(component, &prop.name, &value)?)
            }
            _ => return Err(ComponentError::unknown_prop(component, &prop.name)),
        }
    }

    if let Some(family) = scheme {
        if style.bg.is_none() {
            style.bg = Some(ResponsiveValue::scalar(family.color_token()));
        }
        if style.text.is_none() {
            style.text = Some(ResponsiveValue::scalar(family.text_token()));
        }
    }

    if style.cover.is_some() && style.background.is_some() {
        return Err(ComponentError::invalid_prop_combination(format!(
            "`cover` and `background` cannot be used together on `{}`",
            component.as_str()
        )));
    }

    if style.overlay.is_some() && style.cover.is_none() {
        return Err(ComponentError::invalid_prop_combination(format!(
            "`overlay` requires `cover` on `{}`",
            component.as_str()
        )));
    }

    let position = style.position();
    let has_position_offset = position.top.is_some()
        || position.right.is_some()
        || position.bottom.is_some()
        || position.left.is_some();
    if has_position_offset && matches!(position.mode, BoxPosition::Static | BoxPosition::Relative) {
        return Err(ComponentError::invalid_prop_combination(
            "`top`, `right`, `bottom` and `left` require `position:\"absolute\"` or `position:\"fixed\"`",
        ));
    }
    if position.top.is_some() && position.bottom.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "`top` and `bottom` cannot be used together on positioned `Box`",
        ));
    }
    if position.left.is_some() && position.right.is_some() {
        return Err(ComponentError::invalid_prop_combination(
            "`left` and `right` cannot be used together on positioned `Box`",
        ));
    }

    Ok(style)
}

fn reactive_reference(value: &PropValue) -> Option<String> {
    match value {
        PropValue::String(value) => value.strip_prefix("@signal:").map(str::to_string),
        PropValue::Binding(binding) => Some(binding.path.clone()),
        _ => None,
    }
}

fn style_accepts_colors(mode: StylePropMode) -> bool {
    matches!(
        mode,
        StylePropMode::Box
            | StylePropMode::Banner
            | StylePropMode::Section
            | StylePropMode::Card
            | StylePropMode::Text
    )
}

fn style_accepts_cover(mode: StylePropMode) -> bool {
    matches!(
        mode,
        StylePropMode::Box | StylePropMode::Banner | StylePropMode::Section | StylePropMode::Card
    )
}

fn style_accepts_background(mode: StylePropMode) -> bool {
    matches!(mode, StylePropMode::Section)
}

fn style_accepts_grid_item(mode: StylePropMode) -> bool {
    matches!(
        mode,
        StylePropMode::Box | StylePropMode::Banner | StylePropMode::Section | StylePropMode::Card
    )
}

fn style_accepts_animation(mode: StylePropMode) -> bool {
    let _ = mode;
    true
}

