fn render_compose_flow_node(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    match node {
        ViewNode::Splash {
            binding,
            content,
            children,
            ..
        } => {
            output.push_str(&format!(
                "{pad}if (state.bool(\"{}\")) {{\n",
                escape_kotlin(&context.signal_path(binding))
            ));
            for child in children {
                render_compose_node_in_flow(
                    child,
                    indent + 4,
                    output,
                    flow,
                    inherited_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}}} else {{\n"));
            for child in content {
                render_compose_node_in_flow(
                    child,
                    indent + 4,
                    output,
                    flow,
                    inherited_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
        }
        ViewNode::Scope {
            constants,
            signals,
            actions,
            children,
        } => {
            let context = context.with_scope(constants, signals, actions);
            for child in children {
                render_compose_node_in_flow(
                    child,
                    indent,
                    output,
                    flow,
                    inherited_font,
                    default_family,
                    &context,
                );
            }
        }
        ViewNode::Each {
            item,
            collection,
            children,
            ..
        } => {
            output.push_str(&format!(
                "{pad}state.rows(\"{}\").forEach {{ row ->\n",
                escape_kotlin(&context.signal_path(collection))
            ));
            let context = context.with_item(item, "row.value".to_string());
            for child in children {
                render_compose_node_in_flow(
                    child,
                    indent + 4,
                    output,
                    flow,
                    inherited_font,
                    default_family,
                    &context,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
        }
        ViewNode::Box { props, children } => {
            if props.position().mode != BoxPosition::Fixed {
                render_compose_box(
                    props,
                    children,
                    indent,
                    output,
                    flow,
                    inherited_font,
                    default_family,
                    context,
                    false,
                );
            }
        }
        ViewNode::Section { props, children } => {
            let current_font = props.font.as_ref().or(inherited_font);
            if props.cover.is_some() {
                output.push_str(&format!(
                    "{pad}DoweCoverBox(modifier = {}, source = {}, overlay = {}) {{\n",
                    modifier_for_section_container(props, flow),
                    compose_cover_value(props.cover.as_ref().expect("cover")),
                    compose_optional_overlay(props.overlay.as_ref())
                ));
                render_compose_section_body(
                    props,
                    children,
                    indent + 4,
                    output,
                    current_font,
                    default_family,
                    context,
                );
                output.push_str(&format!("{pad}}}\n"));
            } else if let Some(background) = props.background.as_ref() {
                output.push_str(&format!(
                    "{pad}DoweSectionBackgroundBox(modifier = {}, background = {}) {{\n",
                    modifier_for_section_container(props, flow),
                    compose_section_background_value(background)
                ));
                render_compose_section_body(
                    props,
                    children,
                    indent + 4,
                    output,
                    current_font,
                    default_family,
                    context,
                );
                output.push_str(&format!("{pad}}}\n"));
            } else {
                output.push_str(&format!(
                    "{pad}Column(modifier = {}) {{\n",
                    modifier_for_section_container(props, flow)
                ));
                render_compose_section_body(
                    props,
                    children,
                    indent + 4,
                    output,
                    current_font,
                    default_family,
                    context,
                );
                output.push_str(&format!("{pad}}}\n"));
            }
        }
        ViewNode::Flex { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            output.push_str(&format!(
                "{pad}if ({} == DoweFlexDirection.Column) {{\n",
                compose_flex_direction_value(&props.direction)
            ));
            output.push_str(&format!(
                "{pad}    Column(modifier = {}, verticalArrangement = {}, horizontalAlignment = {}) {{\n",
                modifier_for_layout(props, flow),
                compose_vertical_arrangement(props.justify.as_ref(), props.gap.as_ref()),
                compose_horizontal_alignment(props.align.as_ref())
            ));
            let color_scope = compose_content_color(&props.style);
            if let Some(color) = color_scope.as_ref() {
                output.push_str(&format!(
                            "{pad}        CompositionLocalProvider(LocalContentColor provides ({color} ?: LocalContentColor.current)) {{\n"
                        ));
            }
            for child in children {
                render_compose_node_in_flow(
                    child,
                    indent + if color_scope.is_some() { 12 } else { 8 },
                    output,
                    ComposeFlow::Block,
                    current_font,
                    default_family,
                    context,
                );
            }
            if color_scope.is_some() {
                output.push_str(&format!("{pad}        }}\n"));
            }
            output.push_str(&format!("{pad}    }}\n"));
            output.push_str(&format!("{pad}}} else {{\n"));
            if props.wrap {
                output.push_str(&format!(
                    "{pad}    FlowRow(modifier = {}, horizontalArrangement = {}, verticalArrangement = Arrangement.spacedBy({}), itemVerticalAlignment = {}) {{\n",
                    modifier_for_layout(props, flow),
                    compose_horizontal_arrangement(props.justify.as_ref(), props.gap.as_ref()),
                    compose_grid_vertical_gap(props.gap.as_ref()),
                    compose_vertical_alignment(props.align.as_ref())
                ));
            } else {
                output.push_str(&format!(
                    "{pad}    Row(modifier = {}, horizontalArrangement = {}, verticalAlignment = {}) {{\n",
                    modifier_for_layout(props, flow),
                    compose_horizontal_arrangement(props.justify.as_ref(), props.gap.as_ref()),
                    compose_vertical_alignment(props.align.as_ref())
                ));
            }
            if let Some(color) = color_scope.as_ref() {
                output.push_str(&format!(
                    "{pad}        CompositionLocalProvider(LocalContentColor provides ({color} ?: LocalContentColor.current)) {{\n"
                ));
            }
            for child in children {
                render_compose_node_in_flow(
                    child,
                    indent + if color_scope.is_some() { 12 } else { 8 },
                    output,
                    ComposeFlow::Inline,
                    current_font,
                    default_family,
                    context,
                );
            }
            if color_scope.is_some() {
                output.push_str(&format!("{pad}        }}\n"));
            }
            output.push_str(&format!("{pad}    }}\n"));
            output.push_str(&format!("{pad}}}\n"));
        }
        ViewNode::Grid { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            output.push_str(&format!(
                        "{pad}DoweGrid(modifier = {}, columns = {}, horizontalGap = {}, verticalGap = {}, horizontalAlignment = {}) {{\n",
                        modifier_for_grid(props, flow),
                        compose_grid_column_count(props.columns.as_ref()),
                        compose_grid_horizontal_gap(props.gap.as_ref()),
                        compose_grid_vertical_gap(props.gap.as_ref()),
                        compose_grid_horizontal_alignment(props.justify.as_ref())
                    ));
            let color_scope = compose_content_color(&props.style);
            if let Some(color) = color_scope.as_ref() {
                output.push_str(&format!(
                            "{pad}    CompositionLocalProvider(LocalContentColor provides ({color} ?: LocalContentColor.current)) {{\n"
                        ));
            }
            for child in children {
                render_compose_node_in_flow(
                    child,
                    indent + if color_scope.is_some() { 8 } else { 4 },
                    output,
                    ComposeFlow::Block,
                    current_font,
                    default_family,
                    context,
                );
            }
            if color_scope.is_some() {
                output.push_str(&format!("{pad}    }}\n"));
            }
            output.push_str(&format!("{pad}}}\n"));
        }
        ViewNode::Card { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            let mut modifier = modifier_for_container_style(&props.style, flow);
            if props.style.element.on_click.is_some() {
                modifier.push_str(&format!(
                    ".clickable(onClick = {})",
                    compose_component_action(props.style.element.on_click.as_deref(), None, context)
                ));
            }
            output.push_str(&format!(
                        "{pad}Card(modifier = {}, shape = RoundedCornerShape({}), colors = CardDefaults.cardColors(containerColor = {}, contentColor = {}), border = {}, elevation = {}) {{\n",
                        modifier,
                        compose_card_radius(&props.style),
                        card_variant_container(props),
                        card_variant_content(props),
                        compose_card_border(props),
                        "CardDefaults.cardElevation(defaultElevation = 0.dp)"
                    ));
            output.push_str(&format!(
                "{pad}    CompositionLocalProvider(LocalDoweTitleColor provides {}) {{\n",
                card_variant_title(props)
            ));
            if props.style.cover.is_some() {
                output.push_str(&format!(
                            "{pad}    DoweCoverBox(modifier = Modifier.fillMaxWidth(), source = {}, overlay = {}) {{\n",
                            compose_cover_value(props.style.cover.as_ref().expect("cover")),
                            compose_optional_overlay(props.style.overlay.as_ref())
                        ));
                output.push_str(&format!("{pad}        Column {{\n"));
                for child in children {
                    render_compose_node_in_flow(
                        child,
                        indent + 12,
                        output,
                        ComposeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
                output.push_str(&format!("{pad}        }}\n"));
                output.push_str(&format!("{pad}    }}\n"));
            } else {
                for child in children {
                    render_compose_node_in_flow(
                        child,
                        indent + 4,
                        output,
                        ComposeFlow::Block,
                        current_font,
                        default_family,
                        context,
                    );
                }
            }
            output.push_str(&format!("{pad}    }}\n{pad}}}\n"));
        }
        ViewNode::Brand { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            let mut modifier = modifier_for_container_style(&props.style, ComposeFlow::Inline);
            if props.navigation.is_some() {
                modifier.push_str(&format!(
                    ".clickable(onClick = {})",
                    compose_navigation_action(props.navigation.as_ref())
                ));
            }
            if let Some(label) = props.label.as_deref() {
                modifier.push_str(&format!(
                    ".semantics {{ contentDescription = \"{}\" }}",
                    escape_kotlin(label)
                ));
            }
            output.push_str(&format!(
                "{pad}Row(modifier = {modifier}, verticalAlignment = Alignment.CenterVertically) {{\n"
            ));
            render_compose_scoped_children(
                &props.style,
                children,
                indent + 4,
                output,
                ComposeFlow::Inline,
                current_font,
                default_family,
                context,
            );
            output.push_str(&format!("{pad}}}\n"));
        }
        ViewNode::Banner { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            let mut modifier = modifier_for_container_style(&props.style, ComposeFlow::Block);
            modifier.push_str(&format!(
                ".clickable(onClick = {})",
                compose_navigation_action(Some(&props.navigation))
            ));
            if let Some(label) = props.label.as_deref() {
                modifier.push_str(&format!(
                    ".semantics {{ contentDescription = \"{}\" }}",
                    escape_kotlin(label)
                ));
            }
            if props.style.cover.is_some() {
                output.push_str(&format!(
                    "{pad}DoweCoverBox(modifier = {modifier}, source = {}, overlay = {}) {{\n",
                    compose_cover_value(props.style.cover.as_ref().expect("cover")),
                    compose_optional_overlay(props.style.overlay.as_ref())
                ));
                output.push_str(&format!(
                    "{pad}    Column(verticalArrangement = Arrangement.spacedBy(0.dp)) {{\n"
                ));
                render_compose_scoped_children(
                    &props.style,
                    children,
                    indent + 8,
                    output,
                    ComposeFlow::Block,
                    current_font,
                    default_family,
                    context,
                );
                output.push_str(&format!("{pad}    }}\n"));
                output.push_str(&format!("{pad}}}\n"));
            } else {
                output.push_str(&format!(
                    "{pad}Column(modifier = {modifier}, verticalArrangement = Arrangement.spacedBy(0.dp)) {{\n"
                ));
                render_compose_scoped_children(
                    &props.style,
                    children,
                    indent + 4,
                    output,
                    ComposeFlow::Block,
                    current_font,
                    default_family,
                    context,
                );
                output.push_str(&format!("{pad}}}\n"));
            }
        }
        ViewNode::Button { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            let reactive_text = |path: &str, fallback: &str| {
                context
                    .item_value(path)
                    .map(|item| {
                        format!(
                            "state.text(\"{}\", {item})",
                            escape_kotlin(&context.item_path(path).expect("item path"))
                        )
                    })
                    .unwrap_or_else(|| {
                        format!(
                            "state.text(\"{}\", \"{fallback}\")",
                            escape_kotlin(&context.signal_path(path))
                        )
                    })
            };
            let reactive_bool = |path: &str| {
                context
                    .item_value(path)
                    .map(|item| {
                        format!(
                            "state.bool(\"{}\", {item})",
                            escape_kotlin(&context.item_path(path).expect("item path"))
                        )
                    })
                    .unwrap_or_else(|| {
                        format!(
                            "state.bool(\"{}\", true)",
                            escape_kotlin(&context.signal_path(path))
                        )
                    })
            };
            let icon_condition = |path: &str,
                                  comparison: Option<&dowe_components::ReactiveNumberComparison>| {
                comparison
                    .map(|comparison| {
                        format!(
                            "(({}).toDoubleOrNull() ?: 0.0) {} {}",
                            reactive_text(path, "0"),
                            comparison.operator.as_str(),
                            comparison.value
                        )
                    })
                    .unwrap_or_else(|| reactive_bool(path))
            };
            let action = props
                .element
                .on_click
                .as_deref()
                .and_then(|name| context.action_id(name))
                .map(|id| {
                    let item = context
                        .active_item()
                        .map(|value| format!(", {value}"))
                        .unwrap_or_default();
                    format!(
                        "{{ actionScope.launch {{ state.run(\"{}\"{item}) }} }}",
                        escape_kotlin(id)
                    )
                })
                .unwrap_or_else(|| compose_navigation_action(props.navigation.as_ref()));
            let loading = props.reactive.loading.as_ref().map(|path| reactive_bool(path));
            let disabled = props.reactive.disabled.as_ref().map(|path| reactive_bool(path));
            let variant = props.reactive.variant.as_ref().map(|path| reactive_text(path, "solid"));
            let scheme = props.reactive.scheme.as_ref().map(|path| reactive_text(path, "primary"));
            let size = props.reactive.size.as_ref().map(|path| reactive_text(path, "md"));
            let variant_value = variant.clone().unwrap_or_else(|| {
                format!(
                    "\"{}\"",
                    props.variant.unwrap_or(ComponentVariant::Solid).as_str()
                )
            });
            let scheme_value = scheme.clone().unwrap_or_else(|| {
                format!(
                    "\"{}\"",
                    props.color.unwrap_or(ColorFamily::Primary).as_str()
                )
            });
            let reactive_visual = variant.is_some() || scheme.is_some();
            let radius = props.reactive.rounded.as_ref().map(|path| format!("doweButtonRadius({})", reactive_text(path, "md"))).unwrap_or_else(|| compose_control_radius(&props.style));
            let container = if reactive_visual {
                format!("doweButtonContainer({variant_value}, {scheme_value})")
            } else {
                variant_container(props).to_string()
            };
            let content = if reactive_visual {
                format!("doweButtonContent({variant_value}, {scheme_value})")
            } else {
                variant_content(props).to_string()
            };
            let border = if reactive_visual {
                format!("if ({variant_value} == \"outlined\") BorderStroke(1.dp, {content}) else null")
            } else {
                compose_button_border(props)
            };
            let mut button_style = props.style.clone();
            button_style.spacing = Default::default();
            if props.reactive.rounded.is_some() {
                button_style.rounded = None;
            }
            let button_modifier = modifier_for_style_with_shadow_shape(
                &button_style,
                &format!("RoundedCornerShape({radius})"),
            );
            let mut modifier = button_modifier;
            if props.icon_only {
                modifier.push_str(&format!(
                    ".semantics {{ contentDescription = \"{}\" }}",
                    escape_kotlin(props.label.as_deref().unwrap_or_default())
                ));
            }
            let content_padding = size
                .as_ref()
                .map(|size| format!("PaddingValues(horizontal = doweButtonHorizontalPadding({size}), vertical = doweButtonVerticalPadding({size}))"))
                .unwrap_or_else(|| compose_content_padding(&props.style.spacing));
            let min_height = size.as_ref().map(|size| format!("doweButtonMinHeight({size})")).unwrap_or_else(|| "0.dp".to_string());
            let enabled = if loading.is_some() && disabled.is_some() {
                let loading_value = loading.as_deref().unwrap_or("false");
                let disabled_value = disabled.as_deref().unwrap_or("false");
                format!("!(({loading_value}) || ({disabled_value}))")
            } else if let Some(loading) = loading.as_deref() {
                format!("!({loading})")
            } else if let Some(disabled) = disabled.as_deref() {
                format!("!({disabled})")
            } else {
                "true".to_string()
            };
            let has_disabled_state = loading.is_some() || disabled.is_some();
            if has_disabled_state {
                modifier.push_str(&format!(
                    ".graphicsLayer {{ alpha = if ({enabled}) 1f else 0.5f }}"
                ));
            }
            let colors = if has_disabled_state {
                format!(
                    "ButtonDefaults.buttonColors(containerColor = {container}, contentColor = {content}, disabledContainerColor = {container}, disabledContentColor = {content})"
                )
            } else {
                format!(
                    "ButtonDefaults.buttonColors(containerColor = {container}, contentColor = {content})"
                )
            };
            output.push_str(&format!(
                        "{pad}Button(modifier = {}.defaultMinSize(minWidth = 0.dp, minHeight = {min_height}), shape = RoundedCornerShape({}), colors = {colors}, border = {}, contentPadding = {content_padding}, enabled = {enabled}, onClick = {}) {{\n",
                        modifier,
                        radius,
                        border,
                        action
                    ));
            let render_contents = |content_indent: usize, output: &mut String| {
                let content_pad = " ".repeat(content_indent);
                if let Some(icon) = props.icon_start.as_ref() {
                    if let Some(path) = props.reactive.icon_start_when.as_ref() {
                        output.push_str(&format!("{content_pad}if ({}) {{\n", icon_condition(path, props.reactive.icon_start_comparison.as_ref())));
                        render_compose_side_icon(icon, content_indent + 4, output);
                        output.push_str(&format!("{content_pad}}}\n"));
                    } else {
                        render_compose_side_icon(icon, content_indent, output);
                    }
                }
                for child in children {
                    render_compose_node_in_flow(
                        child,
                        content_indent,
                        output,
                        ComposeFlow::Inline,
                        current_font,
                        default_family,
                        context,
                    );
                }
                if let Some(icon) = props.icon_end.as_ref() {
                    if let Some(path) = props.reactive.icon_end_when.as_ref() {
                        output.push_str(&format!("{content_pad}if ({}) {{\n", icon_condition(path, props.reactive.icon_end_comparison.as_ref())));
                        render_compose_side_icon(icon, content_indent + 4, output);
                        output.push_str(&format!("{content_pad}}}\n"));
                    } else {
                        render_compose_side_icon(icon, content_indent, output);
                    }
                }
            };
            if let Some(loading) = loading.as_ref() {
                output.push_str(&format!("{pad}    Box(contentAlignment = Alignment.Center) {{\n"));
                output.push_str(&format!(
                    "{pad}        Row(modifier = Modifier.graphicsLayer {{ alpha = if ({loading}) 0f else 1f }}, verticalAlignment = Alignment.CenterVertically) {{\n"
                ));
                render_contents(indent + 12, output);
                output.push_str(&format!("{pad}        }}\n"));
                output.push_str(&format!("{pad}        if ({loading}) {{\n"));
                if let Some(icon) = props.loading_icon.as_ref() {
                    render_compose_button_spinner(icon, indent + 12, output);
                }
                output.push_str(&format!("{pad}        }}\n"));
                output.push_str(&format!("{pad}    }}\n"));
            } else {
                render_contents(indent + 4, output);
            }
            output.push_str(&format!("{pad}}}\n"));
        }
        _ => {}
    }
}

fn render_compose_section_body(
    props: &StyleProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let column_indent = if props.boxed { indent + 4 } else { indent };
    if props.boxed {
        output.push_str(&format!(
            "{pad}Box(modifier = Modifier.fillMaxWidth(), contentAlignment = Alignment.TopCenter) {{\n"
        ));
    }
    let column_pad = " ".repeat(column_indent);
    let horizontal_alignment = props
        .center
        .as_ref()
        .map(|value| {
            format!(
                ", horizontalAlignment = {}",
                compose_section_horizontal_alignment(value)
            )
        })
        .unwrap_or_default();
    let vertical_arrangement = compose_section_vertical_arrangement(props.gap.as_ref());
    output.push_str(&format!(
        "{column_pad}Column(modifier = {}{}{}) {{\n",
        modifier_for_section_content(props),
        vertical_arrangement,
        horizontal_alignment
    ));
    let color_scope = compose_content_color(props);
    if let Some(color) = color_scope.as_ref() {
        output.push_str(&format!(
            "{column_pad}    CompositionLocalProvider(LocalContentColor provides ({color} ?: LocalContentColor.current)) {{\n"
        ));
    }
    for child in children {
        render_compose_node_in_flow(
            child,
            column_indent + if color_scope.is_some() { 8 } else { 4 },
            output,
            ComposeFlow::Block,
            inherited_font,
            default_family,
            context,
        );
    }
    if color_scope.is_some() {
        output.push_str(&format!("{column_pad}    }}\n"));
    }
    output.push_str(&format!("{column_pad}}}\n"));
    if props.boxed {
        output.push_str(&format!("{pad}}}\n"));
    }
}

fn render_compose_fixed_box(
    props: &StyleProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    render_compose_box(
        props,
        children,
        indent,
        output,
        ComposeFlow::Inline,
        inherited_font,
        default_family,
        context,
        true,
    );
}

fn render_compose_box(
    props: &StyleProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
    render_fixed: bool,
) {
    let pad = " ".repeat(indent);
    let current_font = props.font.as_ref().or(inherited_font);
    let position = props.position();
    let positioned = position.mode == BoxPosition::Absolute
        || position.mode == BoxPosition::Fixed && render_fixed;
    let has_absolute_children = position.mode == BoxPosition::Relative
        && children.iter().any(|child| {
            matches!(child, ViewNode::Box { props, .. } if props.position().mode == BoxPosition::Absolute)
        });
    let mut modifier = modifier_for_container_style(
        props,
        if positioned { ComposeFlow::Inline } else { flow },
    );
    if props.element.on_click.is_some() {
        modifier.push_str(&format!(
            ".clickable(onClick = {})",
            compose_component_action(props.element.on_click.as_deref(), None, context)
        ));
    }
    if positioned {
        modifier.push_str(&compose_position_modifier(position));
    }

    if props.cover.is_some() {
        output.push_str(&format!(
            "{pad}DoweCoverBox(modifier = {modifier}, source = {}, overlay = {}) {{\n",
            compose_cover_value(props.cover.as_ref().expect("cover")),
            compose_optional_overlay(props.overlay.as_ref())
        ));
        render_compose_box_children(
            props,
            children,
            indent + 4,
            output,
            current_font,
            default_family,
            context,
            has_absolute_children || positioned,
        );
        output.push_str(&format!("{pad}}}\n"));
    } else if has_absolute_children || positioned {
        output.push_str(&format!("{pad}Box(modifier = {modifier}) {{\n"));
        render_compose_box_children(
            props,
            children,
            indent + 4,
            output,
            current_font,
            default_family,
            context,
            true,
        );
        output.push_str(&format!("{pad}}}\n"));
    } else {
        output.push_str(&format!("{pad}Column(modifier = {modifier}) {{\n"));
        render_compose_box_children(
            props,
            children,
            indent + 4,
            output,
            current_font,
            default_family,
            context,
            false,
        );
        output.push_str(&format!("{pad}}}\n"));
    }
}

fn render_compose_box_children(
    props: &StyleProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
    layered: bool,
) {
    let pad = " ".repeat(indent);
    let color_scope = compose_content_color(props);
    if let Some(color) = color_scope.as_ref() {
        output.push_str(&format!(
            "{pad}CompositionLocalProvider(LocalContentColor provides ({color} ?: LocalContentColor.current)) {{\n"
        ));
    }
    let child_indent = indent + if color_scope.is_some() { 4 } else { 0 };
    let child_pad = " ".repeat(child_indent);
    if layered {
        output.push_str(&format!("{child_pad}Column {{\n"));
        render_compose_box_flow_children(
            children,
            child_indent + 4,
            output,
            inherited_font,
            default_family,
            context,
        );
        output.push_str(&format!("{child_pad}}}\n"));
        for child in children.iter().filter(|child| {
            matches!(child, ViewNode::Box { props, .. } if props.position().mode == BoxPosition::Absolute)
        }) {
            render_compose_node_in_flow(
                child,
                child_indent,
                output,
                ComposeFlow::Inline,
                inherited_font,
                default_family,
                context,
            );
        }
    } else {
        render_compose_box_flow_children(
            children,
            child_indent,
            output,
            inherited_font,
            default_family,
            context,
        );
    }
    if color_scope.is_some() {
        output.push_str(&format!("{pad}}}\n"));
    }
}

fn render_compose_box_flow_children(
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    for child in children.iter().filter(|child| {
        !matches!(child, ViewNode::Box { props, .. } if matches!(props.position().mode, BoxPosition::Absolute | BoxPosition::Fixed))
    }) {
        render_compose_node_in_flow(
            child,
            indent,
            output,
            ComposeFlow::Block,
            inherited_font,
            default_family,
            context,
        );
    }
}

fn render_compose_scoped_children(
    props: &StyleProps,
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let color_scope = compose_content_color(props);
    if let Some(color) = color_scope.as_ref() {
        output.push_str(&format!(
            "{pad}CompositionLocalProvider(LocalContentColor provides ({color} ?: LocalContentColor.current)) {{\n"
        ));
    }
    let child_indent = indent + if color_scope.is_some() { 4 } else { 0 };
    for child in children {
        render_compose_node_in_flow(
            child,
            child_indent,
            output,
            flow,
            inherited_font,
            default_family,
            context,
        );
    }
    if color_scope.is_some() {
        output.push_str(&format!("{pad}}}\n"));
    }
}
