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
    let modifier = if props.align.is_some() && props.style.sizing.w.is_none() {
        modifier_for_style_with_base(&props.style, "Modifier.fillMaxWidth()".to_string())
    } else {
        modifier_for_style(&props.style)
    };
    let text_align = props
        .align
        .as_ref()
        .map(|value| format!(", textAlign = {}", compose_text_align(value)))
        .unwrap_or_default();
    output.push_str(&format!(
        "{pad}Text({}, modifier = {modifier}, color = {}, fontSize = {size}, lineHeight = {}, fontFamily = {}, fontWeight = {}{spacing}{text_align})\n",
        compose_visible_text_expression(value, props.i18n.as_deref(), context),
        text_color(title, props),
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
        return format!(
            "stringResource(R.string.{})",
            translation_resource_name(key)
        );
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
        return format!(
            "stringResource(R.string.{})",
            translation_resource_name(key)
        );
    }
    let segments = text_template_segments(value);
    if segments.iter().all(|(_, binding)| binding.is_none()) {
        return format!("\"{}\"", escape_kotlin(value));
    }
    segments
        .into_iter()
        .map(|(literal, binding)| match binding {
            Some(binding) => match context.dynamic_path(&binding) {
                Some(path) => context
                    .item_value(&binding)
                    .map(|item| format!("state.text(\"{}\", {item})", escape_kotlin(&path)))
                    .unwrap_or_else(|| format!("state.text(\"{}\")", escape_kotlin(&path))),
                None => format!("\"{{{}}}\"", escape_kotlin(&binding)),
            },
            None => format!("\"{}\"", escape_kotlin(&literal)),
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

fn compose_localized_literal(value: &str, i18n: Option<&str>) -> String {
    i18n.map(|key| {
        format!(
            "stringResource(R.string.{})",
            translation_resource_name(key)
        )
    })
    .unwrap_or_else(|| format!("\"{}\"", escape_kotlin(value)))
}

fn modifier_for_layout(props: &LayoutProps, flow: ComposeFlow) -> String {
    modifier_for_container_style(&props.style, flow)
}

fn compose_grid_has_full_height(props: &GridProps) -> bool {
    props.style.sizing.h.as_ref().is_some_and(|value| {
        value
            .entries
            .iter()
            .any(|entry| entry.value == SizeValue::Full)
    })
}

fn compose_grid_fills_height(props: &GridProps) -> String {
    props
        .style
        .sizing
        .h
        .as_ref()
        .map(|value| {
            format!(
                "({} ?: false)",
                compose_responsive_value(value, |value| matches!(value, SizeValue::Full)
                    .to_string())
            )
        })
        .unwrap_or_else(|| "false".to_string())
}

fn compose_grid_vertical_stretch(value: Option<&ResponsiveValue<GridAlignment>>) -> String {
    value
        .map(|value| {
            format!(
                "{} ?: true",
                compose_responsive_value(value, |value| matches!(value, GridAlignment::Stretch)
                    .to_string())
            )
        })
        .unwrap_or_else(|| "true".to_string())
}

fn modifier_for_grid(props: &GridProps, flow: ComposeFlow) -> String {
    let mut modifier = String::from("Modifier");
    if flow.is_block() && props.style.sizing.w.is_none() {
        modifier.push_str(".fillMaxWidth()");
    }
    if flow == ComposeFlow::Grid && props.style.sizing.h.is_none() {
        modifier.push_str(".fillMaxHeight()");
    }
    if flow.is_block() && compose_grid_has_full_height(props) {
        modifier.push_str(&format!(
            ".then(if {} {{ Modifier.weight(1f, fill = true) }} else {{ Modifier }})",
            compose_grid_fills_height(props)
        ));
    }
    let mut modifier = modifier_for_style_with_base_and_shadow_shape(
        &props.style,
        modifier,
        Some("RoundedCornerShape(0.dp)"),
    );
    append_compose_flex_item_modifier(&mut modifier, props.style.flex.as_ref(), flow);
    modifier
}

fn compose_grid_tracks(value: Option<&ResponsiveValue<GridTracks>>) -> String {
    value
        .map(|value| {
            format!(
                "{} ?: listOf(1f)",
                compose_responsive_value(value, |value| match value {
                    GridTracks::Count(count) =>
                        format!("listOf({})", vec!["1f"; *count as usize].join(", ")),
                    GridTracks::Fractions(weights) => format!(
                        "listOf({})",
                        weights
                            .iter()
                            .map(|weight| format!("{weight}f"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    GridTracks::Auto => "listOf(1f)".to_string(),
                })
            )
        })
        .unwrap_or_else(|| "listOf(1f)".to_string())
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

fn compose_svg_modifier(props: &SvgProps) -> String {
    let mut modifier = modifier_for_style(&props.style);
    if props.data.is_none()
        && props.icon_name.is_none()
        && props.motion.is_none()
        && props.style.sizing.w.is_some() != props.style.sizing.h.is_some()
        && let Some(ratio) = props.view_box.aspect_ratio()
    {
        modifier.push_str(&format!(
            ".aspectRatio({ratio:.6}f, matchHeightConstraintsFirst = true)"
        ));
    }
    modifier
}

fn modifier_for_style_with_shadow_shape(props: &StyleProps, shadow_shape: &str) -> String {
    modifier_for_style_with_base_and_shadow_shape(props, "Modifier".to_string(), Some(shadow_shape))
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
    if flow.is_block() && props.sizing.w.is_none() {
        modifier.push_str(".fillMaxWidth()");
    }
    let mut modifier = modifier_for_style_with_base(props, modifier);
    if (flow == ComposeFlow::Grid || props.center_y.is_some()) && props.sizing.h.is_none() {
        modifier.push_str(".fillMaxHeight()");
    }
    append_compose_flex_item_modifier(&mut modifier, props.flex.as_ref(), flow);
    modifier
}

fn append_compose_flex_item_modifier(
    modifier: &mut String,
    value: Option<&ResponsiveValue<FlexItem>>,
    flow: ComposeFlow,
) {
    if !flow.is_flex_item() {
        return;
    }
    let Some(value) = value else {
        return;
    };
    let item_modifier = compose_responsive_value(value, |value| match value {
        FlexItem::Initial | FlexItem::None => "Modifier".to_string(),
        FlexItem::Auto => "Modifier.weight(1f, fill = false)".to_string(),
        FlexItem::Fill => "Modifier.weight(1f, fill = true)".to_string(),
    });
    modifier.push_str(&format!(".then({item_modifier} ?: Modifier)"));
}

