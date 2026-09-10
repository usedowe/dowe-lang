fn swift_gesture_modifier(props: &StyleProps) -> Option<String> {
    let motion = props.motion();
    let gesture = motion.gesture?;
    if gesture == ViewGesture::None {
        return None;
    }
    let transition = motion.transition.unwrap_or(ViewTransition::Smooth);
    Some(format!(
        ".modifier(DoweGestureModifier(preset: .{}, transition: .{}))",
        gesture.as_str(),
        transition.as_str()
    ))
}

fn swift_style_without_gesture(props: &StyleProps) -> StyleProps {
    let mut style = props.clone();
    if style.motion().gesture.is_some() {
        style.motion_mut().gesture = None;
    }
    style
}

fn swift_shadow_modifier(props: &StyleProps) -> Option<String> {
    props.shadow.as_ref().map(|value| {
        format!(
            ".shadow(color: {}, radius: {} ?? CGFloat(0), x: CGFloat(0), y: {} ?? CGFloat(0))",
            swift_shadow_color(props, value),
            swift_shadow_value(value),
            swift_shadow_offset_value(value)
        )
    })
}

fn swift_shadow_modifier_with_radius(props: &StyleProps, corner_radius: &str) -> Option<String> {
    swift_shadow_spec(props).map(|shadow| {
        format!(".background(DoweShadowSurface(shadow: {shadow}, cornerRadius: {corner_radius}))")
    })
}

fn swift_shadow_spec(props: &StyleProps) -> Option<String> {
    props.shadow.as_ref().map(|value| {
        format!(
            "DoweShadowSpec(color: {}, blurRadius: {} ?? CGFloat(0), offsetY: {} ?? CGFloat(0))",
            swift_shadow_color(props, value),
            swift_shadow_value(value),
            swift_shadow_offset_value(value)
        )
    })
}

fn swift_style_border(
    props: &StyleProps,
    fallback_color: &str,
    fallback_width: &str,
) -> (String, String) {
    props.border.as_ref().map_or_else(
        || (fallback_color.to_string(), fallback_width.to_string()),
        |width| {
            let width = swift_border_value(width);
            let color = props
                .border_color
                .map(family_color)
                .map(color_ref)
                .unwrap_or("DoweDesign.backgroundText");
            (
                format!("({width}) == nil ? {fallback_color} : Optional({color})"),
                format!("{width} ?? {fallback_width}"),
            )
        },
    )
}

fn swift_shadow_color(props: &StyleProps, value: &ResponsiveValue<ShadowSize>) -> String {
    props
        .shadow_color
        .map(family_color)
        .map(color_ref)
        .map(|color| format!("{color}.opacity(0.28)"))
        .unwrap_or_else(|| {
            format!(
                "Color.black.opacity({} ?? Double(0))",
                swift_shadow_opacity_value(value)
            )
        })
}

fn swift_padding_edge(
    side: Option<&ResponsiveValue<ScaleValue>>,
    axis: Option<&ResponsiveValue<ScaleValue>>,
    all: Option<&ResponsiveValue<ScaleValue>>,
) -> String {
    let values = [side, axis, all]
        .into_iter()
        .flatten()
        .map(swift_scale_value)
        .collect::<Vec<_>>();
    if values.is_empty() {
        "CGFloat(0)".to_string()
    } else {
        format!("{} ?? CGFloat(0)", values.join(" ?? "))
    }
}

fn append_swift_modifiers(output: &mut String, indent: usize, modifiers: &[String]) {
    let pad = " ".repeat(indent + 4);
    for modifier in modifiers {
        output.push_str(&format!("{pad}{modifier}\n"));
    }
}

fn swift_gap(value: Option<&ResponsiveValue<GapValue>>) -> String {
    value
        .map(|value| swift_responsive_value(value, |value| swift_gap_value(value)))
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_flex_justify(value: Option<&ResponsiveValue<Justify>>) -> String {
    value
        .map(swift_justify_value)
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_flex_align(value: Option<&ResponsiveValue<Align>>) -> String {
    value
        .map(swift_align_value)
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_vertical_alignment(value: Option<&ResponsiveValue<Align>>) -> String {
    value
        .map(|value| format!("doweVerticalAlignment({})", swift_align_value(value)))
        .unwrap_or_else(|| ".center".to_string())
}

fn swift_horizontal_alignment(value: Option<&ResponsiveValue<Align>>) -> String {
    value
        .map(|value| format!("doweHorizontalAlignment({})", swift_align_value(value)))
        .unwrap_or_else(|| ".leading".to_string())
}

fn swift_frame_alignment(value: Option<&ResponsiveValue<Justify>>) -> String {
    value
        .map(|value| format!("doweFrameAlignment({})", swift_justify_value(value)))
        .unwrap_or_else(|| ".leading".to_string())
}

fn swift_column_frame_alignment(value: Option<&ResponsiveValue<Align>>) -> String {
    value
        .map(|value| format!("doweColumnFrameAlignment({})", swift_align_value(value)))
        .unwrap_or_else(|| ".leading".to_string())
}

fn swift_grid_alignment(value: Option<&ResponsiveValue<GridAlignment>>) -> String {
    value
        .map(swift_grid_alignment_value)
        .unwrap_or_else(|| "nil".to_string())
}

fn swift_control_radius(props: &StyleProps) -> String {
    props
        .rounded
        .as_ref()
        .map(|value| format!("{} ?? DoweDesign.radius", swift_rounded_value(value)))
        .unwrap_or_else(|| "DoweDesign.radius".to_string())
}

fn swift_card_radius(props: &StyleProps) -> String {
    props
        .rounded
        .as_ref()
        .map(swift_rounded_value)
        .map(|value| format!("{value} ?? DoweDesign.radius"))
        .unwrap_or_else(|| "DoweDesign.radius".to_string())
}

fn swift_drawer_radius(props: &StyleProps) -> String {
    props
        .rounded
        .as_ref()
        .map(swift_rounded_value)
        .map(|value| format!("{value} ?? CGFloat(0)"))
        .unwrap_or_else(|| "CGFloat(0)".to_string())
}

fn swift_animation_preset(value: ViewAnimation) -> &'static str {
    match value {
        ViewAnimation::None => ".none",
        ViewAnimation::FadeIn => ".fadeIn",
        ViewAnimation::SlideUp => ".slideUp",
        ViewAnimation::SlideDown => ".slideDown",
        ViewAnimation::SlideLeft => ".slideLeft",
        ViewAnimation::SlideRight => ".slideRight",
        ViewAnimation::ScaleIn => ".scaleIn",
    }
}
