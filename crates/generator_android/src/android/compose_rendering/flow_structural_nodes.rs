fn render_compose_structural_flow_node(
    node: &ViewNode,
    indent: usize,
    output: &mut String,
    flow: ComposeFlow,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) -> bool {
    if !matches!(
        node,
        ViewNode::Splash { .. }
            | ViewNode::Scope { .. }
            | ViewNode::Each { .. }
            | ViewNode::Box { .. }
            | ViewNode::Section { .. }
            | ViewNode::Flex { .. }
            | ViewNode::Grid { .. }
            | ViewNode::Card { .. }
            | ViewNode::Brand { .. }
            | ViewNode::Banner { .. }
    ) {
        return false;
    }
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
                        "{pad}DoweGrid(modifier = {}, tracks = {}, horizontalGap = {}, verticalGap = {}, horizontalAlignment = {}, verticalAlignment = {}, horizontalStretch = {}, fillHeight = {}, verticalStretch = {}) {{\n",
                        modifier_for_grid(props, flow),
                        compose_grid_tracks(props.columns.as_ref()),
                        compose_grid_horizontal_gap(props.gap.as_ref()),
                        compose_grid_vertical_gap(props.gap.as_ref()),
                        compose_grid_horizontal_alignment(props.justify.as_ref()),
                        compose_grid_vertical_alignment(props.align.as_ref()),
                        compose_grid_horizontal_stretch(props.justify.as_ref()),
                        compose_grid_fills_height(props),
                        compose_grid_vertical_stretch(props.align.as_ref())
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
                    ComposeFlow::Grid,
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
            let mut card_outer_style = props.style.clone();
            let has_cover = card_outer_style.cover.is_some();
            if has_cover {
                card_outer_style.spacing = Default::default();
            }
            let mut modifier = modifier_for_container_style(&card_outer_style, flow);
            if props.style.element.on_click.is_some() {
                modifier.push_str(&format!(
                    ".clickable(onClick = {})",
                    compose_component_action(
                        props.style.element.on_click.as_deref(),
                        None,
                        context
                    )
                ));
            }
            let reactive_text = |path: &str, fallback: &str| {
                context
                    .item_value(path)
                    .map(|item| format!("state.text(\\\"{}\\\", {item})", escape_kotlin(&context.item_path(path).expect("item path"))))
                    .unwrap_or_else(|| format!("state.text(\\\"{}\\\", \\\"{fallback}\\\")", escape_kotlin(&context.signal_path(path))))
            };
            let variant = props
                .reactive
                .variant
                .as_deref()
                .map(|path| reactive_text(path, "solid"))
                .unwrap_or_else(|| format!("\\\"{}\\\"", props.variant.unwrap_or(ComponentVariant::Solid).as_str()));
            let scheme = props
                .reactive
                .scheme
                .as_deref()
                .map(|path| reactive_text(path, "primary"))
                .unwrap_or_else(|| format!("\\\"{}\\\"", props.color.unwrap_or(ColorFamily::Primary).as_str()));
            let is_reactive = props.reactive.variant.is_some() || props.reactive.scheme.is_some();
            let card_content_color = props
                .style
                .text
                .as_ref()
                .map(compose_color_value)
                .unwrap_or_else(|| if is_reactive { format!("doweCardContent({variant}, {scheme})") } else { card_surface_content(props).to_string() });
            let card_title_color = props
                .style
                .text
                .as_ref()
                .map(compose_color_value)
                .unwrap_or_else(|| if is_reactive { format!("doweCardTitle({variant}, {scheme})") } else { card_surface_title(props).to_string() });
            let card_container = if is_reactive { format!("doweCardContainer({variant}, {scheme})") } else { card_surface_container(props).to_string() };
            let card_border = if is_reactive { format!("if ({variant} == \\\"outlined\\\") BorderStroke(1.dp, {card_content_color}) else null") } else { compose_card_border(props) };
            output.push_str(&format!(
                        "{pad}Card(modifier = {}, shape = RoundedCornerShape({}), colors = CardDefaults.cardColors(containerColor = {}, contentColor = {}), border = {}, elevation = {}) {{\n",
                        modifier,
                        compose_card_radius(&props.style),
                        card_container,
                        card_content_color,
                        card_border,
                        "CardDefaults.cardElevation(defaultElevation = 0.dp)"
                    ));
            output.push_str(&format!(
                "{pad}    CompositionLocalProvider(LocalDoweTitleColor provides {card_title_color}) {{\n"
            ));
            if props.style.cover.is_some() {
                output.push_str(&format!(
                            "{pad}    DoweCoverBox(modifier = Modifier.fillMaxWidth().fillMaxHeight(), source = {}, overlay = {}) {{\n",
                            compose_cover_value(props.style.cover.as_ref().expect("cover")),
                            compose_optional_overlay(props.style.overlay.as_ref())
                        ));
                let content_padding = compose_content_padding(&props.style.spacing);
                output.push_str(&format!("{pad}        Column(modifier = Modifier.fillMaxWidth().fillMaxHeight().padding({content_padding})) {{\n"));
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
        _ => {}
    }
    true
}
