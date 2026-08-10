fn render_compose_text(
    title: bool,
    props: &TextProps,
    value: &str,
    font: Option<&ResponsiveValue<FontFamily>>,
    indent: usize,
    output: &mut String,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let size = text_size(title, props);
    let spacing = if title || props.letter_spacing.is_some() {
        format!(", letterSpacing = {}", text_spacing(title, props))
    } else {
        String::new()
    };
    output.push_str(&format!(
        "{pad}Text({}, modifier = {}, color = {}, fontSize = {size}, lineHeight = {}, fontFamily = {}, fontWeight = {}{spacing})\n",
        compose_visible_text_expression(value, props.i18n.as_deref(), context),
        modifier_for_style(&props.style),
        text_color(props),
        text_line_height(title, props, &size),
        compose_font_value(font, default_family),
        text_weight(title, props)
    ));
}

fn compose_text_expression(
    value: &str,
    i18n: Option<&str>,
    context: &ComposeReactiveContext,
) -> String {
    if let Some(key) = i18n {
        return format!("stringResource(R.string.{})", translation_resource_name(key));
    }
    match context.dynamic_path(value) {
        Some(path) => context
            .item_value(value)
            .map(|item| format!("state.text(\"{}\", {item})", escape_kotlin(&path)))
            .unwrap_or_else(|| format!("state.text(\"{}\")", escape_kotlin(&path))),
        None => format!("\"{}\"", escape_kotlin(value)),
    }
}

fn compose_visible_text_expression(
    value: &str,
    i18n: Option<&str>,
    context: &ComposeReactiveContext,
) -> String {
    if let Some(key) = i18n {
        return format!("stringResource(R.string.{})", translation_resource_name(key));
    }
    let Some(binding) = text_binding_path(value) else {
        return format!("\"{}\"", escape_kotlin(value));
    };
    match context.dynamic_path(binding) {
        Some(path) => context
            .item_value(binding)
            .map(|item| format!("state.text(\"{}\", {item})", escape_kotlin(&path)))
            .unwrap_or_else(|| format!("state.text(\"{}\")", escape_kotlin(&path))),
        None => format!("\"{}\"", escape_kotlin(value)),
    }
}

fn compose_localized_literal(value: &str, i18n: Option<&str>) -> String {
    i18n
        .map(|key| format!("stringResource(R.string.{})", translation_resource_name(key)))
        .unwrap_or_else(|| format!("\"{}\"", escape_kotlin(value)))
}

fn modifier_for_layout(props: &LayoutProps, flow: ComposeFlow) -> String {
    modifier_for_container_style(&props.style, flow)
}

fn modifier_for_grid(props: &GridProps, flow: ComposeFlow) -> String {
    modifier_for_container_style(&props.style, flow)
}

fn compose_grid_column_count(value: Option<&ResponsiveValue<GridTracks>>) -> String {
    value
        .map(|value| {
            format!(
                "{} ?: 1",
                compose_responsive_value(value, |value| value.count().unwrap_or(1).to_string())
            )
        })
        .unwrap_or_else(|| "1".to_string())
}

fn compose_grid_horizontal_gap(value: Option<&ResponsiveValue<GapValue>>) -> String {
    value
        .map(|value| {
            format!(
                "{} ?: 0.dp",
                compose_responsive_value(value, |value| match value {
                    GapValue::Single(value) | GapValue::Pair(_, value) => compose_gap_size(value),
                })
            )
        })
        .unwrap_or_else(|| "0.dp".to_string())
}

fn compose_grid_vertical_gap(value: Option<&ResponsiveValue<GapValue>>) -> String {
    value
        .map(|value| format!("{} ?: 0.dp", compose_gap_value(value)))
        .unwrap_or_else(|| "0.dp".to_string())
}

fn compose_navigation_action(action: Option<&NavigationAction>) -> String {
    match action {
        Some(NavigationAction::Internal {
            path,
            fragment,
            operation,
        }) => format!(
            r#"{{ navigate("{}", "{}", {}) }}"#,
            operation.as_str(),
            escape_kotlin(path),
            fragment
                .as_ref()
                .map(|value| format!(r#""{}""#, escape_kotlin(value)))
                .unwrap_or_else(|| "null".to_string())
        ),
        Some(NavigationAction::Section {
            fragment,
            operation,
        }) => {
            format!(
                r#"{{ navigate("{}", "", "{}") }}"#,
                operation.as_str(),
                escape_kotlin(fragment)
            )
        }
        Some(NavigationAction::External {
            url,
            native_external_mode,
            ..
        }) => format!(
            r#"{{ openExternal("{}", "{}") }}"#,
            native_external_mode.as_str(),
            escape_kotlin(url)
        ),
        Some(NavigationAction::Back) => "{ goBack() }".to_string(),
        None => "{}".to_string(),
    }
}

fn modifier_for_style(props: &StyleProps) -> String {
    modifier_for_style_with_base(props, "Modifier".to_string())
}

fn modifier_for_style_with_shadow_shape(props: &StyleProps, shadow_shape: &str) -> String {
    modifier_for_style_with_base_and_shadow_shape(
        props,
        "Modifier".to_string(),
        Some(shadow_shape),
    )
}

fn modifier_for_avatar_style(props: &StyleProps) -> String {
    modifier_for_style_with_base_and_shadow_shape(
        props,
        "Modifier".to_string(),
        Some("RoundedCornerShape(999.dp)"),
    )
}

fn compose_content_color(props: &StyleProps) -> Option<String> {
    props.text.as_ref().map(compose_color_value)
}

fn modifier_for_container_style(props: &StyleProps, flow: ComposeFlow) -> String {
    let mut modifier = String::from("Modifier");
    if flow == ComposeFlow::Block && props.sizing.w.is_none() {
        modifier.push_str(".fillMaxWidth()");
    }
    modifier_for_style_with_base(props, modifier)
}

fn compose_position_modifier(props: &PositionProps) -> String {
    let vertical = if props.bottom.is_some() { "Bottom" } else { "Top" };
    let horizontal = if props.right.is_some() { "End" } else { "Start" };
    let mut modifier = format!(".align(Alignment.{vertical}{horizontal})");
    let mut values = Vec::new();
    if let Some(value) = props.top.as_ref() {
        values.push(format!("top = {} ?: 0.dp", compose_scale_value(value)));
    }
    if let Some(value) = props.right.as_ref() {
        values.push(format!("end = {} ?: 0.dp", compose_scale_value(value)));
    }
    if let Some(value) = props.bottom.as_ref() {
        values.push(format!("bottom = {} ?: 0.dp", compose_scale_value(value)));
    }
    if let Some(value) = props.left.as_ref() {
        values.push(format!("start = {} ?: 0.dp", compose_scale_value(value)));
    }
    if !values.is_empty() {
        modifier.push_str(&format!(".padding({})", values.join(", ")));
    }
    modifier
}

fn modifier_for_section_container(props: &StyleProps, flow: ComposeFlow) -> String {
    let mut outer = props.clone();
    outer.spacing = Default::default();
    modifier_for_container_style(&outer, flow)
}

fn modifier_for_section_content(props: &StyleProps) -> String {
    let mut content = StyleProps::default();
    content.spacing = dowe_components::section_content_spacing(&props.spacing);
    let modifier = if props.boxed {
        "Modifier.widthIn(max = 1536.dp).fillMaxWidth()".to_string()
    } else {
        "Modifier".to_string()
    };
    modifier_for_style_with_base(&content, modifier)
}

fn modifier_for_bar(props: &BarProps, flow: ComposeFlow) -> String {
    let mut modifier = String::from("Modifier");
    if flow == ComposeFlow::Block && props.style.style.sizing.w.is_none() {
        modifier.push_str(".fillMaxWidth()");
    }
    modifier.push_str(".heightIn(min = 48.dp)");
    if props.position != BarPosition::Static {
        modifier.push_str(".zIndex(1f)");
    }
    if props.floating && !props.dock_on_scroll {
        modifier.push_str(".padding(horizontal = 16.dp, vertical = 8.dp)");
    }
    modifier = modifier_for_style_with_base(&props.style.style, modifier);
    if props.dock_on_scroll {
        return format!(
            "doweDockingAppBarModifier({modifier}, scrollState, {})",
            variant_container(&props.style)
        );
    }
    if props.floating {
        modifier.push_str(".clip(RoundedCornerShape(DoweDesign.radius))");
    }
    modifier.push_str(&format!(".background({})", variant_container(&props.style)));
    if props.bordered || props.floating {
        let radius = if props.floating {
            "DoweDesign.radius"
        } else {
            "0.dp"
        };
        modifier.push_str(&format!(
            ".border(1.dp, DoweDesign.muted, RoundedCornerShape({radius}))"
        ));
    }
    modifier
}

fn modifier_for_divider(props: &DividerProps, flow: ComposeFlow) -> String {
    let mut modifier = String::from("Modifier");
    match props.orientation {
        DividerOrientation::Horizontal => {
            if flow == ComposeFlow::Block && props.style.sizing.w.is_none() {
                modifier.push_str(".fillMaxWidth()");
            }
            if props.style.sizing.h.is_none() {
                modifier.push_str(".height(1.dp)");
            }
        }
        DividerOrientation::Vertical => {
            if props.style.sizing.w.is_none() {
                modifier.push_str(".width(1.dp)");
            }
            if props.style.sizing.h.is_none() {
                modifier.push_str(".fillMaxHeight()");
            }
        }
    }
    modifier_for_style_with_base(&props.style, modifier)
}

fn modifier_for_style_with_base(props: &StyleProps, modifier: String) -> String {
    modifier_for_style_with_base_and_shadow_shape(props, modifier, None)
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
            .unwrap_or("DoweDesign.onBackground");
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
                    compose_responsive_value(value, |value| format!(
                        "{}.dp",
                        value.native_units()
                    ))
                )
            })
            .unwrap_or_else(|| "0f".to_string());
        let translate_y = motion
            .translate_y
            .as_ref()
            .map(|value| {
                format!(
                    "({} ?: 0.dp).toPx()",
                    compose_responsive_value(value, |value| format!(
                        "{}.dp",
                        value.native_units()
                    ))
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

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    chars
        .next()
        .map(|first| first.to_ascii_uppercase().to_string() + chars.as_str())
        .unwrap_or_default()
}

fn compose_horizontal_arrangement(
    justify: Option<&ResponsiveValue<Justify>>,
    gap: Option<&ResponsiveValue<GapValue>>,
) -> String {
    format!(
        "doweHorizontalArrangement({}, {})",
        compose_optional_justify(justify),
        compose_optional_gap(gap)
    )
}

fn compose_vertical_arrangement(
    justify: Option<&ResponsiveValue<Justify>>,
    gap: Option<&ResponsiveValue<GapValue>>,
) -> String {
    format!(
        "doweVerticalArrangement({}, {})",
        compose_optional_justify(justify),
        compose_optional_gap(gap)
    )
}

fn compose_horizontal_alignment(value: Option<&ResponsiveValue<Align>>) -> String {
    format!("doweHorizontalAlignment({})", compose_optional_align(value))
}

fn compose_grid_horizontal_alignment(value: Option<&ResponsiveValue<GridAlignment>>) -> String {
    format!(
        "doweGridHorizontalAlignment({})",
        compose_optional_grid_alignment(value)
    )
}

fn compose_vertical_alignment(value: Option<&ResponsiveValue<Align>>) -> String {
    format!("doweVerticalAlignment({})", compose_optional_align(value))
}

fn compose_card_radius(props: &StyleProps) -> String {
    props
        .rounded
        .as_ref()
        .map(compose_rounded_value)
        .map(|value| format!("{value} ?: DoweDesign.radius"))
        .unwrap_or_else(|| "DoweDesign.radius".to_string())
}

fn compose_drawer_radius(props: &StyleProps) -> String {
    props
        .rounded
        .as_ref()
        .map(compose_rounded_value)
        .map(|value| format!("{value} ?: 0.dp"))
        .unwrap_or_else(|| "0.dp".to_string())
}

fn compose_control_radius(props: &StyleProps) -> String {
    props
        .rounded
        .as_ref()
        .map(compose_rounded_value)
        .map(|value| format!("{value} ?: DoweDesign.radius"))
        .unwrap_or_else(|| "DoweDesign.radius".to_string())
}

fn compose_button_border(props: &VariantProps) -> String {
    if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        format!("BorderStroke(1.dp, {})", variant_content(props))
    } else {
        "null".to_string()
    }
}

fn compose_card_border(props: &VariantProps) -> String {
    if props.variant.unwrap_or(ComponentVariant::Solid) == ComponentVariant::Outlined {
        format!("BorderStroke(1.dp, {})", variant_content(props))
    } else {
        "null".to_string()
    }
}

fn compose_animation_preset(value: ViewAnimation) -> &'static str {
    match value {
        ViewAnimation::None => "DoweAnimationPreset.None",
        ViewAnimation::FadeIn => "DoweAnimationPreset.FadeIn",
        ViewAnimation::SlideUp => "DoweAnimationPreset.SlideUp",
        ViewAnimation::SlideDown => "DoweAnimationPreset.SlideDown",
        ViewAnimation::SlideLeft => "DoweAnimationPreset.SlideLeft",
        ViewAnimation::SlideRight => "DoweAnimationPreset.SlideRight",
        ViewAnimation::ScaleIn => "DoweAnimationPreset.ScaleIn",
    }
}

fn compose_optional_scale(value: Option<&ResponsiveValue<ScaleValue>>) -> String {
    value
        .map(compose_scale_value)
        .unwrap_or_else(|| "null".to_string())
}

fn compose_content_padding(spacing: &dowe_components::SpacingProps) -> String {
    if spacing.p.is_none()
        && spacing.px.is_none()
        && spacing.py.is_none()
        && spacing.pl.is_none()
        && spacing.pr.is_none()
        && spacing.pt.is_none()
        && spacing.pb.is_none()
    {
        return "PaddingValues(0.dp)".to_string();
    }
    let edge = |side: Option<&ResponsiveValue<ScaleValue>>,
                axis: Option<&ResponsiveValue<ScaleValue>>| {
        [side, axis, spacing.p.as_ref()]
            .into_iter()
            .flatten()
            .map(compose_scale_value)
            .chain(std::iter::once("0.dp".to_string()))
            .collect::<Vec<_>>()
            .join(" ?: ")
    };
    format!(
        "PaddingValues(start = {}, top = {}, end = {}, bottom = {})",
        edge(spacing.pl.as_ref(), spacing.px.as_ref()),
        edge(spacing.pt.as_ref(), spacing.py.as_ref()),
        edge(spacing.pr.as_ref(), spacing.px.as_ref()),
        edge(spacing.pb.as_ref(), spacing.py.as_ref())
    )
}

fn compose_optional_rounded(value: Option<&ResponsiveValue<RoundedSize>>) -> String {
    value
        .map(compose_rounded_value)
        .unwrap_or_else(|| "null".to_string())
}

fn compose_optional_justify(value: Option<&ResponsiveValue<Justify>>) -> String {
    value
        .map(compose_justify_value)
        .unwrap_or_else(|| "null".to_string())
}

fn compose_optional_align(value: Option<&ResponsiveValue<Align>>) -> String {
    value
        .map(compose_align_value)
        .unwrap_or_else(|| "null".to_string())
}

fn compose_optional_grid_alignment(value: Option<&ResponsiveValue<GridAlignment>>) -> String {
    value
        .map(compose_grid_alignment_value)
        .unwrap_or_else(|| "null".to_string())
}

fn compose_optional_gap(value: Option<&ResponsiveValue<GapValue>>) -> String {
    value
        .map(compose_gap_value)
        .unwrap_or_else(|| "null".to_string())
}

fn compose_scale_value(value: &ResponsiveValue<ScaleValue>) -> String {
    compose_responsive_value(value, |value| format!("{}.dp", value.native_units()))
}

fn compose_gap_value(value: &ResponsiveValue<GapValue>) -> String {
    compose_responsive_value(value, compose_gap_expr)
}

fn compose_size_value(value: &ResponsiveValue<SizeValue>) -> String {
    compose_responsive_value(value, |value| match value {
        SizeValue::Scale(value) => format!("DoweSize.Fixed({}.dp)", value.native_units()),
        SizeValue::Full => "DoweSize.Full".to_string(),
        SizeValue::ViewportMinus(value) => {
            format!("DoweSize.ViewportMinus({}.dp)", value.native_units())
        }
    })
}

fn compose_color_value(value: &ResponsiveValue<ColorToken>) -> String {
    compose_responsive_value(value, |value| color_ref(*value).to_string())
}

fn compose_bool_value(value: &ResponsiveValue<bool>) -> String {
    compose_responsive_value(value, |value| value.to_string())
}
