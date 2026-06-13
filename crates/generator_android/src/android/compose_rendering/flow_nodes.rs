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
        ViewNode::Scope {
            signals,
            actions,
            children,
        } => {
            let context = context.with_scope(signals, actions);
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
            let current_font = props.font.as_ref().or(inherited_font);
            if props.cover.is_some() {
                output.push_str(&format!(
                    "{pad}DoweCoverBox(modifier = {}, source = {}, overlay = {}) {{\n",
                    modifier_for_container_style(props, flow),
                    compose_cover_value(props.cover.as_ref().expect("cover")),
                    compose_optional_overlay(props.overlay.as_ref())
                ));
                output.push_str(&format!("{pad}    Column {{\n"));
                let color_scope = compose_content_color(props);
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
                output.push_str(&format!("{pad}}}\n"));
            } else {
                output.push_str(&format!(
                    "{pad}Column(modifier = {}) {{\n",
                    modifier_for_container_style(props, flow)
                ));
                let color_scope = compose_content_color(props);
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
        }
        ViewNode::Section { props, children } => {
            let current_font = props.font.as_ref().or(inherited_font);
            if props.cover.is_some() {
                output.push_str(&format!(
                    "{pad}DoweCoverBox(modifier = {}, source = {}, overlay = {}) {{\n",
                    modifier_for_container_style(props, flow),
                    compose_cover_value(props.cover.as_ref().expect("cover")),
                    compose_optional_overlay(props.overlay.as_ref())
                ));
                output.push_str(&format!("{pad}    Column {{\n"));
                let color_scope = compose_content_color(props);
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
                output.push_str(&format!("{pad}}}\n"));
            } else if let Some(background) = props.background.as_ref() {
                output.push_str(&format!(
                    "{pad}DoweSectionBackgroundBox(modifier = {}, background = {}) {{\n",
                    modifier_for_container_style(props, flow),
                    compose_section_background_value(background)
                ));
                output.push_str(&format!("{pad}    Column {{\n"));
                let color_scope = compose_content_color(props);
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
                output.push_str(&format!("{pad}}}\n"));
            } else {
                output.push_str(&format!(
                    "{pad}Column(modifier = {}) {{\n",
                    modifier_for_container_style(props, flow)
                ));
                let color_scope = compose_content_color(props);
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
        }
        ViewNode::Flex { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
            output.push_str(&format!(
                "{pad}Row(modifier = {}, horizontalArrangement = {}, verticalAlignment = {}) {{\n",
                modifier_for_layout(props, flow),
                compose_horizontal_arrangement(props.justify.as_ref(), props.gap.as_ref()),
                compose_vertical_alignment(props.align.as_ref())
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
                    ComposeFlow::Inline,
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
            output.push_str(&format!(
                        "{pad}Card(modifier = {}, shape = RoundedCornerShape({}), colors = CardDefaults.cardColors(containerColor = {}, contentColor = {}), border = {}) {{\n",
                        modifier_for_container_style(&props.style, flow),
                        compose_card_radius(&props.style),
                        card_variant_container(props),
                        card_variant_content(props),
                        compose_card_border(props)
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
            output.push_str(&format!("{pad}}}\n"));
        }
        ViewNode::Button { props, children } => {
            let current_font = props.style.font.as_ref().or(inherited_font);
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
            output.push_str(&format!(
                        "{pad}Button(modifier = {}.defaultMinSize(minWidth = 0.dp, minHeight = 0.dp), shape = RoundedCornerShape({}), colors = ButtonDefaults.buttonColors(containerColor = {}, contentColor = {}), border = {}, contentPadding = PaddingValues(0.dp), onClick = {}) {{\n",
                        modifier_for_style(&props.style),
                        compose_control_radius(&props.style),
                        variant_container(props),
                        variant_content(props),
                        compose_button_border(props),
                        action
                    ));
            for child in children {
                render_compose_node_in_flow(
                    child,
                    indent + 4,
                    output,
                    ComposeFlow::Inline,
                    current_font,
                    default_family,
                    context,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
        }
        _ => {}
    }
}
