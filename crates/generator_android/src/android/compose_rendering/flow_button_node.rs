fn render_compose_button_flow_node(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    _flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) -> bool {
    if !matches!(node, ViewNode::Button { .. }) {
        return false;
    }
    let pad = " ".repeat(indent);
    match node {
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
            let icon_condition =
                |path: &str, comparison: Option<&dowe_components::ReactiveNumberComparison>| {
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
            let action = if let Some(path) = props.swap_bind.as_deref() {
                let bind = escape_kotlin(&context.signal_path(path));
                let body = action
                    .strip_prefix("{ ")
                    .and_then(|value| value.strip_suffix(" }"))
                    .unwrap_or("");
                format!("{{ state.write(\"{bind}\", !state.bool(\"{bind}\")); {body} }}")
            } else {
                action
            };
            let loading = props
                .reactive
                .loading
                .as_ref()
                .map(|path| reactive_bool(path));
            let disabled = props
                .reactive
                .disabled
                .as_ref()
                .map(|path| reactive_bool(path));
            let variant = props
                .reactive
                .variant
                .as_ref()
                .map(|path| reactive_text(path, "solid"));
            let scheme = props
                .reactive
                .scheme
                .as_ref()
                .map(|path| reactive_text(path, "primary"));
            let size = props
                .reactive
                .size
                .as_ref()
                .map(|path| reactive_text(path, "md"));
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
            let radius = props
                .reactive
                .rounded
                .as_ref()
                .map(|path| format!("doweButtonRadius({})", reactive_text(path, "md")))
                .unwrap_or_else(|| compose_control_radius(&props.style));
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
                format!(
                    "if ({variant_value} == \"outlined\") BorderStroke(1.dp, {content}) else null"
                )
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
            let mut modifier = format!("{button_modifier}.doweGridCompactWidth()");
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
            let min_height = size
                .as_ref()
                .map(|size| format!("doweButtonMinHeight({size})"))
                .or_else(|| {
                    props
                        .size
                        .map(|size| format!("doweButtonMinHeight(\"{}\")", size.as_str()))
                })
                .unwrap_or_else(|| "0.dp".to_string());
            if props.icon_only && (size.is_some() || props.size.is_some()) {
                modifier.push_str(&format!(".width({min_height})"));
            }
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
                        "{pad}Button(modifier = {}.height({min_height}), shape = RoundedCornerShape({}), colors = {colors}, border = {}, contentPadding = {content_padding}, enabled = {enabled}, onClick = {}) {{\n",
                        modifier,
                        radius,
                        border,
                        action
                    ));
            let render_contents = |content_indent: usize, output: &mut String| {
                let content_pad = " ".repeat(content_indent);
                if let Some(path) = props.swap_bind.as_deref() {
                    output.push_str(&format!(
                        "{content_pad}if (state.bool(\"{}\")) {{\n",
                        escape_kotlin(&context.signal_path(path))
                    ));
                    if let Some(icon) = props.icon_start.as_ref() {
                        render_compose_side_icon(icon, content_indent + 4, output);
                    }
                    output.push_str(&format!("{content_pad}}} else {{\n"));
                    if let Some(icon) = props.swap_icon_off.as_ref() {
                        render_compose_side_icon(icon, content_indent + 4, output);
                    }
                    output.push_str(&format!("{content_pad}}}\n"));
                } else if let Some(icon) = props.icon_start.as_ref() {
                    if let Some(path) = props.reactive.icon_start_when.as_ref() {
                        output.push_str(&format!(
                            "{content_pad}if ({}) {{\n",
                            icon_condition(path, props.reactive.icon_start_comparison.as_ref())
                        ));
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
                        output.push_str(&format!(
                            "{content_pad}if ({}) {{\n",
                            icon_condition(path, props.reactive.icon_end_comparison.as_ref())
                        ));
                        render_compose_side_icon(icon, content_indent + 4, output);
                        output.push_str(&format!("{content_pad}}}\n"));
                    } else {
                        render_compose_side_icon(icon, content_indent, output);
                    }
                }
            };
            if let Some(loading) = loading.as_ref() {
                output.push_str(&format!(
                    "{pad}    Box(contentAlignment = Alignment.Center) {{\n"
                ));
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
    true
}
