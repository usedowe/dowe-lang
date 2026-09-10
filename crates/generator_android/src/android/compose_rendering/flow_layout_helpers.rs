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
        .center_x
        .as_ref()
        .map(|value| {
            format!(
                ", horizontalAlignment = {}",
                compose_section_horizontal_alignment(value)
            )
        })
        .unwrap_or_default();
    let vertical_arrangement = props
        .center_y
        .as_ref()
        .map(compose_section_vertical_arrangement_centered)
        .unwrap_or_else(|| compose_section_vertical_arrangement(props.gap.as_ref()));
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
        if positioned {
            ComposeFlow::Inline
        } else {
            flow
        },
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
        let horizontal_alignment = props.center_x.as_ref().map(|value| format!(", horizontalAlignment = {}", compose_section_horizontal_alignment(value))).unwrap_or_default();
        output.push_str(&format!("{pad}Column(modifier = {modifier}{horizontal_alignment}) {{\n"));
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

fn compose_section_vertical_arrangement_centered(
    value: &ResponsiveValue<bool>,
) -> String {
    format!(
        ", verticalArrangement = if ({} ?: false) Arrangement.Center else Arrangement.Top",
        compose_bool_value(value)
    )
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
        let horizontal_alignment = props
            .center_x
            .as_ref()
            .map(compose_section_horizontal_alignment)
            .unwrap_or_else(|| "Alignment.Start".to_string());
        output.push_str(&format!(
            "{child_pad}Column(horizontalAlignment = {horizontal_alignment}) {{\n"
        ));
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
