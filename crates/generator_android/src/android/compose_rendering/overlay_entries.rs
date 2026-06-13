fn render_compose_overlay_entry(
    entry: &OverlayEntry,
    indent: usize,
    output: &mut String,
    props: &DropdownProps,
    context: &ComposeReactiveContext,
) {
    match entry {
        OverlayEntry::Item(item) => render_compose_overlay_item(
            item,
            indent,
            output,
            variant_container(&props.style),
            variant_content(&props.style),
            context,
        ),
        OverlayEntry::Divider => {
            let pad = " ".repeat(indent);
            output.push_str(&format!(
                "{pad}Box(modifier = Modifier.fillMaxWidth().height(1.dp).background(DoweDesign.muted))\n"
            ));
        }
    }
}

fn render_compose_command_entry(
    entry: &CommandEntry,
    indent: usize,
    output: &mut String,
    _inherited_font: Option<&ResponsiveValue<FontFamily>>,
    _default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    match entry {
        CommandEntry::Item(item) => render_compose_overlay_item(
            item,
            indent,
            output,
            "Color.Transparent",
            "DoweDesign.onBackground",
            context,
        ),
        CommandEntry::Group { label, icon, items } => {
            output.push_str(&format!(
                "{pad}Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {{\n"
            ));
            output.push_str(&format!(
                "{pad}    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {{\n"
            ));
            if let Some(icon) = icon {
                render_compose_side_icon(icon, indent + 8, output);
            }
            output.push_str(&format!(
                "{pad}        Text(text = {}, color = DoweDesign.onMuted, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)\n",
                compose_string_literal(label)
            ));
            output.push_str(&format!("{pad}    }}\n"));
            for item in items {
                render_compose_overlay_item(
                    item,
                    indent + 4,
                    output,
                    "Color.Transparent",
                    "DoweDesign.onBackground",
                    context,
                );
            }
            output.push_str(&format!("{pad}}}\n"));
        }
    }
}

fn render_compose_overlay_item(
    item: &OverlayItemProps,
    indent: usize,
    output: &mut String,
    background_color: &str,
    content_color: &str,
    context: &ComposeReactiveContext,
) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweOverlayItem(label = {}, description = {}, disabled = {}, backgroundColor = {background_color}, contentColor = {content_color}, onClick = {}) {{\n",
        compose_string_literal(&item.label),
        compose_optional_string(item.description.as_deref()),
        item.disabled,
        compose_optional_component_action(item.on_click.as_deref(), item.navigation.as_ref(), context)
    ));
    if let Some(icon) = item.icon.as_ref() {
        render_compose_side_icon(icon, indent + 4, output);
    }
    output.push_str(&format!("{pad}}}\n"));
}

fn render_compose_optional_region_lambda(
    children: &[ViewNode],
    indent: usize,
    output: &mut String,
    inherited_font: Option<&ResponsiveValue<FontFamily>>,
    default_family: FontFamily,
    context: &ComposeReactiveContext,
) {
    if children.is_empty() {
        output.push_str("null");
        return;
    }
    output.push_str("{\n");
    for child in children {
        render_compose_node_in_flow(
            child,
            indent + 4,
            output,
            ComposeFlow::Block,
            inherited_font,
            default_family,
            context,
        );
    }
    output.push_str(&format!("{}}}", " ".repeat(indent)));
}

fn render_compose_optional_icon_lambda(
    icon: Option<&SideNavIcon>,
    indent: usize,
    output: &mut String,
) {
    if let Some(icon) = icon {
        output.push_str("{\n");
        render_compose_side_icon(icon, indent + 4, output);
        output.push_str(&format!("{}}}", " ".repeat(indent)));
    } else {
        output.push_str("null");
    }
}

fn render_compose_side_icon(icon: &SideNavIcon, indent: usize, output: &mut String) {
    let pad = " ".repeat(indent);
    output.push_str(&format!(
        "{pad}DoweSvg(viewBox = {}, modifier = {}, color = {}, paths = {})\n",
        compose_svg_view_box(&icon.props.view_box),
        modifier_for_style(&icon.props.style),
        compose_svg_color(&icon.props.style),
        compose_svg_paths(&icon.paths)
    ));
}

fn compose_optional_component_action(
    action: Option<&str>,
    navigation: Option<&NavigationAction>,
    context: &ComposeReactiveContext,
) -> String {
    action
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
        .or_else(|| navigation.map(|action| compose_navigation_action(Some(action))))
        .unwrap_or_else(|| "null".to_string())
}

fn compose_component_action(
    action: Option<&str>,
    navigation: Option<&NavigationAction>,
    context: &ComposeReactiveContext,
) -> String {
    let value = compose_optional_component_action(action, navigation, context);
    if value == "null" {
        "{}".to_string()
    } else {
        value
    }
}

fn compose_view_icon_label(icon: ViewIcon) -> &'static str {
    match icon {
        ViewIcon::Plus => "+",
        ViewIcon::Link => "link",
        ViewIcon::Edit => "edit",
        ViewIcon::Trash => "trash",
        ViewIcon::Search => "search",
        ViewIcon::Settings => "settings",
        ViewIcon::Upload => "upload",
        ViewIcon::File => "file",
        ViewIcon::Dismiss => "x",
        ViewIcon::Moon => "moon",
        ViewIcon::Sun => "sun",
    }
}

fn compose_fab_horizontal_alignment(position: OverlayCornerPosition) -> &'static str {
    match position {
        OverlayCornerPosition::TopLeft | OverlayCornerPosition::BottomLeft => "Alignment.Start",
        OverlayCornerPosition::TopRight | OverlayCornerPosition::BottomRight => "Alignment.End",
    }
}

fn compose_fab_vertical_arrangement(position: OverlayCornerPosition) -> &'static str {
    match position {
        OverlayCornerPosition::TopLeft | OverlayCornerPosition::TopRight => "Arrangement.Top",
        OverlayCornerPosition::BottomLeft | OverlayCornerPosition::BottomRight => {
            "Arrangement.Bottom"
        }
    }
}

fn compose_close_action(
    path: &str,
    action: Option<&str>,
    context: &ComposeReactiveContext,
) -> String {
    let after_close = action
        .and_then(|name| context.action_id(name))
        .map(|id| {
            format!(
                "; actionScope.launch {{ state.run(\"{}\") }}",
                escape_kotlin(id)
            )
        })
        .unwrap_or_default();
    format!("{{ state.write(\"{path}\", false){after_close} }}")
}
