fn modifier_for_style_with_base(props: &StyleProps, modifier: String) -> String {
    let mut modifier = modifier_for_style_with_base_and_shadow_shape(props, modifier, None);
    for binding in props.bindings() {
        let path = escape_kotlin(&binding.binding.path);
        modifier.push_str(&format!(".doweReactiveStyle(\"{}\", state.text(\"{}\"))", binding.property.as_str(), path));
    }
    modifier
}

fn modifier_for_style_with_base_and_shadow_shape(
    props: &StyleProps,
    mut modifier: String,
    shadow_shape: Option<&str>,
) -> String {
    if let Some(id) = props.element.id.as_ref() {
        modifier.push_str(&format!(
            ".doweSection(sectionRegistry, \"{}\")",
            escape_kotlin(id)
        ));
    }
    if let Some(value) = props.shadow.as_ref() {
        let (color, alpha) = props
            .shadow_color
            .map(|family| (color_ref(family_color(family)), "0.28f"))
            .unwrap_or(("Color.Black", "null"));
        let shape = shadow_shape.map(str::to_string).unwrap_or_else(|| {
            format!(
                "RoundedCornerShape({} ?: DoweDesign.radius)",
                compose_optional_rounded(props.rounded.as_ref())
            )
        });
        modifier.push_str(&format!(
            ".doweShadow(radius = {} ?: 0.dp, shape = {shape}, color = {color}, alpha = {alpha})",
            compose_shadow_value(value)
        ));
    }
    if let Some(value) = props.rounded.as_ref() {
        modifier.push_str(&format!(".doweRounded({})", compose_rounded_value(value)));
    }
    if let Some(value) = props.bg.as_ref() {
        modifier.push_str(&format!(".doweBackground({})", compose_color_value(value)));
    }
    if props.spacing.p.is_some()
        || props.spacing.px.is_some()
        || props.spacing.py.is_some()
        || props.spacing.pl.is_some()
        || props.spacing.pr.is_some()
        || props.spacing.pt.is_some()
        || props.spacing.pb.is_some()
    {
        modifier.push_str(&format!(
            ".dowePadding(all = {}, horizontal = {}, vertical = {}, start = {}, end = {}, top = {}, bottom = {})",
            compose_optional_scale(props.spacing.p.as_ref()),
            compose_optional_scale(props.spacing.px.as_ref()),
            compose_optional_scale(props.spacing.py.as_ref()),
            compose_optional_scale(props.spacing.pl.as_ref()),
            compose_optional_scale(props.spacing.pr.as_ref()),
            compose_optional_scale(props.spacing.pt.as_ref()),
            compose_optional_scale(props.spacing.pb.as_ref())
        ));
    }
    if let Some(value) = props.sizing.max_w.as_ref() {
        modifier.push_str(&format!(".doweMaxWidth({})", compose_size_value(value)));
    }
    if let Some(value) = props.sizing.max_h.as_ref() {
        modifier.push_str(&format!(".doweMaxHeight({})", compose_size_value(value)));
    }
    if let Some(value) = props.sizing.w.as_ref() {
        modifier.push_str(&format!(".doweWidth({})", compose_size_value(value)));
    }
    if let Some(value) = props.sizing.h.as_ref() {
        modifier.push_str(&format!(".doweHeight({})", compose_size_value(value)));
    }
    if let Some(value) = props.sizing.min_w.as_ref() {
        modifier.push_str(&format!(".doweMinWidth({})", compose_size_value(value)));
    }
    if let Some(value) = props.sizing.min_h.as_ref() {
        modifier.push_str(&format!(".doweMinHeight({})", compose_size_value(value)));
    }
    if let Some(value) = props.border.as_ref() {
        let color = props
            .border_color
            .map(family_color)
            .map(color_ref)
            .unwrap_or("DoweDesign.backgroundText");
        modifier.push_str(&format!(
            ".border({} ?: 0.dp, {color}, RoundedCornerShape({} ?: DoweDesign.radius))",
            compose_border_value(value),
            compose_optional_rounded(props.rounded.as_ref())
        ));
    }
    let motion = props.motion();
    if motion.rotate.is_some()
        || motion.scale.is_some()
        || motion.translate_x.is_some()
        || motion.translate_y.is_some()
    {
        let rotation = motion
            .rotate
            .as_ref()
            .map(|value| {
                format!(
                    "{} ?: 0f",
                    compose_responsive_value(value, |value| format!("{}f", value.degrees()))
                )
            })
            .unwrap_or_else(|| "0f".to_string());
        let scale = motion
            .scale
            .as_ref()
            .map(|value| {
                format!(
                    "{} ?: 1f",
                    compose_responsive_value(value, |value| format!("{}f", value.factor()))
                )
            })
            .unwrap_or_else(|| "1f".to_string());
        let translate_x = motion
            .translate_x
            .as_ref()
            .map(|value| {
                format!(
                    "({} ?: 0.dp).toPx()",
                    compose_responsive_value(value, |value| format!("{}.dp", value.native_units()))
                )
            })
            .unwrap_or_else(|| "0f".to_string());
        let translate_y = motion
            .translate_y
            .as_ref()
            .map(|value| {
                format!(
                    "({} ?: 0.dp).toPx()",
                    compose_responsive_value(value, |value| format!("{}.dp", value.native_units()))
                )
            })
            .unwrap_or_else(|| "0f".to_string());
        modifier.push_str(&format!(
            ".graphicsLayer {{ rotationZ = {rotation}; scaleX = {scale}; scaleY = {scale}; translationX = {translate_x}; translationY = {translate_y} }}"
        ));
    }
    if let Some(gesture) = motion.gesture
        && gesture != ViewGesture::None
    {
        let transition = motion.transition.unwrap_or(ViewTransition::Smooth);
        modifier.push_str(&format!(
            ".doweGesture(DoweGesturePreset.{}, DoweTransitionPreset.{})",
            title_case(gesture.as_str()),
            title_case(transition.as_str())
        ));
    }
    if let Some(animation) = props.animation() {
        modifier.push_str(&format!(
            ".doweAnimation({})",
            compose_animation_preset(animation)
        ));
    }
    modifier
}

