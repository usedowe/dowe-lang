fn render_compose_theme_toggle(props: &ThemeToggleProps, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweThemeToggle(modifier = {}, backgroundColor = {}, contentColor = {}, borderColor = {}, lightIconViewBox = {}, lightIconPaths = {}, lightIconModifier = {}, darkIconViewBox = {}, darkIconPaths = {}, darkIconModifier = {})\n",
        modifier_for_style(&props.style.style),
        variant_container(&props.style),
        variant_content(&props.style),
        compose_variant_border(&props.style),
        compose_svg_view_box(&props.light_icon.props.view_box),
        compose_svg_paths(&props.light_icon.paths),
        modifier_for_style(&props.light_icon.props.style),
        compose_svg_view_box(&props.dark_icon.props.view_box),
        compose_svg_paths(&props.dark_icon.paths),
        modifier_for_style(&props.dark_icon.props.style),
    ));
}

fn render_compose_theme_select(
    props: &ThemeSelectProps,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
) {
    let pad = " ".repeat(indent);
    let border = if props.style.variant.unwrap_or(ComponentVariant::Outlined)
        == ComponentVariant::Outlined
    {
        card_variant_content(&props.style)
    } else {
        "null"
    };
    let modifier = if flow == ComposeFlow::Inline && props.style.style.sizing.w.is_none() {
        format!("{}.weight(1f)", modifier_for_style(&props.style.style))
    } else {
        modifier_for_style(&props.style.style)
    };
    output.push_str(&format!(
        "{pad}DoweThemeSelect(modifier = {}, label = {}, placeholder = {}, backgroundColor = {}, contentColor = {}, borderColor = {border})\n",
        modifier,
        compose_string_literal(&props.label),
        compose_string_literal(&props.placeholder),
        card_variant_container(&props.style),
        card_variant_content(&props.style),
    ));
}

fn render_compose_fab(
    props: &FabProps,
    actions: &[FabAction],
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
    open_state: Option<&str>,
) {
    let pad = " ".repeat(indent);
    if props.fixed && open_state.is_none() {
        return;
    }
    let modifier = if props.fixed {
        format!(
            "Modifier.fillMaxSize().padding(horizontal = {}, vertical = {})",
            compose_scale_literal(props.offset_x),
            compose_scale_literal(props.offset_y)
        )
    } else {
        modifier_for_style(&props.style.style)
    };
    output.push_str(&format!(
        "{pad}Column(modifier = {modifier}, horizontalAlignment = {}, verticalArrangement = Arrangement.spacedBy(12.dp, alignment = {})) {{\n",
        compose_fab_horizontal_alignment(props.position),
        compose_fab_vertical_arrangement(props.position)
    ));
    let top = matches!(
        props.position,
        OverlayCornerPosition::TopLeft | OverlayCornerPosition::TopRight
    );
    if top {
        render_compose_fab_trigger(props, actions, indent + 4, output, context, open_state);
    }
    render_compose_fab_actions(props, actions, indent + 4, output, context, open_state);
    if !top {
        render_compose_fab_trigger(props, actions, indent + 4, output, context, open_state);
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_fab_actions(
    props: &FabProps,
    actions: &[FabAction],
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
    open_state: Option<&str>,
) {
    let pad = " ".repeat(indent);
    if let Some(open_state) = open_state.filter(|_| !actions.is_empty()) {
        output.push_str(&format!("{pad}if ({open_state}) {{\n"));
    }
    let item_indent = if open_state.is_some() && !actions.is_empty() {
        indent + 4
    } else {
        indent
    };
    let item_pad = " ".repeat(item_indent);
    for action in actions {
        let action_props = VariantProps {
            color: Some(action.color),
            variant: props.style.variant,
            ..VariantProps::default()
        };
        let icon = view_icon(action.icon);
        output.push_str(&format!(
            "{item_pad}Button(onClick = {}, colors = ButtonDefaults.buttonColors(containerColor = {}, contentColor = {}), border = {}, contentPadding = PaddingValues(horizontal = 12.dp, vertical = 8.dp)) {{\n{item_pad}    Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {{\n{item_pad}        Text({})\n",
            compose_component_action(action.on_click.as_deref(), action.navigation.as_ref(), context),
            variant_container(&action_props),
            variant_content(&action_props),
            compose_variant_border(&action_props),
            compose_string_literal(&action.label)
        ));
        render_compose_side_icon(&icon, item_indent + 8, output);
        output.push_str(&format!("{item_pad}    }}\n{item_pad}}}\n"));
    }
    if open_state.is_some() && !actions.is_empty() {
        output.push_str(&format!("{pad}}}\n"));
    }
}

fn render_compose_fab_trigger(
    props: &FabProps,
    actions: &[FabAction],
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
    open_state: Option<&str>,
) {
    let pad = " ".repeat(indent);
    let trigger_action = open_state
        .filter(|_| !actions.is_empty())
        .map(|state| format!("{{ {state} = !{state} }}"))
        .unwrap_or_else(|| {
            compose_component_action(
                props.style.element.on_click.as_deref(),
                props.style.navigation.as_ref(),
                context,
            )
        });
    let icon = view_icon(props.icon);
    let modifier = open_state
        .filter(|_| !actions.is_empty())
        .map(|state| {
            format!(
                "{}.rotate(if ({state}) 45f else 0f)",
                modifier_for_style(&props.style.style)
            )
        })
        .unwrap_or_else(|| modifier_for_style(&props.style.style));
    output.push_str(&format!(
        "{pad}Button(onClick = {trigger_action}, colors = ButtonDefaults.buttonColors(containerColor = {}, contentColor = {}), border = {}, contentPadding = PaddingValues(0.dp), modifier = {}) {{\n",
        variant_container(&props.style),
        variant_content(&props.style),
        compose_variant_border(&props.style),
        modifier
    ));
    render_compose_side_icon(&icon, indent + 4, output);
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_slider(
    props: &SliderProps,
    indent: usize,
    output: &mut String,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    let value = props.value.parse::<f32>().unwrap_or(0.0);
    let min = props.min.parse::<f32>().unwrap_or(0.0);
    let max = props.max.parse::<f32>().unwrap_or(100.0);
    let (value_expr, change_expr, bound) = props
        .style
        .element
        .bind
        .as_deref()
        .map(|path| {
            let path = escape_kotlin(&context.signal_path(path));
            (
                format!("state.text(\"{path}\").toFloatOrNull() ?: {value}f"),
                format!("{{ state.write(\"{path}\", it.toDouble()) }}"),
                "true",
            )
        })
        .unwrap_or_else(|| (format!("{value}f"), "{}".to_string(), "false"));
    output.push_str(&format!(
        "{pad}DoweSliderField(value = {value_expr}, onValueChange = {change_expr}, bound = {bound}, label = {}, hideLabel = {}, min = {min}f, max = {max}f, size = {}, modifier = {}, accentColor = {})\n",
        compose_optional_string(props.style.label.as_deref()),
        props.hide_label,
        compose_string_literal(props.size.as_str()),
        modifier_for_style(&props.style.style),
        compose_scheme_color(&props.style)
    ));
}

fn render_compose_dropzone(props: &DropzoneProps, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweDropzone(label = {}, placeholder = {}, accept = {}, multiple = {}, maxSize = {}, disabled = {}, helpText = {}, errorText = {}, size = {}, modifier = {}, backgroundColor = {}, contentColor = {}, borderColor = {})\n",
        compose_optional_string(props.style.label.as_deref()),
        compose_string_literal(
            props
                .style
                .placeholder
                .as_deref()
                .unwrap_or("Drag & drop files here or click to select")
        ),
        compose_optional_string(props.accept.as_deref()),
        props.multiple,
        props
            .max_size
            .map(|value| format!("{value}L"))
            .unwrap_or_else(|| "null".to_string()),
        props.disabled,
        compose_optional_string(props.help_text.as_deref()),
        compose_optional_string(props.error_text.as_deref()),
        compose_string_literal(props.size.as_str()),
        modifier_for_style(&props.style.style),
        variant_container(&props.style),
        variant_content(&props.style),
        compose_variant_border(&props.style)
    ));
}

