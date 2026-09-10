fn swift_modifiers_for_style(props: &StyleProps) -> Vec<String> {
    let mut modifiers = swift_modifiers_for_style_with_width_alignment(props, None);
    modifiers.extend(swift_modifiers_for_style_bindings(props));
    modifiers
}

fn swift_modifiers_for_style_bindings(props: &StyleProps) -> Vec<String> {
    props
        .bindings()
        .into_iter()
        .map(|binding| {
            let path = escape_swift(&binding.binding.path);
            let value = format!("state.text(\"{}\")", path);
            match binding.property {
                dowe_components::StyleBindingProperty::TextColor => format!(".foregroundStyle(doweDynamicColor({value}))"),
                dowe_components::StyleBindingProperty::BackgroundColor => format!(".background(doweDynamicColor({value}))"),
                dowe_components::StyleBindingProperty::Padding => format!(".padding(CGFloat(Double({value}) ?? 0) / 8.0)"),
                dowe_components::StyleBindingProperty::PaddingInline => format!(".padding(.horizontal, CGFloat(Double({value}) ?? 0) / 8.0)"),
                dowe_components::StyleBindingProperty::PaddingBlock => format!(".padding(.vertical, CGFloat(Double({value}) ?? 0) / 8.0)"),
                dowe_components::StyleBindingProperty::PaddingLeft => format!(".padding(.leading, CGFloat(Double({value}) ?? 0) / 8.0)"),
                dowe_components::StyleBindingProperty::PaddingRight => format!(".padding(.trailing, CGFloat(Double({value}) ?? 0) / 8.0)"),
                dowe_components::StyleBindingProperty::PaddingTop => format!(".padding(.top, CGFloat(Double({value}) ?? 0) / 8.0)"),
                dowe_components::StyleBindingProperty::PaddingBottom => format!(".padding(.bottom, CGFloat(Double({value}) ?? 0) / 8.0)"),
                dowe_components::StyleBindingProperty::Width => format!(".frame(width: CGFloat(Double({value}) ?? 0) / 8.0)"),
                dowe_components::StyleBindingProperty::Height => format!(".frame(height: CGFloat(Double({value}) ?? 0) / 8.0)"),
                dowe_components::StyleBindingProperty::MinWidth => format!(".frame(minWidth: CGFloat(Double({value}) ?? 0) / 8.0)"),
                dowe_components::StyleBindingProperty::MinHeight => format!(".frame(minHeight: CGFloat(Double({value}) ?? 0) / 8.0)"),
                dowe_components::StyleBindingProperty::MaxWidth => format!(".frame(maxWidth: CGFloat(Double({value}) ?? 0) / 8.0)"),
                dowe_components::StyleBindingProperty::MaxHeight => format!(".frame(maxHeight: CGFloat(Double({value}) ?? 0) / 8.0)"),
                dowe_components::StyleBindingProperty::BorderWidth => format!(".overlay(RoundedRectangle(cornerRadius: 0).stroke(Color.primary, lineWidth: CGFloat(Double({value}) ?? 0)) )"),
                dowe_components::StyleBindingProperty::BorderRadius => format!(".clipShape(RoundedRectangle(cornerRadius: CGFloat(Double({value}) ?? 0)))"),
            }
        })
        .collect()
}

fn swift_modifiers_for_svg(props: &SvgProps) -> Vec<String> {
    let mut modifiers = swift_modifiers_for_style(&props.style);
    let has_full_dimension = props.style.sizing.w.as_ref().is_some_and(|value| {
        value
            .entries
            .iter()
            .any(|entry| entry.value == SizeValue::Full)
    }) || props.style.sizing.h.as_ref().is_some_and(|value| {
        value
            .entries
            .iter()
            .any(|entry| entry.value == SizeValue::Full)
    });
    let has_single_dimension = props.style.sizing.w.is_some() != props.style.sizing.h.is_some();
    if props.data.is_none()
        && props.icon_name.is_none()
        && props.motion.is_none()
        && (has_single_dimension || has_full_dimension)
        && let Some(ratio) = props.view_box.aspect_ratio()
    {
        modifiers.push(format!(
            ".aspectRatio(CGFloat({ratio:.6}), contentMode: .fit)"
        ));
    }
    modifiers
}

fn swift_modifiers_for_style_with_width_alignment(
    props: &StyleProps,
    width_alignment: Option<&str>,
) -> Vec<String> {
    let mut modifiers = Vec::new();
    if let Some(id) = props.element.id.as_ref() {
        modifiers.push(format!(".id(\"{}\")", escape_swift(id)));
    }
    if props.spacing.p.is_some()
        || props.spacing.px.is_some()
        || props.spacing.py.is_some()
        || props.spacing.pl.is_some()
        || props.spacing.pr.is_some()
        || props.spacing.pt.is_some()
        || props.spacing.pb.is_some()
    {
        modifiers.push(format!(
            ".padding(EdgeInsets(top: {}, leading: {}, bottom: {}, trailing: {}))",
            swift_padding_edge(
                props.spacing.pt.as_ref(),
                props.spacing.py.as_ref(),
                props.spacing.p.as_ref()
            ),
            swift_padding_edge(
                props.spacing.pl.as_ref(),
                props.spacing.px.as_ref(),
                props.spacing.p.as_ref()
            ),
            swift_padding_edge(
                props.spacing.pb.as_ref(),
                props.spacing.py.as_ref(),
                props.spacing.p.as_ref()
            ),
            swift_padding_edge(
                props.spacing.pr.as_ref(),
                props.spacing.px.as_ref(),
                props.spacing.p.as_ref()
            )
        ));
    }
    if let Some(value) = props.sizing.w.as_ref() {
        let expression = swift_size_value(value);
        let alignment = width_alignment
            .map(|value| format!(", alignment: {value}"))
            .unwrap_or_default();
        modifiers.push(format!(
            ".frame(width: doweFixedSize({expression}){alignment})"
        ));
        modifiers.push(format!(
            ".frame(maxWidth: doweMaxSize({expression}){alignment})"
        ));
    }
    if let Some(value) = props.sizing.h.as_ref() {
        let expression = swift_size_value(value);
        modifiers.push(format!(
            ".frame(height: doweFixedSize({0}, viewportHeight: viewportHeight))",
            expression
        ));
        modifiers.push(format!(".frame(maxHeight: doweMaxSize({0}))", expression));
    }
    if let Some(value) = props.sizing.min_w.as_ref() {
        let expression = swift_size_value(value);
        modifiers.push(format!(".frame(minWidth: doweFixedSize({expression}))"));
    }
    if let Some(value) = props.sizing.min_h.as_ref() {
        modifiers.push(format!(
            ".frame(minHeight: doweFixedSize({}, viewportHeight: viewportHeight))",
            swift_size_value(value)
        ));
        if value
            .entries
            .iter()
            .any(|entry| entry.value == SizeValue::Full)
        {
            modifiers.push(format!(
                ".frame(maxHeight: doweMaxSize({}))",
                swift_size_value(value)
            ));
        }
    }
    if let Some(value) = props.sizing.max_w.as_ref() {
        modifiers.push(format!(
            ".frame(maxWidth: doweFixedSize({}))",
            swift_size_value(value)
        ));
    }
    if let Some(value) = props.sizing.max_h.as_ref() {
        let expression = swift_size_value(value);
        modifiers.push(format!(
            ".frame(maxHeight: doweFixedSize({expression}, viewportHeight: viewportHeight))"
        ));
        modifiers.push(format!(".doweMaxHeight({expression})"));
    }
    let has_percentage_width = props
        .sizing
        .w
        .iter()
        .chain(props.sizing.min_w.iter())
        .flat_map(|value| &value.entries)
        .any(|entry| matches!(entry.value, SizeValue::Percent(_)));
    if has_percentage_width {
        let width = props
            .sizing
            .w
            .as_ref()
            .map(swift_size_value)
            .unwrap_or_else(|| "nil".to_string());
        let min_width = props
            .sizing
            .min_w
            .as_ref()
            .map(swift_size_value)
            .unwrap_or_else(|| "nil".to_string());
        modifiers.push(format!(
            ".dowePercentageWidth(width: {width}, minWidth: {min_width})"
        ));
    }
    if let Some(value) = props.bg.as_ref() {
        modifiers.push(format!(
            ".background({} ?? Color.clear)",
            swift_color_value(value)
        ));
    }
    if let Some(value) = props.text.as_ref() {
        modifiers.push(format!(
            ".foregroundStyle({} ?? DoweDesign.backgroundText)",
            swift_color_value(value)
        ));
    }
    if let Some(value) = props.rounded.as_ref() {
        modifiers.push(format!(
            ".clipShape(RoundedRectangle(cornerRadius: {} ?? DoweDesign.radius))",
            swift_rounded_value(value)
        ));
    }
    if let Some(value) = props.border.as_ref() {
        let radius = props
            .rounded
            .as_ref()
            .map(|value| format!("{} ?? DoweDesign.radius", swift_rounded_value(value)))
            .unwrap_or_else(|| "DoweDesign.radius".to_string());
        let border_color = props
            .border_color
            .map(family_color)
            .map(color_ref)
            .unwrap_or("DoweDesign.backgroundText");
        modifiers.push(format!(
            ".overlay(RoundedRectangle(cornerRadius: {radius}).stroke({border_color}, lineWidth: {} ?? CGFloat(0)))",
            swift_border_value(value)
        ));
    }
    if let Some(modifier) = swift_shadow_modifier(props) {
        modifiers.push(modifier);
    }
    let motion = props.motion();
    if let Some(value) = motion.rotate.as_ref() {
        modifiers.push(format!(
            ".rotationEffect(.degrees({} ?? Double(0)))",
            swift_responsive_value(value, |value| format!("Double({})", value.degrees()))
        ));
    }
    if let Some(value) = motion.scale.as_ref() {
        modifiers.push(format!(
            ".scaleEffect(CGFloat({} ?? Double(1)))",
            swift_responsive_value(value, |value| format!("Double({})", value.factor()))
        ));
    }
    let translate_x = motion
        .translate_x
        .as_ref()
        .map(|value| {
            format!(
                "CGFloat({} ?? Double(0))",
                swift_responsive_value(value, |value| format!("Double({})", value.native_units()))
            )
        })
        .unwrap_or_else(|| "CGFloat(0)".to_string());
    let translate_y = motion
        .translate_y
        .as_ref()
        .map(|value| {
            format!(
                "CGFloat({} ?? Double(0))",
                swift_responsive_value(value, |value| format!("Double({})", value.native_units()))
            )
        })
        .unwrap_or_else(|| "CGFloat(0)".to_string());
    if motion.translate_x.is_some() || motion.translate_y.is_some() {
        modifiers.push(format!(".offset(x: {translate_x}, y: {translate_y})"));
    }
    if let Some(modifier) = swift_gesture_modifier(props) {
        modifiers.push(modifier);
    }
    if let Some(animation) = props.animation() {
        modifiers.push(format!(
            ".modifier(DoweAnimationModifier(preset: {}))",
            swift_animation_preset(animation)
        ));
    }
    modifiers
}

